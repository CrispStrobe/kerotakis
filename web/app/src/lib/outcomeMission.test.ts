import { describe, expect, it } from "vitest";
import {
  allCriteria,
  completedRoute,
  criterionSecured,
  eventSecuresCriterion,
  outcomeComplete,
  outcomeMissionContract,
  resolvedComponents,
  routeProgress,
  secureOutcomeEvidence,
} from "./outcomeMission";

describe("open-ended mission outcomes", () => {
  const contract = outcomeMissionContract("silver-and-salt")!;
  const criterion = contract.routes[0]!.criteria[0]!;

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
    const criterion = outcomeMissionContract("first-warmth")!.routes[0]!.criteria[0]!;
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
    const criterion = outcomeMissionContract("one-thing-at-a-time")!.routes[0]!.criteria[0]!;
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

describe("two materially different valid solutions (GUI-080)", () => {
  const separation = outcomeMissionContract("one-thing-at-a-time")!;
  // What the funnel actually reports for the lesson's own mixture: each
  // solute divides on its own partition coefficient, so the shares differ.
  const funnel = [
    { event: "partitioned", species: "methanol", fraction_lower: 0.96 },
    { event: "partitioned", species: "ethanol", fraction_lower: 0.84 },
    { event: "partitioned", species: "propanone", fraction_lower: 0.41 },
    { event: "drained", from: 1, to: 2, solvent: "water", moles: 5.4 },
  ];

  it("offers a column route and a funnel route, and neither is the intended one", () => {
    expect(separation.routes.map((route) => route.id)).toEqual(["column", "extraction"]);
    expect(separation.extraKit).toContain("hexane");
    expect(separation.extraTools).toContain("drain");
  });

  it("completes on the extraction route with no chromatogram at all", () => {
    const secured = secureOutcomeEvidence(separation, [], funnel);
    expect(outcomeComplete(separation, secured)).toBe(true);
    expect(completedRoute(separation, secured)?.id).toBe("extraction");
    expect(secured).not.toContain("resolved-components");
  });

  it("completes on the column route with no funnel at all", () => {
    const secured = secureOutcomeEvidence(separation, [], [{
      event: "chromatographed",
      peaks: [
        { species: "methanol", retention_time_s: 63, width_s: 2.5 },
        { species: "ethanol", retention_time_s: 68, width_s: 2.7 },
        { species: "propanone", retention_time_s: 115, width_s: 4.6 },
      ],
    }]);
    expect(outcomeComplete(separation, secured)).toBe(true);
    expect(completedRoute(separation, secured)?.id).toBe("column");
  });

  it("refuses a solvent that carried the whole sample across — that separates nothing", () => {
    const carried = [
      { event: "partitioned", species: "methanol", fraction_lower: 0.93 },
      { event: "partitioned", species: "ethanol", fraction_lower: 0.9 },
      { event: "drained", from: 1, to: 2, solvent: "water", moles: 5.4 },
    ];
    const secured = secureOutcomeEvidence(separation, [], carried);
    expect(secured).toContain("layers-drawn-off");
    expect(secured).not.toContain("partition-discriminates");
    expect(outcomeComplete(separation, secured)).toBe(false);
  });

  it("passes a real equal-volume extraction and refuses a token splash", () => {
    // Both readings come from the engine itself, running the lesson's own
    // sample through `drain` (100 mL water; hexane as marked). The threshold
    // has to fall between them, or the mission teaches the wrong lesson
    // about how much extracting solvent a separation needs.
    const spread = allCriteria(separation).find((c) => c.partitionSpread)!;
    const measured = (fractions: Record<string, number>) =>
      Object.entries(fractions).map(([species, fraction_lower]) => ({
        event: "partitioned", species, fraction_lower,
      }));
    // 100 mL hexane: spread 0.190.
    expect(criterionSecured(spread, measured({
      methanol: 0.9884828398233436, ethanol: 0.9632931020927237, propanone: 0.7980447115886924,
    }))).toBe(true);
    // 50 mL hexane: spread 0.107 — the layers barely discriminate.
    expect(criterionSecured(spread, measured({
      methanol: 0.9942080665993177, ethanol: 0.9813034040265565, propanone: 0.8876806082130925,
    }))).toBe(false);
  });

  it("refuses a single solute, however cleanly it partitions", () => {
    const spread = allCriteria(separation).find((c) => c.partitionSpread)!;
    expect(criterionSecured(spread, [
      { event: "partitioned", species: "propanone", fraction_lower: 0.05 },
    ])).toBe(false);
  });

  it("reads the first report per solute, so re-draining cannot manufacture a spread", () => {
    const spread = allCriteria(separation).find((c) => c.partitionSpread)!;
    // Second drain of the same vessel: what stayed behind reports again,
    // now against a different remaining volume. One separation, one reading.
    expect(criterionSecured(spread, [
      { event: "partitioned", species: "ethanol", fraction_lower: 0.84 },
      { event: "partitioned", species: "ethanol", fraction_lower: 0.12 },
    ])).toBe(false);
  });

  it("draining without partitioning secures only the physical half", () => {
    const secured = secureOutcomeEvidence(separation, [], [
      { event: "drained", from: 1, to: 2, solvent: "water", moles: 5.4 },
    ]);
    expect(secured).toEqual(["layers-drawn-off"]);
  });

  it("counts progress against the nearest route, not every route at once", () => {
    // One of two extraction criteria secured: 1/2 on that route, never 1/3
    // of a combined pile that includes the column the learner did not use.
    const partial = routeProgress(separation, ["layers-drawn-off"]);
    expect(partial.route.id).toBe("extraction");
    expect([partial.done, partial.total]).toEqual([1, 2]);
    // Nothing secured: the shortest route is the honest denominator.
    expect(routeProgress(separation, []).total).toBe(1);
  });

  it("keeps every criterion id distinct across routes in every contract", () => {
    for (const id of ["silver-and-salt", "first-warmth", "one-thing-at-a-time", "never-mix"]) {
      const ids = allCriteria(outcomeMissionContract(id)!).map((c) => c.id);
      expect(new Set(ids).size, `${id} reuses a criterion id across routes`).toBe(ids.length);
    }
  });

  it("leaves the single-route missions single-route", () => {
    for (const id of ["silver-and-salt", "first-warmth", "never-mix"]) {
      expect(outcomeMissionContract(id)!.routes).toHaveLength(1);
    }
  });
});
