import { describe, expect, it } from "vitest";
import {
  applyBuoyancy,
  computeDivergence,
  enforceBoundaries,
  fieldTotal,
  layerTarget,
  makeGrid,
  project,
  relaxToward,
  sampleCell,
  step,
} from "./fluid";

const grid = (w = 16, h = 24) => makeGrid(w, h, 1, 2);

const totalDivergence = (g: ReturnType<typeof grid>) => {
  computeDivergence(g);
  return g.divergence.reduce((s, d) => s + Math.abs(d), 0);
};

describe("the fluid core is honest math", () => {
  it("bilinear sampling interpolates and clamps", () => {
    const g = grid(4, 4);
    const f = new Float32Array(16);
    f[5] = 1; // (1,1)
    expect(sampleCell(g, f, 1, 1)).toBeCloseTo(1);
    expect(sampleCell(g, f, 1.5, 1)).toBeCloseTo(0.5);
    expect(Number.isFinite(sampleCell(g, f, -10, 99))).toBe(true);
  });

  it("a heavy species sinks, a light one rises (buoyancy sign)", () => {
    const g = grid();
    const mid = 12 * g.w + 8;
    g.fields[0]![mid] = 1; // heavy
    applyBuoyancy(g, [1.3, 0.66], 9.8, 0.1);
    // The face below the cell gained downward (+y) velocity.
    expect(g.v[13 * g.w + 8]!).toBeGreaterThan(0);

    const g2 = grid();
    g2.fields[1]![mid] = 1; // light (hexane-ish)
    applyBuoyancy(g2, [1.3, 0.66], 9.8, 0.1);
    expect(g2.v[13 * g2.w + 8]!).toBeLessThan(0);
  });

  it("projection on the MAC grid crushes divergence (no checkerboard stall)", () => {
    const g = grid();
    for (let y = 4; y < 20; y++) {
      for (let x = 2; x < 14; x++) {
        g.u[y * (g.w + 1) + x] = (x - 8) * 0.5;
        g.v[y * g.w + x] = (y - 12) * 0.5;
      }
    }
    const before = totalDivergence(g);
    project(g, 60);
    const after = totalDivergence(g);
    expect(after).toBeLessThan(before * 0.05);
  });

  it("a step conserves each species' total (nothing created or destroyed)", () => {
    const g = grid();
    for (let i = 0; i < 40; i++) g.fields[0]![6 * g.w + (2 + i) % g.w] = 0.5;
    g.fields[1]![18 * g.w + 8] = 2;
    for (let i = 0; i < g.u.length; i++) g.u[i] = 0.4;
    const t0 = fieldTotal(g, 0);
    const t1 = fieldTotal(g, 1);
    for (let k = 0; k < 10; k++) step(g, [1.2, 0.7], 0.05);
    expect(fieldTotal(g, 0)).toBeCloseTo(t0, 4);
    expect(fieldTotal(g, 1)).toBeCloseTo(t1, 4);
  });

  it("walls stay empty and no-flow through steps", () => {
    const g = grid();
    for (let y = 0; y < g.h; y++) {
      g.solid[y * g.w] = 1;
      g.solid[y * g.w + g.w - 1] = 1;
    }
    g.fields[0]![12 * g.w + 8] = 1;
    for (let i = 0; i < g.u.length; i++) g.u[i] = 1;
    enforceBoundaries(g);
    for (let k = 0; k < 5; k++) step(g, [1.1, 1], 0.1);
    for (let y = 0; y < g.h; y++) {
      expect(g.fields[0]![y * g.w]).toBe(0);
      // Both faces of a wall cell carry no flow.
      expect(g.u[y * (g.w + 1)]).toBe(0);
      expect(g.u[y * (g.w + 1) + 1]).toBe(0);
    }
  });

  it("buoyant separation emerges: light fluid rises past heavy", () => {
    // Inverted layers at RESOLVED scale (a cell-size checkerboard is
    // sub-grid — bilinear advection rightly blends it, and its mean
    // buoyancy is neutral): heavy on top, light underneath. The
    // instability must swap them — light's centre of mass ends higher.
    const g = grid(12, 30);
    for (let y = 8; y < 14; y++) {
      for (let x = 0; x < g.w; x++) g.fields[0]![y * g.w + x] = 1; // heavy, upper
    }
    for (let y = 18; y < 24; y++) {
      for (let x = 0; x < g.w; x++) g.fields[1]![y * g.w + x] = 1; // light, lower
    }
    const com = (s: number) => {
      let m = 0;
      let my = 0;
      for (let y = 0; y < g.h; y++) {
        for (let x = 0; x < g.w; x++) {
          const c = g.fields[s]![y * g.w + x]!;
          m += c;
          my += c * y;
        }
      }
      return my / m;
    };
    // The instability takes simulated time to grow (probed: crossover
    // after ~300 steps at this dt); 450 leaves margin without a slow test.
    for (let k = 0; k < 450; k++) step(g, [1.3, 0.66], 0.05, 20);
    expect(com(1)).toBeLessThan(com(0)); // smaller y = higher up
  });

  it("THE HONESTY GATE: transport relaxes to exactly the engine's layers", () => {
    const g = grid(8, 20);
    for (let i = 0; i < g.fields[0]!.length; i++) g.fields[0]![i] = 0.3;
    const target = layerTarget(g, 0, 0, 7);
    for (let k = 0; k < 200; k++) relaxToward(g, 0, target, 3, 0.05);
    for (let y = 0; y < g.h; y++) {
      for (let x = 0; x < g.w; x++) {
        const got = g.fields[0]![y * g.w + x]!;
        const want = y <= 7 ? 1 : 0;
        expect(Math.abs(got - want)).toBeLessThan(0.01);
      }
    }
  });
});
