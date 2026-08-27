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

  it("maps precipitated with moles → precipitate + magnitude from event.moles", () => {
    const e = effectFromEvent({ event: "precipitated", vessel: 0, species: "BaSO4", moles: 0.01 });
    expect(e!.kind).toBe("precipitate");
    expect(e!.magnitude).toBeGreaterThan(0);
  });

  it("maps evaporated → evaporate + magnitude from event.moles", () => {
    const e = effectFromEvent({ event: "evaporated", vessel: 0, moles: 0.2 });
    expect(e!.kind).toBe("evaporate");
    expect(e!.magnitude).toBeGreaterThan(0.3);
    expect(e).toMatchObject({ reading: .2, unit: "mol" });
  });

  it("maps distilled → evaporate", () => {
    const e = effectFromEvent({
      event: "distilled",
      from: 0,
      to: 1,
      water: 0.5,
      ethanol: 0.1,
      at: 351.4,
      ended: 354.2,
      stages: 4,
      energy_kj: 21.5,
      azeotropic: true,
    });
    expect(e!.kind).toBe("evaporate");
    expect(e).toMatchObject({ source: 0, target: 1, operation: "distil" });
    expect(e!.magnitude).toBeGreaterThan(.3);
    expect(e!.distillation).toEqual({
      waterMoles: .5,
      ethanolMoles: .1,
      startK: 351.4,
      endK: 354.2,
      stages: 4,
      energyKj: 21.5,
      azeotropic: true,
    });
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
    expect(e).toMatchObject({
      durationMs: 2800,
      dilution: { volumeL: .25, waterMoles: 13.8 },
    });
  });

  it("keeps flame tests distinct from combustion and carries their colour", () => {
    const e = effectFromEvent({ event: "flame_test", vessel: 0, species: "Na+", colour: "yellow" });
    expect(e!.kind).toBe("flame_test");
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

  it("retains the typed result of each physical headspace gas test", () => {
    const positive = effectFromEvent({ event: "gas_tested", vessel: 0, test: "glowing_splint", positive: true, notes: "the splint relights" });
    const negative = effectFromEvent({ event: "gas_tested", vessel: 0, test: "limewater", positive: false, notes: "stays clear" });
    expect(positive).toMatchObject({
      kind: "gas_test", magnitude: .85, durationMs: 4500,
      gasTest: { test: "glowing_splint", positive: true, notes: "the splint relights" },
    });
    expect(negative).toMatchObject({
      kind: "gas_test", magnitude: .25,
      gasTest: { test: "limewater", positive: false, notes: "stays clear" },
    });
  });

  it("retains curated headspace observations for a safe physical waft", () => {
    const effect = effectFromEvent({
      event: "smelled", vessel: 0,
      notes: [["NH3", "sharp, pungent"], ["CH3COOH", "vinegar-like"]],
    });
    expect(effect).toMatchObject({
      kind: "waft", durationMs: 4200,
      waft: { notes: [
        { species: "NH3", description: "sharp, pungent" },
        { species: "CH3COOH", description: "vinegar-like" },
      ] },
    });
    expect(effect!.magnitude).toBeGreaterThan(.2);
  });

  it("retains the applied piston pressure, volume and trapped gas", () => {
    const effect = effectFromEvent({
      event: "vessel_pressure_controlled", vessel: 0,
      pressure: 250_000, initial_volume: .35, trapped_gas: .014,
    });
    expect(effect).toMatchObject({
      kind: "regulate", durationMs: 4500,
      pressureControl: { pressurePa: 250_000, initialVolumeL: .35, trappedGasMoles: .014 },
    });
    expect(effect!.magnitude).toBeGreaterThan(.3);
  });

  it("retains the applied carrier-gas sweep pressure", () => {
    const low = effectFromEvent({ event: "vessel_swept", vessel: 0, pressure: 100_000 });
    const high = effectFromEvent({ event: "vessel_swept", vessel: 0, pressure: 400_000 });
    expect(low).toMatchObject({ kind: "sweep", durationMs: 3800, sweep: { pressurePa: 100_000 } });
    expect(high!.magnitude).toBeGreaterThan(low!.magnitude);
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

  it("keeps the engine-selected lower-layer cut for a separatory funnel", () => {
    const e = effectFromEvent({ event: "drained", from: 0, to: 2, solvent: "water", moles: .35 });
    expect(e).toMatchObject({
      kind: "pour",
      source: 0,
      target: 2,
      operation: "drain",
      drain: { solvent: "water", moles: .35 },
    });
    expect(e!.magnitude).toBeGreaterThan(.1);
  });

  it("maps the engine magnetic classification to a physical two-vessel effect", () => {
    const e = effectFromEvent({
      event: "magnet_separated",
      from: 1,
      to: 3,
      attracted: ["Fe", "Ni"],
      remained: ["S"],
    });
    expect(e).toMatchObject({
      kind: "magnet",
      source: 1,
      target: 3,
      operation: "magnet",
      magnetic: { attractedSpecies: ["Fe", "Ni"], remainedSpecies: ["S"], attracted: [] },
    });
    expect(e!.magnitude).toBeGreaterThan(0);
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
    expect(fast!.stir).toMatchObject({
      rpm: 0,
      seconds: 6,
      tipSpeedMS: 2,
      resuspendedFraction: 0,
      rateCoupled: false,
    });
  });

  it("retains the complete computed centrifuge result for the machine", () => {
    const effect = effectFromEvent({
      event: "centrifuged",
      vessel: 0,
      rpm: 4200,
      seconds: 12,
      rotor_radius_m: .08,
      rcf: 1577,
      sample_mass_g: 4.2,
      counterbalance_g: 4,
      imbalance_g: .2,
      fluid_density_kg_m3: 998,
      dynamic_viscosity_pa_s: .001,
      state_coupled: false,
      separations: [{
        species: "SiO2", particle_diameter_um: 50, particle_size_assumed: true,
        particle_density_kg_m3: 2650, terminal_speed_m_s: .02,
        distance_m: .04, separated_fraction: .9, direction: "outward",
      }],
    });
    expect(effect?.centrifuge).toMatchObject({
      rpm: 4200, seconds: 12, rotorRadiusM: .08, rcf: 1577,
      imbalanceG: .2, stateCoupled: false,
      populations: [{ species: "SiO2", separatedFraction: .9, particleSizeAssumed: true }],
    });
  });

  it("scales mortar motion from computed powder surface area", () => {
    const coarse = effectFromEvent({ event: "ground", vessel: 0, surface_area_m2: 0.01 });
    const fine = effectFromEvent({ event: "ground", vessel: 0, surface_area_m2: 1 });
    expect(coarse?.kind).toBe("grind");
    expect(fine!.magnitude).toBeGreaterThan(coarse!.magnitude);
  });

  it("scales gravity settling from the strongest Stokes-law separation", () => {
    const effect = effectFromEvent({
      event: "gravity_settled",
      vessel: 0,
      seconds: 4,
      separations: [
        { species: "SiO2", particle_diameter_um: 40, terminal_speed_m_s: .002, distance_m: .008, separated_fraction: .2, direction: "settles" },
        { species: "Fe", particle_diameter_um: 120, terminal_speed_m_s: .01, distance_m: .04, separated_fraction: .85, direction: "settles" },
      ],
    });
    expect(effect).toMatchObject({ kind: "settle", magnitude: .85, durationMs: 4000 });
    expect(effect?.settling?.populations[1]).toMatchObject({ species: "Fe", distanceM: .04, separatedFraction: .85 });
  });

  it("scales centrifuge rotor motion from computed relative force", () => {
    const slow = effectFromEvent({ event: "centrifuged", vessel: 0, rcf: 20 });
    const fast = effectFromEvent({ event: "centrifuged", vessel: 0, rcf: 8000 });
    expect(slow?.kind).toBe("centrifuge");
    expect(fast!.magnitude).toBeGreaterThan(slow!.magnitude);
  });

  it("retains applied light separately from the honest photolysis boundary", () => {
    const low = effectFromEvent({
      event: "irradiated", vessel: 0, wavelength_nm: 254,
      irradiance_w_m2: 2, photolysis_coupled: false,
    });
    const high = effectFromEvent({
      event: "irradiated", vessel: 0, wavelength_nm: 365,
      irradiance_w_m2: 80, photolysis_coupled: false,
    });
    expect(low).toMatchObject({
      kind: "irradiate", durationMs: 4200,
      irradiation: { wavelengthNm: 254, irradianceWM2: 2, photolysisCoupled: false },
    });
    expect(high!.magnitude).toBeGreaterThan(low!.magnitude);
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
