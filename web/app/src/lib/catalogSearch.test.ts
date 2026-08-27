import { describe, expect, it } from "vitest";
import { normalizeCatalogText, reagentMatches } from "./catalogSearch";

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
