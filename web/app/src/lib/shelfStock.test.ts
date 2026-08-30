import { describe, expect, it } from "vitest";
import { formatStockAmount, isExhausted, stockBadge, stockLevels } from "./shelfStock";

const t = (key: string, values?: Record<string, string | number>) =>
  key.replace(/\{(\w+)\}/g, (_, name) => String(values?.[name] ?? `{${name}}`));

describe("shelf stock levels", () => {
  it("treats a missing list as an unlimited shelf, not an empty one", () => {
    expect(stockLevels(undefined)).toEqual({});
    expect(isExhausted(undefined)).toBe(false);
    expect(stockBadge(undefined, t)).toBe(null);
  });

  it("indexes the scene's bottles by shelf key", () => {
    const levels = stockLevels([
      { key: "NaCl", remaining: 0.2, unit: "mol" },
      { key: "white_vinegar_5_percent", remaining: 40, unit: "g" },
    ]);
    expect(levels["NaCl"].remaining).toBe(0.2);
    expect(levels["white_vinegar_5_percent"].unit).toBe("g");
    expect(levels["water"]).toBeUndefined();
  });

  it("drops a malformed entry rather than rendering NaN on the shelf", () => {
    const levels = stockLevels([
      { key: "NaCl", remaining: Number.NaN, unit: "mol" },
      // deliberately shaped wrong, the way a stale host could send it
      { remaining: 1, unit: "g" } as never,
    ]);
    expect(levels).toEqual({});
  });

  it("calls a bottle empty only at zero", () => {
    expect(isExhausted({ key: "a", remaining: 0, unit: "g" })).toBe(true);
    expect(isExhausted({ key: "a", remaining: 0.0001, unit: "mol" })).toBe(false);
  });

  it("never rounds a real remainder down to nothing", () => {
    // The reason the floor exists: 0.00004 mol is a poor last dose, but
    // it is not "0", and a shelf that says 0 while the engine still
    // pours is the exact dishonesty this feature removes.
    expect(formatStockAmount(0.00004)).toBe("0.0001");
    expect(formatStockAmount(0)).toBe("0");
    expect(formatStockAmount(-1)).toBe("0");
  });

  it("shortens the number as it grows, so a row stays a row", () => {
    expect(formatStockAmount(0.2)).toBe("0.2");
    expect(formatStockAmount(39.999999999)).toBe("40");
    expect(formatStockAmount(2.34)).toBe("2.3");
    expect(formatStockAmount(250.4)).toBe("250");
  });

  it("says how much is left, in the unit the engine dispensed in", () => {
    expect(stockBadge({ key: "NaCl", remaining: 0.2, unit: "mol" }, t)).toBe("0.2 mol left");
    expect(stockBadge({ key: "v", remaining: 40, unit: "g" }, t)).toBe("40 g left");
    expect(stockBadge({ key: "v", remaining: 0, unit: "g" }, t)).toBe("empty");
  });
});
