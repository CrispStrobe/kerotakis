import { describe, expect, it } from "vitest";
import { equationsFromEvents } from "./benchEquation";

const reaction = (equation: string) => ({ event: "reaction_occurred", vessel: 0, equation });

describe("the equations a step's events carry", () => {
  it("reads the equation off the event that carries it", () => {
    expect(equationsFromEvents([reaction("HCO₃⁻ + CH₃COOH → CH₃COO⁻ + H₂O + CO₂↑")]))
      .toEqual(["HCO₃⁻ + CH₃COOH → CH₃COO⁻ + H₂O + CO₂↑"]);
  });

  it("keeps every equation in the step, oldest first", () => {
    expect(equationsFromEvents([reaction("A → B"), { event: "observed" }, reaction("C ⇌ D")]))
      .toEqual(["A → B", "C ⇌ D"]);
  });

  /**
   * The two that were pinned on a live German bench, and the class they
   * belong to. All three carry an arrow and none is chemistry; a scrape of
   * the rendered prose caught all three, and reading the event catches
   * none — which is the whole point of the change.
   */
  it.each([
    ["the aqueous routing announcement", "solution_routed",
      "v1: Route → Kerotakis analytic equilibrium evaluator · phreeqc.dat, wie vom USGS mitgeliefert"],
    ["a temperature change", "temperature_changed", "v1: T 298,150 K → 299,356 K (ΔT = +1,206 K)"],
    ["a transfer", "transferred", "v1 → v2: 50 mL"],
  ])("is not fooled by %s", (_name, event, rendered) => {
    // The rendered text is passed as a field the function does not read, so
    // the assertion is about the EVENT kind and not about the words: a
    // sentence with an arrow in it is not an equation no matter how much it
    // looks like one.
    expect(equationsFromEvents([{ event, rendered }])).toEqual([]);
  });

  it("ignores a reaction event whose equation is half an equation", () => {
    // The engine does not write these. If it ever does, an empty rail is
    // the honest answer — a pinned fragment reads as a title that lost its
    // heading, which is the bug this rail was born from.
    expect(equationsFromEvents([reaction("A →"), reaction("→ B"), reaction("no arrow at all")]))
      .toEqual([]);
  });

  it("answers for a step with nothing in it", () => {
    expect(equationsFromEvents([])).toEqual([]);
    expect(equationsFromEvents(undefined)).toEqual([]);
    expect(equationsFromEvents(null)).toEqual([]);
    // A host that predates the field, and a malformed row.
    expect(equationsFromEvents([{ event: "reaction_occurred" }, null, "not an object"])).toEqual([]);
  });
});
