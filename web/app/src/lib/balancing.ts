/**
 * GUI-095 — balancing as a *generated* exercise: choosing the questions.
 *
 * The engine balances by the null space of the element-and-charge count
 * matrix, so practice does not have to be authored: take any equation the
 * codex or a session produced, strip its coefficients, and ask. Unlimited,
 * never wrong, and with no question bank to maintain. What is left in this
 * file is the *pool* — which strings are worth offering as questions, and
 * which one to ask next.
 *
 * **The marking used to be here, and that was the leak.** It was careful
 * code and it stays worth understanding: comparing a learner's coefficients
 * to the solver's would mark `4 Mg + 2 O₂ → 4 MgO` as wrong, when it is not
 * wrong — it balances, it is merely not the smallest whole-number ratio, and
 * saying *that* is the lesson. So the mark was arithmetic over the learner's
 * own vector against `matrix`, and it never once consulted the answer.
 *
 * But the matrix is the answer one null space away, and it crossed the wire
 * to get here. A drill whose questions arrive with the means to solve them
 * is not a drill; anyone who opens the network pane has it. The client
 * renders the exercise and never carries its answer, so marking moved to
 * `kerotakis_core::stoich::mark` — one implementation, reached by
 * `balanceMark`, shared with the CLI drill that had been transliterating it.
 * Nothing about the four verdicts changed, and the reasoning above is now
 * documented where the code lives.
 */
import type { BalanceMark } from "./host/EngineHost";

/** Where a question came from, which is also why it is being offered. */
export type BalancingOrigin = "bench" | "mission" | "codex";

/** One equation a round can be built from, and where it came from. */
export type BalancingSource = {
  /** Stable id — a codex slug, or `session:<n>` for one the bench produced. */
  id: string;
  equation: string;
  /** `codex` for a catalogue entry, `bench` for a reaction this session ran,
   * `mission` for a catalogue entry the active mission has the materials for. */
  origin: BalancingOrigin;
  /** Already answered correctly, so it is offered last rather than dropped. */
  solved: boolean;
};

/** The one-line reason beside a candidate. One complete sentence per origin. */
export function originReason(
  origin: BalancingOrigin,
): "from your bench" | "from your mission" | "next in the catalogue" {
  switch (origin) {
    case "bench":
      return "from your bench";
    case "mission":
      return "from your mission";
    case "codex":
      return "next in the catalogue";
  }
}

const ARROW = /⇌|⟶|→|->|<=>|=>|⇄|↔/;
const ARROWS = /⇌|⟶|→|->|<=>|=>|⇄|↔/g;

/**
 * The equation with its coefficients taken off — a label that names the
 * question without answering it.
 *
 * The engine renders the authoritative skeleton and always will; this is
 * display only, for the picker, and its one job is to make sure a
 * candidate list cannot hand over the answer to the very question it is
 * about to ask. Splitting on the arrow and then on `+` is enough because a
 * coefficient can only ever lead a term.
 */
export function skeletonOf(equation: string): string {
  return equation
    .split(ARROWS)
    .map((side) =>
      side
        .split("+")
        .map((term) => term.trim().replace(/^\d+(?:\.\d+)?\s+/, ""))
        .filter((term) => term.length > 0)
        .join(" + "),
    )
    .join(" → ");
}

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
  return balancingCandidates({ entries, benchEquations });
}

export type BalancingCandidateSources = {
  entries: readonly { id: string; equation?: string | null }[];
  /** Newest first, the order `Session.benchEquations` keeps them in. */
  benchEquations?: readonly string[];
  /** Shelf keys the active mission's own commands use, and the reagents each
   * catalogue entry's setup script uses — a mission tier without either is
   * simply absent rather than guessed at. */
  missionKit?: readonly string[];
  entryKit?: (id: string) => readonly string[];
  /** Codex entry ids the learner ran to a green check. */
  met?: ReadonlySet<string>;
  /** Ids already balanced correctly, so they sink within their own tier. */
  solved?: ReadonlySet<string>;
};

/**
 * The offered questions, in the order a learner would want them.
 *
 * Three tiers, and the order IS the pedagogy — it is what makes the drill
 * answer "why this equation?" instead of looking like a shuffle:
 *
 *   1. the bench, newest first: a reaction the learner just made happen;
 *   2. the active mission, meaning a catalogue entry whose every reagent the
 *      mission already put on the shelf, so the question is about the
 *      chemistry in front of them;
 *   3. the rest of the catalogue, with entries they have run to a green
 *      check first — a balanced equation for a reaction they have watched
 *      is a different exercise from one they have only read about.
 *
 * Inside every tier an already-solved equation sinks to the end rather than
 * disappearing: practice may be repeated, it just should not be offered
 * ahead of something new.
 */
export function balancingCandidates(sources: BalancingCandidateSources): BalancingSource[] {
  const { entries, benchEquations = [], missionKit, entryKit, met, solved } = sources;
  const isSolved = (id: string) => solved?.has(id) ?? false;
  const tiers: Record<BalancingOrigin, BalancingSource[]> = { bench: [], mission: [], codex: [] };
  const seen = new Set<string>();
  const offer = (id: string, equation: string, origin: BalancingOrigin) => {
    const text = equation.trim();
    if (!ARROW.test(text)) return;
    // A skeleton needs at least two species; a lone arrow with nothing on
    // one side of it is prose that happens to contain a dash.
    if (!/\S/.test(text.split(ARROW)[1] ?? "")) return;
    if (seen.has(text)) return;
    seen.add(text);
    tiers[origin].push({ id, equation: text, origin, solved: isSolved(id) });
  };
  // The bench's own reactions first: an equation the learner just made
  // happen is a better question than one from a catalogue they have not read.
  benchEquations.forEach((equation, index) => offer(`session:${index}`, equation, "bench"));
  const shelf = missionKit === undefined ? null : new Set(missionKit);
  const supplied = (id: string): boolean => {
    if (shelf === null || shelf.size === 0 || entryKit === undefined) return false;
    const needed = entryKit(id);
    // An entry that uses no reagents is a subset of every kit; it belongs to
    // the catalogue tier, not to whichever mission happens to be running.
    return needed.length > 0 && needed.every((item) => shelf.has(item));
  };
  for (const entry of entries) {
    if (typeof entry.equation !== "string") continue;
    offer(entry.id, entry.equation, supplied(entry.id) ? "mission" : "codex");
  }
  const byProgress = (a: BalancingSource, b: BalancingSource) =>
    Number(a.solved) - Number(b.solved) ||
    Number(met?.has(b.id) ?? false) - Number(met?.has(a.id) ?? false);
  // `sort` is stable, so catalogue order survives inside each progress band.
  tiers.mission.sort(byProgress);
  tiers.codex.sort(byProgress);
  tiers.bench.sort((a, b) => Number(a.solved) - Number(b.solved));
  return [...tiers.bench, ...tiers.mission, ...tiers.codex];
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
        vars: { factor: String(mark.factor) },
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
