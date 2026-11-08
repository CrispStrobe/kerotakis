/**
 * Register-aware quick amounts: lv1 speaks kitchen units the grammar
 * already parses; lv2/lv3 speak the lab's. One module so tap-to-add and
 * drag-to-add compose identical commands.
 */

export function quickAmounts(register: string, phase: string): string[] {
  if (register === "lv1") {
    return phase === "liquid" ? ["1cup", "100mL"] : ["1pinch", "1g"];
  }
  // A lab register counts in moles first: 0.1 mol is a sensible bench
  // quantity for a solid or a liquid alike, and it is the unit every
  // equation and readout on these registers speaks.
  return phase === "liquid" ? ["0.1mol", "10mL", "100mL"] : ["0.1mol", "0.01mol", "1g"];
}

/** What a drop uses: the register's most natural amount. */
export function defaultAmount(register: string, phase: string): string {
  return quickAmounts(register, phase)[0]!;
}

export type AmountUnit = "mL" | "L" | "g" | "mol" | "drop" | "pinch";

export function amountUnits(register: string, phase: string): AmountUnit[] {
  if (phase === "liquid") {
    return register === "lv1" ? ["mL", "L", "drop", "g", "mol"] : ["mol", "mL", "L", "g"];
  }
  return register === "lv1" ? ["g", "pinch", "mol"] : ["mol", "g"];
}

/** What a lab register starts the amount form at, for every phase. */
export const LAB_DEFAULT_MOL = 0.1;

/**
 * A useful starting value. lv1 is scaled to the selected vessel in kitchen
 * units; lv2/lv3 start at {@link LAB_DEFAULT_MOL} mol whatever the phase.
 */
export function suggestedAmount(
  phase: string,
  capacityMl: number,
  register = "lv1",
): { value: number; unit: AmountUnit } {
  if (register !== "lv1") return { value: LAB_DEFAULT_MOL, unit: "mol" };
  if (phase !== "liquid") return { value: 1, unit: "g" };
  if (capacityMl >= 1000) {
    return { value: Math.max(0.1, Math.round(capacityMl / 250) / 4), unit: "L" };
  }
  const fraction = capacityMl >= 100 ? capacityMl / 4 : capacityMl / 5;
  return { value: Math.max(1, Math.round(fraction / 5) * 5), unit: "mL" };
}
