/**
 * Register-aware quick amounts: lv1 speaks kitchen units the grammar
 * already parses; lv2/lv3 speak the lab's. One module so tap-to-add and
 * drag-to-add compose identical commands.
 */

export function quickAmounts(register: string, phase: string): string[] {
  if (register === "lv1") {
    return phase === "liquid" ? ["1cup", "100mL"] : ["1pinch", "1g"];
  }
  return phase === "liquid" ? ["10mL", "100mL", "1mol"] : ["1g", "0.01mol", "0.1mol"];
}

/** What a drop uses: the register's most natural amount. */
export function defaultAmount(register: string, phase: string): string {
  return quickAmounts(register, phase)[0]!;
}

export type AmountUnit = "mL" | "L" | "g" | "mol" | "drop" | "pinch";

export function amountUnits(register: string, phase: string): AmountUnit[] {
  if (phase === "liquid") {
    return register === "lv1" ? ["mL", "L", "drop", "g", "mol"] : ["mL", "L", "mol", "g"];
  }
  return register === "lv1" ? ["g", "pinch", "mol"] : ["g", "mol"];
}

/** A useful starting value, scaled to the selected vessel rather than a global button. */
export function suggestedAmount(
  phase: string,
  capacityMl: number,
): { value: number; unit: AmountUnit } {
  if (phase !== "liquid") return { value: 1, unit: "g" };
  if (capacityMl >= 1000) {
    return { value: Math.max(0.1, Math.round(capacityMl / 250) / 4), unit: "L" };
  }
  const fraction = capacityMl >= 100 ? capacityMl / 4 : capacityMl / 5;
  return { value: Math.max(1, Math.round(fraction / 5) * 5), unit: "mL" };
}
