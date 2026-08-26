export type QuickActionTone = "primary" | "instrument" | "action" | "discovery";

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
    { id: "stir", icon: "↻", label: "stir", line: `stir ${v}`, tone: "instrument" },
    { id: "heat", icon: "↑", label: "heat", line: `heat ${v} 10kJ`, tone: "action" },
    { id: "cool", icon: "❄", label: "cool", line: `cool ${v} 10kJ`, tone: "instrument" },
    { id: "look", icon: "◉", label: "look", line: `measure ${v} eyes`, tone: "primary" },
    { id: "temperature", icon: "°", label: "temperature", line: `measure ${v} thermometer`, tone: "primary" },
    { id: "ph", icon: "pH", label: "pH", line: `measure ${v} ph`, tone: "primary" },
    boundary === "open"
      ? { id: "seal", icon: "⌒", label: "seal", line: `seal ${v} 500mL`, tone: "discovery" }
      : { id: "open", icon: "⌁", label: "open", line: `open ${v}`, tone: "discovery" },
  ];
}

export type TwoVesselAction = "filter" | "decant" | "drain" | "cell" | "distil";

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
