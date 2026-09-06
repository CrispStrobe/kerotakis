/**
 * One list of everything a learner can pick up (GUI-101).
 *
 * The app offered instruments in three places. The `MESSEN` strip listed the
 * twelve measurements; the Geräteschrank listed the same twelve again beside
 * the apparatus, the transfer verbs and the kits; and the vessel dock listed
 * three of the measurements and three of the apparatus a third time. Three
 * lists of overlapping things, each with its own idea of what a tool IS —
 * "take a reading now", "install on the bench", "do a thing to this vessel".
 *
 * This is the one list. Every entry carries:
 *
 *   - `id`, in the catalog's OWN id space — `measure:<token>` for an
 *     instrument, the bare verb otherwise — so `equipmentAccess()` can be
 *     asked about any entry without a second mapping table to keep in step;
 *   - `group`, which is what the thing DOES, because that is the question a
 *     learner is actually asking when they open a cupboard;
 *   - `action`, a five-way union that maps one-to-one onto the handlers the
 *     app already passes down. Nothing here invents a new path into the
 *     engine: the cupboard is a different way to reach the existing ones.
 *   - `boundary`, the sentence behind the (i). The kits had one from the
 *     start; the instruments and apparatus never did, and "what this models
 *     and what it does not" is the single most useful thing a simulated
 *     instrument can say about itself.
 *
 * A kit is not a parallel list. It is an `aliasOf` entry over a tool that is
 * already here, so its availability and its action come from the tool it is
 * a skin over and only the name, the picture and the preset differ. Since
 * GUI-103 it is not a shelf either: the sets are a header chip that renames
 * the tools they stand for, so the candle has one slot in both states.
 */
import { APPARATUS } from "./apparatus";
import type { TwoVesselAction } from "./directActions";
import type { InfoRow } from "./infoPanel";
import { INSTRUMENTS, instrumentCommand, instrumentVerb } from "./instruments";
import { KIDS_EQUIPMENT } from "./kidsEquipment";
import { TRANSFER_TOOLS } from "./transferTools";

/**
 * What the thing does — the shelf it stands on.
 *
 * Five shelves, not six (GUI-103). *antreiben* ("drive and power") was the
 * one heading that did not answer "what do I want to do": a stirrer, a
 * centrifuge, a lamp, a pair of electrodes and the reaction studio share a
 * word, not a purpose. Its members went to the two neighbours that already
 * described them — the stirrer, the centrifuge and the mortar to
 * *vorbereiten*, the electrodes and the half-cell to *verbinden*, beside
 * the tubing and the gas line they are wired to.
 *
 * The kits' shelf is gone too, and for a different reason: a kit is not a
 * kind of tool, it is a NAME for one. It is a header chip now, not a shelf.
 */
export type EquipmentGroup = "measure" | "thermal" | "prepare" | "contain" | "separate";

/** The shelves, in the order the cupboard shows them. */
export const EQUIPMENT_GROUPS: readonly EquipmentGroup[] = [
  "measure",
  "thermal",
  "prepare",
  "contain",
  "separate",
];

/** Shelf headings. Literals, so the translation scan can see them. */
export const GROUP_LABELS: Record<EquipmentGroup, string> = {
  measure: "observe and measure",
  thermal: "heat and cool",
  prepare: "prepare and convert",
  contain: "contain and connect",
  separate: "transfer and separation",
};

/**
 * One sentence per shelf: what lives here, in the learner's own terms.
 *
 * A heading of two words is a label, not an explanation, and "vorbereiten"
 * does not by itself say that the mortar is on that shelf. The cupboard
 * shows these on the shelf heading — on hover and focus, and in the tip
 * strip a touch screen can reach.
 */
export const GROUP_BLURBS: Record<EquipmentGroup, string> = {
  measure: "Ask the selected vessel a question. Every one of these reads a value and changes nothing.",
  thermal: "Put energy in or take it out as heat: flame, hotplate, cooling bath, evaporating dish.",
  prepare: "Get the contents ready, or drive a change in them: stir, grind, dilute, spin, irradiate, react.",
  contain: "Hold a boundary or join vessels: balloon, gas line, burette, mixer, tubing, electrodes, half-cell.",
  separate: "Move one part of a vessel into another and leave the rest behind.",
};

