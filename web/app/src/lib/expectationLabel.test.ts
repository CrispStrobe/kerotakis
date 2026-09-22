/**
 * The verdict list under a finished catalogue run, in the reader's language.
 *
 * Every assertion here is about a token the shipped codex actually writes.
 * The bug was not that one string was missed; it was that the whole row was
 * handed to the dictionary as one compound key, so no translation of any
 * language could ever have matched it. So the split is what is pinned, and
 * `i18n.test.ts` pins that the halves have German.
 */
import { describe, expect, it } from "vitest";
import {
  EXPECTATION_VERBS,
  EXPECTATION_VERBS_WITH,
  expectationLabel,
  expectationOperand,
  parseExpectation,
  type Translate,
} from "./expectationLabel";

/** A dictionary with just enough German in it to answer these questions. */
const GERMAN: Record<string, string> = {
  "{what} added": "{what} hinzugefügt",
  "{what} evolved as gas": "{what} als Gas freigesetzt",
  "not yet modelled": "noch nicht modelliert",
  "something was observed": "etwas wurde beobachtet",
  phenolphthalein: "Phenolphthalein",
  "methyl orange": "Methylorange",
  "peroxide decomposition": "Peroxidzerfall",
};

const de: Translate = (message, vars = {}) =>
  (GERMAN[message] ?? message).replace(/\{(\w+)\}/g, (_, key: string) => String(vars[key] ?? `{${key}}`));

/** English is the source text, so the identity function IS the English build. */
const en: Translate = (message, vars = {}) =>
  message.replace(/\{(\w+)\}/g, (_, key: string) => String(vars[key] ?? `{${key}}`));

describe("parseExpectation", () => {
  it("splits a verb from what the verb is about", () => {
    expect(parseExpectation("added:phenolphthalein")).toEqual({
      verb: "added",
      what: "phenolphthalein",
    });
  });

  it("reports no operand for a verb that stands alone", () => {
    expect(parseExpectation("not_yet_modelled")).toEqual({ verb: "not_yet_modelled", what: null });
  });

  it("splits at the first colon, so an operand may carry spaces", () => {
    // `polymer_heated:cured thermoset resin` is a real catalogue row, and
    // splitting on whitespace or on every colon would lose half of it.
    expect(parseExpectation("polymer_heated:cured thermoset resin")).toEqual({
      verb: "polymer_heated",
      what: "cured thermoset resin",
    });
  });
});

describe("expectationOperand", () => {
  it("looks a species up by its words, not by its slug", () => {
    expect(expectationOperand("methyl_orange", de)).toBe("Methylorange");
  });

  it("normalises a hyphenated reaction slug the same way", () => {
    expect(expectationOperand("peroxide-decomposition", de)).toBe("Peroxidzerfall");
  });

  it("leaves a formula alone — NaCl is NaCl in every language", () => {
    expect(expectationOperand("NaCl", de)).toBe("NaCl");
    expect(expectationOperand("CO2", de)).toBe("CO2");
  });
});

describe("expectationLabel", () => {
  it("renders German with the operand where German puts it", () => {
    // The whole point of a template rather than a verb beside a colon: the
    // operand leads in German and trails in English.
    expect(expectationLabel("added:phenolphthalein", de)).toBe("Phenolphthalein hinzugefügt");
    expect(expectationLabel("added:phenolphthalein", en)).toBe("phenolphthalein added");
  });

  it("translates a bare verb", () => {
    expect(expectationLabel("not_yet_modelled", de)).toBe("noch nicht modelliert");
    expect(expectationLabel("observed", de)).toBe("etwas wurde beobachtet");
  });

  it("keeps a formula intact inside a translated sentence", () => {
    expect(expectationLabel("gas_evolved:CO2", de)).toBe("CO2 als Gas freigesetzt");
  });

  it("never renders the wire token — that was the bug", () => {
    for (const want of ["added:phenolphthalein", "gas_evolved:CO2", "not_yet_modelled"]) {
      expect(expectationLabel(want, de)).not.toContain(":");
      expect(expectationLabel(want, de)).not.toContain("_");
    }
  });

  it("degrades to words for a verb nobody has taught it yet", () => {
    // Not an error: one row reads in English while the rest of the list is
    // German, which is the per-string fallback this whole app is built on.
    // `i18n.test.ts` is what makes this a build failure instead of a habit.
    expect(expectationLabel("some_new_event", de)).toBe("some new event");
    expect(expectationLabel("some_new_event:NaCl", de)).toBe("some new event: NaCl");
  });

  it("covers every verb the tables claim, with no verb in both", () => {
    // The two tables are a partition, not an overlap: a verb in both would
    // mean the bare form and the operand form disagree about what the same
    // token means.
    const bare = Object.keys(EXPECTATION_VERBS);
    const withOperand = Object.keys(EXPECTATION_VERBS_WITH);
    expect(bare.filter((verb) => withOperand.includes(verb))).toEqual([]);
    expect(bare.length + withOperand.length).toBe(29);
    // Every operand template must actually have somewhere to put the
    // operand, or the name it renders vanishes.
    for (const [verb, template] of Object.entries(EXPECTATION_VERBS_WITH)) {
      expect({ verb, template }).toEqual({ verb, template: expect.stringContaining("{what}") });
    }
  });
});
