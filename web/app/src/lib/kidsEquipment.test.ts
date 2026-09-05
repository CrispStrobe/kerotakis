import { describe, expect, it } from "vitest";
import { KIDS_EQUIPMENT } from "./kidsEquipment";

describe("children's apparatus skins", () => {
  it("maps every familiar object to an existing GUI action", () => {
    expect(KIDS_EQUIPMENT.map((item) => [item.id, item.engineVerb, item.action])).toEqual([
      ["balloon-kit", "regulate", "apparatus"],
      ["candle-kit", "bunsen", "apparatus"],
      ["paper-chromatography-kit", "measure:chromatograph", "instrument"],
      ["filter-funnel-kit", "filter", "transfer"],
      ["magnet-kit", "magnet", "transfer"],
    ]);
  });

  it("names physical parts and an honest boundary for every skin", () => {
    for (const item of KIDS_EQUIPMENT) {
      expect(item.parts.length).toBeGreaterThanOrEqual(3);
      expect(item.boundary.length).toBeGreaterThan(20);
    }
    expect(KIDS_EQUIPMENT.find((item) => item.id === "paper-chromatography-kit")?.parts)
      .toContain("spotting tile");
  });
});
