export type OutcomeCriterion = {
  id: string;
  label: string;
  /** The engine event that can secure this criterion. Absent on criteria
   * that read the whole event list rather than one event. */
  event?: string;
  species?: string;
  amountField?: string;
  minimum?: number;
  thermalMix?: {
    minimumSourceDeltaK: number;
    minimumFraction: number;
  };
  /** Matches the engine's stable hazard rule id (`HazardWarning.rule`) —
   * never its prose, which is localized on the way out of the engine. */
  hazardRule?: string;
  /** Secured by a chromatogram whose peak table shows at least this many
   * BASELINE-RESOLVED components: co-eluting peaks (resolution < 1)
   * merge into one apparent component, exactly as they would on the
   * recorder trace, so a failed separation cannot pass on peak count. */
  chromatography?: {
    minimumResolvedComponents: number;
  };
  /** Aggregate over every `partitioned` event: the funnel discriminated
   * between solutes rather than moving them all together. `minimumSpread`
   * is the gap between the most- and least-aqueous solute's share of the
   * lower layer, so a solvent that carried the whole sample across —
   * which separates nothing — cannot pass. */
  partitionSpread?: {
    minimumSolutes: number;
    minimumSpread: number;
  };
};

/**
 * One materially different way to satisfy a mission.
 *
 * A route is an AND of its criteria; a mission is an OR of its routes. That
 * is the whole combinator, and it is deliberately the smallest one that can
 * express "two valid solutions": a learner who separates a mixture on a
 * column and a learner who separates it in a funnel have both separated the
 * mixture, and the contract must not prefer the one the author happened to
 * think of first.
 */
export type OutcomeRoute = {
  id: string;
  /** Names the approach, not the procedure — shown when a mission offers
   * more than one route so the learner knows alternatives exist. */
  label: string;
  criteria: OutcomeCriterion[];
};

export type OutcomeMissionContract = {
  missionId: string;
  objective: string;
  brief: string;
  hint: string;
  extraKit: string[];
  /** Equipment verbs this mission loans beyond those its script uses.
   * A route the script never walks still needs its apparatus on the wall. */
  extraTools: string[];
  routes: OutcomeRoute[];
};

