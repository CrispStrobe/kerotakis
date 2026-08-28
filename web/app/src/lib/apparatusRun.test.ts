import { describe, expect, it } from "vitest";
import { apparatusRunsCommand } from "./apparatusRun";

describe("apparatus running ownership", () => {
  it("matches the deployed verb and exact vessel", () => {
    expect(apparatusRunsCommand("stir v1 600rpm 10s", "stir", 0)).toBe(true);
    expect(apparatusRunsCommand("stir v2 600rpm 10s", "stir", 0)).toBe(false);
    expect(apparatusRunsCommand("heat v1 5kJ", "stir", 0)).toBe(false);
  });

  it("maps a physical burette to titration without matching unrelated work", () => {
    expect(apparatusRunsCommand("titrate v3 NaOH 0.1M 25mL", "burette", 2)).toBe(true);
    expect(apparatusRunsCommand("add v3 NaOH 1mL", "burette", 2)).toBe(false);
  });

  it("never energizes a machine without a complete deployment", () => {
    expect(apparatusRunsCommand("centrifuge v1 3000rpm 60s 8cm", null, 0)).toBe(false);
    expect(apparatusRunsCommand(null, "centrifuge", 0)).toBe(false);
    expect(apparatusRunsCommand("centrifuge v1 3000rpm 60s 8cm", "centrifuge", null)).toBe(false);
  });
});
