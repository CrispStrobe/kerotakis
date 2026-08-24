import { describe, expect, it } from "vitest";
import { ELEMENTS, elementsInFormula } from "./elements";

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

  it("formulas resolve to their element symbols, case-correctly", () => {
    expect(elementsInFormula("NaCl")).toEqual(["Na", "Cl"]);
    expect(elementsInFormula("Ca(OH)2")).toEqual(["Ca", "O", "H"]);
    expect(elementsInFormula("CH3COOH")).toEqual(["C", "H", "O"]);
    expect(elementsInFormula("KMnO4")).toEqual(["K", "Mn", "O"]);
    expect(elementsInFormula("CO")).toEqual(["C", "O"]); // not cobalt
    expect(elementsInFormula("CuSO4")).toEqual(["Cu", "S", "O"]);
    expect(elementsInFormula("CO3-2")).toEqual(["C", "O"]);
  });
});
