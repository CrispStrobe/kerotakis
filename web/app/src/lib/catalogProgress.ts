/**
 * WORLD-003, client half: availability is READ, not recomputed.
 *
 * This file used to hold the progression rules — a milestone per verb, a
 * hazard ladder per material — and compute availability from them. The
 * engine holds them now (`kerotakis_core::catalog`), and every host answers
 * from that one table, so what remains here is lookup and presentation.
 *
 * The tables did not move because duplication is untidy. They moved because
 * a rule with two copies eventually disagrees with itself, and the copy the
 * learner sees was the one that could not be tested against the engine that
 * would actually refuse them. While both existed a fixture pinned them to
 * each other; with one gone, so is the fixture.
 */
import type { CatalogItem } from "./host/EngineHost";

export type CatalogAccess = {
  available: boolean;
  loaned: boolean;
  /** Permanently earned by closing a case, rather than reached by count. */
  granted: boolean;
  minimumCompleted: number;
};

/** The engine's answer, indexed by stable id. */
export type CatalogMap = ReadonlyMap<string, CatalogItem>;

export function catalogMap(items: readonly CatalogItem[]): CatalogMap {
  return new Map(items.map((item) => [item.id, item]));
}

/**
 * What the engine said about one id.
 *
 * Returns null when the catalog has not arrived yet — a caller must decide
 * what to show while the engine is still loading, rather than being handed a
 * confident `false` that looks like a refusal.
 */
export function access(catalog: CatalogMap, id: string): CatalogAccess | null {
  const item = catalog.get(id);
  if (item === undefined) return null;
  return {
    available: item.available,
    loaned: item.reason.reason === "loaned",
    granted: item.reason.reason === "awarded",
    minimumCompleted: item.minimum_completed,
  };
}

/**
 * Cabinet access, which differs from `access` in exactly one case.
 *
 * The engine tiers equipment. It deliberately does NOT tier bench controls
 * — `kerotakis_core::catalog::NOT_CABINET` holds `cool`, `wait`, `open`,
 * `seal` and the rest — because they are not things a learner earns. Those
 * verbs therefore have no catalog row at all, and `access` cannot tell "the
 * engine gates this and the answer is no" apart from "the engine does not
 * gate this at all": both are a missing key.
 *
 * The instrument wall read that silence as a refusal. `cool` is a card in
 * the cabinet (the cooling bath), so in Sandbox — where the engine derives
 * everything as reachable — the card was still disabled, and the wall's own
 * tally said 33/34 while the sentence under it promised that everything was
 * available. Both halves of that contradiction are this one lookup.
 *
 * So: an id a LOADED catalog does not mention is ungated — reachable, at
 * tier zero, neither loaned nor awarded. While the catalog is still empty
 * nothing at all is known, and the caller gets the same conservative answer
 * `access` would have given rather than a promise the engine never made.
 */
export function equipmentAccess(catalog: CatalogMap, id: string): CatalogAccess {
  const answered = access(catalog, id);
  if (answered) return answered;
  return {
    available: catalog.size > 0,
    loaned: false,
    granted: false,
    minimumCompleted: 0,
  };
}

/** Availability alone, with an unloaded catalog reading as not-yet-available. */
export function available(catalog: CatalogMap, id: string): boolean {
  return catalog.get(id)?.available ?? false;
}

/** The progress that would earn this id, for the "after N missions" label.
 * Null while the catalog is unloaded, so the label can stay silent rather
 * than promise a number it does not have. */
export function requirement(catalog: CatalogMap, id: string): number | null {
  return catalog.get(id)?.minimum_completed ?? null;
}

/** Instruments are addressed `measure:<token>` in the catalog's id space. */
export function instrumentId(token: string): string {
  return `measure:${token}`;
}

export type EquipmentReward = { verb: string; title: string; description: string };

/** Milestone rewards are PRESENTATION: what the debrief celebrates when a
 * count is reached. The catalog decides what is reachable; this decides what
 * is worth a card. */
const REWARDS: Record<number, EquipmentReward> = {
  1: { verb: "evaporate", title: "evaporating dish", description: "Concentrate solutions and recover dissolved solids." },
  2: { verb: "regulate", title: "piston lid", description: "Control pressure and headspace above a vessel." },
  3: { verb: "electrolyse", title: "electrodes and supply", description: "Drive and measure electrochemical change." },
  4: { verb: "distil", title: "still", description: "Separate liquids through a connected distillation rig." },
};

export function equipmentRewardAt(completed: number): EquipmentReward | null {
  return REWARDS[completed] ?? null;
}
