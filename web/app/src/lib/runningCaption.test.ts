/**
 * While a script runs, the catalogue is a caption and not a lid.
 *
 * The behaviour is measured in a real browser by `tools/test-ux-quality.mjs`,
 * which is the only place that can answer what a box actually covers. This
 * test answers the other half: that the RULE is still written down. A
 * `max-height` deleted in a refactor reads as tidying up and restores the
 * exact defect it replaced — 713 px of a 900 px viewport at 200% text
 * zoom, with 100% of the stage behind it — and no unit test of behaviour
 * can see a stylesheet.
 *
 * Read out of the component's own source, in the style
 * `overlayStacking.test.ts` uses: the declaration is the contract.
 */
import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";

const SOURCE = readFileSync(join(import.meta.dirname, "components/Catalog.svelte"), "utf8");

/** The body of one CSS rule, by its selector, or "". */
function rule(selector: string): string {
  const escaped = selector.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const found = new RegExp(`(?:^|\\})\\s*${escaped}\\s*\\{([^}]*)\\}`, "m").exec(SOURCE);
  return found?.[1] ?? "";
}

describe("the running caption", () => {
  it("finds the rules at all, so nothing below is vacuous", () => {
    expect(rule(".panel.running")).not.toBe("");
    expect(rule(".dock-controls")).not.toBe("");
    expect(rule(".dock.waiting .dock-account")).not.toBe("");
  });

  it("is capped, and the cap is measured in the viewport that contains it", () => {
    const running = rule(".panel.running");
    // `max-height: none` is the value that shipped, and it is the defect.
    expect(running).not.toMatch(/max-height:\s*none/);
    const cap = /max-height:\s*([^;]+);/.exec(running)?.[1] ?? "";
    // The panel is a fixed overlay whose container IS the viewport, so the
    // bound belongs in viewport units. A rem-only cap doubles under text
    // zoom, which is the one regime where the box needed to shrink.
    expect(cap).toMatch(/\d+vh/);
  });

  it("scrolls inside itself rather than as a whole", () => {
    // A scrolling panel takes the buttons off the bottom of the caption.
    expect(rule(".panel.running")).toMatch(/overflow:\s*hidden/);
    expect(rule(".dock.waiting .dock-account")).toMatch(/overflow-y:\s*auto/);
  });

  it("never lets the controls be the thing that gives way", () => {
    // GUI-121 and GUI-122's standing lesson, a third time: what gives way
    // is the middle, never the ends. These are the press a learner repeats.
    expect(rule(".dock-controls")).toMatch(/flex:\s*none/);
  });

  it("puts no second scroller over the account's own text", () => {
    // `.dock-produced` had `max-height: 5.5rem` — 88 px at rest and 176 px
    // at 200% text zoom, a nested scroller that GREW in the regime where
    // the outer box needed it to shrink.
    expect(rule(".dock-produced")).not.toMatch(/max-height/);
  });
});
