import { describe, expect, it } from "vitest";
import { kidsConnections, kidsExperimentMatches, kidsText, parseKidsCatalog, type KidsExperiment } from "./kidsCatalog";

const apple: KidsExperiment = {
  id: "K45", title: "Stop an apple going brown", phenomenon: "Enzymatic browning",
  status: "boundary", topics: ["food", "enzymes"], ingredients: ["ascorbic_acid", "apple"],
  apparatus: ["look"], safety: "home", boundary: "Browning is not modeled.",
  title_de: "Verhindern, dass ein Apfel braun wird", phenomenon_de: "Enzymatische Bräunung",
  boundary_de: "Bräunung wird nicht modelliert.",
};

describe("kids catalog", () => {
  it("fails closed when the payload is absent or malformed", () => {
    expect(parseKidsCatalog(null)).toEqual([]);
    expect(parseKidsCatalog({ schema: 1, experiments: [{ id: "K45" }] })).toEqual([]);
    expect(parseKidsCatalog({ schema: 2, experiments: [apple] })).toEqual([]);
    expect(parseKidsCatalog({ schema: 1, experiments: [{ ...apple, safety: "unknown" }] })).toEqual([]);
  });

  it("accepts the current schema", () => {
    expect(parseKidsCatalog({ schema: 1, experiments: [apple] })).toEqual([apple]);
  });

  it("rejects malformed optional cross-references", () => {
    expect(parseKidsCatalog({ schema: 1, experiments: [{ ...apple, capabilities: [] }] })).toEqual([]);
    expect(parseKidsCatalog({ schema: 1, experiments: [{ ...apple, codex: [42] }] })).toEqual([]);
  });

  it("resolves only available links and reports real persisted progress", () => {
    const linked = { ...apple, lesson: "apple-browning.lab", capabilities: ["bio-095", "gone"], codex: ["vitamin-c", "gone"] };
    expect(kidsConnections(
      linked,
      new Set(["bio-095"]),
      new Set(["vitamin-c"]),
      new Set(["apple-browning"]),
      new Set(["vitamin-c"]),
    )).toEqual({
      capabilities: ["bio-095"], codex: ["vitamin-c"], lessonCompleted: true, codexCompleted: ["vitamin-c"],
    });
  });

  it("searches number, title, ingredient, apparatus and boundary text", () => {
    for (const query of ["K45", "apple", "ascorbic acid", "look", "not modeled"]) {
      expect(kidsExperimentMatches(apple, query)).toBe(true);
    }
    expect(kidsExperimentMatches(apple, "electrolysis")).toBe(false);
  });

  it("searches reviewed connection identifiers", () => {
    const linked = { ...apple, capabilities: ["bio-095"], codex: ["fruit-browning"] };
    expect(kidsExperimentMatches(linked, "bio 095")).toBe(true);
    expect(kidsExperimentMatches(linked, "fruit browning")).toBe(true);
  });

  it("renders and searches the selected catalog locale with English fallback", () => {
    expect(kidsText(apple, "title", "de")).toContain("Apfel");
    expect(kidsText(apple, "boundary", "de")).toContain("Bräunung");
    expect(kidsExperimentMatches(apple, "verhindern", "de")).toBe(true);
    expect(kidsText({ ...apple, title_de: undefined }, "title", "de")).toBe(apple.title);
  });
});

describe("prepared KIDS routes", () => {
  it("exposes the three computed mechanisms as guided lessons", () => {
    const rows = parseKidsCatalog({ schema: 1, experiments: [
      { ...apple, status: "computed", ingredients: ["cut_apple", "ascorbic_acid"], lesson: "apple-browning.lab" },
      { ...apple, id: "K14", title: "Naked egg", status: "computed", ingredients: ["naked_egg", "water"], lesson: "naked-egg-osmosis.lab" },
      { ...apple, id: "K39", title: "Soap scum", status: "computed", ingredients: ["fatty_soap", "CaCl2"], lesson: "hard-water-soap-scum.lab" },
    ] });
    expect(rows.map((row) => row.lesson)).toEqual([
      "apple-browning.lab", "naked-egg-osmosis.lab", "hard-water-soap-scum.lab",
    ]);
  });
});
