import { describe, expect, it } from "vitest";
import { amountUnits, suggestedAmount } from "./amounts";

describe("capacity-aware amounts", () => {
  it("uses millilitres for ordinary bench glassware", () => {
    expect(suggestedAmount("liquid", 400)).toEqual({ value: 100, unit: "mL" });
    expect(suggestedAmount("liquid", 50)).toEqual({ value: 10, unit: "mL" });
  });

  it("switches to litres for large vessels", () => {
    expect(suggestedAmount("liquid", 2000)).toEqual({ value: 2, unit: "L" });
  });

  it("offers physically meaningful phase and register choices", () => {
    expect(amountUnits("lv1", "liquid")).toContain("drop");
    expect(amountUnits("lv3", "solid")).toEqual(["g", "mol"]);
  });
});
