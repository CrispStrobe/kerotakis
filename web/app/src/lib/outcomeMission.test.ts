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

  it("assesses the thermal baseline from solver-emitted mixing temperatures", () => {
    const thermal = outcomeMissionContract("first-warmth")!;
    const result = secureOutcomeEvidence(thermal, [], [{
      event: "mixed",
      fraction_a: 0.5,
      fraction_b: 0.5,
      temperature_a: 293.15,
      temperature_b: 353.15,
      temperature_into: 323.15,
    }]);
    expect(outcomeComplete(thermal, result)).toBe(true);
  });

  it("rejects isothermal, endpoint, tiny-stream, and untyped thermal claims", () => {
    const criterion = outcomeMissionContract("first-warmth")!.criteria[0]!;
    const base = { event: "mixed", fraction_a: 0.5, fraction_b: 0.5, temperature_a: 293.15, temperature_b: 353.15 };
    expect(eventSecuresCriterion(criterion, { ...base, temperature_into: 323.15 })).toBe(true);
    expect(eventSecuresCriterion(criterion, { ...base, temperature_b: 298.15, temperature_into: 295.15 })).toBe(false);
    expect(eventSecuresCriterion(criterion, { ...base, temperature_into: 293.15 })).toBe(false);
    expect(eventSecuresCriterion(criterion, { ...base, fraction_a: 0.01, temperature_into: 352.55 })).toBe(false);
    expect(eventSecuresCriterion(criterion, { event: "temperature_changed", from: 293.15, to: 323.15 })).toBe(false);
  });
});
