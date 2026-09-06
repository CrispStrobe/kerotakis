/**
 * The quick-access row: what this learner reached for last.
 *
 * The `MESSEN` strip listed all twelve instruments in one non-wrapping row,
 * which on a phone put roughly half of them past the right edge — the
 * calorimeter and the Geiger counter were present in the DOM and absent from
 * the product. A row nobody can see the end of is not a tray, it is a
 * drawer, and the cupboard is the better drawer.
 *
 * So the strip keeps only six, chosen by use. The full list lives one tap
 * away and the row never scrolls. Ordering is most-recent-first because that
 * is the only ranking a learner can predict without being told the rule;
 * a frequency count would silently outrank the thing they just used.
 */

/** Six fits a 320 px row with names; more is the scroll this replaces. */
export const QUICK_ACCESS_SIZE = 6;

export const RECENT_INSTRUMENTS_KEY = "kerotakis.instruments.recent";

/**
 * The seed, for a learner who has measured nothing yet.
 *
 * Not the first six of `INSTRUMENTS` — that order is the catalogue's, and it
 * opens with the safe waft. These are the six measurements almost every
 * investigation begins with, so the untouched strip is already useful.
 */
export const DEFAULT_RECENT_INSTRUMENTS: readonly string[] = [
  "eyes",
  "thermometer",
  "ph",
  "balance",
  "volume",
  "conductivity",
];

const clean = (tokens: readonly string[], known: readonly string[]): string[] => {
  const seen = new Set<string>();
  const kept: string[] = [];
  for (const token of tokens) {
    if (!known.includes(token) || seen.has(token)) continue;
    seen.add(token);
    kept.push(token);
  }
  return kept;
};

/**
 * The row, padded from the seed and capped.
 *
 * A retired instrument token, a hand-edited save, or a duplicate all reduce
 * to "not one of these six" rather than to a broken button. Padding matters:
 * a learner who has used exactly one instrument should still see a full row.
 */
export function quickAccess(recent: readonly string[], known: readonly string[]): string[] {
  const kept = clean(recent, known);
  for (const token of DEFAULT_RECENT_INSTRUMENTS) {
    if (kept.length >= QUICK_ACCESS_SIZE) break;
    if (!kept.includes(token) && known.includes(token)) kept.push(token);
  }
  return kept.slice(0, QUICK_ACCESS_SIZE);
}

/**
 * The row as it is drawn: the same six, in the catalogue's fixed order.
 *
 * Membership is by recency; POSITION is not. If the row re-sorted on every
 * tap, measuring pH would slide the thermometer one place left and the next
 * tap would land on the wrong instrument — a row that rearranges itself
 * under the finger that is using it. So only entering and leaving the row
 * moves anything.
 */
export function quickAccessRow(recent: readonly string[], known: readonly string[]): string[] {
  const row = quickAccess(recent, known);
  return known.filter((token) => row.includes(token));
}

/** Most recent first; using something already in the row moves it, not adds it. */
export function rememberInstrument(
  recent: readonly string[],
  token: string,
  known: readonly string[],
): string[] {
  if (!known.includes(token)) return clean(recent, known).slice(0, QUICK_ACCESS_SIZE);
  return clean([token, ...recent], known).slice(0, QUICK_ACCESS_SIZE);
}

export interface RecentsStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}

/** Reads the stored order; anything unreadable is the seed, never a crash. */
export function loadRecentInstruments(
  storage: Pick<RecentsStorage, "getItem"> | null,
  key: string,
): string[] {
  if (!storage) return [];
  try {
    const value: unknown = JSON.parse(storage.getItem(key) ?? "null");
    if (!Array.isArray(value)) return [];
    return value.filter((item): item is string => typeof item === "string");
  } catch {
    return [];
  }
}

export function saveRecentInstruments(
  storage: Pick<RecentsStorage, "setItem"> | null,
  key: string,
  tokens: readonly string[],
): void {
  if (!storage) return;
  try {
    storage.setItem(key, JSON.stringify(tokens));
  } catch {
    // The row still works for this visit when persistence is unavailable.
  }
}
