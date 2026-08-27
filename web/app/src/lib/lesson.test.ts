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
