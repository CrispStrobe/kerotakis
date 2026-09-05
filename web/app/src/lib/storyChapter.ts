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

/**
 * What closing the case is worth: a permanent analytical instrument.
 *
 * DERIVED from the completed leads, never stored as an entitlement. A
 * granted-rewards list is a second copy of the truth, and two copies can
 * disagree — a failed write drops the reward, a retried one grants it twice.
 * Deriving makes both impossible: the case is complete or it is not, and the
 * spectrometer is on the wall exactly when it is.
 */
export const CONTAMINATED_SAMPLE_AWARD = "measure:uvvis";

/** What the case award is called on the debrief and the instrument wall. */
export const CASE_AWARDS: Record<string, { verb: string; title: string; description: string }> = {
  [CONTAMINATED_SAMPLE_AWARD]: {
    verb: CONTAMINATED_SAMPLE_AWARD,
    title: "UV/Vis spectrophotometer",
    description: "Identify dissolved substances by the light they absorb. Yours permanently for closing the case.",
  },
};

export function caseAwardDetail(verb: string): { verb: string; title: string; description: string } | null {
  return CASE_AWARDS[verb] ?? null;
}

const CORE_LEAD_IDS: readonly string[] = LEADS.filter((lead) => !lead.optional).map((lead) => lead.id);

/** Every core lead secured. The optional safety audit is not required — it
 * is a discovery, and the case must close without it. */
export function contaminatedSampleComplete(completed: ReadonlySet<string>): boolean {
  return CORE_LEAD_IDS.every((id) => completed.has(id));
}

/** Permanent equipment the learner's closed cases have earned. */
export function caseAwardedTools(completed: ReadonlySet<string>): string[] {
  return contaminatedSampleComplete(completed) ? [CONTAMINATED_SAMPLE_AWARD] : [];
}

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

/**
 * Discovery Hall's missions that the case file does not name.
 *
 * The case is four leads. The district is seven missions — `start here`
 * plus `safety` in `tools/lessons-index.py` — and the board rendered the
 * four, so the district header honestly counted "0 of 7 complete" beside a
 * board on which three of them could not be opened at all. Nothing locked
 * them; they were simply not drawn, which is the worse failure because it
 * has no explanation to read.
 *
 * These are not leads and are deliberately not promoted into the case:
 * the case's evidence bar is three core leads and stays three. They are
 * the rest of the district, listed so the learner can reach them.
 */
export function missionsBeyondTheCase(missions: MissionSummary[]): MissionSummary[] {
  const named = new Set(LEADS.map((lead) => lead.id as string));
  return missions.filter((mission) => !named.has(missionId(mission.file)));
}

export function contaminatedSampleProgress(leads: CaseLead[]): { done: number; total: number; complete: boolean } {
  const core = leads.filter((lead) => !lead.optional);
  const done = core.filter((lead) => lead.done).length;
  return { done, total: core.length, complete: core.length === 3 && done === core.length };
}
