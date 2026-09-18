/**
 * Which rows of the catalogue are actually in the DOM.
 *
 * GUI-105 folded three populations into one index, so the default view is
 * ~752 cards. Each one is an `<article>` with a head, a hook, two
 * definition lists, a readiness line and a footer of buttons — the browser
 * was building somewhere north of twenty thousand elements to show the
 * dozen a reader can see. On the low-end target the roadmap names, that is
 * the difference between a list that opens and a list that hangs.
 *
 * The usual objection to windowing is that it breaks find-in-page: Ctrl-F
 * can only see what is painted. The answer here is that the in-app search
 * greps the DESCRIPTIONS — equation, summary, procedure, observations,
 * boundary, the reason a question is refused — in the reader's language
 * AND the canonical English underneath, so it reaches strictly more than
 * find-in-page ever did. `catalogEntry.test.ts` pins that, and this module
 * is only safe while it passes.
 *
 * The shape is deliberately plain: no dependency, no virtual scroller
 * framework, no fixed row height.
 *
 *   - Cards are NOT uniform. A refused question is three lines; a guided
 *     experiment with a kit, a recipe preview and four connection buttons
 *     is twenty. Any fixed height would put the scrollbar somewhere the
 *     content is not.
 *   - So heights are MEASURED, per grid row, as rows are drawn, and cached
 *     by row index. A row that has never been on screen uses an estimate,
 *     which is the running mean of the rows that have. The cost is stated
 *     plainly: the scrollbar is approximate in territory the reader has
 *     not visited yet, and exact everywhere they have. The caller anchors
 *     scroll position across a measurement so the content does not jump.
 *   - A grid ROW is the unit, not a card, because the cards sit in a
 *     `repeat(auto-fill, minmax(18rem, 1fr))` grid where every card in a
 *     row stretches to the tallest one. The column count is read back from
 *     the resolved computed style rather than recomputed from the
 *     breakpoints, so the CSS stays the single source of truth.
 *
 * Everything here is pure: the caller passes measurements in and gets a
 * range out. That is what lets the behaviour be tested at all — the suite
 * has no DOM, and a windowing bug that only shows up in a browser is a
 * windowing bug nobody catches.
 */

/** Grid rows kept above and below the viewport, so a flick never shows a gap. */
export const CATALOG_OVERSCAN = 3;

/**
 * How many cards to draw before anything has been measured.
 *
 * The first frame has no layout to read, so the column count is unknown
 * and the list is drawn as an ordinary grid of this many cards. Small
 * enough to be cheap, large enough to fill any viewport that will then
 * report its real height on the next frame.
 */
export const CATALOG_FIRST_DRAW = 24;

/** A last-resort row height, used only until one real row has been measured. */
export const CATALOG_ESTIMATE = 260;

export interface CatalogWindowInput {
  /** Rows in the FILTERED list — never the unfiltered index, never the drawn subset. */
  total: number;
  /** Resolved grid columns, as the browser laid them out. */
  columns: number;
  /** The scroller's `scrollTop`. */
  scrollTop: number;
  /** Where the grid starts inside the scroller's content. */
  viewportTop: number;
  /** The scroller's `clientHeight`. Zero means "not measurable yet". */
  viewportHeight: number;
  /** Measured heights by grid-row index. */
  heights: ReadonlyMap<number, number>;
  /** Vertical gap between grid rows, in px. */
  gap: number;
  /** Rows above and below the viewport. */
  overscan?: number;
  /**
   * The height to assume for a row nobody has measured.
   *
   * The caller passes this rather than letting it be recomputed, because
   * the mean moves a little on EVERY measurement, and a moving estimate
   * moves every unmeasured row with it — including rows above the reader,
   * whose offsets are what the scroll anchor is defined against. Holding
   * it still between material changes is the difference between a list
   * that settles and one that re-anchors on every frame. Omitted, it
   * falls back to the mean of what has been measured.
   */
  estimate?: number;
  /**
   * A row that must stay in the DOM wherever it is.
   *
   * This is the focused card's row. Dropping the element that holds focus
   * sends focus to `<body>`, which silently resets the tab order to the
   * top of the document — the reader presses Tab expecting the next card
   * and lands in the browser chrome. One extra row in the DOM is a cheap
   * price for that never happening.
   */
  pinned?: number | null;
}

