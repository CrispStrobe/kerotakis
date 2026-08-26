export type MissionSummary = { file: string; name: string; blurb?: string; topic?: string };

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
    topics: ["acids & bases", "water chemistry"],
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
