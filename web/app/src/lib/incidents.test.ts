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
});
