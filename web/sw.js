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

/** The cached document that owns `path`, for an offline navigation miss. */
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
    caches.match(event.request, { ignoreSearch: true }).then(
      (hit) =>
        hit ||
        fetch(event.request)
          .then((response) => {
            if (response.ok) {
              const copy = response.clone();
              caches.open(CACHE).then((cache) => cache.put(event.request, copy));
            }
            return response;
          })
          .catch(async (err) => {
            // Offline and uncached. For a navigation that is a directory
            // URL or a client-side route, the right answer is the shell
            // that owns it; for anything else the failure is honest.
            if (event.request.mode !== "navigate") throw err;
            const fallback = await caches.match(shellFor(url.pathname));
            if (fallback) return fallback;
            throw err;
          }),
    ),
  );
});
