import type { ShelfItem } from "./session.svelte";

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
