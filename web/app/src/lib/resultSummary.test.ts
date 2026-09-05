import { describe, expect, it } from "vitest";
import { hasGermanTranslation } from "./i18n.svelte";
import type { Scene } from "./host/EngineHost";
import { reactantsOf, summarizeResult } from "./resultSummary";

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

/**
 * GUI-091: the class badge and the temperature both carry GUI-023's
 * confidence encoding, and the classification is absent-or-honest rather
 * than guessed. `kindConfidence` is what the card puts in
 * `data-confidence`, so it is the field that decides whether a raw event
 * tag renders as a named reaction class or as an unknown one.
 */
describe("how strongly the card stands behind its labels", () => {
  it("names the reaction class where an exact tag supplies one", () => {
    expect(summarizeResult(
      [{ event: "precipitated", vessel: 0, species: "AgCl", moles: 0.01 }],
      ["a white solid forms"], null, null,
    )).toMatchObject({ kind: "precipitation", reactionClass: "precipitation" });
  });

  it("counts a computed absence of reaction as a classification", () => {
    // "Nothing reacted" is a confident answer, not a missing one: the engine
    // evaluated the pair and found no reaction.
    expect(summarizeResult(
      [{ event: "inert", vessel: 0, species: "Cu", why: "copper is below hydrogen" }],
      ["nothing happens"], null, null,
    )?.reactionClass).toBe("no reaction");
  });

  it("leaves the class badge absent for an operation that is not a reaction", () => {
    // GUI-091's rule, and the one that is easy to get wrong: most of what a
    // bench does has no reaction class at all. Stirring is honestly
    // "mixing" — it is not an unknown reaction, it is not a reaction. The
    // card must name the operation and draw no class badge, rather than
    // promote "mixing" into a classification the engine never made.
    const summary = summarizeResult(
      [{ event: "stirred", vessel: 0, rpm: 400, seconds: 10 }],
      ["the liquid swirls"], null, null,
    );
    expect(summary?.kind).toBe("mixing");
    expect(summary?.reactionClass).toBeUndefined();
    for (const tag of ["measured", "transferred", "gravity_settled", "centrifuged"]) {
      expect([tag, summarizeResult([{ event: tag, vessel: 0 }], ["x"], null, null)?.reactionClass])
        .toEqual([tag, undefined]);
    }
  });

  it("takes the before/after pair from the engine's own event when it reports one", () => {
    // `temperature_changed` carries from/to, which is exact. Reading the
    // scenes instead would be a second, rounder answer to a question the
    // engine already answered.
    const summary = summarizeResult(
      [{ event: "temperature_changed", vessel: 0, from: 298.15, to: 363.15 }],
      ["it warms"], scene(298.15), scene(363.15),
    );
    expect(summary?.temperature).toEqual({
      beforeK: 298.15, afterK: 363.15, deltaK: 65, confidence: "computed",
    });
    // 25 °C → 90 °C, +65 K — the worked example GUI-091 names.
    expect(summary?.temperatureDeltaK).toBe(65);
  });

  it("falls back to the two scenes when no event names the temperature", () => {
    expect(summarizeResult(
      [{ event: "dissolved", vessel: 0, species: "NaCl", moles: 0.1 }],
      ["it dissolves"], scene(298.15), scene(296.15),
    )?.temperature).toEqual({
      beforeK: 298.15, afterK: 296.15, deltaK: -2, confidence: "computed",
    });
  });

  it("calls a heat of mixing modeled, because a fitted model produced it", () => {
    // UNIFAC-derived excess enthalpy: verified pair by pair, but a model
    // with fitted parameters (hmix.rs). Dashed rather than solid.
    expect(summarizeResult(
      [{ event: "heat_of_mixing", vessel: 0, joules: 1200 }],
      ["it warms as they mix"], scene(298.15), scene(301.15),
    )?.temperature?.confidence).toBe("modeled");
  });

  it("shows no temperature at all when nothing moved", () => {
    const summary = summarizeResult(
      [{ event: "stirred", vessel: 0, rpm: 400 }], ["stirred"],
      scene(298.15), scene(298.16),
    );
    expect(summary?.temperature).toBeUndefined();
    expect(summary?.temperatureDeltaK).toBeUndefined();
  });

  it("has German for every confidence word the card can render", () => {
    // Exactly the two the temperature row can produce. Listing the whole
    // Confidence enum would demand translations for words this surface
    // cannot reach, and a string nobody asked for looks exactly like a
    // string nobody translated (I18N.md).
    for (const word of ["computed", "modeled"]) {
      expect([word, hasGermanTranslation(word)]).toEqual([word, true]);
    }
  });
});

