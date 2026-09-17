/**
 * Translated lesson prose, shipped beside the lessons (I18N-9).
 *
 * A lesson's title, description, section comments and boundary note are
 * the `.lab` file's own `#` comments. They were rendered verbatim, so a
 * German learner met a German lesson name above six English lines.
 *
 * A labelled comment — `#@intro Citric acid and baking soda…` — names the
 * PLACE the prose is said, and `lessons/prose/<code>.toml` answers with
 * that language's paragraph. `tools/lessons-index.py` compiles those files
 * into one `lessons/prose.json` beside `lessons/index.json`, so the payload
 * carries no parser and the shells and the web read the same bytes.
 *
 * This is deliberately NOT `t()`. The interface dictionary is keyed by its
 * English source, which is right for a button whose label is three words
 * and wrong for a paragraph: rewording the English would orphan every
 * translation of it at once, and two lessons that happen to open with the
 * same sentence would share one row. The label survives a rewrite.
 *
 * Adding French is `lessons/prose/fr.toml` and nothing else — no import
 * here, no map to extend, no existing translation touched.
 */

import { i18n, t } from "./i18n.svelte";

/** locale code to `<lesson-stem>.<label>` to that language's paragraph. */
type ProseTables = Record<string, Record<string, string>>;

let TABLES: ProseTables = {};

/**
 * Accept `lessons/prose.json`.
 *
 * Total: a payload built before the file existed, or served by a host that
 * 404s it, leaves the tables empty and every lesson renders the English the
 * `.lab` carries inline — the same honest degradation the rest of the
 * payload makes.
 */
export function loadLessonProse(raw: unknown): void {
  const tables: ProseTables = {};
  if (raw && typeof raw === "object") {
    for (const [code, rows] of Object.entries(raw as Record<string, unknown>)) {
      if (!rows || typeof rows !== "object") continue;
      const table: Record<string, string> = {};
      for (const [key, value] of Object.entries(rows as Record<string, unknown>)) {
        if (typeof value === "string") table[key] = value;
      }
      tables[code] = table;
    }
  }
  TABLES = tables;
}

/** For the tests, and for a reload that has to start from nothing. */
export function clearLessonProse(): void {
  TABLES = {};
}

/**
 * This language's wording for a labelled piece of lesson prose.
 *
 * The English the `.lab` carries is the fallback, per key: a lesson half
 * translated renders its translated paragraphs in German and the rest in
 * English, rather than waiting for a complete file before any of it ships.
 *
 * Unlabelled prose (every lesson not yet migrated) arrives with no key and
 * takes the path it always took, through the interface dictionary.
 */
export function lessonProse(key: string | undefined, english: string): string {
  if (key) {
    const translated = TABLES[i18n.locale]?.[key];
    if (translated) return translated;
  }
  return t(english);
}

/** Does this language carry a row for that label? For the render tests. */
export function hasLessonProse(key: string, locale: string): boolean {
  return Boolean(TABLES[locale]?.[key]);
}
