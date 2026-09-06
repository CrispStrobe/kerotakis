/**
 * ONE entry model for the whole catalogue.
 *
 * The catalogue used to be two catalogues wearing one component. Both
 * sources opened the same dialog and ran through the same runner, but they
 * were still drawn as two tiers: one a list of equation rows, the other a
 * grid of wordy cards, each with its own filters, its own idea of what a
 * card offers, and a name for the second tier that told a learner it was
 * for younger children. A learner therefore had to know which of the two
 * halves of the library their experiment lived in before they could look
 * for it, and the answer was an accident of which file the content shipped
 * in.
 *
 * So the two shapes are mapped into one view model here, and the component
 * draws exactly one kind of card. Where one source has a field the other
 * lacks, the missing value is DERIVED from what the entry actually says
 * rather than left as a hole the card has to hide:
 *
 *   - level and age come from the curriculum placements where an entry has
 *     them, and from whether it needs supervision where it does not;
 *   - duration comes from the number of lines the bench will actually run
 *     (or, for an entry whose procedure is a guided lesson, from how much
 *     it puts on the bench), at one pace shared by both;
 *   - topics come from a single coarse vocabulary that BOTH sources map
 *     into, so one topic chip means the same thing wherever the entry came
 *     from. The fine-grained codex concepts survive underneath it, for
 *     search and for the entry panel.
 *
 * The point of the derivation is that a filter must be answerable for
 * every one of the ~167 entries. A filter that only half the library can
 * answer re-creates the tier split inside the filter bar.
 *
 * Nothing here renders. It takes a translator and a locale so the strings
 * it builds are the ones the reader will see (and therefore search), and
 * is otherwise pure.
 */
import { runnableLines } from "./catalogRunner";
import { normalizeCatalogText, type ExperimentProgressFilter } from "./catalogSearch";
import { scriptKit, type CodexEntry } from "./codex";
import { guidedLearningLabel, kidsText, type KidsExperiment, type KidsSafety, type KidsStatus } from "./kidsCatalog";
import { kidsShelfKeys } from "./kidsSandbox";

/** Which corpus an entry came from. An INTERNAL identifier: never displayed. */
export type CatalogSourceKind = "codex" | "guided";

/** How far in a learner is, as a band rather than a tier. */
export type CatalogLevel = "starter" | "intermediate" | "advanced";

export const CATALOG_LEVELS: readonly CatalogLevel[] = ["starter", "intermediate", "advanced"];

/** Roughly how long it takes, as a band the filter can offer. */
export type CatalogDurationBand = "short" | "medium" | "long";

export const CATALOG_DURATIONS: readonly CatalogDurationBand[] = ["short", "medium", "long"];

/**
 * What "do this one" means for this entry.
 *
 * Every entry has one, so the card always has a primary action and the
 * model never hands the component a hole to paper over. Which one it is
 * follows the CONTENT — an entry with a script runs, an entry whose
 * procedure was written as a guided lesson opens that lesson — never which
 * corpus it shipped in.
 */
export type CatalogRunTarget =
  | { kind: "script"; entry: CodexEntry }
  | { kind: "lesson"; file: string }
  | { kind: "quest"; id: string }
  | { kind: "sandbox" }
  /** Documented model boundary: the honest answer is to read it. */
  | { kind: "boundary" };

export interface CatalogEntry {
  /** Stable id: the codex slug, or the guided task's own id. */
  id: string;
  source: CatalogSourceKind;
  /** Localized display title. Never empty. */
  title: string;
  /** One line on what happens. Never empty. */
  hook: string;
  level: CatalogLevel;
  /** The youngest age any source claims for it. */
  ageMin: number;
  /** Lines the bench would run, or the work an entry puts on it. */
  steps: number;
  minutes: number;
  duration: CatalogDurationBand;
  /** Shelf keys the entry needs, as the shelf spells them. */
  needs: string[];
  apparatus: string[];
  /** The shared coarse vocabulary. At least one, always. */
  topics: string[];
  /** The codex's fine-grained concepts, where the entry names any. */
  concepts: string[];
  /** Curriculum placements, as `system` / `stage` pairs. */
  placements: { system: string; stage: string }[];
  /** What the bench is expected to show, in the engine's own tags. */
  expectations: string[];
  run: CatalogRunTarget;
  /** The codex entry that can be opened and run, when there is one. */
  script: CodexEntry | null;
  lesson: string | null;
  quest: string | null;
  capabilities: string[];
  /** Codex entries this one cross-references, by id. */
  codexLinks: string[];
  /** Localized statement of where the model stops, when the entry has one. */
  boundary: string | null;
  equation: string | null;
  status: KidsStatus;
  safety: KidsSafety;
  /** The guided task behind this entry, for the sandbox hand-over. */
  guided: KidsExperiment | null;
  done: boolean;
  /** Everything it needs is on the learner's shelf right now. */
  onShelf: boolean;
  /** Every string worth matching a query against, localized and canonical. */
  search: string[];
}

