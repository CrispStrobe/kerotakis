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
 * a skin over and only the name, the picture and the preset differ.
 */
import { APPARATUS } from "./apparatus";
import type { TwoVesselAction } from "./directActions";
import type { InfoRow } from "./infoPanel";
import { INSTRUMENTS, instrumentCommand, instrumentVerb } from "./instruments";
import { KIDS_EQUIPMENT } from "./kidsEquipment";
import { TRANSFER_TOOLS } from "./transferTools";

/** What the thing does — the shelf it stands on. */
export type EquipmentGroup = "measure" | "thermal" | "contain" | "separate" | "drive" | "sets";

/** The shelves, in the order the cupboard shows them. */
export const EQUIPMENT_GROUPS: readonly EquipmentGroup[] = [
  "measure",
  "thermal",
  "contain",
  "separate",
  "drive",
  "sets",
];

/** Shelf headings. Literals, so the translation scan can see them. */
export const GROUP_LABELS: Record<EquipmentGroup, string> = {
  measure: "observe and measure",
  thermal: "heat and cool",
  contain: "contain and connect",
  separate: "transfer and separation",
  drive: "drive and power",
  sets: "children's activity kits",
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
 * decision and a derivation would hide it. `grind` is filed under separation
 * because grain size is what a learner changes in order to separate or
 * dissolve something, not because a mortar separates anything by itself.
 */
const APPARATUS_GROUPS: Record<string, EquipmentGroup> = {
  bunsen: "thermal",
  heat: "thermal",
  cool: "thermal",
  evaporate: "thermal",
  dilute: "contain",
  regulate: "contain",
  sweep: "contain",
  grind: "separate",
  centrifuge: "drive",
  electrolyse: "drive",
  irradiate: "drive",
  stir: "drive",
};

const TRANSFER_GROUPS: Record<TwoVesselAction, EquipmentGroup> = {
  filter: "separate",
  decant: "separate",
  drain: "separate",
  magnet: "separate",
  distil: "separate",
  cell: "drive",
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
    group: APPARATUS_GROUPS[item.verb] ?? "drive",
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
    id: "transport",
    group: "separate",
    name: "column train",
    blurb: "move solution through connected cells",
    boundary: boundaryOf("transport"),
    render: { kind: "icon", name: "transport" },
    action: { kind: "install", verb: "transport" },
  },
  {
    id: "react",
    group: "drive",
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

const kitEntries = (): EquipmentEntry[] =>
  KIDS_EQUIPMENT.map((item) => ({
    id: item.id,
    group: "sets" as const,
    name: item.title,
    blurb: item.blurb,
    boundary: item.boundary,
    render: { kind: "icon" as const, name: item.icon },
    action: kitAction(item.engineVerb, item.preset),
    aliasOf: item.engineVerb,
    parts: item.parts,
  }));

/** Everything, once. The order within a shelf is the order it is declared. */
export const EQUIPMENT_CATALOGUE: EquipmentEntry[] = [
  ...instrumentEntries(),
  ...apparatusEntries(),
  ...transferEntries(),
  ...SPECIAL_ENTRIES,
  ...kitEntries(),
];

/** The id availability is read for: a kit borrows its skinned tool's. */
export const accessId = (entry: EquipmentEntry): string => entry.aliasOf ?? entry.id;

export const equipmentIn = (group: EquipmentGroup): EquipmentEntry[] =>
  EQUIPMENT_CATALOGUE.filter((entry) => entry.group === group);

export const equipmentById = (id: string): EquipmentEntry | undefined =>
  EQUIPMENT_CATALOGUE.find((entry) => entry.id === id);

/** The ids the wall tallies. Kits are skins, not extra equipment to unlock. */
export const gatedIds = (reactAvailable: boolean): string[] =>
  EQUIPMENT_CATALOGUE
    .filter((entry) => entry.group !== "sets" && (reactAvailable || entry.id !== "react"))
    .map((entry) => entry.id);

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
