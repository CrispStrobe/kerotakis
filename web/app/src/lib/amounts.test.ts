import { describe, expect, it } from "vitest";
import { amountUnits, defaultAmount, quickAmounts, suggestedAmount } from "./amounts";

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
    expect(amountUnits("lv3", "solid")).toEqual(["mol", "g"]);
  });
});

describe("register default units", () => {
  it("starts lv2 and lv3 at 0.1 mol for solids and liquids", () => {
    for (const register of ["lv2", "lv3"]) {
      for (const phase of ["solid", "liquid"]) {
        expect(suggestedAmount(phase, 400, register)).toEqual({ value: 0.1, unit: "mol" });
        expect(defaultAmount(register, phase)).toBe("0.1mol");
        expect(amountUnits(register, phase)[0]).toBe("mol");
        expect(quickAmounts(register, phase)[0]).toBe("0.1mol");
      }
    }
  });

  it("keeps kitchen units at lv1", () => {
    expect(suggestedAmount("solid", 400, "lv1")).toEqual({ value: 1, unit: "g" });
    expect(suggestedAmount("liquid", 400, "lv1")).toEqual({ value: 100, unit: "mL" });
    expect(defaultAmount("lv1", "solid")).toBe("1pinch");
    expect(defaultAmount("lv1", "liquid")).toBe("1cup");
    expect(amountUnits("lv1", "solid")[0]).toBe("g");
    expect(amountUnits("lv1", "liquid")[0]).toBe("mL");
  });

  it("offers every suggested unit in the register's selector", () => {
    for (const register of ["lv1", "lv2", "lv3"]) {
      for (const phase of ["solid", "liquid"]) {
        expect(amountUnits(register, phase)).toContain(suggestedAmount(phase, 400, register).unit);
      }
    }
  });
});