// ── Level ─────────────────────────────────────────────────────────────

/**
 * Where the three levels divide, and why there.
 *
 * The boundaries come from the curriculum placements the content carries,
 * which are stated in school years; the learner sees only the level name
 * these produce.
 *
 * Twelve is where the two corpora actually meet: the youngest curriculum
 * placement in the codex is KS3 at eleven, and the guided tasks that need
 * no supervision are written for around eight. Fifteen is where the German
 * placements start naming the upper secondary stages. So the bands are the
 * content's own, not three equal thirds of it.
 */
export function levelForAge(ageMin: number): CatalogLevel {
  if (ageMin < 12) return "starter";
  return ageMin < 15 ? "intermediate" : "advanced";
}

/**
 * The label a level wears.
 *
 * A NAME, and only a name. The card used to print the age band beside it
 * ("ab 8 Jahren"), and that is the one thing this catalogue must not say:
 * every person is addressed here, an adult is welcome in "first steps",
 * and an age band beside a title reads as a permission slip ("not for
 * me"). The bands below are still how the CONTENT is sorted — they are
 * the curriculum's own placements — but they are an implementation
 * detail of the ordering, never a label a learner is shown.
 */
export function levelLabel(level: CatalogLevel): string {
  if (level === "starter") return "first steps";
  return level === "intermediate" ? "going further" : "in depth";
}

/** The youngest age a codex entry is placed at, or a schoolroom default. */
export function codexAgeMin(entry: Pick<CodexEntry, "curriculum">): number {
  const ages = (entry.curriculum ?? [])
    .map((placement) => placement.ages?.min)
    .filter((age): age is number => typeof age === "number");
  return ages.length === 0 ? 12 : Math.min(...ages);
}

// ── Duration ──────────────────────────────────────────────────────────

/**
 * One pace for both corpora.
 *
 * The runner already walks a script at a fixed pace with a beat between
 * steps, and a learner reads the account of each step; a minute and a half
 * a step is the honest arithmetic of that, rounded to something a card can
 * say. An entry the bench does not run has no lines to count, so its work
 * is counted instead — the materials and instruments it asks for, which is
 * what its lesson will walk the learner through.
 */
export const MINUTES_PER_STEP = 1.5;

export function minutesForSteps(steps: number): number {
  return Math.min(60, Math.max(2, Math.round(steps * MINUTES_PER_STEP)));
}

export function durationBand(minutes: number): CatalogDurationBand {
  if (minutes <= 5) return "short";
  return minutes <= 15 ? "medium" : "long";
}

export function durationLabel(band: CatalogDurationBand): string {
  if (band === "short") return "under 5 minutes";
  return band === "medium" ? "5 to 15 minutes" : "over 15 minutes";
}

// ── Topics ────────────────────────────────────────────────────────────

/**
 * The shared coarse vocabulary.
 *
 * Thirteen buckets, because the filter is a chip rail a thumb scrolls and
 * not a taxonomy: the guided corpus named twenty-eight topics and the
 * codex a hundred and fifty-seven concepts, which as one list is a
 * scrolling wall that answers no question. Both sources fold into these,
 * so "acids and bases" selects across the whole library rather than across
 * whichever half used that word.
 */
export const CATALOG_TOPICS = [
  "acids",
  "gases",
  "heat",
  "rates",
  "redox",
  "solutions",
  "crystals",
  "separations",
  "materials",
  "measurement",
  "food",
  "colour",
  "boundaries",
] as const;

export type CatalogTopic = (typeof CATALOG_TOPICS)[number];

export function topicLabel(topic: string): string {
  return TOPIC_LABELS[topic] ?? topic;
}

