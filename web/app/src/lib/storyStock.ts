import type { ShelfItem } from "./session.svelte";

export const STORY_STOCK_KEY = "kero.story-stock.v1";

export interface StockStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}

/** Stock is counted in labelled dispenses, not invented mass conversions.
 * Every dispense still carries its real engine-parsed amount in the notebook. */
export function stockCapacity(item: Pick<ShelfItem, "hazards" | "hazard_assessed">): number {
  if (item.hazard_assessed === false) return 3;
  const hazards = new Set(item.hazards ?? []);
  if (hazards.has("toxic") || hazards.has("corrosive")) return 4;
  if (hazards.has("flammable") || hazards.has("oxidizer")) return 6;
  return 10;
}

export function stockRemaining(
  item: Pick<ShelfItem, "key" | "hazards" | "hazard_assessed">,
  used: Readonly<Record<string, number>>,
): number {
  return Math.max(0, stockCapacity(item) - Math.max(0, Math.floor(used[item.key] ?? 0)));
}

/** Commands that physically draw a named substance from the cabinet. */
export function suppliedSpecies(line: string): string | null {
  const words = line.trim().split(/\s+/);
  if ((words[0] === "add" || words[0] === "titrate") && words.length >= 4) return words[2] ?? null;
  return null;
}

export function restoreStockUsed(storage: StockStorage | null): Record<string, number> {
  try {
    const parsed = JSON.parse(storage?.getItem(STORY_STOCK_KEY) ?? "{}");
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) return {};
    return Object.fromEntries(
      Object.entries(parsed).filter((entry): entry is [string, number] =>
        typeof entry[1] === "number" && Number.isFinite(entry[1]) && entry[1] >= 0,
      ).map(([key, value]) => [key, Math.floor(value)]),
    );
  } catch {
    return {};
  }
}

export function persistStockUsed(storage: StockStorage | null, used: Readonly<Record<string, number>>): void {
  try {
    storage?.setItem(STORY_STOCK_KEY, JSON.stringify(used));
  } catch {
    // A storage-blocked visit keeps its in-memory stock ledger.
  }
}
