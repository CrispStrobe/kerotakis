/** Every authored sentence in the CODEX has a German sibling (I18N-1).
 *
 * `codexTerms.test.ts` covers the labels the catalogue refers to by SLUG —
 * titles and concept names, which live in the locale bundle. This covers the
 * other half: the authored PROSE that travels inside the catalogue itself and
 * never passes through a bundle at all. The hook on a card, the three
 * registers behind an entry, the prediction and every diagnosis of a wrong
 * answer, and — for a model — its name, what it buys you, what it explains
 * and where it stops.
 *
 * That prose reaches the shell as `_de` siblings folded in from
 * `codex/i18n/de.toml` by `Codex::load_dir`, and is picked up by `tEngine`
 * and by three hand-written locale pickers in `Catalog.svelte` and
 * `catalogEntry.ts`. Every one of them falls back to English per string,
 * which is the behaviour that lets a translation ship unfinished — and is
 * exactly why a hole is invisible: a missing `lv2_de` renders a fluent
 * English paragraph under a German heading, and nothing anywhere reports it.
 * That is failure mode #2 of I18N.md's five.
 *
 * `tools/codex-locale-lint.py --check` gates the same claim from the data
 * side, in preflight. This gates it from the side the reader is on: the
 * exported document the app actually parses.
 */
import { describe, expect, it } from "vitest";
import { i18n, tEngine } from "./i18n.svelte";
import codexExportJson from "../../../../crates/kerotakis-codex/tests/golden/codex-export.json?raw";

type Registers = Record<string, string>;
type Diagnosis = {
  reveals?: string | null;
  next?: string | null;
  reveals_de?: string | null;
  next_de?: string | null;
};
type Predict = {
  question?: string | null;
  options?: string[] | null;
  misconception?: string | null;
  diagnosis?: Diagnosis[];
  question_de?: string;
  options_de?: string[];
  misconception_de?: string;
};
type Reaction = {
  id: string;
  summary?: string | null;
  summary_de?: string | null;
  registers: Registers;
  expect?: { predict?: Predict };
};
type Model = {
  id: string;
  name?: string | null;
  power?: string | null;
  explains?: string[] | null;
  fails_at?: string[] | null;
  registers: Registers;
} & Record<string, unknown>;

const codex = JSON.parse(codexExportJson) as { reactions: Reaction[]; models: Model[] };

/** One authored string and its translation, named by where it lives. */
type Pair = { where: string; english: string | string[]; german: unknown };

const pairs: Pair[] = [];

/** `null`, not `undefined`, is how an absent field arrives.
 *
 * `summary` is `Option<String>` with no `skip_serializing_if`, so the 56
 * entries that carry an `equation` instead serialise `"summary": null` —
 * present as a key, empty as a value. An empty list is the same statement in
 * the other shape. Neither is an untranslated string; there is nothing there
 * to translate.
 */
const authored = (v: string | string[] | null | undefined): v is string | string[] =>
  v !== null && v !== undefined && v.length > 0;

const add = (where: string, english: string | string[] | null | undefined, german: unknown) => {
  if (!authored(english)) return;
  pairs.push({ where, english, german });
};

/** The register map carries its languages as suffixed KEYS, not as fields. */
const registerPairs = (id: string, registers: Registers) => {
  for (const [key, value] of Object.entries(registers)) {
    if (key.includes("_")) continue; // `lv1_de` is the translation, not a source
    add(`${id}.registers.${key}`, value, registers[`${key}_de`]);
  }
};

for (const r of codex.reactions) {
  add(`${r.id}.summary`, r.summary, r.summary_de);
  registerPairs(r.id, r.registers);
  const p = r.expect?.predict;
  if (p) {
    add(`${r.id}.question`, p.question, p.question_de);
    add(`${r.id}.options`, p.options, p.options_de);
    add(`${r.id}.misconception`, p.misconception, p.misconception_de);
    (p.diagnosis ?? []).forEach((d, i) => {
      add(`${r.id}.diagnosis.${i}.reveals`, d.reveals, d.reveals_de);
      add(`${r.id}.diagnosis.${i}.next`, d.next, d.next_de);
    });
  }
}

