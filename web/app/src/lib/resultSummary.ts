import type { Scene } from "./host/EngineHost";

export type ResultQuantity = {
  label: string;
  value: number;
  unit: string;
};

/**
 * The engine's confidence vocabulary (`ops.rs::Confidence`), spelled the way
 * the wire spells it. GUI-023 fixes one visual encoding for these five in
 * `app.css`, keyed off `data-confidence`, so anything carrying one of these
 * strings into the DOM is encoded without the component choosing a style.
 */
export type ResultConfidence =
  | "computed"
  | "modeled"
  | "template_match"
  | "curated"
  | "unknown";

/** Before → after for one vessel, with how strongly the engine stands behind it. */
export type ResultTemperature = {
  beforeK: number;
  afterK: number;
  deltaK: number;
  confidence: ResultConfidence;
};

/** A hazard the safety screen raised while this command ran. */
export type ResultSafety = {
  severity: string;
  hazard: string;
  realWorld: string;
};

export type ResultSummary = {
  /** What the bench did, in the engine's own vocabulary. Always present. */
  kind: string;
  /**
   * GUI-091's class badge: the REACTION class, and only where an exact
   * event tag names one.
   *
   * Absent — never guessed — for everything else. Most of what a bench does
   * is not a reaction: stirring, weighing, transferring and settling all
   * produce a perfectly good `kind`, and none of them has a reaction class.
   * Calling those "unknown" would be as wrong as inventing a class for them;
   * the honest answer is that the question does not arise, so the badge
   * simply is not drawn.
   */
  reactionClass?: string;
  vessel?: number;
  equation?: string;
  /** The left-hand side of the engine's own balanced equation, one chip each. */
  reactants: string[];
  observation?: string;
  temperature?: ResultTemperature;
  /** Retained for callers that only want the magnitude. */
  temperatureDeltaK?: number;
  quantities: ResultQuantity[];
  boundary?: string;
  provenance?: string;
  /** The concept note: why nothing happened, or what the model did not couple. */
  note?: string;
  safety?: ResultSafety;
};

type EngineEvent = Record<string, unknown>;

const CLASSIFICATIONS: Record<string, string> = {
  precipitated: "precipitation",
  plated: "metal plating",
  electrolysed: "electrolysis",
  gas_evolved: "gas evolution",
  gas_absorbed: "gas absorption",
  gas_contained: "gas formation",
  distilled: "distillation",
  filtered: "filtration",
  chromatographed: "chromatography",
  measured: "measurement",
  temperature_changed: "temperature change",
  energy_transferred: "energy transfer",
  heat_of_mixing: "heat of mixing",
  stirred: "mixing",
  mixed: "mixing",
  transported: "transport",
  gravity_settled: "settling",
  centrifuged: "centrifugation",
  ground: "grinding",
  irradiated: "irradiation",
  org_reacted: "reaction",
  decayed: "radioactive decay",
  burst: "vessel failure",
  layers_formed: "phase separation",
  dissolved: "dissolution",
  dissolved_in_solvent: "dissolution",
  transferred: "transfer",
  added: "addition",
  material_added: "addition",
  observed: "observation",
  inert: "no reaction",
  inert_in_solvent: "no reaction",
};

/**
 * The subset of those tags that name a REACTION class (GUI-091).
 *
 * Deliberately a second, smaller table rather than a flag on the first one,
 * because the two answer different questions. `CLASSIFICATIONS` says what
 * the bench did — and "mixing", "measurement", "transfer" and "settling"
 * are perfectly true answers to that. None of them is a reaction, so none
 * of them earns the class badge. The roadmap's rule is that the badge is
 * *absent* rather than guessed when the classification is not clean, and a
 * table that only lists clean cases is how that rule is enforced rather
 * than merely intended: adding a tag here is a deliberate claim that the
 * engine has classified a reaction.
 */
