import { describe, expect, it } from "vitest";
import { persistStockUsed, restoreStockUsed, stockCapacity, stockRemaining, suppliedSpecies } from "./storyStock";

describe("Story stockroom", () => {
  it("allocates smaller supervised stocks for higher-risk substances", () => {
    expect(stockCapacity({ hazards: [], hazard_assessed: true })).toBe(10);
    expect(stockCapacity({ hazards: ["flammable"], hazard_assessed: true })).toBe(6);
    expect(stockCapacity({ hazards: ["corrosive"], hazard_assessed: true })).toBe(4);
    expect(stockCapacity({ hazards: [], hazard_assessed: false })).toBe(3);
  });

  it("tracks bounded dispenses and recognises material-drawing commands", () => {
    const item = { key: "NaCl", hazards: [], hazard_assessed: true };
    expect(stockRemaining(item, { NaCl: 3 })).toBe(7);
    expect(stockRemaining(item, { NaCl: 99 })).toBe(0);
    expect(suppliedSpecies("add v1 NaCl 1g")).toBe("NaCl");
    expect(suppliedSpecies("titrate v1 NaOH 1M 1mL until ph 7")).toBe("NaOH");
    expect(suppliedSpecies("measure v1 ph")).toBeNull();
  });

  it("persists only a valid non-negative integer ledger", () => {
    const values = new Map<string, string>();
    const storage = { getItem: (key: string) => values.get(key) ?? null, setItem: (key: string, value: string) => values.set(key, value) };
    persistStockUsed(storage, { water: 2 });
    expect(restoreStockUsed(storage)).toEqual({ water: 2 });
    values.set("kero.story-stock.v1", '{"water":2.9,"bad":-1,"no":"x"}');
    expect(restoreStockUsed(storage)).toEqual({ water: 2 });
  });
});