export type EquipmentAction =
  | { kind: "measure"; token: string }
  | { kind: "install"; verb: string; preset?: Record<string, string | number> }
  | { kind: "transfer"; verb: TwoVesselAction }
  | { kind: "mix" }
  | { kind: "burette" };

/** A line-art portrait, or the instrument's own typographic glyph. */
export type EquipmentRender =
  | { kind: "icon"; name: string }
  | { kind: "glyph"; text: string };

export interface EquipmentEntry {
  /** The catalog id `equipmentAccess()` answers about. */
  id: string;
  group: EquipmentGroup;
  /** English source text; `t()` at the call site. */
  name: string;
  blurb: string;
  /** What the model computes, and what it does not. Behind the (i). */
  boundary: string;
  render: EquipmentRender;
  action: EquipmentAction;
  /** A kit stands for this id; availability is read from it, not from the kit. */
  aliasOf?: string;
  /** Kits only: the physical parts, for the (i). */
  parts?: string[];
}

/**
 * Which shelf each verb stands on.
 *
 * Written out rather than derived, because the grouping is a teaching
 * decision and a derivation would hide it. The mortar is filed under
 * *prepare* rather than under separation: grinding is what a learner does
 * BEFORE the separation, and a mortar separates nothing by itself.
 * Electrolysis sits under *connect* because the thing a learner has to get
 * right is the wiring — two electrodes in one vessel, like the half-cell
 * beside it.
 */
const APPARATUS_GROUPS: Record<string, EquipmentGroup> = {
  bunsen: "thermal",
  heat: "thermal",
  cool: "thermal",
  evaporate: "thermal",
  dilute: "prepare",
  grind: "prepare",
  stir: "prepare",
  centrifuge: "prepare",
  irradiate: "prepare",
  regulate: "contain",
  sweep: "contain",
  electrolyse: "contain",
};

/**
 * The two-vessel verbs are the separation shelf, with one exception.
 *
 * `cell` moves nothing: it wires two half-cells together and reads what is
 * between them, which is the connection shelf's question, not the
 * separation shelf's.
 */
const TRANSFER_GROUPS: Record<TwoVesselAction, EquipmentGroup> = {
  filter: "separate",
  decant: "separate",
  drain: "separate",
  magnet: "separate",
  distil: "separate",
  cell: "contain",
};

/**
 * The boundary sentence, per id.
 *
 * Every one of these says two things: what the engine actually computes for
 * this tool, and the part of the real instrument that is NOT there. The
 * second half is the one that matters — a simulated calorimeter that never
 * admits it loses no heat to the room teaches a wrong number confidently.
 */
