// Offline-first for a lab whose premise is offline-first: precache the
// shell and both engines, serve cache-first, and let each deploy retire
// the previous cache by version. The version is stamped by build-web.sh;
// in an unstamped checkout the placeholder still yields a working cache.
//
// The Emscripten engine files may be absent in an engineless build —
// precaching tolerates missing entries, the same honest degradation the
// build script and the page itself already make.
const CACHE = "kero-__KERO_CACHE__";
const SHELL = [
  "./",
  "index.html",
  "privacy.html",
  "privacy.de.html",
  "kerotakis.mjs",
  "kerotakis_wasm.js",
  "kerotakis_wasm_bg.wasm",
  "results.postcard",
  "iphreeqc.mjs",
  "iphreeqc.wasm",
  "db/wateq4f.dat",
  "db/minteq.v4.dat",
  "db/pitzer.dat",
  "manifest.webmanifest",
  "manifest.de.webmanifest",
  "icon.svg",
  "icon-192.png",
  "icon-512.png",
  "icon-maskable-512.png",
  "apple-touch-icon.png",
  "screenshot-wide.png",
  "screenshot-narrow.png",
];

// The bench app (web/app): entries stamped by build-web.sh from the built
// output, because the filenames are content-hashed. The placeholder is one
// unfetchable entry in an unstamped checkout, and allSettled tolerates it
// exactly as it tolerates absent engine files.
const APP = ["__KERO_APP_ASSETS__"];

// The two documents this payload serves. A navigation that misses the
// cache falls back to whichever of these owns its path — without that, a
// cold offline open of /app/ (the URL the README advertises) fails, because
// only `app/index.html` is precached and the navigation asks for `app/`.
const SHELLS = [
  { prefix: "app/", document: "app/index.html" },
  { prefix: "", document: "index.html" },
];

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches
      .open(CACHE)
      .then((cache) =>
        Promise.allSettled(SHELL.concat(APP).map((url) => cache.add(url))),
      )
      .then(async () => {
        // Take over at once on a first install, or on an update with no
        // page open — nothing is running whose assets could be pulled out
        // from under it. Otherwise wait and tell the open pages: activating
        // deletes the previous cache, and a running page may still ask for
        // a content-hashed chunk that lived only there. The page decides
        // when to swap (see web/app/src/lib/pwa.svelte.ts).
        if (!self.registration.active) return self.skipWaiting();
        const clients = await self.clients.matchAll({ type: "window" });
        if (clients.length === 0) return self.skipWaiting();
        for (const client of clients) client.postMessage({ kero: "update-ready" });
      }),
  );
});

self.addEventListener("message", (event) => {
  if (event.data && event.data.kero === "skip-waiting") self.skipWaiting();
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((keys) =>
        Promise.all(keys.filter((k) => k !== CACHE).map((k) => caches.delete(k))),
      )
      .then(() => self.clients.claim()),
  );
});

/** The cached document that owns `path`, for a navigation the cache does
 * not hold under its own URL. */
function shellFor(path) {
  const scope = new URL("./", self.location.href).pathname;
  const rest = path.startsWith(scope) ? path.slice(scope.length) : path;
  const shell = SHELLS.find((s) => rest.startsWith(s.prefix));
  return new URL(shell.document, new URL("./", self.location.href)).href;
}

self.addEventListener("fetch", (event) => {
  if (event.request.method !== "GET") return;
  const url = new URL(event.request.url);
  if (url.origin !== self.location.origin) return;
  event.respondWith(
    caches.match(event.request, { ignoreSearch: true }).then(async (hit) => {
      if (hit) return hit;
      // A navigation this cache does not hold under its own URL — a
      // directory URL like `/app/`, or a client-side route — is answered
      // by the cached document that owns it, ONLINE as well as off.
      //
      // This used to happen only when the network had already failed, and
      // that was a release defect rather than a nicety. `/app/` is the URL
      // the README advertises and the manifest starts at, and nothing
      // precaches it: only `app/index.html` is. So a learner whose first
      // visit to `/app/` fell after a deploy — the worker installed from
      // the console page, the new worker still waiting for this tab to let
      // go — was served the NEW deployment's document over the network,
      // while `kerotakis_wasm.js` and the databases, whose names never
      // change, came cache-first out of the PREVIOUS deploy. A new app
      // bundle against an older engine, which is exactly the pairing that
      // produced #599's shelf of 188 locked bottles: an engine with no
      // `catalog` method, asked for a catalog. Worse, the network answer
      // was then `cache.put` into the old generation, so the mismatch
      // outlived the deploy that caused it.
      //
      // Exact matches still win, so `privacy.html` is still `privacy.html`
      // and not the shell that would otherwise claim its path. A new
      // document reaches a controlled page the one way it was always meant
      // to: the waiting worker activates, the cache turns over as a whole,
      // and the page reloads (see web/app/src/lib/pwa.svelte.ts).
      if (event.request.mode === "navigate") {
        const shell = await caches.match(shellFor(url.pathname));
        if (shell) return shell;
      }
      return fetch(event.request)
        .then((response) => {
          if (response.ok) {
            const copy = response.clone();
            caches.open(CACHE).then((cache) => cache.put(event.request, copy));
          }
          return response;
        })
        .catch((err) => {
          // Offline, uncached, and no shell owns it: the failure is honest.
          throw err;
        });
    }),
  );
});
