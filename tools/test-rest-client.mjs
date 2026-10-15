import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import vm from 'node:vm';

const html = readFileSync('web/rest.html', 'utf8');
const code = html.match(/<script type="module">([\s\S]*?)<\/script>/)[1];
const elements = Object.fromEntries(['run', 'result', 'submit', 'key', 'endpoint', 'script', 'clear'].map(
  id => [id, { value: '', textContent: '', addEventListener: (_event, callback) => { elements[id].handler = callback; } }],
));
let requests = [];
const context = { document: { querySelector: selector => elements[selector.slice(1)] },
  URL, AbortSignal, Error, fetch: async (...args) => {
    requests.push(args);
    return { status: 200, json: async () => ({ diagnostics: '<script>never execute</script>' }) };
  },
};
vm.runInNewContext(code, context);
const submit = () => elements.run.handler({ preventDefault() {} });
elements.key.value = 'runtime-only-personal-key';
elements.endpoint.value = 'https://api.example';
elements.script.value = 'inspect v1';
await submit();
assert.equal(requests.length, 1);
assert.equal(String(requests[0][0]), 'https://api.example/run');
assert.equal(requests[0][1].redirect, 'error');
assert.equal(requests[0][1].credentials, 'omit');
assert.equal(requests[0][1].headers.Authorization, 'Bearer runtime-only-personal-key');
assert.equal(JSON.parse(requests[0][1].body).script, 'inspect v1');
assert.match(elements.result.textContent, /never execute/);
assert.equal(elements.submit.disabled, false);
for (const url of ['http://api.example', 'https://user:pass@api.example', 'https://api.example/path', 'https://api.example?x=1']) {
  elements.endpoint.value = url;
  await submit();
}
assert.equal(requests.length, 1);
elements.endpoint.value = 'https://api.example/api';
await submit();
assert.equal(String(requests[1][0]), 'https://api.example/api/run');
elements.clear.onclick();
assert.equal(elements.key.value, '');
assert.equal(elements.result.textContent, '');
assert.doesNotMatch(code, /localStorage|sessionStorage|innerHTML|document\.cookie/);
assert.match(readFileSync('tools/build-web.sh', 'utf8'), /\$ROOT\/web\/rest\.html/);
console.log('REST client: destination, credentials, safe rendering, clearing and payload controls passed');

for (const prefix of ['/', '/kerotakis/']) {
  const listeners = {};
  const origin = 'https://site.example';
  vm.runInNewContext(readFileSync('web/sw.js', 'utf8'), {
    URL, self: { location: { origin, href: origin + prefix + 'sw.js' },
      addEventListener: (event, callback) => { listeners[event] = callback; } },
    caches: { match: async () => ({ cached: true }) },
  });
  for (const path of ['api/run', 'api/health', 'api/docs', 'api/openapi.json', 'rest.html']) {
    let intercepted = false;
    listeners.fetch({ request: { method: 'GET', url: origin + prefix + path, mode: 'navigate' },
      respondWith: () => { intercepted = true; } });
    assert.equal(intercepted, false, 'service worker intercepted ' + path);
  }
  let intercepted = false;
  listeners.fetch({ request: { method: 'GET', url: origin + prefix + 'engine.wasm' },
    respondWith: () => { intercepted = true; } });
  assert.equal(intercepted, true, 'static engine caching changed');
}
console.log('Service worker: API/client bypass and static caching controls passed at root and project scope');
