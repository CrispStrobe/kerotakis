/**
 * Consumer types for the GUI-092 net ionic contract
 * (`kerotakis-core/src/ionic.rs` is authoritative; PROTOCOL.md documents
 * it).
 *
 * The molecular equation says which bottles were opened. This says what
 * happened, and it is derived from the solved speciation rather than
 * stored per reaction — so the shell's only job is to display the line the
 * engine assembled and never to assemble one of its own. A step with no
 * `ionic` entry is a step for which the engine had nothing honest to say;
 * the strip then shows the molecular equation alone.
 */

export type IonicBasis = "precipitation" | "neutralisation";

export interface IonTerm {
  /** The engine's own name, in PHREEQC notation: `Ag+`, `SO4-2`. */
  species: string;
  /** The same thing typeset for a reader: `Ag⁺`, `SO₄²⁻`. Display this. */
  label: string;
  coefficient: number;
  charge: number;
  phase: "solid" | "liquid" | "aqueous" | "gas";
}

export interface NetIonic {
  vessel: number;
  basis: IonicBasis;
  reactants: IonTerm[];
  products: IonTerm[];
  /** Charged species the solver left in solution, most abundant first. */
  spectators: IonTerm[];
  /** The assembled line: `Ag⁺(aq) + Cl⁻(aq) → AgCl(s)`. */
  equation: string;
  provenance?: string;
}

/** Whether an unknown value looks like the ionic contract. */
export function isNetIonic(v: unknown): v is NetIonic {
  const n = v as NetIonic;
  return (
    typeof n === "object" &&
    n !== null &&
    typeof n.equation === "string" &&
    n.equation.length > 0 &&
    (n.basis === "precipitation" || n.basis === "neutralisation") &&
    Array.isArray(n.reactants) &&
    Array.isArray(n.products)
  );
}

/**
 * The net ionic equation a batch of steps ended on, or null.
 *
 * The last one wins, for the same reason the molecular strip pins the
 * latest rendered equation: the strip shows what the bench is doing now.
 * A malformed entry is skipped rather than shown — the strip is a claim
 * about chemistry and half a claim is worse than none.
 */
export function latestNetIonic(
  steps: ReadonlyArray<{ ionic?: unknown }>,
): NetIonic | null {
  let latest: NetIonic | null = null;
  for (const step of steps) {
    if (!Array.isArray(step?.ionic)) continue;
    for (const entry of step.ionic) {
      if (isNetIonic(entry)) latest = entry;
    }
  }
  return latest;
}

/** `Na⁺, NO₃⁻`, or null where the solver left nothing beside the reaction. */
export function spectatorPhrase(net: NetIonic): string | null {
  if (!Array.isArray(net.spectators) || net.spectators.length === 0) {
    return null;
  }
  return net.spectators.map((t) => t.label).join(", ");
}
