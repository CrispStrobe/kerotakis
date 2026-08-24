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
