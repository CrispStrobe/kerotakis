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
  CATALOG_SOURCES,
  CATALOG_TOPICS,
  authoredRelatedEntries,
  catalogEntries,
  catalogEntryMatches,
  durationBand,
  durationLabel,
  filterCatalogEntries,
  levelCounts,
  levelLabel,
  minutesForSteps,
  NO_CATALOG_FILTERS,
  placementKey,
  presentPlacements,
  presentTopics,
  experimentOnlyFilters,
  oneIndex,
  runTargetLabel,
  sourceCounts,
  sourceLabel,
  sourceLabelPlural,
  topicLabel,
  type CatalogFilters,
} from "./catalogEntry";
import type { CapabilityPrompt } from "./capabilities";
import { entryLocked, metConcepts, parseCodexIndex, type CodexEntry } from "./codex";
import { type KidsExperiment } from "./kidsCatalog";
import codexExportJson from "../../../../crates/kerotakis-codex/tests/golden/codex-export.json?raw";
import kidsCatalogJson from "../../../../data/kids/experiments-v1.json?raw";
import kidsGermanJson from "../../../../data/kids/experiments-de-v1.json?raw";

const identity = (value: string) => value;
const context = (over: Partial<Parameters<typeof catalogEntries>[2]> = {}) => ({
  locale: "en",
  translate: identity,
  completed: new Set<string>(),
  ...over,
});

const codexEntry = (over: Partial<CodexEntry> = {}): CodexEntry => ({
  id: "silver-chloride-precipitation",
  progress: "starter",
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
  status: "computed", progress: "starter", topics: ["gases", "acids"],
  ingredients: ["baking_soda", "white_vinegar_5_percent"], apparatus: ["beaker"],
  safety: "home", ...over,
});

const question = (over: Partial<CapabilityPrompt> = {}): CapabilityPrompt => ({
  id: "aq-001", question: "What happens when salt is stirred into water?",
  age_band: "age9_to12", topic: "mix_and_dissolve", material_class: "salt-water",
  tags: ["dissolution"], script: ["add v1 water 100mL", "add v1 NaCl 5g", "stir v1"],
  owning_task: "CAP-23", support: "computed", reason_code: "computed-route",
  ...over,
});

