import { describe, expect, it } from "vitest";
import { buretteTargetAfterChoice, deploymentAfterChoice } from "./apparatusTarget";

describe("physical apparatus targets", () => {
  it("moves explicitly to a newly selected vessel and toggles off on its target", () => {
    expect(deploymentAfterChoice("stir", 0, "stir", 2)).toEqual({ tool: "stir", target: 2 });
    expect(deploymentAfterChoice("stir", 2, "stir", 2)).toEqual({ tool: null, target: null });
    expect(deploymentAfterChoice("stir", 0, "heat", 1)).toEqual({ tool: "heat", target: 1 });
  });

  it("applies the same explicit move/toggle rule to the burette", () => {
    expect(buretteTargetAfterChoice(0, 2)).toBe(2);
    expect(buretteTargetAfterChoice(2, 2)).toBeNull();
  });
});