describe("the expanded card's remaining fields", () => {
  it("takes the equation from the whole command, not only the winning event", () => {
    // A curated precipitation emits `reaction_occurred` (which carries the
    // equation) beside `precipitated` (which wins the priority list and
    // does not). Reading only the winner is why the card showed no
    // equation for the commonest reaction there is.
    expect(summarizeResult([
      { event: "reaction_occurred", vessel: 0, equation: "AgNO₃ + NaCl → AgCl↓ + NaNO₃" },
      { event: "precipitated", vessel: 0, species: "AgCl", moles: 0.01 },
    ], ["a white solid forms"], null, null)).toMatchObject({
      kind: "precipitation",
      equation: "AgNO₃ + NaCl → AgCl↓ + NaNO₃",
      reactants: ["AgNO₃", "NaCl"],
    });
  });

  it("reads reactant chips off the equation's own left-hand side", () => {
    expect(reactantsOf("2 Mg + O₂ → 2 MgO")).toEqual(["Mg", "O₂"]);
    expect(reactantsOf("CaCO₃ + 2 CH₃COOH → Ca²⁺ + 2 CH₃COO⁻ + H₂O + CO₂↑"))
      .toEqual(["CaCO₃", "CH₃COOH"]);
    // A precipitate mark belongs to the equation, not to the species.
    expect(reactantsOf("AgCl↓ + Na⁺ → AgCl + Na⁺")).toEqual(["AgCl", "Na⁺"]);
    expect(reactantsOf(undefined)).toEqual([]);
    expect(reactantsOf("no arrow here")).toEqual([]);
  });

  it("carries the safety note the same command raised", () => {
    // `hazard_warning` always precedes the chemistry it warns about, so a
    // card showing the outcome without it shows half the operation.
    expect(summarizeResult([
      { event: "hazard_warning", severity: "caution", hazard: "irritant vapour", real_world: "work in a fume hood" },
      { event: "hazard_warning", severity: "danger", hazard: "chlorine gas", real_world: "this has killed people" },
      { event: "gas_evolved", vessel: 0, species: "Cl2", moles: 0.02 },
    ], ["bubbles"], null, null)?.safety).toEqual({
      severity: "danger", hazard: "chlorine gas", realWorld: "this has killed people",
    });
  });

  it("carries the concept note where an event explains rather than reports", () => {
    expect(summarizeResult(
      [{ event: "inert", vessel: 0, species: "Cu", why: "copper is below hydrogen in the reactivity series" }],
      ["nothing happens"], null, null,
    )).toMatchObject({
      kind: "no reaction",
      note: "copper is below hydrogen in the reactivity series",
    });
    expect(summarizeResult(
      [{ event: "org_reacted", vessel: 0, name: "esterification", equation: "A + B → C", boundary: "products only; no rate was computed" }],
      ["it reacts"], null, null,
    )?.note).toBe("products only; no rate was computed");
  });

  it("adds up the passes that report the same quantity about the same species", () => {
    // The German bench: the feed read "0,0090 mol CO₂ gebildet" and then
    // "0,0033 mol CO₂ gebildet" — two solver passes at one gas — while the
    // card above it showed 0,003284 mol under the label "amount". It had
    // picked one of the two events. The amount of CO₂ this command made is
    // the sum, and that is what the label promises.
    const summary = summarizeResult([
      { event: "gas_contained", vessel: 0, species: "CO2", moles: 0.008993 },
      { event: "gas_contained", vessel: 0, species: "CO2", moles: 0.003284 },
    ], ["CO₂ forms"], null, null);
    expect(summary?.kind).toBe("gas formation");
    expect(summary?.quantities[0]?.value).toBeCloseTo(0.012277, 9);
  });

  it("never adds a different species, or a different vessel, into that total", () => {
    // Two gases in one step are two answers, not one bigger one.
    const twoSpecies = summarizeResult([
      { event: "gas_contained", vessel: 0, species: "CO2", moles: 0.004 },
      { event: "gas_contained", vessel: 0, species: "H2", moles: 0.5 },
    ], ["gas forms"], null, null);
    // The priority list takes the last matching event, so H2 is the subject
    // here — and its amount is its own.
    expect(twoSpecies?.quantities[0]?.value).toBeCloseTo(0.5, 9);
    const twoVessels = summarizeResult([
      { event: "gas_contained", vessel: 1, species: "CO2", moles: 0.004 },
      { event: "gas_contained", vessel: 0, species: "CO2", moles: 0.006 },
    ], ["gas forms"], null, null);
    expect(twoVessels?.vessel).toBe(0);
    expect(twoVessels?.quantities[0]?.value).toBeCloseTo(0.006, 9);
  });

  it("leaves every optional field absent rather than empty", () => {
    const summary = summarizeResult(
      [{ event: "gas_evolved", vessel: 0, species: "CO2", moles: 0.25 }],
      ["bubbles form"], null, null,
    );
    expect(summary?.note).toBeUndefined();
    expect(summary?.safety).toBeUndefined();
    expect(summary?.equation).toBeUndefined();
    expect(summary?.reactants).toEqual([]);
  });
});
