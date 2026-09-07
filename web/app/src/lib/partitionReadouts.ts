import type { ScenePartition } from "./host/EngineHost";

export interface PartitionReadout extends ScenePartition {
  fractionLower: number;
  fractionUpper: number;
  lowerPercent: number;
  upperPercent: number;
}

/** Display normalization only; the equilibrium remains engine-owned. */
export function partitionReadouts(rows?: ScenePartition[]): PartitionReadout[] {
  return (rows ?? []).map((row) => {
    const raw = Number(row.fraction_lower);
    const fractionLower = Number.isFinite(raw) ? Math.min(1, Math.max(0, raw)) : 0;
    const fractionUpper = 1 - fractionLower;
    return {
      ...row,
      fractionLower,
      fractionUpper,
      lowerPercent: Math.round(fractionLower * 100),
      upperPercent: Math.round(fractionUpper * 100),
    };
  });
}
