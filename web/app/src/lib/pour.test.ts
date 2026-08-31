import { describe, expect, it } from "vitest";
import type { SceneVessel } from "./host/EngineHost";
import { fieldTotal } from "./fluid";
import { simFromScene } from "./fluidScene";
import { inFlight, mulberry32, pourDone, startPour, stepPour } from "./pour";

const LOOKUP = (key: string) => ({
  key,
  srgb: [80, 60, 200] as [number, number, number],
  density: 1,
});

const sim = () =>
  simFromScene(
    {
      id: 0,
      label: "beaker",
      liquid: { volume_l: 0.1, srgb: [200, 200, 255], colour_word: "pale", cloudiness: 0, path_length_cm: 4 },
      layers: [{ species: "water", name: "water", volume_l: 0.1, srgb: [200, 200, 255], colour_word: "pale" }],
      solids: [],
      bubbling: false,
      boundary: "open",
      temperature_k: 298,
      pressure_pa: 101325,
      elapsed_s: 0,
      words: "",
      badges: [],
    } as unknown as SceneVessel,
    20,
    40,
    1,
    LOOKUP,
  )!;

describe("the pour hands off exactly what it emits", () => {
  it("mulberry32 is deterministic", () => {
    const a = mulberry32(7);
    const b = mulberry32(7);
    for (let i = 0; i < 10; i++) expect(a()).toBe(b());
  });

  it("THE LEDGER: emitted mass = deposited + in flight, and all lands eventually", () => {
    const v = sim();
    const rand = mulberry32(42);
    const before = fieldTotal(v.grid, 0);
    const p = startPour(0, 3.0, 0.5);
    let guard = 0;
    while (!pourDone(p) && guard++ < 2000) {
      stepPour(p, v, 0.03, rand);
      // Invariant at every tick, not just the end.
      expect(p.deposited + inFlight(p) + p.remaining).toBeCloseTo(3.0, 6);
    }
    expect(pourDone(p)).toBe(true);
    expect(p.deposited).toBeCloseTo(3.0, 6);
    expect(fieldTotal(v.grid, 0) - before).toBeCloseTo(3.0, 4);
  });

  it("droplets never rest below the surface — they convert on crossing", () => {
    const v = sim();
    const rand = mulberry32(9);
    const p = startPour(0, 1.0, 0.5);
    for (let k = 0; k < 300 && !pourDone(p); k++) {
      stepPour(p, v, 0.03, rand);
      for (const d of p.droplets) {
        // Ejecta may sit fractionally at the surface on their way out.
        expect(d.y).toBeLessThanOrEqual(v.liquidTopRow + 0.5);
      }
    }
  });

  it("a hard landing splashes: ejecta appear and then land too", () => {
    const v = sim();
    const rand = mulberry32(3);
    const p = startPour(0, 1.5, 0.5);
    let sawEjecta = false;
    let guard = 0;
    while (!pourDone(p) && guard++ < 2000) {
      stepPour(p, v, 0.03, rand);
      if (p.droplets.some((d) => d.ejecta)) sawEjecta = true;
    }
    expect(sawEjecta).toBe(true);
    expect(p.deposited).toBeCloseTo(1.5, 6);
  });

  it("mass never lands in a wall cell", () => {
    const v = sim();
    // Wall off the left third BEFORE seeding would matter: rebuild the
    // seeded fields to respect the mask, as simFromScene(solidMask) does.
    for (let y = 0; y < v.grid.h; y++) {
      for (let x = 0; x < 6; x++) {
        const i = y * v.grid.w + x;
        v.grid.solid[i] = 1;
        for (const f of v.grid.fields) f[i] = 0;
        v.targets.forEach((t) => (t[i] = 0));
      }
    }
    const rand = mulberry32(11);
    const p = startPour(0, 2.0, 0.1); // aimed at the walled side
    let guard = 0;
    while (!pourDone(p) && guard++ < 2000) stepPour(p, v, 0.03, rand);
    for (let i = 0; i < v.grid.fields[0]!.length; i++) {
      if (v.grid.solid[i]) expect(v.grid.fields[0]![i]).toBe(0);
    }
    expect(p.deposited).toBeCloseTo(2.0, 6);
  });
});
