export const BENCH_ZONES = ["prepare", "react", "analyse"] as const;
export type BenchZone = (typeof BENCH_ZONES)[number];

export interface BenchPlacement {
  /** Optional workflow hint inferred from x; chemistry never depends on it. */
  zone: BenchZone;
  /** Normalised centre coordinates on the usable bench surface. */
  x: number;
  y: number;
}

export interface BenchLayout {
  version: 2;
  placements: Record<number, BenchPlacement>;
  /** Explicit positions only; an undeployed/unmoved tool follows its target. */
  apparatus: Record<string, BenchPlacement>;
}

export interface ApparatusRoute {
  from: Pick<BenchPlacement, "x" | "y">;
  to: Pick<BenchPlacement, "x" | "y">;
  control1: Pick<BenchPlacement, "x" | "y">;
  control2: Pick<BenchPlacement, "x" | "y">;
  midpoint: Pick<BenchPlacement, "x" | "y">;
}

export const EMPTY_BENCH_LAYOUT: BenchLayout = { version: 2, placements: {}, apparatus: {} };
// Keep the storage key: parseBenchLayout migrates the version-1 value in place.
export const BENCH_LAYOUT_KEY = "kerotakis.bench.layout.v1";
export const LAB_LAYOUT_PREFIX = "# kerotakis-bench-layout-v2 ";

const X_MIN = 0.08;
const X_MAX = 0.92;
const Y_MIN = 0.28;
const Y_MAX = 0.84;
const APPARATUS_Y_MIN = 0.12;
const APPARATUS_Y_MAX = 0.88;

export function isBenchZone(value: unknown): value is BenchZone {
  return typeof value === "string" && BENCH_ZONES.includes(value as BenchZone);
}

const finite = (value: unknown): value is number => typeof value === "number" && Number.isFinite(value);
const clamp = (value: number, low: number, high: number) => Math.max(low, Math.min(high, value));

export function zoneAt(x: number): BenchZone {
  if (x < 1 / 3) return "prepare";
  if (x > 2 / 3) return "analyse";
  return "react";
}

function zoneCentre(zone: BenchZone): number {
  return zone === "prepare" ? 1 / 6 : zone === "analyse" ? 5 / 6 : 1 / 2;
}

/** Stable defaults spread new vessels across the surface instead of stacking them. */
export function defaultPosition(vessel: number): BenchPlacement {
  const column = vessel % 4;
  const row = Math.floor(vessel / 4) % 2;
  const x = 0.2 + column * 0.2;
  const y = 0.58 + row * 0.22;
  return { zone: zoneAt(x), x, y };
}

export function positionFor(layout: BenchLayout, vessel: number): BenchPlacement {
  return layout.placements[vessel] ?? defaultPosition(vessel);
}

export function zoneFor(layout: BenchLayout, vessel: number): BenchZone {
  return positionFor(layout, vessel).zone;
}

export function positionVessel(layout: BenchLayout, vessel: number, x: number, y: number): BenchLayout {
  const nextX = clamp(x, X_MIN, X_MAX);
  const nextY = clamp(y, Y_MIN, Y_MAX);
  const current = positionFor(layout, vessel);
  const next = { zone: zoneAt(nextX), x: nextX, y: nextY };
  if (current.zone === next.zone && current.x === next.x && current.y === next.y) return layout;
  return { ...layout, placements: { ...layout.placements, [vessel]: next } };
}

export function apparatusPositionFor(
  layout: BenchLayout,
  tool: string,
  fallback: BenchPlacement,
): BenchPlacement {
  return layout.apparatus[tool] ?? fallback;
}

export function positionApparatus(layout: BenchLayout, tool: string, x: number, y: number): BenchLayout {
  const nextX = clamp(x, X_MIN, X_MAX);
  const nextY = clamp(y, APPARATUS_Y_MIN, APPARATUS_Y_MAX);
  const next = { zone: zoneAt(nextX), x: nextX, y: nextY };
  const current = layout.apparatus[tool];
  if (current?.zone === next.zone && current.x === next.x && current.y === next.y) return layout;
  return { ...layout, apparatus: { ...layout.apparatus, [tool]: next } };
}

/** Screen-space footprint check for draggable bench objects. The caller owns
 * each object's footprint; chemistry and connectivity never depend on this
 * presentation constraint. Touching edges are valid, overlapping interiors
 * are not. */
export function placementsOverlap(
  a: Pick<BenchPlacement, "x" | "y">,
  b: Pick<BenchPlacement, "x" | "y">,
  separationX = 0.14,
  separationY = 0.2,
): boolean {
  const epsilon = 1e-9;
  return Math.abs(a.x - b.x) < separationX - epsilon
    && Math.abs(a.y - b.y) < separationY - epsilon;
}

/** Route a visible workstation/sample relationship from object edges rather
 * than drawing through both objects' centres. The lifted cubic keeps the
 * association readable around glassware and gives the UI a stable badge
 * position; it remains presentation-only and never implies chemistry. */
