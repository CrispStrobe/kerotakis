import { describe, expect, it } from "vitest";
import { isStoryDispenseEvent, persistStockUsed, restoreStockUsed, stockCapacity, stockRemaining, suppliedSpecies, suppliedSpeciesFromOperator } from "./storyStock";

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

  it("reads canonical identities from typed engine operators", () => {
    expect(suppliedSpeciesFromOperator({ op: "add", species: "water" })).toBe("water");
    expect(suppliedSpeciesFromOperator({ op: "add_material", material: "vinegar" })).toBe("vinegar");
    expect(suppliedSpeciesFromOperator({ op: "titrate", titrant: "NaOH" })).toBe("NaOH");
    for (const operator of [null, [], "add v1 H2O 1mol", { op: "measure" }, { op: "add", species: 1 }]) {
      expect(suppliedSpeciesFromOperator(operator)).toBeNull();
    }
  });

  it("recognises actual dispenses independently of their visualization", () => {
    expect(isStoryDispenseEvent({ event: "added", moles: 0.25 })).toBe(true);
    expect(isStoryDispenseEvent({ event: "material_added", total_amount: 1 })).toBe(true);
    expect(isStoryDispenseEvent({ event: "titrated", total_volume: 0.003 })).toBe(true);
    for (const total_volume of [0, -1, NaN, Infinity, undefined, "0.003"]) {
      expect(isStoryDispenseEvent({ event: "titrated", total_volume })).toBe(false);
    }
    for (const event of ["safety_veto", "stock_exhausted", "measured"]) {
      expect(isStoryDispenseEvent({ event, total_volume: 0.003 })).toBe(false);
    }
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
