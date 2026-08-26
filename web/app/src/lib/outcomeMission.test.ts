import { describe, expect, it } from "vitest";
import {
  eventSecuresCriterion,
  outcomeComplete,
  outcomeMissionContract,
  secureOutcomeEvidence,
} from "./outcomeMission";

describe("open-ended mission outcomes", () => {
  const contract = outcomeMissionContract("silver-and-salt")!;
  const criterion = contract.criteria[0]!;

  it("accepts the shared computed result independent of the chloride source", () => {
    const sodiumRoute = secureOutcomeEvidence(contract, [], [
      { event: "dissolved", species: "NaCl", moles: 0.01 },
      { event: "precipitated", species: "AgCl", moles: 0.0099 },
    ]);
    const potassiumRoute = secureOutcomeEvidence(contract, [], [
      { event: "dissolved", species: "KCl", moles: 0.01 },
      { event: "precipitated", species: "AgCl", moles: 0.0099 },
    ]);

    expect(outcomeComplete(contract, sodiumRoute)).toBe(true);
    expect(outcomeComplete(contract, potassiumRoute)).toBe(true);
    expect(contract.extraKit).toContain("KCl");
  });

  it("rejects direct solid placement, the wrong precipitate, and sub-visible traces", () => {
    expect(eventSecuresCriterion(criterion, { event: "added", species: "AgCl", moles: 0.01 })).toBe(false);
    expect(eventSecuresCriterion(criterion, { event: "precipitated", species: "CaCO3", moles: 0.01 })).toBe(false);
    expect(eventSecuresCriterion(criterion, { event: "precipitated", species: "AgCl", moles: 0.5e-6 })).toBe(false);
  });

  it("counts repeated qualifying events only once", () => {
    const secured = secureOutcomeEvidence(contract, ["observable-agcl"], [
      { event: "precipitated", species: "AgCl", moles: 0.02 },
    ]);
    expect(secured).toEqual(["observable-agcl"]);
  });
});
