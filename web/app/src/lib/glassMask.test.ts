import { describe, expect, it } from "vitest";
import { KINDS } from "./glassware";
import { inPolygon, maskFromPath, pathToPolygon } from "./glassMask";

describe("the glass mask is the actual glassware", () => {
  it("parses every shipped glass kind without guessing", () => {
    for (const [kind, geom] of Object.entries(KINDS)) {
      const poly = pathToPolygon(geom.inner);
      expect(poly.length, kind).toBeGreaterThan(3);
    }
  });

  it("refuses unsupported path commands loudly", () => {
    expect(() => pathToPolygon("M 0 0 C 1 1 2 2 3 3 Z")).toThrow(/unsupported/);
  });

  it("a square path masks exactly its outside", () => {
    // Square covering the middle of the viewBox.
    const mask = maskFromPath("M 25 35 L 75 35 L 75 105 L 25 105 Z", 10, 14);
    // Cell (0,0) centre = (5, 5): outside.
    expect(mask[0]).toBe(1);
    // Centre cell (5,7) centre = (55, 75): inside.
    expect(mask[7 * 10 + 5]).toBe(0);
  });

  it("the flask's cone: narrow at the neck, wide at the base", () => {
    const flask = KINDS.flask!;
    const w = 20;
    const h = 28;
    const mask = maskFromPath(flask.inner, w, h);
    const fluidCells = (row: number) => {
      let n = 0;
      for (let x = 0; x < w; x++) if (!mask[row * w + x]) n++;
      return n;
    };
    // A neck row (upper quarter) admits fewer cells than a base row.
    const neck = fluidCells(Math.floor(h * 0.15));
    const base = fluidCells(Math.floor(h * 0.8));
    expect(neck).toBeGreaterThan(0);
    expect(base).toBeGreaterThan(neck * 2);
  });

  it("point-in-polygon agrees with itself across the boundary", () => {
    const poly = pathToPolygon("M 10 10 L 90 10 L 90 130 L 10 130 Z");
    expect(inPolygon(poly, 50, 70)).toBe(true);
    expect(inPolygon(poly, 5, 70)).toBe(false);
    expect(inPolygon(poly, 95, 70)).toBe(false);
  });
});