describe("one entry model", () => {
  it("uses authored Codex progress without reading curriculum ages", () => {
    const entry = codexEntry({
      progress: "advanced",
      curriculum: [{ system: "a", stage: "s", ages: { min: 8 }, source: "x" }],
    });
    expect(catalogEntries([entry], [], context())[0]?.level).toBe("advanced");
    expect(catalogEntries([entry], [], context())[0]?.safety).toBeNull();
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

  it("uses authored guided progress without inferring it from supervision", () => {
    const entries = catalogEntries([], [
      guidedEntry({ id: "K97", progress: "advanced", safety: "home" }),
      guidedEntry({ id: "K98", progress: "starter", safety: "school" }),
    ], context());
    expect(Object.fromEntries(entries.map((entry) => [entry.id, entry.level]))).toEqual({
      K97: "advanced",
      K98: "starter",
    });
  });

  it("connects a structured preview to the exact familiar kit", () => {
    const [entry] = catalogEntries([], [guidedEntry({
      recipe: [{ ingredient: "baking_soda", quantity: "5 g" }],
      procedure: ["Add the powder."], observations: ["Gas forms."], kits: ["balloon-kit"],
    })], context());
    expect(entry?.recipe[0]?.quantity).toBe("5 g");
    expect(entry?.procedure).toEqual(["Add the powder."]);
    expect(entry?.observations).toEqual(["Gas forms."]);
    expect(entry?.kits[0]?.parts).toEqual(["balloon or gas bag", "sealed connection", "sample vessel"]);
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

  it("marks a mixed-route experiment done when either its lesson or primary Codex run is complete", () => {
    const script = codexEntry({ id: "hot-pack" });
    const guided = guidedEntry({ codex: ["hot-pack"], lesson: "kitchen-hot-and-cold-packs.lab" });
    const lessonDone = catalogEntries([script], [guided], context({
      completedMissions: new Set(["kitchen-hot-and-cold-packs"]),
    }))[1];
    const codexDone = catalogEntries([script], [guided], context({ completed: new Set(["hot-pack"]) }))[1];
    expect(lessonDone?.done).toBe(true);
    expect(codexDone?.done).toBe(true);
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
    [script, codexEntry({ id: "silver-chloride", progress: "advanced", curriculum: [{ system: "bayern", stage: "Jgst. 10", ages: { min: 16 }, source: "ISB" }] })],
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

  it("filters readiness from current shelf plus exact engine answers", () => {
    const catalog = new Map([
      ["beaker", { id: "beaker", kind: "apparatus" as const, minimum_completed: 2, available: false,
        reason: { reason: "locked" as const, minimum_completed: 2 } }],
    ]);
    const [entry] = catalogEntries([], [guidedEntry()], context({
      shelfKeys: new Set(["baking_soda"]), catalog,
    }));
    expect(entry?.missingNeeds).toEqual(["white_vinegar_5_percent"]);
    expect(entry?.readyNow).toBe(false);
    expect(entry?.access).toContainEqual(expect.objectContaining({ id: "beaker", available: false }));
    expect(filterCatalogEntries([entry!], { ...NO_CATALOG_FILTERS, readiness: "missing" })).toHaveLength(1);
    expect(filterCatalogEntries([entry!], { ...NO_CATALOG_FILTERS, readiness: "ready" })).toHaveLength(0);
  });

  it("never promotes an unloaded catalog to ready or missing", () => {
    const [entry] = catalogEntries([], [guidedEntry({ ingredients: [], apparatus: [] })], context({
      shelfKeys: new Set(), catalog: new Map(),
    }));
    expect(entry).toMatchObject({ availabilityKnown: false, readyNow: false });
    expect(filterCatalogEntries([entry!], { ...NO_CATALOG_FILTERS, readiness: "ready" })).toHaveLength(0);
    expect(filterCatalogEntries([entry!], { ...NO_CATALOG_FILTERS, readiness: "missing" })).toHaveLength(0);
  });

  it("uses the equipment catalogue's canonical instrument id for readiness", () => {
    const catalog = new Map([
      ["measure:ph", { id: "measure:ph", kind: "instrument" as const, minimum_completed: 2, available: false,
        reason: { reason: "locked" as const, minimum_completed: 2 } }],
    ]);
    const [entry] = catalogEntries([], [guidedEntry({ apparatus: ["ph"] })], context({
      shelfKeys: new Set(["baking_soda", "white_vinegar_5_percent"]), catalog,
    }));
    expect(entry?.access.map((item) => item.id)).toContain("measure:ph");
    expect(entry?.readyNow).toBe(false);
  });

  it("relates entries only through authored exact identifiers", () => {
    const codex = codexEntry({ id: "foam-model" });
    const entries = catalogEntries([codex], [
      guidedEntry({ id: "K98", codex: ["foam-model"] }),
      guidedEntry({ id: "K99", title: "Foam lookalike", codex: [] }),
    ], context());
    const script = entries.find((entry) => entry.id === "foam-model")!;
    expect(authoredRelatedEntries(script, entries).map((entry) => entry.id)).toEqual(["K98"]);
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

  // 252, not 208: GUI-104 gave the 44 lessons that shipped reachable only
  // from the picker's "more" bucket a catalogue row each, so the library now
  // counts every runnable thing it ships rather than the subset someone had
  // got round to listing.
  it("is one list of both corpora", () => {
    expect(codex).toHaveLength(131);
    expect(guided).toHaveLength(121);
    expect(entries).toHaveLength(252);
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

  /**
   * Sandbox locks nothing, over the whole library rather than a sample.
   *
   * Owner, from the German deploy: "why are some experiments 'gesperrt' in
   * Sandbox mode???". They were: the authored learning progression
   * (`CodexEntry.requires`) is a real ordering and the concept map printed
   * it as a lock in both laboratories, so a reader who had come through
   * the door marked "everything unlocked" was told most of the library was
   * shut — by a badge over a button that would have opened it anyway.
   *
   * Walked over every shipped entry at three points on the progression —
   * nothing run, one concept met, everything met — because a rule that
   * only reads correctly for a learner with an empty record is the bug
   * this replaces. The Story half is asserted alongside it: if the
   * prerequisites stopped gating there too, the fix would have deleted the
   * progression rather than scoped it.
   */
  it("locks nothing in Sandbox, at any point in a learner's progression", () => {
    const everything = new Set(entries.flatMap((entry) => [
      ...entry.concepts,
      ...(entry.script?.requires ?? []),
    ]));
    const records: ReadonlySet<string>[] = [
      new Set<string>(),
      metConcepts(codex, new Set(["hot-pack"])),
      everything,
    ];
    const scripted = entries.filter((entry) => entry.script !== null);
    expect(scripted.length).toBeGreaterThan(100);

    for (const met of records) {
      const sandbox = scripted.filter((entry) => entryLocked(entry.script!, met, "sandbox"));
      expect(sandbox.map((entry) => entry.id)).toEqual([]);
    }

    // Story still orders the library, and the fresh record is the one where
    // that ordering has the most to say.
    const story = scripted.filter((entry) => entryLocked(entry.script!, new Set(), "story"));
    expect(story.length).toBeGreaterThan(50);
    // ...and it is progress, not a wall: meeting every concept opens it.
    expect(scripted.filter((entry) => entryLocked(entry.script!, everything, "story")).map((entry) => entry.id)).toEqual([]);
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
      keys.add(runTargetLabel({ kind: "question" }, done));
      keys.add(runTargetLabel({ kind: "unanswered" }, done));
      keys.add(runTargetLabel({ kind: "boundary" }, done));
    }
    for (const source of CATALOG_SOURCES) {
      keys.add(sourceLabel(source));
      keys.add(sourceLabelPlural(source));
    }
    return [...keys];
  })();

  it("finds the strings, so the sweep is not vacuous", () => {
    expect(rendered.length).toBeGreaterThan(40);
    expect(rendered).toContain("run it on the bench");
    expect(rendered).toContain("answered questions");
    expect(rendered).toContain("run this question on the bench");
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

/**
 * The third population, and the line this index must not cross.
 *
 * GUI-105 put the five hundred reviewed corpus questions into the same
 * list as the experiments, because nobody arrives asking whether their
 * question is a runnable experiment or a reviewed capability claim — they
 * ask "can it do this?", and that used to have to be asked in two places
 * or it got a wrong "no".
 *
 * What is NOT allowed is the flattening. A question is not an experiment:
 * sixty of them are refusals that ship no script at all. So every test
 * below is about the same thing from a different side — the row says what
 * it IS, the counts stay separate, and nothing offers a button that
 * produces nothing.
 */
describe("the one index over three populations", () => {
  it("keeps the kind on the row, as the facet it now is", () => {
    const entries = oneIndex([codexEntry()], [guidedEntry()], [question()], context());
    expect(entries.map((entry) => entry.source).sort())
      .toEqual(["capability", "codex", "guided"]);
    expect(CATALOG_SOURCES).toEqual(["codex", "guided", "capability"]);
  });

  it("counts each kind separately, derived from the rows", () => {
    const entries = oneIndex(
      [codexEntry()],
      [guidedEntry(), guidedEntry({ id: "K98" })],
      [question(), question({ id: "aq-002" }), question({ id: "aq-003" })],
      context(),
    );
    expect(sourceCounts(entries)).toEqual({ codex: 1, guided: 2, capability: 3 });
    // The experiments' own total is unchanged by the questions arriving,
    // which is the whole reason the headline can be honest.
    expect(catalogEntries([codexEntry()], [guidedEntry()], context())).toHaveLength(2);
  });

  it("orders the runnable half first, so the default list is not five hundred questions", () => {
    const entries = oneIndex([codexEntry()], [guidedEntry()], [question()], context());
    expect(entries.map((entry) => entry.source)).toEqual(["codex", "guided", "capability"]);
  });

  it("offers the run only where there is a script to run", () => {
    const runnable = oneIndex([], [], [question()], context())[0]!;
    expect(runnable.run.kind).toBe("question");
    expect(runTargetLabel(runnable.run, false)).toBe("run this question on the bench");
  });

  /**
   * The defect this task was warned about, in its exact costume.
   *
   * Every one of the sixty `boundary` rows the corpus ships carries an
   * EMPTY script — the refusal is the answer — and the retired explorer
   * gated its run button on `support !== "missing"` alone. Sixty rows
   * therefore offered a button that handed the runner an empty string.
   */
  it("offers no run for a refusal, which ships no script", () => {
    const refusal = oneIndex([], [], [
      question({ id: "aq-025", support: "boundary", reason_code: "fictional-material", script: [] }),
    ], context())[0]!;
    expect(refusal.run.kind).toBe("unanswered");
    expect(refusal.support).toBe("boundary");
    expect(refusal.reason).toBe("fictional material");
    // Not the run label, and not the model-boundary label either: a
    // refusal is an answer this corpus reviewed, not a model that stops.
    expect(runTargetLabel(refusal.run, false)).toBe("read why this one has no answer");
  });

  it("offers no run for a question the science has not reached, script or not", () => {
    const missing = oneIndex([], [], [
      question({ id: "mat-054", support: "missing", reason_code: "not-yet-modeled" }),
    ], context())[0]!;
    expect(missing.prompt?.script.length).toBeGreaterThan(0);
    expect(missing.run.kind).toBe("unanswered");
    expect(missing.reason).toBe("not yet modeled");
  });

  it("makes no claim a question never made", () => {
    const row = oneIndex([], [], [question()], context({
      shelfKeys: new Set(["water"]),
      catalog: new Map([["water", { id: "water", kind: "reagent" as const, minimum_completed: 0,
        available: true, reason: { reason: "sandbox" as const } }]]),
    }) as Parameters<typeof oneIndex>[3])[0]!;
    // The corpus script writes formulae (`add v1 NaCl 5g`) where the shelf
    // is keyed by registry id, so a derived shopping list reported salt
    // that IS on the shelf as missing. It asks for nothing instead.
    expect(row.needs).toEqual([]);
    expect(row.apparatus).toEqual([]);
    expect(row.missingNeeds).toEqual([]);
    expect(row.availabilityKnown).toBe(false);
    expect(row.readyNow).toBe(false);
    // Progress is a record of successful codex runs; a question id is not
    // a codex id, so "completed" cannot be true and "not tried" is not a
    // claim this row is entitled to make.
    expect(row.done).toBe(false);
    expect(row.script).toBeNull();
    expect(row.safety).toBeNull();
  });

  it("gives a refusal no duration to round up", () => {
    const refusal = oneIndex([], [], [question({ support: "boundary", script: [] })], context())[0]!;
    expect(refusal.steps).toBe(0);
    expect(refusal.minutes).toBe(0);
  });

  it("reads the corpus band as a learning level, never as an age", () => {
    const rows = oneIndex([], [], [
      question({ id: "a", age_band: "age9_to12" }),
      question({ id: "b", age_band: "age13_to15" }),
      question({ id: "c", age_band: "age16_to18" }),
    ], context());
    expect(rows.map((row) => row.level)).toEqual(["starter", "intermediate", "advanced"]);
    expect(rows.every((row) => !row.anyLevel)).toBe(true);
  });

  it("hides an all-levels question behind no level chip", () => {
    const rows = oneIndex([], [], [question({ age_band: "all" })], context());
    expect(rows[0]!.anyLevel).toBe(true);
    for (const level of CATALOG_LEVELS) {
      expect(filterCatalogEntries(rows, { ...NO_CATALOG_FILTERS, level })).toHaveLength(1);
    }
    // And the chip's number is the number of rows the chip will show.
    expect(levelCounts(rows)).toEqual({ starter: 1, intermediate: 1, advanced: 1 });
  });

  it("selects one kind with the facet, and the query across all three", () => {
    const entries = oneIndex([codexEntry()], [guidedEntry()], [question()], context());
    const only = (source: Parameters<typeof sourceLabel>[0]) =>
      filterCatalogEntries(entries, { ...NO_CATALOG_FILTERS, source }).map((entry) => entry.id);
    expect(only("capability")).toEqual(["aq-001"]);
    expect(only("codex")).toEqual(["silver-chloride-precipitation"]);
    expect(only("guided")).toEqual(["K99"]);
    // ONE query, over every population. `water` is a reagent the codex
    // script adds and a word the question asks about, and BOTH come back
    // from one box — which is the wrong "no" this task existed to remove.
    const found = filterCatalogEntries(entries, { ...NO_CATALOG_FILTERS, query: "water" });
    expect(found.map((entry) => entry.source)).toEqual(["codex", "capability"]);
    // And the facet still narrows that same query rather than replacing it.
    expect(filterCatalogEntries(entries, { ...NO_CATALOG_FILTERS, query: "water", source: "capability" }))
      .toHaveLength(1);
  });

  it("folds a question into the shared topic vocabulary, so one chip means one thing", () => {
    const rows = oneIndex([], [], [
      question({ id: "a", topic: "mix_and_dissolve" }),
      question({ id: "b", topic: "burn_and_oxidise", tags: ["combustion"] }),
      question({ id: "c", support: "boundary", script: [], topic: "materials", tags: [] }),
    ], context());
    expect(rows.find((row) => row.id === "a")!.topics).toContain("solutions");
    expect(rows.find((row) => row.id === "b")!.topics).toEqual(expect.arrayContaining(["heat", "redox"]));
    // A refusal is a statement about where this bench stops, which is a
    // topic the catalogue already had a chip for.
    expect(rows.find((row) => row.id === "c")!.topics).toContain("boundaries");
    for (const row of rows) {
      expect(row.topics.every((topic) => CATALOG_TOPICS.includes(topic as never))).toBe(true);
    }
  });

  it("links a question to the experiment that cites it, with no second door", () => {
    // `capabilities` on a guided row is a list of corpus ids. Both sides
    // of that authored relation are rows of this one index now, so the
    // link opens a page here instead of handing over to another dialog.
    const entries = oneIndex(
      [],
      [guidedEntry({ capabilities: ["aq-001"] })],
      [question()],
      context(),
    );
    const guided = entries.find((entry) => entry.id === "K99")!;
    const asked = entries.find((entry) => entry.id === "aq-001")!;
    expect(authoredRelatedEntries(guided, entries).map((entry) => entry.id)).toContain("aq-001");
    expect(authoredRelatedEntries(asked, entries).map((entry) => entry.id)).toContain("K99");
  });

  it("names the filters a question cannot answer, rather than looking broken", () => {
    // A concept, a curriculum stage, a shelf and a completion record are
    // properties of an experiment. Each of these drops all five hundred
    // questions by construction, and the rail has to say so.
    for (const over of [
      { concept: "precipitation" },
      { curriculum: placementKey({ system: "england-national-curriculum", stage: "KS3" }) },
      { shelfOnly: true },
      { readiness: "ready" as const },
      { readiness: "missing" as const },
      { progress: "completed" as const },
    ]) {
      expect(experimentOnlyFilters({ ...NO_CATALOG_FILTERS, ...over })).toBe(true);
    }
    // What a question CAN answer is left alone: free text, level, topic,
    // duration and the facet itself all select across all three kinds.
    for (const over of [
      { query: "salt" },
      { level: "starter" as const },
      { topic: "solutions" },
      { duration: "short" as const },
      { source: "capability" as const },
      { progress: "not-tried" as const },
    ]) {
      expect(experimentOnlyFilters({ ...NO_CATALOG_FILTERS, ...over })).toBe(false);
    }
  });

  it("searches a question in the language it is read in", () => {
    const rows = oneIndex([], [], [
      { ...question(), question_de: "Was passiert, wenn Kochsalz in Wasser gerührt wird?" } as CapabilityPrompt,
    ], context({ locale: "de" }) as Parameters<typeof oneIndex>[3]);
    expect(rows[0]!.title).toBe("Was passiert, wenn Kochsalz in Wasser gerührt wird?");
    expect(catalogEntryMatches(rows[0]!, "Kochsalz")).toBe(true);
    // Accent-folded, which the retired predicate was not.
    expect(catalogEntryMatches(rows[0]!, "geruhrt")).toBe(true);
    // And still findable by the English an author or a link would paste.
    expect(catalogEntryMatches(rows[0]!, "stirred into water")).toBe(true);
  });
});

/**
 * The haystack, pinned — because the list is now a WINDOW.
 *
 * Only the cards near the viewport are in the DOM, so find-in-page can no
 * longer be the fallback for anything the in-app box fails to match. That
 * makes `entry.search` the whole of a reader's reach into seven hundred
 * rows, and a field quietly left out of it is a card that cannot be found
 * by words printed on its own face.
 *
 * So the contract is narrow and loud: for all three row kinds, in both
 * languages, the PROSE is findable — not only the title. If a later edit
 * narrows a haystack back to titles and keys, these fail.
 *
 * They were written because three of them did not pass. In the built
 * payload a guided card draws its safety rationale (58 rows do) and
 * ships German procedure and observation lists (22 rows do), and none of
 * that prose was in the haystack; a corpus question drew a German refusal
 * that only existed after `t()`, which the haystack never called.
 */
describe("the search reaches the prose, not only the title", () => {
  /** A dictionary with just the words a case needs, English passing through. */
  const dictionary = (words: Record<string, string>) => (value: string) => words[value] ?? value;

  const german = (words: Record<string, string> = {}) =>
    context({ locale: "de", translate: dictionary(words) }) as Parameters<typeof oneIndex>[3];
  const english = () => context() as Parameters<typeof oneIndex>[3];

  describe("a codex experiment", () => {
    const script = codexEntry({
      id: "silver-chloride-precipitation",
      summary: "A cloudy curd settles out the instant the two clear liquids meet.",
      summary_de: "Ein trüber Quark fällt aus, sobald die beiden klaren Flüssigkeiten sich treffen.",
    });

    it("is found by its summary, which is nowhere in its title", () => {
      const [row] = oneIndex([script], [], [], english());
      expect(row!.title).toBe("silver chloride precipitation");
      expect(catalogEntryMatches(row!, "cloudy curd settles")).toBe(true);
      // The equation is the card's hook when there is no summary.
      expect(catalogEntryMatches(row!, "AgNO3")).toBe(true);
    });

    it("is found by the German summary it draws, and by the English underneath", () => {
      const [row] = oneIndex([script], [], [], german());
      expect(row!.hook).toBe("Ein trüber Quark fällt aus, sobald die beiden klaren Flüssigkeiten sich treffen.");
      expect(catalogEntryMatches(row!, "truber Quark")).toBe(true);
      expect(catalogEntryMatches(row!, "cloudy curd")).toBe(true);
    });

    it("is found by the material names and topic chips as the card prints them", () => {
      const [row] = oneIndex([script], [], [], german({
        NaCl: "Kochsalz",
        "crystals and precipitates": "Kristalle und Niederschläge",
      }));
      expect(row!.needs).toContain("NaCl");
      expect(catalogEntryMatches(row!, "Kochsalz")).toBe(true);
      // The registry key the script writes is still the fastest way in.
      expect(catalogEntryMatches(row!, "NaCl")).toBe(true);
      expect(row!.topics).toContain("crystals");
      expect(catalogEntryMatches(row!, "Kristalle")).toBe(true);
    });
  });

  describe("a guided experiment", () => {
    const guided = guidedEntry({
      id: "K42",
      title: "Volcano", title_de: "Vulkan",
      phenomenon: "Soap traps the gas as a climbing foam",
      phenomenon_de: "Seife fängt das Gas als kletternden Schaum",
      boundary: "The bench models the gas, not the smell",
      boundary_de: "Die Bank modelliert das Gas, nicht den Geruch",
      safety_rationale: "Vinegar at five percent stings a cut but will not burn",
      safety_rationale_de: "Essig mit fünf Prozent brennt in einer Wunde, verätzt aber nicht",
      safety_guidance: "Keep it out of eyes and rinse a splash away",
      safety_guidance_de: "Von den Augen fernhalten und Spritzer abspülen",
      procedure: ["Pour the vinegar into the bottle", "Drop the powder in and step back"],
      procedure_de: ["Gieße den Essig in die Flasche", "Wirf das Pulver hinein und tritt zurück"],
      observations: ["Foam climbs the neck and overflows"],
      observations_de: ["Schaum steigt den Hals hinauf und läuft über"],
    });

    it("is found by its procedure, observations and boundary in English", () => {
      const [row] = oneIndex([], [guided], [], english());
      expect(row!.title).toBe("Volcano");
      expect(catalogEntryMatches(row!, "drop the powder in")).toBe(true);
      expect(catalogEntryMatches(row!, "climbs the neck")).toBe(true);
      expect(catalogEntryMatches(row!, "not the smell")).toBe(true);
    });

    it("is found by the German procedure and observations the card draws", () => {
      const [row] = oneIndex([], [guided], [], german());
      expect(row!.procedure[1]).toBe("Wirf das Pulver hinein und tritt zurück");
      expect(catalogEntryMatches(row!, "Wirf das Pulver")).toBe(true);
      expect(catalogEntryMatches(row!, "steigt den Hals hinauf")).toBe(true);
      expect(catalogEntryMatches(row!, "nicht den Geruch")).toBe(true);
      // Still reachable by the canonical English in a German session.
      expect(catalogEntryMatches(row!, "drop the powder in")).toBe(true);
    });

    it("is found by the safety line it prints, in both languages", () => {
      const [en] = oneIndex([], [guided], [], english());
      expect(catalogEntryMatches(en!, "stings a cut")).toBe(true);
      expect(catalogEntryMatches(en!, "rinse a splash")).toBe(true);
      const [de] = oneIndex([], [guided], [], german());
      expect(de!.safetyRationale).toBe("Essig mit fünf Prozent brennt in einer Wunde, verätzt aber nicht");
      expect(catalogEntryMatches(de!, "brennt in einer Wunde")).toBe(true);
      expect(catalogEntryMatches(de!, "Spritzer abspulen")).toBe(true);
      expect(catalogEntryMatches(de!, "stings a cut")).toBe(true);
    });
  });

  describe("a reviewed corpus question", () => {
    const refusal = question({
      id: "aq-500",
      question: "Does the glaze on this mug crack when it cools too fast?",
      script: [],
      support: "boundary",
      reason_code: "unsupported-fracture-mechanics",
      material_class: "fired-ceramic",
      tags: ["thermal_shock"],
    });

    it("is found by the reason it is refused, in English", () => {
      const [row] = oneIndex([], [], [refusal], english());
      expect(row!.title).toBe("Does the glaze on this mug crack when it cools too fast?");
      expect(catalogEntryMatches(row!, "fracture mechanics")).toBe(true);
      expect(catalogEntryMatches(row!, "fired ceramic")).toBe(true);
      expect(catalogEntryMatches(row!, "thermal shock")).toBe(true);
    });

    it("is found by the German refusal the card actually draws", () => {
      const [row] = oneIndex([], [], [{
        ...refusal,
        question_de: "Springt die Glasur an dieser Tasse, wenn sie zu schnell abkühlt?",
      } as CapabilityPrompt], german({
        "unsupported fracture mechanics": "Bruchmechanik wird nicht unterstützt",
      }));
      expect(row!.reason).toBe("unsupported fracture mechanics");
      expect(catalogEntryMatches(row!, "Bruchmechanik")).toBe(true);
      expect(catalogEntryMatches(row!, "Glasur")).toBe(true);
      // The machine code and the English words stay reachable.
      expect(catalogEntryMatches(row!, "fracture mechanics")).toBe(true);
      expect(catalogEntryMatches(row!, "aq-500")).toBe(true);
    });
  });

  /**
   * The same contract over the payload the browser is actually served.
   *
   * The checked-in guided file is English only; `tools/kids-catalog.py`
   * merges `experiments-de-v1.json` into it at build time, which is where
   * the German procedure, observation and safety prose comes from. A
   * fixture cannot see the hole that merge opens, so this walks it.
   */
  describe("over the built payload", () => {
    const englishRows = (JSON.parse(kidsCatalogJson) as { experiments: KidsExperiment[] }).experiments;
    const germanRows = (JSON.parse(kidsGermanJson) as { experiments: Record<string, unknown>[] }).experiments;
    const germanById = new Map(germanRows.map((row) => [row.id as string, row]));
    // The merge the build tool performs, in the same shape.
    const merged: KidsExperiment[] = englishRows.map((row) => {
      const twin = germanById.get(row.id) ?? {};
      const withGerman: Record<string, unknown> = { ...row };
      for (const field of [
        "title", "phenomenon", "boundary", "safety_rationale", "safety_guidance",
        "procedure", "observations", "recipe",
      ]) {
        if (row[field as keyof KidsExperiment] !== undefined && twin[field] !== undefined) {
          withGerman[`${field}_de`] = twin[field];
        }
      }
      return withGerman as unknown as KidsExperiment;
    });

    const rows = oneIndex([], merged, [], context({ locale: "de" }) as Parameters<typeof oneIndex>[3]);
    const byId = new Map(rows.map((row) => [row.id, row]));

    /** The longest word of a sentence: distinctive, and never a stop word. */
    const longestWord = (text: string) =>
      text.split(/[^\p{L}\p{N}]+/u).sort((a, b) => b.length - a.length)[0] ?? "";

    it("finds every German safety rationale a card prints", () => {
      const withRationale = merged.filter((row) => row.safety_rationale_de);
      expect(withRationale.length).toBeGreaterThan(50);
      for (const row of withRationale) {
        const needle = longestWord(row.safety_rationale_de!);
        expect(catalogEntryMatches(byId.get(row.id)!, needle), `${row.id}: ${needle}`).toBe(true);
      }
    });

    it("finds every German procedure and observation line a card ships", () => {
      const withProcedure = merged.filter((row) => row.procedure_de?.length);
      expect(withProcedure.length).toBeGreaterThan(20);
      for (const row of withProcedure) {
        for (const line of row.procedure_de!) {
          const needle = longestWord(line);
          expect(catalogEntryMatches(byId.get(row.id)!, needle), `${row.id}: ${needle}`).toBe(true);
        }
      }
      for (const row of merged.filter((candidate) => candidate.observations_de?.length)) {
        for (const line of row.observations_de!) {
          const needle = longestWord(line);
          expect(catalogEntryMatches(byId.get(row.id)!, needle), `${row.id}: ${needle}`).toBe(true);
        }
      }
    });
  });
});
