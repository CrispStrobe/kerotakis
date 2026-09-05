import { describe, expect, it } from "vitest";
import { equationFromRenderedLine } from "./benchEquation";

describe("the equation inside a rendered line", () => {
  it("drops the vessel prefix the engine renders in front of it", () => {
    // The live German bench pinned `": HCO₃⁻ + CH₃COOH → …"` — a heading
    // whose title looked lost. It was the colon out of `v1: {equation}`.
    expect(equationFromRenderedLine("v1: HCO₃⁻ + CH₃COOH → CH₃COO⁻ + H₂O + CO₂↑"))
      .toBe("HCO₃⁻ + CH₃COOH → CH₃COO⁻ + H₂O + CO₂↑");
  });

  it("never returns something that starts with punctuation", () => {
    for (const line of [
      "v1: A + B → C",
      "v12: net ionic: Ag⁺ + Cl⁻ → AgCl↓",
      "2 H₂ + O₂ → 2 H₂O",
      "v3: N₂O₄ ⇌ 2 NO₂",
    ]) {
      const equation = equationFromRenderedLine(line);
      expect(equation).not.toBeNull();
      expect(equation![0]).toMatch(/[^\s:;.]/);
    }
  });

  it("keeps the equation and drops the sentence that follows it", () => {
    expect(equationFromRenderedLine("v1: CaCO₃ → CaO + CO₂↑. The gas escapes."))
      .toBe("CaCO₃ → CaO + CO₂↑");
  });

  it("takes the chemistry out of a tagged ionic line", () => {
    expect(equationFromRenderedLine("v2: net ionic: Ag⁺ + Cl⁻ → AgCl↓"))
      .toBe("Ag⁺ + Cl⁻ → AgCl↓");
  });

  it("reads an unprefixed equation unchanged", () => {
    expect(equationFromRenderedLine("2 H₂ + O₂ → 2 H₂O")).toBe("2 H₂ + O₂ → 2 H₂O");
  });

  it("refuses a line with no arrow, and a lone arrow with nothing beside it", () => {
    expect(equationFromRenderedLine("the mixture warms by 3 K")).toBeNull();
    expect(equationFromRenderedLine("v1: →")).toBeNull();
    expect(equationFromRenderedLine("v1: A →")).toBeNull();
  });
});
