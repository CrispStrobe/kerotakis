const SEPARATION = new Set(["filter", "decant", "distil", "evaporate", "chromatograph", "transport"]);
const CONDITIONS = new Set(["heat", "cool", "ignite", "wait"]);
const ELECTRICAL = new Set(["cell", "electrolyze"]);

export function commandVerb(command: string): string {
  return command.trim().split(/\s+/, 1)[0]?.toLowerCase() ?? "";
}

/** Human objective beside the exact operator command; never hides the latter. */
export function missionObjective(command: string): string {
  const verb = commandVerb(command);
  if (verb === "add") return "Prepare the requested material";
  if (verb === "new") return "Set up another vessel";
  if (verb === "register") return "Change how closely you observe";
  if (verb === "measure") return "Take the next measurement";
  if (verb === "look" || verb === "inspect") return "Observe the evidence";
  if (verb === "react" || verb === "mix") return "Run the reaction";
  if (verb === "titrate") return "Add carefully until the endpoint";
  if (verb === "seal") return "Control the vessel";
  if (SEPARATION.has(verb)) return "Separate the mixture";
  if (CONDITIONS.has(verb)) return "Change the conditions";
  if (ELECTRICAL.has(verb)) return "Build and test the electrical system";
  return "Carry out the next investigation step";
}

/** Optional procedural orientation, deliberately not an answer key. */
export function missionHint(command: string): string {
  const verb = commandVerb(command);
  if (verb === "add") return "Check the selected vessel and material before adding it.";
  if (verb === "new") return "Empty vessels appear in the Prepare zone.";
  if (verb === "register") return "The register changes detail, never the underlying chemistry.";
  if (verb === "measure" || verb === "look" || verb === "inspect") return "Select the named vessel, then use its observation or measurement tools.";
  if (verb === "titrate") return "Use the burette for controlled additions and watch the instrument reading.";
  if (SEPARATION.has(verb)) return "Connected apparatus can be placed from the equipment cabinet.";
  if (CONDITIONS.has(verb)) return "Condition controls are in the selected vessel's action dock.";
  if (ELECTRICAL.has(verb)) return "Place the electrical apparatus, then connect the named vessels.";
  return "The exact operator instruction remains visible if you want to run it directly.";
}

