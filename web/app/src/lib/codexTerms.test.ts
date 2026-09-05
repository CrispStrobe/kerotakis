/** Every name the CODEX refers to by slug has a German word (I18N-2).
 *
 * Two families of label live in `src/locales/*.json` without any component
 * ever writing them down:
 *
 *   - the 105 experiment titles, rendered by `Catalog` and
 *     `ConceptMap` as `t(entry.id.replace(/-/g, " "))`;
 *   - the concept slugs each experiment declares, rendered through
 *     `tSlug`, which is the same lookup with the dashes turned to spaces.
 *
 * `i18n.test.ts` cannot see either: it scans source files for literal
 * `t("…")` arguments, and these arguments are computed at runtime from
 * catalogue data. So the whole surface was invisible to every gate — a
 * missing one renders the English slug, which reads as a design choice
 * rather than as a hole. That is failure mode #1 of I18N.md's five, and
 * the only defence named there that works is to walk a source of truth and
 * compare.
 *
 * The source of truth is the exported catalogue rather than `codex/*.toml`:
 * it is the shape the app actually reads, it needs no TOML parser here,
 * and `crates/kerotakis-codex/tests/export_snapshot.rs` fails if it drifts
 * from the TOMLs. An experiment added to the codex therefore reaches this
 * test, and fails it until it has a German title.
 */
import { describe, expect, it } from "vitest";
import { hasTranslation, i18n, t, tSlug } from "./i18n.svelte";
// `?raw` rather than `node:fs`: the bundler resolves the path, and
// `vite/client` types the result, so this file needs no `@types/node`.
// The three sibling test files here still import `node:fs` and each
// contributes six svelte-check errors for it; this one adds none.
import codexExportJson from "../../../../crates/kerotakis-codex/tests/golden/codex-export.json?raw";
import germanBundleJson from "../locales/de.json?raw";

type ExportedReaction = { id: string; concepts?: string[] };

const codex = JSON.parse(codexExportJson) as { reactions: ExportedReaction[] };

/** The experiment ids, in catalogue order. */
const experiments = codex.reactions.map((r) => r.id);

/** Every concept slug any experiment declares, deduplicated and sorted.
 *
 * Not `codex/concepts.toml`: that is the OEH topic spine, which carries
 * its own `label_de` and is translated by being German already. These are
 * the teaching words the entries themselves name — `strong-bases`,
 * `spectator-ions` — and they have no German anywhere but the bundle.
 */
const concepts = [...new Set(codex.reactions.flatMap((r) => r.concepts ?? []))].sort();

/** The dictionary key a slug is looked up under, exactly as `tSlug` does. */
const key = (slug: string) => slug.replace(/-/g, " ");

describe("codex labels are translated", () => {
  it("finds the catalogue, so the walk is not vacuous", () => {
    // Every count below is asserted rather than derived, because a test
    // that walks an empty list passes loudly and means nothing.
    expect(experiments).toHaveLength(105);
    expect(concepts.length).toBeGreaterThanOrEqual(153);
  });

  it("has a German title for every experiment", () => {
    const missing = experiments.filter((id) => !hasTranslation(key(id), "de"));
    expect(missing.sort()).toEqual([]);
  });

  it("has a German label for every concept an experiment names", () => {
    const missing = concepts.filter((slug) => !hasTranslation(key(slug), "de"));
    expect(missing.sort()).toEqual([]);
  });

  it("translates rather than echoing the slug", () => {
    // A value equal to its key is indistinguishable from a key nobody
    // filled in: both render the English words. Whatever the German is, it
    // is not the English slug with the dashes taken out.
    const german = JSON.parse(germanBundleJson) as { messages: Record<string, string> };
    const echoed = [...experiments, ...concepts].filter((slug) => german.messages[key(slug)] === key(slug));
    expect(echoed.sort()).toEqual([]);
  });

  it("renders German through the lookups the surfaces actually call", () => {
    // The catalogue and the concept map do not share a helper: one calls
    // `t(id.replace(…))` on the entry, the other `tSlug` on the concept.
    // Both are exercised here, because a fix to one has repeatedly left
    // the other reading English beside it.
    i18n.locale = "de";
    try {
      expect(t(key("strong-base"))).toBe("Starke Base");
      expect(t(key("catalyst-area-and-stirring-change-the-rate"))).toBe(
        "Auch Katalysatorfläche und Rühren ändern die Geschwindigkeit",
      );
      expect(tSlug("spectator-ions")).toBe("Zuschauerionen");
      expect(tSlug("common-ion-effect")).toBe("Gleichioneneffekt");
      expect(tSlug("heterogeneous-catalysis")).toBe("Heterogene Katalyse");
      // English is the source text, so it stays the fallback.
      i18n.locale = "en";
      expect(tSlug("spectator-ions")).toBe("spectator ions");
    } finally {
      i18n.locale = "en";
    }
  });
});
