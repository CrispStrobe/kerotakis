import { describe, expect, it } from "vitest";
import {
  applyCompletion,
  completionsFor,
  COMPLETION_LIMIT,
  rowForVerb,
  slotAt,
  verbWord,
  wordAt,
  type CompletionSources,
} from "./completions";

/**
 * The shapes the engine actually hands over, not invented ones.
 *
 * `grammar` rows are `Lab::grammar()`'s, `example` copied verbatim from
 * `kerotakis_core::script::VERBS` and `typed` from the same alias layer
 * that answers a German session. `shelf` rows are `Lab::species()`'s.
 */
const sources = (over: Partial<CompletionSources> = {}): CompletionSources => ({
  grammar: [
    { verb: "add", example: "add v1 water 100mL", typed: "gib v1 Wasser 100mL" },
    { verb: "heat", example: "heat v1 10kJ", typed: "erhitze v1 10kJ" },
    { verb: "stock", example: "stock NaCl 0.5mol", typed: "lagere NaCl 0.5mol" },
    { verb: "grind", example: "grind v1 NaCl 50um", typed: "zermahle v1 NaCl 50um" },
    { verb: "new", example: "new", typed: null },
    { verb: "measure", example: "measure v1 ph", typed: "miss v1 ph" },
  ],
  vessels: [
    { id: 0, label: "beaker" },
    { id: 1, label: "flask" },
  ],
  shelf: [
    { key: "water", name: "water", formula: "H2O" },
    { key: "NaCl", name: "table salt", formula: "NaCl" },
    { key: "NaOH", name: "sodium hydroxide", formula: "NaOH" },
  ],
  translate: (text) =>
    ({ beaker: "Becherglas", flask: "Kolben", water: "Wasser", "table salt": "Kochsalz", "sodium hydroxide": "Natronlauge" })[text] ?? text,
  ...over,
});

const inserts = (line: string, caret = line.length) =>
  completionsFor(line, caret, sources()).options.map((option) => option.insert);

describe("the word the caret is in", () => {
  it("finds the word and where it sits in the line", () => {
    expect(wordAt("heat v1", 7)).toEqual({ text: "v1", start: 5, end: 7, index: 1 });
    expect(wordAt("heat v1", 4)).toEqual({ text: "heat", start: 0, end: 4, index: 0 });
  });

  it("treats a caret after a space as the start of the next word", () => {
    // Which is exactly when a suggestion is most useful: the reader has
    // finished one argument and has not yet typed a letter of the next.
    expect(wordAt("add v1 ", 7)).toEqual({ text: "", start: 7, end: 7, index: 2 });
  });

  it("completes the word the caret is in, not the last one on the line", () => {
    expect(wordAt("add v1 water", 5)).toEqual({ text: "v1", start: 4, end: 6, index: 1 });
  });
});

describe("the verbs come from the grammar, in the reader's language", () => {
  it("offers this language's word for the verb, which the engine spelled", () => {
    // Nothing in this file knows the word "erhitze". It is the first word
    // of the engine's own localised example.
    expect(verbWord({ verb: "heat", example: "heat v1 10kJ", typed: "erhitze v1 10kJ" })).toBe("erhitze");
    expect(inserts("erh")).toEqual(["erhitze"]);
  });

  it("falls back to the canonical verb where a language has no word for it", () => {
    expect(verbWord({ verb: "new", example: "new", typed: null })).toBe("new");
    expect(inserts("ne")).toEqual(["new"]);
  });

  it("keeps English for somebody typing English", () => {
    // A reader who has written `hea` is writing `heat`. Completing them
    // into `erhitze` would be the bar rewriting their sentence.
    expect(inserts("hea")).toEqual(["heat"]);
    expect(inserts("gi")).toEqual(["gib"]);
  });

  it("carries the whole example, because a bare verb says nothing about its arguments", () => {
    const [option] = completionsFor("erh", 3, sources()).options;
    expect(option?.hint).toBe("erhitze v1 10kJ");
  });

  it("offers every verb on an empty line, up to the popup's limit", () => {
    expect(completionsFor("", 0, sources()).options.length).toBe(6);
    const many = sources({
      grammar: Array.from({ length: 30 }, (_, i) => ({ verb: `verb${i}`, example: `verb${i} v1`, typed: null })),
    });
    expect(completionsFor("", 0, many).options.length).toBe(COMPLETION_LIMIT);
  });
});