const BOUNDARIES: Record<string, string> = {
  "measure:smell": "reports the computed headspace; odour thresholds and adaptation are not modeled",
  "measure:thermometer": "reads the vessel's computed temperature; probe heat capacity and response time are not modeled",
  "measure:ph": "reads the computed activity of aqueous protons; electrode drift and calibration are not modeled",
  "measure:balance": "reads the total mass the engine tracks; air buoyancy and drift are not modeled",
  "measure:volume": "reads the gas a sealed headspace holds; an open vessel has no boundary to measure",
  "measure:conductivity": "estimates conductivity from the computed ions; cell constant and electrode polarisation are not modeled",
  "measure:pressure": "reads the computed headspace pressure; only a sealed or regulated boundary holds one",
  "measure:calorimeter": "reports enthalpy relative to 25 °C; heat lost to the room and to the vessel is not modeled",
  "measure:uvvis": "computes absorbance from the species present; instrument bandwidth and stray light are not modeled",
  "measure:eyes": "reports the colour, phases and solids the engine computed; nothing is invented for the picture",
  "measure:chromatograph": "runs the computed chromatography operator; paper, tile and solvent front are classroom skins",
  "measure:geiger": "counts the computed activity of the nuclides present; shielding and detector geometry are not modeled",
  bunsen: "delivers up to 500 W and caps the vessel at the flame's own ceiling; soot, carbon monoxide and flame shape are not modeled",
  heat: "delivers the chosen energy and caps the vessel at 550 °C; contact area with the plate is not modeled",
  cool: "removes the chosen energy; the coolant and the bath's own heat capacity are not modeled",
  evaporate: "boils away the chosen fraction of the liquid; the heating path and splashing are not modeled",
  stir: "mixes at the chosen speed; vortex shape and shear damage are not modeled",
  centrifuge: "separates by the computed relative centrifugal force; rotor heating is not modeled",
  electrolyse: "passes the chosen charge through the cell; overpotential and electrode fouling are not modeled",
  irradiate: "shines one wavelength at the chosen irradiance; the lamp's own spectrum is not modeled",
  dilute: "adds water up to the chosen volume; the added water's own temperature is not modeled",
  regulate: "holds the chosen pressure with a flexible boundary; rubber stretch and leaks are not modeled",
  sweep: "replaces the headspace with inert gas at the chosen pressure; flow rate is not modeled",
  grind: "sets a solid's grain size; the work done and losses in the mortar are not modeled",
  burette: "adds solution in controlled steps; the tap, the meniscus and drainage errors are not modeled",
  mix: "combines two sources with the engine's thermodynamic mix; the pouring itself is not modeled",
  transport: "moves solution through connected cells; the tubing's own volume is not modeled",
  react: "offers only reaction families the engine has verified; it invents no chemistry of its own",
  filter: "holds back the solids and passes the liquid; pore size and filter losses are not modeled",
  decant: "pours off the chosen fraction of the upper layer; the disturbance of pouring is not modeled",
  drain: "moves the lower liquid layer; the sharpness of the interface is not modeled",
  magnet: "moves only solids the registry declares magnetic; field strength is not modeled",
  cell: "connects two half-cells and reads the computed potential; internal resistance is not modeled",
  distil: "separates by volatility through a connected rig; column plates and hold-up are not modeled",
};

const boundaryOf = (id: string): string => BOUNDARIES[id] ?? "";

const instrumentEntries = (): EquipmentEntry[] =>
  INSTRUMENTS.map((item) => ({
    id: instrumentVerb(item.token),
    group: "measure" as const,
    name: item.label,
    blurb: item.purpose,
    boundary: boundaryOf(instrumentVerb(item.token)),
    render: { kind: "glyph" as const, text: item.glyph },
    action: { kind: "measure" as const, token: item.token },
  }));

const apparatusEntries = (): EquipmentEntry[] =>
  APPARATUS.map((item) => ({
    id: item.verb,
    group: APPARATUS_GROUPS[item.verb] ?? "prepare",
    name: item.title,
    blurb: item.blurb,
    boundary: boundaryOf(item.verb),
    render: { kind: "icon" as const, name: item.verb },
    action: { kind: "install" as const, verb: item.verb },
  }));

const transferEntries = (): EquipmentEntry[] =>
  TRANSFER_TOOLS.map((item) => ({
    id: item.verb,
    group: TRANSFER_GROUPS[item.verb],
    name: item.title,
    blurb: item.blurb,
    boundary: boundaryOf(item.verb),
    render: { kind: "icon" as const, name: item.verb },
    action: { kind: "transfer" as const, verb: item.verb },
  }));

/** The four that are neither an `ApparatusSpec` nor a two-vessel verb. */
const SPECIAL_ENTRIES: EquipmentEntry[] = [
  {
    id: "burette",
    group: "contain",
    name: "burette",
    blurb: "controlled addition",
    boundary: boundaryOf("burette"),
    render: { kind: "icon", name: "burette" },
    action: { kind: "burette" },
  },
  {
    id: "mix",
    group: "contain",
    name: "mixer",
    blurb: "combine two sources into a receiver",
    boundary: boundaryOf("mix"),
    render: { kind: "icon", name: "mix" },
    action: { kind: "mix" },
  },
  {
    // The tubing, filed with what it connects rather than with what it is
    // used for: a column train is a joined rig before it is a separation.
    id: "transport",
    group: "contain",
    name: "column train",
    blurb: "move solution through connected cells",
    boundary: boundaryOf("transport"),
    render: { kind: "icon", name: "transport" },
    action: { kind: "install", verb: "transport" },
  },
  {
    id: "react",
    group: "prepare",
    name: "curated reaction",
    blurb: "choose a verified reaction family",
    boundary: boundaryOf("react"),
    render: { kind: "icon", name: "react" },
    action: { kind: "install", verb: "react" },
  },
];