export interface CatalogWindowRow {
  /** Grid-row index. */
  index: number;
  /** Offset from the top of the grid, in px. */
  top: number;
  /** Height used for this row: measured, or the current estimate. */
  height: number;
  /** Slice of the filtered array this row draws. */
  start: number;
  end: number;
}

export interface CatalogWindowShape {
  rows: CatalogWindowRow[];
  /** Total height of the virtual area, so the scrollbar has something to be. */
  height: number;
  rowCount: number;
  /** First and last row of the contiguous visible band (excludes any pin). */
  first: number;
  last: number;
  /** Cards actually in the DOM. */
  drawn: number;
  /** The row height used where nothing has been measured. */
  estimate: number;
}

/**
 * The height to assume for a row nobody has looked at.
 *
 * The mean of what HAS been measured, rather than a constant: a filter
 * showing only refused questions has rows a third the height of the mixed
 * list, and a constant tuned for the mixed list would make that scrollbar
 * three times too long.
 */
export function estimateRowHeight(heights: ReadonlyMap<number, number>): number {
  if (heights.size === 0) return CATALOG_ESTIMATE;
  let sum = 0;
  for (const height of heights.values()) sum += height;
  return sum / heights.size;
}

/** Top offset of every row, and the height of the whole virtual area. */
export function rowTops(
  rowCount: number,
  heights: ReadonlyMap<number, number>,
  estimate: number,
  gap: number,
): { tops: number[]; height: number } {
  const tops = new Array<number>(rowCount);
  let offset = 0;
  for (let row = 0; row < rowCount; row += 1) {
    tops[row] = offset;
    offset += (heights.get(row) ?? estimate) + gap;
  }
  return { tops, height: rowCount === 0 ? 0 : offset - gap };
}

/** Grid rows needed for `total` cards at `columns` across. */
export function rowCountFor(total: number, columns: number): number {
  const across = Math.max(1, Math.floor(columns) || 1);
  return Math.ceil(Math.max(0, total) / across);
}

export function catalogWindow(input: CatalogWindowInput): CatalogWindowShape {
  const columns = Math.max(1, Math.floor(input.columns) || 1);
  const total = Math.max(0, input.total);
  const rowCount = rowCountFor(total, columns);
  const estimate = input.estimate && input.estimate > 0
    ? input.estimate
    : estimateRowHeight(input.heights);
  const gap = Math.max(0, input.gap);
  const { tops, height } = rowTops(rowCount, input.heights, estimate, gap);

  const rowAt = (index: number): CatalogWindowRow => ({
    index,
    top: tops[index]!,
    height: input.heights.get(index) ?? estimate,
    start: index * columns,
    end: Math.min(total, (index + 1) * columns),
  });

  if (rowCount === 0) {
    return { rows: [], height: 0, rowCount: 0, first: 0, last: -1, drawn: 0, estimate };
  }

  /**
   * No measurable viewport — draw the lot.
   *
   * A zero height is not "nothing is visible", it is "nobody has told us
   * yet": a scroller before its first layout, a headless environment, a
   * print stylesheet. Windowing on that number would hide the whole
   * catalogue from anything that cannot run a layout pass, which is a far
   * worse failure than drawing too much.
   */
  if (!(input.viewportHeight > 0)) {
    const rows = Array.from({ length: rowCount }, (_, index) => rowAt(index));
    return { rows, height, rowCount, first: 0, last: rowCount - 1, drawn: total, estimate };
  }

  const overscan = Math.max(0, input.overscan ?? CATALOG_OVERSCAN);
  const seen = input.scrollTop - input.viewportTop;
  const bottom = seen + input.viewportHeight;

  let first = 0;
  while (first < rowCount - 1 && tops[first + 1]! <= seen) first += 1;
  let last = first;
  while (last < rowCount - 1 && tops[last + 1]! < bottom) last += 1;

  first = Math.max(0, first - overscan);
  last = Math.min(rowCount - 1, last + overscan);

  const indices: number[] = [];
  const pinned = input.pinned;
  if (pinned != null && pinned >= 0 && pinned < rowCount && pinned < first) indices.push(pinned);
  for (let row = first; row <= last; row += 1) indices.push(row);
  if (pinned != null && pinned > last && pinned < rowCount) indices.push(pinned);

  const rows = indices.map(rowAt);
  return {
    rows,
    height,
    rowCount,
    first,
    last,
    drawn: rows.reduce((count, row) => count + (row.end - row.start), 0),
    estimate,
  };
}
