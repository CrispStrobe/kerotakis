/**
 * GUI-052 - the engine's own account of who answered, and who did not.
 *
 * A step object carries three separate kinds of evidence about how it was
 * computed, and until now the shell threw all three away:
 *
 *   * `routes` - the routing record (PROTOCOL.md; `SolverStack::last_routes`
 *     in `kerotakis-core/src/solve.rs`): every solver the stack asked, in
 *     the order it asked them, and what each one answered.
 *   * `provenance` on an event - the engine, dataset, model and routing
 *     sentence behind a computed claim (`vessel::Provenance`). Two events
 *     carry one: `thermal_equilibrium` for the combustion/CEA path, and
 *     `solution_routed` for the aqueous one. The second is why the reader
 *     below is written against the FIELD rather than against a list of
 *     event names - a new event that carries a provenance reaches the
 *     drawer without this file changing.
 *
 *     `solution_routed` fires on CHANGE, not on every characterisation:
 *     the aqueous solver runs far more often than a reader wants a line.
 *     So a step may carry no routing at all and still have been solved -
 *     which is the routing standing, not a routing missing.
 *   * the honesty pass - `not_yet_modeled` events, which say in the
 *     engine's own words what the bench declined to claim.
 *
 * This module turns those into the drawer's model and nothing else. It
 * renders no prose: every sentence it carries is the ENGINE's, copied
 * verbatim, and every label around them belongs to the component and goes
 * through `t()`. A paraphrase here would be the shell inventing chemistry
 * provenance, which is the one thing a provenance surface may not do.
 *
 * Pure and synchronous on purpose - the drawer must be testable without a
 * browser, an engine or a worker.
 */

/** Scientific authority of a route, as `SolverRouteKind` spells it. */
export type ProvenanceRouteKind = "computed" | "curated" | "qualitative";

/**
 * What one solver did with this vessel, collapsed to the three answers a
 * reader can act on. `declined` and `failed` are deliberately distinct: a
 * solver that had nothing to say is the system working, and a solver that
 * broke is not.
 */
export type ProvenanceRouteOutcome = "answered" | "declined" | "failed";

export type ProvenanceRoute = {
  /** The solver's own name, e.g. "phreeqc-aqueous". Never translated. */
  solver: string;
  kind: ProvenanceRouteKind;
  /** Whether this solver claims chemistry, as opposed to physics or honesty. */
  chemistry: boolean;
  outcome: ProvenanceRouteOutcome;
  /** How many events it contributed. Zero for anything but `answered`. */
  eventCount: number;
  /** The solver's OWN sentence for declining or failing, where it gave one. */
  reason?: string;
  vessel?: number;
};

/** A dataset-and-model claim lifted off an event's `provenance`. */
export type ProvenanceSource = {
  engine: string;
  dataset: string;
  model: string;
  /** Why this path was chosen over the alternatives - the engine's words. */
  routing: string;
  /** How the dataset documents its own literature sources. */
  datasetSources: string[];
  /**
   * Stated on an earlier step and still standing.
   *
   * `solution_routed` fires on CHANGE, so most steps carry no routing at
   * all - which means the routing STANDS, not that it is missing. A drawer
   * that showed nothing on those steps would be reporting the engine's
   * quietness as an absence of provenance, which is the opposite of what
   * the quietness means. So the session carries the last routing forward
   * and this flag says which one it is: a reader is entitled to know
   * whether the sentence was computed by the step they are looking at.
   */
  carried?: boolean;
};

/**
 * The last routing each vessel was told about, carried between steps.
 *
 * Keyed by vessel because two beakers can be on two datasets, and reading
 * one beaker's routing against the other is the corpus route-leak bug on a
 * screen. Held by the session and cleared wherever `latestStep` is cleared
 * - a routing read against a bench state that a different script produced
 * is exactly the staleness that clearing guards.
 */
export type CarriedRouting = Record<number, ProvenanceSource>;

