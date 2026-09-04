import { describe, expect, it } from "vitest";
import { capabilityMatches, parseCapabilityIndex } from "./capabilities";

const prompt = {
  id: "bio-010", question: "Why does an egg white change when cooked?", age_band: "age9_to12",
  topic: "food_and_life", material_class: "egg", tags: ["denaturation"], script: ["add v1 egg_white 50g"],
  owning_task: "BRD-052", support: "computed" as const, reason_code: "computed-route",
};

describe("capability explorer data", () => {
  it("rejects unknown envelopes", () => expect(parseCapabilityIndex({ schema: 2, prompts: [prompt] })).toEqual([]));
  it("searches questions, topics, tags and owners", () => {
    expect(capabilityMatches(prompt, "egg white")).toBe(true);
    expect(capabilityMatches(prompt, "food and life")).toBe(true);
    expect(capabilityMatches(prompt, "brd-052")).toBe(true);
    expect(capabilityMatches(prompt, "electrolysis")).toBe(false);
  });
});
