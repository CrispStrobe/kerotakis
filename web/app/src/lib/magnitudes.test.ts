import { describe, it, expect } from "vitest";
import {
  FALLBACK_MOLAR_VOLUME_L,
  INCANDESCENCE_ONSET_K,
  NORMAL_BOILING_K,
  VISIBLE_BUBBLE_MOLES,
  bubblePeriodS,
  compressedVolumeL,
  condensationFilm,
  depositParticles,
  dewPointK,
  electrodeBubbles,
  electrodePairBubbles,
  headspaceVolumeL,
  phaseKind,
  effectFromEvent,
  incandescence,
  vapourIntensity,
  vesselOf,
  type Effect,
} from "./magnitudes";

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
      effectFromEvent({ event: "gas_produced", vessel: 0, species: "O2", moles: 0.05, rate_moles_per_second: 0.002 })
    ).toMatchObject({ kind: "vent", gasProduction: { molesPerSecond: 0.002 } });
    const low = effectFromEvent({ event: "foam_changed", vessel: 0, height_cm: 3, half_life_seconds: 12 });
    const high = effectFromEvent({ event: "foam_changed", vessel: 0, height_cm: 20, half_life_seconds: 40 });
    expect(low!.kind).toBe("foam");
    expect(high!.magnitude).toBeGreaterThan(low!.magnitude);
    expect(low).toMatchObject({ durationMs: 12_000, foam: { halfLifeSeconds: 12 } });
    expect(high).toMatchObject({ durationMs: 40_000, foam: { halfLifeSeconds: 40 } });
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
      amps: 0.75,
      seconds: 120,
      coulombs: 964.85,
      electrons: 0.01,
      grams: 0.318,
      per_ion: 2,
    });
    expect(e!.kind).toBe("electrolyse");
    expect(e!.magnitude).toBeGreaterThan(0.3);
    expect(e).toMatchObject({
      durationMs: 8000,
      electrolysis: {
        species: "Cu", amps: .75, seconds: 120, coulombs: 964.85,
        electronMoles: .01, productMoles: .005, grams: .318, electronsPerIon: 2,
      },
    });
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

  it("retains actual delivered heat and the uncoupled time boundary", () => {
    const heat = effectFromEvent({
      event: "energy_transferred", vessel: 0, heating: true,
      requested_j: 5000, delivered_j: 5000, time_coupled: false,
    });
    const clampedCool = effectFromEvent({
      event: "energy_transferred", vessel: 0, heating: false,
      requested_j: 5000, delivered_j: 2500, time_coupled: false,
    });
    expect(heat).toMatchObject({
      kind: "heat", durationMs: 2600, reading: 5000, unit: "J",
      thermal: { heating: true, requestedJ: 5000, deliveredJ: 5000, timeCoupled: false },
    });
    expect(clampedCool).toMatchObject({
      kind: "cool", thermal: { heating: false, deliveredJ: 2500, timeCoupled: false },
    });
    expect(heat!.magnitude).toBeGreaterThan(clampedCool!.magnitude);
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

  it("maps dissolved → dissolve, sized by the moles that went into solution", () => {
    // GUI-099: this used to be a flat 1, so a speck and a spoonful of salt
    // dissolved with the same picture. An event without moles now reads as
    // the smallest visible amount rather than the largest.
    const e = effectFromEvent({ event: "dissolved", vessel: 0 });
    expect(e!.kind).toBe("dissolve");
    expect(e!.magnitude).toBe(0);
    expect(effectFromEvent({ event: "dissolved", vessel: 0, species: "NaCl", moles: 0.05 })!.magnitude).toBe(1);
  });

  it("maps plated → plate with magnitude 1", () => {
    const e = effectFromEvent({ event: "plated", vessel: 0 });
    expect(e!.kind).toBe("plate");
    expect(e!.magnitude).toBe(1);
  });

  it("maps an engine-confirmed transfer to a spatial pour scaled by its fraction", () => {
    const small = effectFromEvent({ event: "transferred", from: 0, to: 2, fraction: 0.1 });
    const large = effectFromEvent({ event: "transferred", from: 0, to: 2, fraction: 0.9 });
    expect(small).toMatchObject({ kind: "pour", source: 0, target: 2, acceptedTransferFraction: 0.1 });
    expect(large).toMatchObject({ acceptedTransferFraction: 0.9 });
    expect(large!.magnitude).toBeGreaterThan(small!.magnitude);
  });

  it("does not invent an accepted transfer fraction from malformed engine data", () => {
    expect(effectFromEvent({ event: "transferred", from: 0, to: 1, fraction: 2 }))
      .toMatchObject({ kind: "pour", acceptedTransferFraction: 0 });
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

  it("maps accepted spill and breakage events without inferring chemistry", () => {
    const spill = effectFromEvent({
      event: "spill_created", source: 2, fraction: .4,
      destination: { surface: "bench", zone: "react" },
    });
    expect(spill).toMatchObject({
      kind: "spill", source: 2, acceptedTransferFraction: .4,
      spill: { surface: "bench", location: "react", fraction: .4 },
    });
    const broken = effectFromEvent({
      event: "container_broken", vessel: 2, impulse_ns: 4,
      destination: { surface: "tray", tray: "catch-1" },
    });
    expect(broken).toMatchObject({
      kind: "break", source: 2,
      spill: { surface: "tray", location: "catch-1", fraction: 1 },
    });
    expect(broken!.magnitude).toBeGreaterThan(0);
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

  it("more gas is more fizz, on a logarithmic ramp", () => {
    // The ramp is logarithmic from 1 mmol to 100 mmol: doubling the gas
    // adds a fixed step rather than doubling the magnitude, so a spoon of
    // baking soda's ~0.01 mol of CO₂ (a quarter of a litre) reads as a
    // real fizz instead of two shy bubbles.
    const small = effectFromEvent({ event: "gas_evolved", vessel: 0, species: "H2", moles: 0.02 });
    const big = effectFromEvent({ event: "gas_evolved", vessel: 0, species: "H2", moles: 0.04 });
    expect(big!.magnitude).toBeGreaterThan(small!.magnitude);
    expect(big!.magnitude - small!.magnitude).toBeCloseTo(Math.log10(2) / 2, 5);
    const spoon = effectFromEvent({ event: "gas_evolved", vessel: 0, species: "CO2", moles: 0.01 });
    expect(spoon!.magnitude).toBeCloseTo(0.5, 5);
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


describe("thermal magnitudes (GUI-099)", () => {
  it("vapour intensity is monotone in the moles of vapour and bounded", () => {
    const samples = [0, 0.001, 0.01, 0.05, 0.2, 0.5, 5];
    const values = samples.map(vapourIntensity);
    for (const value of values) {
      expect(value).toBeGreaterThanOrEqual(0);
      expect(value).toBeLessThanOrEqual(1);
    }
    for (let i = 1; i < values.length; i += 1) {
      expect(values[i]!).toBeGreaterThanOrEqual(values[i - 1]!);
    }
    expect(vapourIntensity(0.5)).toBe(1);
    expect(vapourIntensity(0)).toBe(0);
  });

  it("a boil carries the engine's own plateau, not a constant", () => {
    const salted = effectFromEvent({
      event: "state_changed", vessel: 0, species: "H2O",
      from: "liquid", to: "gas", at: 374.7, shifted_by: 1.55,
    });
    expect(salted).toMatchObject({ kind: "boil", temperatureK: 374.7 });
    expect(salted!.phase).toMatchObject({ atK: 374.7, shiftedByK: 1.55, to: "gas", species: "H2O" });
    expect(salted!.temperatureK).not.toBe(NORMAL_BOILING_K);
  });

  it("a routed boil names the pressure that set it and still draws a boil", () => {
    const vacuum = effectFromEvent({
      event: "boiling_point_routed", vessel: 0, species: "H2O",
      pressure_kpa: 20, boiling: 333.2, shifted_by: -39.95,
      route: "AntoineWater", model: "Antoine (NIST)",
    });
    expect(vacuum).toMatchObject({ kind: "boil", temperatureK: 333.2 });
    expect(vacuum!.phase).toMatchObject({ pressureKpa: 20, route: "AntoineWater" });
    expect(vacuum!.magnitude).toBe(1);
  });

  it("freezing, melting and condensing each get their own kind", () => {
    const freeze = effectFromEvent({ event: "state_changed", vessel: 0, species: "H2O", from: "liquid", to: "solid", at: 271.2, shifted_by: -1.95 });
    const melt = effectFromEvent({ event: "state_changed", vessel: 0, species: "H2O", from: "solid", to: "liquid", at: 273.15, shifted_by: 0 });
    const condense = effectFromEvent({ event: "state_changed", vessel: 0, species: "H2O", from: "gas", to: "liquid", at: 373.15, shifted_by: 0 });
    expect(freeze!.kind).toBe("freeze");
    expect(melt!.kind).toBe("melt");
    expect(condense!.kind).toBe("condense");
  });

  it("a sealed vessel's contained gas is a visible effect carrying its moles", () => {
    const small = effectFromEvent({ event: "gas_contained", vessel: 0, species: "H2O", moles: 0.001 });
    const large = effectFromEvent({ event: "gas_contained", vessel: 0, species: "H2O", moles: 0.1 });
    expect(small).toMatchObject({ kind: "contain", species: "H2O", unit: "mol", reading: 0.001 });
    expect(large!.magnitude).toBeGreaterThan(small!.magnitude);
    expect(large!.magnitude).toBeLessThanOrEqual(1);
  });

  it("evolved gas names its species so a boil can tell steam from fizz", () => {
    expect(effectFromEvent({ event: "gas_evolved", vessel: 0, species: "CO2", moles: 0.02 }))
      .toMatchObject({ kind: "vent", species: "CO2", unit: "mol" });
  });

  it("precipitation and dissolution both carry their moles", () => {
    const precipitated = effectFromEvent({ event: "precipitated", vessel: 0, species: "AgCl", moles: 0.02 });
    const dissolved = effectFromEvent({ event: "dissolved", vessel: 0, species: "NaCl", moles: 0.02 });
    expect(precipitated).toMatchObject({ species: "AgCl", reading: 0.02, unit: "mol" });
    expect(dissolved).toMatchObject({ species: "NaCl", reading: 0.02, unit: "mol" });
    const bigger = effectFromEvent({ event: "dissolved", vessel: 0, species: "NaCl", moles: 0.05 });
    expect(bigger!.magnitude).toBeGreaterThan(dissolved!.magnitude);
  });
});

describe("incandescence", () => {
  it("nothing glows below the onset", () => {
    expect(incandescence(INCANDESCENCE_ONSET_K - 1)).toBeNull();
    expect(incandescence(293.15)).toBeNull();
    expect(incandescence(773)).toBeNull();
  });

  it("strength is monotone in temperature and bounded", () => {
    const values = [800, 1000, 1400, 1800, 2000, 3000, 6000].map((k) => incandescence(k)!.fraction);
    for (let i = 1; i < values.length; i += 1) {
      expect(values[i]!).toBeGreaterThanOrEqual(values[i - 1]!);
    }
    expect(values[0]).toBe(0);
    expect(values.at(-1)).toBe(1);
  });

  it("colour walks the blackbody locus: red, then amber, then white", () => {
    const dull = incandescence(900)!.rgb;
    const amber = incandescence(2000)!.rgb;
    const white = incandescence(4000)!.rgb;
    expect(dull[0]).toBe(255);
    expect(dull[1]).toBeLessThan(amber[1]);
    expect(amber[1]).toBeLessThan(white[1]);
    expect(dull[2]).toBeLessThanOrEqual(amber[2]);
    expect(amber[2]).toBeLessThan(white[2]);
    for (const rgb of [dull, amber, white]) {
      for (const channel of rgb) {
        expect(channel).toBeGreaterThanOrEqual(0);
        expect(channel).toBeLessThanOrEqual(255);
      }
    }
  });
});

describe("condensation", () => {
  it("room air at 20 °C and 50 % RH dews near 9 °C", () => {
    expect(dewPointK(293.15, 0.5) - 273.15).toBeCloseTo(9.3, 1);
  });

  it("a dew point rises with humidity", () => {
    expect(dewPointK(293.15, 0.9)).toBeGreaterThan(dewPointK(293.15, 0.3));
  });

  it("nothing beads on a wall warmer than the dew point", () => {
    expect(condensationFilm(293.15)).toBe(0);
    expect(condensationFilm(285)).toBe(0);
  });

  it("beading is monotone as the wall gets colder, and bounded", () => {
    const values = [282, 280, 278, 276, 274].map((k) => condensationFilm(k));
    for (let i = 1; i < values.length; i += 1) {
      expect(values[i]!).toBeGreaterThanOrEqual(values[i - 1]!);
    }
    for (const value of values) {
      expect(value).toBeGreaterThanOrEqual(0);
      expect(value).toBeLessThanOrEqual(1);
    }
  });

  it("below freezing the frost layer owns the water, not the droplets", () => {
    expect(condensationFilm(272)).toBe(0);
    expect(condensationFilm(250)).toBe(0);
  });
});


describe("deposits (GUI-099 ANIM-2)", () => {
  it("more moles means more grains, and never fewer", () => {
    const counts = [1e-5, 1e-4, 1e-3, 1e-2, 1e-1, 1].map((moles) => depositParticles(moles).count);
    for (let i = 1; i < counts.length; i += 1) {
      expect(counts[i]!).toBeGreaterThanOrEqual(counts[i - 1]!);
    }
    expect(counts.at(-1)).toBeLessThanOrEqual(12);
    expect(depositParticles(0).count).toBe(0);
  });

  it("a bulkier solid draws bigger grains for the same moles", () => {
    const dense = depositParticles(0.01, 0.027);
    const fluffy = depositParticles(0.01, 0.09);
    expect(dense.count).toBe(fluffy.count);
    expect(fluffy.particleVolumeL).toBeGreaterThan(dense.particleVolumeL);
    expect(fluffy.radiusScale).toBeGreaterThan(dense.radiusScale);
  });

  it("grain radius is bounded whatever the amount", () => {
    for (const moles of [1e-9, 1e-4, 1, 1000]) {
      const grains = depositParticles(moles);
      expect(grains.radiusScale).toBeGreaterThanOrEqual(0.5);
      expect(grains.radiusScale).toBeLessThanOrEqual(3);
    }
  });

  it("an unknown molar volume falls back to a documented mid-range solid", () => {
    expect(depositParticles(0.01).particleVolumeL)
      .toBeCloseTo(depositParticles(0.01, FALLBACK_MOLAR_VOLUME_L).particleVolumeL, 12);
    expect(depositParticles(0.01, 0).particleVolumeL)
      .toBeCloseTo(depositParticles(0.01, FALLBACK_MOLAR_VOLUME_L).particleVolumeL, 12);
  });

  it("the total volume drawn is the volume the engine's moles carry", () => {
    const grains = depositParticles(0.02, 0.05);
    expect(grains.count * grains.particleVolumeL).toBeCloseTo(0.02 * 0.05, 12);
  });
});

describe("headspace (GUI-099 ANIM-2)", () => {
  it("a mole of ideal gas at 273.15 K and one atmosphere is 22.4 L", () => {
    expect(headspaceVolumeL(1, 273.15, 101_325)).toBeCloseTo(22.4, 1);
  });

  it("squeezing the same gas harder shrinks it, monotonically", () => {
    const volumes = [100_000, 200_000, 400_000, 800_000].map((pa) => headspaceVolumeL(0.01, 298.15, pa));
    for (let i = 1; i < volumes.length; i += 1) {
      expect(volumes[i]!).toBeLessThan(volumes[i - 1]!);
      expect(volumes[i]!).toBeGreaterThan(0);
    }
  });

  it("more gas, or hotter gas, takes more room", () => {
    expect(headspaceVolumeL(0.02, 298.15, 101_325)).toBeGreaterThan(headspaceVolumeL(0.01, 298.15, 101_325));
    expect(headspaceVolumeL(0.01, 400, 101_325)).toBeGreaterThan(headspaceVolumeL(0.01, 298.15, 101_325));
  });

  it("nonsense inputs give no volume rather than a NaN piston", () => {
    expect(headspaceVolumeL(0, 298.15, 101_325)).toBe(0);
    expect(headspaceVolumeL(0.01, 298.15, 0)).toBe(0);
    expect(headspaceVolumeL(-1, 298.15, 101_325)).toBe(0);
  });

  it("Boyle's law is the standing fallback and agrees at the reference", () => {
    expect(compressedVolumeL(0.2, 101_325)).toBeCloseTo(0.2, 9);
    expect(compressedVolumeL(0.2, 202_650)).toBeCloseTo(0.1, 9);
    expect(compressedVolumeL(0.2, 50_662.5)).toBeCloseTo(0.4, 9);
    expect(compressedVolumeL(0, 101_325)).toBe(0);
    expect(compressedVolumeL(0.2, 0)).toBe(0);
  });

  it("a sealed vessel reports the headspace the engine named", () => {
    const sealed = effectFromEvent({ event: "vessel_sealed", vessel: 0, headspace_volume: 0.25, trapped_air: 0.0102 });
    expect(sealed).toMatchObject({ kind: "seal" });
    expect(sealed!.headspace).toMatchObject({ volumeL: 0.25, moles: 0.0102, source: "engine" });
  });

  it("an ignition carries the energy its flame is already sized by", () => {
    const big = effectFromEvent({ event: "ignited", vessel: 0, flame: "yellow", energy_j: 40_000 });
    const quiet = effectFromEvent({ event: "ignited", vessel: 0, flame: "yellow", energy_j: 200 });
    expect(big).toMatchObject({ reading: 40_000, unit: "J" });
    expect(big!.magnitude).toBeGreaterThan(quiet!.magnitude);
    // An unquantified ignition must stay a restrained fallback, not a number.
    expect(effectFromEvent({ event: "ignited", vessel: 0, flame: "yellow" })!.reading).toBeUndefined();
  });
});


describe("the invisible ones (GUI-099 ANIM-3)", () => {
  it("an emulsion carries its dispersion and lives for its own half-life", () => {
    const shaken = effectFromEvent({
      event: "emulsion_changed", vessel: 0, material: "vinaigrette",
      from_dispersed_fraction: 0, to_dispersed_fraction: 0.62,
      dispersed_volume_l: 0.004, half_life_seconds: 45,
    });
    expect(shaken).toMatchObject({ kind: "emulsify", unit: "fraction", reading: 0.62 });
    expect(shaken!.emulsion).toMatchObject({ toDispersedFraction: 0.62, halfLifeSeconds: 45 });
    // A tight emulsion is watched for longer than one that breaks at once.
    const fleeting = effectFromEvent({
      event: "emulsion_changed", vessel: 0, material: "oil",
      to_dispersed_fraction: 0.62, dispersed_volume_l: 0.004, half_life_seconds: 0.2,
    });
    expect(shaken!.durationMs!).toBeGreaterThan(fleeting!.durationMs!);
    expect(shaken!.durationMs!).toBeLessThanOrEqual(9000);
  });

  it("more dispersed means a stronger emulsion effect", () => {
    const thin = effectFromEvent({ event: "emulsion_changed", vessel: 0, to_dispersed_fraction: 0.05, half_life_seconds: 10 });
    const thick = effectFromEvent({ event: "emulsion_changed", vessel: 0, to_dispersed_fraction: 0.9, half_life_seconds: 10 });
    expect(thick!.magnitude).toBeGreaterThan(thin!.magnitude);
    expect(thick!.magnitude).toBeLessThanOrEqual(1);
  });

  it("a ferment reports the rate its bubbling is paced by", () => {
    const brew = effectFromEvent({
      event: "fermented", vessel: 0, sucrose_moles: 0.02, ethanol_moles: 0.04,
      carbon_dioxide_moles: 0.04, active_yeast_grams: 0.5, seconds: 3600,
    });
    expect(brew).toMatchObject({ kind: "ferment", unit: "mol", reading: 0.04 });
    expect(brew!.fermentation!.molesPerSecond).toBeCloseTo(0.04 / 3600, 12);
    expect(brew!.durationMs!).toBeGreaterThanOrEqual(2500);
    expect(brew!.durationMs!).toBeLessThanOrEqual(12_000);
  });

  it("a faster ferment bubbles more often, and the tempo stays watchable", () => {
    const slow = bubblePeriodS(0.04 / 3600);
    const brisk = bubblePeriodS(0.04 / 600);
    const furious = bubblePeriodS(1);
    expect(slow).toBeGreaterThan(brisk);
    expect(brisk).toBeGreaterThan(furious);
    expect(slow).toBeLessThanOrEqual(6);
    expect(furious).toBeGreaterThanOrEqual(0.25);
    expect(bubblePeriodS(0)).toBe(6);
    // One bubble a second is one visible bubble's worth of gas a second.
    expect(bubblePeriodS(VISIBLE_BUBBLE_MOLES)).toBeCloseTo(1, 6);
  });

  it("UV attenuation is strongest when the least gets through", () => {
    const blocked = effectFromEvent({
      event: "uv_attenuated", vessel: 0, material: "sunscreen",
      wavelength_nm: 308, band: "UV-B", transmitted_fraction: 0.02, mechanism: "absorption",
    });
    const clear = effectFromEvent({
      event: "uv_attenuated", vessel: 0, material: "water",
      wavelength_nm: 308, band: "UV-B", transmitted_fraction: 0.95, mechanism: "none",
    });
    expect(blocked).toMatchObject({ kind: "uv", unit: "fraction", reading: 0.02 });
    expect(blocked!.magnitude).toBeCloseTo(0.98, 6);
    expect(blocked!.magnitude).toBeGreaterThan(clear!.magnitude);
    expect(blocked!.uv).toMatchObject({ band: "UV-B", wavelengthNm: 308, transmittedFraction: 0.02 });
  });

  it("electrode bubbling follows the charge, monotonically and bounded", () => {
    const counts = [0.5, 1, 10, 100, 1000, 100_000].map(electrodeBubbles);
    for (let i = 1; i < counts.length; i += 1) {
      expect(counts[i]!).toBeGreaterThanOrEqual(counts[i - 1]!);
    }
    expect(counts[0]).toBeGreaterThanOrEqual(1);
    expect(counts.at(-1)).toBeLessThanOrEqual(8);
    expect(electrodeBubbles(0)).toBe(1);
    expect(electrodeBubbles(-5)).toBe(1);
  });

  // ------------------------------------------------ GUI-099 engine numbers

  it("hydrogen bubbles twice as heavily as oxygen once both halves are on the wire", () => {
    // The observation the water-splitting cell exists for: 2 H2 per O2. The
    // ratio is drawn only because the engine now names both electrodes.
    const split = electrodePairBubbles(900, 0.0023, 0.0047);
    expect(split.source).toBe("moles");
    expect(split.cathode).toBeGreaterThan(split.anode);
    expect(split.cathode / split.anode).toBeCloseTo(2, 0);
  });

  it("without both halves the two electrodes fall back to the charge they shared", () => {
    const shared = electrodePairBubbles(900);
    expect(shared.source).toBe("charge");
    expect(shared.anode).toBe(shared.cathode);
    expect(shared.anode).toBe(electrodeBubbles(900));
    // A half-reaction the engine could not resolve is not half a ratio.
    expect(electrodePairBubbles(900, 0, 0.0047).source).toBe("charge");
  });

  it("an electrolysis effect carries both electrodes when the engine names them", () => {
    const run = effectFromEvent({
      event: "electrolysed", vessel: 0, species: "H2",
      amps: 0.5, seconds: 1800, coulombs: 900, electrons: 0.00933,
      moles: 0.00466, grams: 0.0094, per_ion: 2,
      anode_species: "O2", anode_moles: 0.00233,
      cathode_species: "H2", cathode_moles: 0.00466,
    });
    expect(run!.electrolysis).toMatchObject({
      anodeSpecies: "O2", anodeMoles: 0.00233, cathodeSpecies: "H2", cathodeMoles: 0.00466,
    });
    // An older log leaves them undefined rather than zero, so the fallback
    // can tell "no half-reaction" from "no gas".
    const older = effectFromEvent({
      event: "electrolysed", vessel: 0, species: "Cu",
      amps: 0.5, seconds: 120, coulombs: 60, electrons: 0.000622,
      moles: 0.000311, grams: 0.0198, per_ion: 2,
    });
    expect(older!.electrolysis!.anodeMoles).toBeUndefined();
    expect(older!.electrolysis!.cathodeMoles).toBeUndefined();
  });

  it("a sublimation is not a boil, and the engine's own name decides", () => {
    const fog = effectFromEvent({
      event: "state_changed", vessel: 0, species: "CO2",
      from: "solid", to: "gas", at: 194.7, shifted_by: 0,
      kind: "sublimation", moles: 0.08,
    });
    expect(fog!.kind).toBe("sublimate");
    expect(fog!.phase).toMatchObject({ kind: "sublimation", moles: 0.08 });
    // The amount that moved sizes it now; a bigger sublimation reads bigger.
    const wisp = effectFromEvent({
      event: "state_changed", vessel: 0, species: "CO2",
      from: "solid", to: "gas", at: 194.7, shifted_by: 0,
      kind: "sublimation", moles: 0.005,
    });
    expect(fog!.magnitude).toBeGreaterThan(wisp!.magnitude);
  });

  it("phaseKind prefers the engine's name and falls back to the phases", () => {
    expect(phaseKind("liquid", "gas", "boiling")).toBe("boil");
    expect(phaseKind("solid", "gas", "sublimation")).toBe("sublimate");
    expect(phaseKind("gas", "solid", "deposition")).toBe("deposit");
    // An older log carries no name: `from` still separates the two.
    expect(phaseKind("solid", "gas")).toBe("sublimate");
    expect(phaseKind("liquid", "gas")).toBe("boil");
    expect(phaseKind("gas", "liquid")).toBe("condense");
    expect(phaseKind("solid", "liquid")).toBe("melt");
  });

  it("a boil still reads as a boil when the log predates the transition name", () => {
    const legacy = effectFromEvent({
      event: "state_changed", vessel: 0, species: "H2O",
      from: "liquid", to: "gas", at: 374.7, shifted_by: 1.55,
    });
    expect(legacy!.kind).toBe("boil");
    expect(legacy!.phase!.kind).toBeUndefined();
    expect(legacy!.phase!.moles).toBeUndefined();
  });
});
