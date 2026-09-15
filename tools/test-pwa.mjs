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
import { readFile } from "node:fs/promises";
import { join } from "node:path";

const PAYLOAD = process.argv[2];
if (!PAYLOAD) {
  console.error("usage: node tools/test-pwa.mjs <payload-dir>");
  process.exit(2);
}

/**
 * Which "deployment" the server is serving, and the one thing that differs
 * between them.
 *
 * A release is not a file diff: it is a moment when the origin starts
 * answering with different bytes while browsers are still holding the
 * previous ones. Nothing static can be asked whether the worker pairs a
 * document with the engine it was built against — only a server that
 * changes underneath a live registration can.
 */
let deployment = 1;
const deployMarker = (html) =>
  html.replace("<head>", `<head><meta name="kero-deploy" content="${deployment}">`);

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

  // The page can switch manifests before the app starts. Test every target
  // independently so a localized source file omitted by build-web.sh cannot
  // pass merely because this browser launched in English.
  const localizedManifests = await page.evaluate(`Promise.all([
    ["manifest.webmanifest", "en"],
    ["manifest.de.webmanifest", "de"],
  ].map(async ([file, lang]) => {
    const response = await fetch(new URL(file, location.origin + ${JSON.stringify(PREFIX + "/")}));
    let body = null;
    try { body = await response.json(); } catch {}
    return { file, lang, status: response.status, body };
  }))`);
  for (const candidate of localizedManifests) {
    check(candidate.status === 200, `${candidate.file} is served`, `HTTP ${candidate.status}`);
    check(candidate.body?.lang === candidate.lang,
          `${candidate.file} declares ${candidate.lang}`, candidate.body?.lang ?? "unreadable");
    const candidateUrl = `${origin}/${candidate.file}`;
    check(new URL(candidate.body?.start_url ?? "", candidateUrl).pathname === `${PREFIX}/app/`,
          `${candidate.file} start_url resolves to the bench`);
    check(new URL(candidate.body?.scope ?? "", candidateUrl).pathname === `${PREFIX}/`,
          `${candidate.file} scope resolves to the payload root`);
  }

  // Exercise the actual pre-app switcher, not only direct fetches.
  await page.evaluate(`localStorage.setItem("kerotakis.locale", "de")`);
  await page.goto(`${origin}/app/`);
  // Wait for the DOM before asking Chrome. `Page.getAppManifest` answers
  // from Chrome's own parse of the document, which it can take before the
  // inline switcher has rewritten the href — the same payload failed,
  // passed and failed again on three consecutive runs. The switcher itself
  // is synchronous and correct; this check was simply asking too early.
  const switched = await waitFor(page,
    `document.getElementById("app-manifest")?.getAttribute("href")?.endsWith("manifest.de.webmanifest") === true`,
    { timeout: 10000 });
  check(switched === true, "the page switches its manifest link to German");
  const germanManifest = await page.cdp.send("Page.getAppManifest", {}, page.sessionId);
  check(germanManifest.url === `${origin}/manifest.de.webmanifest`,
        "German locale selects the German manifest", germanManifest.url);
  check((germanManifest.errors ?? []).length === 0,
        "Chrome parses the German manifest without complaint",
        JSON.stringify(germanManifest.errors ?? []));
  await page.evaluate(`localStorage.setItem("kerotakis.locale", "en")`);
  await page.goto(`${origin}/app/`);

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
                      "/manifest.de.webmanifest", "/privacy.html", "/privacy.de.html",
                      "/kerotakis_wasm_bg.wasm", "/db/wateq4f.dat"]) {
    check(cached.urls.includes(PREFIX + want), `precached ${want}`);
  }
  check(cached.urls.some((u) => new RegExp(`^${PREFIX}/app/assets/index-.*\\.js$`).test(u)),
        "precached the app's content-hashed bundle");

  /* -- a deploy lands while this worker is still in charge -------------- */
  //
  // The defect: `/app/` is the URL the README advertises and the manifest
  // starts at, and NOTHING precaches it — only `app/index.html` is. The
  // fetch handler therefore missed the cache on it and went to the
  // network, while `kerotakis_wasm.js` and the databases, whose names
  // never change between builds, came cache-first out of the deploy the
  // worker was installed from. A new app bundle against an older engine:
  // the pairing that produced #599's shelf of 188 locked bottles, where
  // the bundle asked an engine that has no `catalog` method for a catalog.
  //
  // Reproduced rather than reasoned about: a second worker registration in
  // a second origin, installed from the console page so that `/app/` has
  // never been fetched, and then the server starts answering as a later
  // deployment would.
  console.log("\n== a deploy lands under a live worker");
  const { server: rollout, origin: rolling } = await serve(PAYLOAD, {
    handleRequest: async (request, response) => {
      if (deployment === 1) return false;
      const path = new URL(request.url, "http://x").pathname;
      if (!/\/app\/(index\.html)?$/.test(path)) return false;
      response.writeHead(200, { "content-type": "text/html" });
      response.end(deployMarker(await readFile(join(PAYLOAD, "app/index.html"), "utf8")));
      return true;
    },
  });
  try {
    // The console page installs the payload-root worker, and the bench URL
    // is never visited: this is a learner who lands where the repository
    // points them and clicks through to the bench later.
    await page.goto(`${rolling}/`);
    await waitFor(page, `navigator.serviceWorker.controller !== null`, { timeout: 30000 })
      .catch(() => {});
    await page.goto(`${rolling}/`);
    const controlled = await page.evaluate(`navigator.serviceWorker.controller !== null`);
    check(controlled, "the console page installs a worker that controls it");
    const before = await page.evaluate(`(async () => {
      const cache = await caches.open((await caches.keys())[0]);
      return (await cache.keys()).map((r) => new URL(r.url).pathname);
    })()`);
    // Non-vacuity: if `/app/` were already cached here the rest would pass
    // whatever the fetch handler does, and the engine has to be cached or
    // there is no older half to pair anything with.
    check(!before.includes(`${PREFIX}/app/`),
          "nothing has cached the bench's directory URL yet");
    check(before.includes(`${PREFIX}/kerotakis_wasm.js`),
          "the engine is in this generation's cache");

    deployment = 2;
    await page.goto(`${rolling}/app/`);
    const served = await page.evaluate(
      `document.querySelector('meta[name="kero-deploy"]')?.getAttribute('content') ?? "1"`);
    check(served === "1",
          "the bench document comes from the same deploy as the engine beside it",
          `document from deployment ${served}, engine from deployment 1`);
    check(await benchBooted(), "and it boots");
    const generations = await page.evaluate(`caches.keys()`);
    check(generations.length === 1,
          "still one cache generation, not a blend of two", generations.join(", "));
  } finally {
    deployment = 1;
    rollout.close();
  }
  await page.goto(`${origin}/app/`);
  await benchBooted();

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
