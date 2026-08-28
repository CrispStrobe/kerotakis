/** The +/− buttons have to land on numbers a person would type.
 *
 * Amounts on this bench span about 0.001 g to 1000 mL, so a fixed step is
 * wrong at both ends: 0.5 → 1.5 overshoots what anyone meant, and
 * 100 → 101 is a tap that achieves nothing.
 */
import { describe, expect, it } from "vitest";
import { stepAmount } from "./stepAmount";

describe("stepAmount", () => {
  it("steps by a tenth, roughly, at each scale", () => {
    expect(stepAmount(1, 1)).toBe(1.1);
    expect(stepAmount(100, 1)).toBe(110);
    expect(stepAmount(0.01, 1)).toBe(0.011);
  });

  it("goes back down the way it came", () => {
    expect(stepAmount(stepAmount(50, 1), -1)).toBe(50);
    expect(stepAmount(stepAmount(2.5, 1), -1)).toBe(2.5);
  });

  it("never reaches zero, which the field rejects anyway", () => {
    expect(stepAmount(0.000001, -1)).toBeGreaterThan(0);
    expect(stepAmount(1, -1)).toBeGreaterThan(0);
  });

  it("stays on numbers a person would type", () => {
    // 1.1 rather than 1.0999999999999999 — float dust in a field the
    // reader is looking at reads as a bug in the chemistry.
    for (let v = 1, i = 0; i < 12; i++) {
      v = stepAmount(v, 1);
      expect(String(v)).not.toMatch(/\d{6,}/);
    }
  });

  it("survives a field that is empty or nonsense", () => {
    expect(stepAmount(Number.NaN, 1)).toBeGreaterThan(0);
    expect(stepAmount(0, 1)).toBeGreaterThan(0);
    expect(stepAmount(-5, 1)).toBeGreaterThan(0);
  });
});
