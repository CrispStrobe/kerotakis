export type CapabilitySupport = "computed" | "curated" | "qualitative" | "boundary" | "missing";

export interface CapabilityPrompt {
  id: string;
  question: string;
  age_band: string;
  topic: string;
  material_class: string;
  tags: string[];
  script: string[];
  owning_task: string;
  support: CapabilitySupport;
  reason_code: string;
  boundary?: string | null;
  // Localised twins also travel on a prompt, as `tools/curiosity-prose.py`
  // folds them in: `question_de` beside `question`, `material_class_de`,
  // and a `tags_de` that indexes together with `tags`. They are not fields
  // here because the set of them is whatever languages shipped, and naming
  // them would make this interface the second place to edit for a new
  // language — the coupling one-file-per-language exists to avoid.
  // `localiseCapability` is the only reader.
}

export function parseCapabilityIndex(raw: unknown): CapabilityPrompt[] {
  const doc = raw as { schema?: number; prompts?: unknown[] };
  if (doc?.schema !== 1 || !Array.isArray(doc.prompts)) return [];
  return (doc.prompts as CapabilityPrompt[]).filter((prompt) =>
    typeof prompt?.id === "string" &&
    typeof prompt?.question === "string" &&
    typeof prompt?.support === "string" &&
    Array.isArray(prompt?.script),
  );
}

/** The English token as the dictionary keys it: `salt-water` -> "salt water". */
const words = (token: string) => token.replaceAll("_", " ").replaceAll("-", " ");

/** One cast, here, rather than one at every reader of a localised twin. */
const twin = (prompt: CapabilityPrompt, key: string): unknown =>
  (prompt as unknown as Record<string, unknown>)[key];

/**
 * What one prompt says in the reader's language.
 *
 * The explorer's chrome was translated (#441) while its CONTENT — the
 * five hundred questions it exists to show, and the material class and
 * concept tags under them — stayed the corpus's authored English. That
 * German cannot live in the corpus: it is a test artefact, joined to a
 * baseline by id and closed to unknown fields. It ships beside it, one
 * file per language, and arrives here as `question_de`-style siblings.
 *
 * Three fallbacks, in order, per field: the corpus translation, then the
 * shell's own dictionary (`t`), then the English itself — because `t`
 * returns its key when nothing has that key. So a language degrades one
 * FIELD at a time. A corpus that grows twenty questions before anyone
 * translates them shows twenty English rows in a German list, which reads
 * as an untranslated question rather than as a missing one.
 *
 * `t` is passed in rather than imported so this stays a plain function
 * over data: the tests can ask what a locale renders without a component,
 * a store, or a bundle.
 */
export interface LocalisedCapability {
  question: string;
  materialClass: string;
  tags: string[];
}

export function localiseCapability(
  prompt: CapabilityPrompt,
  locale: string,
  t: (message: string) => string,
): LocalisedCapability {
  // English is the source text, so it never consults a twin of itself.
  const suffix = locale === "en" ? null : `_${locale}`;
  const question = suffix && twin(prompt, `question${suffix}`);
  const materialClass = suffix && twin(prompt, `material_class${suffix}`);
  const tags = suffix && twin(prompt, `tags${suffix}`);
  return {
    question: typeof question === "string" && question ? question : t(prompt.question),
    materialClass:
      typeof materialClass === "string" && materialClass
        ? materialClass
        : t(words(prompt.material_class)),
    // A twin of a different length cannot be positional, so it is dropped
    // and the tags fall back together rather than label the wrong chips.
    tags:
      Array.isArray(tags) && tags.length === prompt.tags.length
        ? (tags as string[])
        : prompt.tags.map((tag) => t(tag.replaceAll("_", " "))),
  };
}

/**
 * Does this prompt match what the reader typed?
 *
 * Searches the English AND the reader's own language. A German reader
 * given a German list has no reason to guess that the box wants English,
 * and a search that silently only matched the hidden source text would
 * make the translation look broken rather than the search.
 */
export function capabilityMatches(
  prompt: CapabilityPrompt,
  query: string,
  locale = "en",
): boolean {
  const needle = query.trim().toLocaleLowerCase();
  if (!needle) return true;
  const suffix = locale === "en" ? null : `_${locale}`;
  const localised = suffix
    ? [twin(prompt, `question${suffix}`), twin(prompt, `material_class${suffix}`)]
        .concat((twin(prompt, `tags${suffix}`) as unknown[]) ?? [])
        .filter((value): value is string => typeof value === "string")
    : [];
  return [
    prompt.id,
    prompt.question,
    prompt.topic,
    prompt.material_class,
    prompt.owning_task,
    ...prompt.tags,
    ...localised,
  ].some((value) => value.replaceAll("_", " ").toLocaleLowerCase().includes(needle));
}
