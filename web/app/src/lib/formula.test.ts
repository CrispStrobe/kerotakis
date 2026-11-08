import { describe, expect, it } from "vitest";
import { isFormula, typesetFormula, typesetFormulaTokens } from "./formula";

describe("typesetFormula", () => {
  it("subscripts counts and raises charges like ionic.rs typeset", () => {
    expect(typesetFormula("H2O2")).toBe("H₂O₂");
    expect(typesetFormula("SO4-2")).toBe("SO₄²⁻");
    expect(typesetFormula("Na+")).toBe("Na⁺");
    expect(typesetFormula("NO3-")).toBe("NO₃⁻");
    expect(typesetFormula("Fe+3")).toBe("Fe³⁺");
    expect(typesetFormula("C6H8O6")).toBe("C₆H₈O₆");
    expect(typesetFormula("Ca(OH)2")).toBe("Ca(OH)₂");
    expect(typesetFormula("CO2(aq)")).toBe("CO₂(aq)");
  });

  it("keeps a hydrate's leading coefficient on the line", () => {
    expect(typesetFormula("CuSO4·5H2O")).toBe("CuSO₄·5H₂O");
  });

  it("leaves non-formulas alone", () => {
    for (const text of ["acetic acid", "3.2638 mol", "78.2 °C", "v1", "lv2", "Xy2", "2H2O", ""]) {
      expect(typesetFormula(text)).toBe(text);
    }
    expect(isFormula("v1")).toBe(false);
    expect(isFormula("Qz")).toBe(false);
  });
});

describe("typesetFormulaTokens", () => {
  it("typesets formula tokens in prose", () => {
    expect(typesetFormulaTokens("0.0033 mol CO2 formed in v1")).toBe("0.0033 mol CO₂ formed in v1");
    expect(typesetFormulaTokens("Cu+2 and SO4-2 remain")).toBe("Cu²⁺ and SO₄²⁻ remain");
    expect(typesetFormulaTokens("CO2-free air")).toBe("CO₂-free air");
  });

  it("never touches numbers, names, vessels or temperatures", () => {
    for (const text of [
      "3.2638 mol",
      "78.2 °C",
      "v1",
      "add v1 water 10mL",
      "acetic acid",
      "NaCl dissolves",
      "white_vinegar_5_percent",
      "pH 7.00",
      "1e-3 M",
    ]) {
      expect(typesetFormulaTokens(text)).toBe(text);
    }
  });
});
