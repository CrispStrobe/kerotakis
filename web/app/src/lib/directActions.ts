import { heatSource } from "./apparatus";

export type QuickActionTone = "primary" | "instrument" | "action" | "discovery";

/**
 * The three readings the vessel dock carries, as instrument tokens.
 *
 * They are landmarks: the same three buttons in the same place on every
 * vessel, which is what makes them worth hard-coding at all. Because they
 * never move, the quick-access strip must never offer them — a learner who
 * measures pH from the dock does not want pH to also appear in MESSEN and
 * push something they cannot otherwise reach out of a four-slot row. This
 * constant is what `instrumentRecents.ts` excludes, so the two lists cannot
 * drift apart.
 */
export const DOCK_INSTRUMENTS: readonly string[] = ["eyes", "thermometer", "ph"];

export interface QuickAction {
  id: string;
  icon: string;
  label: string;
  line: string;
  tone: QuickActionTone;
}

/** High-frequency vessel gestures, compiled to the same public grammar as
 * the CLI. Keeping this pure makes direct manipulation mechanically testable. */
export function vesselQuickActions(vessel: number, boundary: string): QuickAction[] {
  const v = `v${vessel + 1}`;
  return [
    { id: "stir", icon: "↻", label: "stir", line: `stir ${v} 500rpm 10s`, tone: "instrument" },
    // The source is named even when it is the bench default. The engine
    // caps a vessel at the flame heating it, and a `heat` line with no
    // `on <source>` clause claims a laboratory burner by omission — which
    // is the one thing a candle is not. `ApparatusForm` states it for the
    // same reason; this states it from the same table rather than from a
    // second copy of the word "burner".
    { id: "heat", icon: "↑", label: "heat", line: `heat ${v} 10kJ on ${heatSource(undefined).value}`, tone: "action" },
    // `cool` takes no source: the cooling bath is the only one there is,
    // and the engine's grammar has no clause to name it with.
    { id: "cool", icon: "❄", label: "cool", line: `cool ${v} 10kJ`, tone: "instrument" },
    { id: "look", icon: "◉", label: "look", line: `measure ${v} eyes`, tone: "primary" },
    { id: "temperature", icon: "°", label: "temperature", line: `measure ${v} thermometer`, tone: "primary" },
    { id: "ph", icon: "pH", label: "pH", line: `measure ${v} ph`, tone: "primary" },
    boundary === "open"
      ? { id: "seal", icon: "⌒", label: "seal", line: `seal ${v} 500mL`, tone: "discovery" }
      : { id: "open", icon: "⌁", label: "open", line: `open ${v}`, tone: "discovery" },
  ];
}

export type TwoVesselAction = "filter" | "decant" | "drain" | "magnet" | "cell" | "distil";

/** Three taps become the engine's real thermodynamic MIX operation. */
export function mixLine(
  a: number,
  b: number,
  into: number,
  fractionA = 0.5,
  fractionB = 0.5,
): string | null {
  if (![a, b, into].every((v) => Number.isInteger(v) && v >= 0) || new Set([a, b, into]).size !== 3) {
    return null;
  }
  if (![fractionA, fractionB].every((f) => Number.isFinite(f) && f > 0 && f <= 1)) return null;
  return `mix v${a + 1} ${fractionA} v${b + 1} ${fractionB} into v${into + 1}`;
}

export function twoVesselLine(
  verb: TwoVesselAction,
  from: number,
  to: number,
  fraction = 0.5,
): string | null {
  if (!Number.isInteger(from) || !Number.isInteger(to) || from < 0 || to < 0 || from === to) {
    return null;
  }
  if ((verb === "decant" || verb === "distil") &&
      (!Number.isFinite(fraction) || fraction <= 0 || fraction > 1)) {
    return null;
  }
  return verb === "decant" || verb === "distil"
    ? `${verb} v${from + 1} v${to + 1} ${fraction}`
    : `${verb} v${from + 1} v${to + 1}`;
}
