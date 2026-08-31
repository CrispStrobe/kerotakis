import { describe, expect, it } from "vitest";
import {
  eventSecuresCriterion,
  outcomeComplete,
  outcomeMissionContract,
  resolvedComponents,
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

  it("assesses the separation lead from the solver's own peak table", () => {
    const separation = outcomeMissionContract("one-thing-at-a-time")!;
    // The lesson's own mixture, as the school column actually elutes it.
    const resolved = secureOutcomeEvidence(separation, [], [{
      event: "chromatographed",
      plates: 10000,
      peaks: [
        { species: "methanol", retention_time_s: 63, width_s: 2.5 },
        { species: "ethanol", retention_time_s: 68, width_s: 2.7 },
        { species: "propanone", retention_time_s: 115, width_s: 4.6 },
      ],
    }]);
    expect(outcomeComplete(separation, resolved)).toBe(true);
  });

  it("rejects a failed separation: co-eluting peaks count as one component", () => {
    const criterion = outcomeMissionContract("one-thing-at-a-time")!.criteria[0]!;
    // Three peaks on paper, but the first two overlap (resolution < 1):
    // the trace shows two components, and the contract must agree.
    expect(eventSecuresCriterion(criterion, {
      event: "chromatographed",
      peaks: [
        { species: "methanol", retention_time_s: 63, width_s: 6 },
        { species: "ethanol", retention_time_s: 66, width_s: 6 },
        { species: "propanone", retention_time_s: 115, width_s: 4.6 },
      ],
    })).toBe(false);
    expect(eventSecuresCriterion(criterion, { event: "chromatographed", peaks: [] })).toBe(false);
    expect(eventSecuresCriterion(criterion, { event: "filtered", from: 1, to: 2 })).toBe(false);
  });

  it("clusters overlapping peaks by chromatographic resolution", () => {
    expect(resolvedComponents([
      { retention_time_s: 63, width_s: 2.5 },
      { retention_time_s: 68, width_s: 2.7 },
      { retention_time_s: 115, width_s: 4.6 },
    ])).toBe(3);
    expect(resolvedComponents([
      { retention_time_s: 63, width_s: 6 },
      { retention_time_s: 66, width_s: 6 },
    ])).toBe(1);
    expect(resolvedComponents([])).toBe(0);
  });

  it("assesses the safety audit on the engine's typed rule ids, never prose", () => {
    const audit = outcomeMissionContract("never-mix")!;
    // German-localized prose around the stable rule id — exactly what a
    // de-locale session's events look like. The id is what must match.
    const secured = secureOutcomeEvidence(audit, [], [
      { event: "hazard_warning", rule: "bleach-ammonia-chloramine", hazard: "Bleiche und Ammoniak…", real_world: "…" },
      { event: "hazard_warning", rule: "oxidizer-flammable-liquid", hazard: "…", real_world: "…" },
      { event: "hazard_warning", rule: "acid-metal-hydrogen", hazard: "…", real_world: "…" },
      { event: "hazard_warning", rule: "acid-carbonate-co2", hazard: "…", real_world: "…" },
    ]);
    expect(outcomeComplete(audit, secured)).toBe(true);
  });

  it("safety-audit evidence accumulates one hazard at a time and ignores impostors", () => {
    const audit = outcomeMissionContract("never-mix")!;
    const first = secureOutcomeEvidence(audit, [], [
      { event: "hazard_warning", rule: "bleach-ammonia-chloramine", hazard: "…", real_world: "…" },
    ]);
    expect(first).toEqual(["toxic-gas-chloramine"]);
    expect(outcomeComplete(audit, first)).toBe(false);
    // A different rule, a rule-less warning (old snapshot), and prose that
    // merely MENTIONS chloramine must not secure anything further.
    const second = secureOutcomeEvidence(audit, first, [
      { event: "hazard_warning", rule: "water-reactive-slaking", hazard: "…", real_world: "…" },
      { event: "hazard_warning", hazard: "mixing bleach with ammonia makes chloramine, a toxic gas", real_world: "…" },
    ]);
    expect(second).toEqual(["toxic-gas-chloramine"]);
  });
});
