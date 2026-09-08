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
const LOCALES = ["en-US", "de-DE"];

/** width/height are CSS pixels; the file is width*scale by height*scale. */
const SHOTS = [
  {
    family: "iphone-67",
    displayType: "APP_IPHONE_67",
    width: 645, height: 1398, scale: 2, mobile: true,
  },
  {
    family: "ipad-129",
    displayType: "APP_IPAD_PRO_3GEN_129",
    width: 1032, height: 1376, scale: 2, mobile: false,
  },
  {
    family: "desktop",
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

/** Wait past Chrome's initial about:blank load before using origin storage. */
const waitForCommittedDocument = async (url) => {
  const expected = new URL(url);
  const deadline = Date.now() + 30000;
  while (Date.now() < deadline) {
    try {
      const ready = await page.evaluate(`(() =>
        location.origin === ${JSON.stringify(expected.origin)}
        && location.pathname.replace(/\\/+$/, "") === ${JSON.stringify(expected.pathname.replace(/\/+$/, ""))}
        && location.search === ${JSON.stringify(expected.search)}
        && document.readyState === "complete"
      )()`);
      if (ready) return;
    } catch {
      // Runtime contexts are briefly unavailable while navigation commits.
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error(`navigation did not commit: ${url}`);
};

/** Type a line into the command bar and wait for the engine to answer. */
const run = async (line) => {
  const ready = await waitFor(page, `!!document.querySelector('form.bar input[aria-label="command"]')`,
                              { timeout: 30000 });
  if (!ready) throw new Error("command bar did not become ready");
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
  for (const locale of LOCALES) {
    for (const shot of SHOTS) {
      await page.cdp.send("Emulation.setDeviceMetricsOverride", {
        width: shot.width, height: shot.height,
        deviceScaleFactor: shot.scale, mobile: shot.mobile,
      }, page.sessionId);

      const appBase = `${origin.replace(/\/$/, "")}/app/`;
      const identity = `${locale}-${shot.family}`;
      const primeUrl = `${appBase}?appstore-shot=${encodeURIComponent(identity)}-prime`;
      const appUrl = `${appBase}?appstore-shot=${encodeURIComponent(identity)}-capture`;
      await page.goto(primeUrl);
      await waitForCommittedDocument(primeUrl);
      // Locale and console preference are set before the photographed boot,
      // exactly as returning readers have them stored on-device.
      await page.evaluate(`(() => {
        localStorage.setItem("kerotakis.console.v1", "shown");
        localStorage.setItem("kerotakis.locale", ${JSON.stringify(locale.split("-")[0])});
      })()`);
      await page.goto(appUrl);
      await waitForCommittedDocument(appUrl);
      const entered = await waitFor(page, `(() => {
        const chooser = [...document.querySelectorAll('h1, h2')]
          .some((h) => /Where do you want to work|Wo möchtest du/i.test(h.textContent || ""));
        return !chooser && !!document.querySelector('main .bench-pane')
          && !!document.querySelector('form.bar input')
          && document.documentElement.lang === ${JSON.stringify(locale.split("-")[0])};
      })()`, { timeout: 90000 });
      if (!entered) throw new Error(`${locale} ${shot.family}: the bench never appeared`);

      await waitFor(page, `(() => {
        const status = document.querySelector('.status');
        return status && !status.textContent.includes('starting') && !status.textContent.includes('startet');
      })()`, { timeout: 90000 });

      for (const line of SCRIPT) await run(line);

      const painted = await waitFor(page, `document.querySelectorAll('.bench .vessel').length > 0`,
                                    { timeout: 90000 });
      if (!painted) throw new Error(`${locale} ${shot.family}: the computed vessel scene never painted`);

    // A refused command is not an error — the bench says so calmly and
    // carries on. It is, however, the wrong frame to sell the app with.
      const refused = await page.evaluate(`(() => {
        const text = document.querySelector('.journal, aside')?.textContent ?? "";
        return /not yet available|noch nicht verfügbar|cannot|refus/i.test(text);
      })()`);
      if (refused) throw new Error(`${locale} ${shot.family}: the bench refused a command`);

      const capture = async (position, subject) => {
        const { data } = await page.cdp.send("Page.captureScreenshot",
          { format: "png", captureBeyondViewport: false }, page.sessionId);
        const name = `${locale}-${shot.family}-${position}-${subject}.png`;
        const path = join(OUT, name);
        writeFileSync(path, Buffer.from(data, "base64"));
        const px = `${shot.width * shot.scale}x${shot.height * shot.scale}`;
        made.push({ ...shot, name, locale, path, px });
        console.log(`   ${path}  ${px}  ${shot.displayType}  ${locale}`);
      };

      // Three honest views of the same computed result demonstrate the
      // product's central promise: observation, measurement, then model.
      await capture("01", "observation");
      await page.evaluate(`(() => {
        const select = document.querySelector('.dial select');
        if (!select) throw new Error('no detail-level control');
        select.value = 'lv2';
        select.dispatchEvent(new Event('change', { bubbles: true }));
        if (${shot.mobile}) document.querySelector('nav.tabs button:nth-child(3)')?.click();
      })()`);
      await waitFor(page, `document.querySelector('.dial select')?.value === 'lv2'`,
                    { timeout: 30000 });
      await new Promise((resolve) => setTimeout(resolve, 750));
      await capture("02", "measurements");

      await page.evaluate(`(() => {
        const select = document.querySelector('.dial select');
        select.value = 'lv3';
        select.dispatchEvent(new Event('change', { bubbles: true }));
      })()`);
      await waitFor(page, `document.querySelector('.dial select')?.value === 'lv3'`,
                    { timeout: 30000 });
      await new Promise((resolve) => setTimeout(resolve, 750));
      await capture("03", "model");
    }
  }

  writeFileSync(join(OUT, "manifest.json"), JSON.stringify(
    made.map(({ name, displayType, locale, px }) => ({ name, displayType, locale, pixels: px })), null, 2));
  console.log(`\n   ${made.length} shots, manifest in ${join(OUT, "manifest.json")}`);
} finally {
  await page.close?.();
  server?.close?.();
}
