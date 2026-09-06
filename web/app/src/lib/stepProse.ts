/**
 * One sentence per step, for the runner that walks a script one line at a
 * time.
 *
 * The step-by-step runner already answers "what did that line DO?" — it
 * quotes the bench's own feed. What it could not answer is "what should I
 * be watching for?", because no entry carried prose at that grain: a codex
 * entry has a script and the guided catalogue has one phenomenon for a
 * whole experiment. So the sentences are shipped as data beside the
 * catalogue (`data/steps/step-prose-*-v1.json`, exported to
 * `steps/index.json` beside the payload), keyed by the codex entry whose
 * script they pace. A guided experiment that runs a codex script inherits
 * that script's sentences, which is the point of keying it there: the
 * lesson door and the catalogue door read the same words.
 *
 * ALIGNMENT is the whole contract. Prose one step out of step does not
 * read as missing, it reads as wrong — it narrates the fizz while the
 * learner is still measuring water — so a row whose length does not match
 * the script is dropped here as well as refused by the exporter. Absence
 * is always fine: no prose is exactly the run that shipped before.
 */
import { runnableLines } from "./catalogRunner";

/** One script's sentences, English plus whatever languages shipped. */
export interface StepProse {
  say: string[];
  /** Positional twin of `say` per language code, as the export writes it. */
  [locale: string]: string[];
}

export type StepProseIndex = ReadonlyMap<string, StepProse>;

export const NO_STEP_PROSE: StepProseIndex = new Map<string, StepProse>();

const isSentences = (value: unknown): value is string[] =>
  Array.isArray(value) && value.length > 0 && value.every((s) => typeof s === "string" && s.length > 0);

/**
 * The shipped export, or nothing.
 *
 * Quiet about everything it does not recognise: a payload built before
 * this file existed simply has no `steps/index.json`, and a row the
 * exporter would have refused is skipped rather than allowed to narrate
 * the wrong line.
 */
export function parseStepProse(raw: unknown): Map<string, StepProse> {
  const found = new Map<string, StepProse>();
  if (!raw || typeof raw !== "object") return found;
  const document = raw as { schema?: unknown; scripts?: unknown };
  if (document.schema !== 1) return found;
  const rows = document.scripts;
  if (!rows || typeof rows !== "object") return found;
  for (const [id, value] of Object.entries(rows as Record<string, unknown>)) {
    if (!value || typeof value !== "object") continue;
    const row = value as Record<string, unknown>;
    if (!isSentences(row.say)) continue;
    const prose: StepProse = { say: row.say };
    for (const [key, sentences] of Object.entries(row)) {
      if (key === "say" || !key.startsWith("say_")) continue;
      // A translation of a different length cannot be positional, so it is
      // dropped and that language falls back to English per entry.
      if (isSentences(sentences) && sentences.length === prose.say.length) {
        prose[key.slice("say_".length)] = sentences;
      }
    }
    found.set(id, prose);
  }
  return found;
}

/**
 * The sentences for one script, in the reader's language, or null.
 *
 * Null when the entry has none, and null when the row does not match the
 * script it claims to pace — a payload and a catalogue can be built from
 * different commits, and a stale row must degrade to silence rather than
 * describe the previous version of the experiment.
 */
export function sayForScript(
  prose: StepProseIndex,
  id: string,
  script: string,
  locale: string,
): string[] | null {
  const row = prose.get(id);
  if (!row) return null;
  const localized = locale === "en" ? row.say : (row[locale] ?? row.say);
  return localized.length === runnableLines(script).length ? localized : null;
}
