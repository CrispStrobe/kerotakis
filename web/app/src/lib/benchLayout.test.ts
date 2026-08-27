import { describe, expect, it } from "vitest";
import {
  EMPTY_BENCH_LAYOUT,
  adjacentZone,
  apparatusPositionFor,
  parseBenchLayout,
  placeVessel,
  positionFor,
  positionApparatus,
  positionVessel,
  zoneFor,
} from "./benchLayout";

describe("bench layout", () => {
  it("spreads new vessels across the surface and moves immutably", () => {
    expect(zoneFor(EMPTY_BENCH_LAYOUT, 3)).toBe("analyse");
    const moved = placeVessel(EMPTY_BENCH_LAYOUT, 3, "prepare");
    expect(zoneFor(moved, 3)).toBe("prepare");
    expect(positionFor(moved, 3).x).toBeCloseTo(1 / 6);
    expect(EMPTY_BENCH_LAYOUT.placements).toEqual({});
  });

  it("migrates zone-only saves to coordinates", () => {
    expect(parseBenchLayout('{"version":1,"placements":{"0":"prepare","2":"analyse","x":"react","4":"moon"}}')).toEqual({
      version: 2,
      placements: {
        0: { zone: "prepare", x: 1 / 6, y: 0.58 },
        2: { zone: "analyse", x: 5 / 6, y: 0.58 },
      },
      apparatus: {},
    });
    expect(parseBenchLayout("broken")).toEqual(EMPTY_BENCH_LAYOUT);
  });

  it("loads, clamps and derives hints from coordinate placement", () => {
    expect(parseBenchLayout('{"version":2,"placements":{"0":{"zone":"analyse","x":-4,"y":9}}}')).toEqual({
      version: 2,
      placements: { 0: { zone: "prepare", x: 0.08, y: 0.84 } },
      apparatus: {},
    });
    const moved = positionVessel(EMPTY_BENCH_LAYOUT, 1, 0.74, 0.41);
    expect(positionFor(moved, 1)).toEqual({ zone: "analyse", x: 0.74, y: 0.41 });
  });

  it("persists independently positioned apparatus and clamps it to the surface", () => {
    const fallback = { zone: "react" as const, x: 0.5, y: 0.18 };
    expect(apparatusPositionFor(EMPTY_BENCH_LAYOUT, "grind", fallback)).toBe(fallback);
    const moved = positionApparatus(EMPTY_BENCH_LAYOUT, "grind", 2, -3);
    expect(apparatusPositionFor(moved, "grind", fallback)).toEqual({ zone: "analyse", x: 0.92, y: 0.12 });
    expect(parseBenchLayout(JSON.stringify(moved))).toEqual(moved);
  });

  it("clamps keyboard movement at the outer zones", () => {
    expect(adjacentZone("prepare", -1)).toBe("prepare");
    expect(adjacentZone("prepare", 1)).toBe("react");
    expect(adjacentZone("react", 1)).toBe("analyse");
    expect(adjacentZone("analyse", 1)).toBe("analyse");
  });
});
