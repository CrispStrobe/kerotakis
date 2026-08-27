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
      [{ event: "precipitated", vessel: 0, species: "AgCl", moles: 0.01, equation: "Ag⁺ + Cl⁻ → AgCl" }],
      ["a white solid forms"], scene(298.15), scene(300.15),
    )).toMatchObject({
      kind: "precipitation", vessel: 0, equation: "Ag⁺ + Cl⁻ → AgCl",
      observation: "a white solid forms", temperatureDeltaK: 2,
      quantities: [{ label: "amount", value: 0.01, unit: "mol" }],
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
});
