import type { SceneCorrosion } from "./host/EngineHost";

export interface CorrosionReadout extends SceneCorrosion {
  fraction: number;
  percent: number;
  /** Restrained schematic strength, never a claim about surface coverage. */
  visualStrength: number;
}

export function corrosionReadouts(rows?: SceneCorrosion[]): CorrosionReadout[] {
  return (rows ?? []).map((row) => {
    const raw = Number(row.metal_in_oxide_fraction);
    const fraction = Number.isFinite(raw) ? Math.min(1, Math.max(0, raw)) : 0;
    return {
      ...row,
      fraction,
      percent: Math.round(fraction * 100),
      visualStrength: 0.12 + fraction * 0.68,
    };
  });
}
