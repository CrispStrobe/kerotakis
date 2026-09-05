/**
 * What a concept leads to — the map's missing half.
 *
 * `ConceptMap` could name 157 concepts and, for each, the codex entries
 * that declare it. That is one of four things the bench can teach with,
 * and the other three were unreachable from the map: a learner who tapped
 * "precipitation" saw a list of catalogue slugs and no way into the kids
 * task, the guided lesson, or the mission that puts the same chemistry on
 * the bench. This file is the join, and it is deliberately made of links
 * that already exist rather than a new table someone has to maintain.
 *
 * Three relations, each named for exactly what it is — the label a learner
 * reads is the claim this module can defend:
 *
 *   - `teaches` — the entry itself declares the concept (`concepts`), or a
 *     kids task cross-references such an entry by id (`KidsExperiment.codex`,
 *     "reviewed exact cross-references", never a search).
 *   - `evidence` — a mission's outcome contract secures a typed engine event
 *     (`precipitated:AgCl`) that a teaching entry also claims in its
 *     `expect.events`. Same event, same engine, both sides authored.
 *   - `materials` — every reagent a teaching entry's setup script uses is
 *     in the lesson's own kit, so running the lesson puts that chemistry
 *     within reach. This is a materials affiliation and says so; it is NOT
 *     a claim that the lesson teaches the concept, because nothing in a
 *     `.lab` file makes that claim and inventing one here would be a second
 *     curriculum with no source of truth.
 *
 * Nothing here decides what a learner has *met* — `metConcepts` stays the
 * only answer to that, and it still reads only bench runs that agreed with
 * the entry's claims.
 */
import { scriptKit, type CodexEntry } from "./codex";
import type { KidsExperiment } from "./kidsCatalog";
import { allCriteria, outcomeMissionContract } from "./outcomeMission";
import { missionId, type MissionSummary } from "./storyProgress";

export type ConceptRelation = "teaches" | "evidence" | "materials";

export type ConceptLink =
  | { kind: "experiment"; id: string; relation: "teaches"; done: boolean; entry: CodexEntry }
  | { kind: "kids"; id: string; relation: "teaches"; done: boolean; kid: KidsExperiment }
  | { kind: "mission"; id: string; relation: "evidence" | "materials"; done: boolean; mission: MissionSummary };

export type ConceptSources = {
  entries: readonly CodexEntry[];
  kids?: readonly KidsExperiment[];
  missions?: readonly MissionSummary[];
  /** Codex entry ids run to a green check. */
  completedExperiments?: ReadonlySet<string>;
  /** Stable lesson ids completed in Story. */
  completedMissions?: ReadonlySet<string>;
};

/** The engine events one criterion can secure, in `expect.events` spelling. */
function criterionEvents(event: string | undefined, species: string | undefined): string[] {
  if (event === undefined) return [];
  return species === undefined ? [event] : [event, `${event}:${species}`];
}

/**
 * Every activity affiliated with one concept, experiments first.
 *
 * Order inside each kind is the source order the catalogues ship in, so
 * the list is stable between renders and between devices; the caller is
 * free to re-sort by anything it can render (`ConceptMap` puts ready
 * experiments first, by translated label).
 */
export function conceptLinks(concept: string, sources: ConceptSources): ConceptLink[] {
  const {
    entries,
    kids = [],
    missions = [],
    completedExperiments = new Set<string>(),
    completedMissions = new Set<string>(),
  } = sources;

  const teaching = entries.filter((entry) => entry.concepts?.includes(concept));
  if (teaching.length === 0) return [];
  const teachingIds = new Set(teaching.map((entry) => entry.id));
  const claimedEvents = new Set(teaching.flatMap((entry) => entry.expect?.events ?? []));
  // An entry with no reagents of its own (a pure calculation, say) would
  // make the subset test true for every lesson, so it is not offered.
  const teachingKits = teaching
    .map((entry) => scriptKit(entry.setup.script))
    .filter((kit) => kit.length > 0);

  const links: ConceptLink[] = teaching.map((entry) => ({
    kind: "experiment",
    id: entry.id,
    relation: "teaches",
    done: completedExperiments.has(entry.id),
    entry,
  }));

  for (const kid of kids) {
    const linked = (kid.codex ?? []).filter((id) => teachingIds.has(id));
    if (linked.length === 0) continue;
    const lessonId = kid.lesson?.replace(/\.lab$/, "") ?? null;
    links.push({
      kind: "kids",
      id: kid.id,
      relation: "teaches",
      done:
        linked.every((id) => completedExperiments.has(id)) &&
        (lessonId === null || completedMissions.has(lessonId)),
      kid,
    });
  }

  for (const mission of missions) {
    const id = missionId(mission.file);
    const contract = outcomeMissionContract(id);
    const secures = contract
      ? allCriteria(contract).some((criterion) =>
          criterionEvents(criterion.event, criterion.species).some((event) => claimedEvents.has(event)),
        )
      : false;
    const kit = new Set(mission.kit ?? []);
    const supplies = kit.size > 0 && teachingKits.some((needed) => needed.every((item) => kit.has(item)));
    if (!secures && !supplies) continue;
    links.push({
      kind: "mission",
      id: mission.file,
      relation: secures ? "evidence" : "materials",
      done: completedMissions.has(id),
      mission,
    });
  }

  return links;
}

/** The one-line reason a link is offered. One complete key per relation. */
export function relationLabel(
  relation: ConceptRelation,
): "teaches this concept" | "gathers evidence for this concept" | "puts these materials on the bench" {
  switch (relation) {
    case "teaches":
      return "teaches this concept";
    case "evidence":
      return "gathers evidence for this concept";
    case "materials":
      return "puts these materials on the bench";
  }
}