/**
 * The one idea in the module: the example line IS the argument schema.
 * `VERBS` says `add` is `add v1 water 100mL`, so position 1 is a vessel
 * and position 2 is a chemical — and a verb added to the engine tomorrow
 * arrives with its own schema and needs no edit here.
 */
describe("what follows a verb is read out of that verb's own example", () => {
  it("recognises the verb in either language before answering", () => {
    expect(rowForVerb(sources().grammar, "erhitze")?.verb).toBe("heat");
    expect(rowForVerb(sources().grammar, "HEAT")?.verb).toBe("heat");
    expect(rowForVerb(sources().grammar, "brew")).toBeUndefined();
  });

  it("offers vessels where the example has a vessel", () => {
    expect(slotAt(sources(), sources().grammar[1]!, 1)).toBe("vessel");
    expect(inserts("erhitze ")).toEqual(["v1", "v2"]);
    expect(inserts("add v")).toEqual(["v1", "v2"]);
  });

  it("offers chemicals where the example has a chemical — at whatever position it sits", () => {
    // `add` has its species at 2 and `stock` at 1. Nothing here knows
    // that; both come from the example lines.
    expect(inserts("gib v1 ")).toEqual(["water", "NaCl", "NaOH"]);
    expect(inserts("lagere ")).toEqual(["water", "NaCl", "NaOH"]);
    expect(inserts("zermahle v1 Na")).toEqual(["NaCl", "NaOH"]);
  });

  it("says nothing where the example has a number, a unit or a joining word", () => {
    // `add v1 water 100mL` — position 3 is an amount. Offering a guess at
    // how much somebody meant to add is worse than offering nothing.
    expect(inserts("gib v1 water ")).toEqual([]);
    expect(inserts("miss v1 ")).toEqual([]);
    expect(inserts("neu ")).toEqual([]);
  });

  it("says nothing at all after a word that is not a verb", () => {
    expect(inserts("brauerei ")).toEqual([]);
  });
});

describe("a chemical is shown in the reader's language and inserted in the engine's", () => {
  it("shows the translated name and the formula, and inserts the registry key", () => {
    const [option] = completionsFor("gib v1 Koch", 11, sources()).options;
    expect(option).toEqual({ kind: "reagent", insert: "NaCl", label: "Kochsalz", hint: "NaCl" });
  });

  it("finds a chemical by its German name as well as by its key", () => {
    // The point of the feature for its reader: they think "Kochsalz", and
    // the line that reaches the bench still says NaCl — the word the
    // parser is guaranteed to take, in every language.
    expect(inserts("gib v1 Was")).toEqual(["water"]);
    expect(inserts("gib v1 wat")).toEqual(["water"]);
  });

  it("names a vessel by what it is, not only by its number", () => {
    const [option] = completionsFor("erhitze v", 9, sources()).options;
    expect(option).toEqual({ kind: "vessel", insert: "v1", label: "v1", hint: "Becherglas" });
  });
});

describe("taking a suggestion", () => {
  it("replaces the word the caret is in and leaves a space for the next", () => {
    const line = "gib v1 Koch";
    const result = completionsFor(line, line.length, sources());
    expect(applyCompletion(line, result, result.options[0]!)).toEqual({
      line: "gib v1 NaCl ",
      caret: 12,
    });
  });

  it("does not double a space the line already has", () => {
    const line = "gib v water 100mL";
    const result = completionsFor(line, 5, sources());
    expect(applyCompletion(line, result, result.options[0]!)).toEqual({
      line: "gib v1 water 100mL",
      caret: 7,
    });
  });
});
