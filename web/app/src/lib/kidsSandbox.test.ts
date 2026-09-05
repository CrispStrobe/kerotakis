import { describe, expect, it } from "vitest";
import { briefFor, kidsEquipmentVerbs, kidsShelfKeys, storePendingKidsSandbox, takePendingKidsSandbox } from "./kidsSandbox";

const experiment = {
  id: "K44", title: "Eggshell in cola", phenomenon: "Acid attacks carbonate", status: "computed" as const,
  topics: ["food"], ingredients: ["cola", "chalk_stick"], apparatus: ["beaker", "ph"], safety: "home" as const,
};

function memoryStorage() {
  const values = new Map<string, string>();
  return {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => void values.set(key, value),
    removeItem: (key: string) => void values.delete(key),
  };
}

describe("KIDS sandbox handoff", () => {
  it("resolves display ingredients and cabinet instruments", () => {
    expect(kidsShelfKeys(["cola", "paper", "NaCl"])).toEqual(["cola_drink", "paper_sheet", "NaCl"]);
    expect(kidsEquipmentVerbs(["beaker", "ph", "thermometer"])).toEqual(["beaker", "measure:ph", "measure:thermometer"]);
  });

  it("survives one mode-changing reload and is consumed", () => {
    const storage = memoryStorage();
    const brief = briefFor(experiment);
    storePendingKidsSandbox(storage, brief);
    expect(takePendingKidsSandbox(storage)).toEqual(brief);
    expect(takePendingKidsSandbox(storage)).toBeNull();
  });
});
