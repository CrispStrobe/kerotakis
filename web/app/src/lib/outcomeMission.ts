export type OutcomeCriterion = {
  id: string;
  label: string;
  event: string;
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
};

export type OutcomeMissionContract = {
  missionId: string;
  objective: string;
  brief: string;
  hint: string;
  extraKit: string[];
  criteria: OutcomeCriterion[];
};

const CONTRACTS: Record<string, OutcomeMissionContract> = {
  "silver-and-salt": {
    missionId: "silver-and-salt",
    objective: "Produce observable silver chloride to confirm chloride in the sample.",
    brief: "Plan your own setup. The solver will assess the chemical result, not whether you followed one recipe.",
    hint: "Sodium chloride and potassium chloride are different materials, but both supply chloride ions. Silver nitrate can make that shared ion visible.",
    extraKit: ["KCl"],
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
  "first-warmth": {
    missionId: "first-warmth",
    objective: "Mix two water samples at meaningfully different temperatures and obtain an intermediate temperature.",
    brief: "Build the setup your way. The solver will verify the source temperatures and the adiabatic mixing result.",
    hint: "Prepare water in two separate vessels, make one warmer or cooler than the other, create an empty receiver, then use the mixer from the instrument wall.",
    extraKit: [],
    criteria: [
      {
        id: "thermal-middle",
        label: "Different-temperature samples mixed to a computed middle",
        event: "mixed",
        thermalMix: { minimumSourceDeltaK: 10, minimumFraction: 0.1 },
      },
    ],
  },
  "one-thing-at-a-time": {
    missionId: "one-thing-at-a-time",
    objective: "Resolve the colourless sample into at least three separately measured components.",
    brief: "Any separation that produces distinct measured components counts. The solver reads the actual peak table, not your procedure.",
    hint: "The chromatography column separates dissolved neutral solutes by their partition coefficients — a mixture that looks like one liquid arrives as one peak per component, if the column resolves them.",
    extraKit: [],
    criteria: [
      {
        id: "resolved-components",
        label: "Three components resolved from one sample",
        event: "chromatographed",
        chromatography: { minimumResolvedComponents: 3 },
      },
    ],
  },
  "never-mix": {
    missionId: "never-mix",
    objective: "Document four distinct dangerous combinations before anyone handles the abandoned bench.",
    brief: "Reproduce each hazard in the virtual lab and let the safety layer name it. The solver accepts only its own typed warnings — one per hazard class.",
    hint: "The abandoned bench holds bleach, ammonia, a strong oxidizer, a flammable liquid, a strong acid, an active metal, and a carbonate. Four of their pairings are the classics every lab warns about.",
    extraKit: [],
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
};

/** Only missions with a typed outcome contract leave the procedural player. */
export function outcomeMissionContract(id: string): OutcomeMissionContract | null {
  return CONTRACTS[id] ?? null;
}

/** Match one engine event against one criterion. Amount thresholds are part of
 * the contract so a merely mathematical trace cannot masquerade as visible
 * evidence. */
export function eventSecuresCriterion(
  criterion: OutcomeCriterion,
  event: Record<string, unknown>,
): boolean {
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
  for (const criterion of contract.criteria) {
    if (events.some((event) => eventSecuresCriterion(criterion, event))) next.add(criterion.id);
  }
  return [...next];
}

export function outcomeComplete(contract: OutcomeMissionContract, secured: readonly string[]): boolean {
  const found = new Set(secured);
  return contract.criteria.every((criterion) => found.has(criterion.id));
}
