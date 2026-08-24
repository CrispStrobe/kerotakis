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

describe("codex grouping for the browsers", () => {
  const mk = (id: string, concepts: string[], curriculum?: unknown) =>
    ({
      id,
      concepts,
      curriculum,
      setup: { script: "new" },
      expect: {},
      registers: {},
    }) as unknown as import("./codex").CodexEntry;

  it("conceptIndex counts and orders, most-taught first", async () => {
    const { conceptIndex } = await import("./codex");
    const idx = conceptIndex([
      mk("a", ["solubility", "equilibrium"]),
      mk("b", ["solubility"]),
      mk("c", ["acids"]),
    ]);
    expect(idx[0]).toEqual({ concept: "solubility", count: 2 });
    expect(idx.map((i) => i.concept)).toEqual(["solubility", "acids", "equilibrium"]);
  });

  it("relatedConcepts ranks co-occurrence, excluding the concept itself", async () => {
    const { relatedConcepts } = await import("./codex");
    const entries = [
      mk("a", ["solubility", "equilibrium"]),
      mk("b", ["solubility", "equilibrium", "ksp"]),
      mk("c", ["solubility", "ksp"]),
      mk("d", ["acids"]),
    ];
    expect(relatedConcepts(entries, "solubility")).toEqual(["equilibrium", "ksp"]);
  });

  it("conceptGraph layers by longest prerequisite chain and survives cycles", async () => {
    const { conceptGraph } = await import("./codex");
    const mkq = (id: string, concepts: string[], requires: string[]) =>
      ({ id, concepts, requires, setup: { script: "new" }, expect: {}, registers: {} }) as never;
    const g = conceptGraph([
      mkq("a", ["dissolution"], []),
      mkq("b", ["equilibrium"], ["dissolution"]),
      mkq("c", ["ksp"], ["equilibrium", "dissolution"]),
    ]);
    const byName = Object.fromEntries(g.nodes.map((n) => [n.concept, n.depth]));
    expect(byName).toEqual({ dissolution: 0, equilibrium: 1, ksp: 2 });
    expect(g.edges).toContainEqual({ from: "equilibrium", to: "ksp" });
    // A cycle parks its members rather than hanging.
    const cyclic = conceptGraph([mkq("x", ["p"], ["q"]), mkq("y", ["q"], ["p"])]);
    expect(cyclic.nodes.length).toBe(2);
  });

  it("metConcepts and entryReady gate on completed runs only", async () => {
    const { metConcepts, entryReady } = await import("./codex");
    const mkq = (id: string, concepts: string[], requires: string[]) =>
      ({ id, concepts, requires, setup: { script: "new" }, expect: {}, registers: {} }) as never;
    const entries = [
      mkq("a", ["dissolution"], []),
      mkq("b", ["equilibrium"], ["dissolution"]),
    ];
    const met = metConcepts(entries, new Set(["a"]));
    expect([...met]).toEqual(["dissolution"]);
    expect(entryReady(entries[1]!, met)).toBe(true);
    expect(entryReady(entries[1]!, new Set())).toBe(false);
  });

  it("curriculumIndex groups system → stage, ordered by age band then name", async () => {
    const { curriculumIndex } = await import("./codex");
    const entries = [
      mk("a", [], [
        { system: "england", stage: "KS4", ages: { min: 14 }, source: "doc-1" },
      ]),
      mk("b", [], [
        { system: "england", stage: "KS3", ages: { min: 11 }, source: "doc-1" },
        { system: "bayern", stage: "Jgst. 9", source: "lehrplan" },
      ]),
    ];
    const idx = curriculumIndex(entries);
    expect(idx.map((s) => s.system)).toEqual(["bayern", "england"]);
    const england = idx[1]!;
    expect(england.stages.map((s) => s.stage)).toEqual(["KS3", "KS4"]);
    expect(england.stages[0]!.entries.map((e) => e.id)).toEqual(["b"]);
    expect(england.stages[0]!.sources).toEqual(["doc-1"]);
  });
});
