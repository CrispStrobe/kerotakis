import type { FeedEntry } from "./session.svelte";

/**
 * What the journal shows, as a function rather than as markup.
 *
 * This lived inside `Feed.svelte` until a header rebuild emptied the log
 * without any test noticing: the header had started rendering the session's
 * STATUS notes as icons and, to avoid saying the same thing twice, the
 * filter dropped every entry carrying a `status`. On a freshly loaded or
 * restored bench those two notes are the ONLY entries there are, so the
 * logbook opened completely blank — four grey glyphs and nothing under
 * them. Nothing in the suite could see it, because the rule that decided
 * it was a `$derived` inside a component no test could render.
 *
 * So it is here, where a test can ask it directly, in every toggle state,
 * with a realistic feed.
 */

/**
 * Render only the tail of a very long session: the exports keep every entry,
 * the DOM does not have to (low-end budget). 400 entries is far beyond what
 * a screen shows and well within what a Chromebook lays out.
 */
export const JOURNAL_WINDOW = 400;

/**
 * The session notes that are STATUS rather than observation: whether the
 * solver is attached, and whether a save came back.
 *
 * They keep an icon so they read as the session's own bookkeeping rather
 * than as chemistry — but they stay IN the log, because they are the first
 * two lines of the record and, for a bench that has not run a command yet,
 * the only ones. The notebook export writes them out for the same reason.
 */
export const STATUS_ICONS: Record<string, string> = {
  "bench-live": "◉",
  "bench-shipped": "◌",
  restored: "⟳",
  "restore-failed": "⚠",
};

export const statusIcon = (entry: FeedEntry): string | undefined =>
  entry.status ? STATUS_ICONS[entry.status] : undefined;

/** The vessel an entry is about, where its text names one. */
export function entryVessel(entry: FeedEntry): number | null {
  if (!["command", "line", "error", "refusal"].includes(entry.kind)) return null;
  const match = entry.text.match(entry.kind === "command" ? /\bv(\d+)\b/i : /^\s*v(\d+)\s*:/i);
  return match ? Number(match[1]) - 1 : null;
}

/** The chip carries the vessel, so the line does not repeat it. */
export function displayText(entry: FeedEntry): string {
  return entry.kind === "line" ? entry.text.replace(/^\s*v\d+\s*:\s*/i, "") : entry.text;
}

/**
 * The entries the journal draws.
 *
 * ONE rule, and it is the view toggle's: the typed commands are the trace,
 * and "observations" hides them. Nothing else is ever dropped — no scope,
 * no status, no kind. A journal that hides entries by any rule the reader
 * did not ask for is a journal they cannot trust.
 */
export function journalEntries(
  entries: readonly FeedEntry[],
  options: { showTrace: boolean },
): FeedEntry[] {
  return entries.filter((entry) => options.showTrace || entry.kind !== "command");
}
