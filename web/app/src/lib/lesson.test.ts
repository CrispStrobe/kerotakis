import { describe, expect, it } from "vitest";
import { commandCount, completedCommandCount, parseLesson } from "./lesson";

describe("lesson progress", () => {
  it("counts executable operators rather than narration", () => {
    const lesson = parseLesson("sample", "# Intro\nadd v1 water 1mL\n# Observe\nlook v1\n");
    expect(commandCount(lesson)).toBe(2);
    expect(completedCommandCount(lesson, 1)).toBe(0);
    expect(completedCommandCount(lesson, 3)).toBe(1);
    expect(completedCommandCount(lesson, lesson.steps.length)).toBe(2);
  });
});

describe("labelled lesson prose (I18N-9)", () => {
  it("keys a labelled comment by lesson and label", () => {
    const lesson = parseLesson("dry-then-wet-fizz", "#@title Dry, then wet\nadd v1 water 1mL\n");
    expect(lesson.steps[0]).toEqual({
      kind: "note",
      text: "Dry, then wet",
      key: "dry-then-wet-fizz.title",
    });
  });

  it("joins the comment lines under a label into one paragraph", () => {
    // The `.lab` wraps at 78 columns for the terminal. That wrap is
    // typography: German rebuilds the sentence with the verb somewhere
    // else, so the translatable unit has to be the whole paragraph.
    const lesson = parseLesson(
      "x",
      "#@intro Citric acid and baking soda can sit beside each other\n# while dry. Water lets ions move.\nadd v1 water 1mL\n",
    );
    expect(lesson.steps).toHaveLength(2);
    expect(lesson.steps[0]).toEqual({
      kind: "note",
      text: "Citric acid and baking soda can sit beside each other while dry. Water lets ions move.",
      key: "x.intro",
    });
  });

  it("closes a paragraph at a command, a blank line, or a bare hash", () => {
    const lesson = parseLesson("x", "#@a One\nadd v1 water 1mL\n# loose\n#@b Two\n#\n# not mine\n");
    expect(lesson.steps.map((s) => (s.kind === "note" ? [s.key, s.text] : s.line))).toEqual([
      ["x.a", "One"],
      "add v1 water 1mL",
      [undefined, "loose"],
      ["x.b", "Two"],
      [undefined, ""],
      [undefined, "not mine"],
    ]);
  });

  it("leaves an unmigrated lesson exactly as it was", () => {
    // 109 of 113 lessons are still unlabelled. They must parse to what
    // they parsed to before this existed: one note per comment LINE, no
    // key, translated through the interface dictionary as before.
    const lesson = parseLesson("x", "# One\n# Two\nadd v1 water 1mL\n");
    expect(lesson.steps).toEqual([
      { kind: "note", text: "One" },
      { kind: "note", text: "Two" },
      { kind: "command", line: "add v1 water 1mL" },
    ]);
  });

  it("does not mistake a hash-at for a label when it is malformed", () => {
    const lesson = parseLesson("x", "#@Not A Label\n#@ nor this\n");
    expect(lesson.steps).toEqual([
      { kind: "note", text: "@Not A Label" },
      { kind: "note", text: "@ nor this" },
    ]);
  });
});
