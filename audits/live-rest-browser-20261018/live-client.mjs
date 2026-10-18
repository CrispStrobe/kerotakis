// Isolated browser acceptance. Fixtures test UI behavior, never chemistry.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { chromium } from '@playwright/test';

const workspace = process.env.GITHUB_WORKSPACE;
assert(workspace);
const spec = JSON.parse(await readFile(path.join(workspace,
  'audits/live-rest-browser-20261018/release-binding.json'), 'utf8'));
assert.match(spec.expected_engine_source, /^[a-f0-9]{40}$/);
assert.match(spec.expected_binary_sha256, /^[a-f0-9]{64}$/);
const source = await readFile(path.join(workspace, 'web/rest.html'));
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
assert.equal(sha(source), spec.client_sha256);
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
  browser = await chromium.launch({ headless: true });
  report.browser = { version: browser.version(), web_security_disabled: false };
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
