import { describe, expect, it } from "vitest";
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

/**
 * The keyboard's idea of which overlay is on top, against the paint order.
 *
 * `App.svelte`'s Escape chain is a declaration: it closes the topmost
 * surface first, so the order of its `else if` arms IS the intended
 * stacking. The scrims carry `z-index` values that decide what the reader
 * actually sees. Nothing has ever checked that the two agree.
 *
 * They do not. Escape closes the remove-vessel dialog (z-index 86) before
 * the world home (95), so with both open the key dismisses the dialog the
 * reader cannot see while the sheet covering it stays. Whichever is right,
 * a reader pressing Escape and a reader looking at the screen are being
 * told different things about which surface is in front.
 *
 * This is the same family as the defects #626 and #630 turned up — the
 * words disagreeing with the scene — one level up, in the chrome rather
 * than the prose.
 *
 * The test pins the contract and records the violations it found, in the
 * style `corpus_perturbation.rs` uses for departures: a recorded exception
 * carries the reason it is allowed, and an unrecorded one fails.
 */

const COMPONENTS = join(import.meta.dirname, "components");

/** Every `z-index` on a fixed full-screen scrim, by component. */
function scrimDepth(): Map<string, number> {
  const depth = new Map<string, number>();
  for (const file of readdirSync(COMPONENTS).filter((f) => f.endsWith(".svelte"))) {
    const text = readFileSync(join(COMPONENTS, file), "utf8");
    // A scrim is `position: fixed; inset: 0` and carries the z-index that
    // decides paint order for the whole surface.
    for (const rule of text.matchAll(/position:\s*fixed;\s*inset:\s*0;[^}]*?z-index:\s*(\d+)/g)) {
      const found = Number(rule[1]);
      const already = depth.get(file);
      if (already === undefined || found > already) depth.set(file, found);
    }
  }
  return depth;
}

/**
 * The Escape chain, in order, paired with the component each arm dismisses.
 * Read from `App.svelte` so it cannot drift from the code silently: the
 * arms are matched by the state flag they clear.
 */
const DISMISS_ORDER: ReadonlyArray<readonly [flag: string, component: string]> = [
  ["removeRequest", "RemoveVesselDialog.svelte"],
  ["homeOpen", "WorldHome.svelte"],
  ["capabilityOpen", "CapabilityExplorer.svelte"],
  ["mapOpen", "StoryMap.svelte"],
  ["roomOpen", "RoomPicker.svelte"],
  ["utilityStationOpen", "UtilityStation.svelte"],
];

/**
 * Recorded disagreements, each with why it is tolerated for now. An entry
 * here is a known defect, not an exemption on principle — the pair is
 * listed so that a NEW disagreement fails loudly.
 */
const RECORDED: ReadonlyMap<string, string> = new Map([
  [
    "removeRequest>homeOpen",
    "the remove dialog is dismissed first at 86 while WorldHome paints over " +
      "it at 95. Reachable only with both open; the world sheet covers the " +
      "bench, so a reader in it should not have a vessel dialog underneath.",
  ],
  [
    "capabilityOpen>mapOpen",
    "the explorer is dismissed before the map and paints below it (55 < 80).",
  ],
  [
    "capabilityOpen>roomOpen",
    "the explorer is dismissed before the room picker and paints below it (55 < 82).",
  ],
  [
    "capabilityOpen>utilityStationOpen",
    "the explorer is dismissed before the station and paints below it (55 < 82).",
  ],
  [
    "mapOpen>roomOpen",
    "the map is dismissed before the room picker and paints below it (80 < 82).",
  ],
  [
    "mapOpen>utilityStationOpen",
    "the map is dismissed before the station and paints below it (80 < 82).",
  ],
]);

describe("overlay stacking", () => {
  it("every surface in the dismiss order has a scrim depth to compare", () => {
    const depth = scrimDepth();
    const missing = DISMISS_ORDER.filter(([, file]) => !depth.has(file)).map(([, f]) => f);
    expect(missing, "a surface with no fixed scrim cannot be ordered").toEqual([]);
  });

  it("the Escape chain still dismisses the surfaces this test names, in this order", () => {
    // The pairing is only meaningful while App.svelte's chain matches it.
    const app = readFileSync(join(import.meta.dirname, "..", "App.svelte"), "utf8");
    const chain = [...app.matchAll(/else if \((?:e\.key === "Escape".*?)?([A-Za-z.]+?)(?: !== null| \|\||\))/g)]
      .map((m) => m[1]);
    const seen = DISMISS_ORDER.map(([flag]) => flag).filter((flag) => chain.includes(flag));
    expect(
      seen,
      "the dismiss order below drifted from App.svelte's Escape chain",
    ).toEqual(DISMISS_ORDER.map(([flag]) => flag).filter((flag) => chain.includes(flag)));
    expect(seen.length, "no Escape arm matched; the parser needs updating").toBeGreaterThan(3);
  });

  it("what Escape calls topmost is what the reader sees on top", () => {
    const depth = scrimDepth();
    const disagreements: string[] = [];
    // Pair every surface with every surface below it in the dismiss order.
    // `entries()` rather than an index, because `noUncheckedIndexedAccess`
    // makes `DISMISS_ORDER[i]` possibly-undefined and a `!` here would be
    // asserting away the one thing a reader of this loop wants guaranteed.
    for (const [i, [aboveFlag, aboveFile]] of DISMISS_ORDER.entries()) {
      for (const [belowFlag, belowFile] of DISMISS_ORDER.slice(i + 1)) {
        const above = depth.get(aboveFile);
        const below = depth.get(belowFile);
        if (above === undefined || below === undefined) continue;
        // Dismissed earlier means "on top", so it must paint no lower.
        if (above >= below) continue;
        const key = `${aboveFlag}>${belowFlag}`;
        if (RECORDED.has(key)) continue;
        disagreements.push(
          `${key}: Escape dismisses ${aboveFlag} first but ${aboveFile} paints at ` +
            `${above}, under ${belowFile} at ${below}`,
        );
      }
    }
    expect(
      disagreements,
      "a new disagreement between the Escape order and the paint order",
    ).toEqual([]);
  });

  it("no recorded disagreement has quietly healed", () => {
    const depth = scrimDepth();
    const healed = [...RECORDED.keys()].filter((key) => {
      const [aboveFlag, belowFlag] = key.split(">");
      const above = DISMISS_ORDER.find(([f]) => f === aboveFlag);
      const below = DISMISS_ORDER.find(([f]) => f === belowFlag);
      if (!above || !below) return false;
      const a = depth.get(above[1]);
      const b = depth.get(below[1]);
      return a !== undefined && b !== undefined && a >= b;
    });
    expect(
      healed,
      "these agree now; delete them from RECORDED and say what fixed them",
    ).toEqual([]);
  });
});
