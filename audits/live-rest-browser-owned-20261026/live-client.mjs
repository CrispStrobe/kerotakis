// Isolated browser acceptance. Fixtures test UI behavior, never chemistry.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { chromium } from '@playwright/test';

const workspace = process.env.GITHUB_WORKSPACE;
assert(workspace);
const bindingPath = process.env.KERO_BROWSER_RELEASE_BINDING;
assert(bindingPath, 'KERO_BROWSER_RELEASE_BINDING is required');
const spec = JSON.parse(await readFile(bindingPath, 'utf8'));
assert.match(spec.expected_engine_source, /^[a-f0-9]{40}$/);
assert.match(spec.expected_binary_sha256, /^[a-f0-9]{64}$/);
const source = await readFile(path.join(workspace, 'web/rest.html'));
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
assert.equal(sha(source), spec.client_sha256);
const bridgeScript = 'add v1 water 137mL\nadd v1 NaCl 0.037mol\n';
const nativeReference = spec.native_bridge_reference;
assert(nativeReference, 'Bind the real authenticated native reference before dispatch');
assert(Number.isFinite(nativeReference.solution.ph) && Number.isFinite(nativeReference.solution.ionic_strength));
assert.equal(nativeReference.script_sha256, sha(bridgeScript));
assert.equal(nativeReference.engine.source_head, spec.expected_engine_source);
assert.equal(nativeReference.engine.binary_sha256, spec.expected_binary_sha256);
const assetNames = ['iphreeqc.mjs', 'iphreeqc.wasm', 'kerotakis.mjs',
  'kerotakis_wasm.js', 'kerotakis_wasm_bg.wasm'];
assert.deepEqual(Object.keys(spec.frontend_assets).sort(), assetNames.toSorted());
for (const digest of Object.values(spec.frontend_assets)) assert.match(digest, /^[a-f0-9]{64}$/);
const api = 'https://kerotakis-web-payload.vercel.app/api';
const origins = [
  ['production', 'https://kerotakis-web-payload.vercel.app/rest.html'],
  ['pages', 'https://crispstrobe.github.io/kerotakis/rest.html'],
];
const output = process.env.BROWSER_REPORT_DIR;
assert(output);
await mkdir(output, { recursive: true });
const report = { checks: [], browser: null, source_binding: spec,
  physical_verdict: 'not_assigned', fixtures_are_ui_only: true,
  owner_credentials_supplied: false, new_corpus_executions: 0 };
