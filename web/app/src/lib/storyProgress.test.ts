import { describe, expect, it } from "vitest";
import { continuationLabel, missionDistrictId, missionId, missionTitle, nextUnlockedMission, storyDistricts, type MissionSummary } from "./storyProgress";

/** The seven Discovery Hall ships: six `start here` plus one `safety`,
 * exactly as `tools/lessons-index.py` buckets them. */
const discoveryHall: MissionSummary[] = [
  { file: "silver-and-salt.lab", name: "silver and salt", topic: "start here" },
  { file: "first-warmth.lab", name: "first warmth", topic: "start here" },
  { file: "one-thing-at-a-time.lab", name: "one thing at a time", topic: "start here" },
  { file: "pepper-and-soap.lab", name: "pepper and soap", topic: "start here" },
  { file: "oil-water-colour.lab", name: "oil water colour", topic: "start here" },
  { file: "magic-milk.lab", name: "magic milk", topic: "start here" },
  { file: "never-mix.lab", name: "never mix", topic: "safety" },
];

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

describe("the opening district is a choice, not a queue", () => {
  /**
   * The district header counts every mission it holds, so a board that
   * draws fewer than that reads as "0 of 7 complete" beside a list of
   * three you cannot open. Nothing was ever locked — they were simply not
   * rendered, which is the worse failure because it has no explanation.
   */
  it("holds seven missions, none of them gated behind another", () => {
    const hall = storyDistricts(discoveryHall, new Set())[0]!;
    expect(hall.id).toBe("discovery-hall");
    expect(hall.unlocked).toBe(true);
    expect(hall.missions).toHaveLength(7);
    // `minimumCompleted: 0` is the whole gate, and it is satisfied on a
    // brand-new save: there is no per-mission prerequisite to state.
    expect(hall.minimumCompleted).toBe(0);
  });

  it("recommends one without that being the only one available", () => {
    const completed = new Set<string>();
    const next = nextUnlockedMission(discoveryHall, completed);
    expect(next?.file).toBe("silver-and-salt.lab");
    // Every other mission in the hall is equally startable — the
    // recommendation is an order, not a lock.
    const hall = storyDistricts(discoveryHall, completed)[0]!;
    expect(hall.missions.map((mission) => mission.file)).toContain("magic-milk.lab");
    expect(hall.missions.map((mission) => mission.file)).toContain("never-mix.lab");
  });

  it("keeps recommending from the hall after one anywhere is finished", () => {
    const next = nextUnlockedMission(discoveryHall, new Set(["silver-and-salt"]));
    expect(next?.file).toBe("first-warmth.lab");
  });
});
