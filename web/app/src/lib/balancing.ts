/**
 * GUI-095 — balancing as a *generated* exercise.
 *
 * The engine balances by the null space of the element-and-charge count
 * matrix, so practice does not have to be authored: take any equation the
 * codex or a session produced, strip its coefficients, and mark whatever the
 * learner writes against the matrix the solver used. Unlimited, never wrong,
 * and with no question bank to maintain.
 *
 * The marking is the part worth being careful about, and it is the reason
 * the `balance` protocol command returns the matrix and not just the answer.
 * Comparing a learner's coefficients to the solver's would mark
 * `4 Mg + 2 O₂ → 4 MgO` as wrong. It is not wrong — it balances — it is
 * merely not the smallest whole-number ratio, and saying *that* is the whole
 * lesson. So the mark is arithmetic over the learner's own vector:
 *
 *   - every entry a positive integer, or the answer is not yet an answer;
 *   - `matrix · v = 0`, or it does not balance, and each row that fails says
 *     which element (or the charge) is out and by how much;
 *   - `gcd(v) = 1`, or it balances at a multiple of the simplest ratio.
 *
 * In a one-dimensional null space those three are complete: every positive
 * integer solution is `k` times the smallest one, so `gcd` recovers exactly
 * the `k` that the "correct multiple" verdict names. Where the skeleton is
 * underdetermined — `C + O₂ → CO + CO₂` is two independent reactions wearing
 * one arrow — there is no single right answer to compare against at all, and
 * the same three tests still hold: any primitive positive vector in the null
 * space is a correct balancing of a real reaction. That case is marked
 * correct and *named*, because a learner who has found one member of a
 * family should be told the family exists rather than left believing they
 * found the answer.
 */
import type { BalanceReport } from "./host/EngineHost";

/** What a marked answer turned out to be. */
export type BalanceVerdict =
  /** Balances, in the smallest whole-number ratio. */
  | "correct"
  /** Balances, but every coefficient shares a factor. The actual lesson. */
  | "multiple"
  /** Does not conserve some element, or the charge. */
  | "unbalanced"
  /** Not yet an answer: a blank, a zero, a fraction, a negative. */
  | "incomplete";

export type BalanceMiss = {
  /** An element symbol, or `charge`. */
  element: string;
  /** Signed surplus on the left as the learner wrote it. */
  amount: number;
};

export type BalanceMark = {
  verdict: BalanceVerdict;
  /** For `multiple`: what to divide every coefficient by. */
  factor?: number;
  /** For `unbalanced`: what does not cancel, worst first. */
  misses: BalanceMiss[];
  /** True where the skeleton admits more than one independent reaction. */
  family: boolean;
};

/** Rounding slack on a dot product of integers and small decimals. */
const TOLERANCE = 1e-9;

function gcd(a: number, b: number): number {
  let [x, y] = [Math.abs(a), Math.abs(b)];
  while (y !== 0) [x, y] = [y, x % y];
  return x;
}

/**
 * Mark one answer against the engine's own composition matrix.
 *
 * Nothing here consults the solver's coefficients: an answer it never
 * produced is marked by the same arithmetic as one it did, which is what
 * makes the "correct multiple" and underdetermined cases honest rather than
 * special-cased.
 */
export function markBalance(report: BalanceReport, answer: number[]): BalanceMark {
  const family = report.basis.length > 0;
  if (
    answer.length !== report.species.length ||
    answer.some((value) => !Number.isInteger(value) || value <= 0)
  ) {
    return { verdict: "incomplete", misses: [], family };
  }
  const misses: BalanceMiss[] = [];
  report.matrix.forEach((row, index) => {
    const surplus = row.reduce((sum, count, column) => sum + count * (answer[column] ?? 0), 0);
    if (Math.abs(surplus) > TOLERANCE) {
      misses.push({ element: report.elements[index] ?? `row ${index}`, amount: surplus });
    }
  });
  if (misses.length > 0) {
    misses.sort((a, b) => Math.abs(b.amount) - Math.abs(a.amount));
    return { verdict: "unbalanced", misses, family };
  }
  const factor = answer.reduce(gcd, 0);
  if (factor > 1) return { verdict: "multiple", factor, misses: [], family };
  return { verdict: "correct", misses: [], family };
}

/**
 * The equation as a learner's answer writes it, for echoing a mark back.
 *
 * A coefficient of 1 is not written, the way it is not written on paper.
 */
