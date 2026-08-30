import { describe, expect, it } from "vitest";
import {
  ELEMENTS,
  LAB_ELEMENTS,
  contentRoutesForElement,
  elementCapability,
  elementsInFormula,
  elementsMatchingSearch,
  parseElementCoverage,
  shelfItemsContainingElement,
} from "./elements";

describe("the elements", () => {
  it("all 118, in order, with sane structure", () => {
    expect(ELEMENTS).toHaveLength(118);
    expect(ELEMENTS.map((e) => e.z)).toEqual([...Array(118)].map((_, i) => i + 1));
    for (const e of ELEMENTS) {
      expect(e.group).toBeGreaterThanOrEqual(1);
      expect(e.group).toBeLessThanOrEqual(18);
      expect(e.period).toBeGreaterThanOrEqual(1);
      expect(e.period).toBeLessThanOrEqual(7);
    }
    expect(ELEMENTS.find((e) => e.symbol === "Fe")!.name).toBe("Iron");
    expect(ELEMENTS.find((e) => e.symbol === "Og")!.z).toBe(118);
  });

  it("keeps the default table useful rather than merely block-shaped", () => {
    const symbols = LAB_ELEMENTS.map((element) => element.symbol);
    expect(symbols).toEqual(expect.arrayContaining(["Mn", "Fe", "Cu", "Zn"]));
    expect(["Po", "At", "Fr", "Ra", "Tc", "Pm", ...ELEMENTS.filter((e) => e.z >= 93).map((e) => e.symbol)]
      .every((symbol) => !symbols.includes(symbol))).toBe(true);
    expect(new Set(symbols).size).toBe(symbols.length);
    expect(LAB_ELEMENTS.length).toBeLessThan(ELEMENTS.length / 2);
  });

  it("formulas resolve to their element symbols, case-correctly", () => {
    expect(elementsInFormula("NaCl")).toEqual(["Na", "Cl"]);
    expect(elementsInFormula("Ca(OH)2")).toEqual(["Ca", "O", "H"]);
    expect(elementsInFormula("CH3COOH")).toEqual(["C", "H", "O"]);
    expect(elementsInFormula("KMnO4")).toEqual(["K", "Mn", "O"]);
    expect(elementsInFormula("CO")).toEqual(["C", "O"]); // not cobalt
    expect(elementsInFormula("CuSO4")).toEqual(["Cu", "S", "O"]);
    expect(elementsInFormula("CO3-2")).toEqual(["C", "O"]);
  });

  it("derives element coverage from actual shelf formulas", () => {
    const shelf = [
      { key: "water", formula: "H2O" },
      { key: "salt", formula: "NaCl" },
      { key: "copper-sulfate", formula: "CuSO4" },
    ];
    expect(shelfItemsContainingElement("O", shelf).map((item) => item.key)).toEqual([
      "water",
      "copper-sulfate",
    ]);
    expect(shelfItemsContainingElement("Xe", shelf)).toEqual([]);
  });

  it("derives runnable lesson and experiment links from their executable sources", () => {
    const shelf = [
      { key: "water", formula: "H2O" },
      { key: "CuSO4", formula: "CuSO4" },
      { key: "Zn", formula: "Zn" },
    ];
    const routes = contentRoutesForElement(
      "Cu",
      shelf,
      [{ file: "electrode.lab", name: "electrode", kit: ["CuSO4", "Zn", "water"] }],
      [{ id: "blue-copper", summary: "Blue copper", setup: { script: "add v1 water 1mL\nadd v1 CuSO4 1g" } }],
    );
    expect(routes.map((route) => [route.kind, route.key])).toEqual([
      ["experiment", "blue-copper"],
      ["lesson", "electrode.lab"],
    ]);
    expect(routes.every((route) => route.requiredShelfKeys.every((key) => shelf.some((s) => s.key === key)))).toBe(true);
    expect(elementCapability([], routes)).toBe("lesson_backed");
  });

  it("searches symbol, localized name, formula, and common material name", () => {
    const shelf = [{ name: "blue stone", formula: "CuSO4" }];
    const localize = (value: string) => value === "Iron" ? "Eisen" : value;
    expect(elementsMatchingSearch("Fe", ELEMENTS, shelf, localize).map((e) => e.symbol)).toContain("Fe");
    expect(elementsMatchingSearch("eisen", ELEMENTS, shelf, localize).map((e) => e.symbol)).toEqual(["Fe"]);
    expect(elementsMatchingSearch("CuSO4", ELEMENTS, shelf).map((e) => e.symbol)).toEqual(["O", "S", "Cu"]);
    expect(elementsMatchingSearch("blue stone", ELEMENTS, shelf).map((e) => e.symbol)).toEqual(["O", "S", "Cu"]);
  });

  it("accepts only the complete versioned generated coverage contract", () => {
    const elements = ELEMENTS.map((element) => ({
      symbol: element.symbol,
      capability: "identity_only",
      examples: [],
      routes: [],
    }));
    expect(parseElementCoverage({ schema: 1, elements })?.elements).toHaveLength(118);
    expect(parseElementCoverage({ schema: 2, elements })).toBeNull();
    expect(parseElementCoverage({ schema: 1, elements: elements.slice(1) })).toBeNull();
  });
});
