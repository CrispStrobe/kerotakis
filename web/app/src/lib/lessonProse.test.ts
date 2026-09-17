import { beforeEach, describe, expect, it } from "vitest";
import { i18n } from "./i18n.svelte";
import { clearLessonProse, hasLessonProse, lessonProse, loadLessonProse } from "./lessonProse";

describe("lesson prose (I18N-9)", () => {
  beforeEach(() => {
    clearLessonProse();
    i18n.setLocale("en");
  });

  it("answers in the reader's language when the label is translated", () => {
    loadLessonProse({ de: { "dry-then-wet-fizz.title": "Trocken, dann nass" } });
    i18n.setLocale("de");
    expect(lessonProse("dry-then-wet-fizz.title", "Dry, then wet")).toBe("Trocken, dann nass");
  });

  it("falls back per key, so a half-translated lesson still ships", () => {
    loadLessonProse({ de: { "x.title": "Titel" } });
    i18n.setLocale("de");
    expect(lessonProse("x.title", "Title")).toBe("Titel");
    expect(lessonProse("x.boundary", "What this does not model")).toBe(
      "What this does not model",
    );
  });

  it("renders the English the .lab carries when nothing was fetched", () => {
    // A payload built before the file existed, or a host that 404s it.
    i18n.setLocale("de");
    expect(lessonProse("x.title", "Title")).toBe("Title");
    expect(lessonProse(undefined, "Title")).toBe("Title");
  });

  it("ignores rows that are not strings rather than rendering [object Object]", () => {
    loadLessonProse({ de: { "x.title": { nested: "no" } as unknown as string } });
    i18n.setLocale("de");
    expect(lessonProse("x.title", "Title")).toBe("Title");
    expect(hasLessonProse("x.title", "de")).toBe(false);
  });

  it("survives a payload that is not an object at all", () => {
    loadLessonProse("nope");
    expect(lessonProse("x.title", "Title")).toBe("Title");
  });
});
