import { describe, expect, it } from "vitest";
import { kidsExperimentMatches, parseKidsCatalog, type KidsExperiment } from "./kidsCatalog";

const apple: KidsExperiment = {
  id: "K45", title: "Stop an apple going brown", phenomenon: "Enzymatic browning",
  status: "boundary", topics: ["food", "enzymes"], ingredients: ["ascorbic_acid", "apple"],
  apparatus: ["look"], safety: "home", boundary: "Browning is not modeled.",
};

describe("kids catalog", () => {
  it("fails closed when the payload is absent or malformed", () => {
    expect(parseKidsCatalog(null)).toEqual([]);
    expect(parseKidsCatalog({ schema: 1, experiments: [{ id: "K45" }] })).toEqual([]);
    expect(parseKidsCatalog({ schema: 2, experiments: [apple] })).toEqual([]);
    expect(parseKidsCatalog({ schema: 1, experiments: [{ ...apple, safety: "unknown" }] })).toEqual([]);
  });

  it("accepts the current schema", () => {
    expect(parseKidsCatalog({ schema: 1, experiments: [apple] })).toEqual([apple]);
  });

  it("searches number, title, ingredient, apparatus and boundary text", () => {
    for (const query of ["K45", "apple", "ascorbic acid", "look", "not modeled"]) {
      expect(kidsExperimentMatches(apple, query)).toBe(true);
    }
    expect(kidsExperimentMatches(apple, "electrolysis")).toBe(false);
  });
});
