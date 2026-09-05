/**
 * The balanced equation inside one rendered engine line.
 *
 * The engine renders `Event::ReactionOccurred` at lv2+ as `{vessel}: {equation}`
 * (`kerotakis-core::render`), so the line the bench receives reads
 * `v1: HCO₃⁻ + CH₃COOH → CH₃COO⁻ + H₂O + CO₂↑`. The strip beside the bench
 * pins the equation out of it.
 *
 * The first attempt scanned the line with one regular expression whose
 * leading `\S` happily matched the colon itself — so what got pinned, saved
 * into the balancing drill's pool and shown at lv2 was
 * `": HCO₃⁻ + CH₃COOH → …"`,
 * reading as a heading whose title had gone missing. The label was never
 * missing; the punctuation belonged to the sentence around the equation and
 * was captured with it.
 *
 * So the prefix is *recognised* rather than escaped: everything up to the
 * last separator before the arrow is the engine's own framing (a vessel id,
 * a "net ionic" tag), and the equation is what follows it. That also drops
 * the vessel from the drill's questions, which is the point of a drill.
 */

/** Arrow characters the engine writes between the two sides. */
const ARROW = /→|⇌/;

/**
 * Pull the equation out of one rendered line, or null when it carries none.
 *
 * Returns the equation trimmed of the framing punctuation on either end, so
 * a caller can render it as the equation it is rather than as a fragment of
 * the sentence it arrived in.
 */
export function equationFromRenderedLine(line: string): string | null {
  if (!ARROW.test(line)) return null;
  const arrow = line.search(ARROW);
  // The sentence the equation sits in may end after it ("… → CO₂↑. The gas
  // collects."); a full stop closes the equation the same way it closes the
  // clause. Colons and semicolons only ever precede it.
  const tail = line.slice(arrow);
  const stop = tail.search(/[.;]/);
  const end = stop === -1 ? line.length : arrow + stop;
  // Everything before the arrow, back to the last colon or semicolon: the
  // vessel id and any tag the renderer put in front of the chemistry.
  const head = line.slice(0, arrow);
  const separator = Math.max(head.lastIndexOf(":"), head.lastIndexOf(";"));
  const equation = line.slice(separator + 1, end).trim();
  // A lone arrow with nothing on one side of it is prose, not an equation.
  const [left, right] = equation.split(ARROW);
  if (!left?.trim() || !right?.trim()) return null;
  return equation;
}
