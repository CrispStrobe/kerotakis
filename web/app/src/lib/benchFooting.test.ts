import { describe, expect, it } from "vitest";
import {
  BENCH_DECK_TOP,
  CARRYING_TOOL_FOOTING,
  footingY,
  toolCarriesVessel,
} from "./benchFooting";
import { APPARATUS_Y_MIN, Y_MIN, positionApparatus, EMPTY_BENCH_LAYOUT } from "./benchLayout";

describe("what a bench object stands on", () => {
  it("leaves bare glassware standing on its own base", () => {
    expect(footingY(127, null)).toBe(127);
    expect(footingY(123, undefined)).toBe(123);
  });

  it("puts the contact at the appliance's base when the appliance carries the glass", () => {
    expect(footingY(127, "heat")).toBe(134);
    expect(footingY(127, "stir")).toBe(134);
    expect(footingY(127, "cool")).toBe(134);
    expect(footingY(127, "bunsen")).toBe(133);
    expect(toolCarriesVessel("heat")).toBe(true);
  });

  it("does not invent a lift for a tool that never touches the bench", () => {
    // A lamp shines down onto the glass, electrodes go into it, a mortar
    // takes its contents, a burette hangs off its own stand. None of them
    // lifts the glassware, so the glass keeps its own base.
    for (const tool of ["irradiate", "electrolyse", "grind", "burette", "sweep", "regulate", "dilute"]) {
      expect(footingY(127, tool), tool).toBe(127);
      expect(toolCarriesVessel(tool), tool).toBe(false);
    }
  });

  it("never lifts the contact above the glass base", () => {
    // A carrying tool drawn shallower than a deep piece of glassware still
    // leaves the glass resting where the glass ends.
    expect(footingY(136, "heat")).toBe(136);
  });

  it("keeps everything that stands on the bench below the bench's far edge", () => {
    expect(APPARATUS_Y_MIN).toBeGreaterThanOrEqual(BENCH_DECK_TOP);
    expect(Y_MIN).toBeGreaterThanOrEqual(BENCH_DECK_TOP);
  });

  it("clamps a workstation dragged off the back of the bench onto the bench top", () => {
    const layout = positionApparatus(EMPTY_BENCH_LAYOUT, "centrifuge", 0.5, 0);
    expect(layout.apparatus.centrifuge?.y).toBeGreaterThanOrEqual(BENCH_DECK_TOP);
  });

  it("names every carrying tool with a base inside the vessel viewBox", () => {
    for (const [tool, y] of Object.entries(CARRYING_TOOL_FOOTING)) {
      expect(y, tool).toBeGreaterThan(120);
      expect(y, tool).toBeLessThanOrEqual(140);
    }
  });
});