const TOPIC_LABELS: Record<string, string> = {
  acids: "acids and bases",
  gases: "gases and pressure",
  heat: "heat and energy",
  rates: "reaction rates",
  redox: "metals and electricity",
  solutions: "solutions and density",
  crystals: "crystals and precipitates",
  separations: "mixtures and separations",
  materials: "materials and surfaces",
  measurement: "measuring and testing",
  food: "food and the kitchen",
  colour: "colour and light",
  boundaries: "where the model stops",
};

/** The guided corpus's own topic words, folded into the shared vocabulary. */
const GUIDED_TOPICS: Record<string, CatalogTopic> = {
  acids: "acids",
  indicators: "acids",
  gases: "gases",
  pressure: "gases",
  heat: "heat",
  fire: "heat",
  rates: "rates",
  enzymes: "rates",
  redox: "redox",
  electricity: "redox",
  corrosion: "redox",
  metals: "redox",
  solutions: "solutions",
  water: "solutions",
  density: "solutions",
  crystals: "crystals",
  separations: "separations",
  mixtures: "separations",
  materials: "materials",
  polymers: "materials",
  surfaces: "materials",
  motion: "materials",
  measurement: "measurement",
  tests: "measurement",
  food: "food",
  health: "food",
  colour: "colour",
  light: "colour",
};

/**
 * Codex concepts, folded into the same vocabulary.
 *
 * Read in order, and every match counts: an entry that teaches both
 * `enthalpy-of-solution` and `freezing-point-depression` is honestly about
 * heat AND solutions, and hiding one of those from the filter would make
 * the chip lie about what it selects. An entry no rule recognises is not
 * dropped — it falls back to what the bench does with it.
 */
const CONCEPT_TOPICS: { topic: CatalogTopic; match: RegExp }[] = [
  { topic: "acids", match: /acid|base|alkal|(^|-)ph(-|$)|ph-calculation|buffer|proton|neutralis|pka|titration|equivalence|amphiprotic|autoprotolysis|conjugate|hydrolysis|polyprotic/ },
  { topic: "gases", match: /gas|combustion|open-system/ },
  { topic: "heat", match: /enthalpy|exotherm|endotherm|calorimetry|heat-capacity|hess|adiabatic|thermal|temperature|state-functions|freezing-point|combustion/ },
  { topic: "rates", match: /rate|kinetic|catalys|arrhenius|activation-energy|half-life|reaction-order|surface-area|shared-clock/ },
  { topic: "redox", match: /redox|oxidation|electron-transfer|activity-series|displacement|galvanic|nernst|electroly|faraday|sacrificial|potential|accumulators|hydrogen-as-a-rung|concentration-cells|corrosion|metal/ },
  { topic: "solutions", match: /solubility|dissolv|dilution|concentration|concentrated|ionic-strength|colligative|solutions|activity|common-ion|miscibility|hard-water|water-softening|permanent-hardness|descaling|degree-of-dissociation/ },
  { topic: "crystals", match: /crystallis|hydrate|waters-of-crystallisation|saturation|precipitat|insoluble|sparingly|ksp|solubility-product|calcination/ },
  { topic: "separations", match: /filtration|separation|evaporation|recovery|mixtures|spectator|mass-transfer/ },
  { topic: "materials", match: /material|surface|household-hazards/ },
  { topic: "measurement", match: /mass-accounting|conservation-of-mass|molar-mass|stoichiometric|limiting-reagent|excess-reagent|yield|amount-of-substance|element-budget|detection-limit|instrument-floor|before-and-after-measurement|fair-test|controlled-comparison|balanced-equations|charge-conservation|percentage|ionic-equations|test|negative-result|measurement/ },
  { topic: "colour", match: /colour|atomic-emission|indicator|flame/ },
  { topic: "boundaries", match: /model-boundar|stated-ignorance|unrendered-observable|species-without-a-database|kinetic-barrier|thermodynamic-versus-kinetic|kinetic-versus-thermodynamic|decomposition-threshold|computed-decomposition/ },
];

/**
 * The vocabulary an entry's own words never reach.
 *
 * Every entry must answer the topic filter, so an entry whose concepts no
 * rule recognised (or which names no concepts at all — the export does not
 * promise them) is placed by what its script actually does on the bench.
 * That is a weaker signal than a concept, which is exactly why it is the
 * fallback and not the rule.
 */
