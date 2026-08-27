import { defineConfig, type Plugin, type HtmlTagDescriptor } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

/**
 * The payload-root tags: the manifest, the icons, and the iOS chrome hints.
 *
 * These are NOT this bundle's assets. They are the single copy the console
 * page and the bench share, which `tools/build-web.sh` puts one directory
 * up from `app/`. Left in index.html, Vite resolves `../manifest.webmanifest`
 * to a real file and content-hashes a private copy into `app/assets/` — and
 * that quietly breaks the PWA, because `start_url` and `scope` are resolved
 * against wherever the manifest actually lives. A copy under `app/assets/`
 * scopes the installed app to `app/assets/`.
 *
 * `order: "post"` runs after Vite's asset pass, so what is injected here is
 * emitted verbatim.
 */
function payloadRootTags(): Plugin {
  const tags: HtmlTagDescriptor[] = [
    { tag: "link", attrs: { rel: "icon", href: "../icon.svg", type: "image/svg+xml" }, injectTo: "head" },
    { tag: "link", attrs: { rel: "icon", href: "../icon-192.png", sizes: "192x192", type: "image/png" }, injectTo: "head" },
    // iOS reads this, not the manifest, for Add to Home Screen; without it
    // the home screen gets a screenshot of the page.
    { tag: "link", attrs: { rel: "apple-touch-icon", href: "../apple-touch-icon.png" }, injectTo: "head" },
    // The manifest link is created HERE, by this script, rather than
    // injected as static markup that a later script rewrites. Chrome
    // resolves the manifest while parsing the head and keeps what it
    // first saw, so rewriting the href afterwards moved the DOM and
    // nothing else — a German reader installed the bench and got the
    // English name and icons on their home screen.
    //
    // Appending from script means only one href ever exists. The catch
    // still appends the English link: the wrong language is recoverable,
    // no manifest at all makes the bench uninstallable.
    {
      tag: "script",
      children: '{ var href = "../manifest.webmanifest"; try { var choice = localStorage.getItem("kerotakis.locale"); if ((choice || navigator.language).toLowerCase().startsWith("de")) href = "../manifest.de.webmanifest"; } catch {} var l = document.createElement("link"); l.id = "app-manifest"; l.rel = "manifest"; l.href = href; document.head.appendChild(l); }',
      injectTo: "head",
    },
  ];
  return {
    name: "kerotakis-payload-root-tags",
    transformIndexHtml: { order: "post", handler: () => tags },
  };
}

// The app builds to static files that tools/build-web.sh copies beside the
// wasm-bindgen output — same serving model as the legacy console page.
export default defineConfig({
  plugins: [svelte(), payloadRootTags()],
  base: "./",
  build: {
    target: "es2022",
    outDir: "dist",
  },
  worker: {
    format: "es",
  },
  server: {
    fs: {
      // The worker imports the shared two-wasm bridge from web/ (one
      // source of truth with the legacy console page).
      allow: [".."],
    },
  },
});