const REACTION_CLASSES: Record<string, string> = {
  precipitated: "precipitation",
  plated: "metal plating",
  electrolysed: "electrolysis",
  gas_evolved: "gas evolution",
  gas_absorbed: "gas absorption",
  gas_contained: "gas formation",
  org_reacted: "reaction",
  decayed: "radioactive decay",
  dissolved: "dissolution",
  dissolved_in_solvent: "dissolution",
  // "Nothing reacted" is a classification, and a confident one: the engine
  // evaluated the pair and found no reaction. It is not an absence of an
  // answer, so it belongs here.
  inert: "no reaction",
  inert_in_solvent: "no reaction",
};

const PRIORITY = [
  "burst", "org_reacted", "electrolysed", "precipitated", "plated",
  "gas_evolved", "gas_absorbed", "gas_contained", "distilled", "filtered",
  "chromatographed", "layers_formed", "decayed", "heat_of_mixing",
  "measured", "temperature_changed", "energy_transferred", "centrifuged",
  "stirred", "mixed", "transported", "gravity_settled", "ground", "irradiated", "dissolved", "dissolved_in_solvent",
  "transferred", "added", "material_added", "observed", "inert", "inert_in_solvent",
];

function number(event: EngineEvent, key: string): number | undefined {
  const value = event[key];
  return typeof value === "number" && Number.isFinite(value) ? value : undefined;
}

function eventVessel(event: EngineEvent): number | undefined {
  for (const key of ["vessel", "into", "receiver", "to", "from"]) {
    const value = number(event, key);
    if (value !== undefined) return value;
  }
  return undefined;
}

function quantities(event: EngineEvent): ResultQuantity[] {
  const values: ResultQuantity[] = [];
  const push = (label: string, key: string, unit: string, scale = 1) => {
    const value = number(event, key);
    if (value !== undefined) values.push({ label, value: value * scale, unit });
  };
  push("amount", "moles", "mol");
  push("mass", "grams", "g");
  push("energy", "delivered_j", "kJ", 0.001);
  if (!values.some((value) => value.label === "energy")) push("energy", "joules", "kJ", 0.001);
  push("speed", "rpm", "rpm");
  push("duration", "seconds", "s");
  push("voltage", "volts", "V");
  push("charge", "coulombs", "C");
  push("activity", "activity_bq", "Bq");
  push("resuspended", "resuspended_fraction", "%", 100);
  push("source A", "fraction_a", "%", 100);
  push("source B", "fraction_b", "%", 100);
  push("transferred", "fraction", "%", 100);
  const measured = number(event, "value");
  if (measured !== undefined && typeof event.unit === "string") {
    values.push({ label: "reading", value: measured, unit: event.unit });
  }
  return values.slice(0, 3);
}

function boundary(event: EngineEvent): string | undefined {
  if (event.event === "stirred" && event.rate_coupled === false) {
    return "suspension changed; reaction rates are not yet coupled";
  }
  if (event.event === "irradiated" && event.photolysis_coupled === false) {
    return "light was applied; photolysis is not yet coupled";
  }
  if (event.event === "energy_transferred" && event.time_coupled === false) {
    return "the heat was delivered; the time it would take is not yet coupled";
  }
  return undefined;
}

function provenance(event: EngineEvent): string | undefined {
  if (typeof event.provenance === "string" && event.provenance.trim()) {
    return event.provenance.trim();
  }
  if (event.provenance && typeof event.provenance === "object" && !Array.isArray(event.provenance)) {
    const record = event.provenance as Record<string, unknown>;
    const parts = [record.engine, record.dataset, record.model, record.source]
      .filter((part): part is string => typeof part === "string" && part.trim().length > 0);
    return parts.length > 0 ? parts.join(" · ") : undefined;
  }
  return undefined;
}

/**
 * GUI-090's concept note, taken from the event stream rather than authored.
 *
 * Three events carry a sentence that explains the chemistry rather than
 * reporting it: `inert.why` says why nothing happened, `org_reacted.boundary`
 * says what the organic template did and did not claim, and
 * `safety_veto.reason` says why the bench declined. All three are engine
 * prose, already localized on the way out, so they are shown verbatim — and
 * where none of them is present the card carries no note rather than an
 * authored one.
 */
