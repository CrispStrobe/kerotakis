#!/usr/bin/env node
/**
 * Prove the built payload is actually an installable, offline-first PWA.
 *
 * Static inspection cannot answer this. Whether the manifest's `start_url`
 * resolves where you think, whether the service worker's scope covers the
 * bench, and whether a reload with the network cut still boots the engine
 * are all runtime facts — and every one of them was wrong here at some
 * point, for reasons no file diff would show.
 *
 * `tools/test-web-demo.mjs` is the neighbouring harness: it proves the
 * chemistry still comes out right in a browser. This one proves the
 * *packaging* around it.
 *
 * Usage: node tools/test-pwa.mjs <payload-dir>
 */

import { serve, browser, waitFor, PREFIX } from "./lib/headless.mjs";

const PAYLOAD = process.argv[2];
if (!PAYLOAD) {
  console.error("usage: node tools/test-pwa.mjs <payload-dir>");
  process.exit(2);
}

const failures = [];
const check = (cond, label, detail = "") => {
  console.log(`  ${cond ? "ok  " : "FAIL"} ${label}${detail ? ` — ${detail}` : ""}`);
  if (!cond) failures.push(label + (detail ? ` — ${detail}` : ""));
};

const { server, origin } = await serve(PAYLOAD);
console.log(`serving ${PAYLOAD} at ${origin}/`);
const page = await browser();

const finish = async (code) => {
  await page.close();
  server.close();
  process.exit(code);
};

/** The bench, given time to boot its worker-hosted engine. */
const benchBooted = () =>
  waitFor(page, `/Kerotakis/.test(document.body.innerText)`, { timeout: 30000 });

try {
  /* -- the bench installs its own worker ------------------------------- */
  console.log("\n== the bench registers the payload-root worker");
  await page.goto(`${origin}/app/`);

  const sw = await page.evaluate(`(async () => {
    const reg = await Promise.race([
      navigator.serviceWorker.ready,
      new Promise(r => setTimeout(() => r(null), 25000)),
    ]);
    if (!reg) return { ready: false };
    return {
      ready: true,
      scope: reg.scope,
      script: (reg.active || reg.installing || reg.waiting).scriptURL,
      controlled: !!navigator.serviceWorker.controller,
    };
  })()`);
  check(sw.ready, "a service worker reaches 'ready' from /app/");
  check(sw.script?.endsWith(`${PREFIX}/sw.js`),
        "it is the payload-root worker, not a per-page one", sw.script);
  check(sw.scope === `${origin}/`,
        "its scope covers the console page as well as the bench", sw.scope);

  /* -- the manifest resolves where it claims --------------------------- */
  console.log("\n== the manifest");
  const manifest = await page.cdp.send("Page.getAppManifest", {}, page.sessionId);
  const parsed = manifest.data ? JSON.parse(manifest.data) : null;
  check(!!parsed, "a manifest is served for /app/", manifest.url);
  // The defect this catches: Vite content-hashing a private copy into
  // app/assets/, which re-scopes the installed app to that directory.
  check(manifest.url === `${origin}/manifest.webmanifest`,
        "it is the shared payload-root manifest, not a bundled copy", manifest.url);
  check((manifest.errors ?? []).length === 0,
        "Chrome parses it without complaint", JSON.stringify(manifest.errors ?? []));

  const resolved = (rel) => new URL(rel ?? "", manifest.url).pathname;
  check(resolved(parsed?.start_url) === `${PREFIX}/app/`,
        "start_url resolves to the bench", resolved(parsed?.start_url));
  check(resolved(parsed?.scope) === `${PREFIX}/`,
        "scope resolves to the payload root", resolved(parsed?.scope));
  check(parsed?.display === "standalone", "display is standalone", parsed?.display);

  const icons = parsed?.icons ?? [];
  check(icons.some((i) => i.sizes === "192x192" && i.type === "image/png"),
        "a 192px PNG icon is declared");
  check(icons.some((i) => i.sizes === "512x512" && i.type === "image/png"
                          && i.purpose !== "maskable"),
        "a 512px PNG icon is declared");
  check(icons.some((i) => i.purpose === "maskable"),
        "a maskable icon is declared");

  // A 404 icon is the classic silent PWA defect: installing works, and the
  // home screen is blank. The apple-touch-icon rides along because iOS
  // ignores the manifest's icons entirely and reads that instead.
  const assets = [...icons.map((i) => i.src), "apple-touch-icon.png"];
  const statuses = await page.evaluate(`Promise.all(${JSON.stringify(assets)}.map(
    async (src) => {
      const url = new URL(src, ${JSON.stringify(manifest.url)}).href;
      return [url.split("/").pop(), (await fetch(url)).status];
    }))`);
  for (const [name, status] of statuses) {
    check(status === 200, `${name} is served`, `HTTP ${status}`);
  }
  const linked = await page.evaluate(
    `document.querySelector('link[rel="apple-touch-icon"]')?.href ?? null`,
  );
  check(linked === `${origin}/apple-touch-icon.png`,
        "the page links the apple-touch-icon (iOS reads this, not the manifest)",
        linked ?? "absent");

  /* -- the bench actually boots ---------------------------------------- */
  console.log("\n== the bench boots");
  check(await benchBooted(), "the app rendered");

  /* -- the worker precached the payload -------------------------------- */
  console.log("\n== the precache");
  const cached = await page.evaluate(`(async () => {
    const names = await caches.keys();
    const cache = await caches.open(names[0]);
    return {
      names,
      urls: (await cache.keys()).map(r => new URL(r.url).pathname),
    };
  })()`);
  check(cached.names.length === 1, "exactly one cache generation is live",
        cached.names.join(", "));
  for (const want of ["/app/index.html", "/index.html", "/manifest.webmanifest",
                      "/privacy.html", "/kerotakis_wasm_bg.wasm", "/db/wateq4f.dat"]) {
    check(cached.urls.includes(PREFIX + want), `precached ${want}`);
  }
  check(cached.urls.some((u) => new RegExp(`^${PREFIX}/app/assets/index-.*\\.js$`).test(u)),
        "precached the app's content-hashed bundle");

  /* -- offline --------------------------------------------------------- */
  console.log("\n== offline");
  await page.cdp.send("Network.emulateNetworkConditions",
    { offline: true, latency: 0, downloadThroughput: 0, uploadThroughput: 0 },
    page.sessionId);

  // The easy case: the document is cached under its own URL.
  await page.goto(`${origin}/app/index.html`);
  check(await benchBooted(), "offline reload of app/index.html boots");

  // The hard case, and the one that was broken. Nothing precaches `/app/`,
  // only `/app/index.html`, so a directory URL only resolves if the worker
  // falls back to the document that owns the path.
  await page.goto(`${origin}/app/`);
  check(await benchBooted(), "offline navigation to /app/ falls back to the shell");

  await page.goto(`${origin}/`);
  check(await benchBooted(), "offline navigation to the console page works");
} catch (err) {
  console.error(`\nharness error: ${err.stack ?? err.message}`);
  await finish(2);
}

console.log("");
if (failures.length) {
  console.log(`${failures.length} PWA check(s) failed:`);
  for (const f of failures) console.log(`  - ${f}`);
  await finish(1);
}
console.log("PWA: every check passed");
await finish(0);
