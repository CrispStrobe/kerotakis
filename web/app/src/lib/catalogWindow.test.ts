/**
 * The window, without a browser.
 *
 * The suite has no DOM, so the only way this behaviour can be checked at
 * all is for the arithmetic to be pure and the component to own nothing
 * but the measuring. These pin the parts a reader would notice going
 * wrong: the scrollbar length, the range that is drawn, the filtered
 * total the count is taken from, and the focused card staying in the DOM
 * when the reader has scrolled a long way from it.
 */
import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import {
  CATALOG_ESTIMATE,
  CATALOG_OVERSCAN,
  catalogWindow,
  estimateRowHeight,
  rowCountFor,
  rowTops,
  type CatalogWindowInput,
} from "./catalogWindow";

const input = (over: Partial<CatalogWindowInput> = {}): CatalogWindowInput => ({
  total: 752,
  columns: 3,
  scrollTop: 0,
  viewportTop: 0,
  viewportHeight: 800,
  heights: new Map<number, number>(),
  gap: 11.2,
  ...over,
});

describe("the drawn range", () => {
  it("draws a viewport and its overscan, not seven hundred cards", () => {
    const shape = catalogWindow(input());
    expect(shape.rowCount).toBe(251);
    // 800px of viewport over 271.2px rows (260 + the gap) is three rows,
    // plus the overscan below them.
    expect(shape.first).toBe(0);
    expect(shape.last).toBe(2 + CATALOG_OVERSCAN);
    expect(shape.drawn).toBe((shape.last + 1) * 3);
    expect(shape.drawn).toBeLessThan(40);
  });

  it("keeps overscan above once the reader has scrolled", () => {
    const shape = catalogWindow(input({ scrollTop: 10_000 }));
    expect(shape.first).toBeGreaterThan(CATALOG_OVERSCAN);
    expect(shape.last - shape.first).toBeLessThan(12);
    // Every drawn row is a real slice of the filtered array.
    for (const row of shape.rows) {
      expect(row.end).toBeGreaterThan(row.start);
      expect(row.end).toBeLessThanOrEqual(752);
    }
  });

  it("windows the FILTERED total, and the last row is short, not padded", () => {
    const shape = catalogWindow(input({ total: 7, viewportHeight: 4000 }));
    expect(shape.rowCount).toBe(3);
    expect(shape.rows.at(-1)).toMatchObject({ index: 2, start: 6, end: 7 });
    expect(shape.drawn).toBe(7);
  });

  it("has nothing to draw when the filter matches nothing", () => {
    const shape = catalogWindow(input({ total: 0 }));
    expect(shape).toMatchObject({ rows: [], height: 0, rowCount: 0, drawn: 0 });
  });

  it("clamps at both ends rather than running off the array", () => {
    const shape = catalogWindow(input({ total: 20, scrollTop: 99_999 }));
    expect(shape.last).toBe(shape.rowCount - 1);
    expect(shape.first).toBeGreaterThanOrEqual(0);
    const negative = catalogWindow(input({ total: 20, scrollTop: -500 }));
    expect(negative.first).toBe(0);
  });

  /**
   * A grid that has not been laid out reports zero, which is not the same
   * claim as "nothing is visible". Windowing on it would hide the library
   * from anything that cannot run a layout pass.
   */
  it("draws everything when there is no measurable viewport", () => {
    const shape = catalogWindow(input({ viewportHeight: 0 }));
    expect(shape.drawn).toBe(752);
    expect(shape.rows).toHaveLength(251);
  });
});

describe("rows that are not the same height", () => {
  it("uses measured heights where it has them and the mean where it does not", () => {
    const heights = new Map([[0, 400], [1, 100]]);
    expect(estimateRowHeight(heights)).toBe(250);
    expect(estimateRowHeight(new Map())).toBe(CATALOG_ESTIMATE);
    const { tops, height } = rowTops(4, heights, 250, 10);
    expect(tops).toEqual([0, 410, 520, 780]);
    expect(height).toBe(1030);
  });

  /**
   * The cost of the estimate, stated as a test rather than as a promise:
   * a measured row moves every row below it, so the scrollbar is exact
   * behind the reader and approximate ahead of them. The component
   * anchors scroll across a measurement so the CONTENT does not move.
   */
  it("moves the rows below a row whose real height has just been learnt", () => {
    const before = rowTops(5, new Map(), 200, 0).tops;
    const after = rowTops(5, new Map([[1, 500]]), 200, 0).tops;
    expect(before).toEqual([0, 200, 400, 600, 800]);
    expect(after).toEqual([0, 200, 700, 900, 1100]);
    expect(after[0]).toBe(before[0]);
  });

  it("gives a short-card filter a short scrollbar", () => {
    const questions = catalogWindow(input({ total: 500, heights: new Map([[0, 90], [1, 90]]) }));
    const experiments = catalogWindow(input({ total: 500, heights: new Map([[0, 420], [1, 420]]) }));
    expect(questions.height).toBeLessThan(experiments.height / 4);
  });
});

describe("the focused card", () => {
  it("stays in the DOM after the reader scrolls far away from it", () => {
    const far = catalogWindow(input({ scrollTop: 30_000, pinned: 2 }));
    expect(far.first).toBeGreaterThan(10);
    expect(far.rows[0]).toMatchObject({ index: 2 });
    // In document order, so the tab sequence still runs down the list.
    expect(far.rows.map((row) => row.index)).toEqual([...far.rows.map((row) => row.index)].sort((a, b) => a - b));
  });

  it("pins a row below the window too, and never twice", () => {
    const below = catalogWindow(input({ scrollTop: 0, pinned: 200 }));
    expect(below.rows.at(-1)).toMatchObject({ index: 200 });
    const inside = catalogWindow(input({ scrollTop: 0, pinned: 1 }));
    expect(inside.rows.filter((row) => row.index === 1)).toHaveLength(1);
  });

  it("ignores a pin that is not a row", () => {
    expect(catalogWindow(input({ pinned: null })).rows[0]?.index).toBe(0);
    expect(catalogWindow(input({ total: 10, pinned: 99 })).rows.at(-1)?.index).toBe(3);
    expect(catalogWindow(input({ pinned: -1 })).rows[0]?.index).toBe(0);
  });
});