/**
 * A bound on a number the step produced: `Event::Measured`'s `note`, which
 * is the range a reading's model is good over.
 *
 * `ValidityBounds` (`solve.rs`) is the other half of R0's validity story and
 * is deliberately NOT read here: no solver in the tree populates it - every
 * `capability()` returns `validity: None` - so a drawer that showed it would
 * show an empty box on every step and imply the engine had checked.
 */
export type ProvenanceValidity = {
  /** The engine's sentence, verbatim. */
  note: string;
  /** Which instrument's reading it bounds, where the event names one. */
  instrument?: string;
};

/** Something the bench declined to model, in its own words. */
export type ProvenanceGap = {
  what: string;
  /** `NotModelledCause`, kebab-case as the wire spells it. */
  cause: string;
};

/** The one line lv1 gets: who answered, on what data, and how many declined. */
export type ProvenanceHeadline = {
  solver: string;
  dataset?: string;
  declined: number;
};

export type ProvenanceReport = {
  /** The vessel this report was narrowed to, when it was narrowed. */
  vessel?: number;
  routes: ProvenanceRoute[];
  sources: ProvenanceSource[];
  validity: ProvenanceValidity[];
  gaps: ProvenanceGap[];
  headline: ProvenanceHeadline | null;
  /** Nothing to show at all - the caller hides the affordance rather than
   * opening a drawer onto an empty page. */
  empty: boolean;
};

/** One step of a `run_script` reply, as much of it as this module reads. */
export type ProvenanceStep = {
  events?: unknown;
  routes?: unknown;
};

function record(value: unknown): Record<string, unknown> | null {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : null;
}

function text(value: unknown): string | undefined {
  return typeof value === "string" && value.trim() ? value.trim() : undefined;
}

function vesselOf(value: unknown): number | undefined {
  return typeof value === "number" && Number.isInteger(value) && value >= 0 ? value : undefined;
}

const KINDS: ProvenanceRouteKind[] = ["computed", "curated", "qualitative"];

/**
 * Validate one wire route. Unknown shapes are dropped rather than coerced:
 * an older engine that never learned to send routes must leave the drawer
 * saying "no routing recorded", not showing an invented one.
 */
function routeOf(value: unknown): ProvenanceRoute | null {
  const raw = record(value);
  if (!raw) return null;
  const solver = text(raw.solver);
  if (!solver) return null;
  const kind = KINDS.find((candidate) => candidate === raw.kind);
  if (!kind) return null;
  const reason = text(raw.reason);
  const vessel = vesselOf(raw.vessel);
  const shared = {
    solver,
    kind,
    chemistry: raw.chemistry === true,
    ...(reason ? { reason } : {}),
    ...(vessel === undefined ? {} : { vessel }),
  };
  if (raw.outcome === "not_applicable") {
    return { ...shared, outcome: "declined", eventCount: 0 };
  }
  if (raw.outcome === "failed") {
    return { ...shared, outcome: "failed", eventCount: 0 };
  }
  const succeeded = record(record(raw.outcome)?.succeeded);
  if (!succeeded) return null;
  const count = succeeded.event_count;
  return {
    ...shared,
    outcome: "answered",
    eventCount: typeof count === "number" && count >= 0 ? count : 0,
  };
}

/** Lift `vessel::Provenance` off an event, in either shape it travels in. */
function sourceOf(value: unknown): ProvenanceSource | null {
  const raw = record(value);
  if (!raw) {
    // Some events carry the same claim already flattened to a sentence.
    const sentence = text(value);
    return sentence
      ? { engine: sentence, dataset: "", model: "", routing: "", datasetSources: [] }
      : null;
  }
  const engine = text(raw.engine) ?? "";
  const dataset = text(raw.dataset) ?? "";
  const model = text(raw.model) ?? "";
  const routing = text(raw.routing) ?? "";
  if (!engine && !dataset && !model && !routing) return null;
  const listed = Array.isArray(raw.dataset_sources) ? raw.dataset_sources : [];
  return {
    engine,
    dataset,
    model,
    routing,
    datasetSources: listed.map(text).filter((entry): entry is string => Boolean(entry)),
  };
}

