import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// The app builds to static files that tools/build-web.sh copies beside the
// wasm-bindgen output — same serving model as the legacy console page.
export default defineConfig({
  plugins: [svelte()],
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
