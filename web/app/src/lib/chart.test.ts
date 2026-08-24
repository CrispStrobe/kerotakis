import { describe, expect, it } from "vitest";
import {
  bandPath,
  extent,
  isChartSpec,
  linePath,
  niceTicks,
  scale,
  seriesPoints,
  type ChartSpec,
} from "./chart";

const titration: ChartSpec = {
  title: "titration",
  x: { label: "volume added", unit: "mL" },
  y: { label: "pH" },
  series: [
    {
      kind: "line",
      name: "pH",
      points: [
        [0, 1.0],
        [12.5, 1.5],
        [25, 7.0],
        [37.5, 12.5],
      ],
    },
  ],
  provenance: "PHREEQC (IPhreeqc) · wateq4f.dat",
};

describe("chart core (the CAP-3 contract, consumer side)", () => {
  it("recognises the contract shape", () => {
    expect(isChartSpec(titration)).toBe(true);
    expect(isChartSpec({ title: "x" })).toBe(false);
    expect(isChartSpec(null)).toBe(false);
  });

  it("extent covers lines and both band envelopes", () => {
    const e = extent(titration);
    expect(e.x).toEqual([0, 37.5]);
    expect(e.y).toEqual([1.0, 12.5]);
    const banded: ChartSpec = {
      ...titration,
      series: [
        { kind: "band", name: "b", lower: [[1, 3]], upper: [[1, 9]] },
      ],
    };
    expect(extent(banded).y).toEqual([3, 9]);
  });

  it("a degenerate extent is padded rather than zero-width", () => {
    const flat: ChartSpec = {
      ...titration,
      series: [{ kind: "scatter", name: "f", points: [[2, 4]] }],
    };
    const e = extent(flat);
    expect(e.x[0]).toBeLessThan(e.x[1]);
    expect(e.y[0]).toBeLessThan(e.y[1]);
  });

  it("scale maps linearly, inverted ranges included", () => {
    const s = scale([0, 10], [100, 0]); // SVG y grows downward
    expect(s(0)).toBe(100);
    expect(s(10)).toBe(0);
    expect(s(5)).toBe(50);
  });

  it("nice ticks step by 1/2/5 and include zero exactly", () => {
    expect(niceTicks(0, 10, 5)).toEqual([0, 2, 4, 6, 8, 10]);
    expect(niceTicks(0, 14, 5)).toEqual([0, 5, 10]);
    expect(niceTicks(-4, 4, 4)).toContain(0);
    expect(niceTicks(0, 0.014, 5)[1]).toBeCloseTo(0.005, 10);
  });

  it("paths render moves then lines; bands close over both envelopes", () => {
    const id = (v: number) => v;
    expect(linePath([[0, 1], [2, 3]], id, id)).toBe("M0.00,1.00 L2.00,3.00");
    const band = bandPath([[0, 1], [4, 2]], [[0, 2], [4, 3]], id, id);
    expect(band.startsWith("M0.00,2.00")).toBe(true);
    expect(band.endsWith("Z")).toBe(true);
    expect(
      seriesPoints({ kind: "band", name: "b", lower: [[0, 1]], upper: [[0, 2]] }),
    ).toEqual([
      [0, 1],
      [0, 2],
    ]);
  });
});
