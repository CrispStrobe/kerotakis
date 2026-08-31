import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { build } from "vite";

const outDir = await mkdtemp(join(tmpdir(), "brd080-molstar-test-"));
try {
  await build({
    root: new URL(".", import.meta.url).pathname,
    configFile: false,
    logLevel: "silent",
    build: {
      ssr: new URL("src/molstarAdapter.test.ts", import.meta.url).pathname,
      outDir,
      emptyOutDir: true,
      rollupOptions: { output: { entryFileNames: "test.mjs" } },
    },
  });
  await import(pathToFileURL(join(outDir, "test.mjs")));
} finally {
  await rm(outDir, { recursive: true, force: true });
}
