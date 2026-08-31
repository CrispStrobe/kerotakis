import type { Effect } from "./magnitudes";

/**
 * The renderer's sole combustion authority is a live engine-derived ignite
 * effect. Temperature, colour and other presentation state cannot create one.
 */
export function liveIgnitionEffect(
  effects: readonly Effect[],
  nowMs: number,
  fallbackLifetimeMs = 3000,
): Effect | undefined {
  for (let index = effects.length - 1; index >= 0; index -= 1) {
    const effect = effects[index]!;
    const age = nowMs - effect.at;
    if (effect.kind === "ignite" && age >= 0 && age < (effect.durationMs ?? fallbackLifetimeMs)) {
      return effect;
    }
  }
  return undefined;
}
