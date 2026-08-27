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

  it("translates the one line the engine catalogue has not reached", () => {
    // The settling and centrifuge lines used to be here. render.rs now
    // renders them from its own catalogue, so those patterns could never
    // fire again and were deleted; their contract moved to
    // crates/kerotakis-core/tests/render_locale.rs and to the browser
    // test. What is left comes from bench.rs, which has no catalogue yet.
    i18n.setLocale("de");
    expect(engineText("ethanol vapour is hazardous to inhale")).toBe(
      "Ethanoldampf ist beim Einatmen gefährlich",
    );
  });

  it("compounds onto whatever the dictionary gives, right or not", () => {
    // The sharp edge in this pattern, pinned rather than hidden: it builds
    // a German compound by appending "dampf" to a translated species name.
    // That works when the dictionary has the name AND its German is the
    // right stem — ethanol -> Ethanol -> Ethanoldampf.
    //
    // "ammonia" is not a key (the dictionary has "ammonia solution"), so
    // it falls through untranslated and the compound comes out
    // "ammoniadampf": lowercase, and the wrong stem for Ammoniakdampf.
    // Compounding is not something a regex can do correctly across
    // languages, which is the argument for this line moving into the
    // engine catalogue with bench.rs rather than being patched here.
    i18n.setLocale("de");
    expect(engineText("ammonia vapour is hazardous to inhale")).toBe(
      "ammoniadampf ist beim Einatmen gefährlich",
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