const CONTRACTS: Record<string, OutcomeMissionContract> = {
  "silver-and-salt": {
    missionId: "silver-and-salt",
    objective: "Produce observable silver chloride to confirm chloride in the sample.",
    brief: "Plan your own setup. The solver will assess the chemical result, not whether you followed one recipe.",
    hint: "Sodium chloride and potassium chloride are different materials, but both supply chloride ions. Silver nitrate can make that shared ion visible.",
    extraKit: ["KCl"],
    extraTools: [],
    routes: [
      {
        id: "precipitation",
        label: "by precipitation",
        criteria: [
          {
            id: "observable-agcl",
            label: "Observable silver chloride formed",
            event: "precipitated",
            species: "AgCl",
            amountField: "moles",
            minimum: 1e-6,
          },
        ],
      },
    ],
  },
  "first-warmth": {
    missionId: "first-warmth",
    objective: "Mix two water samples at meaningfully different temperatures and obtain an intermediate temperature.",
    brief: "Build the setup your way. The solver will verify the source temperatures and the adiabatic mixing result.",
    hint: "Prepare water in two separate vessels, make one warmer or cooler than the other, create an empty receiver, then use the mixer from the instrument wall.",
    extraKit: [],
    extraTools: [],
    routes: [
      {
        id: "adiabatic-mix",
        label: "by mixing",
        criteria: [
          {
            id: "thermal-middle",
            label: "Different-temperature samples mixed to a computed middle",
            event: "mixed",
            thermalMix: { minimumSourceDeltaK: 10, minimumFraction: 0.1 },
          },
        ],
      },
    ],
  },
  "one-thing-at-a-time": {
    missionId: "one-thing-at-a-time",
    // Two routes, so the objective states the RESULT and lets each route
    // state its own bar: a column that resolves the components, or a
    // funnel that sends them into different layers. Both are separations;
    // neither is the "intended" one.
    objective: "Separate the colourless sample into components the instruments can tell apart.",
    brief: "Two different separations both count here: resolve the sample on the column, or split it between two liquid layers. The solver reads the instrument output, not your procedure.",
    hint: "The column separates dissolved solutes by how long each is held back. A separating funnel does it another way: add a solvent that does not mix with water — at least as much of it as you have sample — and each solute divides between the layers by its own partition coefficient.",
    extraKit: ["hexane"],
    extraTools: ["drain"],
    routes: [
      {
        id: "column",
        label: "on the column",
        criteria: [
          {
            id: "resolved-components",
            label: "Three components resolved from one sample",
            event: "chromatographed",
            chromatography: { minimumResolvedComponents: 3 },
          },
        ],
      },
      {
        id: "extraction",
        label: "in the separating funnel",
        criteria: [
          {
            id: "layers-drawn-off",
            label: "The lower layer drawn off into its own vessel",
            event: "drained",
          },
          {
            id: "partition-discriminates",
            label: "Solutes left in materially different layers",
            // 0.15 is measured, not chosen. Against the lesson's own sample
            // (100 mL water, methanol/ethanol/propanone) the engine's spread
            // rises with the extracting solvent: 10 mL → 0.024, 25 → 0.057,
            // 50 → 0.107, 100 → 0.190, 200 → 0.313, 500 → 0.504. The bar
            // therefore sits between a token splash and a real extraction of
            // roughly sample-sized volume — which is the lesson.
            partitionSpread: { minimumSolutes: 2, minimumSpread: 0.15 },
          },
        ],
      },
    ],
  },
  "never-mix": {
    missionId: "never-mix",
    objective: "Document four distinct dangerous combinations before anyone handles the abandoned bench.",
    brief: "Reproduce each hazard in the virtual lab and let the safety layer name it. The solver accepts only its own typed warnings — one per hazard class.",
    hint: "The abandoned bench holds bleach, ammonia, a strong oxidizer, a flammable liquid, a strong acid, an active metal, and a carbonate. Four of their pairings are the classics every lab warns about.",
    extraKit: [],
    extraTools: [],
    routes: [
      {
        id: "hazard-audit",
        label: "by reproducing each hazard",
        criteria: [
          {
            id: "toxic-gas-chloramine",
            label: "Bleach + ammonia: chloramine gas identified",
            event: "hazard_warning",
            hazardRule: "bleach-ammonia-chloramine",
          },
          {
            id: "oxidizer-flammable",
            label: "Strong oxidizer + flammable liquid: fire risk identified",
            event: "hazard_warning",
            hazardRule: "oxidizer-flammable-liquid",
          },
          {
            id: "hydrogen-from-acid",
            label: "Strong acid + active metal: hydrogen evolution identified",
            event: "hazard_warning",
            hazardRule: "acid-metal-hydrogen",
          },
          {
            id: "carbonate-spatter",
            label: "Strong acid + carbonate: CO₂ spattering identified",
            event: "hazard_warning",
            hazardRule: "acid-carbonate-co2",
          },
        ],
      },
    ],
  },
};

/** Only missions with a typed outcome contract leave the procedural player. */
export function outcomeMissionContract(id: string): OutcomeMissionContract | null {
  return CONTRACTS[id] ?? null;
}

/** Every criterion the contract can secure, across all its routes. */
export function allCriteria(contract: OutcomeMissionContract): OutcomeCriterion[] {
  return contract.routes.flatMap((route) => route.criteria);
}

/** Match one engine event against one criterion. Amount thresholds are part of
 * the contract so a merely mathematical trace cannot masquerade as visible
 * evidence. */
export function eventSecuresCriterion(
  criterion: OutcomeCriterion,
  event: Record<string, unknown>,
): boolean {
  if (criterion.event === undefined) return false;
  if (event.event !== criterion.event) return false;
  if (criterion.species !== undefined && event.species !== criterion.species) return false;
  if (criterion.amountField !== undefined) {
    const amount = Number(event[criterion.amountField]);
    if (!Number.isFinite(amount) || amount < (criterion.minimum ?? 0)) return false;
  }
  if (criterion.hazardRule !== undefined && event.rule !== criterion.hazardRule) return false;
  if (criterion.chromatography !== undefined) {
    const peaks = Array.isArray(event.peaks) ? (event.peaks as Record<string, unknown>[]) : [];
    if (resolvedComponents(peaks) < criterion.chromatography.minimumResolvedComponents) return false;
  }
  if (criterion.thermalMix !== undefined) {
    const a = Number(event.temperature_a);
    const b = Number(event.temperature_b);
    const into = Number(event.temperature_into);
    const fractionA = Number(event.fraction_a);
    const fractionB = Number(event.fraction_b);
    if (![a, b, into, fractionA, fractionB].every(Number.isFinite)) return false;
    if (Math.abs(a - b) < criterion.thermalMix.minimumSourceDeltaK) return false;
    if (fractionA < criterion.thermalMix.minimumFraction || fractionB < criterion.thermalMix.minimumFraction) return false;
    const low = Math.min(a, b);
    const high = Math.max(a, b);
    // A real contribution from both streams must place the computed result
    // strictly inside their temperatures, not merely on an endpoint.
    if (!(into > low + 0.01 && into < high - 0.01)) return false;
  }
  return true;
}

