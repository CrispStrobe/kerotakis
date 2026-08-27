import { missionId, type MissionSummary } from "./storyProgress";
import { outcomeMissionContract } from "./outcomeMission";

export type CaseLead = {
  id: string;
  mission: MissionSummary;
  objective: string;
  evidence: string;
  optional: boolean;
  outcomeAssessed: boolean;
  done: boolean;
};

const LEADS = [
  {
    id: "silver-and-salt",
    objective: "Trace the mineral contamination",
    evidence: "Use a selective reaction to reveal a dissolved ion.",
    optional: false,
  },
  {
    id: "first-warmth",
    objective: "Establish the thermal baseline",
    evidence: "Show how mixing history changes the sample temperature.",
    optional: false,
  },
  {
    id: "one-thing-at-a-time",
    objective: "Separate the unknown mixture",
    evidence: "Turn one colourless sample into distinct measured components.",
    optional: false,
  },
  {
    id: "never-mix",
    objective: "Audit the abandoned workbench",
    evidence: "Optional: identify dangerous combinations before anyone handles them.",
    optional: true,
  },
] as const;

export function contaminatedSampleLeads(
  missions: MissionSummary[],
  completed: ReadonlySet<string>,
): CaseLead[] {
  return LEADS.flatMap((lead) => {
    const mission = missions.find((candidate) => missionId(candidate.file) === lead.id);
    return mission ? [{
      ...lead,
      mission,
      outcomeAssessed: outcomeMissionContract(lead.id) !== null,
      done: completed.has(lead.id),
    }] : [];
  });
}

export function contaminatedSampleProgress(leads: CaseLead[]): { done: number; total: number; complete: boolean } {
  const core = leads.filter((lead) => !lead.optional);
  const done = core.filter((lead) => lead.done).length;
  return { done, total: core.length, complete: core.length === 3 && done === core.length };
}
