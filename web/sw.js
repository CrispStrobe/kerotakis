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
  "icon.svg",
];

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches
      .open(CACHE)
      .then((cache) => Promise.allSettled(SHELL.map((url) => cache.add(url))))
      .then(() => self.skipWaiting()),
  );
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

self.addEventListener("fetch", (event) => {
  if (event.request.method !== "GET") return;
  const url = new URL(event.request.url);
  if (url.origin !== self.location.origin) return;
  event.respondWith(
    caches.match(event.request, { ignoreSearch: true }).then(
      (hit) =>
        hit ||
        fetch(event.request).then((response) => {
          if (response.ok) {
            const copy = response.clone();
            caches.open(CACHE).then((cache) => cache.put(event.request, copy));
          }
          return response;
        }),
    ),
  );
});
