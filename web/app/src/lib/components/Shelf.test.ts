/**
 * The shelf, RENDERED, in both laboratories.
 *
 * The owner read "Der dauerhafte Vorrat wird nach 0 abgeschlossenen
 * Missionen freigeschaltet" — the permanent stock after zero missions — in
 * SANDBOX, where nothing is gated at all. Two faults stacked: the shelf
 * asked the raw catalog lookup rather than the mode-aware one, and it
 * turned that lookup's null (the engine has not answered) into a confident
 * refusal carrying a fabricated count of zero.
 *
 * Rendered rather than unit-tested through the helper alone, because the
 * defect was in the wiring: `shelfAccess` could be perfect and the
 * component could still call `access()` beside it. An empty catalog is the
 * exact condition that produced it — one dropped round trip, or a
 * service-worker cache pairing a new app with an engine that has no
 * `catalog` method at all, and every bottle wears a lock.
 */
import { describe, expect, it } from "vitest";
import { render } from "svelte/server";
import Shelf from "./Shelf.svelte";
import { catalogMap } from "../catalogProgress";
import type { CatalogItem } from "../host/EngineHost";
import type { ShelfItem } from "../session.svelte";

const items: ShelfItem[] = [
  { key: "water", name: "water", formula: "H2O", phase: "liquid" },
  { key: "NaCl", name: "table salt", formula: "NaCl", phase: "solid" },
  { key: "HCl", name: "hydrochloric acid", formula: "HCl", phase: "liquid" },
];

const catalogue = (mode: "story" | "sandbox") => catalogMap([
  ...items.map((item): CatalogItem => ({
    id: item.key,
    kind: "reagent",
    minimum_completed: item.key === "HCl" ? 3 : 0,
    available: mode === "sandbox" || item.key !== "HCl",
    reason: mode === "sandbox"
      ? { reason: "sandbox" }
      : item.key === "HCl"
        ? { reason: "locked", minimum_completed: 3 }
        : { reason: "earned", minimum_completed: 0 },
  })),
]);

function body(props: Record<string, unknown>): string {
  return render(Shelf, {
    props: {
      items,
      register: "lv2",
      target: 0,
      onadd: () => {},
      catalog: catalogMap([]),
      cabinet: "pending",
      ...props,
    },
  }).body;
}

/** How many rows wear the lock. The chip and the class are one state. */
function locked(html: string): number {
  return (html.match(/class="[^"]*\blocked\b[^"]*"/g) ?? []).length;
}

describe("the shelf in Sandbox", () => {
  it("locks nothing, even before the engine has answered", () => {
    // The reported screen: Sandbox, and a catalog that never arrived.
    expect(locked(body({ mode: "sandbox", catalog: catalogMap([]) }))).toBe(0);
  });

  it("locks nothing the engine itself would gate in Story", () => {
    // A Sandbox response derives everything as reachable; a client that
    // paints a lock over a Story answer is disagreeing with the engine.
    expect(locked(body({ mode: "sandbox", catalog: catalogue("story") }))).toBe(0);
  });

  // The sentence itself is only rendered for an OPENED row, which a server
  // render never has; `lockNote` in shelfStock.test.ts is where the words
  // are pinned, and tools/test-ux-quality.mjs reads them off a real screen.
});

describe("a shelf with nothing on it says which nothing it is", () => {
  it("blames the filter when the filter is to blame", () => {
    // The catalogue answered and a filter is what emptied the list, so
    // the filter is what the sentence is allowed to blame.
    const html = body({ mode: "story", scope: "mission", kit: [], catalog: catalogue("story"), cabinet: "answered" });
    expect(html).toContain("nothing on the shelf matches");
  });

  it("blames the cabinet when the cabinet is to blame", () => {
    // Story opens on the "unlocked" scope, whose filter asks `available`,
    // which answers no for everything while the catalogue is silent. The
    // shelf then reads "nothing on the shelf matches" over a cabinet of
    // 188 bottles, with the filter apparently at fault.
    const html = body({ mode: "story", scope: "unlocked", catalog: catalogMap([]), cabinet: "unanswered" });
    expect(html).toContain("did not answer");
    expect(html).not.toContain("nothing on the shelf matches");
  });

  it("keeps saying 'not yet' while the asks are still in flight", () => {
    const html = body({ mode: "story", scope: "unlocked", catalog: catalogMap([]), cabinet: "pending" });
    expect(html).toContain("nothing on the shelf matches");
    expect(html).not.toContain("did not answer");
  });
});

describe("the shelf in Story", () => {
  it("shows the engine's own gate", () => {
    // One of the three is locked at three missions, and it is the one the
    // engine said so about.
    expect(locked(body({ mode: "story", catalog: catalogue("story") }))).toBe(1);
  });

  it("still refuses what the catalog has not answered for", () => {
    // Conservative on purpose: the hazard ladder is not a thing to open on
    // a guess. What changed is the sentence under it, which no longer
    // promises stock after zero missions — see `lockNote`.
    expect(locked(body({ mode: "story", catalog: catalogMap([]) }))).toBe(items.length);
  });
});