for (const m of codex.models) {
  add(`${m.id}.name`, m.name, m.name_de);
  add(`${m.id}.power`, m.power, m.power_de);
  add(`${m.id}.explains`, m.explains, m.explains_de);
  add(`${m.id}.fails_at`, m.fails_at, m.fails_at_de);
  registerPairs(m.id, m.registers);
}

/** Values that are the same string in German because they are not prose.
 *
 * Each is here with its reason. Notation and bare numerals are the only two
 * shapes that qualify: anything with a verb in it has a German form, and a
 * translator who cannot find one has found a sentence worth rewriting in
 * English first.
 */
const SAME_IN_BOTH = new Set([
  // "Na⁺ → Na* → Na + hν (589 nm)" — an emission written in symbols. There
  // is no German spelling of an arrow or a wavelength.
  "flame-test-sodium.summary",
  // ["4", "1", "3"] — the options are bare numerals answering "how many
  // tenfold dilutions?". Digits are digits; a decimal comma cannot arise
  // in an integer.
  "dilution-by-ten.options",
]);

describe("the German catalogue renders no English prose", () => {
  it("finds the catalogue, so the walk is not vacuous", () => {
    // Asserted rather than derived: a walk over an empty list passes
    // loudly and proves nothing.
    expect(codex.reactions).toHaveLength(108);
    expect(codex.models).toHaveLength(28);
    expect(pairs.length).toBeGreaterThanOrEqual(1250);
  });

  it("has a German sibling for every authored English string", () => {
    const missing = pairs.filter((p) => !authored(p.german as string | string[] | null));
    expect(missing.map((p) => p.where).sort()).toEqual([]);
  });

  it("translates rather than echoing the English", () => {
    // A German value byte-identical to its English source is, almost always,
    // a translation nobody wrote — and it renders as fluent English under a
    // German heading, which is the failure this whole file exists to catch.
    //
    // "Almost always" is why SAME_IN_BOTH exists rather than a blanket
    // exemption for short strings: an echo has to be ARGUED, one value at a
    // time, or the rule decays into whatever the current data happens to be.
    const echoed = pairs.filter(
      (p) => !SAME_IN_BOTH.has(p.where) && JSON.stringify(p.german) === JSON.stringify(p.english),
    );
    expect(echoed.map((p) => p.where).sort()).toEqual([]);
  });

  it("keeps every positional list the same length as its English", () => {
    // `options` is read BY INDEX: `answer` is an index into it and each
    // diagnosis attaches by index, so a translated list of a different
    // length marks a different answer correct. The UI treats a mismatch as
    // absent, which is safe and silent — hence this.
    const ragged = pairs.filter(
      (p) => Array.isArray(p.english) && Array.isArray(p.german) && p.german.length !== p.english.length,
    );
    expect(ragged.map((p) => p.where).sort()).toEqual([]);
  });

  it("reaches the reader through the helper the surfaces actually call", () => {
    // The claim above is about the document. This is about the lookup:
    // `tEngine(record, field)` is what `Catalog.svelte` calls for the
    // prediction, the misconception and both halves of a diagnosis.
    const entry = codex.reactions.find((r) => r.id === "strong-base");
    expect(entry).toBeDefined();
    const predict = entry!.expect!.predict!;
    i18n.locale = "de";
    try {
      expect(tEngine(predict, "question")).toBe(predict.question_de);
      expect(tEngine(predict, "question")).not.toBe(predict.question);
      // English is the source text, so it stays the fallback.
      i18n.locale = "en";
      expect(tEngine(predict, "question")).toBe(predict.question);
    } finally {
      i18n.locale = "en";
    }
  });
});
