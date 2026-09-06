/**
 * The unified catalogue, walked over the shipped content.
 *
 * Two halves, and both matter. The fixtures pin the RULES — which age band
 * a placement produces, what a card's primary action is when an entry has
 * no script of its own. The walk over the real codex export and the real
 * guided catalogue pins the COVERAGE: this model exists so that ONE card
 * design and ONE filter rail can answer for every entry, and a derivation
 * that is correct on four hand-made rows while leaving a hole in twenty
 * shipped ones puts the tier split straight back into the filter bar.
 */
import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import {
  CATALOG_DURATIONS,
  CATALOG_LEVELS,
  CATALOG_TOPICS,
  catalogEntries,
  catalogEntryMatches,
  codexAgeMin,
  durationBand,
  durationLabel,
  filterCatalogEntries,
  levelCounts,
  levelForAge,
  levelLabel,
  minutesForSteps,
  NO_CATALOG_FILTERS,
  placementKey,
  presentPlacements,
  presentTopics,
  runTargetLabel,
  topicLabel,
  type CatalogFilters,
} from "./catalogEntry";
import { parseCodexIndex, type CodexEntry } from "./codex";
import { type KidsExperiment } from "./kidsCatalog";
import codexExportJson from "../../../../crates/kerotakis-codex/tests/golden/codex-export.json?raw";
import kidsCatalogJson from "../../../../data/kids/experiments-v1.json?raw";

const identity = (value: string) => value;
const context = (over: Partial<Parameters<typeof catalogEntries>[2]> = {}) => ({
  locale: "en",
  translate: identity,
  completed: new Set<string>(),
  ...over,
});

const codexEntry = (over: Partial<CodexEntry> = {}): CodexEntry => ({
  id: "silver-chloride-precipitation",
  equation: "AgNO3 + NaCl -> AgCl",
  concepts: ["precipitation"],
  setup: { script: "add v1 water 100mL\nadd v1 AgNO3 1mmol\nadd v1 NaCl 1mmol\n" },
  expect: { events: ["precipitated:AgCl"] },
  registers: { lv2: "Silver chloride leaves the solution." },
  curriculum: [{ system: "england-national-curriculum", stage: "KS3", ages: { min: 11 }, source: "DfE" }],
  ...over,
});

const guidedEntry = (over: Partial<KidsExperiment> = {}): KidsExperiment => ({
  id: "K99", title: "Volcano", phenomenon: "Soap traps gas as foam",
  title_de: "Vulkan", phenomenon_de: "Seife fängt Gas als Schaum",
  status: "computed", topics: ["gases", "acids"],
  ingredients: ["baking_soda", "white_vinegar_5_percent"], apparatus: ["beaker"],
  safety: "home", ...over,
});

