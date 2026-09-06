#!/usr/bin/env node
/**
 * Photograph the real bench at the App Store's sizes.
 *
 * `gen-pwa-screenshots.mjs` does this for the PWA manifest at two
 * convenient sizes; Apple wants specific ones, and wants them per device
 * class. This is the same idea and the same honesty rule: these are
 * captures of the actual app running the actual solver, mid-experiment,
 * with the numbers its own engine produced. Nothing here is a mockup and
 * nothing is composited onto a device frame.
 *
 * The sizes are Apple's, and the enum names have drifted away from the
 * marketing ones:
 *
 *   APP_IPHONE_67            1290x2796  (6.7" — also accepts 6.9" 1320x2868)
 *   APP_IPAD_PRO_3GEN_129    2064x2752  (13" iPad Pro M4/M5)
 *   APP_DESKTOP              2880x1800  (macOS)
 *
 * Captured at deviceScaleFactor 2 from a half-size viewport, because the
 * bench lays out for CSS pixels and a 1290-wide CSS viewport would be a
 * tablet layout on a phone screenshot.
 *
 * The experiment is silver and salt, as in the PWA shots: the precipitate
 * is the one result that shows the whole stack — solver, ledger, scene —
 * working in a single frame.
 *
 * Usage:
 *   node tools/gen-appstore-screenshots.mjs <payload-dir|url> [out-dir]
 *
 * A directory is served locally; anything starting with http is used as
 * it stands, so the deployed app can be photographed without a local
 * wasm build.
 */

import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { serve, browser, waitFor } from "./lib/headless.mjs";

const SOURCE = process.argv[2];
const OUT = process.argv[3] ?? "appstore-shots";
if (!SOURCE) {
  console.error("usage: node tools/gen-appstore-screenshots.mjs <payload-dir|url> [out-dir]");
  process.exit(2);
}

const SCRIPT = ["add v1 water 200mL", "add v1 NaCl 0.1mol", "add v1 AgNO3 0.01mol"];

/** width/height are CSS pixels; the file is width*scale by height*scale. */
const SHOTS = [
  {
    name: "iphone-67-bench.png",
    displayType: "APP_IPHONE_67",
    width: 645, height: 1398, scale: 2, mobile: true,
  },
  {
    name: "ipad-129-bench.png",
    displayType: "APP_IPAD_PRO_3GEN_129",
    width: 1032, height: 1376, scale: 2, mobile: false,
  },
  {
    name: "desktop-bench.png",
    displayType: "APP_DESKTOP",
    width: 1440, height: 900, scale: 2, mobile: false,
  },
];

let server = null;
let origin = SOURCE;
if (!SOURCE.startsWith("http")) {
  ({ server, origin } = await serve(SOURCE));
}
const page = await browser();

/** Type a line into the command bar and wait for the engine to answer. */
const run = async (line) => {
  await page.evaluate(`(() => {
    const input = document.querySelector('form.bar input[aria-label="command"]');
    if (!input) throw new Error("no command bar");
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")
      .set.call(input, ${JSON.stringify(line)});
    input.dispatchEvent(new Event("input", { bubbles: true }));
    input.form.dispatchEvent(new SubmitEvent("submit", { bubbles: true, cancelable: true }));
  })()`);
  await waitFor(page, `!document.querySelector('form.bar input[aria-label="command"]').disabled`,
                { timeout: 90000 });
};

mkdirSync(OUT, { recursive: true });
const made = [];

try {
  for (const shot of SHOTS) {
    await page.cdp.send("Emulation.setDeviceMetricsOverride", {
      width: shot.width, height: shot.height,
      deviceScaleFactor: shot.scale, mobile: shot.mobile,
    }, page.sessionId);

    await page.goto(`${origin.replace(/\/$/, "")}/app/`);
    // The bench is entered directly now, and the `kero>` line is opt-in and
    // remembered — so ask for it before the app boots, the same way a
    // reader who turned it on in the utilities menu arrives.
    await page.evaluate(`localStorage.setItem("kerotakis.console.v1", "shown")`);
    await page.goto(`${origin.replace(/\/$/, "")}/app/`);
    const entered = await waitFor(page, `(() => {
      const chooser = [...document.querySelectorAll('h1, h2')]
        .some((h) => /Where do you want to work|Wo möchtest du/i.test(h.textContent || ""));
      return !chooser && !!document.querySelector('main .bench-pane')
        && !!document.querySelector('form.bar input');
    })()`, { timeout: 90000 });
    if (!entered) throw new Error(`${shot.name}: the bench never appeared`);

    await waitFor(page, `(() => {
      const status = document.querySelector('.status');
      return status && !status.textContent.includes('starting') && !status.textContent.includes('startet');
    })()`, { timeout: 90000 });

    for (const line of SCRIPT) await run(line);

    const painted = await waitFor(page, `document.querySelectorAll('.bench .vessel').length > 0`,
                                  { timeout: 90000 });
    if (!painted) throw new Error(`${shot.name}: the computed vessel scene never painted`);

    // A refused command is not an error — the bench says so calmly and
    // carries on. It is, however, the wrong frame to sell the app with.
    const refused = await page.evaluate(`(() => {
      const text = document.querySelector('.journal, aside')?.textContent ?? "";
      return /not yet available|noch nicht verfügbar|cannot|refus/i.test(text);
    })()`);
    if (refused) throw new Error(`${shot.name}: the bench refused a command`);

    const { data } = await page.cdp.send("Page.captureScreenshot",
      { format: "png", captureBeyondViewport: false }, page.sessionId);
    const path = join(OUT, shot.name);
    writeFileSync(path, Buffer.from(data, "base64"));
    const px = `${shot.width * shot.scale}x${shot.height * shot.scale}`;
    made.push({ ...shot, path, px });
    console.log(`   ${path}  ${px}  ${shot.displayType}`);
  }

  writeFileSync(join(OUT, "manifest.json"), JSON.stringify(
    made.map(({ name, displayType, px }) => ({ name, displayType, pixels: px })), null, 2));
  console.log(`\n   ${made.length} shots, manifest in ${join(OUT, "manifest.json")}`);
} finally {
  await page.close?.();
  server?.close?.();
}
