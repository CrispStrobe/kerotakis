import { describe, expect, it } from "vitest";
import { loadApparatusInstallation, saveApparatusInstallation } from "./apparatusInstallation";

class MemoryStorage {
  value: string | null = null;
  getItem() { return this.value; }
  setItem(_key: string, value: string) { this.value = value; }
  removeItem() { this.value = null; }
}

describe("apparatus installation persistence", () => {
  it("round-trips target and physical settings", () => {
    const storage = new MemoryStorage();
    saveApparatusInstallation(storage, "installation", {
      tool: "stir", target: 2, values: { rpm: 650, seconds: 15 },
    });
    expect(loadApparatusInstallation(storage, "installation")).toEqual({
      tool: "stir", target: 2, values: { rpm: 650, seconds: 15 },
    });
  });

  it("rejects malformed persisted state and removes put-away tools", () => {
    const storage = new MemoryStorage();
    storage.value = JSON.stringify({ tool: "stir", target: "v1", values: {} });
    expect(loadApparatusInstallation(storage, "installation")).toBeNull();
    saveApparatusInstallation(storage, "installation", null);
    expect(storage.value).toBeNull();
  });
});
