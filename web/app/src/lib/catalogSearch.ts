/**
 * Search and progress predicates, one per record.
 *
 * The catalogue itself now matches through `catalogEntry.ts`, which builds
 * ONE index over both corpora — a single box cannot be allowed to search
 * half the shelf. These stay because they are the per-record predicates the
 * shelf and the instrument wall use, and because `searchLocalized.test.ts`
 * pins the defect they were written for: a surface that displayed German
 * and filtered on English, so typing the word on screen returned nothing.
 */
import type { ShelfItem } from "./session.svelte";
import type { CodexEntry } from "./codex";

export type ExperimentProgressFilter = "all" | "not-tried" | "completed";

/** Completion is a persisted successful Codex run, never catalog availability. */
export function experimentHasProgress(
  entry: Pick<CodexEntry, "id">,
  completedIds: ReadonlySet<string>,
  filter: ExperimentProgressFilter,
): boolean {
  if (filter === "all") return true;
  const done = completedIds.has(entry.id);
  return filter === "completed" ? done : !done;
}

export function experimentProgressLabel(
  entry: Pick<CodexEntry, "id">,
  completedIds: ReadonlySet<string>,
): "completed" | "not tried" {
  return completedIds.has(entry.id) ? "completed" : "not tried";
}

/** Search as a learner types: case- and accent-insensitive, without changing formulae. */
export function normalizeCatalogText(value: string): string {
  return value
    .normalize("NFKD")
    .replace(/\p{Diacritic}/gu, "")
    .toLocaleLowerCase();
}

export function reagentMatches(
  item: Pick<ShelfItem, "key" | "name" | "formula">,
  query: string,
  localizedName: string,
): boolean {
  const needle = normalizeCatalogText(query.trim());
  if (!needle) return true;
  return [localizedName, item.name, item.formula, item.key]
    .some((value) => normalizeCatalogText(value).includes(needle));
}

/** Match an instrument in both its stable command vocabulary and displayed locale. */
export function equipmentMatches(
  item: { verb: string; title: string; blurb: string },
  query: string,
  localizedTitle: string,
  localizedBlurb: string,
): boolean {
  const needle = normalizeCatalogText(query.trim());
  if (!needle) return true;
  return [item.verb, item.title, item.blurb, localizedTitle, localizedBlurb]
    .some((value) => normalizeCatalogText(value).includes(needle));
}

/** Match both canonical codex data and everything the current locale displays. */
export function experimentMatches(
  entry: Pick<CodexEntry, "id" | "equation" | "summary" | "concepts" | "apparatus" | "models" | "registers">,
  query: string,
  localize: (value: string) => string,
): boolean {
  const needle = normalizeCatalogText(query.trim());
  if (!needle) return true;
  const localized = (value: string) => localize(value.replace(/-/g, " "));
  const values = [
    entry.id,
    entry.id.replace(/-/g, " "),
    entry.equation ?? "",
    entry.summary ?? "",
    ...(entry.concepts ?? []),
    ...(entry.apparatus ?? []),
    ...(entry.models ?? []),
    ...Object.values(entry.registers ?? {}),
  ];
  return values.some((value) =>
    normalizeCatalogText(value).includes(needle)
    || normalizeCatalogText(localized(value)).includes(needle)
  );
}
