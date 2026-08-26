#!/usr/bin/env node
/**
 * Photograph the real bench for the manifest's `screenshots`.
 *
 * Chrome's install dialog is a plain confirmation without them and a
 * proper app card with them, and the same two images serve as the honest
 * "this is what it looks like" for the README. Honest is the operative
 * word: these are captures of the actual app, mid-experiment, with the
 * numbers its own solver produced. Nothing here mocks a screen.
 *
 * The experiment is silver and salt, because the precipitate is the one
 * result that shows the whole stack working in a single frame.
 *
 * Usage: node tools/gen-pwa-screenshots.mjs <payload-dir> [out-dir]
 */

import { writeFile } from "node:fs/promises";
import { join } from "node:path";
import { serve, browser, waitFor } from "./lib/headless.mjs";

const PAYLOAD = process.argv[2];
const OUT = process.argv[3] ?? "web";
if (!PAYLOAD) {
  console.error("usage: node tools/gen-pwa-screenshots.mjs <payload-dir> [out-dir]");
  process.exit(2);
}

const SCRIPT = ["add v1 water 200mL", "add v1 NaCl 0.1mol", "add v1 AgNO3 0.01mol"];

const SHOTS = [
  {
    name: "screenshot-wide.png",
    width: 1280, height: 800, scale: 1, mobile: false,
    form_factor: "wide",
    label: "The bench, the shelf and the notebook side by side, with silver chloride settling out.",
  },
  {
    name: "screenshot-narrow.png",
    width: 430, height: 932, scale: 2, mobile: true,
    form_factor: "narrow",
    label: "The same experiment on a phone, one pane at a time.",
  },
];

const { server, origin } = await serve(PAYLOAD);
const page = await browser();

/** Type a line into the command bar and wait for the engine to answer. */
const run = async (line) => {
  await page.evaluate(`(() => {
    const input = document.querySelector('form.bar input[aria-label="command"]');
    if (!input) throw new Error("no command bar");
    // Svelte 5 binds through the property, so the native setter plus an
    // input event is what makes the binding see a programmatic value.
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")
      .set.call(input, ${JSON.stringify(line)});
    input.dispatchEvent(new Event("input", { bubbles: true }));
    input.form.dispatchEvent(new SubmitEvent("submit", { bubbles: true, cancelable: true }));
  })()`);
  // The bar disables itself while the solver runs; its return is the signal.
  await waitFor(page, `!document.querySelector('form.bar input[aria-label="command"]').disabled`,
                { timeout: 60000 });
};

try {
  for (const shot of SHOTS) {
    await page.cdp.send("Emulation.setDeviceMetricsOverride", {
      width: shot.width, height: shot.height,
      deviceScaleFactor: shot.scale, mobile: shot.mobile,
    }, page.sessionId);

    await page.goto(`${origin}/app/`);
    const ready = await waitFor(page, `document.querySelector('form.bar input')`, { timeout: 60000 });
    if (!ready) throw new Error("the command bar never appeared");
    // The engine attaches after the shell paints; running before it does
    // would photograph a refusal.
    await waitFor(page, `(() => {
      const status = document.querySelector('.status');
      return status && !status.textContent.includes('starting') && !status.textContent.includes('startet');
    })()`, { timeout: 60000 });
    for (const line of SCRIPT) await run(line);
    // A command response can beat the session's parallel first scene load
    // on a cold WASM start. Photograph the computed glassware, never the
    // transient "warming up" shell.
    const painted = await waitFor(page, `document.querySelectorAll('.bench .vessel').length > 0`,
                                  { timeout: 60000 });
    if (!painted) throw new Error("the computed vessel scene never painted");

    const { data } = await page.cdp.send("Page.captureScreenshot",
      { format: "png", captureBeyondViewport: false }, page.sessionId);
    const path = join(OUT, shot.name);
    await writeFile(path, Buffer.from(data, "base64"));
    console.log(`   ${path}  ${shot.width * shot.scale}x${shot.height * shot.scale}  (${shot.form_factor})`);
  }

  console.log("\nManifest entries:");
  console.log(JSON.stringify(SHOTS.map((s) => ({
    src: s.name,
    sizes: `${s.width * s.scale}x${s.height * s.scale}`,
    type: "image/png",
    form_factor: s.form_factor,
    label: s.label,
  })), null, 2));
} catch (err) {
  console.error(`screenshots: ${err.stack ?? err.message}`);
  await page.close();
  server.close();
  process.exit(1);
}

await page.close();
server.close();