const VERB_TOPICS: { topic: CatalogTopic; match: RegExp }[] = [
  { topic: "measurement", match: /^(measure|balance|weigh)\b/ },
  { topic: "separations", match: /^(filter|decant|drain|distil|centrifuge|magnet|chromatograph)\b/ },
  { topic: "redox", match: /^(electrolyse|cell)\b/ },
  { topic: "heat", match: /^(heat|cool|bunsen|ignite)\b/ },
  { topic: "crystals", match: /^evaporate\b/ },
  { topic: "gases", match: /^(seal|sweep|regulate)\b/ },
];

function codexTopics(entry: Pick<CodexEntry, "concepts" | "setup" | "expect">): string[] {
  const words = (entry.concepts ?? []).join(" ");
  const found = new Set<string>();
  for (const rule of CONCEPT_TOPICS) {
    if (rule.match.test(words)) found.add(rule.topic);
  }
  if (found.size === 0) {
    const lines = runnableLines(entry.setup.script);
    for (const rule of VERB_TOPICS) {
      if (lines.some((line) => rule.match.test(line))) found.add(rule.topic);
    }
  }
  // Still nothing recognised: it is a reaction on a bench, which is the
  // one thing every entry in this library has in common.
  if (found.size === 0) found.add("solutions");
  return [...found].sort();
}

function guidedTopics(entry: Pick<KidsExperiment, "topics" | "boundary">): string[] {
  const found = new Set<string>();
  for (const topic of entry.topics) {
    const coarse = GUIDED_TOPICS[topic];
    if (coarse) found.add(coarse);
  }
  if (entry.boundary) found.add("boundaries");
  if (found.size === 0) found.add("solutions");
  return [...found].sort();
}

// ── Building the model ────────────────────────────────────────────────

export interface CatalogViewContext {
  locale: string;
  /** The shell's `t()`. Titles are searched in the language they are read. */
  translate: (value: string) => string;
  /** Codex ids whose run checked out, the one progress record. */
  completed: ReadonlySet<string>;
  /** Guided lessons the learner has finished, by mission id. */
  completedMissions?: ReadonlySet<string>;
  /** Shelf keys the learner can reach right now. */
  shelfKeys?: ReadonlySet<string>;
}

/** A slug as a card says it: hyphens are not a word separator on screen. */
export function slugWords(value: string): string {
  return value.replaceAll("-", " ").replaceAll("_", " ");
}

function expectations(entry: Pick<CodexEntry, "expect">): string[] {
  const expect = entry.expect ?? {};
  const wanted = [...(expect.events ?? [])];
  if (expect.ph) wanted.push("ph");
  if (expect.temperature_c) wanted.push("temperature");
  return wanted;
}

function onShelf(needs: readonly string[], shelfKeys: ReadonlySet<string> | undefined): boolean {
  if (!shelfKeys || shelfKeys.size === 0) return false;
  return needs.length > 0 && needs.every((key) => shelfKeys.has(key));
}

function fromCodex(entry: CodexEntry, context: CatalogViewContext): CatalogEntry {
  const words = slugWords(entry.id);
  const title = context.translate(words);
  const ageMin = codexAgeMin(entry);
  const steps = runnableLines(entry.setup.script).length;
  const minutes = minutesForSteps(steps);
  const needs = scriptKit(entry.setup.script);
  const apparatus = entry.apparatus ?? [];
  const summary = context.locale === "de" ? (entry.summary_de ?? entry.summary) : entry.summary;
  // Order matters: the equation is the shortest true sentence about the
  // entry, and a card with no hook at all is the hole this model exists to
  // refuse. The title is the last resort, never an empty string.
  const hook = summary?.trim() || entry.equation?.trim() || title;
  return {
    id: entry.id,
    source: "codex",
    title: title || words,
    hook,
    level: levelForAge(ageMin),
    ageMin,
    steps,
    minutes,
    duration: durationBand(minutes),
    needs,
    apparatus,
    topics: codexTopics(entry),
    concepts: entry.concepts ?? [],
    placements: (entry.curriculum ?? []).map((p) => ({ system: p.system, stage: p.stage })),
    expectations: expectations(entry),
    run: { kind: "script", entry },
    script: entry,
    lesson: null,
    quest: null,
    capabilities: [],
    codexLinks: [],
    boundary: null,
    equation: entry.equation ?? null,
    status: "computed",
    safety: ageMin < 12 ? "home" : "school",
    guided: null,
    done: context.completed.has(entry.id),
    onShelf: onShelf(needs, context.shelfKeys),
    search: [
      entry.id,
      words,
      title,
      hook,
      entry.equation ?? "",
      entry.summary ?? "",
      ...(entry.concepts ?? []),
      ...(entry.models ?? []),
      ...apparatus,
      ...needs,
      ...Object.values(entry.registers ?? {}),
    ],
  };
}

