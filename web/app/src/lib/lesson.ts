/**
 * Lessons are .lab scripts read as a guided walk: comment lines are the
 * narration, command lines are the steps the learner performs (or watches
 * run). The same file the CLI replays byte-for-byte drives the player —
 * lessons have exactly one source of truth (GUI-020).
 */

export type LessonStep =
  | { kind: "note"; text: string }
  | { kind: "command"; line: string };

export interface Lesson {
  name: string;
  steps: LessonStep[];
}

export function parseLesson(name: string, text: string): Lesson {
  const steps: LessonStep[] = [];
  for (const raw of text.split("\n")) {
    const line = raw.trim();
    if (!line) continue;
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
