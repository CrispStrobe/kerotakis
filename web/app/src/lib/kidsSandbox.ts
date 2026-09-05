import type { KidsExperiment } from "./kidsCatalog";

export const PENDING_KIDS_SANDBOX_KEY = "kerotakis.kids.pending-sandbox.v1";

export type KidsSandboxBrief = Pick<KidsExperiment, "id" | "title" | "ingredients" | "apparatus" | "boundary">;

type StorageLike = Pick<Storage, "getItem" | "setItem" | "removeItem">;

const SHELF_ALIASES: Record<string, string> = {
  cola: "cola_drink",
  cooking_oil: "vegetable_oil",
  food_colouring: "food_colour_red",
  milk: "whole_milk",
  paper: "paper_sheet",
  pepper: "ground_black_pepper",
  sand: "quartz_sand",
  starch: "corn_starch",
  yeast: "dry_yeast",
};

/** Turn display-friendly KIDS ingredient names into actual shelf keys. */
export function kidsShelfKeys(ingredients: readonly string[]): string[] {
  return [...new Set(ingredients.map((key) => SHELF_ALIASES[key] ?? key))];
}

const EQUIPMENT_ALIASES: Record<string, string> = {
  balance: "measure:balance",
  burette: "burette",
  chromatograph: "measure:chromatograph",
  density: "measure:density",
  look: "measure:eyes",
  ph: "measure:ph",
  thermometer: "measure:thermometer",
};

/** Translate KIDS apparatus labels into cabinet affordance tokens. */
export function kidsEquipmentVerbs(apparatus: readonly string[]): string[] {
  return [...new Set(apparatus.map((key) => EQUIPMENT_ALIASES[key] ?? key))];
}

export function briefFor(experiment: KidsExperiment): KidsSandboxBrief {
  return {
    id: experiment.id,
    title: experiment.title,
    ingredients: [...experiment.ingredients],
    apparatus: [...experiment.apparatus],
    ...(experiment.boundary ? { boundary: experiment.boundary } : {}),
  };
}

export function storePendingKidsSandbox(storage: StorageLike | null, brief: KidsSandboxBrief): void {
  storage?.setItem(PENDING_KIDS_SANDBOX_KEY, JSON.stringify(brief));
}

export function takePendingKidsSandbox(storage: StorageLike | null): KidsSandboxBrief | null {
  if (!storage) return null;
  const raw = storage.getItem(PENDING_KIDS_SANDBOX_KEY);
  if (raw === null) return null;
  storage.removeItem(PENDING_KIDS_SANDBOX_KEY);
  try {
    const value = JSON.parse(raw) as Partial<KidsSandboxBrief>;
    if (!/^K\d{2}$/.test(value.id ?? "") || typeof value.title !== "string"
      || !Array.isArray(value.ingredients) || !value.ingredients.every((item) => typeof item === "string")
      || !Array.isArray(value.apparatus) || !value.apparatus.every((item) => typeof item === "string")
      || (value.boundary !== undefined && typeof value.boundary !== "string")) return null;
    return value as KidsSandboxBrief;
  } catch {
    return null;
  }
}