function sourceKey(source: ProvenanceSource): string {
  return [source.engine, source.dataset, source.model, source.routing].join(" ");
}

/**
 * `previous`, updated with whatever routing this step announced.
 *
 * Pure, so a caller can hold the result as ordinary state and a test can
 * drive it a step at a time. A step that announced nothing returns the map
 * it was given, unchanged - the routing standing is not a routing gone.
 * An event that does not say which vessel it belongs to is dropped rather
 * than filed under a guess.
 */
export function carryRouting(
  previous: CarriedRouting,
  step: ProvenanceStep | null | undefined,
): CarriedRouting {
  const events = Array.isArray(step?.events) ? step.events : [];
  let next = previous;
  for (const entry of events) {
    const event = record(entry);
    if (!event) continue;
    const owner = vesselOf(event.vessel);
    if (owner === undefined) continue;
    const source = sourceOf(event.provenance);
    if (!source) continue;
    if (next === previous) next = { ...previous };
    next[owner] = source;
  }
  return next;
}

/**
 * Build the drawer's model from one step.
 *
 * `vessel` narrows to one beaker: a step may equilibrate more than one, and
 * a route or a refusal read against the wrong glass is worse than none.
 * Records that do not say which vessel they belong to are kept - the
 * alternative is silently hiding evidence because an older engine did not
 * label it.
 */
export function buildProvenance(
  step: ProvenanceStep | null | undefined,
  options: { vessel?: number; carried?: CarriedRouting } = {},
): ProvenanceReport {
  const { vessel, carried } = options;
  const mine = (owner: number | undefined): boolean =>
    vessel === undefined || owner === undefined || owner === vessel;

  const wireRoutes = Array.isArray(step?.routes) ? step.routes : [];
  const routes = wireRoutes
    .map(routeOf)
    .filter((route): route is ProvenanceRoute => route !== null)
    .filter((route) => mine(route.vessel));

  const events = Array.isArray(step?.events) ? step.events : [];
  const sources: ProvenanceSource[] = [];
  const seen = new Set<string>();
  const validity: ProvenanceValidity[] = [];
  const gaps: ProvenanceGap[] = [];

  for (const entry of events) {
    const event = record(entry);
    if (!event) continue;
    if (!mine(vesselOf(event.vessel))) continue;
    const source = sourceOf(event.provenance);
    if (source && !seen.has(sourceKey(source))) {
      seen.add(sourceKey(source));
      sources.push(source);
    }
    if (event.event === "measured") {
      const note = text(event.note);
      const instrument = text(event.instrument);
      if (note) validity.push({ note, ...(instrument ? { instrument } : {}) });
    }
    if (event.event === "not_yet_modeled") {
      const what = text(event.what);
      if (what) gaps.push({ what, cause: text(event.cause) ?? "unclassified" });
    }
  }

  // A routing this step did not restate, but which has not been withdrawn
  // either. Added only when the step said nothing about that source, so a
  // fresh statement always wins and is never marked carried.
  if (carried && vessel !== undefined) {
    const standing = carried[vessel];
    if (standing && !seen.has(sourceKey(standing))) {
      seen.add(sourceKey(standing));
      sources.push({ ...standing, carried: true });
    }
  }

  const answered = routes.filter((route) => route.outcome === "answered");
  // The chemistry engine is the one a reader means by "who answered"; the
  // physical mixing pass answers on nearly every step and saying so first
  // would bury the interesting half.
  const lead = answered.find((route) => route.chemistry) ?? answered[0];
  const dataset = sources.map((source) => source.dataset).find(Boolean);
  const headline: ProvenanceHeadline | null = lead
    ? {
        solver: lead.solver,
        ...(dataset ? { dataset } : {}),
        declined: routes.length - answered.length,
      }
    : null;

  return {
    ...(vessel === undefined ? {} : { vessel }),
    routes,
    sources,
    validity,
    gaps,
    headline,
    empty:
      routes.length === 0 && sources.length === 0 && validity.length === 0 && gaps.length === 0,
  };
}
