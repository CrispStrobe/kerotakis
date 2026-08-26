import { describe, expect, it } from "vitest";
import { equipmentAvailable, equipmentRewardAt, reagentAccess, reagentRequirement } from "./catalogProgress";

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
});
