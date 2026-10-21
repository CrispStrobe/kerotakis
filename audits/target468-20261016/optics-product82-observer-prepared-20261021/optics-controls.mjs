// SOURCE PREPARATION ONLY. Importing launches no browser and performs no requests.
// This observes the real app worker; it never substitutes a response or Lab state.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
export const state = 'PREPARED_NOT_EXECUTED_REQUIRES_PARENT_REVIEW_AND_ACTUAL_APP_ASSET_BINDING';

// Installed before navigation. All messages retain real IDs/content and ordering.
// Observer-issued operations are existing read-only snapshot and scene commands.
export function installWorkerObserver() {
  const NativeWorker = window.Worker;
  const nativePost = NativeWorker.prototype.postMessage;
  const entries = [], workers = new Map(); let seq = 0, diagnosticId = -1;
  const copy = value => structuredClone(value);
  window.Worker = class extends NativeWorker {
    constructor(url, options) {
      super(url, options);
      const worker = this, index = workers.size;
      workers.set(index, worker);
      entries.push({sequence: ++seq, direction: 'created', worker: index, url: String(url), options: copy(options)});
      this.addEventListener('message', event => entries.push({sequence: ++seq, direction: 'response', worker: index, value: copy(event.data)}));
      this.postMessage = function (...args) {
        entries.push({sequence: ++seq, direction: 'request', worker: index, origin: 'app', value: copy(args[0])});
        return Reflect.apply(nativePost, worker, args);
      };
    }
  };
  Object.defineProperty(window, '__keroOpticsObserver', {value: {
    entries,
    readOnly(workerIndex, cmd) {
      if (!['snapshot','scene'].includes(cmd)) throw Error('observer operation is not read-only');
      const worker = workers.get(workerIndex); if (!worker) throw Error('actual worker absent');
      const request = {id: diagnosticId--, cmd};
      entries.push({sequence: ++seq, direction: 'request', worker: workerIndex, origin: 'observer-read-only-' + cmd, value: copy(request)});
      Reflect.apply(nativePost, worker, [request]); return request.id;
    },
  }, writable: false});
}
const pairsFrom = entries => entries.filter(e => e.direction === 'request').map(e => {
  const r = entries.find(x => x.direction === 'response' && x.worker === e.worker && x.value.id === e.value.id && ['done','error','cancelled'].includes(x.value.type));
  return r && {request: e.value, response: r.value, request_sequence: e.sequence, response_sequence: r.sequence, worker: e.worker, origin: e.origin};
}).filter(Boolean);
const trace = page => page.evaluate(() => structuredClone(window.__keroOpticsObserver.entries));
async function waitPair(page, cmd, after, extra = {}) {
  await page.waitForFunction(({cmd, after, extra}) => {
    const es = window.__keroOpticsObserver.entries;
    return es.some(q => q.direction === 'request' && q.sequence > after && q.value.cmd === cmd && Object.entries(extra).every(([k,v]) => q.value[k] === v) && es.some(r => r.direction === 'response' && r.worker === q.worker && r.value.id === q.value.id && ['done','error','cancelled'].includes(r.value.type)));
  }, {cmd, after, extra}, {timeout: 45000});
  const pair = pairsFrom(await trace(page)).find(p => p.request_sequence > after && p.request.cmd === cmd && Object.entries(extra).every(([k,v]) => p.request[k] === v));
  assert(pair && pair.response.type === 'done', 'real worker refused ' + cmd); return pair;
}
async function readOnly(page, worker, cmd) {
  const id = await page.evaluate(({worker, cmd}) => window.__keroOpticsObserver.readOnly(worker, cmd), {worker, cmd});
  return waitPair(page, cmd, 0, {id});
}
async function ensureUtilities(page) {
  const toggle = page.locator('button.utility-toggle');
  await toggle.waitFor({state: 'visible'});
  if (await toggle.getAttribute('aria-expanded') !== 'true') await toggle.click();
  await page.waitForFunction(() => document.querySelector('button.utility-toggle')?.getAttribute('aria-expanded') === 'true', null, {timeout: 15000});
  await page.locator('section.utility-drawer').waitFor({state: 'visible'});
}
async function settled(page) {
  await page.waitForFunction(() => document.querySelector('input[role="combobox"]')?.disabled === false, null, {timeout: 45000});
}
async function runIndependentLab(page, protocol, locale, source) {
  return page.evaluate(async ({protocol, locale, source}) => {
    const wasm = await import('/kerotakis_wasm.js'); await wasm.default();
    const {PhreeqcPool} = await import('/kerotakis.mjs');
    const factory = (await import('/iphreeqc.mjs')).default;
    const lab = new wasm.Lab();
    try {
      lab.setLocale(locale); const meta = JSON.parse(lab.meta());
      if (!(typeof meta.git_rev === 'string' && meta.git_rev.length >= 7 && source.startsWith(meta.git_rev))) throw Error('independent actual Lab source differs');
      const pool = await PhreeqcPool.create(factory, async file => {
        const response = await fetch('/db/' + file, {redirect: 'error'});
        if (!response.ok) throw Error('database response ' + response.status); return response.text();
      }, tag => lab.aqueousDatabase(tag));
      let calls = 0; lab.setSolver((tag, input) => {calls++; return pool.solve(tag, input);});
      const setup_result = JSON.parse(lab.runScript(protocol.setup));
      const snapshot_before = JSON.parse(lab.snapshot()); const calls_before_measure = calls;
      const continuation_result = JSON.parse(lab.runScript(protocol.continuation));
      return {meta, setup_script: protocol.setup, continuation_script: protocol.continuation,
        setup_result, continuation_result, snapshot_before, snapshot_after: JSON.parse(lab.snapshot()),
        scene_after: JSON.parse(lab.scene()), calls_before_measure, calls_after_measure: calls,
        instance_role: 'separate-real-Lab-not-the-GUI-worker'};
    } finally {lab.free();}
  }, {protocol, locale, source});
}
async function readDom(page) {
  return page.locator('[data-vessel-id="0"]').evaluate(vessel => {
    const rect = node => Object.fromEntries(['x','y','width','height'].map(k => [k, Number(node.getAttribute(k))]));
    return {vessel_id: vessel.getAttribute('data-vessel-id'), locale_value: document.querySelector('.locale select')?.value,
      layers: [...vessel.querySelectorAll('.scene-liquid-layer')].map(n => ({...rect(n), fill: n.getAttribute('fill'), data_srgb: n.getAttribute('data-srgb')})),
      hatches: [...vessel.querySelectorAll('.spectral-gap-hatch')].map(n => ({...rect(n), data_ph_gaps: n.parentElement.getAttribute('data-ph-gaps'), data_gaps: n.parentElement.getAttribute('data-gaps'), title: n.querySelector('title')?.textContent ?? n.parentElement.querySelector('title')?.textContent})),
      ph_readouts: [...vessel.querySelectorAll('.optical-ph-gap-readout')].map(n => ({small: n.querySelector('small')?.textContent, strong: n.querySelector('strong')?.textContent, aria_label: n.getAttribute('aria-label'), visible_text: n.textContent})),
      legacy_gap_readouts: [...vessel.querySelectorAll('.spectral-gap-readout')].map(n => ({text: n.textContent})),
      vessel_bounds: vessel.getBoundingClientRect().toJSON()};
  });
}
// Caller must authenticate this mapping against actual verified frontend archive;
// hashing self-supplied expected values is NOT release acceptance.
async function verifyAssets(origin, binding) {
  assert(binding.app_assets && Object.keys(binding.app_assets).includes('app/index.html'), 'actual app archive binding is mandatory');
  const required = {...binding.frontend_assets, ...binding.app_assets}; const observed = {};
  for (const [name, expected] of Object.entries(required)) {
    assert(/^[a-zA-Z0-9_./-]+$/.test(name) && !name.split('/').includes('..') && !name.startsWith('/'));
    assert.match(expected, /^[a-f0-9]{64}$/);
    const response = await fetch(new URL('/' + name, origin), {redirect: 'error', cache: 'no-store', signal: AbortSignal.timeout(20000)});
    assert(response.ok); let count = 0; const h = createHash('sha256');
    for await (const chunk of response.body) {count += chunk.length; assert(count <= 16 * 1024 * 1024, 'per-asset bound'); h.update(chunk);}
    const sha256 = h.digest('hex'); assert.equal(sha256, expected, 'served asset ' + name); observed[name] = {sha256, bytes: count, status: response.status};
  }
  return observed;
}
export async function observeOptics18(browser, binding, protocols, retainPartial) {
  assert.match(binding.expected_engine_source, /^[a-f0-9]{40}$/);
  assert.equal(new URL(binding.origin).protocol, 'https:');
  assert.equal(protocols.cases.length, 6); assert.equal(protocols.additional_locale_case_checks, 18);
  assert.equal(typeof retainPartial, 'function', 'caller must persist genuine partial rows/traffic on first failure');
  const report = {state: 'OBSERVATIONS_ONLY_REQUIRES_INDEPENDENT_SAVED_REVIEW', binding, rows: [], scientific_selections: 0};
  report.assets = await verifyAssets(binding.origin, binding);
  for (const protocol of protocols.cases) for (const locale of protocol.locales) {
    const context = await browser.newContext({serviceWorkers: 'block'});
    const row = {case_id: protocol.id, locale}; report.rows.push(row);
    const actualAppResponses = [], captures = [];
    // Actual loaded bodies are independent evidence from corroborating GETs.
    // BrowserContext response events must cover each created dedicated worker.
    context.on('response', response => {
      const url = new URL(response.url());
      if (url.origin !== new URL(binding.origin).origin || !url.pathname.startsWith('/app/')) return;
      const record = {url: response.url(), status: response.status(), state: 'CAPTURE_PENDING'};
      actualAppResponses.push(record);
      captures.push((async () => {
        const path = url.pathname === '/app/' ? 'app/index.html' : url.pathname.slice(1);
        assert(Object.hasOwn(binding.app_assets, path), 'unbound actual loaded app path ' + path);
        assert(response.status() >= 200 && response.status() < 300, 'actual loaded app response failed');
        const body = await response.body();
        assert(body.length <= 16 * 1024 * 1024, 'actual loaded asset body exceeds bound');
        record.archive_path = path; record.bytes = body.length;
        record.sha256 = createHash('sha256').update(body).digest('hex');
        assert.equal(record.sha256, binding.app_assets[path], 'actual loaded app bytes differ ' + path);
        record.state = 'ACTUAL_LOADED_BYTES_MATCH';
      })().catch(error => {record.state = 'CAPTURE_FAILED'; record.error = String(error);}));
    });
    let gui;
    try {
      // Lab and GUI are genuinely independent, both inside this fresh context.
      const direct = await context.newPage(); await direct.goto(new URL('/rest.html', binding.origin).href);
      row.lab = await runIndependentLab(direct, protocol, locale, binding.expected_engine_source); await direct.close();
      await context.addInitScript(installWorkerObserver); gui = await context.newPage();
      await gui.goto(new URL('/app/', binding.origin).href);
      const hello = await waitPair(gui, 'hello', 0); const meta = JSON.parse(hello.response.result_json);
      assert(meta.engine_loaded && meta.can_solve && meta.aqueous_note === null && !meta.load_failure);
      assert(typeof meta.git_rev === 'string' && meta.git_rev.length >= 7 && binding.expected_engine_source.startsWith(meta.git_rev));
      const initial = await trace(gui); assert(!initial.some(e => e.direction === 'request' && ['run_script','restore'].includes(e.value.cmd)), 'fresh GUI startup already mutated/restored chemistry');
      // Fresh app utilities start closed. Open genuine UI before settings/console.
      // In this source LocaleSwitcher is in the header; console/file import are
      // drawer-owned. Recheck before every drawer-owned operation.
      await ensureUtilities(gui);
      // Force a real change if initial selected locale equals target; no chemistry.
      const select = gui.locator('.locale select');
      if (await select.inputValue() === locale) {const seq = (await trace(gui)).at(-1).sequence; await select.selectOption(locale === 'en' ? 'de' : 'en'); await waitPair(gui, 'set_locale', seq);}
      const localeSeq = (await trace(gui)).at(-1).sequence; await select.selectOption(locale);
      const locale_request = await waitPair(gui, 'set_locale', localeSeq, {code: locale});
      await ensureUtilities(gui);
      const toggle = gui.locator('button.console-toggle');
      if (await toggle.getAttribute('aria-pressed') !== 'true') await toggle.click();
      await gui.locator('input[role="combobox"]').waitFor(); await settled(gui);
      const start = (await trace(gui)).at(-1).sequence;
      const setupLines = protocol.setup.trim().split('\n');
      // The real existing .lab file input calls importLab, which intercepts register.
      await ensureUtilities(gui);
      await gui.locator('input[type="file"][accept=".lab,text/plain"]').setInputFiles({name: 'optics-setup.lab', mimeType: 'text/plain', buffer: Buffer.from(protocol.setup)});
      const lastSetup = setupLines.at(-1); const endSetup = lastSetup === 'register lv3'
        ? await waitPair(gui, 'set_register', start, {level: 'lv3'})
        : await waitPair(gui, 'run_script', start, {script: lastSetup});
      await settled(gui);
      const setup_snapshot_pair = await readOnly(gui, endSetup.worker, 'snapshot');
      const contStart = (await trace(gui)).at(-1).sequence;
      await ensureUtilities(gui);
      await gui.locator('input[type="file"][accept=".lab,text/plain"]').setInputFiles({name: 'optics-observe.lab', mimeType: 'text/plain', buffer: Buffer.from(protocol.continuation)});
      const measure = await waitPair(gui, 'run_script', contStart, {script: 'measure v1 uvvis'});
      await settled(gui);
      const scene_pair = await readOnly(gui, measure.worker, 'scene');
      await gui.locator('[data-vessel-id="0"]').waitFor();
      // Await actual network settlement and all context-body captures before rows.
      // Missing worker coverage is HOLD, never inferred from a separate GET.
      await gui.waitForLoadState('networkidle', {timeout: 45000});
      let captured = 0;
      do {captured = captures.length; await Promise.all(captures.slice());} while (captured !== captures.length);
      const workerTraffic = await trace(gui);
      assert(actualAppResponses.some(r => new URL(r.url).pathname === '/app/' && r.archive_path === 'app/index.html' && r.state === 'ACTUAL_LOADED_BYTES_MATCH'), 'actual navigated app document bytes not authenticated');
      assert(actualAppResponses.every(r => r.state === 'ACTUAL_LOADED_BYTES_MATCH'), 'actual loaded app response capture failed');
      for (const created of workerTraffic.filter(e => e.direction === 'created')) {
        const url = new URL(created.url, gui.url());
        assert(url.origin === new URL(binding.origin).origin && url.pathname.startsWith('/app/'), 'worker is outside authenticated app archive');
        assert(actualAppResponses.some(r => r.url === url.href && r.state === 'ACTUAL_LOADED_BYTES_MATCH'), 'actual dedicated-worker loaded body unobserved: HOLD');
      }
      row.gui = {hello, locale_request, setup_snapshot_pair, scene_pair, dom: await readDom(gui), worker_traffic: workerTraffic, app_responses: actualAppResponses,
        external_solver_calls: null, external_solver_calls_not_exposed_by_public_GUI_protocol: true};
      row.gui.protocol_pairs = pairsFrom(row.gui.worker_traffic).filter(p => p.origin === 'app' && p.request_sequence > start && ['set_register','run_script'].includes(p.request.cmd));
      await retainPartial(report);
    } catch (error) {
      if (gui) row.partial_worker_traffic = await trace(gui).catch(() => null);
      await Promise.all(captures.slice()); row.partial_app_responses = actualAppResponses;
      report.error = String(error); await retainPartial(report); throw error;
    } finally {await context.close();}
  }
  assert.equal(report.rows.length, 18); return report;
}
