import { describe, expect, it } from "vitest";
import type { SceneVessel } from "./host/EngineHost";
import { KINDS } from "./glassware";
import { maskFor } from "./glassMask";
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
    const full = sim.liquidTopRow * sim.grid.w + 1;
    const quarter = full + 1;
    sim.grid.fields[0]![full] = 1;
    sim.grid.fields[0]![quarter] = 0.25;
    const out = new Uint8ClampedArray(4 * 4 * 4);
    paint(sim, out);
    expect(out[full * 4]).toBe(80); // water srgb r
    expect(out[full * 4 + 3]).toBeGreaterThan(out[quarter * 4 + 3]!); // fades
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
    const rows = (field: Float32Array) => [...field.entries()]
      .filter(([, amount]) => amount > 0).map(([index]) => Math.floor(index / sim.grid.w));
    const waterRows = rows(water);
    const oilRows = rows(oil);
    expect(waterRows.length).toBeGreaterThan(0);
    expect(oilRows.length).toBeGreaterThan(0);
    expect(Math.min(...waterRows)).toBeGreaterThan(Math.max(...oilRows));
    expect(Math.min(...oilRows)).toBeGreaterThan(0);
    expect(Math.max(...waterRows)).toBeLessThan(sim.grid.h - 1);
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


describe("the fluid overlay respects accepted scene bounds and colour", () => {
  it("100 mL in a 400 mL beaker leaves the headspace and vessel bottom unpainted", () => {
    const sim = simFromScene(vessel([{ species: "water", volume_l: 0.1 }]), 50, 70, 2, LOOKUP)!;
    const out = new Uint8ClampedArray(50 * 70 * 4);
    paint(sim, out);
    const painted = [...Array(70).keys()].filter((y) => out[(y * 50 + 25) * 4 + 3]! > 0);
    expect(painted.length).toBeGreaterThan(0);
    // SVG reference: bottom=127, fill height=89*(0.1/0.4)=22.25.
    for (const y of painted) {
      expect((y + 0.5) * 2).toBeGreaterThanOrEqual(104.75);
      expect((y + 0.5) * 2).toBeLessThan(127);
    }
    expect(out[(30 * 50 + 25) * 4 + 3]).toBe(0);
    expect(out[(69 * 50 + 25) * 4 + 3]).toBe(0);
  });

  it.each(Object.keys(KINDS))("keeps %s geometry masked at several liquid levels", (kind) => {
    const geometry = KINDS[kind]!;
    for (const fraction of [0.1, 0.25, 0.5, 1]) {
      const v = vessel([{ species: "water", volume_l: geometry.fullAtL * fraction }]);
      v.label = kind;
      const mask = maskFor(geometry.inner, 50, 70);
      const original = [...mask];
      const sim = simFromScene(v, 50, 70, 2, LOOKUP, mask)!;
      const out = new Uint8ClampedArray(50 * 70 * 4);
      paint(sim, out);
      expect([...mask]).toEqual(original); // cached shape masks remain reusable
      expect(out.some((channel, index) => index % 4 === 3 && channel > 0)).toBe(true);
      for (let i = 0; i < mask.length; i++) {
        if (mask[i] || Math.floor(i / 50) < sim.liquidTopRow || Math.floor(i / 50) > sim.liquidBottomRow) {
          expect(out[i * 4 + 3]).toBe(0);
        }
      }
    }
  });

  it("increasing the accepted volume raises the surface without moving the vessel bottom", () => {
    const small = simFromScene(vessel([{ species: "water", volume_l: 0.1 }]), 50, 70, 2, LOOKUP)!;
    const large = simFromScene(vessel([{ species: "water", volume_l: 0.3 }]), 50, 70, 2, LOOKUP)!;
    expect(large.liquidTopRow).toBeLessThan(small.liquidTopRow);
    expect(large.liquidBottomRow).toBe(small.liquidBottomRow);
  });

  it("uses the computed mixed-layer colour even when no shelf species is called solution", () => {
    const v = vessel([{ species: "solution", volume_l: 0.1 }]);
    v.layers![0]!.srgb = [20, 167, 64];
    const sim = simFromScene(v, 50, 70, 2, (key) => ({ key, srgb: [0, 0, 0], density: 1.04 }))!;
    expect(sim.species[0]!.srgb).toEqual([20, 167, 64]);
    expect(sim.species[0]!.density).toBe(1.04);
    const out = new Uint8ClampedArray(50 * 70 * 4);
    paint(sim, out);
    const pixel = (sim.liquidTopRow * 50 + 25) * 4;
    expect([...out.slice(pixel, pixel + 3)]).toEqual([20, 167, 64]);
  });

  it("uses separate engine colours for immiscible layers", () => {
    const v = vessel([{ species: "water", volume_l: 0.1 }, { species: "hexane", volume_l: 0.1 }]);
    v.layers![0]!.srgb = [12, 150, 70];
    v.layers![1]!.srgb = [210, 100, 20];
    const sim = simFromScene(v, 50, 70, 2, LOOKUP)!;
    expect(sim.species.map((item) => item.srgb)).toEqual([[12, 150, 70], [210, 100, 20]]);
  });

  it("an empty or invalid liquid and a fully blocked domain produce no animation", () => {
    for (const volume of [0, -1, NaN, Infinity]) {
      expect(simFromScene(vessel([{ species: "water", volume_l: volume }]), 50, 70, 2, LOOKUP)).toBeNull();
    }
    expect(simFromScene(vessel([{ species: "water", volume_l: 0.1 }]), 50, 70, 2, LOOKUP,
      new Uint8Array(50 * 70).fill(1))).toBeNull();
  });
});
