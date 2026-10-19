// Bounded Pages acceptance: real served bytes and a fresh, uncached chemistry solve.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { chromium } from '@playwright/test';

const spec = JSON.parse(await readFile(process.env.PAGES_SPEC, 'utf8'));
const output = process.env.BROWSER_REPORT_DIR;
assert(output);
const sha = value => createHash('sha256').update(value).digest('hex');
const script = 'add v1 water 137mL\nadd v1 NaCl 0.037mol\n';
assert.equal(sha(script), spec.native_reference.script_sha256);
assert.equal(spec.pages_origin, 'https://crispstrobe.github.io/kerotakis/');
assert.match(spec.pages_source, /^[a-f0-9]{40}$/);
assert.equal(spec.native_reference.engine.source_head, spec.native_runtime_source);
const report = { checks: [], owner_credentials_supplied: false,
  new_rest_requests: 0, new_corpus_executions: 0, spec };
await mkdir(output, { recursive: true });
let browser;
function check(name, condition, observed) {
  report.checks.push({ name, passed: Boolean(condition), observed });
  assert(condition, name);
}
try {
  report.served_assets = {};
  for (const [name, expected] of Object.entries(spec.artifact_assets)) {
    const url = new URL(name, spec.pages_origin).href;
    const r = await fetch(url, { redirect: 'error', cache: 'no-store',
      signal: AbortSignal.timeout(20000) });
    const raw = Buffer.from(await r.arrayBuffer());
    const observed = { url, status: r.status, sha256: sha(raw), bytes: raw.length };
    report.served_assets[name] = observed;
    check('actual Pages artifact bytes: ' + name,
      r.status === 200 && observed.sha256 === expected.sha256 && raw.length === expected.bytes,
      observed);
  }
  browser = await chromium.launch({ headless: true, args: ['--no-sandbox'] });
  const context = await browser.newContext({ serviceWorkers: 'block' });
  const page = await context.newPage();
  await page.goto(new URL('rest.html', spec.pages_origin).href, { waitUntil: 'networkidle' });
  report.actual = await page.evaluate(async script => {
    const base = new URL('.', location.href);
    const module = await import(new URL('kerotakis_wasm.js', base).href);
    await module.default();
    const { PhreeqcPool } = await import(new URL('kerotakis.mjs', base).href);
    const factory = (await import(new URL('iphreeqc.mjs', base).href)).default;
    const lab = new module.Lab();
    try {
      const meta = JSON.parse(lab.meta());
      const bareCanSolve = lab.canSolve();
      const databaseLoads = [];
      const pool = await PhreeqcPool.create(factory, async file => {
        const r = await fetch(new URL('db/' + file, base), { redirect: 'error' });
        databaseLoads.push({ file, status: r.status });
        if (!r.ok) throw new Error('Database load failed: ' + r.status);
        return r.text();
      }, tag => lab.aqueousDatabase(tag));
      let calls = 0;
      lab.setSolver((tag, input) => { calls++; return pool.solve(tag, input); });
      const canSolve = lab.canSolve();
      // Capture chemistry before the separate USER_PUNCH interpreter probe.
      const result = JSON.parse(lab.runScript(script));
      const basic = JSON.parse(pool.solve('wateq4f',
        'SOLUTION 777\n pH 7\nSELECTED_OUTPUT\n -reset false\nUSER_PUNCH\n10 PUNCH 2+3\nEND\n'));
      return { meta, bareCanSolve, canSolve, solver_hook_calls: calls,
        result, basic, databaseLoads };
    } finally { lab.free(); }
  }, script);
  const live = report.actual;
  check('actual Pages source revision', typeof live.meta.git_rev === 'string' &&
    live.meta.git_rev.length >= 7 && spec.pages_source.startsWith(live.meta.git_rev), live.meta);
  check('both original commands execute without solver refusal',
    live.result.steps?.length === 2 &&
    live.result.steps[0].canonical === 'add v1 water 137mL' &&
    live.result.steps[1].canonical === 'add v1 NaCl 0.037mol' &&
    live.result.steps.every(s => s.events.every(e =>
      !['solver_failed', 'not_yet_modeled'].includes(e.event))), live.result.steps);
  const vessel = live.result.bench.vessels[0];
  check('fresh Pages Lab actually calls native bridge', !live.bareCanSolve && live.canSolve &&
    live.solver_hook_calls > 0 && vessel.solution != null,
    { bareCanSolve: live.bareCanSolve, canSolve: live.canSolve, calls: live.solver_hook_calls });
  const deltaPh = Math.abs(vessel.solution.ph - spec.native_reference.solution.ph);
  const deltaI = Math.abs(vessel.solution.ionic_strength - spec.native_reference.solution.ionic_strength);
  check('Pages chemistry agrees with bound native reference', deltaPh < 1e-6 && deltaI < 1e-9,
    { delta_ph: deltaPh, delta_ionic_strength: deltaI,
      native_response_sha256: spec.native_reference.response_sha256 });
  const amount = name => vessel.contents.filter(p => p.species === name)
    .reduce((sum, p) => sum + p.moles, 0);
  check('actual Pages finite salt inventory', Math.abs(amount('Na+') - 0.037) <= 1e-8 &&
    Math.abs(amount('Cl-') - 0.037) <= 1e-8,
    { sodium_mol: amount('Na+'), chloride_mol: amount('Cl-') });
  check('actual Pages interpreter executes arithmetic', (live.basic.selected ?? []).flat()
    .some(v => String(v).trim() !== '' && Number(v) === 5), live.basic);
  report.all_checks_passed = report.checks.length === 11 && report.checks.every(c => c.passed);
  assert(report.all_checks_passed);
  await context.close();
} catch (error) {
  report.failure = { name: error.name, message: String(error.message) };
  report.all_checks_passed = false;
  process.exitCode = 1;
} finally {
  if (browser) await browser.close();
  await writeFile(output + '/report.json', JSON.stringify(report, null, 2) + '\n');
}
