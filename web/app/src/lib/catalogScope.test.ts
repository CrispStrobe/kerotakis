import { describe, expect, it } from "vitest";
import { missionEquipment } from "./catalogScope";

describe("missionEquipment", () => {
  it("maps mission commands to cabinet instruments without duplicates", () => {
    expect(missionEquipment(["add v1 water 10mL", "decant v1 v2 0.5", "decant v2 v3 0.5", "titrate v1 NaOH 1M"])).toEqual(["decant", "burette"]);
  });
});
