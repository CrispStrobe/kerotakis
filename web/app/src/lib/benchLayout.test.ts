import { describe, expect, it } from "vitest";
import {
  EMPTY_BENCH_LAYOUT,
  adjacentZone,
  parseBenchLayout,
  placeVessel,
  zoneFor,
} from "./benchLayout";

describe("bench layout", () => {
  it("places new vessels in the reaction zone and moves immutably", () => {
    expect(zoneFor(EMPTY_BENCH_LAYOUT, 3)).toBe("react");
    const moved = placeVessel(EMPTY_BENCH_LAYOUT, 3, "analyse");
    expect(zoneFor(moved, 3)).toBe("analyse");
    expect(EMPTY_BENCH_LAYOUT.placements).toEqual({});
  });

  it("loads only versioned, valid placements", () => {
    expect(parseBenchLayout('{"version":1,"placements":{"0":"prepare","2":"analyse","x":"react","4":"moon"}}')).toEqual({
      version: 1,
      placements: { 0: "prepare", 2: "analyse" },
    });
    expect(parseBenchLayout("broken")).toEqual(EMPTY_BENCH_LAYOUT);
    expect(parseBenchLayout('{"version":2,"placements":{"0":"prepare"}}')).toEqual(EMPTY_BENCH_LAYOUT);
  });

  it("clamps keyboard movement at the outer zones", () => {
    expect(adjacentZone("prepare", -1)).toBe("prepare");
    expect(adjacentZone("prepare", 1)).toBe("react");
    expect(adjacentZone("react", 1)).toBe("analyse");
    expect(adjacentZone("analyse", 1)).toBe("analyse");
  });
});
