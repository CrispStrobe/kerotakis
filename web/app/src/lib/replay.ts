/**
 * GUI-064: the playback scheduler — multi-step operations replayed over
 * real time instead of jumping to their result.
 *
 * The engine computes a whole titration (or distillation, or transport
 * train) synchronously and returns per-step data; this module paces the
 * REVEAL of that data: one callback per recorded point, evenly spaced,
 * cancellable, and honest — the data is complete before the first tick,
 * nothing is invented, and reduced-motion (or cancel) jumps straight to
 * the end state.
 *
 * Injectable clock so every behaviour is testable without waiting.
 */

export interface Playback {
  /** Jump to the end now: onPoint for every remaining index, then done. */
  finish(): void;
  /** Abandon: no further callbacks at all (not even done). */
  cancel(): void;
  /** True once done or cancelled. */
  readonly settled: boolean;
}

export interface Clock {
  setTimeout(fn: () => void, ms: number): unknown;
  clearTimeout(id: unknown): void;
}

const REAL_CLOCK: Clock = {
  setTimeout: (fn, ms) => setTimeout(fn, ms),
  clearTimeout: (id) => clearTimeout(id as ReturnType<typeof setTimeout>),
};

/**
 * Schedule `count` points at `msPerPoint` intervals. `onPoint(i)` fires
 * for i = 0..count-1 in order; `onDone` after the last. The first point
 * fires after one interval (the burette drips, THEN the reading moves).
 *
 * Total playback time is clamped to `maxMs` (default 6 s) — a 200-point
 * curve compresses its pacing rather than boring the bench to death.
 */
export function schedule(
  count: number,
  msPerPoint: number,
  onPoint: (i: number) => void,
  onDone: () => void,
  opts: { clock?: Clock; maxMs?: number; reducedMotion?: boolean } = {},
): Playback {
  const clock = opts.clock ?? REAL_CLOCK;
  const maxMs = opts.maxMs ?? 6000;
  const pace = count > 0 ? Math.min(msPerPoint, maxMs / count) : msPerPoint;

  let next = 0;
  let settled = false;
  let timer: unknown = null;

  const finishNow = () => {
    if (settled) return;
    settled = true;
    if (timer !== null) clock.clearTimeout(timer);
    for (; next < count; next++) onPoint(next);
    onDone();
  };

  if (opts.reducedMotion || count === 0) {
    // Straight to the settled truth — same callbacks, no waiting.
    settled = true;
    for (; next < count; next++) onPoint(next);
    onDone();
    return { finish: () => {}, cancel: () => {}, settled: true };
  }

  const tick = () => {
    if (settled) return;
    onPoint(next);
    next += 1;
    if (next >= count) {
      settled = true;
      timer = null;
      onDone();
      return;
    }
    timer = clock.setTimeout(tick, pace);
  };
  timer = clock.setTimeout(tick, pace);

  return {
    finish: finishNow,
    cancel: () => {
      if (settled) return;
      settled = true;
      if (timer !== null) clock.clearTimeout(timer);
    },
    get settled() {
      return settled;
    },
  };
}
