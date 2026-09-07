import type { SceneAdsorption } from "./host/EngineHost";

export interface AdsorptionReadout extends SceneAdsorption {
  loading_mg_per_g: number | null;
  heldFraction: number;
  heldPercent: number;
  loadingFraction: number;
}

const finite = (value: unknown, fallback = 0) => {
  const number = Number(value);
  return Number.isFinite(number) ? number : fallback;
};

/** Defensive display normalization; chemistry stays wholly engine-owned. */
export function adsorptionReadouts(rows?: SceneAdsorption[]): AdsorptionReadout[] {
  return (rows ?? []).map((row) => {
    const heldFraction = Math.min(1, Math.max(0, finite(row.held_fraction)));
    const loadingFraction = Math.min(1, Math.max(0, finite(row.loading_fraction)));
    return {
      ...row,
      held_mg: Math.max(0, finite(row.held_mg)),
      still_dissolved_mg: Math.max(0, finite(row.still_dissolved_mg)),
      loading_mg_per_g: row.loading_mg_per_g == null
        ? null
        : Math.max(0, finite(row.loading_mg_per_g)),
      heldFraction,
      heldPercent: Math.round(heldFraction * 100),
      loadingFraction,
    };
  });
}
