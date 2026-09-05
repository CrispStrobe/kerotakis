import { describe, expect, it } from "vitest";
import { missionEquipment } from "./catalogScope";

describe("missionEquipment", () => {
  it("maps mission commands to cabinet instruments without duplicates", () => {
    expect(missionEquipment(["add v1 water 10mL", "decant v1 v2 0.5", "decant v2 v3 0.5", "titrate v1 NaOH 1M"])).toEqual(["decant", "burette"]);
  });

  it("includes measurement, smell, and chromatography instruments", () => {
    expect(missionEquipment([
      "measure v1 thermometer", "smell v1", "chromatograph v1", "measure v2 thermometer",
    ])).toEqual(["measure:thermometer", "measure:smell", "measure:chromatograph"]);
  });

  it("loans the familiar flame and magnet objects for their engine commands", () => {
    expect(missionEquipment(["ignite v1", "magnet v1 v2"]))
      .toEqual(["bunsen", "magnet"]);
  });
});
