/**
 * Lessons are .lab scripts read as a guided walk: comment lines are the
 * narration, command lines are the steps the learner performs (or watches
 * run). The same file the CLI replays byte-for-byte drives the player —
 * lessons have exactly one source of truth (GUI-020).
 */

export type LessonStep =
  | {
      kind: "note";
      /** The English the `.lab` carries, and the fallback when nothing
       * translated it. */
      text: string;
      /** `<lesson-stem>.<label>` when the comment is labelled (I18N-9).
       * Absent for a lesson not yet migrated, which renders as before. */
      key?: string;
    }
  | { kind: "command"; line: string };

export interface Lesson {
  name: string;
  steps: LessonStep[];
}

/**
 * `#@label text` — a comment that names the place its prose is said.
 *
 * Mirrors `tools/lesson_prose.py`, which is what the payload's prose
 * tables are keyed by; `tools/lesson-prose-lint.py` fails if the two
 * disagree about which lines a label owns.
 */
const LABELLED = /^#@([a-z0-9][a-z0-9._-]*)[ \t]+(\S.*?)[ \t]*$/;
/** A plain comment carrying text: a continuation while a label is open. */
const CONTINUATION = /^#[ \t]+(\S.*?)[ \t]*$/;

export function parseLesson(name: string, text: string): Lesson {
  const steps: LessonStep[] = [];
  // The translatable unit is the PARAGRAPH. A `.lab` wraps its prose at 78
  // columns for the terminal, and that wrap is typography — German rebuilds
  // the sentence with the verb somewhere else entirely, so a labelled
  // comment owns the plain comment lines under it and renders as one note.
  let open: { kind: "note"; text: string; key?: string } | null = null;
  for (const raw of text.split("\n")) {
    const line = raw.trim();
    if (!line) {
      open = null;
      continue;
    }
    const labelled = LABELLED.exec(line);
    if (labelled) {
      open = { kind: "note", text: labelled[2]!, key: `${name}.${labelled[1]!}` };
      steps.push(open);
      continue;
    }
    if (open && CONTINUATION.test(line)) {
      open.text = `${open.text} ${CONTINUATION.exec(line)![1]!}`;
      continue;
    }
    open = null;
    if (line.startsWith("#")) {
      steps.push({ kind: "note", text: line.replace(/^#\s?/, "") });
    } else {
      steps.push({ kind: "command", line });
    }
  }
  return { name, steps };
}

/** How many of the lesson's steps are commands (for progress display). */
export function commandCount(lesson: Lesson): number {
  return lesson.steps.filter((s) => s.kind === "command").length;
}

/** Commands completed before the mixed narration/command cursor. */
export function completedCommandCount(lesson: Lesson, cursor: number): number {
  return lesson.steps.slice(0, cursor).filter((step) => step.kind === "command").length;
}
