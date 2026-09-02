import { describe, expect, it } from "vitest";
import {
  CONTAMINATED_SAMPLE_AWARD,
  caseAwardDetail,
  caseAwardedTools,
  contaminatedSampleComplete,
  contaminatedSampleLeads,
  contaminatedSampleProgress,
} from "./storyChapter";

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
    // All four leads are solver-assessed since the typed-leads slice —
    // a lead falling OFF this list means its contract went missing.
    expect(leads.filter((lead) => lead.outcomeAssessed).map((lead) => lead.id)).toEqual([
      "silver-and-salt",
      "first-warmth",
      "one-thing-at-a-time",
      "never-mix",
    ]);
  });

  it("completes the case from core evidence without requiring the optional safety lead", () => {
    const completed = new Set(["silver-and-salt", "first-warmth", "one-thing-at-a-time"]);
    const progress = contaminatedSampleProgress(contaminatedSampleLeads(missions, completed));
    expect(progress).toEqual({ done: 3, total: 3, complete: true });
  });
});

describe("the case-level award (GUI-080)", () => {
  const core = ["silver-and-salt", "first-warmth", "one-thing-at-a-time"];

  it("grants the instrument exactly when all three core leads are secured", () => {
    expect(caseAwardedTools(new Set())).toEqual([]);
    expect(caseAwardedTools(new Set(core.slice(0, 2)))).toEqual([]);
    expect(caseAwardedTools(new Set(core))).toEqual([CONTAMINATED_SAMPLE_AWARD]);
  });

  it("does not require the optional safety audit, and is not granted by it alone", () => {
    expect(contaminatedSampleComplete(new Set(core))).toBe(true);
    expect(contaminatedSampleComplete(new Set([...core.slice(0, 2), "never-mix"]))).toBe(false);
  });

  it("is derived, so asking twice cannot grant twice", () => {
    // The property that makes a rewards ledger unnecessary: the answer is a
    // function of the leads, so a retried commit and a replayed mission
    // return the same single award rather than accumulating one each.
    const completed = new Set(core);
    expect(caseAwardedTools(completed)).toEqual(caseAwardedTools(completed));
    expect(caseAwardedTools(new Set([...core, "never-mix", "fizz"]))).toEqual([CONTAMINATED_SAMPLE_AWARD]);
  });

  it("names the award for the debrief and the instrument wall", () => {
    expect(caseAwardDetail(CONTAMINATED_SAMPLE_AWARD)?.title).toBe("UV/Vis spectrophotometer");
    expect(caseAwardDetail("nothing-earned-this")).toBeNull();
  });
});
