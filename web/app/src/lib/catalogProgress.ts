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
import type { LabMode } from "./worldState";

export type CatalogAccess = {
  available: boolean;
  loaned: boolean;
  /** Permanently earned by closing a case, rather than reached by count. */
  granted: boolean;
  /** Story access is deliberately limited to an authored mission kit. */
  missionOnly: boolean;
  minimumCompleted: number;
};

/**
 * Whether the cabinet has answered at all, which is a different question
 * from what it answered.
 *
 * `shelfAccess` collapses "no answer" into an unavailable row, because an
 * unanswered row is the only one it can safely paint. What it cannot carry
 * is WHY there is no answer, and the two whys read very differently: a
 * catalog still in flight becomes a shelf a moment later, and a catalog
 * that was asked to exhaustion and never replied never will. PR #599 made
 * the first honest; this is the second.
 */
export type CabinetStatus =
  /** Asked, still waiting. Every session starts here. */
  | "pending"
  /** The engine replied, and `catalog` is what it said. */
  | "answered"
  /** Asked to exhaustion and never answered. The shelf says so. */
  | "unanswered";

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
    missionOnly: item.reason.reason === "mission_only",
    minimumCompleted: item.minimum_completed,
  };
}

/**
 * The answer for a laboratory that gates nothing: reachable, and nothing
 * lent or awarded it, because nothing had to be.
 *
 * `minimumCompleted: 0` means "there is no count to report", never "after
 * zero missions". The engine cannot emit a locked row with a minimum of
 * zero — `kerotakis_core::catalog::decide` reaches `Locked { minimum }`
 * only when `completed < minimum`, and `completed` is never negative — so
 * any label that renders a zero is reporting a client-invented refusal
 * rather than an engine answer.
 */
const UNGATED: CatalogAccess = {
  available: true,
  loaned: false,
  granted: false,
  missionOnly: false,
  minimumCompleted: 0,
};

/** Nothing is known yet: not reachable, and with no count to justify it. */
const UNANSWERED: CatalogAccess = { ...UNGATED, available: false };

/**
 * Material access, mode first.
 *
 * Three facts decide a shelf row, and the first two are not in the catalog
 * at all:
 *
 * 1. **Sandbox gates nothing.** That is the whole difference between the
 *    two laboratories (PR #517, and `catalog::decide` answers Sandbox
 *    before it looks at anything else). A shelf that asks the catalog
 *    first can still paint a lock there — it did, on all 188 bottles,
 *    under a sentence saying the stock unlocks after zero missions.
 * 2. **Silence is not a refusal.** A catalog that has not arrived, or one
 *    that does not mention this id, has said nothing. The shelf shows the
 *    row as not-yet-known and says so in words; it must not invent the
 *    count that would unlock it. This is the same distinction
 *    `equipmentAccess` draws for the cabinet, and the same one
 *    `Session.submit` already drew at the bench, where a null answer is
 *    explicitly not allowed to refuse a pour.
 * 3. Otherwise the engine's answer, unchanged.
 *
 * `mode` is required rather than defaulted on purpose. Every instance of
 * this defect so far — the codex badge, the concept map's district gate,
 * this shelf — was a gate that simply never asked which laboratory it was
 * in, and a parameter with a default is a question a new call site can
 * forget to answer.
 */
export function shelfAccess(catalog: CatalogMap, id: string, mode: LabMode): CatalogAccess {
  if (mode === "sandbox") return UNGATED;
  return access(catalog, id) ?? UNANSWERED;
}

/**
 * Cabinet access, which differs from `shelfAccess` in exactly one case.
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
 *
 * Materials are the other way round, which is why they have their own
 * helper: the catalog holds a row for every species, so a missing one is a
 * disagreement rather than a deliberate omission, and a hazard ladder is
 * not a thing to open on a guess.
 */
export function equipmentAccess(catalog: CatalogMap, id: string, mode: LabMode): CatalogAccess {
  if (mode === "sandbox") return UNGATED;
  const answered = access(catalog, id);
  if (answered) return answered;
  return { ...UNGATED, available: catalog.size > 0 };
}

/**
 * Availability alone, with an unloaded catalog reading as not-yet-available.
 *
 * Mode-blind by design and named for what it collapses: it answers only
 * what the engine said. Every caller that decides whether to REFUSE
 * something must go through `shelfAccess` or `equipmentAccess` instead, or
 * ask the mode itself first — a lone `available()` in Sandbox is the shape
 * this defect keeps coming back in.
 */
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