/**
 * A kit's action is the action of the tool it skins.
 *
 * `KidsEquipment.action` is a three-way string ("apparatus" | "transfer" |
 * "instrument") that predates this union; converting here rather than
 * changing that file keeps the kit data a description of a kit and lets the
 * conversion be the thing that is tested.
 */
const kitAction = (verb: string, preset?: Record<string, string | number>): EquipmentAction => {
  if (verb.startsWith("measure:")) return { kind: "measure", token: verb.slice("measure:".length) };
  if (TRANSFER_TOOLS.some((tool) => tool.verb === verb)) return { kind: "transfer", verb: verb as TwoVesselAction };
  return { kind: "install", verb, preset };
};

/** The tools themselves: one slot each, and the only things on a shelf. */
const TOOL_ENTRIES: EquipmentEntry[] = [
  ...instrumentEntries(),
  ...apparatusEntries(),
  ...transferEntries(),
  ...SPECIAL_ENTRIES,
];

/**
 * A kit stands on the shelf of the tool it skins.
 *
 * It is never DRAWN there — the shelves render tools, and a set replaces
 * the name on one of them — but a catalogue entry with no shelf would be an
 * entry that cannot answer "where is this". The group is read from the tool
 * rather than restated, so a tool that moves shelf takes its set with it.
 */
const kitEntries = (): EquipmentEntry[] =>
  KIDS_EQUIPMENT.map((item) => ({
    id: item.id,
    group: TOOL_ENTRIES.find((tool) => tool.id === item.engineVerb)?.group ?? "prepare",
    name: item.title,
    blurb: item.blurb,
    boundary: item.boundary,
    render: { kind: "icon" as const, name: item.icon },
    action: kitAction(item.engineVerb, item.preset),
    aliasOf: item.engineVerb,
    parts: item.parts,
  }));

/** Everything, once. The order within a shelf is the order it is declared. */
export const EQUIPMENT_CATALOGUE: EquipmentEntry[] = [...TOOL_ENTRIES, ...kitEntries()];

/** The id availability is read for: a kit borrows its skinned tool's. */
export const accessId = (entry: EquipmentEntry): string => entry.aliasOf ?? entry.id;

export const equipmentIn = (group: EquipmentGroup): EquipmentEntry[] =>
  EQUIPMENT_CATALOGUE.filter((entry) => entry.group === group);

export const equipmentById = (id: string): EquipmentEntry | undefined =>
  EQUIPMENT_CATALOGUE.find((entry) => entry.id === id);

/**
 * The slots the shelves draw: every tool, and no set.
 *
 * A set used to be a sixth shelf, which put the candle on the wall twice —
 * once as *Kerze und Docht* and once as *Kerze / Bunsenbrenner*. It is a
 * naming of the same slot, so it is drawn INTO that slot instead.
 */
export const SHELF_ENTRIES: EquipmentEntry[] = EQUIPMENT_CATALOGUE.filter(
  (entry) => entry.aliasOf === undefined,
);

/** The set that names this tool, if a set does. */
export const setSkinOf = (id: string): EquipmentEntry | undefined =>
  EQUIPMENT_CATALOGUE.find((entry) => entry.aliasOf === id);

/**
 * One slot, as it is drawn.
 *
 * With the *Experimentierkästen* chip off, a tool is itself. With it on, a
 * tool a set names is drawn as that set — its name, its picture, its parts
 * and its purpose, and its `preset` too, because "Kerze und Docht" that
 * opened the flame panel on a laboratory burner's 1500 °C default would be
 * a candle in name only. A tool no set names is untouched: the chip renames
 * what it can and hides nothing.
 */
export const asShown = (entry: EquipmentEntry, sets: boolean): EquipmentEntry =>
  (sets ? setSkinOf(entry.id) ?? entry : entry);