function fromGuided(
  entry: KidsExperiment,
  byId: ReadonlyMap<string, CodexEntry>,
  context: CatalogViewContext,
): CatalogEntry {
  const title = kidsText(entry, "title", context.locale);
  const hook = kidsText(entry, "phenomenon", context.locale);
  const boundary = entry.boundary ? kidsText(entry, "boundary", context.locale) : null;
  // Supervision is the only age signal the guided corpus carries, and it
  // is a real one: a task written to be done at a kitchen table is a task
  // written for the youngest readers in the library.
  const ageMin = entry.safety === "home" ? 8 : 12;
  const script = (entry.codex ?? []).map((id) => byId.get(id)).find((found) => found != null) ?? null;
  const needs = kidsShelfKeys(entry.ingredients);
  const steps = script
    ? runnableLines(script.setup.script).length
    : Math.max(2, entry.ingredients.length + entry.apparatus.length);
  const minutes = minutesForSteps(steps);
  const codexLinks = (entry.codex ?? []).filter((id) => byId.has(id));
  const lessonId = entry.lesson?.replace(/\.lab$/, "") ?? null;
  const done = codexLinks.length > 0
    ? codexLinks.every((id) => context.completed.has(id))
    : lessonId !== null && (context.completedMissions?.has(lessonId) ?? false);
  return {
    id: entry.id,
    source: "guided",
    title: title || entry.title,
    hook: hook || entry.phenomenon,
    level: levelForAge(ageMin),
    ageMin,
    steps,
    minutes,
    duration: durationBand(minutes),
    needs,
    apparatus: entry.apparatus,
    topics: guidedTopics(entry),
    concepts: script?.concepts ?? [],
    placements: (script?.curriculum ?? []).map((p) => ({ system: p.system, stage: p.stage })),
    expectations: script ? expectations(script) : [],
    run: runTargetFor(entry, script),
    script,
    lesson: entry.lesson ?? null,
    quest: entry.quest ?? null,
    capabilities: entry.capabilities ?? [],
    codexLinks,
    boundary,
    equation: script?.equation ?? null,
    status: entry.status,
    safety: entry.safety,
    guided: entry,
    done,
    onShelf: onShelf(needs, context.shelfKeys),
    search: [
      entry.id,
      title,
      hook,
      boundary ?? "",
      entry.title,
      entry.phenomenon,
      entry.boundary ?? "",
      ...entry.topics,
      ...entry.ingredients,
      ...needs,
      ...entry.apparatus,
      ...(entry.capabilities ?? []),
      ...(entry.codex ?? []),
    ],
  };
}

function runTargetFor(entry: KidsExperiment, script: CodexEntry | null): CatalogRunTarget {
  if (script) return { kind: "script", entry: script };
  if (entry.lesson) return { kind: "lesson", file: entry.lesson };
  if (entry.quest) return { kind: "quest", id: entry.quest };
  if (entry.status === "computed" || entry.status === "partial") return { kind: "sandbox" };
  return { kind: "boundary" };
}

/** What the primary button on a card says, for each kind of action. */
export function runTargetLabel(target: CatalogRunTarget, done: boolean): string {
  switch (target.kind) {
    case "script":
      return "run it on the bench";
    case "lesson":
      // The same two words the card's secondary lesson button uses, so one
      // action never reads as two different offers.
      return guidedLearningLabel(done);
    case "quest":
      return "start quest";
    case "sandbox":
      return "explore in Sandbox";
    default:
      return "read the documented boundary";
  }
}

/**
 * Both corpora as one list.
 *
 * Ordered by level and then by the title the reader sees, so the list a
 * German reader scrolls is alphabetical in German. Sorting on the id would
 * be stable but would look shuffled to everyone.
 */
