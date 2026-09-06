import { describe, expect, it } from "vitest";
import { catalogEntries } from "./catalogEntry";
import { codexLearningLabel, guidedLearningLabel, kidsConnections, kidsExperimentMatches, kidsText, parseKidsCatalog, type KidsExperiment } from "./kidsCatalog";

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
      linkedLearning: 2, completedLearning: 2, progress: "all",
    });
  });

  it("reports none, some, and all from saved lesson and Codex ids only", () => {
    const linked = { ...apple, lesson: "apple-browning.lab", quest: "temporary-quest", codex: ["vitamin-c"] };
    const progress = (missions: string[], experiments: string[]) => kidsConnections(
      linked, new Set(), new Set(["vitamin-c"]), new Set(missions), new Set(experiments),
    ).progress;
    expect(progress([], [])).toBe("none");
    expect(progress(["apple-browning"], [])).toBe("some");
    expect(progress(["apple-browning"], ["vitamin-c"])).toBe("all");
  });

  it("uses Continue until stable completion exists, then Replay", () => {
    expect(guidedLearningLabel(false)).toBe("continue guided lesson");
    expect(guidedLearningLabel(true)).toBe("replay guided lesson");
    expect(codexLearningLabel(false)).toBe("continue Codex investigation");
    expect(codexLearningLabel(true)).toBe("replay Codex investigation");
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

/**
 * A guided task is a catalogue card like any other.
 *
 * The corpus keeps its own shape on disk — a phenomenon, a safety note, a
 * cross-reference — but nothing downstream of `catalogEntries` may still be
 * able to tell it apart from a codex entry, because that is exactly the
 * distinction the two-tier catalogue kept showing the learner.
 */
describe("a guided task as one catalogue card", () => {
  const script = {
    id: "vitamin-c", setup: { script: "add v1 water 100mL\nadd v1 ascorbic_acid 1g\n" },
    expect: {}, registers: {}, concepts: ["redox-reactions"],
    curriculum: [{ system: "england-national-curriculum", stage: "KS4", ages: { min: 14 }, source: "DfE" }],
  };

  it("carries a title, a level, a duration and a run door", () => {
    const [card] = catalogEntries([script], [{ ...apple, codex: ["vitamin-c"] }], {
      locale: "en", translate: (value) => value, completed: new Set(),
    }).filter((entry) => entry.source === "guided");
    expect(card?.title).toBe(apple.title);
    // Supervision-free means the youngest band; the level is a claim about
    // the experiment, not about which file it shipped in.
    expect(card?.level).toBe("starter");
    expect(card?.minutes).toBeGreaterThan(0);
    expect(card?.run).toEqual({ kind: "script", entry: script });
    expect(card?.topics.length).toBeGreaterThan(0);
  });

  it("keeps its documented boundary rather than hiding it behind a tier", () => {
    const [card] = catalogEntries([], [apple], {
      locale: "de", translate: (value) => value, completed: new Set(),
    });
    expect(card?.boundary).toBe(apple.boundary_de);
    expect(card?.topics).toContain("boundaries");
  });
});
