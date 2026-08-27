/**
 * A BCP-47 primary tag.
 *
 * Open by design. It was `"en" | "de"`, which made the type the second
 * place to edit when adding a language and the one people forget; now the
 * set of languages is whatever is in `src/locales`, and `availableLocales()`
 * reports it.
 */
export type Locale = string;

type Vars = Record<string, string | number>;

/** Domain vocabulary returned by the engine rather than authored in components. */
/** A translation bundle: one JSON file per language, in `src/locales`. */
type Bundle = {
  "@@locale": string;
  /** What this language calls itself — "Deutsch", not "German". */
  "@@name"?: string;
  /** The engine's vocabulary — species, colours, hazards. */
  terms?: Record<string, string>;
  /** The interface's own strings, plus the names the codex refers to by slug. */
  messages?: Record<string, string>;
};

/**
 * Every language shipped, discovered by filename.
 *
 * Adding French is `locales/fr.json` and nothing else: no import to add,
 * no map to extend, no existing translation touched. That file boundary is
 * the point — two translators working on two languages never edit the same
 * file, which is what makes the work parallelisable at all.
 *
 * `eager`, so bundles are inlined at build time. A language fetched over
 * the network is a language you cannot read on a train, and this app is
 * offline-first.
 *
 * `_template.json` is the empty shape a new translation starts from, so it
 * is skipped rather than offered as a language nobody can read.
 */
const BUNDLES: Record<string, Bundle> = Object.fromEntries(
  Object.entries(
    import.meta.glob<Bundle>("../locales/*.json", { eager: true, import: "default" }),
  )
    .filter(([path]) => !path.split("/").pop()!.startsWith("_"))
    .map(([, bundle]) => [bundle["@@locale"], bundle]),
);

/**
 * One flat lookup per language.
 *
 * `terms` first so `messages` wins where both carry a key: that is the
 * order the two literal maps had, and six keys actually collide.
 */
const TABLES: Record<string, Record<string, string>> = Object.fromEntries(
  Object.entries(BUNDLES).map(([code, b]) => [
    code,
    { ...(b.terms ?? {}), ...(b.messages ?? {}) },
  ]),
);

/**
 * The languages this build can render, English first, each with the name
 * it calls itself.
 *
 * The endonym, deliberately: a reader who cannot read the current
 * interface language still has to find their own in the list, and
 * "Deutsch" is findable in a way that a translated "German" is not. It
 * comes from the bundle, so a new language names itself without needing an
 * entry in every other language's bundle first.
 */
export function availableLocales(): { code: Locale; name: string }[] {
  return [
    { code: "en", name: "English" },
    ...Object.entries(BUNDLES)
      .filter(([code]) => code !== "en")
      .map(([code, b]) => ({ code, name: b["@@name"] || code }))
      .sort((a, b) => a.name.localeCompare(b.name)),
  ];
}

function detectLocale(): Locale {
  if (typeof window !== "undefined") {
    try {
      const saved = window.localStorage.getItem("kerotakis.locale");
      if (saved && (saved === "en" || saved in TABLES)) return saved;
    } catch {
      // Storage may be unavailable in privacy modes; browser language remains enough.
    }
  }
  // Match the browser's language against what is actually shipped, by
  // primary subtag: de-AT is German. An unshipped language falls back to
  // English rather than to a half-translated screen.
  if (typeof navigator !== "undefined") {
    const primary = navigator.language.toLowerCase().split("-")[0] ?? "";
    if (primary in TABLES) return primary;
  }
  return "en";
}

class I18n {
  locale = $state<Locale>(detectLocale());

  constructor() {
    this.applyDocumentLanguage();
  }

  /**
   * Told when the language changes, so the ENGINE can follow.
   *
   * A subscription rather than a call in the switcher: the engine renders
   * its own prose and has to be switched separately, and putting that in
   * one caller means the next caller added forgets it. Here, every caller
   * of setLocale updates the engine whether or not they know about it.
   */
  private readonly watchers = new Set<(locale: Locale) => void>();

