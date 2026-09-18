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
 * Everything worth matching a query against, in both languages.
 *
 * NOT a predicate. This used to be `capabilityMatches`, a second search
 * predicate beside the catalogue's own — which is exactly how the app
 * ended up with two doors onto one question: two matchers cannot be kept
 * in step, and a reader typing into one of them got a wrong "no" about
 * the population behind the other. So a prompt now contributes its
 * haystack to the ONE index and `catalogEntryMatches` does the matching,
 * with `normalizeCatalogText` giving a German reader accent-insensitive
 * search the old predicate never had.
 *
 * The corpus's own translations only — never the shell dictionary. A
 * German haystack handed to an English reader would match words that
 * appear nowhere on their screen.
 */
export function capabilitySearchText(prompt: CapabilityPrompt, locale: string): string[] {
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
    prompt.reason_code,
    ...prompt.tags,
    ...prompt.script,
    ...localised,
  ];
}

/**
 * Why the corpus answers this question the way it does, as words.
 *
 * The corpus writes a machine code (`unsupported-fracture-mechanics`) and
 * ships no prose beside it — `boundary` is null on all five hundred rows,
 * so the explorer's `prompt.boundary ?? prompt.reason_code` always
 * rendered the code. Handing it to the dictionary as words gives it
 * German; `capabilities.reasons.test.ts` pins that every code the corpus
 * actually ships has an entry, because this reaches `t()` as a variable
 * and the literal scan in `i18n.test.ts` walks straight past it.
 */
export function capabilityReasonText(prompt: Pick<CapabilityPrompt, "boundary" | "reason_code">): string {
  return prompt.boundary?.trim() || words(prompt.reason_code);
}

/**
 * Can the bench actually run this question's script?
 *
 * SCRIPT PRESENCE, not support level — because they disagree. All sixty
 * `boundary` rows ship an EMPTY script (the refusal is the answer), and
 * the explorer gated its run button on `support !== "missing"` alone: it
 * therefore offered sixty rows a button that called the runner with an
 * empty string and produced nothing. A `missing` row has a script but no
 * science behind it yet, so it is excluded too.
 */
export function capabilityRunnable(prompt: Pick<CapabilityPrompt, "support" | "script">): boolean {
  return prompt.support !== "missing" && prompt.script.length > 0;
}