/**
 * Is this criterion secured by the run so far?
 *
 * Most criteria are secured by a single event. A criterion that reads the
 * whole list — the partition spread — is answering a question no one event
 * can: whether the separation actually told two solutes apart.
 */
export function criterionSecured(
  criterion: OutcomeCriterion,
  events: readonly Record<string, unknown>[],
): boolean {
  if (criterion.partitionSpread !== undefined) {
    return partitionSpreadSecures(criterion.partitionSpread, events);
  }
  return events.some((event) => eventSecuresCriterion(criterion, event));
}

/** The spread between the most- and least-aqueous solute the funnel divided.
 * Each `drain` emits one `partitioned` event per solute; a later drain of the
 * same vessel re-reports the ones that stayed, so the widest spread any single
 * separation achieved is the honest reading. */
function partitionSpreadSecures(
  wanted: { minimumSolutes: number; minimumSpread: number },
  events: readonly Record<string, unknown>[],
): boolean {
  const fractions = new Map<string, number[]>();
  for (const event of events) {
    if (event.event !== "partitioned") continue;
    const species = typeof event.species === "string" ? event.species : null;
    const fraction = Number(event.fraction_lower);
    if (species === null || !Number.isFinite(fraction)) continue;
    fractions.set(species, [...(fractions.get(species) ?? []), fraction]);
  }
  if (fractions.size < wanted.minimumSolutes) return false;
  const firsts = [...fractions.values()].map((seen) => seen[0]!);
  return Math.max(...firsts) - Math.min(...firsts) >= wanted.minimumSpread;
}

/** How many separately visible components a peak table shows. Peaks whose
 * chromatographic resolution R = 2·Δt_R/(w₁+w₂) is under 1 co-elute into
 * one apparent component on the trace, so they count once — a column run
 * that failed to separate cannot pass on raw peak count. */
export function resolvedComponents(peaks: readonly Record<string, unknown>[]): number {
  const ordered = peaks
    .map((peak) => ({
      retention: Number(peak.retention_time_s),
      width: Number(peak.width_s),
    }))
    .filter((peak) => Number.isFinite(peak.retention) && Number.isFinite(peak.width))
    .sort((a, b) => a.retention - b.retention);
  let components = 0;
  let previous: { retention: number; width: number } | null = null;
  for (const peak of ordered) {
    if (previous === null) {
      components = 1;
    } else {
      const resolution = (2 * (peak.retention - previous.retention)) / (previous.width + peak.width);
      if (resolution >= 1) components += 1;
    }
    previous = peak;
  }
  return components;
}

/** Add newly secured criterion ids without losing prior evidence or allowing
 * duplicate events to inflate progress. */
export function secureOutcomeEvidence(
  contract: OutcomeMissionContract,
  secured: readonly string[],
  events: readonly Record<string, unknown>[],
): string[] {
  const next = new Set(secured);
  for (const criterion of allCriteria(contract)) {
    if (criterionSecured(criterion, events)) next.add(criterion.id);
  }
  return [...next];
}

/** The route the learner actually completed, or null while none is complete. */
export function completedRoute(
  contract: OutcomeMissionContract,
  secured: readonly string[],
): OutcomeRoute | null {
  const found = new Set(secured);
  return contract.routes.find((route) => route.criteria.every((c) => found.has(c.id))) ?? null;
}

export function outcomeComplete(contract: OutcomeMissionContract, secured: readonly string[]): boolean {
  return completedRoute(contract, secured) !== null;
}

/**
 * Progress against the route the learner is closest to finishing.
 *
 * Counting every criterion across every route would punish a learner for the
 * existence of an alternative they did not take: three of six, when they are
 * one step from done. The nearest route is the honest denominator.
 */
export function routeProgress(
  contract: OutcomeMissionContract,
  secured: readonly string[],
): { route: OutcomeRoute; done: number; total: number } {
  const found = new Set(secured);
  const scored = contract.routes.map((route) => ({
    route,
    done: route.criteria.filter((c) => found.has(c.id)).length,
    total: route.criteria.length,
  }));
  // Fewest steps remaining wins; where two routes are equally close, the one
  // the learner has actually started does. Without that tie-break, securing
  // the first of the funnel's two criteria still reports progress against the
  // untouched column — telling a learner who is halfway that they are nowhere.
  return scored.reduce((best, candidate) => {
    const remaining = candidate.total - candidate.done;
    const bestRemaining = best.total - best.done;
    if (remaining !== bestRemaining) return remaining < bestRemaining ? candidate : best;
    return candidate.done > best.done ? candidate : best;
  });
}
