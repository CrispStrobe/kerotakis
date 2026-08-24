/**
 * Consumer types + numeric core for the CAP-3 chart contract
 * (`kerotakis-core/src/chart.rs` is authoritative; PROTOCOL.md documents
 * it). Hand-rolled linear scales and nice ticks: a titration curve needs
 * no charting library, and the licence surface stays empty.
 */

export interface ChartSpec {
  title: string;
  x: ChartAxis;
  y: ChartAxis;
  series: ChartSeries[];
  /** A chart without provenance is a picture, not a result. */
  provenance: string;
}

export interface ChartAxis {
  label: string;
  unit?: string;
}

/** Tagged as the Rust enum serializes: kind = line | scatter | band. */
export type ChartSeries =
  | { kind: "line"; name: string; points: [number, number][] }
  | { kind: "scatter"; name: string; points: [number, number][] }
  | {
      kind: "band";
      name: string;
      lower: [number, number][];
      upper: [number, number][];
    };

/** Whether an unknown value looks like the chart contract. */
export function isChartSpec(v: unknown): v is ChartSpec {
  const c = v as ChartSpec;
  return (
    typeof c === "object" &&
    c !== null &&
    typeof c.title === "string" &&
    Array.isArray(c.series) &&
    typeof c.x?.label === "string" &&
    typeof c.y?.label === "string"
  );
}

/** Every point that bounds a series — both envelopes for a band. */
export function seriesPoints(s: ChartSeries): [number, number][] {
  return s.kind === "band" ? [...s.lower, ...s.upper] : s.points;
}

/** Data extent across all series, padded when degenerate. */
export function extent(spec: ChartSpec): { x: [number, number]; y: [number, number] } {
  let xs: number[] = [];
  let ys: number[] = [];
  for (const s of spec.series) {
    for (const [x, y] of seriesPoints(s)) {
      xs.push(x);
      ys.push(y);
    }
  }
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

/** SVG path for a polyline through the given scales. */
export function linePath(
  points: [number, number][],
  sx: (v: number) => number,
  sy: (v: number) => number,
): string {
  return points
    .map(([x, y], i) => `${i === 0 ? "M" : "L"}${sx(x).toFixed(2)},${sy(y).toFixed(2)}`)
    .join(" ");
}

/** Closed SVG path for a band: along the upper envelope, back along the lower. */
export function bandPath(
  lower: [number, number][],
  upper: [number, number][],
  sx: (v: number) => number,
  sy: (v: number) => number,
): string {
  if (lower.length === 0 && upper.length === 0) return "";
  const up = upper.map(
    ([x, y], i) => `${i === 0 ? "M" : "L"}${sx(x).toFixed(2)},${sy(y).toFixed(2)}`,
  );
  const down = [...lower]
    .reverse()
    .map(([x, y]) => `L${sx(x).toFixed(2)},${sy(y).toFixed(2)}`);
  return `${up.join(" ")} ${down.join(" ")} Z`;
}
