import { describe, expect, it } from "vitest";
import { resultCardFilename, resultCardSvg, wrapCardText, type ResultCardImageText } from "./resultCardImage";
import type { ResultSummary } from "./resultSummary";

const result: ResultSummary = {
  kind: "precipitation",
  vessel: 0,
  equation: "Ag⁺ + Cl⁻ → AgCl",
  observation: "a white solid forms",
  quantities: [{ label: "amount", value: 0.01, unit: "mol" }],
  temperatureDeltaK: 2,
  provenance: "PHREEQC · wateq4f.dat",
};
const labels: ResultCardImageText = {
  title: "latest computed result", vessel: "v1", equation: "equation",
  observation: "observation", results: "results", provenance: "PHREEQC · wateq4f.dat",
  emptyEquation: "no equation", emptyObservation: "no observation",
};

describe("shareable result card image", () => {
  it("is deterministic and contains every hand-in field plus accessible text", () => {
    const first = resultCardSvg(result, labels, String);
    expect(resultCardSvg(result, labels, String)).toBe(first);
    expect(first).toContain('viewBox="0 0 800 530"');
    expect(first).toContain('role="img" aria-labelledby="title description"');
    for (const value of ["Ag⁺ + Cl⁻ → AgCl", "a white solid forms", "0.01 mol", "ΔT +2 K", "PHREEQC · wateq4f.dat"]) {
      expect(first).toContain(value);
    }
  });

  it("escapes engine text and bounds long prose", () => {
    const svg = resultCardSvg({
      ...result,
      equation: "A < B & C",
      observation: "word ".repeat(200),
      quantities: [{ label: "quantity ".repeat(100), value: 1, unit: "unit ".repeat(100) }],
    }, labels, String);
    expect(svg).toContain("A &lt; B &amp; C");
    expect(svg).not.toContain("A < B & C");
    expect(wrapCardText("word ".repeat(200), 20, 2)).toHaveLength(2);
    expect(wrapCardText("word ".repeat(200), 20, 2)[1]).toMatch(/…$/);
    const renderedNumbers = [...svg.matchAll(/class="number">([^<]*)<\/text>/g)].map((match) => match[1]);
    expect(renderedNumbers).toHaveLength(4);
    expect(renderedNumbers.every((line) => line.length <= 82)).toBe(true);
    expect(renderedNumbers[3]).toMatch(/…$/);
  });

  it("creates a stable safe download name", () => {
    expect(resultCardFilename({ ...result, kind: "Gas evolution!" }, "png"))
      .toBe("kerotakis-gas-evolution.png");
  });
});
