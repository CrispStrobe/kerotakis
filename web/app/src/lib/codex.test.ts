import { describe, expect, it } from "vitest";
import { checkExpect, parseCodexIndex } from "./codex";

describe("the codex checker compares, never computes", () => {
  it("events must occur, forbidden ones must not", () => {
    const r = checkExpect(
      { events: ["precipitated:AgCl", "solution_characterized"], absent: ["hazard_warning"] },
      ["added", "precipitated:AgCl", "precipitated", "solution_characterized"],
      { phValues: [], temperaturesC: [] },
    );
    expect(r.events).toEqual([
      { want: "precipitated:AgCl", seen: true },
      { want: "solution_characterized", seen: true },
    ]);
    expect(r.forbidden).toEqual([{ want: "hazard_warning", violated: false }]);
    expect(r.allOk).toBe(true);
  });

  it("a missing event or a violated absence fails the check", () => {
    const r = checkExpect(
      { events: ["dissolved:NaCl"], absent: ["safety_veto"] },
      ["added", "safety_veto"],
      { phValues: [], temperaturesC: [] },
    );
    expect(r.events[0]!.seen).toBe(false);
    expect(r.forbidden[0]!.violated).toBe(true);
    expect(r.allOk).toBe(false);
  });

  it("numeric ranges check against any vessel's final state", () => {
    const r = checkExpect(
      { ph: { min: 6.5, max: 7.5 }, temperature_c: { min: 20 } },
      [],
      { phValues: [2.9, 7.1], temperaturesC: [25.0] },
    );
    expect(r.ph!.ok).toBe(true);
    expect(r.temperature_c!.ok).toBe(true);
    expect(r.allOk).toBe(true);
  });

  it("the index parser tolerates both shapes and drops malformed rows", () => {
    const entries = [
      { id: "a", setup: { script: "add v1 water 100mL" }, expect: {}, registers: {} },
      { id: 42, setup: {} },
    ];
    expect(parseCodexIndex(entries)).toHaveLength(1);
    expect(parseCodexIndex({ entries })).toHaveLength(1);
    expect(parseCodexIndex(null)).toEqual([]);
  });
});