describe("one entry model", () => {
  it("bands an age rather than a corpus", () => {
    expect(levelForAge(8)).toBe("starter");
    expect(levelForAge(11)).toBe("starter");
    expect(levelForAge(12)).toBe("intermediate");
    expect(levelForAge(14)).toBe("intermediate");
    expect(levelForAge(15)).toBe("advanced");
    expect(levelForAge(17)).toBe("advanced");
  });

  it("reads the youngest curriculum placement, and defaults without one", () => {
    expect(codexAgeMin(codexEntry({
      curriculum: [
        { system: "a", stage: "s", ages: { min: 16 }, source: "x" },
        { system: "b", stage: "t", ages: { min: 11 }, source: "y" },
      ],
    }))).toBe(11);
    expect(codexAgeMin(codexEntry({ curriculum: [] }))).toBe(12);
    expect(codexAgeMin(codexEntry({ curriculum: [{ system: "a", stage: "s", source: "x" }] }))).toBe(12);
  });

  it("prices a run at one pace for both corpora", () => {
    expect(minutesForSteps(2)).toBe(3);
    expect(minutesForSteps(10)).toBe(15);
    // Never zero minutes, and never a claim longer than an afternoon.
    expect(minutesForSteps(0)).toBe(2);
    expect(minutesForSteps(500)).toBe(60);
    expect(durationBand(3)).toBe("short");
    expect(durationBand(15)).toBe("medium");
    expect(durationBand(16)).toBe("long");
  });

  it("gives a guided task with a shipped codex entry the same run as the codex card", () => {
    const script = codexEntry({ id: "vinegar-and-baking-soda" });
    const [entry] = catalogEntries([script], [guidedEntry({ codex: ["vinegar-and-baking-soda"] })], context())
      .filter((item) => item.source === "guided");
    expect(entry?.run).toEqual({ kind: "script", entry: script });
    expect(entry?.script).toBe(script);
    // The duration is the SCRIPT's, because that is what will actually run.
    expect(entry?.steps).toBe(3);
  });

  it("falls back through lesson, quest and sandbox to the documented boundary", () => {
    const target = (over: Partial<KidsExperiment>) =>
      catalogEntries([], [guidedEntry(over)], context())[0]?.run;
    expect(target({ lesson: "volcano-foam.lab" })).toEqual({ kind: "lesson", file: "volcano-foam.lab" });
    expect(target({ quest: "gas-tests" })).toEqual({ kind: "quest", id: "gas-tests" });
    expect(target({})).toEqual({ kind: "sandbox" });
    expect(target({ status: "declined", boundary: "Bulk motion is out of scope.", boundary_de: "…" }))
      .toEqual({ kind: "boundary" });
  });

  it("ignores a cross-reference to an entry the export does not ship", () => {
    const [entry] = catalogEntries([], [guidedEntry({ codex: ["not-shipped"], lesson: "volcano-foam.lab" })], context());
    expect(entry?.script).toBeNull();
    expect(entry?.codexLinks).toEqual([]);
    expect(entry?.run).toEqual({ kind: "lesson", file: "volcano-foam.lab" });
  });

  it("reads one progress record for both corpora", () => {
    const script = codexEntry({ id: "hot-pack" });
    const entries = catalogEntries(
      [script],
      [guidedEntry({ codex: ["hot-pack"] }), guidedEntry({ id: "K98", lesson: "grit.lab" })],
      context({ completed: new Set(["hot-pack"]), completedMissions: new Set(["grit"]) }),
    );
    expect(entries.find((e) => e.id === "hot-pack")?.done).toBe(true);
    expect(entries.find((e) => e.id === "K99")?.done).toBe(true);
    // No codex link: the guided lesson's own completion is the record.
    expect(entries.find((e) => e.id === "K98")?.done).toBe(true);
  });

  it("answers the shelf question only when every material is reachable", () => {
    const entries = catalogEntries([], [guidedEntry({ ingredients: ["baking_soda", "milk"] })], context({
      shelfKeys: new Set(["baking_soda", "whole_milk"]),
    }));
    // `milk` is not a shelf key; the alias table is what makes it one.
    expect(entries[0]?.onShelf).toBe(true);
    expect(catalogEntries([], [guidedEntry({ ingredients: ["baking_soda", "milk"] })], context({
      shelfKeys: new Set(["baking_soda"]),
    }))[0]?.onShelf).toBe(false);
  });

  it("localizes the title, the hook and therefore the search", () => {
    const [entry] = catalogEntries([], [guidedEntry()], context({ locale: "de" }));
    expect(entry?.title).toBe("Vulkan");
    expect(catalogEntryMatches(entry!, "vulkan")).toBe(true);
    // The canonical English stays searchable in every locale.
    expect(catalogEntryMatches(entry!, "volcano")).toBe(true);
  });
});

describe("the filter rail composes", () => {
  const script = codexEntry({
    id: "hot-pack", concepts: ["enthalpy-of-solution"],
    equation: "CaCl2(s) -> Ca2+ + 2 Cl-", registers: { lv2: "Dissolving warms the water." },
  });
  const entries = catalogEntries(
    [script, codexEntry({ id: "silver-chloride", curriculum: [{ system: "bayern", stage: "Jgst. 10", ages: { min: 16 }, source: "ISB" }] })],
    [guidedEntry({ codex: ["hot-pack"] })],
    context({ completed: new Set(["hot-pack"]), shelfKeys: new Set(["baking_soda", "white_vinegar_5_percent"]) }),
  );
  const shown = (over: Partial<CatalogFilters>) =>
    filterCatalogEntries(entries, { ...NO_CATALOG_FILTERS, ...over }).map((entry) => entry.id);

  it("selects one axis at a time", () => {
    expect(shown({ level: "advanced" })).toEqual(["silver-chloride"]);
    expect(shown({ level: "starter" }).sort()).toEqual(["K99", "hot-pack"]);
    expect(shown({ progress: "completed" }).sort()).toEqual(["K99", "hot-pack"]);
    expect(shown({ progress: "not-tried" })).toEqual(["silver-chloride"]);
    expect(shown({ shelfOnly: true })).toEqual(["K99"]);
    // A guided task inherits the concepts of the script it actually runs,
    // so the concept filter reaches it too rather than stopping at the
    // corpus boundary.
    expect(shown({ concept: "enthalpy-of-solution" })).toEqual(["hot-pack", "K99"]);
    expect(shown({ curriculum: placementKey({ system: "bayern", stage: "Jgst. 10" }) })).toEqual(["silver-chloride"]);
  });

  it("composes axes rather than replacing them", () => {
    expect(shown({ level: "starter", shelfOnly: true })).toEqual(["K99"]);
    expect(shown({ level: "starter", progress: "not-tried" })).toEqual([]);
    expect(shown({ level: "advanced", query: "silver" })).toEqual(["silver-chloride"]);
    expect(shown({ level: "starter", query: "silver" })).toEqual([]);
    expect(shown({ topic: "gases", level: "starter" })).toEqual(["K99"]);
  });

  it("an unfiltered rail hides nothing", () => {
    expect(shown({}).length).toBe(entries.length);
  });
});