  onChange(fn: (locale: Locale) => void): () => void {
    this.watchers.add(fn);
    return () => this.watchers.delete(fn);
  }

  setLocale(locale: Locale) {
    this.locale = locale;
    for (const fn of this.watchers) fn(locale);
    if (typeof window !== "undefined") {
      try {
        window.localStorage.setItem("kerotakis.locale", locale);
      } catch {
        // The live choice still works when persistence is blocked.
      }
    }
    this.applyDocumentLanguage();
  }

  t(message: string, vars: Vars = {}): string {
    // English is the source text, so it is the key AND the fallback: a
    // message no bundle carries renders as itself rather than as a key.
    const template = TABLES[this.locale]?.[message] ?? message;
    return template.replace(/\{(\w+)\}/g, (_, key: string) => String(vars[key] ?? `{${key}}`));
  }

  private applyDocumentLanguage() {
    if (typeof document === "undefined") return;
    document.documentElement.lang = this.locale;
    document.documentElement.dir = "ltr";
    document.title = this.t("Kerotakis — the bench");
    document
      .querySelector<HTMLMetaElement>('meta[name="description"]')
      ?.setAttribute(
        "content",
        this.t(
          "A virtual chemistry laboratory that computes real chemistry — drag reagents onto drawn glassware and watch a real aqueous solver answer. Offline once loaded.",
        ),
      );
  }
}

export const i18n = new I18n();
export const t = (message: string, vars?: Vars) => i18n.t(message, vars);

/**
 * Does `locale` have a translation for this exact source string?
 *
 * For the coverage tests, which scan every `t("…")` call site and every
 * registry name and assert nothing is missing. It answers about the
 * DICTIONARY, not about the screen — a key present here can still be a
 * key nobody renders, which is a distinction this codebase has learned
 * the hard way. `tools/test-i18n-render.mjs` answers the other question.
 *
 * Recovered during a merge: main added this and the tests that use it
 * while this branch was replacing the maps it read from, and resolving
 * the conflict in favour of the new loader dropped it. The tests survived
 * and failed loudly, which is the only reason it came back.
 */
export const hasTranslation = (message: string, locale: Locale = "de"): boolean =>
  Object.hasOwn(TABLES[locale] ?? {}, message);

/** @deprecated Use `hasTranslation(message, "de")`. Kept for the tests
 * main wrote against the old two-language shape. */
export const hasGermanTranslation = (message: string): boolean => hasTranslation(message, "de");

/** Translate a codex identifier: `strong-bases` -> "starke Basen".
 *
 * The catalogue names concepts, models and apparatus as slugs; the
 * dictionary is keyed by the words. Something has to bridge the two, and
 * it belongs here rather than in each caller — ConceptMap inlined its own
 * `.replace(/-/g, " ")` for node labels and every list beside those nodes
 * forgot to, which is how German nodes ended up over English slugs.
 */
export const tSlug = (slug: string): string => t(slug.replace(/-/g, " "));


/** Pick the localised variant of a field the *engine* supplied (I18N-3).
 *
 * Engine records carry their German in a sibling key — `purpose` beside
 * `purpose_de` — rather than in a nested per-locale map. That shape
 * degrades one field at a time: a string nobody has translated yet falls
 * back to English on its own, without the record needing a complete
 * German twin before any of it can ship. `t()` cannot do this job: these
 * strings are not in the shell's dictionary and never will be, because
 * they belong to the engine and travel with it.
 */
export function tEngine(record: object | undefined | null, field: string): string {
  if (!record) return "";
  // One cast here rather than one at every call site: the codex types are
  // interfaces without index signatures, and widening them at each caller
  // would discard the field-name checking that makes them worth having.
  const r = record as Record<string, unknown>;
  // The catalogue names its translations by locale suffix: `purpose_de`,
  // and `purpose_fr` when there is one. English is the unsuffixed field.
  const translated = i18n.locale === "en" ? undefined : r[`${field}_${i18n.locale}`];
  return String(translated ?? r[field] ?? "");
}
