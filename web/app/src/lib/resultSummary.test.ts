import { describe, expect, it } from "vitest";
import type { Scene } from "./host/EngineHost";
import { summarizeResult } from "./resultSummary";

function scene(temperatureK: number): Scene {
  return {
    scene: 1,
    vessels: [{
      id: 0, label: "v1", liquid: null, solids: [], bubbling: false,
      boundary: "open", temperature_k: temperatureK, pressure_pa: 101325,
      elapsed_s: 0, mass_g: 1, words: "a beaker", badges: [],
    }],
  };
}

describe("computed result summary", () => {
  it("uses the structured event classification and equation", () => {
    expect(summarizeResult(
      [{ event: "precipitated", vessel: 0, species: "AgCl", moles: 0.01, equation: "Ag⁺ + Cl⁻ → AgCl", provenance: { engine: "phreeqc", dataset: "PHREEQC", model: "wateq4f.dat", dataset_sources: [], routing: "equilibrium" } }],
      ["a white solid forms"], scene(298.15), scene(300.15),
    )).toMatchObject({
      kind: "precipitation", vessel: 0, equation: "Ag⁺ + Cl⁻ → AgCl",
      observation: "a white solid forms", temperatureDeltaK: 2,
      quantities: [{ label: "amount", value: 0.01, unit: "mol" }],
      provenance: "phreeqc · PHREEQC · wateq4f.dat",
    });
  });

  it("does not invent a result when no classified event exists", () => {
    expect(summarizeResult([{ event: "hazard_warning" }], ["warning"], null, null)).toBeNull();
  });

  it("prefers a chemical outcome over bookkeeping events", () => {
    expect(summarizeResult([
      { event: "added", vessel: 0, moles: 1 },
      { event: "gas_evolved", vessel: 0, species: "CO2", moles: 0.25 },
    ], ["added", "bubbles form"], null, null)?.kind).toBe("gas evolution");
  });

  it("reports physical stirring without claiming rate coupling", () => {
    expect(summarizeResult([{
      event: "stirred", vessel: 0, rpm: 800, seconds: 20,
      resuspended_fraction: 0.75, rate_coupled: false,
    }], ["the solid is lifted"], scene(298.15), scene(298.15))).toMatchObject({
      kind: "mixing",
      boundary: "suspension changed; reaction rates are not yet coupled",
      quantities: [
        { label: "speed", value: 800, unit: "rpm" },
        { label: "duration", value: 20, unit: "s" },
        { label: "resuspended", value: 75, unit: "%" },
      ],
    });
  });
});
