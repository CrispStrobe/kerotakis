/**
 * One runner for both catalogue tiers.
 *
 * The experiment catalogue used to "run" an entry by firing every line of
 * its setup script at the bench in a tight loop while a full-screen modal
 * sat on top of the stage. The chemistry was real — `submit()` was always
 * the path — but nobody could SEE it: no pacing, so the animations of ten
 * commands collapsed into one frame, and the scrim hid the frame anyway.
 * The learner got a verdict and no experiment. That is the whole bug the
 * owner reported, and it is a presentation bug, not a chemistry one.
 *
 * So the loop moved out of the component and grew the two things it was
 * missing: a per-step callback (which the panel uses to name the line it is
 * about to run, and to get out of the way while it runs) and a pause
 * between steps (which is what makes a reaction visible). The Kids tier
 * drives the same function, because "runs on the bench" should not mean two
 * different things depending on which door the learner came through.
 *
 * The pause was still a timer, though, and a timer is a guess about how
 * long the learner needs. So the walk grew a second pacer: hand the runner
 * an `onstepdone` gate and it submits ONE line, reports what that line
 * produced, and waits to be told to go on. Same door, same checks, same
 * recording — the only difference is who says "next". The one thing that
 * had to change underneath is what counts as done: the expectations are
 * checked against the bench whatever happened, and a bench can satisfy
 * them after one line, so a run the learner stopped is no longer allowed
 * to record progress it did not earn.
 *
 * Everything here is pure or takes its bench as a structural interface, so
 * the whole flow is testable without a DOM: `Session` satisfies
 * `RunnerBench` by shape, and a fake satisfies it in the tests.
 */
import { checkExpect, type CheckResult, type CodexExpect } from "./codex";

/** What the runner needs of a bench. `Session` satisfies this by shape. */
export interface RunnerBench {
  readonly scene: { vessels?: readonly BenchVesselLike[] } | null;
  readonly completedExperiments: ReadonlySet<string>;
  submit(line: string): Promise<boolean>;
  clear(): Promise<void>;
  beginEventCapture(): void;
  endEventCapture(): string[];
  finalStateForCheck(): { phValues: number[]; temperaturesC: number[] };
  markExperimentDone(id: string): void;
  /**
   * The learner-visible feed, oldest first.
   *
   * Step mode has to answer "what did that line just DO?", and the bench
   * already writes that answer down: the feed is the same account the
   * learner reads when they type a command themselves. Reading its tail is
   * therefore not a second rendering of the run, it is the run's own words.
   * Optional, because a bench without a feed is still a runnable bench —
   * it simply reports nothing produced.
   */
  readonly feed?: readonly RunnerFeedLine[];
}

/** One line of the feed, as a step report carries it. */
export interface RunnerFeedLine {
  kind: string;
  text: string;
}

/** Only the fields that answer "is there work on this bench?". */
export interface BenchVesselLike {
  id?: number;
  liquid?: unknown;
  solids?: readonly unknown[];
  bulk_objects?: readonly unknown[];
  material_objects?: readonly unknown[];
}

export interface RunnableEntry {
  id: string;
  setup: { script: string };
  expect?: CodexExpect;
}

/** The learner's answer to "your bench is not empty". */
export type BenchDecision = "clear" | "fresh" | "keep";

export type RunGate = "ready" | "ask";

export interface RunStep {
  line: string;
  /** Zero-based position among the lines the runner will submit. */
  index: number;
  total: number;
}

/** What the learner asked for after watching one step land. */
export type StepVerdict = "next" | "rest" | "stop";

/** A step that has already happened, and what the bench said about it. */
export interface StepReport {
  line: string;
  index: number;
  total: number;
  /** The feed lines this line wrote, in order. */
  produced: RunnerFeedLine[];
  /** True while lines remain after this one. */
  more: boolean;
}

/** How the learner wants the script walked. Remembered between runs. */
export type RunMode = "auto" | "step";

const RUN_MODE_KEY = "kerotakis.catalogRunMode";

/**
 * The learner's remembered choice of pace, defaulting to "auto".
 *
 * Storage is absent in private modes and in the test environment, and a
 * missing preference is not an error — it is the default. So every path
 * that cannot read a choice returns the same thing an untouched browser
 * would.
 */
