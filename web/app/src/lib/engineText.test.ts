import { afterEach, describe, expect, it } from "vitest";
import { engineText } from "./engineText";
import { i18n } from "./i18n.svelte";

afterEach(() => i18n.setLocale("en"));

/**
 * What is left of this layer, and where the rest went.
 *
 * `engineText` rewrites the engine's ENGLISH output into German by
 * matching it with regexes. It predates the engine's own message
 * catalogue, and most of what it used to cover has moved there — the
 * vessel summary, additions, stirring, grinding, dissolution. Those tests
 * did not fail because the behaviour broke; they failed because they
 * asserted on the MECHANISM, and the mechanism moved.
 *
 * The contract they protected — a German reader sees German — is still
 * tested, in two better places:
 *
 *   crates/kerotakis-core/tests/render_locale.rs
 *       asserts the engine renders those sentences in German, which also
 *       covers the CLI, where this file has never had any effect
 *   tools/test-i18n-render.mjs
 *       opens the real page in German and reads the journal and the
 *       vessel line off the screen
 *
 * The engine version can do something this one cannot: the DECIMAL. By
 * the time a line reaches here the number is already a formatted string,
 * so `11.0686` can never become `11,0686`. That is the clearest signal
 * for which layer produced a line.
 *
 * What remains here is the lines the catalogue has not reached yet. They
 * compose safely: a pattern that matches English never fires on a
 * sentence the engine already rendered in German.
 */
describe("engine text localization", () => {
  it("keeps the canonical engine prose in English", () => {
    i18n.setLocale("en");
    expect(engineText("The mini centrifuge spins v1; the particles travel 42% of the tube path.")).toBe(
      "The mini centrifuge spins v1; the particles travel 42% of the tube path.",
    );
  });

  it("translates the lines the engine catalogue has not reached", () => {
    i18n.setLocale("de");
    expect(engineText("While you wait, particles in v1 sink toward the bottom.")).toBe(
      "Während du wartest, sinken Teilchen in v1 zum Boden.",
    );
    expect(engineText("v1: 63% of the suspended particles settle in 120 s")).toBe(
      "v1: 63 % der schwebenden Teilchen setzen sich in 120 s ab",
    );
  });

  it("leaves a sentence the engine already rendered in German untouched", () => {
    // The patterns match English, so German passes through. This is what
    // makes the two layers safe to run together while the catalogue takes
    // lines over one at a time.
    i18n.setLocale("de");
    const fromEngine = "v1: +11,0686 mol Wasser";
    expect(engineText(fromEngine)).toBe(fromEngine);
  });

  it("passes through prose it has no pattern for", () => {
    i18n.setLocale("de");
    const unknown = "v1: some sentence nobody has taught this layer";
    expect(engineText(unknown)).toBe(unknown);
  });
});
