export type MissionSummary = {
  file: string;
  name: string;
  blurb?: string;
  topic?: string;
  collection?: string;
  outcome_note?: string;
  boundary_note?: string;
};

export type StoryDistrict = {
  id: string;
  name: string;
  description: string;
  icon: string;
  minimumCompleted: number;
  topics: string[];
  missions: MissionSummary[];
  unlocked: boolean;
  completed: number;
};

const DISTRICTS = [
  {
    id: "discovery-hall",
    name: "Discovery Hall",
    description: "Start with visible changes, careful observation, and safe habits.",
    icon: "✦",
    minimumCompleted: 0,
    topics: ["start here", "safety"],
  },
  {
    id: "matter-gardens",
    name: "Matter Gardens",
    description: "Follow acids, bases, minerals, and water through the living campus.",
    icon: "⚗",
    minimumCompleted: 1,
    topics: ["acids & bases", "water chemistry", "corrosion & materials", "crystals & solubility"],
  },
  {
    id: "energy-yard",
    name: "Energy Yard",
    description: "Work with heat, fire, pressure, and the energy hidden in matter.",
    icon: "♨",
    minimumCompleted: 1,
    topics: ["heat & fire", "gases & pressure"],
  },
  {
    id: "electron-works",
    name: "Electron Works",
    description: "Make electrons move, plate metals, and build chemical power.",
    icon: "ϟ",
    minimumCompleted: 3,
    topics: ["redox & electricity"],
  },
  {
    id: "systems-dock",
    name: "Systems Dock",
    description: "Control rates and separate mixtures with connected apparatus.",
    icon: "◇",
    minimumCompleted: 4,
    topics: ["rates", "separations", "more"],
  },
] as const;

export function missionId(file: string): string {
  return file.replace(/\.lab$/, "");
}

/** Stable ids stay in saves; human titles are translated from spaced words. */
export function missionTitle(id: string): string {
  return missionId(id).replaceAll("-", " ");
}

/** Build the visible research map from shipped missions and stable progress ids. */
export function storyDistricts(
  missions: MissionSummary[],
  completedIds: ReadonlySet<string>,
): StoryDistrict[] {
  const completedTotal = completedIds.size;
  return DISTRICTS.map((district) => {
    const districtMissions = missions.filter((mission) =>
      (district.topics as readonly string[]).includes(mission.topic ?? "more"),
    );
    return {
      ...district,
      topics: [...district.topics],
      missions: districtMissions,
      unlocked: completedTotal >= district.minimumCompleted,
      completed: districtMissions.filter((mission) => completedIds.has(missionId(mission.file))).length,
    };
  });
}

/** Pick one useful continuation without inventing a second progression model.
 * District order and mission export order are stable, so the answer is stable.
 * An active, unlocked, incomplete mission wins; otherwise take the first
 * unlocked incomplete mission on the existing map. */
export function nextUnlockedMission(
  missions: MissionSummary[],
  completedIds: ReadonlySet<string>,
  activeId: string | null = null,
): MissionSummary | null {
  const available = storyDistricts(missions, completedIds)
    .filter((district) => district.unlocked)
    .flatMap((district) => district.missions)
    .filter((mission) => !completedIds.has(missionId(mission.file)));
  return available.find((mission) => missionId(mission.file) === activeId) ?? available[0] ?? null;
}

export function continuationLabel(mission: MissionSummary, activeId: string | null): "continue investigation" | "next investigation" {
  return missionId(mission.file) === activeId ? "continue investigation" : "next investigation";
}

/** Locate a mission on the existing map so its district can be selected. */
export function missionDistrictId(
  missions: MissionSummary[],
  completedIds: ReadonlySet<string>,
  mission: MissionSummary | null,
): string | null {
  if (mission === null) return null;
  return storyDistricts(missions, completedIds)
    .find((district) => district.missions.some((candidate) => candidate.file === mission.file))?.id ?? null;
}
