import { describe, expect, it } from "vitest";
import { commandVerb, missionHint, missionObjective } from "./missionJournal";

describe("mission journal language", () => {
  it("extracts the operator verb without interpreting chemistry", () => {
    expect(commandVerb("  titrate v1 NaOH 1M until ph 7")).toBe("titrate");
  });

  it("describes every verb used by the shipped lessons", () => {
    const verbs = ["add", "cell", "chromatograph", "cool", "decant", "distil", "electrolyze", "evaporate", "heat", "ignite", "inspect", "look", "measure", "mix", "new", "react", "register", "seal", "titrate", "transport", "wait"];
    for (const verb of verbs) {
      expect(missionObjective(`${verb} example`)).not.toBe("Carry out the next investigation step");
      expect(missionHint(`${verb} example`).length).toBeGreaterThan(20);
    }
  });

  it("keeps unknown future operators usable", () => {
    expect(missionObjective("future-operator v1")).toBe("Carry out the next investigation step");
  });
});
