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
 *   - Codex level comes from curriculum placement; guided level is authored
 *     directly as learning progress, never inferred from age or supervision;
 *   - duration comes from the number of lines the bench will actually run
 *     (or, for an entry whose procedure is a guided lesson, from how much
 *     it puts on the bench), at one pace shared by both;
 *   - topics come from a single coarse vocabulary that BOTH sources map
 *     into, so one topic chip means the same thing wherever the entry came
 *     from. The fine-grained codex concepts survive underneath it, for
 *     search and for the entry panel.
 *
 * The point of the derivation is that a filter must be answerable for
 * every one of the ~168 entries. A filter that only half the library can
 * answer re-creates the tier split inside the filter bar.
 *
 * Nothing here renders. It takes a translator and a locale so the strings
 * it builds are the ones the reader will see (and therefore search), and
 * is otherwise pure.
 */
import {
  capabilityReasonText,
  capabilityRunnable,
  capabilitySearchText,
  localiseCapability,
  type CapabilityPrompt,
  type CapabilitySupport,
} from "./capabilities";
import { runnableLines } from "./catalogRunner";
import { normalizeCatalogText, type ExperimentProgressFilter } from "./catalogSearch";
import { scriptKit, type CodexEntry } from "./codex";
import { guidedLearningLabel, kidsList, kidsRecipe, kidsText, type KidsExperiment, type KidsRecipeLine, type KidsSafety, type KidsStatus } from "./kidsCatalog";
import { kidsShelfKeys } from "./kidsSandbox";
import { KIDS_EQUIPMENT, type KidsEquipment } from "./kidsEquipment";
import type { CatalogItem } from "./host/EngineHost";
import { catalogIdForApparatus } from "./equipmentCatalogue";

/**
 * WHAT a row is, which is now the reader's question and not the author's.
 *
 * This was "an INTERNAL identifier: never displayed", because the only two
 * values were two files the same kind of thing shipped in. A third
 * population is not that: a reviewed corpus question is genuinely a
 * different kind of row from a runnable experiment, and pretending
 * otherwise would claim the bench can run five hundred experiments when
 * sixty of those questions are refusals with no script at all.
 *
 * So the discriminant became the FACET — one badge, one filter, one
 * separately-labelled count per kind — rather than a second door. It stays
 * one vocabulary because the alternative (a parallel result type beside
 * `CatalogEntry`) means a second search predicate over a second shape,
 * and a second predicate is precisely how the two doors happened.
 */
export type CatalogSourceKind = "codex" | "guided" | "capability";

export const CATALOG_SOURCES: readonly CatalogSourceKind[] = ["codex", "guided", "capability"];

/**
 * What to CALL a kind on a badge.
 *
 * The two experiment kinds read as siblings and the question does not,
 * because the difference a reader needs is "will this run" and not "which
 * corpus authored it".
 */
export function sourceLabel(source: CatalogSourceKind): string {
  if (source === "codex") return "bench experiment";
  return source === "guided" ? "guided experiment" : "answered question";
}

/**
 * The same three kinds as a chip that carries a count.
 *
 * A separate string rather than a suffix, because German does not pluralise
 * by appending an "s" and a chip reading "answered question 500" is a chip
 * that has to be read twice.
 */
export function sourceLabelPlural(source: CatalogSourceKind): string {
  if (source === "codex") return "bench experiments";
  return source === "guided" ? "guided experiments" : "answered questions";
}

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
  /** A corpus question whose reviewed script the bench will run. */
  | { kind: "question" }
  /**
   * A corpus question with NO run behind it — the answer is the reason.
   *
   * Separate from `boundary` because the two say different things: a
   * boundary is where a working model stops, while this is a question the
   * corpus reviewed and answered with a refusal (sixty of them, every one
   * shipping an empty script) or with "not modelled yet, owned by BRD-060"
   * (one). Both are answers. Neither is a button.
   */
  | { kind: "unanswered" }
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
  /** The corpus's own verdict on a question row; null for an experiment. */
  support: CapabilitySupport | null;
  /** The corpus prompt behind a question row, for the script and the run. */
  prompt: CapabilityPrompt | null;
  /**
   * Authored as suitable at EVERY level, so no level chip may hide it.
   *
   * Twenty-four corpus questions are banded `all` rather than to a school
   * age. `level` still holds a value (the floor, so the row sorts and the
   * chip has something to say), but a row with this set passes the level
   * filter whatever it is asking — placing an all-levels question under
   * one band would be a claim the corpus did not make.
   */
  anyLevel: boolean;
  /** Why a question is answered the way it is, as words for the dictionary. */
  reason: string | null;
  /** Guided safety classification; Codex entries make no invented claim. */
  safety: KidsSafety | null;
  /** Localized safety explanation and action, independent of progress. */
  safetyRationale: string | null;
  safetyGuidance: string | null;
  recipe: KidsRecipeLine[];
  procedure: string[];
  observations: string[];
  kits: KidsEquipment[];
  /** The guided task behind this entry, for the sandbox hand-over. */
  guided: KidsExperiment | null;
  done: boolean;
  /** Everything it needs is on the learner's shelf right now. */
  onShelf: boolean;
  /** Exact reagent ids absent from the currently reachable shelf. */
  missingNeeds: string[];
  /** Exact engine catalog answers for requirements it knows. */
  access: CatalogItem[];
  /** Shelf requirements plus engine-known apparatus refusals. */
  readyNow: boolean;
  /** False until the engine has answered; unknown is neither ready nor missing. */
  availabilityKnown: boolean;
  /** Every string worth matching a query against, localized and canonical. */
  search: string[];
}