/**
 * The ids the tally counts: every tool the learner can EVER have.
 *
 * A constant, deliberately. The old denominator dropped `react` whenever
 * this session had no curated reaction to offer, so the wall read "31/33"
 * and then "32/34" a command later — a fraction whose bottom half moved is
 * a fraction that measures nothing. Kits are excluded because they are
 * names for these ids, not extra things to unlock.
 */
export const GATED_IDS: readonly string[] = SHELF_ENTRIES.map((entry) => entry.id);

export interface CupboardTally {
  available: number;
  total: number;
  /** False when everything is reachable: "34/34" is a fact, not information. */
  show: boolean;
}

/** The header fraction, and whether it is worth printing. */
export function cupboardTally(isAvailable: (id: string) => boolean): CupboardTally {
  const available = GATED_IDS.filter((id) => isAvailable(id)).length;
  return { available, total: GATED_IDS.length, show: available < GATED_IDS.length };
}

/** Whether the cupboard opens showing set names. Remembered per browser. */
export const SETS_VIEW_KEY = "kerotakis.equipment.sets";

/** Off unless this browser says otherwise; anything unreadable is off. */
export function loadSetsView(storage: { getItem(key: string): string | null } | null, key: string): boolean {
  if (!storage) return false;
  try {
    return storage.getItem(key) === "on";
  } catch {
    // A private window throws on the property itself.
    return false;
  }
}

export function saveSetsView(
  storage: { setItem(key: string, value: string): void } | null,
  key: string,
  on: boolean,
): void {
  if (!storage) return;
  try {
    storage.setItem(key, on ? "on" : "off");
  } catch {
    // The chip still works for this visit when persistence is unavailable.
  }
}

export interface EquipmentHandlers {
  onmeasure: (line: string) => void;
  onapparatus: (verb: string, preset?: Record<string, string | number>) => void;
  ontransfer: (verb: TwoVesselAction) => void;
  onmix: () => void;
  onburette: () => void;
}

/** One selection, routed to the handler that already existed for it. */
export function runEquipment(entry: EquipmentEntry, target: number, handlers: EquipmentHandlers): void {
  const action = entry.action;
  switch (action.kind) {
    case "measure":
      return handlers.onmeasure(instrumentCommand(target, action.token));
    case "install":
      return handlers.onapparatus(action.verb, action.preset);
    case "transfer":
      return handlers.ontransfer(action.verb);
    case "mix":
      return handlers.onmix();
    case "burette":
      return handlers.onburette();
  }
}

export interface DeploymentState {
  apparatusOut: string | null;
  buretteOut: boolean;
  transferVerb: TwoVesselAction | null;
  mixActive: boolean;
}

/**
 * What the entry is currently doing on the bench, as a badge — or null.
 *
 * A measurement is never "deployed": it happens and it is over, so the only
 * honest answer for an instrument is nothing at all.
 */
export function deployedLabel(entry: EquipmentEntry, state: DeploymentState): string | null {
  const action = entry.action;
  switch (action.kind) {
    case "install":
      return state.apparatusOut === action.verb ? "on bench" : null;
    case "burette":
      return state.buretteOut ? "on bench" : null;
    case "transfer":
      return state.transferVerb === action.verb ? "select source" : null;
    case "mix":
      return state.mixActive ? "select sources" : null;
    case "measure":
      return null;
  }
}

/**
 * The (i) panel for one entry: its parts, if it is a kit, and its boundary.
 *
 * `translate` is passed in so this stays a pure function of the data —
 * `kidsEquipment.ts` settled that pattern, and the row labels are literals
 * here for the same reason: so the translation scan can see them.
 */
export function equipmentInfoRows(
  entry: EquipmentEntry,
  translate: (key: string) => string,
): InfoRow[] {
  const rows: InfoRow[] = [];
  if (entry.parts && entry.parts.length > 0) {
    rows.push({ term: translate("parts"), detail: entry.parts.map((part) => translate(part)).join(" · ") });
  }
  if (entry.boundary) {
    // A sentence set opposite its label starts in a different place on
    // every row; anything sentence-shaped takes the full width.
    rows.push({ term: translate("what the model computes"), detail: translate(entry.boundary), block: true });
  }
  return rows;
}
