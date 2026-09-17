import { describe, expect, it } from "vitest";
import { capabilityReasonText, capabilityRunnable, capabilitySearchText, parseCapabilityIndex } from "./capabilities";
import { catalogEntryMatches } from "./catalogEntry";

const prompt = {
  id: "bio-010", question: "Why does an egg white change when cooked?", age_band: "age9_to12",
  topic: "food_and_life", material_class: "egg", tags: ["denaturation"], script: ["add v1 egg_white 50g"],
  owning_task: "BRD-052", support: "computed" as const, reason_code: "computed-route",
};

/** The ONE matcher, over the haystack a prompt contributes to the one index. */
const matches = (query: string, locale = "en") =>
  catalogEntryMatches({ search: capabilitySearchText(prompt, locale) }, query);

describe("capability explorer data", () => {
  it("rejects unknown envelopes", () => expect(parseCapabilityIndex({ schema: 2, prompts: [prompt] })).toEqual([]));

  it("searches questions, topics, tags and owners", () => {
    expect(matches("egg white")).toBe(true);
    expect(matches("food and life")).toBe(true);
    expect(matches("brd-052")).toBe(true);
    expect(matches("electrolysis")).toBe(false);
  });

  it("searches the reviewed script, which is what the bench would run", () => {
    // A reader who knows the reagent should find the question about it
    // even when the question's own words never name it.
    expect(matches("egg_white")).toBe(true);
    expect(matches("add v1")).toBe(true);
  });
});

describe("what a question offers", () => {
  /**
   * The affordance follows SCRIPT PRESENCE, not support level.
   *
   * All sixty `boundary` rows in the shipped corpus carry an empty
   * script — the refusal IS the answer — and the old explorer gated its
   * run button on `support !== "missing"` alone, so it offered sixty rows
   * a button that ran an empty string and produced nothing.
   */
  it("offers no run for a refusal, which ships no script", () => {
    expect(capabilityRunnable({ support: "boundary", script: [] })).toBe(false);
  });

  it("offers no run for a question the science has not reached", () => {
    // `missing` HAS a script; what it lacks is the model behind it.
    expect(capabilityRunnable({ support: "missing", script: ["add v1 water 100mL"] })).toBe(false);
  });

  it("offers the run for every reviewed question with a script", () => {
    for (const support of ["computed", "curated", "qualitative"] as const) {
      expect(capabilityRunnable({ support, script: ["add v1 water 100mL"] })).toBe(true);
    }
  });

  it("reads the machine code as words, so the dictionary can reach it", () => {
    expect(capabilityReasonText({ boundary: null, reason_code: "unsupported-fracture-mechanics" }))
      .toBe("unsupported fracture mechanics");
    // Authored prose wins when the corpus ever ships any; today it never does.
    expect(capabilityReasonText({ boundary: "The bench models no crack growth.", reason_code: "x" }))
      .toBe("The bench models no crack growth.");
  });
});