describe("the shipped library", () => {
  const codex = parseCodexIndex(JSON.parse(codexExportJson));
  // The shipped English source, as `conceptLinks.test.ts` reads it: the
  // German twins are merged into the payload by `tools/kids-catalog.py`
  // at build time, so the checked-in file has none and the runtime parser
  // would reject every row.
  const guided = (JSON.parse(kidsCatalogJson) as { experiments: KidsExperiment[] }).experiments;
  const entries = catalogEntries(codex, guided, context());

  it("is one list of both corpora", () => {
    expect(codex).toHaveLength(108);
    expect(guided).toHaveLength(60);
    expect(entries).toHaveLength(168);
    expect(entries).toHaveLength(codex.length + guided.length);
    expect(new Set(entries.map((entry) => entry.id)).size).toBe(entries.length);
  });

  it("makes adsorption and polymer heat response runnable and searchable", () => {
    const charcoal = entries.find((entry) => entry.id === "charcoal-holds-the-dye");
    const polymers = entries.find((entry) => entry.id === "chains-slide-networks-do-not");

    expect(charcoal?.run.kind).toBe("script");
    expect(charcoal?.needs).toEqual(expect.arrayContaining(["activated_charcoal", "methyl_orange", "water"]));
    expect(charcoal?.expectations).toEqual(expect.arrayContaining(["adsorbed:methyl_orange", "filtered"]));
    expect(charcoal?.topics).toEqual(["boundaries", "materials", "rates", "separations"]);
    expect(catalogEntryMatches(charcoal!, "activated charcoal")).toBe(true);
    expect(catalogEntryMatches(charcoal!, "adsorption")).toBe(true);

    expect(polymers?.run.kind).toBe("script");
    expect(polymers?.needs).toEqual(expect.arrayContaining(["thermoplastic", "thermoset_resin"]));
    expect(polymers?.expectations).toEqual(expect.arrayContaining([
      "polymer_heated:thermoplastic sheet",
      "polymer_heated:cured thermoset resin",
    ]));
    expect(polymers?.topics).toEqual(["boundaries", "heat", "materials"]);
    expect(catalogEntryMatches(polymers!, "thermoplastic")).toBe(true);
    expect(catalogEntryMatches(polymers!, "thermoset")).toBe(true);
  });

  it("leaves no entry without a title, a level, a hook or a run target", () => {
    for (const entry of entries) {
      expect(entry.title.trim(), entry.id).not.toBe("");
      expect(entry.hook.trim(), entry.id).not.toBe("");
      expect(entry.minutes, entry.id).toBeGreaterThan(0);
      expect(entry.topics.length, entry.id).toBeGreaterThan(0);
      expect(runTargetLabel(entry.run, entry.done), entry.id).not.toBe("");
      expect(levelLabel(entry.level), entry.id).not.toBe("");
    }
  });

  it("places every entry in the shared topic vocabulary", () => {
    const stray = [...new Set(entries.flatMap((entry) => entry.topics))]
      .filter((topic) => !(CATALOG_TOPICS as readonly string[]).includes(topic));
    expect(stray).toEqual([]);
    for (const topic of presentTopics(entries)) {
      expect(topicLabel(topic), topic).not.toBe(topic);
    }
  });

  it("fills all three levels, so the band is a filter and not a label", () => {
    const counts = levelCounts(entries);
    expect(counts.starter).toBeGreaterThan(20);
    expect(counts.intermediate).toBeGreaterThan(20);
    expect(counts.advanced).toBeGreaterThan(20);
    expect(counts.starter + counts.intermediate + counts.advanced).toBe(entries.length);
  });

  it("mixes both corpora inside the same level", () => {
    // The whole point: a level chip must not select one source. If it does,
    // the tier split has come back wearing a filter's clothes.
    for (const level of ["starter", "intermediate"] as const) {
      const sources = new Set(entries.filter((entry) => entry.level === level).map((entry) => entry.source));
      expect(sources, level).toEqual(new Set(["codex", "guided"]));
    }
  });

  it("keeps every guided task reachable through some action", () => {
    const stuck = entries.filter((entry) => entry.run.kind === "boundary" && !entry.boundary);
    expect(stuck).toEqual([]);
  });

  it("keeps the curriculum browsable as a filter", () => {
    const placements = presentPlacements(entries, "en");
    expect(placements.length).toBeGreaterThan(5);
    const first = placements[0]!;
    const shown = filterCatalogEntries(entries, { ...NO_CATALOG_FILTERS, curriculum: first.key });
    expect(shown.length).toBeGreaterThan(0);
  });
});

