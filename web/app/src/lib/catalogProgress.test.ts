import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { equipmentAccess, equipmentAvailable, equipmentRequirement, equipmentRewardAt, reagentAccess, reagentRequirement } from "./catalogProgress";

describe("progression-aware catalog", () => {
  it("keeps Sandbox fully unlocked and gives Story understandable milestones", () => {
    expect(equipmentAvailable("sandbox", 0, "distil")).toBe(true);
    expect(equipmentAvailable("story", 0, "evaporate")).toBe(false);
    expect(equipmentAvailable("story", 1, "evaporate")).toBe(true);
    expect(equipmentAvailable("story", 3, "electrolyse")).toBe(true);
    expect(equipmentAvailable("story", 3, "distil")).toBe(false);
  });

  it("loans mission reagents without permanently unlocking the stockroom", () => {
    const hazardous = { key: "HCl", hazards: ["corrosive"], hazard_assessed: true };
    expect(reagentRequirement(hazardous)).toBe(3);
    expect(reagentAccess("story", 0, hazardous, false)).toMatchObject({ available: false, loaned: false });
    expect(reagentAccess("story", 0, hazardous, true)).toMatchObject({ available: true, loaned: true });
    expect(reagentAccess("story", 0, { key: "NaCl", hazards: [], hazard_assessed: true }, true)).toMatchObject({ available: true, loaned: true });
    expect(reagentAccess("story", 3, hazardous, false)).toMatchObject({ available: true, loaned: false });
    expect(reagentAccess("sandbox", 0, hazardous, false).available).toBe(true);
  });

  it("offers one permanent instrument reward at each early milestone", () => {
    expect(equipmentRewardAt(1)?.verb).toBe("evaporate");
    expect(equipmentRewardAt(3)?.verb).toBe("electrolyse");
    expect(equipmentRewardAt(5)).toBeNull();
  });

  it("loans mission equipment without turning it into a permanent unlock", () => {
    expect(equipmentAccess("story", 0, "distil", false)).toMatchObject({ available: false, loaned: false });
    expect(equipmentAccess("story", 0, "distil", true)).toMatchObject({ available: true, loaned: true });
    expect(equipmentAccess("story", 4, "distil", false)).toMatchObject({ available: true, loaned: false });
  });
});

describe("a closed case grants its instrument permanently (GUI-080)", () => {
  // The spectrometer's own milestone is three completed missions; the award
  // is what makes it reachable the moment the case closes instead.
  const award: ReadonlySet<string> = new Set(["measure:uvvis"]);

  it("puts the awarded instrument on the wall below its milestone", () => {
    expect(equipmentAvailable("story", 0, "measure:uvvis")).toBe(false);
    expect(equipmentAvailable("story", 0, "measure:uvvis", award)).toBe(true);
  });

  it("grants only what was awarded, and says the grant is why", () => {
    expect(equipmentAvailable("story", 0, "distil", award)).toBe(false);
    const access = equipmentAccess("story", 0, "measure:uvvis", false, award);
    expect(access.available).toBe(true);
    expect(access.granted).toBe(true);
    // A grant is not a loan: it outlives the mission, and must not be
    // reported as one.
    expect(access.loaned).toBe(false);
    expect(access.minimumCompleted).toBe(3);
  });

  it("leaves every existing caller unchanged when nothing is awarded", () => {
    expect(equipmentAvailable("story", 3, "measure:uvvis")).toBe(true);
    expect(equipmentAccess("story", 0, "measure:uvvis", false).granted).toBe(false);
    expect(equipmentAccess("sandbox", 0, "distil", false).available).toBe(true);
  });
});

describe("the progression rules match the shared contract fixture (WORLD-003)", () => {
  // The engine now owns these rules (kerotakis-core::catalog). Until the
  // client reads them off the protocol, this table is a copy — and a copy
  // that drifts shows a learner an instrument the engine will refuse. Both
  // languages check themselves against the same pin, so drift fails here.
  const pin = JSON.parse(
    readFileSync(join(import.meta.dirname, "../../../../tests/contract/catalog-milestones-v1.json"), "utf8"),
  ) as {
    last_tier: number;
    starter_stock: string[];
    apparatus: Record<string, number>;
    instruments: Record<string, number>;
  };

  it("agrees with the engine on every apparatus and instrument tier", () => {
    const disagreements: string[] = [];
    for (const [id, tier] of Object.entries({ ...pin.apparatus, ...pin.instruments })) {
      const mine = equipmentRequirement(id);
      if (mine !== tier) disagreements.push(`${id}: client ${mine}, engine ${tier}`);
    }
    expect(disagreements).toEqual([]);
  });

  it("agrees on the starter stock, which is what costs nothing to reach", () => {
    for (const key of pin.starter_stock) {
      expect(reagentRequirement({ key, hazards: ["toxic"], hazard_assessed: true })).toBe(0);
    }
    // And something outside it is not free.
    expect(reagentRequirement({ key: "AgNO3", hazards: [], hazard_assessed: true })).toBeGreaterThan(0);
  });

  it("agrees that an unassessed species sits at the last tier, not the first", () => {
    expect(reagentRequirement({ key: "mystery", hazards: [], hazard_assessed: false })).toBe(pin.last_tier);
  });
});
