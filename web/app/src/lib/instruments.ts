/**
 * The twelve readings, and how each of them is drawn.
 *
 * GUI-109 — the owner, from the German deploy: *"most symbols are not even
 * intuitive. we should have much better icons/symbols/drawings of the
 * devices."* They were right. `≋` for a safe waft, `λ` for a UV-Vis, `Rf`
 * for a chromatograph and `bar` for a manometer are not drawings of
 * anything; they are labels standing where a picture was promised, and two
 * of them are text a reader in a third language will never be shown in
 * their own.
 *
 * So most of these now carry an `icon` — a path in `ToolIcon.svelte`,
 * inline SVG in `currentColor`, the same stroke-only hand the apparatus
 * portraits were drawn in for GUI-033. `glyph` stays as the fallback, and
 * it is still what the vessel readouts use, where 24 px of glyph sits
 * beside a number and a line drawing would be the wrong register.
 *
 * TWO instruments deliberately keep a glyph and get no drawing:
 *
 *   * **pH.** It is the quantity's own notation, it is not translated in
 *     any language the app ships, and no 26 px drawing of a probe says
 *     "this reads pH" to someone who does not already know what a pH probe
 *     looks like.
 *   * **Bq.** The same argument. A Geiger counter is a box, a cable and a
 *     tube, and at this size that is three grey rectangles; the trefoil
 *     would be honest about RADIOACTIVITY but dishonest about the
 *     instrument, since it is a hazard mark and not a counter.
 *
 * A letter is honest about being a label. A shape that means nothing is
 * not, so those two stay letters until someone draws better than this.
 */
export type InstrumentSpec = {
  token: string;
  label: string;
  /** The typographic fallback, and what the vessel readouts draw. */
  glyph: string;
  /** A `ToolIcon.svelte` path name, where a drawing beats the glyph. */
  icon?: string;
  purpose: string;
};

export const INSTRUMENTS: InstrumentSpec[] = [
  { token: "smell", label: "safe waft", glyph: "≋", icon: "waft", purpose: "check headspace odour safely" },
  { token: "thermometer", label: "thermometer", glyph: "🌡", icon: "thermometer", purpose: "measure sample temperature" },
  // pH stays a glyph: see the note above.
  { token: "ph", label: "pH meter", glyph: "pH", purpose: "measure aqueous acidity" },
  { token: "balance", label: "balance", glyph: "⚖", icon: "balance", purpose: "measure total material mass" },
  { token: "volume", label: "gas volume meter", glyph: "mL", icon: "gas-syringe", purpose: "measure sealed gas volume" },
  { token: "conductivity", label: "conductivity meter", glyph: "⚡", icon: "conductivity", purpose: "estimate ionic conductivity" },
  { token: "pressure", label: "pressure gauge", glyph: "bar", icon: "manometer", purpose: "measure headspace pressure" },
  { token: "calorimeter", label: "calorimeter", glyph: "kJ", icon: "calorimeter", purpose: "measure enthalpy relative to 25 °C" },
  { token: "uvvis", label: "UV-Vis", glyph: "λ", icon: "spectrophotometer", purpose: "measure an absorbance spectrum" },
  { token: "eyes", label: "look closely", glyph: "🔍", icon: "magnifier", purpose: "inspect visible appearance" },
  { token: "chromatograph", label: "chromatograph", glyph: "Rf", icon: "chromatography-jar", purpose: "separate and compare components" },
  // Bq stays a glyph: see the note above.
  { token: "geiger", label: "Geiger counter", glyph: "Bq", purpose: "measure radioactive activity" },
];

export const instrumentVerb = (token: string) => `measure:${token}`;

export function instrumentCommand(vessel: number, token: string): string {
  const target = `v${vessel + 1}`;
  if (token === "chromatograph") return `chromatograph ${target}`;
  if (token === "smell") return `smell ${target}`;
  return `measure ${target} ${token}`;
}
