/**
 * WORLD-003, client half: availability is READ, not recomputed.
 *
 * These used to be tests of a milestone table this file owned, pinned
 * against the engine's copy by a shared fixture. Both the table and the pin
 * are gone: the engine answers, and the client shows the answer. What is
 * left to test is that the client reads the answer faithfully — including
 * the case that has no answer yet.
 */
import { describe, expect, it } from "vitest";
import { access, available, catalogMap, equipmentRewardAt, instrumentId, requirement } from "./catalogProgress";
import type { CatalogItem } from "./host/EngineHost";

const item = (
  id: string,
  overrides: Partial<CatalogItem> = {},
): CatalogItem => ({
  id,
  kind: "apparatus",
  minimum_completed: 0,
  available: true,
  reason: { reason: "earned", minimum_completed: 0 },
  ...overrides,
});

const CATALOG = catalogMap([
  item("filter"),
  item("distil", {
    minimum_completed: 4,
    available: false,
    reason: { reason: "locked", minimum_completed: 4 },
  }),
  item("measure:uvvis", {
    kind: "instrument",
    minimum_completed: 3,
    available: true,
    reason: { reason: "awarded" },
  }),
  item("drain", {
    minimum_completed: 1,
    available: true,
    reason: { reason: "loaned" },
  }),
  item("HCl", {
    kind: "reagent",
    minimum_completed: 3,
    available: false,
    reason: { reason: "locked", minimum_completed: 3 },
  }),
]);

describe("the client reads the engine's catalog", () => {
  it("reports availability as the engine gave it", () => {
    expect(available(CATALOG, "filter")).toBe(true);
    expect(available(CATALOG, "distil")).toBe(false);
    expect(available(CATALOG, "HCl")).toBe(false);
  });

  it("distinguishes a permanent award from a mission's loan", () => {
    // Both are available; only one survives leaving the mission, and the
    // cabinet says which — so a learner is told they earned a thing rather
    // than that something lent it to them.
    const awarded = access(CATALOG, "measure:uvvis")!;
    expect(awarded.available).toBe(true);
    expect(awarded.granted).toBe(true);
    expect(awarded.loaned).toBe(false);

    const loaned = access(CATALOG, "drain")!;
    expect(loaned.available).toBe(true);
    expect(loaned.loaned).toBe(true);
    expect(loaned.granted).toBe(false);
  });

  it("carries the milestone through for the 'after N missions' label", () => {
    expect(requirement(CATALOG, "distil")).toBe(4);
    expect(access(CATALOG, "HCl")!.minimumCompleted).toBe(3);
  });

  it("says nothing rather than guessing about an id the engine did not mention", () => {
    // The important case: before the engine answers, the catalog is empty.
    // `access` returns null so a caller can render "not yet known", and
    // `requirement` returns null so a label cannot promise a number it does
    // not have. Only `available` collapses to false, and it is named for it.
    const empty = catalogMap([]);
    expect(access(empty, "filter")).toBeNull();
    expect(requirement(empty, "filter")).toBeNull();
    expect(available(empty, "filter")).toBe(false);
    expect(access(CATALOG, "no-such-thing")).toBeNull();
  });

  it("addresses instruments the way the catalog does", () => {
    expect(instrumentId("uvvis")).toBe("measure:uvvis");
    expect(available(CATALOG, instrumentId("uvvis"))).toBe(true);
  });
});

describe("milestone rewards remain presentation", () => {
  it("still names what a completion count is worth celebrating", () => {
    // The catalog decides what is REACHABLE; this decides what earns a card
    // in the debrief. They are different questions, so this one stayed.
    expect(equipmentRewardAt(1)?.verb).toBe("evaporate");
    expect(equipmentRewardAt(4)?.verb).toBe("distil");
    expect(equipmentRewardAt(7)).toBeNull();
  });
});