function conceptNote(events: EngineEvent[], event: EngineEvent): string | undefined {
  const own = event.event === "inert" || event.event === "inert_in_solvent"
    ? event.why
    : event.event === "org_reacted"
      ? event.boundary
      : undefined;
  if (typeof own === "string" && own.trim().length > 0) return own.trim();
  for (let index = events.length - 1; index >= 0; index -= 1) {
    const candidate = events[index];
    if (candidate?.event === "safety_veto" && typeof candidate.reason === "string") {
      return candidate.reason.trim() || undefined;
    }
  }
  return undefined;
}

/**
 * The safety note, where L0 raised one for this command.
 *
 * `hazard_warning` always precedes the chemistry it warns about, so a card
 * that omits it shows the outcome of an operation without the warning that
 * came with it. The most severe warning wins; ties go to the last one.
 */
function safetyNote(events: EngineEvent[]): ResultSafety | undefined {
  let best: ResultSafety | undefined;
  let bestRank = -1;
  for (const event of events) {
    if (event.event !== "hazard_warning") continue;
    const hazard = typeof event.hazard === "string" ? event.hazard : "";
    const realWorld = typeof event.real_world === "string" ? event.real_world : "";
    if (hazard === "" && realWorld === "") continue;
    const severity = typeof event.severity === "string" ? event.severity : "";
    const rank = severity === "danger" ? 2 : severity === "caution" ? 1 : 0;
    if (rank >= bestRank) {
      bestRank = rank;
      best = { severity, hazard, realWorld };
    }
  }
  return best;
}

/**
 * The reactant chips: the left-hand side of the equation the engine wrote.
 *
 * Nothing is looked up and nothing is guessed — a balanced equation already
 * names its reactants, so splitting its arrow is a projection rather than a
 * second source of truth. Stoichiometric coefficients belong to the equation
 * on the line above, so a chip carries the species alone; the phase and state
 * marks the engine writes (`↓`, `↑`) are dropped for the same reason.
 */
export function reactantsOf(equation: string | undefined): string[] {
  if (!equation) return [];
  const [left] = equation.split(/⇌|⟶|→|<=>|=>|->|⇄|↔/);
  if (left === undefined || left === equation) return [];
  // A SPACED plus, because a bare one is also the charge on `Ag+` — the
  // same rule `stoich.rs` splits sides by, for the same reason.
  return left
    .split(" + ")
    .map((term) => term.trim().replace(/^\d+(?:[.,]\d+)?\s+/, "").replace(/[↓↑]/g, "").trim())
    .filter((term) => term.length > 0 && term.length <= 40);
}

/**
 * The equation this command produced, from wherever in the stream it is.
 *
 * Only four event variants carry one (`reaction_occurred`, `org_reacted`,
 * `decayed`, `cell_voltage`), and none of them is usually the event that
 * wins the priority list: a curated precipitation emits `reaction_occurred`
 * with the equation *beside* `precipitated` with the observation. Looking
 * only at the winner is why the card showed no equation for the commonest
 * reaction there is. Still one step's output, still no second engine call.
 */
function equationOf(
  events: EngineEvent[],
  event: EngineEvent,
  vessel?: number,
): string | undefined {
  const own = typeof event.equation === "string" ? event.equation.trim() : "";
  if (own.length > 0) return own;
  for (let index = events.length - 1; index >= 0; index -= 1) {
    const candidate = events[index];
    if (!candidate || typeof candidate.equation !== "string") continue;
    const text = candidate.equation.trim();
    if (text.length === 0) continue;
    const where = eventVessel(candidate);
    if (vessel === undefined || where === undefined || where === vessel) return text;
  }
  return undefined;
}

