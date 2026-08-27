import { describe, expect, it } from "vitest";
import {
  EMPTY_BENCH_LAYOUT,
  adjacentZone,
  apparatusRoute,
  apparatusPositionFor,
  benchLayoutFromLab,
  labWithBenchLayout,
  parseBenchLayout,
  placeNewVessel,
  placeVessel,
  placementsOverlap,
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

  it("detects overlapping object footprints without rejecting touching edges", () => {
    const centre = { x: 0.5, y: 0.5 };
    expect(placementsOverlap(centre, { x: 0.6, y: 0.62 })).toBe(true);
    expect(placementsOverlap(centre, { x: 0.64, y: 0.5 })).toBe(false);
    expect(placementsOverlap(centre, { x: 0.5, y: 0.7 })).toBe(false);
    expect(placementsOverlap(centre, { x: 0.62, y: 0.66 }, 0.1, 0.15)).toBe(false);
  });

  it("routes apparatus relationships between object edges with a lifted badge point", () => {
    const route = apparatusRoute({ x: 0.2, y: 0.62 }, { x: 0.7, y: 0.58 });
    expect(route.from).toEqual({ x: 0.255, y: 0.62 });
    expect(route.to.x).toBeCloseTo(0.655);
    expect(route.to.y).toBe(0.58);
    expect(route.control1.y).toBeLessThan(route.from.y);
    expect(route.control2.y).toBeLessThan(route.to.y);
    expect(route.midpoint.x).toBeGreaterThan(route.from.x);
    expect(route.midpoint.x).toBeLessThan(route.to.x);

    const reversed = apparatusRoute({ x: 0.8, y: 0.5 }, { x: 0.3, y: 0.7 });
    expect(reversed.from.x).toBeLessThan(0.8);
    expect(reversed.to.x).toBeGreaterThan(0.3);
  });

  it("places new glassware in a stable open slot instead of stacking it", () => {
    let layout = positionVessel(EMPTY_BENCH_LAYOUT, 0, 0.2, 0.58);
    layout = positionVessel(layout, 1, 0.4, 0.58);
    // v8's legacy default repeats v0's occupied position.
    const placed = placeNewVessel(layout, 8, [0, 1, 8]);
    expect(positionFor(placed, 8)).toEqual({ zone: "prepare", x: 0.12, y: 0.31 });
    expect(placementsOverlap(positionFor(placed, 8), positionFor(placed, 0))).toBe(false);

    const withMachine = placeNewVessel(EMPTY_BENCH_LAYOUT, 0, [0], [{ x: 0.2, y: 0.58 }]);
    expect(positionFor(withMachine, 0)).toEqual({ zone: "prepare", x: 0.12, y: 0.31 });
  });
});

describe(".lab arrangement metadata", () => {
  it("round-trips vessel and apparatus positions in an ignorable comment", () => {
    const layout = positionApparatus(
      positionVessel(EMPTY_BENCH_LAYOUT, 2, 0.72, 0.64),
      "centrifuge",
      0.18,
      0.22,
    );
    const lab = labWithBenchLayout("new tube\nadd v1 water 10mL\n", layout);
    expect(lab).toContain("# kerotakis-bench-layout-v2 ");
    expect(lab).toContain("\nnew tube\nadd v1 water 10mL\n");
    expect(benchLayoutFromLab(lab)).toEqual(layout);
  });

  it("leaves legacy and malformed files without an imported arrangement", () => {
    expect(benchLayoutFromLab("add v1 water 10mL\n")).toBeNull();
    expect(benchLayoutFromLab("# kerotakis-bench-layout-v2 nope\nnew\n")).toBeNull();
  });
});
