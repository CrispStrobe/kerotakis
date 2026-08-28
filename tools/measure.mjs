#!/usr/bin/env node
// Measure elements in the real page, so a layout claim is a number.
//
// "The Menge field is higher than Einheit and the zugeben button" is
// checkable, and eyeballing a screenshot is not the way to check it —
// a 4px difference is visible on a tablet and invisible in a review.
//
//   node tools/measure.mjs <site-dir> --clicks "A|B" --sel ".add-form input,.add-form select"
import { serve, browser, waitFor } from "./lib/headless.mjs";

const [site] = process.argv.slice(2).filter((a) => !a.startsWith("--"));
const arg = (name, fallback) => {
  const i = process.argv.indexOf(`--${name}`);
  return i === -1 ? fallback : process.argv[i + 1];
};
if (!site) {
  console.error("usage: node tools/measure.mjs <site-dir> [--clicks '…'] --sel '<css>'");
  process.exit(2);
}
const sel = arg("sel", "");
const locale = arg("locale", "de");


/** Wait until the DOM stops growing.
 *
 * A fixed sleep after a click is a guess, and it was wrong here: the
 * shelf loads 105 species asynchronously, so measuring 2.6s in caught 21
 * buttons and none of the reagents. Two identical counts in a row is a
 * better signal and usually faster.
 */
async function settle(page, { tries = 40, step = 250 } = {}) {
  let last = -1;
  for (let i = 0; i < tries; i++) {
    const n = Number(await page.evaluate(`document.querySelectorAll("button, a, input, select").length`));
    if (n === last && n > 0) return n;
    last = n;
    await new Promise((r) => setTimeout(r, step));
  }
  return last;
}

const { server, origin } = await serve(site);
const page = await browser();
try {
  await page.cdp.send(
    "Emulation.setDeviceMetricsOverride",
    { width: Number(arg("width", 1180)), height: Number(arg("height", 820)), deviceScaleFactor: 1, mobile: false },
    page.sessionId,
  );
  await page.goto(`${origin}/app/`);
  await page.evaluate(`localStorage.setItem("kerotakis.locale", ${JSON.stringify(locale)})`);
  await page.goto(`${origin}/app/`);
  await waitFor(page, `document.querySelectorAll("button").length > 3`, { timeout: 60000 });

  const need = arg("wait", "");
  for (const label of arg("clicks", "").split("|").map((s) => s.trim()).filter(Boolean)) {
    if (need) {
      // Before each click, not after the last one: the control being
      // clicked may itself be the thing that has not appeared yet.
      await waitFor(page, `document.querySelectorAll(${JSON.stringify(need)}).length > 0`, {
        timeout: 60000,
      }).catch(() => {});
    }
    const hit = await page.evaluate(
      `(() => {
        const wanted = ` + JSON.stringify(label) + `;
        const els = [...document.querySelectorAll("button, a, [role=button], summary")];
        const found = els.filter((b) => (b.textContent || "").trim() === wanted).pop()
          || els.filter((b) => (b.textContent || "").trim().includes(wanted)).pop();
        if (!found) return false;
        found.click();
        return true;
      })()`,
    );
    if (!hit) console.error(`  ! no control matching ${JSON.stringify(label)}`);
    await settle(page);
  }
  const rows = await page.evaluate(
    `JSON.stringify([...document.querySelectorAll(` + JSON.stringify(sel) + `)].map((el) => {
      const r = el.getBoundingClientRect();
      return {
        tag: el.tagName.toLowerCase(),
        cls: (el.className || "").toString().slice(0, 28),
        text: (el.textContent || "").trim().slice(0, 22),
        x: Math.round(r.x), y: Math.round(r.y),
        w: Math.round(r.width), h: Math.round(r.height),
        overflowsRight: r.right > (el.parentElement?.getBoundingClientRect().right ?? Infinity) + 1,
      };
    }))`,
  );
  for (const r of JSON.parse(rows)) {
    console.log(
      `${r.tag.padEnd(7)} ${String(r.w).padStart(5)}x${String(r.h).padStart(4)}` +
        ` at ${String(r.x).padStart(5)},${String(r.y).padStart(4)}` +
        `${r.overflowsRight ? "  OVERFLOWS" : "          "}  ${r.cls}  ${r.text}`,
    );
  }
} finally {
  await page.close?.();
  server.close();
}
