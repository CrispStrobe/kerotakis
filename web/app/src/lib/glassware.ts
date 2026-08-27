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
  /** Real capacity of this kind, in millilitres. */
  capacity_ml: number;
  /**
   * Maps a volume fraction (0–1 of capacity) to a fill-height fraction (0–1
   * of fh). Linear for prismatic vessels; cubic-root for cones. The inverse
   * is `heightToVolume`.
   */
  volumeToHeight: (vFrac: number) => number;
  /** Inverse of volumeToHeight: height fraction → volume fraction. */
  heightToVolume: (hFrac: number) => number;
  glass: string;
  inner: string;
  svgW: number;
};

const linear = (f: number) => f;

/**
 * Conical profile: V ∝ h³ in a cone, so h = V^(1/3). The flask's body is
 * conical from the base up to ~78% of its fill height, then transitions to
 * the narrow cylindrical neck. A smooth two-segment approximation:
 *   - 0–85% of volume fills the conical body (maps to 0–78% of height)
 *   - 85–100% fills the narrow neck linearly (maps to 78–100% of height)
 */
function flaskVolumeToHeight(vFrac: number): number {
  if (vFrac <= 0) return 0;
  if (vFrac >= 1) return 1;
  const vBody = 0.85;
  const hBody = 0.78;
  if (vFrac <= vBody) {
    return Math.cbrt(vFrac / vBody) * hBody;
  }
  return hBody + ((vFrac - vBody) / (1 - vBody)) * (1 - hBody);
}

function flaskHeightToVolume(hFrac: number): number {
  if (hFrac <= 0) return 0;
  if (hFrac >= 1) return 1;
  const vBody = 0.85;
  const hBody = 0.78;
  if (hFrac <= hBody) {
    const norm = hFrac / hBody;
    return norm * norm * norm * vBody;
  }
  return vBody + ((hFrac - hBody) / (1 - hBody)) * (1 - vBody);
}

/**
 * Crucible profile: a truncated cone (wider at top than bottom). The width
 * grows linearly with height, so V ∝ h(w_bottom² + w_top² + w_bottom·w_top).
 * Approximated as quadratic: h ≈ sqrt(V).
 */
function crucibleVolumeToHeight(vFrac: number): number {
  if (vFrac <= 0) return 0;
  if (vFrac >= 1) return 1;
  return Math.sqrt(vFrac);
}

function crucibleHeightToVolume(hFrac: number): number {
  if (hFrac <= 0) return 0;
  if (hFrac >= 1) return 1;
  return hFrac * hFrac;
}

export const KINDS: Record<string, Glassware> = {
  beaker: {
    ix: 14, iw: 72, by: 127, fh: 89, fullAtL: 0.4, capacity_ml: 400, svgW: 150,
    volumeToHeight: linear, heightToVolume: linear,
    glass: "M 12 14 L 12 122 Q 12 128 20 128 L 80 128 Q 88 128 88 122 L 88 14",
    inner: "M 13 14 L 13 127 L 87 127 L 87 14 Z",
  },
  flask: {
    ix: 14, iw: 72, by: 127, fh: 97, fullAtL: 0.3, capacity_ml: 300, svgW: 150,
    volumeToHeight: flaskVolumeToHeight, heightToVolume: flaskHeightToVolume,
    glass: "M 42 8 L 42 44 L 14 118 Q 12 128 22 128 L 78 128 Q 88 128 86 118 L 58 44 L 58 8",
    inner: "M 43 8 L 43 45 L 15 120 Q 14 127 22 127 L 78 127 Q 86 127 85 120 L 57 45 L 57 8 Z",
  },
  tube: {
    ix: 38, iw: 24, by: 127, fh: 109, fullAtL: 0.05, capacity_ml: 50, svgW: 90,
    volumeToHeight: linear, heightToVolume: linear,
    glass: "M 38 10 L 38 114 Q 38 128 50 128 Q 62 128 62 114 L 62 10",
    inner: "M 39 10 L 39 114 Q 39 127 50 127 Q 61 127 61 114 L 61 10 Z",
  },
  cylinder: {
    ix: 38, iw: 24, by: 123, fh: 107, fullAtL: 0.1, capacity_ml: 100, svgW: 90,
    volumeToHeight: linear, heightToVolume: linear,
    glass: "M 38 8 L 38 124 L 62 124 L 62 8 M 30 130 L 70 130",
    inner: "M 39 8 L 39 123 L 61 123 L 61 8 Z",
  },
  crucible: {
    ix: 24, iw: 52, by: 127, fh: 39, fullAtL: 0.08, capacity_ml: 80, svgW: 150,
    volumeToHeight: crucibleVolumeToHeight, heightToVolume: crucibleHeightToVolume,
    glass: "M 18 92 L 26 126 Q 27 128 30 128 L 70 128 Q 73 128 74 126 L 82 92",
    inner: "M 20 93 L 27 127 L 73 127 L 80 93 Z",
  },
};

/**
 * Pixel fill height for a given volume in litres, respecting the kind's
 * volume→height profile. Clamps to [minPx, fh].
 */
export function fillHeight(g: Glassware, volume_l: number, minPx = 6): number {
  if (volume_l <= 0) return 0;
  const vFrac = Math.min(1, volume_l / g.fullAtL);
  const hFrac = g.volumeToHeight(vFrac);
  return Math.max(minPx, hFrac * g.fh);
}

/** Display height for a settled solid volume. Trace layers would be sub-pixel
 * at bench scale, so sqrt scaling supplies a perceptual magnifier while
 * remaining zero-safe, monotone, capacity-aware, and capped below the liquid
 * body. The exact engine volume stays attached to each rendered layer. */
export function depositDisplayHeight(g: Glassware, volume_l: number): number {
  if (volume_l <= 0) return 0;
  const fraction = Math.min(1, volume_l / g.fullAtL);
  return Math.min(g.fh * 0.28, Math.max(1.25, Math.sqrt(fraction) * g.fh * 0.42));
}

/**
 * Cylinder graduation ticks: returns {y, label} for each graduation line.
 * Ticks are spaced at equal volume intervals and placed using the kind's
 * volume→height profile (linear for a cylinder, but correct for any shape).
 */
export function graduationTicks(
  g: Glassware,
  count = 5,
): { y: number; ml: number }[] {
  const ticks: { y: number; ml: number }[] = [];
  for (let i = 1; i <= count; i++) {
    const vFrac = i / count;
    const hFrac = g.volumeToHeight(vFrac);
    const y = g.by - hFrac * g.fh;
    ticks.push({ y: Math.round(y * 10) / 10, ml: Math.round(vFrac * g.capacity_ml) });
  }
  return ticks;
}

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

/** Proportional settled layers, top-to-bottom. The supplied engine volumes
 * determine each share while the returned rectangles tile [by-solidH, by]
 * exactly, including the last floating-point remainder. */
export function solidLayers(
  volumes: readonly number[],
  solidH: number,
  by: number,
): { y: number; h: number }[] {
  const weights = volumes.map((volume) => Math.max(0, volume));
  const total = weights.reduce((sum, volume) => sum + volume, 0);
  if (weights.length === 0 || total <= 0 || solidH <= 0) return [];
  let y = by - solidH;
  return weights.map((volume, index) => {
    const h = index === weights.length - 1 ? by - y : solidH * volume / total;
    const layer = { y, h };
    y += h;
    return layer;
  });
}
