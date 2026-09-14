/** BRD-002: what the cabinet card says about a bottle.
 *
 * The engine owns the number. This file owns only the two questions the
 * shelf asks of it — is this key limited at all, and is it empty — plus
 * the rounding that turns 39.999999999 g into "40 g" without ever
 * rounding a nearly-empty bottle up to a full one.
 *
 * Deliberately separate from `storyStock.ts`. That ledger counts
 * *dispenses* in the story mode's own progression and lives in the
 * browser; this one is the engine's conserved amount, arrives on the
 * scene, and is restored by undo along with everything else.
 */
import type { SceneStockBottle } from "./host/EngineHost";
import type { CatalogAccess } from "./catalogProgress";

export type StockLevels = Readonly<Record<string, SceneStockBottle>>;

/** Index the scene's list by shelf key. An absent key is unlimited. */
export function stockLevels(bottles: readonly SceneStockBottle[] | undefined): StockLevels {
  const out: Record<string, SceneStockBottle> = {};
  for (const bottle of bottles ?? []) {
    if (typeof bottle?.key === "string" && Number.isFinite(bottle.remaining)) {
      out[bottle.key] = bottle;
    }
  }
  return out;
}

/** True only for a bottle the engine tracks AND has emptied. An untracked
 * key is never exhausted — that is the sandbox, not an empty shelf. */
export function isExhausted(bottle: SceneStockBottle | undefined): boolean {
  return bottle !== undefined && bottle.remaining <= 0;
}

/** The amount as a label reads it: enough digits to stay honest near
 * empty, never so many that a shelf row turns into a readout.
 *
 * The floor at four decimals matters — a bottle with 0.0004 mol left is
 * not empty, and showing "0" for it would be the lie this whole feature
 * exists to avoid. */
export function formatStockAmount(remaining: number): string {
  if (!Number.isFinite(remaining) || remaining <= 0) return "0";
  if (remaining >= 100) return String(Math.round(remaining));
  if (remaining >= 1) return String(Math.round(remaining * 10) / 10);
  const rounded = Math.round(remaining * 10000) / 10000;
  // Never round a real remainder down to nothing.
  return String(rounded > 0 ? rounded : 0.0001);
}

/** The badge text: "40 g left", or null when nothing limits this key. */
export function stockBadge(
  bottle: SceneStockBottle | undefined,
  translate: (key: string, values?: Record<string, string | number>) => string,
): string | null {
  if (!bottle) return null;
  if (isExhausted(bottle)) return translate("empty");
  return translate("{amount} {unit} left", {
    amount: formatStockAmount(bottle.remaining),
    unit: bottle.unit,
  });
}

/**
 * Why a material cannot be taken from the shelf, in the learner's language.
 *
 * Extracted from the markup because the branch is the defect: the count
 * arm used to be the fallback, so an access with nothing to say about a
 * count still rendered "Permanent stock unlocks after 0 completed
 * missions" — a sentence no progression could ever produce and one the
 * owner read in Sandbox, where nothing is gated at all. A count is
 * printed only when there IS one; zero means the cabinet has not answered
 * for this material, and the shelf says that instead of inventing a
 * milestone.
 *
 * Null for an available material: a row that can be poured has no reason
 * to explain.
 */
export function lockNote(
  access: CatalogAccess,
  translate: (key: string, values?: Record<string, string | number>) => string,
): string | null {
  if (access.available) return null;
  if (access.missionOnly) {
    return translate("Supervised mission kit only — this material never becomes permanent Story stock.");
  }
  if (access.minimumCompleted === 1) {
    return translate("Permanent stock unlocks after one completed mission. Mission kits loan required materials.");
  }
  if (access.minimumCompleted > 1) {
    return translate("Permanent stock unlocks after {count} completed missions. Mission kits loan required materials.", {
      count: access.minimumCompleted,
    });
  }
  return translate("The supply cabinet has not said anything about this material yet. Mission kits still loan required materials.");
}
