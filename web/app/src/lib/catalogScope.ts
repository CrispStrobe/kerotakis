export type CatalogScope = "mission" | "unlocked" | "all";

const CABINET_VERBS = new Set([
  "filter", "decant", "drain", "cell", "distil", "mix", "transport", "react",
  "dilute", "evaporate", "electrolyse", "grind", "irradiate", "regulate", "stir", "sweep",
]);

/** Instruments a mission script asks the learner to take from the cabinet. */
export function missionEquipment(lines: string[]): string[] {
  const verbs = lines.flatMap((line) => {
    const command = line.trim().split(/\s+/, 1)[0]?.toLowerCase();
    if (command === "titrate") return ["burette"];
    return command && CABINET_VERBS.has(command) ? [command] : [];
  });
  return [...new Set(verbs)];
}
