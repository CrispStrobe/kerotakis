/**
 * The balanced equation the bench is currently showing.
 *
 * It used to be scraped out of the RENDERED PROSE: any line carrying a `→`
 * was an equation, and everything between the last colon before the arrow
 * and the first full stop after it was the chemistry. That worked for the
 * line it was written against — `v1: HCO₃⁻ + CH₃COOH → CH₃COO⁻ + H₂O +
 * CO₂↑` — and it was wrong about the rest of the bench, because the arrow
 * is not the reaction's private punctuation. **41 of the engine's rendered
 * lines carry one**, and exactly one of them is an equation.
 *
 * Two of those were caught on a live German bench, both pinned into the
 * REAKTION rail and both saved into the balancing drill's question pool:
 *
 *   - `v1: Route → Kerotakis analytic equilibrium evaluator · phreeqc.dat…`
 *     — the aqueous routing announcement, whose arrow separates a label
 *     from a solver name. The rail read *"Route → Kerotakis analytic
 *     equilibrium evaluator · phreeqc"*, which is a sentence about
 *     provenance presented as a claim about chemistry.
 *   - `v1: T 298,150 K → 299,356 K (ΔT = +1,206 K)` — a temperature
 *     change. In German there is not even a full stop to cut it at,
 *     because the decimal separator is a comma, so the whole line pinned.
 *
 * A heuristic cannot separate those from chemistry, because they are all
 * the same shape: a thing, an arrow, another thing. So the scrape is gone
 * and the equation is read from the EVENT that carries it —
 * `Event::ReactionOccurred { vessel, equation }`, which is what the prose
 * was rendered from in the first place. The same move GUI-092 made for the
 * ionic form, and for the same reason: the structured claim is the claim,
 * and its prose is one rendering of it.
 *
 * One property is worth naming because it is easy to lose: this reads the
 * equation at EVERY register, including `lv1`, where the engine renders
 * "the mixture changes — something new is forming!" and no equation at
 * all. Whether an equation belongs on the rail at lv1 is the reader's
 * setting and therefore the caller's decision, not this function's.
 */

/** The wire shape this needs of an event. Everything else is ignored. */
interface EventLike {
  event?: unknown;
  equation?: unknown;
}

/**
 * Every balanced equation the step's events carry, oldest first.
 *
 * A step may hold more than one — a script line that sets off two curated
 * reactions writes two — and the caller pins the last, which is the
 * reaction the bench is showing now.
 */
export function equationsFromEvents(events: readonly unknown[] | undefined | null): string[] {
  if (!Array.isArray(events)) return [];
  const found: string[] = [];
  for (const entry of events) {
    if (!entry || typeof entry !== "object") continue;
    const event = entry as EventLike;
    if (event.event !== "reaction_occurred") continue;
    if (typeof event.equation !== "string") continue;
    const equation = event.equation.trim();
    // An equation with nothing on one side of its arrow is not one. The
    // engine does not write those, and a pinned fragment is worse than an
    // empty rail — it was the original bug, wearing the other hat.
    if (!equation || !/→|⇌/.test(equation)) continue;
    const [left, right] = equation.split(/→|⇌/);
    if (!left?.trim() || !right?.trim()) continue;
    found.push(equation);
  }
  return found;
}
