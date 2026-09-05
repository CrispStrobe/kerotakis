import type { SceneVessel } from "./host/EngineHost";

export interface EnzymeReadout {
  family: string;
  material: string;
  substrate: string;
  percent: number;
}

/** Format only the engine's optional persistent projection. Older scene-v1
 * payloads have no readout; malformed fractions are safely bounded. */
export function enzymeReadouts(vessel: Pick<SceneVessel, "enzyme_hydrolysis">): EnzymeReadout[] {
  return (vessel.enzyme_hydrolysis ?? []).map((progress) => ({
    family: progress.family,
    material: progress.material,
    substrate: progress.substrate,
    percent: Math.round(Math.min(1, Math.max(0, progress.converted_fraction)) * 100),
  }));
}
