import { defineConfig, type Plugin, type HtmlTagDescriptor } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

import { execSync } from "node:child_process";

/** The commit this was built from: CI's stamp, else git, else unknown. */
function buildCommit(): string {
  const fromCi = process.env.GITHUB_SHA;
  if (fromCi) return fromCi.slice(0, 7);
  try {
    return (
      execSync("git rev-parse --short=7 HEAD", { stdio: ["ignore", "pipe", "ignore"] })
        .toString()
        .trim() || "unknown"
    );
  } catch {
    // A tarball with no .git is a legitimate way to build this.
    return "unknown";
  }
}

/** The tag or branch, when there is one. Never invented. */
function buildRef(): string {
  const fromCi = process.env.GITHUB_REF_NAME;
  if (fromCi) return fromCi;
  try {
    const tag = execSync("git describe --tags --exact-match", {
      stdio: ["ignore", "pipe", "ignore"],
    })
      .toString()
      .trim();
    if (tag) return tag;
  } catch {
    /* not on a tag, which is the normal case */
  }
  try {
    return (
      execSync("git rev-parse --abbrev-ref HEAD", { stdio: ["ignore", "pipe", "ignore"] })
        .toString()
        .trim() || ""
    );
  } catch {
    return "";
  }
}


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
    {
      tag: "script",
      // Create the manifest link only after choosing its locale. Browsers may
      // consume a rel=manifest link as soon as the parser sees it, so mutating
      // an already-present English link can leave Chrome installing the
      // English manifest even though the DOM later shows the German href.
      children: '{ let choice; try { choice = localStorage.getItem("kerotakis.locale"); } catch {} const manifest = document.createElement("link"); manifest.id = "app-manifest"; manifest.rel = "manifest"; manifest.href = (choice || navigator.language).toLowerCase().startsWith("de") ? "../manifest.de.webmanifest" : "../manifest.webmanifest"; document.head.append(manifest); }',
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
  // Stamped in rather than read at runtime: the running app has no way to
  // ask what it was built from, and "which build?" is the first question
  // of every bug report.
  define: {
    __KERO_COMMIT__: JSON.stringify(buildCommit()),
    __KERO_REF__: JSON.stringify(buildRef()),
    __KERO_BUILT_AT__: JSON.stringify(new Date().toISOString()),
  },
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