/**
 * How strongly the engine stands behind the before → after temperature.
 *
 * `temperature_changed` is the engine reporting the two numbers itself, so it
 * is `computed`. `heat_of_mixing` is a UNIFAC-derived excess enthalpy — a
 * model with fitted parameters, verified pair by pair (`hmix.rs`) — so it is
 * `modeled` even though the arithmetic afterwards is exact. Everything else
 * is the difference between two scenes the solver produced, which is
 * `computed` for the same reason a scene badge is.
 */
function temperatureConfidence(events: EngineEvent[], vessel?: number): ResultConfidence {
  const forVessel = (event: EngineEvent) =>
    vessel === undefined || eventVessel(event) === undefined || eventVessel(event) === vessel;
  if (events.some((event) => event.event === "temperature_changed" && forVessel(event))) {
    return "computed";
  }
  if (events.some((event) => event.event === "heat_of_mixing" && forVessel(event))) {
    return "modeled";
  }
  return "computed";
}

function significantEvent(events: EngineEvent[]): EngineEvent | undefined {
  for (const kind of PRIORITY) {
    for (let index = events.length - 1; index >= 0; index -= 1) {
      if (events[index]?.event === kind) return events[index];
    }
  }
  return undefined;
}

function observation(rendered: string[], equation?: string): string | undefined {
  for (let index = rendered.length - 1; index >= 0; index -= 1) {
    const line = rendered[index];
    if (!line) continue;
    const text = line.trim();
    if (text.length > 0 && text !== equation && !/^(?:.+\s)?(?:→|⇌)(?:\s.+)?$/.test(text)) {
      return text;
    }
  }
  return undefined;
}

/** Build a truthful UI digest from one accepted engine command. */
export function summarizeResult(
  events: unknown[],
  rendered: string[],
  before: Scene | null,
  after: Scene | null,
): ResultSummary | null {
  const typed = events.filter(
    (event): event is EngineEvent => Boolean(event && typeof event === "object" && !Array.isArray(event)),
  );
  const event = significantEvent(typed);
  if (!event || typeof event.event !== "string") return null;

  const vessel = eventVessel(event);
  // Only `org_reacted`, `reaction_occurred`, `decayed` and `cell_voltage`
  // carry an equation, and the significant event is usually one of the
  // observations *beside* them (a precipitate, a gas). So the equation is
  // taken from the whole accepted command rather than from the one event
  // that won the priority list — still one step's output, still no new call.
  const equation = equationOf(typed, event, vessel);
  // `temperature_changed` reports the pair itself; otherwise the two scenes
  // the command was computed between hold it.
  const reported = typed.find(
    (item) => item.event === "temperature_changed"
      && (vessel === undefined || eventVessel(item) === vessel)
      && number(item, "from") !== undefined
      && number(item, "to") !== undefined,
  );
  const beforeTemperature = reported
    ? number(reported, "from")
    : before?.vessels.find((item) => item.id === vessel)?.temperature_k;
  const afterTemperature = reported
    ? number(reported, "to")
    : after?.vessels.find((item) => item.id === vessel)?.temperature_k;
  const temperatureDeltaK = beforeTemperature !== undefined && afterTemperature !== undefined
    ? afterTemperature - beforeTemperature
    : undefined;
  const moved = temperatureDeltaK !== undefined && Math.abs(temperatureDeltaK) >= 0.05;

  return {
    kind: CLASSIFICATIONS[event.event] ?? event.event.replaceAll("_", " "),
    reactionClass: REACTION_CLASSES[event.event],
    vessel,
    equation,
    reactants: reactantsOf(equation),
    observation: observation(rendered, equation),
    temperature: moved
      ? {
        beforeK: beforeTemperature!,
        afterK: afterTemperature!,
        deltaK: temperatureDeltaK!,
        confidence: temperatureConfidence(typed, vessel),
      }
      : undefined,
    temperatureDeltaK: moved ? temperatureDeltaK : undefined,
    quantities: quantities(event),
    boundary: boundary(event),
    provenance: provenance(event),
    note: conceptNote(typed, event),
    safety: safetyNote(typed),
  };
}
