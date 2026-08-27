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
