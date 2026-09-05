export type CatalogScope = "mission" | "unlocked" | "all";

const CABINET_VERBS = new Set([
  "filter", "decant", "drain", "magnet", "cell", "distil", "mix", "transport", "react",
  "bunsen", "centrifuge", "dilute", "evaporate", "electrolyse", "grind", "heat", "irradiate", "regulate", "stir", "sweep",
]);

/** Instruments a mission script asks the learner to take from the cabinet. */
export function missionEquipment(lines: string[]): string[] {
  const verbs = lines.flatMap((line) => {
    const words = line.trim().split(/\s+/);
    const command = words[0]?.toLowerCase();
    if (command === "titrate") return ["burette"];
    if (command === "measure" && words[2]) return [`measure:${words[2].toLowerCase()}`];
    if (command === "smell") return ["measure:smell"];
    if (command === "chromatograph") return ["measure:chromatograph"];
    if (command === "ignite") return ["bunsen"];
    return command && CABINET_VERBS.has(command) ? [command] : [];
  });
  return [...new Set(verbs)];
}
