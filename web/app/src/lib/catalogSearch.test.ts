import { describe, expect, it } from "vitest";
import { equipmentMatches, experimentHasProgress, experimentMatches, experimentProgressLabel, normalizeCatalogText, reagentMatches } from "./catalogSearch";

const water = { key: "water", name: "water", formula: "H2O" };

describe("reagentMatches", () => {
  it("matches substrings in the localized display name", () => {
    expect(reagentMatches(water, "Wasser", "Wasser")).toBe(true);
    expect(reagentMatches(water, "asser", "Wasser")).toBe(true);
  });

  it("keeps canonical names and formulae searchable in every locale", () => {
    expect(reagentMatches(water, "water", "Wasser")).toBe(true);
    expect(reagentMatches(water, "2o", "Wasser")).toBe(true);
  });

  it("ignores case and accents", () => {
    expect(normalizeCatalogText("LÖSCH-Kalk")).toBe("losch-kalk");
  });
});

describe("experimentMatches", () => {
  const entry = {
    id: "hydrogen-peroxide-decomposition",
    equation: "2 H2O2 -> 2 H2O + O2",
    summary: "Catalytic decomposition",
    concepts: ["reaction-rate"],
    apparatus: ["catalyst"],
    models: ["kinetics"],
    registers: { lv1: "Watch oxygen form." },
  };
  const de = (value: string) => ({
    "hydrogen peroxide decomposition": "Zersetzung von Wasserstoffperoxid",
    "reaction rate": "Reaktionsgeschwindigkeit",
    "Watch oxygen form.": "Beobachte, wie Sauerstoff entsteht.",
  })[value] ?? value;

  it("matches localized titles, concepts, and register prose", () => {
    expect(experimentMatches(entry, "Wasserstoffperoxid", de)).toBe(true);
    expect(experimentMatches(entry, "geschwindigkeit", de)).toBe(true);
    expect(experimentMatches(entry, "Sauerstoff", de)).toBe(true);
  });

  it("keeps canonical ids and formulae searchable", () => {
    expect(experimentMatches(entry, "peroxide", de)).toBe(true);
    expect(experimentMatches(entry, "H2O2", de)).toBe(true);
  });
});

describe("experimentHasProgress", () => {
  const entry = { id: "known-result" };
  const completed = new Set(["known-result"]);

  it("filters only by persisted successful-run ids", () => {
    expect(experimentHasProgress(entry, completed, "all")).toBe(true);
    expect(experimentHasProgress(entry, completed, "completed")).toBe(true);
    expect(experimentHasProgress(entry, completed, "not-tried")).toBe(false);
    expect(experimentHasProgress({ id: "new-result" }, completed, "not-tried")).toBe(true);
  });

  it("partitions a mixed catalog and labels both states", () => {
    const entries = [{ id: "known-result" }, { id: "new-result" }, { id: "another-result" }];
    expect(entries.filter((item) => experimentHasProgress(item, completed, "all"))).toHaveLength(3);
    expect(entries.filter((item) => experimentHasProgress(item, completed, "completed"))).toEqual([entries[0]]);
    expect(entries.filter((item) => experimentHasProgress(item, completed, "not-tried"))).toEqual(entries.slice(1));
    expect(experimentProgressLabel(entries[0]!, completed)).toBe("completed");
    expect(experimentProgressLabel(entries[1]!, completed)).toBe("not tried");
  });
});

describe("equipmentMatches", () => {
  const centrifuge = {
    verb: "centrifuge",
    title: "mini centrifuge",
    blurb: "separate particles by spinning a balanced tube",
  };

  it("matches localized substrings from the card", () => {
    expect(equipmentMatches(
      centrifuge,
      "zentrif",
      "Mini-Zentrifuge",
      "Teilchen in einem ausgewuchteten Röhrchen durch Drehen trennen",
    )).toBe(true);
    expect(equipmentMatches(
      centrifuge,
      "Röhrchen",
      "Mini-Zentrifuge",
      "Teilchen in einem ausgewuchteten Röhrchen durch Drehen trennen",
    )).toBe(true);
  });

  it("keeps canonical apparatus vocabulary searchable in every locale", () => {
    expect(equipmentMatches(centrifuge, "centrifuge", "Mini-Zentrifuge", "Trennen")).toBe(true);
    expect(equipmentMatches(centrifuge, "balanced tube", "Mini-Zentrifuge", "Trennen")).toBe(true);
  });
});
