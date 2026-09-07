import { describe, expect, it } from "vitest";
import { incidentEffects, incidentNotebookEvidence } from "./incidents";

describe("spill and breakage presentation", () => {
  it("keeps only live engine-confirmed incidents", () => {
    expect(incidentEffects({
      0: [{ kind: "spill", at: 9000, durationMs: 5000, magnitude: .5 }],
      1: [{ kind: "break", at: 1000, durationMs: 5000, magnitude: 1 }],
      2: [{ kind: "burst", at: 9000, durationMs: 5000, magnitude: 1 }],
    }, 10_000).map((effect) => effect.kind)).toEqual(["spill"]);
  });

  it("coalesces the spill emitted alongside atomic breakage", () => {
    expect(incidentEffects({ 0: [
      { kind: "break", at: 9000, durationMs: 5000, magnitude: 1, source: 0 },
      { kind: "spill", at: 9001, durationMs: 5000, magnitude: 1, source: 0 },
    ] }, 10_000).map((effect) => effect.kind)).toEqual(["break"]);
  });

  it("writes precise, animation-independent notebook evidence", () => {
    expect(incidentNotebookEvidence({
      event: "spill_created", source: 1, fraction: .375,
      destination: { surface: "tray", tray: "acid-tray" },
    })).toBe("Evidence: 37.5% of vessel v2 entered tray acid-tray.");
    expect(incidentNotebookEvidence({
      event: "container_broken", vessel: 0,
      destination: { surface: "floor", zone: "east" },
    })).toBe("Evidence: vessel v1 broke; contents routed to floor east.");
    expect(incidentNotebookEvidence({
      event: "spill_recovered", to: 2, fraction: .5,
      destination: { surface: "bench", zone: "react" },
    })).toBe("Evidence: 50.0% of bench react was recovered into vessel v3.");
    expect(incidentNotebookEvidence({ event: "transferred" })).toBeNull();
  });

  /**
   * A disposal is the one incident the reader MEANT to cause, which is
   * exactly why the notebook has to record it. An experiment that ends
   * "and then we threw it away" is missing the line saying how much of
   * what — and the engine keeps the matter in the waste ledger precisely
   * so that line can be written.
   */
  it("records a disposal with what it weighed, not just that it happened", () => {
    expect(incidentNotebookEvidence({
      event: "discarded", vessel: 0, grams_total: 100.0004, moles_total: 5.5509,
      into: { surface: "waste" },
      species: [{ species: "water", moles: 5.5509, phase: "Liquid" }],
    })).toBe(
      "Evidence: vessel v1 was emptied into the waste — 100.000 g (5.551 mol), "
      + "still held and still weighed by the bench.",
    );
  });
});
