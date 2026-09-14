import { describe, expect, it } from "vitest";
import { formatStockAmount, isExhausted, lockNote, stockBadge, stockLevels } from "./shelfStock";
import type { CatalogAccess } from "./catalogProgress";

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
    // Asserted through the whole object rather than by indexing into a
    // possibly-undefined one: `noUncheckedIndexedAccess` is right that a
    // lookup can miss, and this says what the entry should BE anyway.
    expect(levels["NaCl"]).toEqual({ key: "NaCl", remaining: 0.2, unit: "mol" });
    expect(levels["white_vinegar_5_percent"]).toEqual({
      key: "white_vinegar_5_percent",
      remaining: 40,
      unit: "g",
    });
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

describe("why a material cannot be taken", () => {
  const access = (overrides: Partial<CatalogAccess> = {}): CatalogAccess => ({
    available: false,
    loaned: false,
    granted: false,
    missionOnly: false,
    minimumCompleted: 0,
    ...overrides,
  });

  it("says nothing about a material that can be poured", () => {
    expect(lockNote(access({ available: true }), t)).toBe(null);
  });

  it("prints a milestone only when there is one", () => {
    expect(lockNote(access({ minimumCompleted: 1 }), t)).toBe(
      "Permanent stock unlocks after one completed mission. Mission kits loan required materials.",
    );
    expect(lockNote(access({ minimumCompleted: 3 }), t)).toBe(
      "Permanent stock unlocks after 3 completed missions. Mission kits loan required materials.",
    );
  });

  it("never offers a count of zero", () => {
    // The owner's sentence: "Der dauerhafte Vorrat wird nach 0
    // abgeschlossenen Missionen freigeschaltet". No progression produces
    // it — a locked engine row always carries a minimum above the
    // learner's own completed count — so a zero here is the client saying
    // it does not know, and the label has to say that instead.
    const note = lockNote(access(), t);
    expect(note).not.toContain("0");
    expect(note).toBe(
      "The supply cabinet has not said anything about this material yet. Mission kits still loan required materials.",
    );
  });

  it("keeps a supervised kit distinct from a milestone", () => {
    expect(lockNote(access({ missionOnly: true, minimumCompleted: 4 }), t)).toBe(
      "Supervised mission kit only — this material never becomes permanent Story stock.",
    );
  });
});
