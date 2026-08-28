#!/usr/bin/env node
// Screenshot the bench at a tablet viewport, so layout can be looked at
// rather than guessed at.
//
// The first layout bug I tried to fix from a written report was not a bug:
// I assumed the vessel was positioned by its left edge, and `Bench.svelte`
// already centres it. Reading a description of a screen is not the same as
// seeing it, and this is the cheap way to see it.
//
//   node tools/shot.mjs <site-dir> <out-dir> [--locale de] [--steps "…"]
//
// --steps runs commands in the bench command bar first, so a screenshot
// can show a bench with something in it rather than an empty one.
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { serve, browser, waitFor } from "./lib/headless.mjs";

const [site, outDir = "shots"] = process.argv.slice(2).filter((a) => !a.startsWith("--"));
if (!site) {
  console.error("usage: node tools/shot.mjs <site-dir> [out-dir] [--locale de] [--steps '…']");
  process.exit(2);
}
const arg = (name, fallback) => {
  const i = process.argv.indexOf(`--${name}`);
  return i === -1 ? fallback : process.argv[i + 1];
};
const locale = arg("locale", "de");
const steps = arg("steps", "");
// Text of a button to press after boot, e.g. "Sandbox betreten".

// iPad Air (3rd gen) in landscape, which is the device this was reported
// from. CSS pixels, not device pixels.
const WIDTH = Number(arg("width", 1180));
const HEIGHT = Number(arg("height", 820));

mkdirSync(outDir, { recursive: true });
const { server, origin } = await serve(site);
const page = await browser();
const shots = [];

async function shot(name) {
  const { data } = await page.cdp.send("Page.captureScreenshot", { format: "png" }, page.sessionId);
  const file = join(outDir, `${name}.png`);
  writeFileSync(file, Buffer.from(data, "base64"));
  shots.push(file);
  console.log(`  ${file}`);
}

try {
  await page.cdp.send(
    "Emulation.setDeviceMetricsOverride",
    { width: WIDTH, height: HEIGHT, deviceScaleFactor: 2, mobile: false },
    page.sessionId,
  );

  await page.goto(`${origin}/app/`);
  await page.evaluate(`localStorage.setItem("kerotakis.locale", ${JSON.stringify(locale)})`);
  // A second load so the locale is in place before the app boots, the same
  // way a returning reader arrives.
  await page.goto(`${origin}/app/`);
  await waitFor(page, `document.querySelectorAll("button").length > 3`, { timeout: 60000 });
  // The engine answers `hello` before the bench is usable; without this the
  // first screenshot catches a half-built page and every diff is noise.
  await waitFor(page, `!document.body.textContent.includes("starting")`, { timeout: 60000 }).catch(
    () => {},
  );
  await shot("01-boot");

  // Walk a path through the UI, one shot per step.
  const path = arg("clicks", arg("click", ""));
  let n = 1;
  for (const label of path.split("|").map((x) => x.trim()).filter(Boolean)) {
    const hit = await page.evaluate(
      `(() => {
        const wanted = ` + JSON.stringify(label) + `;
        const els = [...document.querySelectorAll("button, a, [role=button], summary")];
        // Last match, not first: a heading often repeats a control's words,
        // and the control is usually further down the document.
        const found = els.filter((b) => (b.textContent || "").trim().includes(wanted)).pop();
        if (!found) return false;
        found.scrollIntoView({ block: "center" });
        found.click();
        return true;
      })()`,
    );
    if (!hit) {
      console.error(`  ! no control matching ${JSON.stringify(label)} — skipped`);
      continue;
    }
    await new Promise((r) => setTimeout(r, 1800));
    const slug = label.replace(/[^\w]+/g, "-").slice(0, 28);
    await shot(`${String(++n).padStart(2, "0")}-${slug}`);
  }

  if (steps) {
    for (const line of steps.split(";").map((s) => s.trim()).filter(Boolean)) {
      await page.evaluate(`(() => {
        const input = document.querySelector(".bar input");
        if (!input) return false;
        const setter = Object.getOwnPropertyDescriptor(
          window.HTMLInputElement.prototype, "value").set;
        setter.call(input, ${JSON.stringify(line)});
        input.dispatchEvent(new Event("input", { bubbles: true }));
        input.form?.requestSubmit?.() ??
          input.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
        return true;
      })()`);
      await new Promise((r) => setTimeout(r, 1200));
    }
    await shot("02-after-steps");
  }
} finally {
  await page.close?.();
  server.close();
}
console.log(`${shots.length} screenshot(s)`);
