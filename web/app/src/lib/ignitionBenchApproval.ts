import type { Effect } from "./magnitudes";
import { liveIgnitionEffect } from "./ignitionPresentation";

export type VesselEffects = Readonly<Record<number, readonly Effect[] | undefined>>;

/**
 * Pure bench-level approval projection for the optional ignition renderer.
 * A shared device is useful only while at least one vessel owns a live,
 * engine-derived ignite effect. Temperature and flame-test presentation are
 * deliberately absent from this boundary.
 */
export function liveIgnitionVessels(
  effectsByVessel: VesselEffects,
  nowMs: number,
  fallbackLifetimeMs = 3000,
): number[] {
  const vessels: number[] = [];
  for (const [identity, effects] of Object.entries(effectsByVessel)) {
    if (!effects || !liveIgnitionEffect(effects, nowMs, fallbackLifetimeMs)) continue;
    const vessel = Number(identity);
    if (Number.isSafeInteger(vessel) && vessel >= 0) vessels.push(vessel);
  }
  vessels.sort((left, right) => left - right);
  return vessels;
}

export function benchIgnitionApproved(
  effectsByVessel: VesselEffects,
  nowMs: number,
  fallbackLifetimeMs = 3000,
): boolean {
  for (const effects of Object.values(effectsByVessel)) {
    if (effects && liveIgnitionEffect(effects, nowMs, fallbackLifetimeMs)) return true;
  }
  return false;
}
