/**
 * Chart JSON v1 (PROTOCOL.md, the CAP-3 contract) — consumer types and the
 * small numeric core of the renderer. Hand-rolled linear scales and nice
 * ticks: a titration curve needs no charting library, and the licence
 * surface stays empty.
 */

export interface ChartSpec {
  chart: number;
  title: string;
  x: ChartAxis;
  y: ChartAxis;
  series: ChartSeries[];
  markers?: { x: number; label: string }[];
  provenance?: string;
}

export interface ChartAxis {
  label: string;
  unit?: string;
  scale?: "linear";
}

export interface ChartSeries {
  label: string;
  /** The confidence vocabulary; the stroke follows GUI-023's encoding. */
  confidence?: string;
  /** [x, y] pairs, in data units, assumed x-sorted. */
  points: [number, number][];
  /** Optional uncertainty band: [x, low, high] (CAP-8). */
  band?: [number, number, number][];
}

/** Data extent across all series (and bands), padded when degenerate. */
export function extent(spec: ChartSpec): { x: [number, number]; y: [number, number] } {
  let xs: number[] = [];
  let ys: number[] = [];
  for (const s of spec.series) {
    for (const [x, y] of s.points) {
      xs.push(x);
      ys.push(y);
    }
    for (const [x, lo, hi] of s.band ?? []) {
      xs.push(x);
      ys.push(lo, hi);
    }
  }
  for (const m of spec.markers ?? []) xs.push(m.x);
  if (xs.length === 0) {
    xs = [0, 1];
    ys = [0, 1];
  }
  const pad = ([lo, hi]: [number, number]): [number, number] =>
    lo === hi ? [lo - 0.5, hi + 0.5] : [lo, hi];
  return {
    x: pad([Math.min(...xs), Math.max(...xs)]),
    y: pad([Math.min(...ys), Math.max(...ys)]),
  };
}

/** A linear scale mapping [d0,d1] onto [r0,r1]. */
export function scale(
  [d0, d1]: [number, number],
  [r0, r1]: [number, number],
): (v: number) => number {
  const k = (r1 - r0) / (d1 - d0);
  return (v) => r0 + (v - d0) * k;
}

/**
 * Nice tick positions covering [lo, hi]: steps of 1/2/5 × 10^n, at most
 * `want` + 1 ticks, endpoints included in coverage.
 */
export function niceTicks(lo: number, hi: number, want = 5): number[] {
  if (!(hi > lo) || !Number.isFinite(lo) || !Number.isFinite(hi)) return [lo];
  const span = hi - lo;
  const rawStep = span / Math.max(1, want);
  const mag = 10 ** Math.floor(Math.log10(rawStep));
  const norm = rawStep / mag;
  const step = (norm <= 1 ? 1 : norm <= 2 ? 2 : norm <= 5 ? 5 : 10) * mag;
  const start = Math.ceil(lo / step) * step;
  const ticks: number[] = [];
  // Guard float drift at the top end.
  for (let v = start; v <= hi + step * 1e-9; v += step) {
    ticks.push(Math.abs(v) < step * 1e-9 ? 0 : Number(v.toPrecision(12)));
  }
  return ticks;
}

/** SVG path for a series' points through the given scales. */
export function linePath(
  points: [number, number][],
  sx: (v: number) => number,
  sy: (v: number) => number,
): string {
  return points
    .map(([x, y], i) => `${i === 0 ? "M" : "L"}${sx(x).toFixed(2)},${sy(y).toFixed(2)}`)
    .join(" ");
}

/** Closed SVG path for an uncertainty band: along the highs, back along the lows. */
export function bandPath(
  band: [number, number, number][],
  sx: (v: number) => number,
  sy: (v: number) => number,
): string {
  if (band.length === 0) return "";
  const upper = band.map(
    ([x, , hi], i) => `${i === 0 ? "M" : "L"}${sx(x).toFixed(2)},${sy(hi).toFixed(2)}`,
  );
  const lower = [...band]
    .reverse()
    .map(([x, lo]) => `L${sx(x).toFixed(2)},${sy(lo).toFixed(2)}`);
  return `${upper.join(" ")} ${lower.join(" ")} Z`;
}

/** GUI-023's encoding, applied to strokes. */
export function dashFor(confidence?: string): string | undefined {
  switch (confidence) {
    case "modeled":
      return "6 3";
    case "curated":
    case "template_match":
      return "2 3";
    default:
      return undefined;
  }
}