export function loadRunMode(): RunMode {
  if (typeof window === "undefined") return "auto";
  try {
    return window.localStorage.getItem(RUN_MODE_KEY) === "step" ? "step" : "auto";
  } catch {
    return "auto";
  }
}

/** Remember the pace. A blocked store costs the memory, never the run. */
export function saveRunMode(mode: RunMode): void {
  if (typeof window === "undefined") return;
  try {
    window.localStorage.setItem(RUN_MODE_KEY, mode);
  } catch {
    /* The live choice still works when persistence is blocked. */
  }
}

export interface CatalogRunOutcome {
  /** Lines actually submitted, in order. */
  ran: string[];
  observed: string[];
  result: CheckResult;
  /** Index of the line the engine refused, or null when the script ran out. */
  refusedAt: number | null;
  /** True only on the run that first records this entry as completed. */
  recorded: boolean;
  /** True when the learner ended the run before the script ran out. */
  halted: boolean;
}

/** Lines the runner will submit: comments and blanks are not steps. */
export function runnableLines(script: string): string[] {
  return script
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.length > 0 && !line.startsWith("#"));
}

/**
 * Does the bench hold work the learner would mind losing?
 *
 * An empty vessel is not work — a fresh bench ships with one — so the
 * question is about contents, not about vessel count. Unknown scene shapes
 * (an older payload, a scene that has not arrived) read as empty, because
 * the cost of a wrong "empty" here is one extra `clear` on a bench with
 * nothing on it, and the cost of a wrong "occupied" is a prompt in front of
 * every single run.
 */
export function benchOccupied(scene: { vessels?: readonly BenchVesselLike[] } | null | undefined): boolean {
  return (scene?.vessels ?? []).some((vessel) =>
    (vessel.liquid ?? null) !== null
    || (vessel.solids?.length ?? 0) > 0
    || (vessel.bulk_objects?.length ?? 0) > 0
    || (vessel.material_objects?.length ?? 0) > 0,
  );
}

/**
 * Whether the panel must ask before running.
 *
 * A decision already taken is honoured; an empty bench never asks. Nothing
 * here wipes anything — the decision is the learner's, and `runCatalogEntry`
 * only clears when it is handed `"clear"`.
 */
export function runGate(
  scene: { vessels?: readonly BenchVesselLike[] } | null | undefined,
  decision: BenchDecision | null,
): RunGate {
  return decision === null && benchOccupied(scene) ? "ask" : "ready";
}

/** The highest vessel NUMBER as the grammar spells it (`v3` → 3). */
export function highestVesselNumber(scene: { vessels?: readonly BenchVesselLike[] } | null | undefined): number {
  return (scene?.vessels ?? []).reduce((top, vessel) => Math.max(top, (vessel.id ?? 0) + 1), 0);
}

/**
 * Can this script be moved onto fresh glassware?
 *
 * The shift below renumbers `vN` tokens and prepends one `new` per vessel
 * the script uses, which is only sound while the script's own vessels are
 * exactly the ones it names. A script that allocates glassware itself
 * would have its `new` counted twice and land in the wrong beaker, so that
 * shape is refused the option rather than served a subtly wrong run.
 */
export function canUseFreshVessels(script: string): boolean {
  const lines = runnableLines(script);
  if (lines.some((line) => /^new\b/i.test(line))) return false;
  return scriptVesselNumbers(script).length > 0;
}

/** Distinct `vN` numbers a script names, ascending. */
export function scriptVesselNumbers(script: string): number[] {
  const seen = new Set<number>();
  for (const match of script.matchAll(/\bv(\d+)\b/g)) seen.add(Number(match[1]));
  return [...seen].sort((a, b) => a - b);
}

/**
 * The same script, run on glassware that does not exist yet.
 *
 * Every `vN` moves up by `offset` and the script opens by asking for one
 * new vessel per vessel it uses. `new` allocates the next free number, so
 * with `offset` set to the highest number currently on the bench the
 * prepended vessels are exactly the ones the shifted body then addresses.
 */
export function freshVesselScript(script: string, offset: number): string {
  if (offset <= 0) return script;
  const prelude = scriptVesselNumbers(script).map(() => "new");
  const body = script.replace(/\bv(\d+)\b/g, (_, digits: string) => `v${Number(digits) + offset}`);
  return [...prelude, body].join("\n");
}

