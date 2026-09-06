/**
 * The concept map's links, walked against the shipped catalogues.
 *
 * Two halves, and both matter. The fixtures pin the RULES — which relation
 * each kind of affiliation produces, and what it refuses to claim. The
 * walk over the real codex export and the real kids catalogue pins the
 * COVERAGE: a join that is correct on four hand-made rows and empty on the
 * shipped content is a feature nobody can reach, and that is exactly the
 * defect this file exists to prevent coming back.
 */
import { describe, expect, it } from "vitest";
import { conceptLinks, relationLabel } from "./conceptLinks";
import { hasGermanTranslation } from "./i18n.svelte";
import { parseCodexIndex, type CodexEntry } from "./codex";
import type { KidsExperiment } from "./kidsCatalog";
import type { MissionSummary } from "./storyProgress";
import codexExportJson from "../../../../crates/kerotakis-codex/tests/golden/codex-export.json?raw";
import kidsCatalogJson from "../../../../data/kids/experiments-v1.json?raw";

const entry = (
  id: string,
  concepts: string[],
  script: string,
  events: string[] = [],
): CodexEntry => ({ id, concepts, setup: { script }, expect: { events }, registers: {} });

const entries: CodexEntry[] = [
  entry("silver-chloride-precipitation", ["precipitation"], "add v1 water 100mL\nadd v1 AgNO3 1mmol\nadd v1 NaCl 1mmol\n", ["precipitated:AgCl"]),
  entry("pure-arithmetic", ["precipitation"], "# no reagents at all\n"),
  entry("strong-base", ["bases"], "add v1 water 1000mL\nadd v1 NaOH 1mmol\n", ["dissolved:NaOH"]),
];

const kids: KidsExperiment[] = [
  { id: "K01", title: "Cloudy water", phenomenon: "A solid appears", status: "computed", progress: "starter", topics: [], ingredients: [], apparatus: [], codex: ["silver-chloride-precipitation"], lesson: "silver-and-salt.lab", safety: "home" },
  { id: "K02", title: "Something else", phenomenon: "Unrelated", status: "computed", progress: "starter", topics: [], ingredients: [], apparatus: [], safety: "home" },
];

const missions: MissionSummary[] = [
  { file: "silver-and-salt.lab", name: "silver and salt", topic: "start here", kit: ["water", "AgNO3", "NaCl"] },
  { file: "never-mix.lab", name: "never mix", topic: "safety", kit: ["bleach", "vinegar"] },
  { file: "no-kit.lab", name: "no kit", topic: "more" },
];

describe("a concept leads somewhere", () => {
  it("offers the entries that declare it, and says that is what they do", () => {
    const links = conceptLinks("precipitation", { entries });
    expect(links.map((link) => [link.kind, link.id, link.relation])).toEqual([
      ["experiment", "silver-chloride-precipitation", "teaches"],
      ["experiment", "pure-arithmetic", "teaches"],
    ]);
  });

  it("follows a kids task's reviewed cross-reference, not a word it shares", () => {
    const links = conceptLinks("precipitation", { entries, kids });
    expect(links.filter((link) => link.kind === "kids").map((link) => link.id)).toEqual(["K01"]);
    // K02 names no codex entry, so nothing links it — a title that happened
    // to contain "precipitation" must not be enough.
    expect(conceptLinks("bases", { entries, kids }).some((link) => link.kind === "kids")).toBe(false);
  });

  it("calls a mission's shared typed event evidence, and a shared shelf materials", () => {
    const links = conceptLinks("precipitation", { entries, missions });
    // `silver-and-salt` has an outcome contract securing `precipitated:AgCl`,
    // which the teaching entry also claims — the stronger of the two names.
    expect(links.find((link) => link.id === "silver-and-salt.lab")?.relation).toBe("evidence");
    expect(
      conceptLinks("bases", {
        entries,
        missions: [{ file: "soap.lab", name: "soap", kit: ["water", "NaOH", "oil"] }],
      }).find((link) => link.kind === "mission")?.relation,
    ).toBe("materials");
  });

  it("refuses the two ways a materials link could be vacuous", () => {
    // An entry with no reagents is a subset of every kit; a mission with no
    // kit is a superset of nothing. Neither may produce a link.
    expect(conceptLinks("precipitation", { entries: [entries[1]!], missions }).some((link) => link.kind === "mission")).toBe(false);
    expect(conceptLinks("precipitation", { entries, missions: [missions[2]!] }).some((link) => link.kind === "mission")).toBe(false);
    // `never-mix` shares no reagent with the teaching entry.
    expect(conceptLinks("precipitation", { entries, missions }).some((link) => link.id === "never-mix.lab")).toBe(false);
  });

  it("marks progress from the two sets that record it, and nothing else", () => {
    const done = conceptLinks("precipitation", {
      entries,
      kids,
      missions,
      completedExperiments: new Set(["silver-chloride-precipitation"]),
      completedMissions: new Set(["silver-and-salt"]),
    });
    expect(done.find((link) => link.id === "silver-chloride-precipitation")?.done).toBe(true);
    expect(done.find((link) => link.id === "pure-arithmetic")?.done).toBe(false);
    // The kids task needs both its linked entry AND its guided lesson.
    expect(done.find((link) => link.id === "K01")?.done).toBe(true);
    expect(done.find((link) => link.id === "silver-and-salt.lab")?.done).toBe(true);
  });

  it("returns nothing for a concept no entry declares, so the panel can say so", () => {
    expect(conceptLinks("nobody-teaches-this", { entries, kids, missions })).toEqual([]);
  });
});

describe("the join reaches the shipped content", () => {
  const shipped = parseCodexIndex(JSON.parse(codexExportJson));
  // The authored source, not the merged payload: `parseKidsCatalog` demands
  // the `_de` fields that `tools/kids-catalog.py` folds in at build time,
  // and the join reads only `id`, `codex` and `lesson` — the three fields
  // this file authors and the ones a broken cross-reference would break.
  const shippedKids = (JSON.parse(kidsCatalogJson) as { experiments: KidsExperiment[] }).experiments;
  const concepts = [...new Set(shipped.flatMap((e) => e.concepts ?? []))];

  it("finds the catalogues, so the walk is not vacuous", () => {
    expect(shipped.length).toBeGreaterThanOrEqual(105);
    expect(shippedKids.length).toBeGreaterThanOrEqual(60);
    expect(concepts.length).toBeGreaterThanOrEqual(153);
  });

  it("gives every shipped concept at least one activity to open", () => {
    const orphans = concepts.filter(
      (concept) => conceptLinks(concept, { entries: shipped, kids: shippedKids }).length === 0,
    );
    expect(orphans).toEqual([]);
  });

  it("reaches kids tasks from the concepts their reviewed entries teach", () => {
    const reached = new Set(
      concepts.flatMap((concept) =>
        conceptLinks(concept, { entries: shipped, kids: shippedKids })
          .filter((link) => link.kind === "kids")
          .map((link) => link.id),
      ),
    );
    // Eleven kids tasks carry a `codex` cross-reference today; every one of
    // them must be reachable from at least one concept.
    expect(reached.size).toBeGreaterThanOrEqual(11);
  });
});

describe("every reason a link can give is a translated sentence", () => {
  it("has German for all three relations", () => {
    const untranslated = (["teaches", "evidence", "materials"] as const)
      .map(relationLabel)
      .filter((key) => !hasGermanTranslation(key));
    expect(untranslated).toEqual([]);
  });
});
