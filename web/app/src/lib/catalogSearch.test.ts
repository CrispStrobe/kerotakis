import { describe, expect, it } from "vitest";
import { experimentMatches, normalizeCatalogText, reagentMatches } from "./catalogSearch";

const water = { key: "water", name: "water", formula: "H2O" };

describe("reagentMatches", () => {
  it("matches substrings in the localized display name", () => {
    expect(reagentMatches(water, "Wasser", "Wasser")).toBe(true);
    expect(reagentMatches(water, "asser", "Wasser")).toBe(true);
  });

  it("keeps canonical names and formulae searchable in every locale", () => {
    expect(reagentMatches(water, "water", "Wasser")).toBe(true);
    expect(reagentMatches(water, "2o", "Wasser")).toBe(true);
  });

  it("ignores case and accents", () => {
    expect(normalizeCatalogText("LÖSCH-Kalk")).toBe("losch-kalk");
  });
});

describe("experimentMatches", () => {
  const entry = {
    id: "hydrogen-peroxide-decomposition",
    equation: "2 H2O2 -> 2 H2O + O2",
    summary: "Catalytic decomposition",
    concepts: ["reaction-rate"],
    apparatus: ["catalyst"],
    models: ["kinetics"],
    registers: { lv1: "Watch oxygen form." },
  };
  const de = (value: string) => ({
    "hydrogen peroxide decomposition": "Zersetzung von Wasserstoffperoxid",
    "reaction rate": "Reaktionsgeschwindigkeit",
    "Watch oxygen form.": "Beobachte, wie Sauerstoff entsteht.",
  })[value] ?? value;

  it("matches localized titles, concepts, and register prose", () => {
    expect(experimentMatches(entry, "Wasserstoffperoxid", de)).toBe(true);
    expect(experimentMatches(entry, "geschwindigkeit", de)).toBe(true);
    expect(experimentMatches(entry, "Sauerstoff", de)).toBe(true);
  });

  it("keeps canonical ids and formulae searchable", () => {
    expect(experimentMatches(entry, "peroxide", de)).toBe(true);
    expect(experimentMatches(entry, "H2O2", de)).toBe(true);
  });
});
