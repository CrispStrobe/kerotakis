#!/usr/bin/env node
import { browser, serve, waitFor } from "../../tools/lib/headless.mjs";

const { server, origin } = await serve(new URL("dist", import.meta.url).pathname);
let page;
try {
  page = await browser({ disableGpu: false, extraArgs: ["--enable-unsafe-swiftshader"] });
  await page.cdp.send("Performance.enable", {}, page.sessionId);
  const requests = [];
  let maxCanvasPixels = 0;
  let maxJsHeapBytes = 0;
  page.cdp.on((message) => {
    if (message.sessionId === page.sessionId && message.method === "Network.requestWillBeSent") requests.push(message.params.request.url);
  });
  await page.goto(`${origin}/`);
  for (const candidate of ["3dmol", "molstar"]) {
    for (const fixture of ["water", "nacl", "peptide", "orbital", "trajectory"]) {
      await page.evaluate(`(() => {
        const candidate = document.querySelector('input[name="candidate"][value=${JSON.stringify(candidate)}]');
        const fixture = document.querySelector('input[name="fixture"][value=${JSON.stringify(fixture)}]');
        candidate.checked = true;
        candidate.dispatchEvent(new Event('change', { bubbles: true }));
        fixture.checked = true;
        fixture.dispatchEvent(new Event('change', { bubbles: true }));
      })()`);
      const ready = await waitFor(page, `(() => {
        const snapshot = globalThis.__brd080?.snapshot();
        return document.querySelector('#status')?.dataset.state === 'ready'
          && snapshot?.candidate === ${JSON.stringify(candidate)}
          && snapshot?.fixture === ${JSON.stringify(fixture)};
      })()`, { timeout: 45_000, step: 100 });
      if (!ready) throw new Error(`${candidate}/${fixture}: ${await page.evaluate("document.querySelector('#status')?.textContent")}`);
      const rows = await page.evaluate("document.querySelectorAll('#atoms tr').length");
      const canvases = await page.evaluate("document.querySelectorAll('#viewer canvas').length");
      if (rows < 1 || canvases !== 1) throw new Error(`${candidate}/${fixture}: expected semantic rows and exactly one canvas, got ${rows}/${canvases}`);
      const interaction = await page.evaluate(`(async () => {
        const atom = document.querySelector('[data-atom]');
        atom.checked = true;
        const labels = document.querySelector('#labels');
        labels.checked = true;
        await globalThis.__brd080.select([0]);
        await globalThis.__brd080.setLabels(true);
        await globalThis.__brd080.resize(5000, 5000, 9);
        return globalThis.__brd080.snapshot();
      })()`);
      if (interaction?.candidate !== candidate || interaction?.fixture !== fixture || interaction?.status !== "ready"
        || interaction?.selectedAtomIds?.[0] !== 0 || interaction.labelsVisible !== true
        || interaction.width !== 1280 || interaction.height !== 960 || interaction.dpr !== 2) {
        throw new Error(`${candidate}/${fixture}: interaction snapshot failed: ${JSON.stringify(interaction)}`);
      }
      await page.evaluate(`(() => {
        const labels = document.querySelector('#labels');
        labels.checked = false;
        labels.dispatchEvent(new Event('change', { bubbles: true }));
      })()`);
      const canvasPixels = await page.evaluate("[...document.querySelectorAll('#viewer canvas')].reduce((sum, canvas) => sum + canvas.width * canvas.height, 0)");
      maxCanvasPixels = Math.max(maxCanvasPixels, canvasPixels);
      const { metrics } = await page.cdp.send("Performance.getMetrics", {}, page.sessionId);
      maxJsHeapBytes = Math.max(maxJsHeapBytes, metrics.find(({ name }) => name === "JSHeapUsedSize")?.value ?? 0);
    }
  }
  await page.evaluate(`(() => {
    const reducedMotion = document.querySelector('#reduce-motion');
    reducedMotion.checked = !reducedMotion.checked;
    reducedMotion.dispatchEvent(new Event('change', { bubbles: true }));
  })()`);
  if (!await waitFor(page, `document.querySelector('#status')?.dataset.state === 'ready'`, { timeout: 45_000, step: 100 })) {
    throw new Error(`reduced-motion reload failed: ${await page.evaluate("document.querySelector('#status')?.textContent")}`);
  }
  if (maxCanvasPixels > 1_280 * 960 * 4) throw new Error(`canvas backing store exceeded DPR-2 bound: ${maxCanvasPixels} pixels`);
  const external = requests.filter((url) => !url.startsWith(origin) && !url.startsWith("data:") && !url.startsWith("blob:"));
  if (external.length) throw new Error(`external requests observed: ${[...new Set(external)].join(", ")}`);
  console.log(JSON.stringify({
    schema: "kerotakis.brd080-browser-smoke.v1",
    paths: 10,
    localRequests: requests.length,
    externalRequests: 0,
    maxCanvasPixels,
    maxJsHeapBytes,
    memoryQualification: "headless Chromium JS-heap proxy; not physical mobile RAM or GPU memory",
  }));
} finally {
  await page?.close();
  server.close();
}