/**
 * The words a learner reads.
 *
 * The catalogue used to name half of itself after the age of the reader it
 * imagined — "Kinderlabor", "Kids Lab", "Kinderversuch" — and that naming
 * is what made two tiers feel like two libraries. Deleting the tier without
 * deleting the vocabulary would leave the judgement in place with nothing
 * to hang it on, so the vocabulary is checked here rather than trusted.
 *
 * Scoped to the strings the catalogue actually renders: the literal `t("…")`
 * calls in its component, and the labels that reach `t()` as a VARIABLE,
 * which no literal scan can see. Both their English keys and their German
 * renderings are read, because either one alone is a screen a learner sees.
 */
describe("the catalogue's own vocabulary", () => {
  const bundle = (name: string) => JSON.parse(
    readFileSync(join(import.meta.dirname, `../locales/${name}`), "utf8"),
  ) as { terms?: Record<string, string>; messages: Record<string, string> };
  const de = bundle("de.json");
  const template = bundle("_template.json");
  const german = (key: string) => de.messages[key] ?? de.terms?.[key];

  /** Keys the catalogue renders: the literal calls plus the derived labels. */
  const rendered = (() => {
    const source = readFileSync(join(import.meta.dirname, "components/Catalog.svelte"), "utf8")
      .replace(/<!--[\s\S]*?-->/g, "")
      .replace(/\/\*[\s\S]*?\*\//g, "");
    const keys = new Set<string>();
    for (const match of source.matchAll(/\bt\("([^"]+)"/g)) keys.add(match[1]!);
    for (const level of CATALOG_LEVELS) keys.add(levelLabel(level));
    for (const band of CATALOG_DURATIONS) keys.add(durationLabel(band));
    for (const topic of CATALOG_TOPICS) keys.add(topicLabel(topic));
    for (const done of [true, false]) {
      keys.add(runTargetLabel({ kind: "lesson", file: "x.lab" }, done));
      keys.add(runTargetLabel({ kind: "quest", id: "x" }, done));
      keys.add(runTargetLabel({ kind: "sandbox" }, done));
      keys.add(runTargetLabel({ kind: "boundary" }, done));
    }
    return [...keys];
  })();

  it("finds the strings, so the sweep is not vacuous", () => {
    expect(rendered.length).toBeGreaterThan(40);
    expect(rendered).toContain("run it on the bench");
  });

  it("names no reader by age, in either language", () => {
    const forbidden = /kinder|\bkids?\b|kinderlabor/i;
    const offenders = rendered.filter((key) => forbidden.test(key) || forbidden.test(german(key) ?? ""));
    expect(offenders).toEqual([]);
  });

  it("leaves the retired names in neither bundle", () => {
    for (const gone of ["Kids Lab", "Kids Lab bench guide", "kids task",
      "find a kids experiment", "sixty experiments for curious kids"]) {
      expect(de.messages, gone).not.toHaveProperty([gone]);
      expect(template.messages, gone).not.toHaveProperty([gone]);
    }
  });

  it("gives every derived label a German rendering", () => {
    // These reach `t()` as a variable, so `i18n.test.ts`'s literal scan
    // walks straight past them and a missing one ships as English.
    const missing = rendered.filter((key) => german(key) === undefined);
    expect(missing).toEqual([]);
  });

  it("offers every one of them to a new translator", () => {
    const missing = rendered.filter((key) =>
      de.messages[key] !== undefined && !Object.hasOwn(template.messages, key));
    expect(missing).toEqual([]);
  });
});
