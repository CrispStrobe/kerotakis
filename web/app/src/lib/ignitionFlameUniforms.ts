import type { Effect } from "./magnitudes";

export type FlameRgb = readonly [red: number, green: number, blue: number];

export interface IgnitionFlameUniforms {
  active: boolean;
  /** Presentation intensity only; it must never be fed back to the engine. */
  intensity: number;
  /** Renderer-ready sRGB channels, each bounded to 0–1. */
  colour: FlameRgb;
  /** Stable presentation noise seed, bounded to 0–1. */
  seed: number;
}

export interface IgnitionFlameInput {
  /** The caller's currently-live effect, or null once its UI lifetime ends. */
  effect: Effect | null | undefined;
  vesselIdentity: number | string;
  reducedMotion?: boolean;
}

const DEFAULT_COLOUR: FlameRgb = [1, 0.5490196078431373, 0];

/**
 * This is deliberately the presentation projection of magnitudes.ts's
 * engine-curated flame palette. It is not a chemical lookup table: unknown
 * values receive one deterministic visual fallback and add no new chemistry.
 */
const CURATED_COLOURS: Readonly<Record<string, FlameRgb>> = {
  "#c8a2c8": [200 / 255, 162 / 255, 200 / 255],
  "#9b30ff": [155 / 255, 48 / 255, 1],
  "#ffd700": [1, 215 / 255, 0],
  "#ff8c00": DEFAULT_COLOUR,
  "#cb4154": [203 / 255, 65 / 255, 84 / 255],
  "#ff2400": [1, 36 / 255, 0],
  "#00e676": [0, 230 / 255, 118 / 255],
  "#0dbf8c": [13 / 255, 191 / 255, 140 / 255],
  "#1e90ff": [30 / 255, 144 / 255, 1],
  "#dc143c": [220 / 255, 20 / 255, 60 / 255],
  "#ffffff": [1, 1, 1],
};

function boundedMagnitude(value: unknown): number {
  const magnitude = typeof value === "number" ? value : Number.NaN;
  if (!Number.isFinite(magnitude)) return 0;
  return Math.max(0, Math.min(1, magnitude));
}

/** FNV-1a supplies repeatability, not physical or chemical meaning. */
function vesselSeed(identity: number | string): number {
  const text = String(identity);
  let hash = 0x811c9dc5;
  for (let index = 0; index < text.length; index += 1) {
    hash ^= text.charCodeAt(index);
    hash = Math.imul(hash, 0x01000193);
  }
  return (hash >>> 0) / 0xffffffff;
}

/**
 * Pure visual mapping. Effect is the sole live-state authority; this function
 * neither infers ignition from temperature nor represents soot, smoke, solver
 * state, or any other chemistry. Reduced motion disables the animated tier.
 */
export function ignitionFlameUniforms(input: IgnitionFlameInput): IgnitionFlameUniforms {
  const active = input.effect?.kind === "ignite" && input.reducedMotion !== true;
  const colour = CURATED_COLOURS[input.effect?.flameColour?.toLowerCase() ?? ""] ?? DEFAULT_COLOUR;
  return {
    active,
    intensity: active ? boundedMagnitude(input.effect?.magnitude) : 0,
    colour,
    seed: vesselSeed(input.vesselIdentity),
  };
}
