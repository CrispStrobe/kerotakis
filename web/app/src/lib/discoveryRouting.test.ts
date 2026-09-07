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

  it("does not let Concept Map bypass Story mission locks", () => {
    const concept = source("components/ConceptMap.svelte");
    expect(concept).toContain("missionAvailability(missions, session.completedMissions, link.mission)");
    expect(concept).toContain("else if (access?.unlocked) onopenmission?.(link.id)");
    expect(concept).toContain("data-mission-unlocked");
  });

  it("and gates those mission links in Story only, because Sandbox gates nothing", () => {
    // `entryLocked(entry, met, mode)` already makes Sandbox open every
    // catalogue entry. A mission link reading the district gate with no mode
    // would have been the single surface still refusing in Sandbox.
    const concept = source("components/ConceptMap.svelte");
    expect(concept).toContain('link.kind === "mission" && mode === "story"');
  });

  it("names a locked instrument, never the engine's measure: wire tag", () => {
    const catalog = source("components/Catalog.svelte");
    expect(catalog).toContain("function accessName(item: CatalogItem)");
    expect(catalog).toContain("equipmentById(item.id)");
    expect(catalog).not.toContain("t(slugWords(catalogItem.id))");
  });
});