const fakeKey = 'invalid-key-for-browser-ui-fixture-only';
const fixtureScript = '# browser UI fixture, not a chemical protocol\ninspect v1\n';
const marker = '<img src=x onerror="window.fixtureExecuted=true">';
function check(name, condition, observed) {
  report.checks.push({ name, passed: Boolean(condition), observed });
  assert(condition, name);
}
async function get(url, options = {}) {
  return fetch(url, { ...options, redirect: 'error', signal: AbortSignal.timeout(15000) });
}
async function submit(page) {
  await page.locator('#submit').click();
  await page.waitForFunction(() => !document.querySelector('#submit').disabled &&
    document.querySelector('#result').textContent !== 'Running…');
  return page.locator('#result').innerText();
}
let browser;
try {
  const response = await get(api + '/health');
  const health = await response.json();
  check('live engine identity', response.status === 200 && health.configured === true &&
    health.revision === spec.expected_engine_source &&
    health.engine?.source_head === spec.expected_engine_source &&
    health.engine?.binary_sha256 === spec.expected_binary_sha256,
    { status: response.status, health });
  for (const [name, url] of origins) {
    const served = await get(url);
    const bytes = Buffer.from(await served.arrayBuffer());
    check(name + ' actual client bytes', served.status === 200 && sha(bytes) === spec.client_sha256,
      { status: served.status, sha256: sha(bytes) });
  }
  for (const name of assetNames) {
    const served = await get('https://kerotakis-web-payload.vercel.app/' + name);
    const bytes = Buffer.from(await served.arrayBuffer());
    check('deployed asset ' + name, served.status === 200 && sha(bytes) === spec.frontend_assets[name],
      { status: served.status, sha256: sha(bytes), length: bytes.length });
  }
  browser = await chromium.launch({ headless: true });
  report.browser = { version: browser.version(), web_security_disabled: false };
  // Exercise the actual deployed Rust/Emscripten bridge with no result cache.
  // The numerical reference comes from a separate real authenticated API call.
  const liveContext = await browser.newContext();
  const livePage = await liveContext.newPage();
  await livePage.goto(origins[0][1], { waitUntil: 'networkidle' });
  const live = await livePage.evaluate(async script => {
    const module = await import('/kerotakis_wasm.js');
    await module.default();
    const { PhreeqcPool } = await import('/kerotakis.mjs');
    const factory = (await import('/iphreeqc.mjs')).default;
    const lab = new module.Lab();
    try {
      const meta = JSON.parse(lab.meta());
      const bareCanSolve = lab.canSolve();
      const pool = await PhreeqcPool.create(factory, async file => {
        const response = await fetch('/db/' + file, { redirect: 'error' });
        if (!response.ok) throw new Error('Database load failed: ' + response.status);
        return response.text();
      }, tag => lab.aqueousDatabase(tag));
      let calls = 0;
      lab.setSolver((tag, input) => { calls++; return pool.solve(tag, input); });
      const canSolve = lab.canSolve();
      const result = JSON.parse(lab.runScript(script));
      // Run only after capturing the salt result, so this capability probe
      // cannot leave USER_PUNCH state in the numerical parity question.
      let basic;
      try {
        basic = { response: JSON.parse(pool.solve('wateq4f',
          'SOLUTION 777\n pH 7\nSELECTED_OUTPUT\n -reset false\nUSER_PUNCH\n10 PUNCH 2+3\nEND\n')) };
      } catch (error) { basic = { error: String(error) }; }
      return { meta, bareCanSolve, canSolve, solver_hook_calls: calls, result, basic };
    } finally { lab.free(); }
  }, bridgeScript);
  report.live_browser_control = live;
  check('actual Wasm source revision', typeof live.meta.git_rev === 'string' &&
    live.meta.git_rev.length >= 7 && spec.expected_engine_source.startsWith(live.meta.git_rev), live.meta);
  const vessel = live.result.bench.vessels[0];
  check('uncached browser solve uses live engine', !live.bareCanSolve && live.canSolve && live.solver_hook_calls > 0 &&
    vessel.solution != null, { bareCanSolve: live.bareCanSolve, canSolve: live.canSolve, solver_hook_calls: live.solver_hook_calls,
      solution: vessel.solution });
  const deltaPh = Math.abs(vessel.solution.ph - nativeReference.solution.ph);
  const deltaI = Math.abs(vessel.solution.ionic_strength - nativeReference.solution.ionic_strength);
  check('native and actual browser solution agree', deltaPh < 1e-6 && deltaI < 1e-9,
    { delta_ph: deltaPh, delta_ionic_strength: deltaI,
      native_response_sha256: nativeReference.response_sha256 });
  const amount = species => vessel.contents.filter(p => p.species === species)
    .reduce((sum, p) => sum + p.moles, 0);
  check('browser finite salt retained', Math.abs(amount('Na+') - 0.037) <= 1e-8 &&
    Math.abs(amount('Cl-') - 0.037) <= 1e-8,
    { sodium_mol: amount('Na+'), chloride_mol: amount('Cl-') });
  const punched = (live.basic.response?.selected ?? []).flat()
    .some(value => String(value).trim() !== '' && Number(value) === 5);
  check('actual browser interpreter executes arithmetic', !live.basic.error && punched, live.basic);
  await liveContext.close();
  // Diagnostic only: independently observe the Pages build profile. This is
  // separate from production acceptance and makes no Pages chemistry claim.
  report.pages_asset_observations = {};
  for (const name of assetNames) {
    const url = new URL(name, origins[1][1]).href;
    const served = await get(url);
    const bytes = Buffer.from(await served.arrayBuffer());
    report.pages_asset_observations[name] = { url, status: served.status,
      sha256: sha(bytes), length: bytes.length, scope: 'measurement only, not production asset identity' };
  }
  const pagesContext = await browser.newContext();
  const pagesPage = await pagesContext.newPage();
  await pagesPage.goto(origins[1][1], { waitUntil: 'networkidle' });
  report.pages_interpreter_diagnostic = await pagesPage.evaluate(async () => {
    const base = new URL('.', location.href);
    const observed = { scope: 'Actual published interpreter arithmetic only; not chemical validation',
      mode: 'measurement_only_no_pass_assigned' };
    try {
      const module = await import(new URL('kerotakis_wasm.js', base).href);
      await module.default();
      const { PhreeqcPool } = await import(new URL('kerotakis.mjs', base).href);
      const factory = (await import(new URL('iphreeqc.mjs', base).href)).default;
      const lab = new module.Lab();
      try {
        observed.meta = JSON.parse(lab.meta());
        const pool = await PhreeqcPool.create(factory, async file => {
          const r = await fetch(new URL('db/' + file, base), { redirect: 'error' });
          if (!r.ok) throw new Error('Database load failed: ' + r.status);
          return r.text();
        }, tag => lab.aqueousDatabase(tag));
        observed.basic_response = JSON.parse(pool.solve('wateq4f',
          'SOLUTION 777\n pH 7\nSELECTED_OUTPUT\n -reset false\nUSER_PUNCH\n10 PUNCH 2+3\nEND\n'));
        observed.arithmetic_result_5 = (observed.basic_response.selected ?? []).flat()
          .some(v => String(v).trim() !== '' && Number(v) === 5);
      } finally { lab.free(); }
    } catch (error) { observed.error = String(error); }
    return observed;
  });
  await pagesContext.close();
  for (const [name, url] of origins) {
    const context = await browser.newContext({ serviceWorkers: 'allow' });
    const page = await context.newPage();
    const requests = [], errors = [];
    page.on('request', req => requests.push({ url: req.url(), method: req.method() }));
    page.on('pageerror', error => errors.push(error.message));
    await page.goto(url, { waitUntil: 'networkidle' });
    const initialScript = await page.locator('#script').inputValue();
    await page.locator('#endpoint').fill(api);
    await page.locator('#key').fill(fakeKey);
    await page.locator('#script').fill(fixtureScript);

    // A real invalid-key request observes live browser CORS without an owner key.
    const denied = await submit(page);
    check(name + ' live 401 visible', denied.startsWith('HTTP 401'), { result: denied });

    let captured;
    await page.route(api + '/run', async route => {
      const request = route.request();
      if (request.method() === 'OPTIONS') return route.continue();
      captured = { method: request.method(), payload: request.postDataJSON(),
        fixture_auth_matches: request.headers().authorization === `Bearer ${fakeKey}` };
      await route.fulfill({ status: 200, contentType: 'application/json',
        headers: { 'access-control-allow-origin': new URL(url).origin },
        body: JSON.stringify({ ui_fixture: true, message: marker }) });
    });
    const rendered = await submit(page);
    check(name + ' UI request wiring', captured?.method === 'POST' &&
      captured.payload.script === fixtureScript && captured.fixture_auth_matches, captured);
    check(name + ' fixture rendered as text', rendered.startsWith('HTTP 200') &&
      rendered.includes('ui_fixture') && rendered.includes('<img') &&
      await page.locator('#result img').count() === 0 &&
      await page.evaluate(() => window.fixtureExecuted !== true), { result: rendered });

    const postCount = () => requests.filter(r => r.method === 'POST').length;
    const before = postCount();
    const invalid = [api.replace('https:', 'http:'), api + '/extra', api + '?q=1',
      api + '#fragment', 'https://user:password@kerotakis-web-payload.vercel.app/api'];
    for (const endpoint of invalid) {
      await page.locator('#endpoint').fill(endpoint);
      const result = await submit(page);
      assert(result.includes('Use an HTTPS API URL'), endpoint);
    }
    check(name + ' invalid endpoints refused before fetch', postCount() === before,
      { variants: invalid.length, new_post_requests: postCount() - before });
    await page.locator('#endpoint').fill(api);

    await page.locator('#clear').click();
    const clear = { key_empty: await page.locator('#key').inputValue() === '',
      results_empty: await page.locator('#result').innerText() === '',
      script_retained: await page.locator('#script').inputValue() === fixtureScript };
    check(name + ' clear semantics', clear.key_empty && clear.results_empty && clear.script_retained, clear);
    await page.locator('#key').fill(fakeKey);
    await page.unroute(api + '/run');

    let redirectRequests = 0;
    page.on('request', request => { if (request.url().includes('example.invalid')) redirectRequests++; });
    await page.route(api + '/run', route => route.request().method() === 'OPTIONS'
      ? route.continue() : route.fulfill({ status: 302,
        headers: { location: 'https://example.invalid/do-not-follow',
          'access-control-allow-origin': new URL(url).origin } }));
    const redirect = await submit(page);
    check(name + ' redirects refused', !redirect.startsWith('HTTP ') && redirectRequests === 0,
      { result: redirect, redirected_requests: redirectRequests });
    await page.unroute(api + '/run');

    await context.setOffline(true);
    const offline = await submit(page);
    check(name + ' offline failure clears stale result', offline.length > 0 &&
      !offline.startsWith('HTTP 200') && !offline.includes(marker), { result: offline });
    await context.setOffline(false);
    await page.reload({ waitUntil: 'networkidle' });
    const reload = { key_empty: await page.locator('#key').inputValue() === '',
      initial_script_restored: await page.locator('#script').inputValue() === initialScript,
      results_empty: await page.locator('#result').innerText() === '' };
    check(name + ' reload clears inputs', reload.key_empty && reload.initial_script_restored && reload.results_empty, reload);
    const stores = await page.evaluate(async () => ({
      local: Object.keys(localStorage), session: Object.keys(sessionStorage),
      indexedDB: (await indexedDB.databases()).map(x => x.name), caches: await caches.keys(),
      serviceWorkers: (await navigator.serviceWorker.getRegistrations()).length,
    }));
    const cookies = await context.cookies();
    check(name + ' no persisted client data', stores.local.length === 0 && stores.session.length === 0 &&
      stores.indexedDB.length === 0 && stores.caches.length === 0 &&
      stores.serviceWorkers === 0 && cookies.length === 0, { stores, cookie_count: cookies.length });
    check(name + ' no key in request URLs', requests.every(r => !r.url.includes(fakeKey)), { request_count: requests.length });
    check(name + ' no unhandled page errors', errors.length === 0, { errors });
    await page.screenshot({ path: path.join(output, name + '.png'), fullPage: true });
    await context.close();
  }
  const unapproved = 'https://unapproved.invalid';
  const preflight = await get(api + '/run', { method: 'OPTIONS', headers: {
    Origin: unapproved, 'Access-Control-Request-Method': 'POST',
    'Access-Control-Request-Headers': 'authorization,content-type',
  } });
  check('unapproved preflight denied', preflight.status === 400 &&
    preflight.headers.get('access-control-allow-origin') !== unapproved, { status: preflight.status });
  const context = await browser.newContext();
  const page = await context.newPage();
  // Controlled origin only: never described as a published client.
  await page.route(unapproved + '/rest.html', route => route.fulfill({
    status: 200, contentType: 'text/html', body: source }));
  await page.goto(unapproved + '/rest.html');
  await page.locator('#endpoint').fill(api);
  await page.locator('#key').fill(fakeKey);
  await page.locator('#script').fill(fixtureScript);
  const blocked = await submit(page);
  check('actual browser blocks unapproved origin', blocked.length > 0 && !blocked.startsWith('HTTP '),
    { result: blocked, controlled_document_origin: unapproved });
  await context.close();
  report.all_checks_passed = true;
} catch (error) {
  report.all_checks_passed = false;
  report.failure = error.stack;
  process.exitCode = 1;
} finally {
  if (browser) await browser.close();
  await writeFile(path.join(output, 'report.json'), JSON.stringify(report, null, 2) + '\n');
  console.log(JSON.stringify({ all_checks_passed: report.all_checks_passed,
    checks_executed: report.checks.length, failure: report.failure ?? null }));
}
