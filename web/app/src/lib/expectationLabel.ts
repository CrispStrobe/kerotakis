/**
 * What a catalogue entry PROMISED, said in the reader's language.
 *
 * The verdict list under a finished run is the last thing a learner reads,
 * and it was the only part of the run still written in English. Every row
 * came from `Catalog.svelte` as `t(want.replace(/_/g, " "))`, where `want`
 * is a compound wire token — `added:phenolphthalein`, `gas_evolved:CO2`,
 * `not_yet_modelled`. Two separate reasons that could never render German:
 *
 *   - the whole token was handed to the dictionary as ONE key, so
 *     `added:phenolphthalein` looked up a string no bundle has ever
 *     carried, and `t()` falls back to its key. The colon was not even
 *     spaced.
 *   - the dictionary held no entry for the verbs regardless. Twenty-six of
 *     the twenty-nine verbs the codex uses had no German at all; the three
 *     that resolved (`boiled`, `froze`, `measured`) did so by colliding
 *     with words translated for something else.
 *
 * So a token is SPLIT here, and each half is translated for what it is.
 * The verb is prose and gets a per-verb template, because word order is
 * not shared between languages: "phenolphthalein added" is
 * "Phenolphthalein hinzugefügt", and a German reader meets the operand
 * first. The operand is a name — a formula, a species slug, a reaction
 * slug — and goes through the same term table the rest of the bench uses,
 * where a formula has no entry and therefore renders as itself. `NaCl` is
 * `NaCl` in every language and must never be sent to a translator.
 *
 * The verb tables are exhaustive over the shipped catalogue rather than
 * open-ended, and `i18n.test.ts` walks `codex/*.toml` to keep them that
 * way: a codex author adding a new expectation verb fails that gate rather
 * than quietly reintroducing an English row. An unknown verb still renders
 * — as the words it is made of, which is what the bug rendered — because a
 * missing translation must cost one row, never the list.
 */
import { t as translate } from "./i18n.svelte";

/** The dictionary, as this module needs it. Injectable so the tests can
 *  answer for a language without standing up the bundle loader. */
export type Translate = (message: string, vars?: Record<string, string | number>) => string;

/**
 * Verbs the catalogue asserts on their own, with no operand.
 *
 * Keys are wire tokens; values are the English source text, which is also
 * the dictionary key. Deliberately not derived by replacing underscores:
 * `not_yet_modelled` reads as a sentence fragment and `cell_voltage` names
 * a reading, and a translator needs the phrase rather than the tag.
 */
export const EXPECTATION_VERBS: Readonly<Record<string, string>> = {
  cell_voltage: "a cell voltage was read",
  distilled: "something distilled over",
  evaporated: "something evaporated",
  filtered: "something was filtered",
  gas_tested: "a gas was tested",
  hazard_warning: "a hazard was warned about",
  headspace_partitioned: "the headspace partitioned",
  ignited: "something ignited",
  measured: "something was measured",
  mixed: "something was mixed",
  not_yet_modelled: "not yet modelled",
  observed: "something was observed",
  org_reacted: "an organic reaction ran",
  stirred: "something was stirred",
  temperature_changed: "the temperature changed",
};

/**
 * Verbs the catalogue asserts ABOUT something, with `{what}` standing for
 * the operand.
 *
 * The placeholder is where the operand goes in THIS language and nowhere
 * else: English puts it first for most of these and German puts it first
 * for all of them, and a future language may put it last. That is the
 * whole reason these are templates and not a verb beside a colon.
 */
export const EXPECTATION_VERBS_WITH: Readonly<Record<string, string>> = {
  added: "{what} added",
  adsorbed: "{what} adsorbed",
  boiled: "{what} boiled",
  consumed: "{what} consumed",
  dissolved: "{what} dissolved",
  electrolysed: "{what} electrolysed",
  flame_test: "{what} flame test",
  froze: "{what} froze",
  gas_evolved: "{what} evolved as gas",
  inert: "{what} stayed inert",
  plated: "{what} plated out",
  polymer_heated: "{what} heated as a polymer",
  precipitated: "{what} precipitated",
  reacted: "{what} reacted",
};

/** A wire token split into the two things it actually says. */
export interface ExpectationToken {
  verb: string;
  /** The thing the verb is about, or null when the verb stands alone. */
  what: string | null;
}

/**
 * Split at the FIRST colon only.
 *
 * `polymer_heated:cured thermoset resin` carries spaces in its operand and
 * `flame_test:NaCl` carries an underscore in its verb, so neither half can
 * be recognised by its shape — only by which side of the first colon it
 * falls on.
 */
export function parseExpectation(want: string): ExpectationToken {
  const at = want.indexOf(":");
  if (at < 0) return { verb: want, what: null };
  return { verb: want.slice(0, at), what: want.slice(at + 1) };
}

/**
 * The operand's name in the reader's language.
 *
 * Underscores and hyphens both become spaces before the lookup, because
 * the catalogue writes the same name three ways — `methyl_orange`,
 * `peroxide-decomposition`, `thermoplastic sheet` — while the dictionary
 * is keyed by the words. A name with no entry renders as itself, which is
 * exactly right for a formula: `CO2` is not English, so there is nothing
 * to translate and nothing missing.
 */
export function expectationOperand(what: string, t: Translate = translate): string {
  return t(what.replace(/[_-]/g, " "));
}

/**
 * One verdict row, translated.
 *
 * `t` is injectable so the unit tests can answer for a language without
 * standing up the bundle loader, and so the codex gate can ask what a
 * token WOULD render as.
 */
export function expectationLabel(want: string, t: Translate = translate): string {
  const { verb, what } = parseExpectation(want);
  if (what === null) {
    const phrase = EXPECTATION_VERBS[verb];
    // An unrecognised verb renders as its own words rather than as a wire
    // tag: `some_new_event` reads as "some new event", which is what the
    // English build has always shown and is legible in any language's
    // sentence around it.
    return phrase ? t(phrase) : t(verb.replace(/_/g, " "));
  }
  const operand = expectationOperand(what, t);
  const template = EXPECTATION_VERBS_WITH[verb];
  // No template means a verb the catalogue has not used with an operand
  // before. Naming both halves is still better than printing the token:
  // the reader learns what was expected and about what, and the gate in
  // `i18n.test.ts` is what turns this into a failure rather than a habit.
  if (!template) return `${t(verb.replace(/_/g, " "))}: ${operand}`;
  return t(template, { what: operand });
}
