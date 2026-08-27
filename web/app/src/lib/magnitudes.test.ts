import { describe, it, expect } from "vitest";
import { effectFromEvent, vesselOf, type Effect } from "./magnitudes";

describe("effectFromEvent", () => {
  it("maps gas_evolved with moles → vent + magnitude from event.moles", () => {
    const e = effectFromEvent({ event: "gas_evolved", vessel: 0, species: "CO2", moles: 0.05 });
    expect(e).not.toBeNull();
    expect(e!.kind).toBe("vent");
    expect(e!.magnitude).toBeGreaterThan(0.3);
    expect(e!.magnitude).toBeLessThanOrEqual(1);
  });

  it("small gas_evolved yields small magnitude", () => {
    const e = effectFromEvent({ event: "gas_evolved", vessel: 0, species: "H2", moles: 0.001 });
    expect(e!.magnitude).toBeCloseTo(0, 1);
  });

  it("large gas_evolved clamps to 1", () => {
    const e = effectFromEvent({ event: "gas_evolved", vessel: 0, species: "H2", moles: 1.0 });
    expect(e!.magnitude).toBe(1);
  });

  it("maps produced oxygen and computed foam height to visible effects", () => {
    expect(
      effectFromEvent({ event: "gas_produced", vessel: 0, species: "O2", moles: 0.05 })
    ).toMatchObject({ kind: "vent" });
    const low = effectFromEvent({ event: "foam_changed", vessel: 0, height_cm: 3 });
    const high = effectFromEvent({ event: "foam_changed", vessel: 0, height_cm: 20 });
    expect(low!.kind).toBe("foam");
    expect(high!.magnitude).toBeGreaterThan(low!.magnitude);
  });

  it("maps computed pepper clearing to a surface-spread effect", () => {
    const e = effectFromEvent({
      event: "surface_spread",
      vessel: 0,
      from_cleared_fraction: 0,
      to_cleared_fraction: 0.9,
      coverage_fraction: 1,
    });
    expect(e?.kind).toBe("surface-spread");
    expect(e?.magnitude).toBe(1);
  });

  it("maps computed milk colour motion to a magic-milk effect", () => {
    const e = effectFromEvent({
      event: "surface_colour_spread",
      to_spread_fraction: 0.9,
    });
    expect(e?.kind).toBe("magic-milk");
    expect(e?.magnitude).toBe(1);
  });

  it("maps computed curd formation to a bounded clumping effect", () => {
    const e = effectFromEvent({
      event: "curdling_changed",
      vessel: 0,
      to_formed_fraction: 0.28,
      separation_progress: 1,
      curd_solids_mass_g: 3.75,
    });
    expect(e?.kind).toBe("curdle");
    expect(e?.magnitude).toBe(1);
  });

  it("maps precipitated with moles → precipitate + magnitude from event.moles", () => {
    const e = effectFromEvent({ event: "precipitated", vessel: 0, species: "BaSO4", moles: 0.01 });
    expect(e!.kind).toBe("precipitate");
    expect(e!.magnitude).toBeGreaterThan(0);
  });

  it("maps evaporated → evaporate + magnitude from event.moles", () => {
    const e = effectFromEvent({ event: "evaporated", vessel: 0, moles: 0.2 });
    expect(e!.kind).toBe("evaporate");
    expect(e!.magnitude).toBeGreaterThan(0.3);
  });

  it("maps distilled → evaporate", () => {
    const e = effectFromEvent({
      event: "distilled",
      from: 0,
      to: 1,
      water: 0.5,
      ethanol: 0.1,
      moles: 0.3,
    });
    expect(e!.kind).toBe("evaporate");
    expect(e).toMatchObject({ source: 0, target: 1, operation: "distil" });
  });

  it("maps electrolysed → electrolyse + magnitude from event.moles", () => {
    const e = effectFromEvent({
      event: "electrolysed",
      vessel: 0,
      species: "Cu",
      moles: 0.005,
      coulombs: 964.85,
    });
    expect(e!.kind).toBe("electrolyse");
    expect(e!.magnitude).toBeGreaterThan(0.3);
  });

  it("maps mixed → swirl + magnitude from fractions", () => {
    const e = effectFromEvent({
      event: "mixed",
      a: 0,
      b: 1,
      into: 2,
      fraction_a: 0.5,
      fraction_b: 0.5,
    });
    expect(e!.kind).toBe("swirl");
    expect(e!.magnitude).toBeGreaterThan(0.4);
  });

  it("maps diluted → swirl + magnitude from event.volume", () => {
    const e = effectFromEvent({ event: "diluted", vessel: 0, volume: 0.25, moles: 13.8 });
    expect(e!.kind).toBe("swirl");
    expect(e!.magnitude).toBeGreaterThan(0.3);
  });

  it("maps flame_test → ignite with colour from event.colour", () => {
    const e = effectFromEvent({ event: "flame_test", vessel: 0, species: "Na+", colour: "yellow" });
    expect(e!.kind).toBe("ignite");
    expect(e!.flameColour).toBe("#ffd700");
    expect(e!.magnitude).toBeLessThan(0.4);
  });

  it("maps ignited → ignite with colour and magnitude from computed energy_j", () => {
    const e = effectFromEvent({ event: "ignited", vessel: 0, flame: "blue", energy_j: 25_000 });
    expect(e!.kind).toBe("ignite");
    expect(e!.flameColour).toBe("#1e90ff");
    expect(e!.magnitude).toBeGreaterThan(0.4);
  });

  it("recognises a computed flame colour inside the engine's descriptive phrase", () => {
    const e = effectFromEvent({
      event: "ignited",
      vessel: 0,
      flame: "a blinding white",
      energy_j: 29_680,
    });
    expect(e!.flameColour).toBe("#ffffff");
  });

  it("a larger computed reaction produces a larger flame", () => {
    const small = effectFromEvent({ event: "ignited", vessel: 0, energy_j: 5_000 });
    const large = effectFromEvent({ event: "ignited", vessel: 0, energy_j: 10_000 });
    expect(large!.magnitude).toBeGreaterThan(small!.magnitude);
  });

  it("ignited without flame colour → no flameColour", () => {
    const e = effectFromEvent({ event: "ignited", vessel: 0 });
    expect(e!.kind).toBe("ignite");
    expect(e!.flameColour).toBeUndefined();
  });

  it("maps dissolved → dissolve with magnitude 1", () => {
    const e = effectFromEvent({ event: "dissolved", vessel: 0 });
    expect(e!.kind).toBe("dissolve");
    expect(e!.magnitude).toBe(1);
  });

  it("maps plated → plate with magnitude 1", () => {
    const e = effectFromEvent({ event: "plated", vessel: 0 });
    expect(e!.kind).toBe("plate");
    expect(e!.magnitude).toBe(1);
  });

  it("maps an engine-confirmed transfer to a spatial pour scaled by its fraction", () => {
    const small = effectFromEvent({ event: "transferred", from: 0, to: 2, fraction: 0.1 });
    const large = effectFromEvent({ event: "transferred", from: 0, to: 2, fraction: 0.9 });
    expect(small).toMatchObject({ kind: "pour", source: 0, target: 2 });
    expect(large!.magnitude).toBeGreaterThan(small!.magnitude);
  });

  it("uses the computed temperature delta for heating and cooling", () => {
    const warm = effectFromEvent({ event: "temperature_changed", vessel: 0, from: 293.15, to: 313.15 });
    const hot = effectFromEvent({ event: "temperature_changed", vessel: 0, from: 293.15, to: 493.15 });
    const cool = effectFromEvent({ event: "temperature_changed", vessel: 1, from: 293.15, to: 253.15 });
    expect(warm).toMatchObject({ kind: "heat", temperatureK: 313.15 });
    expect(hot!.magnitude).toBeGreaterThan(warm!.magnitude);
    expect(cool).toMatchObject({ kind: "cool", temperatureK: 253.15 });
  });

  it("only creates an explosion for the engine's typed burst event", () => {
    const justOver = effectFromEvent({ event: "burst", vessel: 0, at_pa: 210000, rating_pa: 200000 });
    const severe = effectFromEvent({ event: "burst", vessel: 0, at_pa: 400000, rating_pa: 200000 });
    expect(justOver!.kind).toBe("burst");
    expect(severe!.magnitude).toBeGreaterThan(justOver!.magnitude);
    expect(effectFromEvent({ event: "hazard_warning", severity: "danger" })).toBeNull();
  });

  it("maps a computed cell voltage to a wired two-vessel rig", () => {
    const e = effectFromEvent({ event: "cell_voltage", anode: 2, cathode: 4, volts: 1.1 });
    expect(e).toMatchObject({ kind: "connection", source: 2, target: 4, operation: "cell" });
    expect(e!.magnitude).toBeGreaterThan(0);
    expect(vesselOf({ event: "cell_voltage", anode: 2, cathode: 4 })).toBe(2);
  });

  it("returns null for unknown events", () => {
    expect(effectFromEvent({ event: "thermal_equilibrium", vessel: 0 })).toBeNull();
  });

  it("doubling moles roughly doubles the magnitude", () => {
    const small = effectFromEvent({ event: "gas_evolved", vessel: 0, species: "H2", moles: 0.02 });
    const big = effectFromEvent({ event: "gas_evolved", vessel: 0, species: "H2", moles: 0.04 });
    expect(big!.magnitude).toBeGreaterThan(small!.magnitude * 1.5);
  });

  it("scales magnetic stirring from the engine-computed bar tip speed", () => {
    const slow = effectFromEvent({ event: "stirred", vessel: 0, tip_speed_m_s: 0.1, seconds: 2 });
    const fast = effectFromEvent({ event: "stirred", vessel: 0, tip_speed_m_s: 2.0, seconds: 6 });
    expect(slow?.kind).toBe("swirl");
    expect(fast!.magnitude).toBeGreaterThan(slow!.magnitude);
    expect(slow!.durationMs).toBe(2000);
    expect(fast!.durationMs).toBe(6000);
  });

  it("scales mortar motion from computed powder surface area", () => {
    const coarse = effectFromEvent({ event: "ground", vessel: 0, surface_area_m2: 0.01 });
    const fine = effectFromEvent({ event: "ground", vessel: 0, surface_area_m2: 1 });
    expect(coarse?.kind).toBe("grind");
    expect(fine!.magnitude).toBeGreaterThan(coarse!.magnitude);
  });

  it("scales centrifuge rotor motion from computed relative force", () => {
    const slow = effectFromEvent({ event: "centrifuged", vessel: 0, rcf: 20 });
    const fast = effectFromEvent({ event: "centrifuged", vessel: 0, rcf: 8000 });
    expect(slow?.kind).toBe("centrifuge");
    expect(fast!.magnitude).toBeGreaterThan(slow!.magnitude);
  });
});

describe("vesselOf", () => {
  it("reads vessel field", () => {
    expect(vesselOf({ event: "precipitated", vessel: 2 })).toBe(2);
  });

  it("falls back to from", () => {
    expect(vesselOf({ event: "distilled", from: 1, to: 3 })).toBe(1);
  });

  it("falls back to into", () => {
    expect(vesselOf({ event: "mixed", a: 0, b: 1, into: 2 })).toBe(2);
  });

  it("defaults to 0", () => {
    expect(vesselOf({ event: "unknown" })).toBe(0);
  });
});
