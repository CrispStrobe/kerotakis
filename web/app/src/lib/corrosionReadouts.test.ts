import { describe, expect, it } from "vitest";
import { corrosionReadouts } from "./corrosionReadouts";

const row = (fraction: number) => ({
  metal: "Fe",
  corroding: true,
  metal_in_oxide_moles: 0.01,
  metal_in_oxide_fraction: fraction,
  words: "Current oxide bookkeeping, not history or surface coverage.",
});

describe("persistent corrosion readouts", () => {
  it("keeps an absent legacy field absent", () => {
    expect(corrosionReadouts()).toEqual([]);
  });

  it("clamps the core fraction and maps it monotonically", () => {
    const [below, clean, half, full, above] = corrosionReadouts([
      row(-1), row(0), row(0.5), row(1), row(2),
    ]);
    expect(below!.fraction).toBe(0);
    expect(above!.fraction).toBe(1);
    expect(clean!.percent).toBe(0);
    expect(half!.percent).toBe(50);
    expect(full!.percent).toBe(100);
    expect(clean!.visualStrength).toBeLessThan(half!.visualStrength);
    expect(half!.visualStrength).toBeLessThan(full!.visualStrength);
  });
});
