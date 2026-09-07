/**
 * The capability explorer, read in German.
 *
 * The dialog was translated by #441 and its CONTENT was not: a German
 * reader got German buttons over five hundred English questions. The
 * German now lives beside the corpus
 * (`tests/coverage/curiosity-v1/i18n/de.toml`, keyed by prompt id) and is
 * folded into the browser index as `question_de`-style siblings.
 *
 * This file asks the question the exporter's own self-test cannot: what
 * does the SCREEN say? It reads the shipped corpus and the shipped
 * translation directly rather than a fixture, because a fixture would go
 * on passing after the corpus grew a shard nobody translated — which is
 * precisely the regression worth catching. `t` is passed in as identity,
 * so anything German here came from the corpus translation and not from
 * the shell's dictionary quietly covering for it.
 */
import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { capabilityMatches, localiseCapability, type CapabilityPrompt } from "./capabilities";

const corpus = join(import.meta.dirname, "../../../../tests/coverage/curiosity-v1");

/** The corpus is authored one `key = "value"` per line, with no escapes. */
function rows(text: string, section: string): Map<string, string> {
  const body = text.split(`\n[${section}]\n`)[1]?.split("\n[")[0] ?? "";
  const found = new Map<string, string>();
  for (const line of body.split("\n")) {
    const [, key, value] = /^"([^"]+)" = "([^"]*)"$/.exec(line.trim()) ?? [];
    if (key !== undefined && value !== undefined) found.set(key, value);
  }
  return found;
}

const german = readFileSync(join(corpus, "i18n/de.toml"), "utf8");
const questions = rows(german, "question");
const materialClasses = rows(german, "material_class");
const tags = rows(german, "tag");

/** The English the shards authored, as `{id: question}`. */
const english = new Map<string, string>();
for (const shard of ["aqueous-household", "thermal-fire", "materials-handling", "food-organic-bio"]) {
  const text = readFileSync(join(corpus, `${shard}.toml`), "utf8");
  let id: string | null = null;
  for (const line of text.split("\n")) {
    const [, foundId] = /^id = "([^"]+)"$/.exec(line) ?? [];
    if (foundId !== undefined) id = foundId;
    const [, question] = /^question = "(.+)"$/.exec(line) ?? [];
    if (question !== undefined && id !== null) english.set(id, question);
  }
}

/** One prompt as the exporter writes it into `capabilities/index.json`. */
function shipped(id: string, materialClass: string, promptTags: string[]): CapabilityPrompt {
  const row = {
    id,
    question: english.get(id)!,
    age_band: "age9_to12",
    topic: "mix_and_dissolve",
    material_class: materialClass,
    tags: promptTags,
    script: [],
    owning_task: "CAP-23",
    support: "computed",
    reason_code: "computed-route",
    question_de: questions.get(id),
    material_class_de: materialClasses.get(materialClass),
    tags_de: promptTags.map((tag) => tags.get(tag) ?? tag),
  };
  return row as unknown as CapabilityPrompt;
}

/** No dictionary behind the corpus — only the corpus's own translation. */
const identity = (message: string) => message;

describe("the capability explorer in German", () => {
  it("has German for every question the corpus ships", () => {
    expect(english.size).toBe(500);
    expect(questions.size).toBe(500);
    const untranslated = [...english.keys()].filter((id) => !questions.get(id));
    expect(untranslated).toEqual([]);
  });

  it("shows no English question for a sample spanning every shard", () => {
    const sample = ["aq-001", "aq-051", "aq-122", "th-001", "th-069", "mat-005", "mat-054", "bio-008", "bio-016", "bio-075"];
    for (const id of sample) {
      const prompt = shipped(id, "salt-water", ["dissolution"]);
      const said = localiseCapability(prompt, "de", identity);
      expect(said.question).toBe(questions.get(id));
      // The English is what a missing translation would render, so a
      // German field that equals it is the failure, not a coincidence.
      expect(said.question).not.toBe(english.get(id));
      expect(said.question.length).toBeGreaterThan(10);
    }
  });

  it("translates the material class and the concept tags", () => {
    const prompt = shipped("aq-001", "salt-water", ["dissolution", "solubility"]);
    const said = localiseCapability(prompt, "de", identity);
    expect(said.materialClass).toBe("Salzwasser");
    expect(said.tags).toEqual(["Auflösung", "Löslichkeit"]);
  });

  it("keeps a task identifier as a name rather than translating it", () => {
    const prompt = shipped("aq-001", "salt-water", ["dissolution", "CAP-5"]);
    expect(localiseCapability(prompt, "de", identity).tags).toEqual(["Auflösung", "CAP-5"]);
  });

  it("reads English when the reader's language is the source", () => {
    const prompt = shipped("aq-001", "salt-water", ["dissolution"]);
    const said = localiseCapability(prompt, "en", identity);
    expect(said.question).toBe(english.get("aq-001"));
    expect(said.materialClass).toBe("salt water");
  });

  it("falls back per field for a prompt this language has not reached", () => {
    // A corpus can grow a question before anyone translates it, and a
    // language can ship the questions before the glossaries. Each hole
    // degrades on its own rather than blanking the row.
    const untranslated = {
      ...shipped("aq-001", "salt-water", ["dissolution"]),
      question_de: undefined,
      material_class_de: "",
      tags_de: ["Auflösung", "zu viele"],
    } as unknown as CapabilityPrompt;
    const said = localiseCapability(untranslated, "de", identity);
    expect(said.question).toBe(english.get("aq-001"));
    expect(said.materialClass).toBe("salt water");
    // A twin of the wrong length cannot be positional, so the chips fall
    // back together instead of labelling the wrong tag.
    expect(said.tags).toEqual(["dissolution"]);
  });

  it("falls back to the shell's dictionary before falling back to English", () => {
    const untranslated = {
      ...shipped("aq-001", "salt-water", ["dissolution"]),
      question_de: undefined,
    } as unknown as CapabilityPrompt;
    const said = localiseCapability(untranslated, "de", () => "Aus dem Wörterbuch");
    expect(said.question).toBe("Aus dem Wörterbuch");
  });

  it("finds a German question from a German search box", () => {
    const prompt = shipped("aq-001", "salt-water", ["dissolution"]);
    expect(capabilityMatches(prompt, "Kochsalz", "de")).toBe(true);
    expect(capabilityMatches(prompt, "Salzwasser", "de")).toBe(true);
    expect(capabilityMatches(prompt, "Auflösung", "de")).toBe(true);
    // The English still matches: an id or a formula is what people paste.
    expect(capabilityMatches(prompt, "table salt", "de")).toBe(true);
    expect(capabilityMatches(prompt, "Elektrolyse", "de")).toBe(false);
    // English readers are not given a German haystack to match against.
    expect(capabilityMatches(prompt, "Kochsalz", "en")).toBe(false);
  });
});
