/** Exercise the actual animated canvas, static scene and command path in Chrome. */
import assert from "node:assert/strict";
import { serve, browser, waitFor } from "./lib/headless.mjs";

const payload = process.argv[2];
if (!payload) throw new Error("usage: node tools/test-fluid-rendering.mjs <payload-dir>");
const { server, origin } = await serve(payload);
let page;
const requirePage = async (expression, message, timeout = 20000) => {
  assert(await waitFor(page, expression, { timeout, step: 25 }), message);
};

const command = async (line) => {
  await page.evaluate(`(() => {
    const box = document.querySelector("form.bar input");
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value").set.call(box, ${JSON.stringify(line)});
    box.dispatchEvent(new Event("input", { bubbles: true }));
  })()`);
  await requirePage(`document.querySelector("form.bar input")?.value === ${JSON.stringify(line)} &&
    !document.querySelector("form.bar input")?.disabled`, "command entry did not become ready");
  await page.evaluate(`document.querySelector("form.bar").requestSubmit()`);
};

const audit = () => page.evaluate(`JSON.stringify((() => {
  const vessel = document.querySelector('[data-vessel-id="0"]');
  const layer = vessel?.querySelector('.scene-liquid-layer');
  const canvas = vessel?.querySelector('foreignObject.visible canvas');
  if (!layer || !canvas) return null;
  const rect = { y: Number(layer.getAttribute('y')), height: Number(layer.getAttribute('height')) };
  const context = canvas.getContext('2d');
  const x = Math.floor(canvas.width * 0.3); // away from legitimate falling droplets
  const y = Math.floor((rect.y + rect.height / 2) / 140 * canvas.height);
  const airY = Math.max(0, Math.floor(rect.y / 140 * canvas.height) - 4);
  return { sameCanvas: canvas === window.fluidCanvasBefore,
    colour: layer.getAttribute('data-srgb').split(',').map(Number),
    liquid: [...context.getImageData(x, y, 1, 1).data],
    air: [...context.getImageData(x, airY, 1, 1).data],
    below: [...context.getImageData(x, canvas.height - 1, 1, 1).data],
    liquidY: rect.y };
})())`);

try {
  page = await browser();
  await page.goto(`${origin}/privacy.html`);
  await page.evaluate(`(() => {
    localStorage.clear();
    localStorage.setItem("kerotakis.mode.v1", "sandbox");
    localStorage.setItem("kerotakis.locale", "en");
    localStorage.setItem("kerotakis.console.v1", "shown");
  })()`);
  await page.goto(`${origin}/app/`);
  await requirePage(`document.querySelector("form.bar input") && document.querySelector('[data-vessel-id="0"] foreignObject canvas')`, "bench did not boot", 60000);
  await page.evaluate(`window.fluidCanvasBefore = document.querySelector('[data-vessel-id="0"] foreignObject canvas')`);
  await command("add v1 food_colour_blue 100mL");
  await requirePage(`document.querySelector('[data-vessel-id="0"] .scene-liquid-layer')`, "the material did not produce a liquid layer");
  await command("stir v1 100rpm 1s");
  await requirePage(`(() => {
    const vessel = document.querySelector('[data-vessel-id="0"]');
    const layer = vessel?.querySelector('.scene-liquid-layer');
    const canvas = vessel?.querySelector('foreignObject.visible canvas');
    if (!layer || !canvas) return false;
    const y = Math.floor((Number(layer.getAttribute('y')) + Number(layer.getAttribute('height')) / 2) / 140 * canvas.height);
    return canvas.getContext('2d').getImageData(Math.floor(canvas.width * .3), y, 1, 1).data[3] > 0;
  })()`, "the running canvas never painted the accepted liquid");
  const result = JSON.parse(await audit());
  assert(result.sameCanvas, "animation replaced its live canvas");
  assert(result.liquidY > 80, "100mL unexpectedly occupies the whole beaker");
  assert.deepEqual(result.liquid.slice(0, 3), result.colour, "animation colour differs from engine layer colour");
  assert.equal(result.air[3], 0, "animation painted empty headspace");
  assert.equal(result.below[3], 0, "animation painted below the vessel");
  console.log("Fluid rendering: live canvas, accepted scene colour and fill boundaries pass");

  await requirePage(`document.querySelector('[data-vessel-id="0"] foreignObject.visible.fading')`, "first animation never began fading");
  await command("stir v1 120rpm 1s");
  await requirePage(`document.querySelector('[data-vessel-id="0"] foreignObject.visible:not(.fading)')`, "replacement activity never started");
  await new Promise((resolve) => setTimeout(resolve, 850));
  assert(await page.evaluate(`Boolean(document.querySelector('[data-vessel-id="0"] foreignObject.visible:not(.fading)'))`),
    "the preceding fade timer hid the new animation");
  assert(await page.evaluate(`document.querySelector('[data-vessel-id="0"] foreignObject canvas') === window.fluidCanvasBefore`),
    "replacement activity changed the live canvas node");
  console.log("Fluid rendering: a superseded fade cannot hide replacement activity");

  await requirePage(`!document.querySelector('[data-vessel-id="0"] foreignObject.visible')`,
    "replacement animation never finished");
  // Deterministically reproduce the backing-store state left by economy mode.
  // Keep the node: the next full-grid run must reset its actual dimensions.
  await page.evaluate(`(() => {
    const canvas = window.fluidCanvasBefore;
    canvas.width = 34;
    canvas.height = 48;
  })()`);
  await command("stir v1 100rpm 1s");
  await requirePage(`(() => {
    const canvas = document.querySelector('[data-vessel-id="0"] foreignObject.visible canvas');
    return canvas?.width === 50 && canvas?.height === 70 &&
      canvas.getContext('2d').getImageData(15, 57, 1, 1).data[3] > 0;
  })()`, "a new full-grid run did not restore its canvas backing store");
  const restarted = JSON.parse(await audit());
  assert(restarted.sameCanvas, "economy restart replaced the persistent canvas");
  assert.deepEqual(restarted.liquid.slice(0, 3), restarted.colour,
    "economy restart clipped or changed the accepted liquid colour");
  assert.equal(restarted.air[3], 0, "economy restart painted headspace");
  assert.equal(restarted.below[3], 0, "economy restart painted below the vessel");
  console.log("Fluid rendering: a new run restores the canvas after economy mode");
} finally {
  await page?.close();
  server.close();
}