export function apparatusRoute(
  machine: Pick<BenchPlacement, "x" | "y">,
  vessel: Pick<BenchPlacement, "x" | "y">,
): ApparatusRoute {
  const horizontal = vessel.x >= machine.x ? 1 : -1;
  const from = { x: machine.x + horizontal * 0.055, y: machine.y };
  const to = { x: vessel.x - horizontal * 0.045, y: vessel.y };
  const lift = clamp(Math.min(from.y, to.y) - 0.09, 0.08, 0.82);
  const control1 = { x: from.x + horizontal * 0.055, y: lift };
  const control2 = { x: to.x - horizontal * 0.055, y: lift };
  const midpoint = {
    x: (from.x + 3 * control1.x + 3 * control2.x + to.x) / 8,
    y: (from.y + 3 * control1.y + 3 * control2.y + to.y) / 8,
  };
  return { from, to, control1, control2, midpoint };
}

/** Place newly created glassware in the first stable free slot. This runs only
 * after the engine confirms creation; it arranges the view without inventing
 * or mutating chemistry. */
export function placeNewVessel(
  layout: BenchLayout,
  vessel: number,
  occupiedVessels: readonly number[],
  obstacles: ReadonlyArray<Pick<BenchPlacement, "x" | "y">> = [],
): BenchLayout {
  const occupied = [
    ...occupiedVessels.filter((id) => id !== vessel).map((id) => positionFor(layout, id)),
    ...obstacles,
  ];
  const preferred = defaultPosition(vessel);
  const slots = [
    preferred,
    ...[0.31, 0.53, 0.75].flatMap((y) =>
      [0.12, 0.31, 0.5, 0.69, 0.88].map((x) => ({ zone: zoneAt(x), x, y })),
    ),
  ];
  const open = slots.find((candidate) =>
    occupied.every((position) => !placementsOverlap(candidate, position)),
  );
  return open ? positionVessel(layout, vessel, open.x, open.y) : layout;
}

/** Zone moves remain useful for keyboard users, but preserve vertical placement. */
export function placeVessel(layout: BenchLayout, vessel: number, zone: BenchZone): BenchLayout {
  const current = positionFor(layout, vessel);
  return positionVessel(layout, vessel, zoneCentre(zone), current.y);
}

export function parseBenchLayout(raw: string | null): BenchLayout {
  if (!raw) return EMPTY_BENCH_LAYOUT;
  try {
    const value = JSON.parse(raw) as { version?: unknown; placements?: unknown; apparatus?: unknown };
    if (!value.placements || typeof value.placements !== "object") return EMPTY_BENCH_LAYOUT;
    const placements: Record<number, BenchPlacement> = {};
    for (const [key, placement] of Object.entries(value.placements)) {
      const vessel = Number(key);
      if (!Number.isInteger(vessel) || vessel < 0) continue;
      if (value.version === 1 && isBenchZone(placement)) {
        const fallback = defaultPosition(vessel);
        placements[vessel] = { zone: placement, x: zoneCentre(placement), y: fallback.y };
      } else if (
        value.version === 2 && placement && typeof placement === "object"
        && isBenchZone((placement as BenchPlacement).zone)
        && finite((placement as BenchPlacement).x)
        && finite((placement as BenchPlacement).y)
      ) {
        const p = placement as BenchPlacement;
        const x = clamp(p.x, X_MIN, X_MAX);
        placements[vessel] = { zone: zoneAt(x), x, y: clamp(p.y, Y_MIN, Y_MAX) };
      }
    }
    const apparatus: Record<string, BenchPlacement> = {};
    if (value.version === 2 && value.apparatus && typeof value.apparatus === "object") {
      for (const [tool, placement] of Object.entries(value.apparatus)) {
        if (!tool || !placement || typeof placement !== "object") continue;
        const p = placement as BenchPlacement;
        if (!finite(p.x) || !finite(p.y)) continue;
        const x = clamp(p.x, X_MIN, X_MAX);
        apparatus[tool] = {
          zone: zoneAt(x),
          x,
          y: clamp(p.y, APPARATUS_Y_MIN, APPARATUS_Y_MAX),
        };
      }
    }
    return { version: 2, placements, apparatus };
  } catch {
    return EMPTY_BENCH_LAYOUT;
  }
}

/** Embed presentation-only placement in a comment old .lab readers ignore. */
export function labWithBenchLayout(script: string, layout: BenchLayout): string {
  const body = script.endsWith("\n") ? script : `${script}\n`;
  return `${LAB_LAYOUT_PREFIX}${JSON.stringify(layout)}\n${body}`;
}

/** Recover an optional shared arrangement without changing chemistry commands. */
export function benchLayoutFromLab(text: string): BenchLayout | null {
  const line = text.split(/\r?\n/).find((candidate) => candidate.startsWith(LAB_LAYOUT_PREFIX));
  if (!line) return null;
  const raw = line.slice(LAB_LAYOUT_PREFIX.length).trim();
  if (!raw) return null;
  try {
    const decoded = JSON.parse(raw) as { version?: unknown; placements?: unknown };
    if (decoded.version !== 2 || !decoded.placements || typeof decoded.placements !== "object") return null;
  } catch {
    return null;
  }
  return parseBenchLayout(raw);
}

export function adjacentZone(zone: BenchZone, direction: -1 | 1): BenchZone {
  const index = BENCH_ZONES.indexOf(zone);
  return BENCH_ZONES[Math.max(0, Math.min(BENCH_ZONES.length - 1, index + direction))];
}