export function catalogEntries(
  codex: readonly CodexEntry[],
  guided: readonly KidsExperiment[],
  context: CatalogViewContext,
): CatalogEntry[] {
  const byId = new Map(codex.map((entry) => [entry.id, entry]));
  const rank = (level: CatalogLevel) => CATALOG_LEVELS.indexOf(level);
  return [
    ...codex.map((entry) => fromCodex(entry, context)),
    ...guided.map((entry) => fromGuided(entry, byId, context)),
  ].sort((a, b) =>
    rank(a.level) - rank(b.level)
    || a.title.localeCompare(b.title, context.locale)
    || a.id.localeCompare(b.id, context.locale),
  );
}

// ── Filters ───────────────────────────────────────────────────────────

/** The rail asks the same question the per-record predicate answers. */
export type CatalogProgressFilter = ExperimentProgressFilter;

export interface CatalogFilters {
  level: CatalogLevel | null;
  topic: string | null;
  duration: CatalogDurationBand | null;
  /** Only entries whose materials are all on the shelf right now. */
  shelfOnly: boolean;
  progress: CatalogProgressFilter;
  /** `system` and `stage` joined by a tab, which no stage name contains. */
  curriculum: string | null;
  concept: string | null;
  query: string;
}

export const NO_CATALOG_FILTERS: CatalogFilters = {
  level: null,
  topic: null,
  duration: null,
  shelfOnly: false,
  progress: "all",
  curriculum: null,
  concept: null,
  query: "",
};

export const PLACEMENT_SEPARATOR = "\t";

export function placementKey(placement: { system: string; stage: string }): string {
  return `${placement.system}${PLACEMENT_SEPARATOR}${placement.stage}`;
}

/** Does this entry answer everything the rail is currently asking? */
export function catalogEntryPasses(entry: CatalogEntry, filters: CatalogFilters): boolean {
  if (filters.level && entry.level !== filters.level) return false;
  // i18n-ok: topic, level and duration are wire keys chosen from a chip,
  // never text a reader typed.
  if (filters.topic && !entry.topics.includes(filters.topic)) return false;
  if (filters.duration && entry.duration !== filters.duration) return false;
  if (filters.shelfOnly && !entry.onShelf) return false;
  if (filters.progress === "completed" && !entry.done) return false;
  if (filters.progress === "not-tried" && entry.done) return false;
  if (filters.concept && !entry.concepts.includes(filters.concept)) return false;
  if (filters.curriculum
    && !entry.placements.some((placement) => placementKey(placement) === filters.curriculum)) {
    return false;
  }
  return catalogEntryMatches(entry, filters.query);
}

/**
 * Free text against everything the entry is, in both languages.
 *
 * `search` already holds the localized strings the card renders as well as
 * the canonical ones underneath, so typing "Säure" and typing "acid" both
 * find the same entry whichever language the interface is in.
 */
export function catalogEntryMatches(entry: Pick<CatalogEntry, "search">, query: string): boolean {
  const needle = normalizeCatalogText(query.trim().replaceAll("_", " ").replaceAll("-", " "));
  if (!needle) return true;
  return entry.search.some((value) =>
    normalizeCatalogText(slugWords(value)).includes(needle));
}

export function filterCatalogEntries(
  entries: readonly CatalogEntry[],
  filters: CatalogFilters,
): CatalogEntry[] {
  return entries.filter((entry) => catalogEntryPasses(entry, filters));
}

/** How many entries sit at each level, for the chip counts. */
export function levelCounts(entries: readonly CatalogEntry[]): Record<CatalogLevel, number> {
  const counts: Record<CatalogLevel, number> = { starter: 0, intermediate: 0, advanced: 0 };
  for (const entry of entries) counts[entry.level] += 1;
  return counts;
}

/** Topics actually present, so the rail never offers an empty chip. */
export function presentTopics(entries: readonly CatalogEntry[]): string[] {
  const found = new Set<string>();
  for (const entry of entries) for (const topic of entry.topics) found.add(topic);
  return CATALOG_TOPICS.filter((topic) => found.has(topic));
}

/** Curriculum placements actually present, ordered by system then stage. */
export function presentPlacements(
  entries: readonly CatalogEntry[],
  locale: string,
): { key: string; system: string; stage: string }[] {
  const found = new Map<string, { key: string; system: string; stage: string }>();
  for (const entry of entries) {
    for (const placement of entry.placements) {
      const key = placementKey(placement);
      if (!found.has(key)) found.set(key, { key, ...placement });
    }
  }
  return [...found.values()].sort((a, b) =>
    a.system.localeCompare(b.system, locale) || a.stage.localeCompare(b.stage, locale));
}
