import type { FeedEntry } from "./session.svelte";

/**
 * Repeated clock ticks, collapsed (E4).
 *
 * A live clock submits `wait 1s` over and over. When a tick's journal block
 * (its rendered lines, notes and hazards) says the same thing as the tick
 * before it — numbers aside, since a temperature or an amount drifts every
 * second — the journal keeps ONE copy showing the latest values plus a
 * single "↻ same as previous tick ×N" marker instead of N identical blocks.
 * Different event kinds or different wording never collapse.
 *
 * Hazards: an identical hazard is shown once per run of adjacent ticks,
 * even when the ticks' other lines differ.
 *
 * Pure: it mutates only the array it is handed, so a test can drive it.
 */

/** A run of adjacent tick blocks currently shown as one. */
export type TickRun = {
  /** Feed index where the shown block starts. */
  start: number;
  /** Feed index one past the shown block (and its marker). */
  end: number;
  signature: string;
  /** Hazards already shown in this run of ticks. */
  hazards: string[];
  /** Ticks represented by the shown block. */
  count: number;
};

const NUMBER = /[-+−]?\d+(?:[.,]\d+)*(?:[eE][-+]?\d+)?/g;

/** Replace every number with `#`, so drifting values compare equal. */
export function normaliseNumbers(text: string): string {
  return text.replace(NUMBER, "#").replace(/\s+/g, " ").trim();
}

/** Whether a canonical command is a clock tick. */
export function isTickCommand(command: string): boolean {
  return /^wait\b/i.test(command.trim());
}

const hazardKey = (entry: FeedEntry): string =>
  `${entry.severity ?? ""}|${normaliseNumbers(entry.hazardText ?? entry.text)}`;

/** The comparison key of one tick's block (the echoed command included). */
export function tickSignature(block: readonly FeedEntry[]): string {
  return block
    .filter((entry) => entry.repeat === undefined)
    .map((entry) => `${entry.kind}:${normaliseNumbers(entry.text)}`)
    .join("\n");
}

/**
 * Fold the tick block `feed[start..]` (just pushed for `command`) into the
 * previous run when it repeats it. Returns the run to remember for the next
 * tick, or `null` when `command` is not a tick.
 */
export function collapseTick(
  feed: FeedEntry[],
  start: number,
  command: string,
  previous: TickRun | null,
  marker: (count: number) => FeedEntry,
): TickRun | null {
  if (!isTickCommand(command) || start > feed.length) return null;
  const adjacent = previous !== null && previous.end === start;
  const signature = tickSignature(feed.slice(start));
  if (adjacent && signature === previous.signature) {
    const count = previous.count + 1;
    const latest = feed.slice(start);
    feed.splice(previous.start, feed.length - previous.start, ...latest, { ...marker(count), repeat: count });
    return { ...previous, end: feed.length, count };
  }
  const seen = adjacent ? [...previous.hazards] : [];
  for (let i = feed.length - 1; i >= start; i -= 1) {
    const entry = feed[i]!;
    if (entry.kind === "hazard" && seen.includes(hazardKey(entry))) feed.splice(i, 1);
  }
  const hazards = [
    ...seen,
    ...feed.slice(start).filter((e) => e.kind === "hazard").map(hazardKey),
  ];
  return { start, end: feed.length, signature, hazards: [...new Set(hazards)], count: 1 };
}
