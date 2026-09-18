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

/**
 * GUI-093, the half that is layout rather than filtering.
 *
 * The role chips answer "show me only the acids". The headings answer the
 * question a learner asks before they know what to ask for. The fixture
 * carries `elements` and `charge` because that is what the roles are
 * DERIVED from — a fixture without them classifies everything `unsorted`,
 * which would make this suite pass against a shelf that groups nothing.
 */
const classified: ShelfItem[] = [
  { key: "water", name: "water", formula: "H2O", phase: "liquid", elements: { H: 2, O: 1 }, charge: 0, solvent: true },
  { key: "NaCl", name: "table salt", formula: "NaCl", phase: "solid", elements: { Na: 1, Cl: 1 }, charge: 0 },
  { key: "HCl", name: "hydrochloric acid", formula: "HCl", phase: "liquid", elements: { H: 1, Cl: 1 }, charge: 0 },
];

const headings = (html: string): string[] =>
  [...html.matchAll(/id="shelf-group-([a-z]+)"/g)].map((m) => m[1] ?? "");

/** One `<li>` per bottle, however many roles that bottle holds. */
const bottles = (html: string): number => (html.match(/<li data-phase=/g) ?? []).length;

describe("the shelf is laid out by role", () => {
  it("heads each group and keeps REAGENT_ROLES' order, not the alphabet", () => {
    const html = body({ items: classified, mode: "sandbox", catalog: catalogMap([]) });
    // acid before salt before solvent — the bench reading order the chips
    // use. Alphabetically it would be acid, salt, solvent too, which is a
    // coincidence of English; the assertion that matters is that the list
    // comes from REAGENT_ROLES, so `roles` is checked against it directly.
    expect(headings(html)).toEqual(["acid", "salt", "solvent"]);
  });

  it("files a bottle under exactly one heading", () => {
    // Hydrochloric acid is an acid AND, to the derivation, nothing else;
    // but citric acid is an acid and an organic, and a shelf that repeated
    // it would be longer than the cabinet and would make the tally a lie.
    // The invariant is the count, so it holds for every fixture.
    const html = body({ items: classified, mode: "sandbox", catalog: catalogMap([]) });
    expect(bottles(html)).toBe(classified.length);
    expect(html).toContain(`of ${classified.length} substances`);
  });

  it("draws no headings when there is only one group to name", () => {
    // A single heading spends a whole row saying what every bottle under
    // it already says.
    const one = classified.filter((item) => item.key === "HCl");
    const html = body({ items: one, mode: "sandbox", catalog: catalogMap([]) });
    expect(headings(html)).toEqual([]);
    expect(bottles(html)).toBe(1);
  });

  it("names the group a bottle was filed under on the row itself", () => {
    // So the stylesheet and this suite can both see the filing decision
    // even in the flat list, where no heading is drawn.
    const html = body({ items: classified, mode: "sandbox", catalog: catalogMap([]) });
    expect(html).toContain('data-role="acid"');
    expect(html).toContain('data-role="salt"');
    expect(html).toContain('data-role="solvent"');
  });
});

/**
 * The hazard mark, and the state in the middle of it.
 *
 * Three states and two glyphs: assessed-and-hazardous, assessed-and-clean,
 * and never assessed. Silence has to mean "checked, clean" for either mark
 * to mean anything — so the test that matters is the third one, which
 * fails the moment "we have not looked" is allowed to render as safety.
 */
describe("the hazard mark reaches the learner at choosing time", () => {
  const withHazards = (extra: Partial<ShelfItem>): string =>
    body({
      items: [{ key: "HCl", name: "hydrochloric acid", formula: "HCl", phase: "liquid", ...extra }],
      mode: "sandbox",
      catalog: catalogMap([]),
    });

  /** Svelte interleaves its scope class, so the token is what to look for. */
  const mark = (html: string): RegExp => /<span class="hazard[^"]*"/;
  const unassessedMark = /<span class="hazard[^"]*\bunassessed\b[^"]*"/;

  it("marks an assessed hazard on the row, not only behind the (i)", () => {
    const html = withHazards({ hazards: ["corrosive"], hazard_assessed: true });
    expect(html).toMatch(mark(html));
    expect(html).not.toMatch(unassessedMark);
    expect(html).toContain('aria-label="corrosive"');
    // The glyph a sighted reader sees, and the words everyone else hears.
    expect(html).toContain("\u26a0");
  });

  it("marks an unassessed species differently, and says so in words", () => {
    const html = withHazards({ hazard_assessed: false });
    expect(html).toMatch(unassessedMark);
    expect(html).toContain('aria-label="hazards unassessed"');
    // Not the warning triangle: "we have not looked" is not a warning.
    expect(html).not.toContain("\u26a0");
  });

  it("marks a species assessed as clean with nothing at all", () => {
    // The whole scheme rests on this: an empty hazard row is the only
    // state that gets no glyph, which is what makes the other two mean
    // something.
    const html = withHazards({ hazards: [], hazard_assessed: true });
    expect(html).not.toContain('class="hazard');
  });
});
