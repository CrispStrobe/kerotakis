import { describe, expect, it } from "vitest";
import type { SceneVessel } from "./host/EngineHost";
import { fieldTotal, relaxToward, step } from "./fluid";
import { fluidVisualPlan, injectAt, injectStir, layerBands, paint, simFromScene } from "./fluidScene";

const LOOKUP = (key: string) =>
  key === "hexane"
    ? { key, srgb: [235, 238, 240] as [number, number, number], density: 0.66 }
    : { key, srgb: [80, 60, 200] as [number, number, number], density: 1.0 };

const vessel = (layers: { species: string; volume_l: number }[]): SceneVessel =>
  ({
    id: 0,
    label: "beaker",
    liquid: {
      volume_l: layers.reduce((s, l) => s + l.volume_l, 0),
      srgb: [200, 200, 255],
      colour_word: "pale",
      cloudiness: 0,
      path_length_cm: 4,
    },
    layers: layers.map((l) => ({
      species: l.species,
      name: l.species,
      volume_l: l.volume_l,
      srgb: LOOKUP(l.species).srgb,
      colour_word: "x",
    })),
    solids: [],
    bubbling: false,
    boundary: "open",
    temperature_k: 298,
    pressure_pa: 101325,
    elapsed_s: 0,
    words: "",
    badges: [],
  }) as unknown as SceneVessel;

describe("the scene-to-grid bridge", () => {
  it("layer bands split rows by volume, bottom first, exactly covering", () => {
    const bands = layerBands([{ volume_l: 0.1 }, { volume_l: 0.05 }], 0, 30);
    // Bottom band (first layer) is twice the top band.
    expect(bands[0]!.bottom).toBe(29);
    const h0 = bands[0]!.bottom - bands[0]!.top + 1;
    const h1 = bands[1]!.bottom - bands[1]!.top + 1;
    expect(h0 + h1).toBe(30);
    expect(Math.abs(h0 - 2 * h1)).toBeLessThanOrEqual(1);
    expect(bands[1]!.bottom).toBe(bands[0]!.top - 1);
  });

  it("a sim seeds already settled: fields equal targets", () => {
    const sim = simFromScene(
      vessel([
        { species: "water", volume_l: 0.1 },
        { species: "hexane", volume_l: 0.05 },
      ]),
      10,
      30,
      1,
      LOOKUP,
    )!;
    expect(sim.grid.fields.length).toBe(2);
    for (let s = 0; s < 2; s++) {
      const f = sim.grid.fields[s]!;
      const t = sim.targets[s]!;
      for (let i = 0; i < f.length; i++) expect(f[i]).toBe(t[i]);
    }
  });

  it("an injection disturbs, relaxation returns to the engine's answer", () => {
    const sim = simFromScene(
      vessel([
        { species: "water", volume_l: 0.1 },
        { species: "hexane", volume_l: 0.05 },
      ]),
      10,
      30,
      1,
      LOOKUP,
    )!;
    injectAt(sim, 1, 0.5, 2);
    injectStir(sim, 1.5);
    const densities = sim.species.map((s) => s.density);
    // Phase 1 — the activity window: full dynamics + gentle relaxation.
    for (let k = 0; k < 120; k++) {
      step(sim.grid, densities, 0.05, 15, 0.93);
      for (let s = 0; s < 2; s++) relaxToward(sim.grid, s, sim.targets[s]!, 1.2, 0.05);
    }
    // Phase 2 — the freeze-out the overlay performs before crossfading
    // to the static render: motion stops, relaxation completes.
    for (let k = 0; k < 60; k++) {
      for (let s = 0; s < 2; s++) relaxToward(sim.grid, s, sim.targets[s]!, 2.5, 0.05);
    }
    // Home EXACTLY: the settled frame is the engine's answer.
    for (let s = 0; s < 2; s++) {
      const f = sim.grid.fields[s]!;
      const t = sim.targets[s]!;
      let worst = 0;
      for (let i = 0; i < f.length; i++) worst = Math.max(worst, Math.abs(f[i]! - t[i]!));
      expect(worst).toBeLessThan(0.02);
    }
  });

  it("paint mixes colours by concentration and fades with amount", () => {
    const sim = simFromScene(vessel([{ species: "water", volume_l: 0.1 }]), 4, 4, 1, LOOKUP)!;
    sim.grid.fields[0]!.fill(0);
    sim.grid.fields[0]![5] = 1; // full
    sim.grid.fields[0]![6] = 0.25; // quarter
    const out = new Uint8ClampedArray(4 * 4 * 4);
    paint(sim, out);
    expect(out[5 * 4]).toBe(80); // water srgb r
    expect(out[5 * 4 + 3]).toBeGreaterThan(out[6 * 4 + 3]!); // fades
    expect(out[0 + 3]).toBe(0); // empty is transparent
  });

  it("a vessel without layers has no sim", () => {
    const v = vessel([{ species: "water", volume_l: 0.1 }]);
    (v as { layers?: unknown[] }).layers = [];
    expect(simFromScene(v, 8, 8, 1, LOOKUP)).toBeNull();
  });

  it("keeps authoritative oil over water by bottom-first engine layer order", () => {
    const sim = simFromScene(
      vessel([
        { species: "water", volume_l: 0.1 },
        { species: "hexane", volume_l: 0.05 },
      ]), 10, 30, 1, LOOKUP,
    )!;
    const water = sim.targets[0]!;
    const oil = sim.targets[1]!;
    expect(water.slice(20 * 10).some((value) => value > 0)).toBe(true);
    expect(oil.slice(0, 10 * 10).some((value) => value > 0)).toBe(true);
    expect(oil.slice(20 * 10).some((value) => value > 0)).toBe(false);
  });

  it("makes syrup more dissipative than water from provenanced viscosity", () => {
    const water = fluidVisualPlan([{ ...LOOKUP("water"), dynamicViscosityPaS: 0.001 }], 1, false);
    const syrup = fluidVisualPlan([{ ...LOOKUP("syrup"), dynamicViscosityPaS: 2.5 }], 1, false);
    expect(syrup.damping).toBeLessThan(water.damping);
  });

  it("uses surface tension only to size conserved visual droplets", () => {
    const low = fluidVisualPlan([{ ...LOOKUP("water"), surfaceTensionNM: 0.02 }], 0.5, false);
    const high = fluidVisualPlan([{ ...LOOKUP("water"), surfaceTensionNM: 0.12 }], 0.5, false);
    expect(high.dropMass).toBeGreaterThan(low.dropMass);
    expect(high.acceptedMass).toBe(low.acceptedMass);
  });

  it("uses accepted transfer fraction as the sole pour amount authority", () => {
    expect(fluidVisualPlan([LOOKUP("water")], 0.25, false, 4).acceptedMass).toBe(1);
    expect(fluidVisualPlan([LOOKUP("water")], 0, false, 4).animate).toBe(false);
    expect(fluidVisualPlan([LOOKUP("water")], 1.1, false, 4).acceptedMass).toBe(0);
  });

  it("reduced motion settles without creating a particle animation", () => {
    const plan = fluidVisualPlan([LOOKUP("water")], 0.75, true);
    expect(plan.animate).toBe(false);
    expect(plan.acceptedMass).toBeCloseTo(1.125);
  });
});
