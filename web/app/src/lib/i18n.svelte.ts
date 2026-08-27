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

/** The languages this build can render, English first. */
export function availableLocales(): Locale[] {
  return ["en", ...Object.keys(TABLES).filter((c) => c !== "en").sort()];
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

  setLocale(locale: Locale) {
    this.locale = locale;
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
