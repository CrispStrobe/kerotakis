/**
 * One shape for "more about this, behind an (i)".
 *
 * The reagent shelf learned this first: a row that carries a chemical's
 * physical state, appearance, family, capability, hazards and stock is not
 * a row, it is a paragraph, and ninety of them is a wall. The fix was a
 * compact row plus a small (i) that opens a `<dl>` — and the moment the
 * kids' kit cards needed the same fix, the choice was to reuse that panel
 * or to write a second one that drifts from it. This is the first half of
 * reusing it: the row shape both callers build.
 *
 * Every string here is already localized. The panel renders text, it does
 * not decide what text means, so `t()` stays at the call site where the
 * key is a literal the translation scan can see.
 */

/** Why a value is coloured. Not a colour: the panel owns those. */
export type InfoTone = "danger" | "warn" | "info";

export interface InfoRow {
  /** The label, already localized. */
  term: string;
  /** The value, already localized. */
  detail: string;
  /** Colour the value carries a meaning worth marking. */
  tone?: InfoTone;
  /**
   * Stack the value under its label instead of setting it opposite.
   *
   * A one-word value reads best right-aligned against its label. A
   * sentence right-aligned against a label is a ragged column that starts
   * in a different place on every line, so anything sentence-shaped asks
   * for the full width.
   */
  block?: boolean;
}
