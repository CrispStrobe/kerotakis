/**
 * Per-kind glassware geometry inside the 100×140 viewBox Vessel.svelte draws.
 *
 * `inner` is the fill silhouette — liquids and deposits are clipped to it, so
 * a conical flask holds its liquid in a cone. `glass` is the stroked outline.
 * `by` is the floor contents rest on: it MUST be the lowest point of `inner`,
 * or a correctly sized layer leaves an unpainted strip between the deposit and
 * the drawn bottom of the vessel. `glasswareFloorsMatchTheirSilhouettes` in
 * glassware.test.ts holds that invariant.
 *
 * `fh` is the height a `fullAtL`-litre fill draws at, measured up from `by`.
 */
export type Glassware = {
  /** Left edge of the fill rect. */
  ix: number;
  /** Width of the fill rect. */
  iw: number;
  /** Floor: the y contents sit on. */
  by: number;
  /** Height of a full fill. */
  fh: number;
  /** Volume, in litres, that draws as a full fill. */
  fullAtL: number;
  glass: string;
  inner: string;
  svgW: number;
};

export const KINDS: Record<string, Glassware> = {
  beaker: {
    ix: 14, iw: 72, by: 127, fh: 89, fullAtL: 0.4, svgW: 150,
    glass: "M 12 14 L 12 122 Q 12 128 20 128 L 80 128 Q 88 128 88 122 L 88 14",
    inner: "M 13 14 L 13 127 L 87 127 L 87 14 Z",
  },
  flask: {
    ix: 14, iw: 72, by: 127, fh: 97, fullAtL: 0.3, svgW: 150,
    glass: "M 42 8 L 42 44 L 14 118 Q 12 128 22 128 L 78 128 Q 88 128 86 118 L 58 44 L 58 8",
    inner: "M 43 8 L 43 45 L 15 120 Q 14 127 22 127 L 78 127 Q 86 127 85 120 L 57 45 L 57 8 Z",
  },
  tube: {
    ix: 38, iw: 24, by: 127, fh: 109, fullAtL: 0.05, svgW: 90,
    glass: "M 38 10 L 38 114 Q 38 128 50 128 Q 62 128 62 114 L 62 10",
    inner: "M 39 10 L 39 114 Q 39 127 50 127 Q 61 127 61 114 L 61 10 Z",
  },
  cylinder: {
    ix: 38, iw: 24, by: 123, fh: 107, fullAtL: 0.1, svgW: 90,
    glass: "M 38 8 L 38 124 L 62 124 L 62 8 M 30 130 L 70 130",
    inner: "M 39 8 L 39 123 L 61 123 L 61 8 Z",
  },
  crucible: {
    ix: 24, iw: 52, by: 127, fh: 39, fullAtL: 0.08, svgW: 150,
    glass: "M 18 92 L 26 126 Q 27 128 30 128 L 70 128 Q 73 128 74 126 L 82 92",
    inner: "M 20 93 L 27 127 L 73 127 L 80 93 Z",
  },
};

/**
 * The lowest y an absolute-coordinate silhouette path reaches. Every `inner`
 * path is M/L/Q/Z over absolute coordinate pairs, so the y values are simply
 * every second number.
 */
export function innerFloor(inner: string): number {
  const nums = (inner.match(/-?\d+(\.\d+)?/g) ?? []).map(Number);
  let floor = -Infinity;
  for (let i = 1; i < nums.length; i += 2) floor = Math.max(floor, nums[i]!);
  return floor;
}

/**
 * One settled deposit layer of a stack of `n`, counting from the top (i = 0).
 * The layers tile the band [by − solidH, by] exactly: equal thicknesses, no
 * gap at the floor and nothing hanging below it.
 */
export function solidLayer(
  i: number,
  n: number,
  solidH: number,
  by: number,
): { y: number; h: number } {
  const h = solidH / n;
  return { y: by - h * (n - i), h };
}
