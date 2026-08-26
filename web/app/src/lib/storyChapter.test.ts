import { describe, expect, it } from "vitest";
import { contaminatedSampleLeads, contaminatedSampleProgress } from "./storyChapter";

const missions = [
  { file: "silver-and-salt.lab", name: "silver and salt" },
  { file: "first-warmth.lab", name: "first warmth" },
  { file: "one-thing-at-a-time.lab", name: "one thing at a time" },
  { file: "never-mix.lab", name: "never mix" },
  { file: "fizz.lab", name: "fizz" },
];

describe("contaminated sample chapter", () => {
  it("builds three concurrent core leads and one optional lead from shipped missions", () => {
    const leads = contaminatedSampleLeads(missions, new Set());
    expect(leads).toHaveLength(4);
    expect(leads.filter((lead) => !lead.optional)).toHaveLength(3);
    expect(leads.filter((lead) => lead.optional).map((lead) => lead.id)).toEqual(["never-mix"]);
  });

  it("completes the case from core evidence without requiring the optional safety lead", () => {
    const completed = new Set(["silver-and-salt", "first-warmth", "one-thing-at-a-time"]);
    const progress = contaminatedSampleProgress(contaminatedSampleLeads(missions, completed));
    expect(progress).toEqual({ done: 3, total: 3, complete: true });
  });
});
