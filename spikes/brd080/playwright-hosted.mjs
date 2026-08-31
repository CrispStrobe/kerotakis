#!/usr/bin/env node
import { chromium, devices } from "@playwright/test";

const origin = process.env.BRD080_ORIGIN?.replace(/\/$/, "");
if (!origin || !origin.startsWith("https://")) throw new Error("BRD080_ORIGIN must be an HTTPS deployment URL");

const browser = await chromium.launch({ headless: true });
const context = await browser.newContext({
  ...devices["Pixel 7"],
  reducedMotion: "reduce",
  serviceWorkers: "block",
});
const page = await context.newPage();
const requests = [];
page.on("request", (request) => requests.push(request.url()));

try {
  await page.goto(origin, { waitUntil: "networkidle" });
  const paths = [];
  for (const candidate of ["3dmol", "molstar"]) {
    for (const fixture of ["water", "nacl", "peptide", "orbital", "trajectory"]) {
      const candidateControl = page.locator(`input[name="candidate"][value="${candidate}"]`);
      await candidateControl.focus();
      if (!await candidateControl.isChecked()) await page.keyboard.press("Space");
      const fixtureControl = page.locator(`input[name="fixture"][value="${fixture}"]`);
      await fixtureControl.focus();
      if (!await fixtureControl.isChecked()) await page.keyboard.press("Space");
      await page.locator("#status[data-state=ready]").waitFor({ timeout: 45_000 });

      const atom = page.locator("[data-atom]").first();
      await atom.focus();
      await page.keyboard.press("Space");
      const labels = page.locator("#labels");
      await labels.focus();
      if (!await labels.isChecked()) await page.keyboard.press("Space");
      await page.waitForFunction(() => {
        const snapshot = globalThis.__brd080?.snapshot();
        return snapshot?.selectedAtomIds?.[0] === 0 && snapshot?.labelsVisible === true;
      });
      await page.evaluate(() => globalThis.__brd080.resize(5000, 5000, 9));
      const snapshot = await page.evaluate(() => globalThis.__brd080.snapshot());
      if (snapshot?.candidate !== candidate || snapshot?.fixture !== fixture || snapshot?.status !== "ready"
        || snapshot.width !== 1280 || snapshot.height !== 960 || snapshot.dpr !== 2) {
        throw new Error(`${candidate}/${fixture}: invalid snapshot ${JSON.stringify(snapshot)}`);
      }
      const canvases = await page.locator("#viewer canvas").count();
      const rows = await page.locator("#atoms tr").count();
      if (canvases !== 1 || rows < 1) throw new Error(`${candidate}/${fixture}: expected one canvas and semantic rows`);
      paths.push(`${candidate}/${fixture}`);

      await labels.focus();
      await page.keyboard.press("Space");
    }
  }

  const reduceMotion = page.locator("#reduce-motion");
  await reduceMotion.focus();
  await page.keyboard.press("Space");
  await page.locator("#status[data-state=ready]").waitFor({ timeout: 45_000 });

  const external = requests.filter((url) => !url.startsWith(origin) && !url.startsWith("data:") && !url.startsWith("blob:"));
  if (external.length) throw new Error(`deployment made external requests: ${[...new Set(external)].join(", ")}`);
  const webgl = await page.evaluate(() => {
    const canvas = document.querySelector("#viewer canvas");
    const gl = canvas?.getContext("webgl2") ?? canvas?.getContext("webgl");
    if (!gl) return null;
    const debug = gl.getExtension("WEBGL_debug_renderer_info");
    return {
      renderer: debug ? gl.getParameter(debug.UNMASKED_RENDERER_WEBGL) : "masked",
      vendor: debug ? gl.getParameter(debug.UNMASKED_VENDOR_WEBGL) : "masked",
      version: gl.getParameter(gl.VERSION),
      maxTextureSize: gl.getParameter(gl.MAX_TEXTURE_SIZE),
    };
  });
  console.log(JSON.stringify({
    schema: "kerotakis.brd080-playwright-hosted.v1",
    origin,
    engine: await browser.version(),
    emulation: "Playwright Pixel 7 profile; not physical-device evidence",
    paths,
    requests: requests.length,
    externalRequests: 0,
    webgl,
  }));
} finally {
  await context.close();
  await browser.close();
}