// ── Level ─────────────────────────────────────────────────────────────

/**
 * The label a level wears.
 *
 * A NAME, and only a name. The card used to print the age band beside it
 * ("ab 8 Jahren"), and that is the one thing this catalogue must not say:
 * every person is addressed here, an adult is welcome in "first steps",
 * and an age band beside a title reads as a permission slip ("not for
 * me"). The bands below sort the authored learning progression. Curriculum
 * placement remains separately searchable metadata and never chooses a band.
 */
export function levelLabel(level: CatalogLevel): string {
  if (level === "starter") return "first steps";
  return level === "intermediate" ? "going further" : "in depth";
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

/**
 * The corpus's eight topics, folded into the same shared vocabulary.
 *
 * Authored, one line per corpus topic, because the corpus words are
 * coarser than the chips: `burn_and_oxidise` is honestly about heat AND
 * about metals-and-electricity, and `handle_and_inspect` is about
 * materials AND about measuring. A rule that picked one of those would
 * make the chip lie about what it selects, so both are recorded.
 *
 * Nothing is inferred from the topic string itself: the eight are a
 * closed set the exporter writes, and a ninth arriving unmapped falls
 * through to the tags, which run through the SAME `CONCEPT_TOPICS` the
 * codex does — so one topic chip means one thing across all three
 * populations.
 */
export const CORPUS_TOPICS: Record<string, CatalogTopic[]> = {
  acids_bases_and_gases: ["acids", "gases"],
  burn_and_oxidise: ["heat", "redox"],
  food_and_life: ["food"],
  handle_and_inspect: ["materials", "measurement"],
  heat_and_cool: ["heat"],
  materials: ["materials"],
  mix_and_dissolve: ["solutions"],
  separate: ["separations"],
};

function capabilityTopics(prompt: CapabilityPrompt): string[] {
  const found = new Set<string>(CORPUS_TOPICS[prompt.topic] ?? []);
  const vocabulary = [prompt.topic, prompt.material_class, ...prompt.tags].join(" ");
  for (const rule of CONCEPT_TOPICS) {
    if (rule.match.test(vocabulary)) found.add(rule.topic);
  }
  // A refusal and a not-yet-modelled question are both statements about
  // where this bench stops, which is a topic the catalogue already has.
  if (prompt.support === "boundary" || prompt.support === "missing") found.add("boundaries");
  if (found.size === 0) found.add("solutions");
  return [...found].sort();
}

/**
 * The corpus's school-age band, read as the catalogue's learning band.
 *
 * The corpus bands by school age and the catalogue never shows an age —
 * every person is addressed here, and an age beside a question reads as a
 * permission slip. The three bands already stand for the three learning
 * levels, which is the mapping the explorer's own `BAND_LABELS` table made
 * by hand; it lives here now so ONE vocabulary answers the level chip for
 * all three populations. `all` is not a band and gets `anyLevel` instead.
 */
export const CORPUS_BANDS: Record<string, CatalogLevel> = {
  age9_to12: "starter",
  age13_to15: "intermediate",
  age16_to18: "advanced",
};

/**
 * The corpus's support verdict, in the catalogue's coarser status words.
 *
 * `status` is the catalogue's own five-word vocabulary and a question row
 * still has to answer it. The corpus's word is kept VERBATIM in `support`
 * and is what the badge shows, so nothing here is what the reader is
 * told — this only feeds the `data-status` attribute and keeps the field
 * from being a hole. `curated` and `qualitative` become `partial` because
 * a reviewed route is not a computed one, and saying `computed` for them
 * would be the overclaim.
 */
const SUPPORT_STATUS: Record<CapabilitySupport, KidsStatus> = {
  computed: "computed",
  curated: "partial",
  qualitative: "partial",
  boundary: "boundary",
  missing: "unreachable",
};

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
  /** Engine-owned availability answers, when they have arrived. */
  catalog?: ReadonlyMap<string, CatalogItem>;
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

function availability(
  needs: readonly string[],
  apparatus: readonly string[],
  context: Pick<CatalogViewContext, "shelfKeys" | "catalog">,
): Pick<CatalogEntry, "missingNeeds" | "access" | "readyNow" | "availabilityKnown"> {
  const availabilityKnown = context.catalog !== undefined && context.catalog.size > 0;
  const missingNeeds = needs.filter((key) => !context.shelfKeys?.has(key));
  const access = [...new Set([...needs, ...apparatus.map(catalogIdForApparatus)])]
    .flatMap((id) => context.catalog?.get(id) ?? []);
  return {
    missingNeeds,
    access,
    availabilityKnown,
    readyNow: availabilityKnown && missingNeeds.length === 0 && access.every((item) => item.available),
  };
}

function fromCodex(entry: CodexEntry, context: CatalogViewContext): CatalogEntry {
  const words = slugWords(entry.id);
  const title = context.translate(words);
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
    level: entry.progress,
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
    support: null,
    prompt: null,
    anyLevel: false,
    reason: null,
    safety: null,
    safetyRationale: null,
    safetyGuidance: null,
    recipe: [],
    procedure: [],
    observations: [],
    kits: [],
    guided: null,
    done: context.completed.has(entry.id),
    onShelf: onShelf(needs, context.shelfKeys),
    ...availability(needs, apparatus, context),
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
  // Guided learning progress is authored. Supervision answers a different
  // question and must never move an experiment between learning bands.
  const script = (entry.codex ?? []).map((id) => byId.get(id)).find((found) => found != null) ?? null;
  const needs = kidsShelfKeys(entry.ingredients);
  const steps = script
    ? runnableLines(script.setup.script).length
    : Math.max(2, entry.ingredients.length + entry.apparatus.length);
  const minutes = minutesForSteps(steps);
  const codexLinks = (entry.codex ?? []).filter((id) => byId.has(id));
  const lessonId = entry.lesson?.replace(/\.lab$/, "") ?? null;
  const lessonDone = lessonId !== null && (context.completedMissions?.has(lessonId) ?? false);
  const primaryCodexDone = script !== null && context.completed.has(script.id);
  // A guided experiment may offer several legitimate routes. Completing its
  // lesson OR its primary runnable Codex route means the experiment was done;
  // the linked-learning counter still reports each optional connection.
  const done = lessonDone || primaryCodexDone;
  return {
    id: entry.id,
    source: "guided",
    title: title || entry.title,
    hook: hook || entry.phenomenon,
    level: entry.progress,
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
    support: null,
    prompt: null,
    anyLevel: false,
    reason: null,
    safety: entry.safety,
    safetyRationale: entry.safety_rationale ? kidsText(entry, "safety_rationale", context.locale) : null,
    safetyGuidance: entry.safety_guidance ? kidsText(entry, "safety_guidance", context.locale) : null,
    recipe: kidsRecipe(entry, context.locale),
    procedure: kidsList(entry, "procedure", context.locale),
    observations: kidsList(entry, "observations", context.locale),
    kits: (entry.kits ?? []).flatMap((id) => {
      const kit = KIDS_EQUIPMENT.find((candidate) => candidate.id === id);
      return kit ? [kit] : [];
    }),
    guided: entry,
    done,
    onShelf: onShelf(needs, context.shelfKeys),
    ...availability(needs, entry.apparatus, context),
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
      ...(entry.procedure ?? []),
      ...(entry.observations ?? []),
      ...(entry.kits ?? []),
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

/**
 * One reviewed corpus question as a row of the one index.
 *
 * Only what the prompt ACTUALLY says. Where the codex and guided mappers
 * derive a missing field from the entry's own content, a question has no
 * content to derive several of them from, and the honest answer is to
 * leave those empty and let the card omit the row rather than to invent a
 * claim:
 *
 *   - `needs` and `apparatus` stay EMPTY even though the script names
 *     materials, because the script writes formulae (`add v1 NaCl 5g`)
 *     and the shelf is keyed by registry id (`sodium_chloride`). Running
 *     `scriptKit` over it would have reported "missing now: NaCl" for a
 *     salt that is on the shelf. `availabilityKnown` is therefore false
 *     and the readiness line is not drawn for a question at all.
 *   - `done` is false for every question, and the card draws no
 *     completion chip: progress is a record of successful codex runs, and
 *     a question id is not a codex id, so "not tried" would be a claim
 *     about a record that cannot exist.
 *   - `minutes` is the script's own length, and zero where there is no
 *     script; the card omits the chip at zero rather than rounding a
 *     refusal up to "about 2 min".
 */
function fromCapability(prompt: CapabilityPrompt, context: CatalogViewContext): CatalogEntry {
  const said = localiseCapability(prompt, context.locale, context.translate);
  const steps = prompt.script.length;
  const minutes = steps > 0 ? minutesForSteps(steps) : 0;
  const runnable = capabilityRunnable(prompt);
  const reason = capabilityReasonText(prompt);
  return {
    id: prompt.id,
    source: "capability",
    title: said.question,
    // The corpus's own classification of the material the question is
    // about, which is the shortest true sentence available about it.
    hook: said.materialClass,
    level: CORPUS_BANDS[prompt.age_band] ?? "starter",
    anyLevel: CORPUS_BANDS[prompt.age_band] === undefined,
    steps,
    minutes,
    duration: durationBand(minutes),
    needs: [],
    apparatus: [],
    topics: capabilityTopics(prompt),
    concepts: [],
    placements: [],
    expectations: [],
    run: runnable ? { kind: "question" } : { kind: "unanswered" },
    script: null,
    lesson: null,
    quest: null,
    capabilities: [prompt.id],
    codexLinks: [],
    boundary: null,
    equation: null,
    status: SUPPORT_STATUS[prompt.support],
    support: prompt.support,
    prompt,
    reason,
    safety: null,
    safetyRationale: null,
    safetyGuidance: null,
    recipe: [],
    procedure: [],
    observations: [],
    kits: [],
    guided: null,
    done: false,
    onShelf: false,
    missingNeeds: [],
    access: [],
    readyNow: false,
    availabilityKnown: false,
    search: [
      ...capabilitySearchText(prompt, context.locale),
      said.question,
      said.materialClass,
      ...said.tags,
      reason,
    ],
  };
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
    case "question":
      // Says what the reader GETS, which is not what an experiment gives
      // them: the reviewed script runs on the bench and they read the
      // answer off it. There is no authored expectation behind a corpus
      // question, so nothing is checked against a prediction — and a
      // button promising "run the experiment" would imply there were.
      return "run this question on the bench";
    case "unanswered":
      return "read why this one has no answer";
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

/**
 * All three populations as one list, which is the whole point.
 *
 * Ordered by KIND first, then by level and title. Kind first because the
 * default list is what someone sees before they have asked anything, and
 * five hundred questions interleaved ahead of the experiments would bury
 * the runnable half of the library; a search still returns every matching
 * row, and the facet chip goes straight to the questions. Within a kind
 * the existing order stands: level, then the title the reader actually
 * sees, so a German list is alphabetical in German.
 *
 * `catalogEntries` above is deliberately left alone. It means "the
 * experiments", it is what the 252 is counted from, and a function whose
 * total silently grew by five hundred would be the false headline this
 * task exists to remove.
 */
export function oneIndex(
  codex: readonly CodexEntry[],
  guided: readonly KidsExperiment[],
  prompts: readonly CapabilityPrompt[],
  context: CatalogViewContext,
): CatalogEntry[] {
  const kind = (source: CatalogSourceKind) => CATALOG_SOURCES.indexOf(source);
  const rank = (level: CatalogLevel) => CATALOG_LEVELS.indexOf(level);
  return [
    ...catalogEntries(codex, guided, context),
    ...prompts.map((prompt) => fromCapability(prompt, context)),
  ].sort((a, b) =>
    kind(a.source) - kind(b.source)
    || rank(a.level) - rank(b.level)
    || a.title.localeCompare(b.title, context.locale)
    || a.id.localeCompare(b.id, context.locale),
  );
}

/**
 * How many rows of each kind, DERIVED from the rows themselves.
 *
 * The headline is built from this rather than from three constants,
 * because a count typed into a sentence is a count that stops being true
 * (#505: `models.toml` reported 100% German over 325 English strings).
 */
export function sourceCounts(entries: readonly CatalogEntry[]): Record<CatalogSourceKind, number> {
  const counts: Record<CatalogSourceKind, number> = { codex: 0, guided: 0, capability: 0 };
  for (const entry of entries) counts[entry.source] += 1;
  return counts;
}

/** The two experiment kinds, which are the only rows the bench can run. */
export function experimentCount(entries: readonly CatalogEntry[]): number {
  const counts = sourceCounts(entries);
  return counts.codex + counts.guided;
}

// ── Filters ───────────────────────────────────────────────────────────

/** The rail asks the same question the per-record predicate answers. */
export type CatalogProgressFilter = ExperimentProgressFilter;

export interface CatalogFilters {
  /** Which kind of row, as a facet rather than as a separate door. */
  source: CatalogSourceKind | null;
  level: CatalogLevel | null;
  topic: string | null;
  duration: CatalogDurationBand | null;
  /** Only entries whose materials are all on the shelf right now. */
  shelfOnly: boolean;
  readiness: "all" | "ready" | "missing";
  progress: CatalogProgressFilter;
  /** `system` and `stage` joined by a tab, which no stage name contains. */
  curriculum: string | null;
  concept: string | null;
  query: string;
}

export const NO_CATALOG_FILTERS: CatalogFilters = {
  source: null,
  level: null,
  topic: null,
  duration: null,
  shelfOnly: false,
  readiness: "all",
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
  if (filters.source && entry.source !== filters.source) return false;
  if (filters.level && entry.level !== filters.level && !entry.anyLevel) return false;
  // i18n-ok: topic, level and duration are wire keys chosen from a chip,
  // never text a reader typed.
  if (filters.topic && !entry.topics.includes(filters.topic)) return false;
  if (filters.duration && entry.duration !== filters.duration) return false;
  if (filters.shelfOnly && !entry.onShelf) return false;
  if (filters.readiness === "ready" && !entry.readyNow) return false;
  if (filters.readiness === "missing" && (!entry.availabilityKnown || entry.readyNow)) return false;
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

/**
 * Do these filters ask about something only an experiment has?
 *
 * A concept, a curriculum stage, a shelf of materials and a completion
 * record are properties of an experiment. A reviewed corpus question
 * carries none of them, so any of these axes drops all five hundred
 * question rows BY CONSTRUCTION rather than by failing to match — and a
 * reader watching the answers vanish from one box gets the same wrong
 * "no" the two doors used to give, only from inside one of them.
 *
 * So the rail says which filter is doing it. Pure, and here rather than in
 * the component, because "which filters a question cannot answer" is a
 * fact about the model and the tests have to be able to ask it.
 */
export function experimentOnlyFilters(filters: CatalogFilters): boolean {
  return filters.concept !== null
    || filters.curriculum !== null
    || filters.shelfOnly
    || filters.readiness !== "all"
    || filters.progress === "completed";
}

export function filterCatalogEntries(
  entries: readonly CatalogEntry[],
  filters: CatalogFilters,
): CatalogEntry[] {
  return entries.filter((entry) => catalogEntryPasses(entry, filters));
}

/** Exact authored relations only; never similarity or title matching. */
export function authoredRelatedEntries(
  entry: CatalogEntry,
  entries: readonly CatalogEntry[],
): CatalogEntry[] {
  return entries.filter((candidate) => {
    if (candidate.id === entry.id) return false;
    if (entry.codexLinks.includes(candidate.id) || candidate.codexLinks.includes(entry.id)) return true;
    if (entry.lesson && candidate.lesson === entry.lesson) return true;
    return entry.capabilities.some((id) => candidate.capabilities.includes(id));
  });
}

/**
 * How many entries sit at each level, for the chip counts.
 *
 * A chip's number has to be the number of rows that chip will show, so an
 * all-levels row — which passes every level filter — counts once under
 * each of them rather than once under its floor. A count that disagreed
 * with the list it labels is the defect, not a rounding.
 */
export function levelCounts(entries: readonly CatalogEntry[]): Record<CatalogLevel, number> {
  const counts: Record<CatalogLevel, number> = { starter: 0, intermediate: 0, advanced: 0 };
  for (const entry of entries) {
    if (entry.anyLevel) for (const level of CATALOG_LEVELS) counts[level] += 1;
    else counts[entry.level] += 1;
  }
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
