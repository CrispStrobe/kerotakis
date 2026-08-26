export const BENCH_ZONES = ["prepare", "react", "analyse"] as const;
export type BenchZone = (typeof BENCH_ZONES)[number];

export interface BenchLayout {
  version: 1;
  placements: Record<number, BenchZone>;
}

export const EMPTY_BENCH_LAYOUT: BenchLayout = { version: 1, placements: {} };
export const BENCH_LAYOUT_KEY = "kerotakis.bench.layout.v1";

export function isBenchZone(value: unknown): value is BenchZone {
  return typeof value === "string" && BENCH_ZONES.includes(value as BenchZone);
}

/** New vessels begin where reactions happen; users can organise from there. */
export function zoneFor(layout: BenchLayout, vessel: number): BenchZone {
  return layout.placements[vessel] ?? "react";
}

export function placeVessel(layout: BenchLayout, vessel: number, zone: BenchZone): BenchLayout {
  if (zoneFor(layout, vessel) === zone) return layout;
  return {
    version: 1,
    placements: { ...layout.placements, [vessel]: zone },
  };
}

export function parseBenchLayout(raw: string | null): BenchLayout {
  if (!raw) return EMPTY_BENCH_LAYOUT;
  try {
    const value = JSON.parse(raw) as { version?: unknown; placements?: unknown };
    if (value.version !== 1 || !value.placements || typeof value.placements !== "object") {
      return EMPTY_BENCH_LAYOUT;
    }
    const placements: Record<number, BenchZone> = {};
    for (const [key, zone] of Object.entries(value.placements)) {
      const vessel = Number(key);
      if (Number.isInteger(vessel) && vessel >= 0 && isBenchZone(zone)) placements[vessel] = zone;
    }
    return { version: 1, placements };
  } catch {
    return EMPTY_BENCH_LAYOUT;
  }
}

export function adjacentZone(zone: BenchZone, direction: -1 | 1): BenchZone {
  const index = BENCH_ZONES.indexOf(zone);
  return BENCH_ZONES[Math.max(0, Math.min(BENCH_ZONES.length - 1, index + direction))];
}
