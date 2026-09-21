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
 *
 * That contract holds for the complete equation too (GUI-092,
 * 2026-09-21), and it is the reason the strike-through is drawn from a
 * FLAG the engine set rather than from anything the shell works out. The
 * spectators' coefficients are solved in the engine and verified there; a
 * complete equation is absent wherever they could not be, and absent means
 * the net line stands alone — never that the shell should fill the gap.
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
  /**
   * Set by the engine on a term that stands on BOTH sides of a complete
   * ionic equation, unchanged. This is the strike-through — as a flag, not
   * as markup. Absent (not `false`) on every term outside `complete`,
   * which is why it is optional here.
   */
  spectator?: boolean;
}

/**
 * The complete ionic equation (GUI-092, 2026-09-21).
 *
 * The net equation says what happened; this says why it is what it is,
 * because a cancellation nobody was shown is not one they can follow. The
 * coefficients in here are SOLVED — every ion the net equation consumes
 * arrived beside a counter-ion in the number that made its salt neutral,
 * so a barium sulfate carries `2 Na⁺` and `2 Cl⁻` and a silver chloride
 * carries one of each.
 *
 * It is absent wherever the engine could not solve and verify them, and
 * that is a correct answer rather than a gap: the net line is the honest
 * fallback. The shell must never build one out of `NetIonic.spectators` —
 * that list is selected by abundance and its coefficients mean nothing.
 */
export interface CompleteIonic {
  reactants: IonTerm[];
  products: IonTerm[];
  /** The whole line, nothing struck through — the engine's own text. */
  equation: string;
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
  /** The same reaction written out in full. Absent is the common case. */
  complete?: CompleteIonic;
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

/**
 * The complete ionic equation an entry carries, or null.
 *
 * Null covers three cases and treats them alike, which is the point: the
 * engine emitted none, the engine is older than the field, or what arrived
 * does not keep the contract. A shell that "repaired" the third would be
 * assembling an equation of its own, and the one job this module has is
 * not to.
 *
 * The checks are the promises the engine makes: both sides are term lists,
 * something is actually struck through, and each side is electrically
 * neutral — because each side of a complete ionic equation is a statement
 * about bottles, and a bottle does not carry a charge.
 */
export function completeIonic(net: NetIonic): CompleteIonic | null {
  const c = net?.complete;
  if (!c || typeof c.equation !== "string" || c.equation.length === 0) {
    return null;
  }
  if (!Array.isArray(c.reactants) || !Array.isArray(c.products)) return null;
  const sound = (side: IonTerm[]) =>
    side.every(
      (t) =>
        typeof t?.label === "string" &&
        Number.isFinite(t.coefficient) &&
        Number.isFinite(t.charge),
    );
  if (!sound(c.reactants) || !sound(c.products)) return null;
  if (!c.reactants.some((t) => t.spectator)) return null;
  const charge = (side: IonTerm[]) =>
    side.reduce((n, t) => n + t.charge * t.coefficient, 0);
  if (charge(c.reactants) !== 0 || charge(c.products) !== 0) return null;
  return c;
}

const PHASE_SUFFIX: Record<IonTerm["phase"], string> = {
  aqueous: "(aq)",
  solid: "(s)",
  liquid: "(l)",
  gas: "(g)",
};

/**
 * One term as it is read: `2 OH⁻(aq)`.
 *
 * This is typography over fields the engine set — a coefficient, a
 * typeset label and a phase — and not a second opinion about chemistry.
 * It exists because a line that is struck through in places has to be
 * drawn term by term, and `equation` is one string.
 */
export function writtenTerm(term: IonTerm): string {
  const coefficient = term.coefficient === 1 ? "" : `${term.coefficient} `;
  return `${coefficient}${term.label}${PHASE_SUFFIX[term.phase] ?? ""}`;
}

/** `Na⁺, NO₃⁻`, or null where the solver left nothing beside the reaction. */
export function spectatorPhrase(net: NetIonic): string | null {
  if (!Array.isArray(net.spectators) || net.spectators.length === 0) {
    return null;
  }
  return net.spectators.map((t) => t.label).join(", ");
}
