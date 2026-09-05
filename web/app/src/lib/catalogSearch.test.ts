import { describe, expect, it } from "vitest";
import { catalogEntries, catalogEntryMatches } from "./catalogEntry";
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

  it("finds a named material by a resolved component", () => {
    const material = {
      ...water,
      key: "teaching_mixture",
      name: "teaching mixture",
      formula: "",
      material_details: {
        basis: "mass_fraction" as const,
        confidence: "curated" as const,
        components: [{ key: "sodium_chloride", lower: 0.1, upper: 0.1 }],
        lot_assumptions: [],
        source_id: "source",
      },
    };
    expect(reagentMatches(material, "sodium chloride", "Lehrmischung")).toBe(true);
    expect(reagentMatches(material, "Natriumchlorid", "Lehrmischung", (value) =>
      value === "sodium chloride" ? "Natriumchlorid" : value,
    )).toBe(true);
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

/**
 * ONE box over the whole library.
 *
 * `experimentMatches` and `kidsExperimentMatches` each answered for one
 * corpus, which is fine while there are two lists and fatal once there is
 * one: a learner typing into a single box would silently be searching half
 * the shelf. The unified index is built with the entries themselves, so a
 * query reaches both, in whichever language is on screen.
 */
describe("the unified catalogue index", () => {
  const de = (value: string) => ({
    "vinegar and baking soda": "Essig und Natron",
  })[value] ?? value;
  const entries = catalogEntries(
    [{
      id: "vinegar-and-baking-soda", equation: "NaHCO3 + CH3COOH -> CO2",
      concepts: ["acid-carbonate"], apparatus: ["beaker"],
      setup: { script: "add v1 white_vinegar_5_percent 50mL\nadd v1 baking_soda 5g\n" },
      expect: {}, registers: { lv2: "Gas leaves the beaker." },
    }],
    [{
      id: "K06", title: "Magic milk", phenomenon: "Soap spreads colour",
      title_de: "Zaubermilch", phenomenon_de: "Seife verteilt Farbe",
      status: "computed", topics: ["surfaces"], ingredients: ["milk", "dish_soap"],
      apparatus: ["beaker"], safety: "home",
    }],
    { locale: "de", translate: de, completed: new Set() },
  );
  const found = (query: string) => entries.filter((entry) => catalogEntryMatches(entry, query)).map((e) => e.id);

  it("reaches both corpora from the same query", () => {
    expect(found("beaker").sort()).toEqual(["K06", "vinegar-and-baking-soda"]);
    expect(found("")).toHaveLength(2);
  });

  it("matches what is on screen and what is underneath it", () => {
    expect(found("Zaubermilch")).toEqual(["K06"]);
    expect(found("magic milk")).toEqual(["K06"]);
    expect(found("Essig")).toEqual(["vinegar-and-baking-soda"]);
    expect(found("acid carbonate")).toEqual(["vinegar-and-baking-soda"]);
    // The shelf key the entry will actually use, hyphen or underscore.
    expect(found("baking soda")).toEqual(["vinegar-and-baking-soda"]);
  });
});
