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
}

export const EMPTY_BENCH_LAYOUT: BenchLayout = { version: 2, placements: {} };
// Keep the storage key: parseBenchLayout migrates the version-1 value in place.
export const BENCH_LAYOUT_KEY = "kerotakis.bench.layout.v1";

const X_MIN = 0.08;
const X_MAX = 0.92;
const Y_MIN = 0.28;
const Y_MAX = 0.84;

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
  return { version: 2, placements: { ...layout.placements, [vessel]: next } };
}

/** Zone moves remain useful for keyboard users, but preserve vertical placement. */
export function placeVessel(layout: BenchLayout, vessel: number, zone: BenchZone): BenchLayout {
  const current = positionFor(layout, vessel);
  return positionVessel(layout, vessel, zoneCentre(zone), current.y);
}

export function parseBenchLayout(raw: string | null): BenchLayout {
  if (!raw) return EMPTY_BENCH_LAYOUT;
  try {
    const value = JSON.parse(raw) as { version?: unknown; placements?: unknown };
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
    return { version: 2, placements };
  } catch {
    return EMPTY_BENCH_LAYOUT;
  }
}

export function adjacentZone(zone: BenchZone, direction: -1 | 1): BenchZone {
  const index = BENCH_ZONES.indexOf(zone);
  return BENCH_ZONES[Math.max(0, Math.min(BENCH_ZONES.length - 1, index + direction))];
}
