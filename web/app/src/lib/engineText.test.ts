import { afterEach, describe, expect, it } from "vitest";
import { engineText } from "./engineText";
import { i18n } from "./i18n.svelte";

afterEach(() => i18n.setLocale("en"));

/**
 * What is left of this layer, and where the rest went.
 *
 * `engineText` used to rewrite the engine's ENGLISH output into German by
 * matching it with regexes. It predates the engine's own message
 * catalogue, and everything it covered has now moved there — the vessel
 * summary, additions, stirring, grinding, dissolution, and finally the
 * hazard notes.
 *
 * The hazard notes were last for a reason. The shell reads `hazard` and
 * `real_world` straight off the serialised event rather than through the
 * renderer, so no catalogue on the Rust side could reach them; they were
 * translated in the shell or not at all. `localize_events` in the wasm
 * wrapper now does it on the way out, at the first point that knows the
 * language — so the shell has nothing left to do.
 *
 * The contract these tests protected — a German reader sees German — is
 * tested in better places now:
 *
 *   crates/kerotakis-core/tests/hazard_locale.rs
 *       walks senses::ODORS and fails if a hazardous substance has no
 *       German, which is the only defence against a silent fallback
 *   crates/kerotakis-core/tests/render_locale.rs
 *       asserts the engine renders its sentences in German, which also
 *       covers the CLI, where this file never had any effect
 *   tools/test-i18n-render.mjs
 *       opens the real page in German and reads the screen
 *
 * The engine version can do something this one never could: the DECIMAL.
 * By the time a line reached here the number was already a formatted
 * string, so `11.0686` could never become `11,0686`. That remains the
 * clearest signal for which layer produced a line.
 */
describe("engine text localization", () => {
  it("passes English through untouched", () => {
    const line = "v1 (beaker) — 25.00 °C, 0.0 g, 0.0 mL liquid, open to the atmosphere";
    expect(engineText(line)).toBe(line);
  });

  it("passes German through untouched — the engine already rendered it", () => {
    i18n.setLocale("de");
    const line = "v1 (Becherglas) — 25,00 °C, 0,0 g, 0,0 mL Flüssigkeit, offen zur Atmosphäre";
    expect(engineText(line)).toBe(line);
  });

  it("no longer translates hazard notes — the engine does", () => {
    // This is the behaviour that moved. The shell used to compound
    // `${t(species)}dampf` here, which could only ever produce German and
    // got the formulas wrong: NH3 needs a hyphen, and Cl2 is a gas, not a
    // vapour. Both are right in the engine catalogue now.
    i18n.setLocale("de");
    const english = "NH3 vapour is hazardous to inhale";
    expect(engineText(english)).toBe(english);
  });

  /**
   * The refusals, which arrive finished for the same reason the hazards do.
   *
   * These were reported from the live German deploy — "many
   * warnings/errors/info strings are NOT showing in German, like 'no vessel
   * v2'". The fix is in the engine (`Refusal` + `error.*` in
   * `crates/kerotakis-core/i18n/de.toml`, gated by
   * `crates/kerotakis-core/tests/refusal_locale.rs`); what is checked HERE
   * is only that the shell does not undo it.
   */
  it("passes a German refusal through with its vessel name intact", () => {
    i18n.setLocale("de");
    const refusal =
      "kein Gefäß v2 — lege es zuerst mit `new` an; das erzeugt das nächste freie Gefäß";
    expect(engineText(refusal)).toBe(refusal);
    // `v2` is an identifier, not a measurement: nothing may reformat it.
    expect(engineText(refusal)).toContain("v2");
  });

  it("does not re-translate an English refusal it happens to recognise", () => {
    // The temptation this file exists to resist. A German string written
    // in the shell is not a missing translation — it is one that only
    // German can ever have, and there would be nowhere to put French.
    i18n.setLocale("de");
    const english = "no vessel v2 — make it first with `new`, which creates the next free vessel";
    expect(engineText(english)).toBe(english);
  });

  it("invents nothing for prose it does not recognise", () => {
    i18n.setLocale("de");
    const unknown = "something no catalogue anywhere has a translation for";
    expect(engineText(unknown)).toBe(unknown);
  });

});
