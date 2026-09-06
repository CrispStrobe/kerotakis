import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const source = (path: string) => readFileSync(new URL(path, import.meta.url), "utf8");

describe("discovery uses the two live product surfaces", () => {
  const app = source("../App.svelte");

  it("mounts StoryMap for Missions and does not mount retired MissionControl", () => {
    expect(app).toContain('import StoryMap from "./lib/components/StoryMap.svelte"');
    expect(app).toContain("<StoryMap");
    expect(app).not.toContain("MissionControl");
  });

  it("mounts the unified Catalog with engine-owned session access", () => {
    expect(app).toContain("<Catalog");
    const catalog = source("components/Catalog.svelte");
    expect(catalog).toContain("catalog: session.catalog");
    expect(catalog).toContain("data-ready-now");
    expect(catalog).toContain("data-catalog-reason");
  });

  it("shows the live map's exact remaining prerequisite", () => {
    const story = source("components/StoryMap.svelte");
    expect(story).toContain("remainingMissions(district.minimumCompleted, completedCount)");
    expect(story).toContain("remainingMissions(selected.minimumCompleted, completedCount)");
  });
});
