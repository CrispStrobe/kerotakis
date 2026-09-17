/**
 * The corpus's own vocabulary, against the index that has to place it.
 *
 * The five hundred reviewed questions are now rows of the ONE catalogue
 * index, which means three of the corpus's fields have to answer the
 * catalogue's filters: its eight topics have to fold into the shared chip
 * vocabulary, its age bands have to become learning levels, and its reason
 * codes have to reach a German reader as words.
 *
 * Every one of those reaches `t()` as a VARIABLE, so the literal scan in
 * `i18n.test.ts` walks straight past all of them — which is exactly how
 * #505 happened (`models.toml` reported 100% German over 325 English
 * strings). So the denominator here is read from the shipped corpus rather
 * than typed: a shard that grows a ninth topic, a fifty-eighth reason code
 * or a new band fails this file instead of quietly rendering English.
 */
import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { hasGermanTranslation } from "./i18n.svelte";
import { CATALOG_TOPICS, CORPUS_BANDS, CORPUS_TOPICS } from "./catalogEntry";

const corpus = join(import.meta.dirname, "../../../../tests/coverage/curiosity-v1");
const SHARDS = ["aqueous-household", "thermal-fire", "materials-handling", "food-organic-bio"];

/** Every value of one `key = "value"` field across the question shards. */
function field(name: string): string[] {
  const found: string[] = [];
  for (const shard of SHARDS) {
    const text = readFileSync(join(corpus, `${shard}.toml`), "utf8");
    for (const [, value] of text.matchAll(new RegExp(`^${name}\\s*=\\s*"([^"]+)"$`, "gm"))) {
      found.push(value!);
    }
  }
  return found;
}

/** The reviewed verdicts live apart from the questions, one per observation. */
const reasonCodes = [...new Set(
  [...readFileSync(join(corpus, "baseline.toml"), "utf8")
    .matchAll(/^reason_code\s*=\s*"([^"]+)"$/gm)].map((m) => m[1]!),
)].sort();

/**
 * The exporter renames this field, and the test has to follow.
 *
 * A shard authors `action = "mix_and_dissolve"`; `curiosity-index.py`
 * writes it into the browser index as `topic`. Reading the browser's name
 * out of the corpus found nothing at all and the mapping check below
 * passed over an empty list — which is why the vacuity guard is the first
 * assertion in this file and not an afterthought.
 */
const topics = [...new Set(field("action"))].sort();
const bands = [...new Set(field("age_band"))].sort();

/** The dictionary is keyed by words, never by the machine's hyphens. */
const words = (code: string) => code.replaceAll("-", " ").replaceAll("_", " ");

describe("the corpus vocabulary the one index has to place", () => {
  it("reads a corpus at all, so none of this passes vacuously", () => {
    expect(field("id").length).toBe(500);
    expect(reasonCodes.length).toBeGreaterThan(40);
    expect(topics.length).toBeGreaterThan(5);
  });

  it("folds every corpus topic into the shared chip vocabulary", () => {
    // Unmapped is not "uncategorised" — it is a question the topic chips
    // cannot select, in a list whose whole promise is one search.
    const unmapped = topics.filter((topic) => !Object.hasOwn(CORPUS_TOPICS, topic));
    expect(unmapped).toEqual([]);
    // And nothing folds into a chip that does not exist.
    const invented = Object.values(CORPUS_TOPICS)
      .flat()
      .filter((topic) => !CATALOG_TOPICS.includes(topic));
    expect(invented).toEqual([]);
  });

  it("reads every corpus age band as a learning level, or as none", () => {
    // `all` is deliberately absent from the map: it is not a band, and a
    // question authored for every level gets `anyLevel` rather than being
    // filed under one. Anything ELSE absent is a row whose level chip
    // would silently read "first steps" it was never given.
    const unread = bands.filter((band) => band !== "all" && !Object.hasOwn(CORPUS_BANDS, band));
    expect(unread).toEqual([]);
    expect(Object.hasOwn(CORPUS_BANDS, "all")).toBe(false);
  });

  it("has German for every reason code the corpus actually ships", () => {
    const untranslated = reasonCodes.filter((code) => !hasGermanTranslation(words(code)));
    expect(untranslated).toEqual([]);
  });
});