export function writeEquation(
  report: BalanceReport,
  coefficients: number[],
): string {
  const term = (index: number) => {
    const coefficient = coefficients[index];
    const species = report.species[index] ?? "";
    return coefficient === undefined || coefficient === 1
      ? species
      : `${coefficient} ${species}`;
  };
  const left = report.species.slice(0, report.reactants).map((_, i) => term(i));
  const right = report.species.slice(report.reactants).map((_, i) => term(report.reactants + i));
  return `${left.join(" + ")} ${report.reversible ? "⇌" : "→"} ${right.join(" + ")}`;
}

/** The question: the same equation with every coefficient taken off. */
export function blankEquation(report: BalanceReport): string {
  return writeEquation(report, report.species.map(() => 1));
}

/** One equation a round can be built from, and where it came from. */
export type BalancingSource = {
  /** Stable id — a codex slug, or `session:<n>` for one the bench produced. */
  id: string;
  equation: string;
  /** `codex` for a catalogue entry, `bench` for a reaction this session ran. */
  origin: "codex" | "bench";
};

/**
 * Which equations are worth *offering* — the cheap filter, before the engine
 * is asked.
 *
 * Deliberately permissive: the codex's `equation` field is documented as a
 * balanced equation but a handful of entries use it for prose, and the only
 * authority on whether a string is an equation is the parser. So this drops
 * what obviously cannot be one (no arrow, one species) and lets the engine
 * refuse the rest, rather than growing a second opinion about formulas here.
 */
export function balancingSources(
  entries: { id: string; equation?: string | null }[],
  benchEquations: string[] = [],
): BalancingSource[] {
  const sources: BalancingSource[] = [];
  const seen = new Set<string>();
  const offer = (id: string, equation: string, origin: "codex" | "bench") => {
    const text = equation.trim();
    if (!/⇌|⟶|→|->|<=>|=>|⇄|↔/.test(text)) return;
    // A skeleton needs at least two species; a lone arrow with nothing on
    // one side of it is prose that happens to contain a dash.
    if (!/\S/.test(text.split(/⇌|⟶|→|->|<=>|=>|⇄|↔/)[1] ?? "")) return;
    if (seen.has(text)) return;
    seen.add(text);
    sources.push({ id, equation: text, origin });
  };
  // The bench's own reactions first: an equation the learner just made
  // happen is a better question than one from a catalogue they have not read.
  benchEquations.forEach((equation, index) => offer(`session:${index}`, equation, "bench"));
  for (const entry of entries) {
    if (typeof entry.equation === "string") offer(entry.id, entry.equation, "codex");
  }
  return sources;
}

/**
 * The next question, given what this session has already asked.
 *
 * Deterministic — it walks the list rather than sampling it — so a round is
 * reproducible and the tests are not flaky. Exhausting the list wraps rather
 * than ending: the point of a generated exercise is that it does not run out.
 */
export function nextSource(
  sources: BalancingSource[],
  asked: string[],
): BalancingSource | null {
  if (sources.length === 0) return null;
  const unseen = sources.find((source) => !asked.includes(source.id));
  if (unseen) return unseen;
  return sources[asked.length % sources.length] ?? null;
}

/**
 * The message key for a mark, with its variables.
 *
 * A key and a bag of named holes, never a composed sentence: the case picks
 * the key and each key holds a complete sentence, which is I18N.md's rule
 * for exactly this shape. That is why "too many" and "too few" are two keys
 * rather than one key and a sign — a language that inflects the verb cannot
 * be served by flipping a minus.
 */
export function markMessage(mark: BalanceMark): { key: string; vars: Record<string, string> } {
  switch (mark.verdict) {
    case "correct":
      return mark.family
        ? { key: "balanced — and this skeleton has more than one answer, so yours is one of a family", vars: {} }
        : { key: "balanced, in the smallest whole numbers", vars: {} };
    case "multiple":
      return {
        key: "balanced — but every coefficient divides by {factor}, so this is not the smallest whole-number ratio",
        vars: { factor: String(mark.factor ?? 1) },
      };
    case "unbalanced": {
      const worst = mark.misses[0];
      if (worst === undefined) return { key: "this does not balance", vars: {} };
      const amount = String(Number(Math.abs(worst.amount).toFixed(4)));
      return worst.amount > 0
        ? { key: "{amount} too much {element} on the left as written", vars: { amount, element: worst.element } }
        : { key: "{amount} too much {element} on the right as written", vars: { amount, element: worst.element } };
    }
    case "incomplete":
      return { key: "every species needs a whole number greater than zero", vars: {} };
  }
}