/** The script one decision produces, given the bench it will land on. */
export function scriptForDecision(
  script: string,
  decision: BenchDecision | null,
  scene: { vessels?: readonly BenchVesselLike[] } | null | undefined,
): string {
  if (decision !== "fresh") return script;
  return freshVesselScript(script, highestVesselNumber(scene));
}

export interface RunOptions {
  /** Announced BEFORE the line is submitted, so a panel can name it. */
  onstep?: (step: RunStep) => void;
  /** Between steps, so the stage has time to show what just happened. */
  pause?: (ms: number) => Promise<void>;
  paceMs?: number;
  decision?: BenchDecision | null;
  /** Polled between steps: a learner who taps stop is obeyed. */
  stopped?: () => boolean;
  /**
   * Awaited AFTER each line, which is what makes a run manual.
   *
   * Its presence is the mode: with it the runner submits one line and then
   * hands the learner what that line produced, and waits. "next" runs one
   * more, "rest" finishes the script at the automatic pace without asking
   * again, "stop" ends the run here. It is never called after the last
   * line — there is nothing left to consent to, and the verdict is the
   * answer to that step.
   */
  onstepdone?: (report: StepReport) => Promise<StepVerdict>;
}

/** The feed a step wrote, minus the echo of the command that wrote it. */
function producedSince(bench: RunnerBench, from: number): RunnerFeedLine[] {
  return (bench.feed ?? [])
    .slice(from)
    // `command` is the line the learner can already read on the button they
    // pressed, and `user-note` is their own writing, not the bench's answer.
    .filter((entry) => entry.kind !== "command" && entry.kind !== "user-note")
    .map((entry) => ({ kind: entry.kind, text: entry.text }));
}

const wait = (ms: number): Promise<void> =>
  new Promise((resolve) => setTimeout(resolve, ms));

/**
 * Run one catalogue entry on the VISIBLE bench, step by step.
 *
 * Every line goes through `submit()` — the same door a typed command uses —
 * so the feed, the log, undo and the stage all behave exactly as they do
 * when the learner drives. The expectations are then checked against the
 * scene that actually resulted, never against a scratch replay.
 */
export async function runCatalogEntry(
  bench: RunnerBench,
  entry: RunnableEntry,
  options: RunOptions = {},
): Promise<CatalogRunOutcome> {
  const { onstep, pause = wait, paceMs = 420, decision = null, stopped, onstepdone } = options;
  if (decision === "clear") await bench.clear();
  const script = scriptForDecision(entry.setup.script, decision, bench.scene);
  const lines = runnableLines(script);

  const ran: string[] = [];
  let refusedAt: number | null = null;
  let halted = false;
  // Manual until the learner says otherwise: "finish the rest" is a
  // one-way door within a run, and the next run reads the mode afresh.
  let manual = onstepdone != null;
  bench.beginEventCapture();
  let observed: string[] = [];
  try {
    for (const [index, line] of lines.entries()) {
      if (stopped?.()) {
        halted = true;
        break;
      }
      onstep?.({ line, index, total: lines.length });
      const feedBefore = bench.feed?.length ?? 0;
      const accepted = await bench.submit(line);
      ran.push(line);
      if (!accepted) {
        refusedAt = index;
        break;
      }
      const more = index < lines.length - 1;
      if (onstepdone && manual && more) {
        const verdict = await onstepdone({
          line,
          index,
          total: lines.length,
          produced: producedSince(bench, feedBefore),
          more,
        });
        if (verdict === "stop") {
          halted = true;
          break;
        }
        // A learner who has seen the step does not need the pace that
        // exists to make an unwatched one visible.
        if (verdict === "rest") manual = false;
        continue;
      }
      if (more) await pause(paceMs);
    }
  } finally {
    observed = bench.endEventCapture();
  }

  const result = checkExpect(entry.expect ?? {}, observed, bench.finalStateForCheck());
  // Completion is recorded once, and only for a script that actually ran
  // out. A run the learner stopped after one lucky line can satisfy the
  // expectations by accident; crediting it would be crediting the check,
  // not the experiment. Asking whether it is already done keeps the second
  // green run from re-writing progress that is already saved, which is
  // what makes "recorded" answerable at all.
  const walked = refusedAt === null && !halted && ran.length === lines.length;
  const recorded = result.allOk && walked && !bench.completedExperiments.has(entry.id);
  if (recorded) bench.markExperimentDone(entry.id);
  return { ran, observed, result, refusedAt, recorded, halted };
}
