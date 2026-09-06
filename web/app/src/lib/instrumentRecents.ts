/**
 * The quick-access row: what this learner reached for last.
 *
 * The `MESSEN` strip listed all twelve instruments in one non-wrapping row,
 * which on a phone put roughly half of them past the right edge — the
 * calorimeter and the Geiger counter were present in the DOM and absent from
 * the product. A row nobody can see the end of is not a tray, it is a
 * drawer, and the cupboard is the better drawer.
 *
 * So the strip keeps only the few this learner reached for, chosen by use.
 * The full list lives one tap away and the row never scrolls. Ordering is
 * most-recent-first because that is the only ranking a learner can predict
 * without being told the rule; a frequency count would silently outrank the
 * thing they just used.
 *
 * GUI-103 narrowed it twice. Four rather than six, because the row also
 * carries the cupboard door and a fifth pill is what pushes that door off a
 * 320 px screen — and the door is the thing the row exists to lead to. And
 * the three readings the vessel dock already carries (`DOCK_INSTRUMENTS`)
 * are not candidates at all: offering `look`, `thermometer` and `pH` twice
 * on one screen spent three of six slots restating buttons that never move,
 * which is exactly the duplication the one-surface work removed everywhere
 * else.
 */
import { DOCK_INSTRUMENTS } from "./directActions";

/**
 * Four pills and the cupboard door fit a 320 px row; a fifth does not.
 *
 * The row was six while it held the dock's three as well. With those gone
 * the remaining four are all instruments the dock cannot reach, so the row
 * is shorter AND carries strictly more than it did.
 */
export const QUICK_ACCESS_SIZE = 4;

export const RECENT_INSTRUMENTS_KEY = "kerotakis.instruments.recent";

/**
 * The seed, for a learner who has measured nothing yet.
 *
 * Not the first four of `INSTRUMENTS` — that order is the catalogue's, and
 * it opens with the safe waft. These are the four commonest measurements
 * that are NOT already on the vessel dock, so the untouched strip is useful
 * and repeats nothing on the screen beside it.
 */
export const DEFAULT_RECENT_INSTRUMENTS: readonly string[] = [
  "balance",
  "volume",
  "conductivity",
  "pressure",
];

/**
 * The tokens the strip may hold: everything the dock does not already carry.
 *
 * Applied to the KNOWN list rather than to the stored history, so a token
 * that becomes a dock landmark later leaves the row on the next render
 * instead of persisting as a duplicate button.
 */
export const quickAccessCandidates = (known: readonly string[]): string[] =>
  known.filter((token) => !DOCK_INSTRUMENTS.includes(token));

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
  const candidates = quickAccessCandidates(known);
  const kept = clean(recent, candidates);
  for (const token of DEFAULT_RECENT_INSTRUMENTS) {
    if (kept.length >= QUICK_ACCESS_SIZE) break;
    if (!kept.includes(token) && candidates.includes(token)) kept.push(token);
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
  return quickAccessCandidates(known).filter((token) => row.includes(token));
}

/**
 * Most recent first; using something already in the row moves it, not adds it.
 *
 * A dock instrument measured from the cupboard is remembered by nobody: it
 * is not a candidate, so it neither enters the row nor evicts what is in it.
 */
export function rememberInstrument(
  recent: readonly string[],
  token: string,
  known: readonly string[],
): string[] {
  const candidates = quickAccessCandidates(known);
  if (!candidates.includes(token)) return clean(recent, candidates).slice(0, QUICK_ACCESS_SIZE);
  return clean([token, ...recent], candidates).slice(0, QUICK_ACCESS_SIZE);
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
