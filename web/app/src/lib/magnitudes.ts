/**
 * GUI-059: effect magnitudes — maps engine event amounts onto visual
 * scale factors for the vessel's transient effects. Every factor names
 * its source event field so the link is auditable.
 *
 * All scale functions return a number in [0, 1] that the Vessel component
 * uses as a CSS variable. 0 = minimum visible, 1 = maximum (clamp).
 */

/** The shape of an engine event, loosely typed (serde JSON). */
export type EngineEvent = Record<string, unknown>;

/** A visual effect with magnitude, produced by {@link effectFromEvent}. */
export interface Effect {
  kind: string;
  at: number;
  /** 0–1 visual intensity, from the event's amount field. */
  magnitude: number;
  /** Flame colour CSS value, if the event carries one. */
  flameColour?: string;
}

/** Clamp `x` into [0, 1], scaling linearly from 0 at `lo` to 1 at `hi`. */
function scale(x: number, lo: number, hi: number): number {
  if (hi <= lo) return x >= lo ? 1 : 0;
  return Math.max(0, Math.min(1, (x - lo) / (hi - lo)));
}

/**
 * Named-colour → CSS colour for flame rendering.
 * Keys match the engine's `FlameTest.colour` / `Ignited.flame` strings.
 */
const FLAME_COLOURS: Record<string, string> = {
  lilac: "#c8a2c8",
  violet: "#9b30ff",
  yellow: "#ffd700",
  orange: "#ff8c00",
  "brick-red": "#cb4154",
  red: "#ff2400",
  green: "#00e676",
  "blue-green": "#0dbf8c",
  blue: "#1e90ff",
  crimson: "#dc143c",
  white: "#ffffff",
};

// ── Per-event-kind magnitude extractors ──────────────────────────────
// Each returns [magnitude, flameColour?]. The magnitude tracks a single
// event field named in the comment; the constants bound what "small" and
// "large" look like on the bench.

// event.moles — GasEvolved: 0.001 mol is a wisp, 0.1 mol is vigorous.
function gasMag(e: EngineEvent): number {
  return scale(Number(e.moles ?? 0), 0.001, 0.1);
}

// event.moles — Precipitated: 0.0005 mol is a few specks, 0.05 mol is
// a heavy snowfall.
function precipMag(e: EngineEvent): number {
  return scale(Number(e.moles ?? 0), 0.0005, 0.05);
}

// event.moles — Evaporated/Distilled: 0.01 mol is gentle, 0.5 mol is
// a rolling boil.
function steamMag(e: EngineEvent): number {
  return scale(Number(e.moles ?? 0), 0.01, 0.5);
}

// event.moles — Electrolysed: 0.0001 mol is a trickle, 0.01 mol is
// vigorous bubbling.
function electroMag(e: EngineEvent): number {
  return scale(Number(e.moles ?? 0), 0.0001, 0.01);
}

// event.fraction_a + event.fraction_b — Mixed: total pour fraction
// from 0 (nothing moved) to 1 (both vessels emptied).
function mixMag(e: EngineEvent): number {
  const a = Number(e.fraction_a ?? 0);
  const b = Number(e.fraction_b ?? 0);
  return scale(a + b, 0.1, 1.5);
}

// event.volume (Liters) — Diluted: 0.01 L is a squirt, 0.5 L is a flood.
function diluteMag(e: EngineEvent): number {
  return scale(Number(e.volume ?? 0), 0.01, 0.5);
}

// event.flame / event.colour — Ignited / FlameTest: magnitude is always
// 1 (fire is fire), but the colour maps to a CSS value.
function flameMag(e: EngineEvent): [number, string | undefined] {
  const colourKey = String(e.flame ?? e.colour ?? "");
  const css = FLAME_COLOURS[colourKey];
  return [1, css];
}

/**
 * Map one engine event to a visual effect with magnitude.
 * Returns null if the event kind has no visual mapping.
 */
export function effectFromEvent(e: EngineEvent): Effect | null {
  const kind = String(e.event ?? "");
  const now = Date.now();

  switch (kind) {
    case "gas_evolved":
      return { kind: "vent", at: now, magnitude: gasMag(e) };
    case "precipitated":
      return { kind: "precipitate", at: now, magnitude: precipMag(e) };
    case "evaporated":
    case "distilled":
      return { kind: "evaporate", at: now, magnitude: steamMag(e) };
    case "electrolysed":
      return { kind: "electrolyse", at: now, magnitude: electroMag(e) };
    case "mixed":
      return { kind: "swirl", at: now, magnitude: mixMag(e) };
    case "diluted":
      return { kind: "swirl", at: now, magnitude: diluteMag(e) };
    case "dissolved":
      return { kind: "dissolve", at: now, magnitude: 1 };
    case "plated":
      return { kind: "plate", at: now, magnitude: 1 };
    case "ignited":
    case "flame_test": {
      const [mag, colour] = flameMag(e);
      return { kind: "ignite", at: now, magnitude: mag, flameColour: colour };
    }
    default:
      return null;
  }
}

/**
 * Vessel ID from an event — events use `vessel`, `from`, or `into`.
 */
export function vesselOf(e: EngineEvent): number {
  return Number(e.vessel ?? e.from ?? e.into ?? 0);
}