describe("the grid the CSS actually made", () => {
  it("chunks by the resolved column count, and survives a nonsense one", () => {
    expect(rowCountFor(752, 3)).toBe(251);
    expect(rowCountFor(752, 1)).toBe(752);
    expect(rowCountFor(0, 3)).toBe(0);
    expect(rowCountFor(752, 0)).toBe(752);
    expect(rowCountFor(752, Number.NaN)).toBe(752);
  });

  it("re-chunks when the window narrows to one column", () => {
    const wide = catalogWindow(input({ columns: 3, viewportHeight: 600 }));
    const narrow = catalogWindow(input({ columns: 1, viewportHeight: 600 }));
    expect(narrow.rowCount).toBe(752);
    expect(narrow.drawn).toBeLessThan(wide.drawn);
  });

  it("measures the grid from where it starts, not from the top of the dialog", () => {
    // The scroller holds a header and a filter rail above the grid, so a
    // window that ignored the offset would draw the wrong rows by exactly
    // that much.
    const offset = catalogWindow(input({ scrollTop: 2000, viewportTop: 2000 }));
    expect(offset.first).toBe(0);
    expect(catalogWindow(input({ scrollTop: 2000, viewportTop: 0 })).first).toBeGreaterThan(0);
  });
});

describe("the estimate the caller holds still", () => {
  it("uses the passed estimate rather than recomputing the mean", () => {
    const heights = new Map([[0, 400]]);
    const drifting = catalogWindow(input({ total: 30, columns: 1, heights, viewportHeight: 100 }));
    const held = catalogWindow(input({ total: 30, columns: 1, heights, estimate: 200, viewportHeight: 100 }));
    expect(drifting.estimate).toBe(400);
    expect(held.estimate).toBe(200);
    expect(held.height).toBeLessThan(drifting.height);
  });

  it("falls back to the mean when the caller has none yet", () => {
    expect(catalogWindow(input({ estimate: 0 })).estimate).toBe(CATALOG_ESTIMATE);
    expect(catalogWindow(input({ estimate: undefined })).estimate).toBe(CATALOG_ESTIMATE);
  });
});

/**
 * What the component must keep doing, read off its own source.
 *
 * The arithmetic above is testable; the wiring is not, because the suite
 * has no DOM. These are the four promises a window makes that a later
 * edit could quietly take back, and each of them is the difference
 * between a windowed list and a broken one:
 *
 *   - it windows the FILTERED array, so the scrollbar and the drawn rows
 *     are about the same list;
 *   - the count a reader and a screen reader are given is that filtered
 *     total, never the dozen cards that happen to be built;
 *   - every card says where it sits in that total, because `751 of 752`
 *     cannot be inferred from a DOM that holds twelve;
 *   - the focused row is pinned, because dropping the element that holds
 *     focus sends focus to `<body>`.
 *
 * Same shape as `overlayStacking.test.ts`: read the declaration rather
 * than restate it, so the test cannot drift from the component silently.
 */
describe("the catalogue component's side of the contract", () => {
  const source = readFileSync(join(import.meta.dirname, "components", "Catalog.svelte"), "utf8");

  it("windows the filtered list, not the whole index", () => {
    // `shown` is `filterCatalogEntries(all, filters)`; `all` is the index.
    expect(source).toMatch(/const shown = \$derived\(filterCatalogEntries\(all, filters\)\)/);
    expect(source).toMatch(/catalogWindow\(\{[\s\S]*?total: shown\.length/);
    expect(source).not.toMatch(/catalogWindow\(\{[\s\S]*?total: all\.length/);
  });

  it("labels the list with the filtered total and not the drawn count", () => {
    expect(source).toMatch(/role="list"\s*\n\s*aria-label=\{t\("\{count\} shown", \{ count: shown\.length \}\)\}/);
    expect(source).not.toMatch(/aria-label=\{t\("\{count\} shown", \{ count: (drawnRows|windowed)/);
  });

  it("tells a screen reader where each card sits in that total", () => {
    expect(source).toMatch(/role="listitem"/);
    expect(source).toMatch(/aria-setsize=\{shown\.length\}/);
    expect(source).toMatch(/aria-posinset=\{row\.start \+ place \+ 1\}/);
  });

  it("pins the focused row so focus is never dropped out of the DOM", () => {
    expect(source).toMatch(/onfocusin=\{onCardsFocusIn\}/);
    expect(source).toMatch(/onfocusout=\{onCardsFocusOut\}/);
    expect(source).toMatch(/pinned: focusedRow/);
  });

  /**
   * The column count and the gap are read back off the resolved computed
   * style. A second copy of the breakpoint in the script would be a
   * silent way for the drawn rows to disagree with the painted ones.
   */
  it("reads the grid back from the CSS rather than recomputing it", () => {
    expect(source).toMatch(/rowStyle\.gridTemplateColumns/);
    expect(source).toMatch(/rowStyle\.columnGap/);
    expect(source).toMatch(/\.cards-row \{[\s\S]*?grid-template-columns: repeat\(auto-fill, minmax\(18rem, 1fr\)\)/);
  });
});
