import type { ShelfItem } from "./session.svelte";
import type { CodexEntry } from "./codex";

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
