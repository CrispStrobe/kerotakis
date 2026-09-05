import { describe, expect, it } from "vitest";
import { continuationLabel, missionDistrictId, missionId, missionTitle, nextUnlockedMission, storyDistricts, type MissionSummary } from "./storyProgress";

const missions: MissionSummary[] = [
  { file: "silver-and-salt.lab", name: "silver and salt", topic: "start here" },
  { file: "never-mix.lab", name: "never mix", topic: "safety" },
  { file: "fizz.lab", name: "fizz", topic: "acids & bases" },
  { file: "rusting.lab", name: "rusting", topic: "corrosion & materials" },
  { file: "fire.lab", name: "fire", topic: "heat & fire" },
  { file: "electrolysis.lab", name: "electrolysis", topic: "redox & electricity" },
  { file: "rates.lab", name: "rates", topic: "rates" },
  { file: "rock-candy.lab", name: "rock candy", topic: "crystals & solubility", collection: "crystal lab" },
];

describe("story progression", () => {
  it("uses stable file ids rather than localized titles", () => {
    expect(missionId("silver-and-salt.lab")).toBe("silver-and-salt");
    expect(missionTitle("silver-and-salt")).toBe("silver and salt");
  });

  it("starts with a real district and opens two routes after one completion", () => {
    const start = storyDistricts(missions, new Set());
    expect(start.find((district) => district.id === "discovery-hall")?.unlocked).toBe(true);
    expect(start.filter((district) => district.unlocked)).toHaveLength(1);

    const afterOne = storyDistricts(missions, new Set(["silver-and-salt"]));
    expect(afterOne.filter((district) => district.unlocked).map((district) => district.id)).toEqual([
      "discovery-hall",
      "matter-gardens",
      "energy-yard",
    ]);
    expect(afterOne[0]?.completed).toBe(1);
    expect(afterOne.find((district) => district.id === "matter-gardens")?.missions.map((mission) => mission.file))
      .toContain("rusting.lab");
    expect(afterOne.find((district) => district.id === "matter-gardens")?.missions)
      .toContainEqual(expect.objectContaining({ file: "rock-candy.lab" }));
  });

  it("preserves every shipped mission exactly once", () => {
    const districts = storyDistricts(missions, new Set());
    expect(districts.flatMap((district) => district.missions).map((mission) => mission.file).sort())
      .toEqual(missions.map((mission) => mission.file).sort());
  });

  it("selects an active continuation, then the first unlocked incomplete mission", () => {
    expect(nextUnlockedMission(missions, new Set(), "never-mix")?.file).toBe("never-mix.lab");
    expect(nextUnlockedMission(missions, new Set(["silver-and-salt"]))?.file).toBe("never-mix.lab");
    expect(nextUnlockedMission(missions, new Set(["silver-and-salt", "never-mix"]))?.file).toBe("fizz.lab");
    expect(nextUnlockedMission(missions, new Set(missions.map((mission) => missionId(mission.file))))).toBeNull();
  });

  it("locates the continuation on the existing mission map", () => {
    const continuation = nextUnlockedMission(missions, new Set(["silver-and-salt", "never-mix"]));
    expect(missionDistrictId(missions, new Set(["silver-and-salt", "never-mix"]), continuation)).toBe("matter-gardens");
    expect(missionDistrictId(missions, new Set(), null)).toBeNull();
  });

  it("labels an active mission Continue and a selected successor Next", () => {
    expect(continuationLabel(missions[1]!, "never-mix")).toBe("continue investigation");
    expect(continuationLabel(missions[2]!, "never-mix")).toBe("next investigation");
  });
});
