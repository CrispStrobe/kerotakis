import { describe, expect, it } from "vitest";
import { notebookMarkdown } from "./notebook";
import type { FeedEntry } from "./session.svelte";

describe("notebook export", () => {
  it("renders each entry kind in its Markdown form", () => {
    const feed: FeedEntry[] = [
      { kind: "command", text: "add v1 water 100mL" },
      { kind: "line", text: "You add water to v1." },
      { kind: "note", text: "speaking at lv2" },
      { kind: "user-note", text: "The precipitate settled slowly.", createdAt: "2026-08-27T08:00:00.000Z" },
      { kind: "hazard", severity: "danger", text: "chloramine — do not mix at home" },
      { kind: "error", text: "no such species" },
      {
        kind: "chart",
        text: "titration",
        chart: {
          title: "titration",
          x: { label: "volume", unit: "mL" },
          y: { label: "pH" },
          series: [
            {
              kind: "line",
              name: "pH",
              points: [
                [0, 1],
                [25, 7],
              ],
            },
          ],
          provenance: "PHREEQC",
        },
      },
    ];
    const md = notebookMarkdown(feed, { date: "2026-08-24", register: "lv2" });
    expect(md).toContain("# Kerotakis lab notebook");
    expect(md).toContain("2026-08-24 · register lv2");
    expect(md).toContain("kero> add v1 water 100mL");
    expect(md).toContain("You add water to v1.");
    expect(md).toContain("*speaking at lv2*");
    expect(md).toContain("**my note · 2026-08-27T08:00:00.000Z**");
    expect(md).toContain("The precipitate settled slowly.");
    expect(md).toContain("> **danger** — chloramine");
    expect(md).toContain("> no such species");
    expect(md).toContain("### titration");
    expect(md).toContain("| volume (mL) | pH |");
    expect(md).toContain("| 25 | 7 |");
    expect(md).toContain("*PHREEQC*");
    expect(md).not.toMatch(/\n{3,}/);
  });
});
