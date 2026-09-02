import type { ShelfItem } from "./session.svelte";
import type { LabMode } from "./worldState";

export type CatalogAccess = {
  available: boolean;
  loaned: boolean;
  /** Permanently earned by closing a case, rather than reached by count. */
  granted?: boolean;
  minimumCompleted: number;
};

const EQUIPMENT_MILESTONES: Record<string, number> = {
  bunsen: 1,
  burette: 0,
  stir: 0,
  heat: 0,
  centrifuge: 0,
  dilute: 0,
  grind: 0,
  filter: 0,
  decant: 0,
  mix: 0,
  evaporate: 1,
  drain: 1,
  magnet: 1,
  react: 1,
  regulate: 2,
  irradiate: 2,
  electrolyse: 3,
  cell: 3,
  distil: 4,
  transport: 4,
  sweep: 4,
  "measure:eyes": 0,
  "measure:thermometer": 0,
  "measure:ph": 0,
  "measure:balance": 0,
  "measure:smell": 1,
  "measure:volume": 1,
  "measure:conductivity": 1,
  "measure:pressure": 2,
  "measure:calorimeter": 2,
  "measure:uvvis": 3,
  "measure:chromatograph": 3,
  "measure:geiger": 4,
};

const STARTER_STOCK = new Set([
  "water", "NaCl", "CH3COOH", "NaHCO3", "CaCO3", "MgSO4", "CaCl2",
]);

export type EquipmentReward = { verb: string; title: string; description: string };

const REWARDS: Record<number, EquipmentReward> = {
  1: { verb: "evaporate", title: "evaporating dish", description: "Concentrate solutions and recover dissolved solids." },
  2: { verb: "regulate", title: "piston lid", description: "Control pressure and headspace above a vessel." },
  3: { verb: "electrolyse", title: "electrodes and supply", description: "Drive and measure electrochemical change." },
  4: { verb: "distil", title: "still", description: "Separate liquids through a connected distillation rig." },
};

export function equipmentRequirement(verb: string): number {
  return EQUIPMENT_MILESTONES[verb] ?? 4;
}

/** Instruments a closed case granted permanently, regardless of mission count. */
const NO_AWARDS: ReadonlySet<string> = new Set();

export function equipmentAvailable(
  mode: LabMode,
  completed: number,
  verb: string,
  awarded: ReadonlySet<string> = NO_AWARDS,
): boolean {
  return mode === "sandbox" || completed >= equipmentRequirement(verb) || awarded.has(verb);
}

export function equipmentAccess(
  mode: LabMode,
  completed: number,
  verb: string,
  inMissionKit: boolean,
  awarded: ReadonlySet<string> = NO_AWARDS,
): CatalogAccess {
  const minimumCompleted = equipmentRequirement(verb);
  const loaned = mode === "story" && inMissionKit;
  // An award is permanent and unconditional: it outranks the milestone that
  // would otherwise still be years of missions away, and unlike a loan it
  // does not expire with the mission that granted it.
  const granted = awarded.has(verb);
  return {
    minimumCompleted,
    loaned,
    granted,
    available: mode === "sandbox" || completed >= minimumCompleted || loaned || granted,
  };
}

export function equipmentRewardAt(completed: number): EquipmentReward | null {
  return REWARDS[completed] ?? null;
}

export function reagentRequirement(item: Pick<ShelfItem, "key" | "hazards" | "hazard_assessed">): number {
  if (STARTER_STOCK.has(item.key)) return 0;
  if (item.hazard_assessed === false) return 4;
  const hazards = new Set(item.hazards ?? []);
  if (hazards.has("toxic") || hazards.has("corrosive")) return 3;
  if (hazards.has("flammable") || hazards.has("oxidizer")) return 2;
  return 1;
}

export function reagentAccess(
  mode: LabMode,
  completed: number,
  item: Pick<ShelfItem, "key" | "hazards" | "hazard_assessed">,
  inMissionKit: boolean,
): CatalogAccess {
  const minimumCompleted = reagentRequirement(item);
  // An accepted mission supplies its whole kit, including common materials;
  // learners never have to spend permanent stock to follow an investigation.
  const loaned = mode === "story" && inMissionKit;
  return {
    minimumCompleted,
    loaned,
    available: mode === "sandbox" || completed >= minimumCompleted || loaned,
  };
}
