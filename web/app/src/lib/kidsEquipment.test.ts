import { describe, expect, it } from "vitest";
import { KIDS_EQUIPMENT, kitInfoRows } from "./kidsEquipment";

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

  it("keeps the parts list and the boundary, behind the (i) rather than on the card", () => {
    // The cabinet printed four blocks per kit and became a wall of text.
    // Nothing was deleted to fix that — it moved — so the check is that
    // every part and the whole boundary sentence are still reachable.
    const identity = (key: string) => key;
    for (const item of KIDS_EQUIPMENT) {
      const rows = kitInfoRows(item, identity);
      expect(rows.map((row) => row.term)).toEqual(["parts", "what the model computes"]);
      for (const part of item.parts) expect(rows[0]?.detail).toContain(part);
      expect(rows[1]?.detail).toBe(item.boundary);
      // A sentence is set under its label, not opposite it.
      expect(rows[1]?.block).toBe(true);
    }
  });

  it("localizes every piece it shows, rather than only the labels", () => {
    // `translate` reaches the part names and the boundary too. Passing it
    // and only using it on the row labels is the silent half-translation
    // this asserts against.
    const shout = (key: string) => key.toUpperCase();
    const rows = kitInfoRows(KIDS_EQUIPMENT[0]!, shout);
    expect(rows[0]?.detail).toBe(KIDS_EQUIPMENT[0]!.parts.map((p) => p.toUpperCase()).join(" · "));
    expect(rows[1]?.detail).toBe(KIDS_EQUIPMENT[0]!.boundary.toUpperCase());
  });

  it("deploys the candle kit as a CANDLE, not as a Bunsen burner", () => {
    // Both open the same flame panel, and the engine caps a vessel at the
    // source heating it — so a kit called "candle and wick" that opened on
    // the burner's default would quietly offer 100 °C the thing it is
    // named after cannot reach.
    const candle = KIDS_EQUIPMENT.find((item) => item.id === "candle-kit")!;
    expect(candle.engineVerb).toBe("bunsen");
    expect(candle.preset).toEqual({ source: "candle" });
    // Nothing else claims a source it has no business claiming.
    for (const item of KIDS_EQUIPMENT) {
      if (item.id !== "candle-kit") expect(item.preset).toBeUndefined();
    }
  });
});
