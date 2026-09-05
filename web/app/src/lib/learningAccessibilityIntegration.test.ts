import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

const components = resolve(import.meta.dirname, "components");
const source = (name: string) => readFileSync(resolve(components, name), "utf8");

describe("learning navigation accessibility wiring", () => {
  it("offers the derived continuation and opens its existing district", () => {
    const story = source("StoryMap.svelte");
    expect(story).toContain("missionDistrictId(missions, completed, continuation)");
    expect(story).toContain('class="next-investigation"');
    expect(story).toContain("t(continuationLabel(nextMission, active))");
    expect(story).toContain("onstart(nextMission.file)");
  });

  // Both tiers live in one component now (the unified catalogue), so both
  // sets of assertions read the same file. They are kept separate because
  // they are separate promises: the experiment tier's filters and the KIDS
  // tier's progress cards can regress independently.
  it("exposes a named, pressed-state experiment completion filter", () => {
    const catalog = source("Catalog.svelte");
    expect(catalog).toContain('role="group" aria-label={t("completion status")}');
    expect(catalog).toContain("aria-pressed={progress === value}");
    expect(catalog).toContain("aria-pressed={view === key}");
    expect(catalog).toContain("experimentProgressLabel(e, session.completedExperiments)");
  });

  it("keeps KIDS progress and Continue/Replay actions in the catalog cards", () => {
    const kids = source("Catalog.svelte");
    expect(kids).toContain('data-progress={links.progress}');
    expect(kids).toContain("aria-pressed={status === null}");
    expect(kids).toContain("aria-pressed={status === value}");
    expect(kids).toContain("{links.completedLearning}/{links.linkedLearning}");
    expect(kids).toContain("guidedLearningLabel(links.lessonCompleted)");
    expect(kids).toContain("codexLearningLabel(links.codexCompleted.includes(id))");
  });
});

describe("persistent vessel accessibility wiring", () => {
  it("names the whole current scene and exposes visible gel/enzyme readouts", () => {
    const vessel = source("Vessel.svelte");
    expect(vessel).toContain("${t(vessel.words)}");
    expect(vessel).toContain('class="gel-status"');
    expect(vessel).toContain('class="persistent-readout"');
    expect(vessel).toContain('aria-label={t("{family} enzyme model: {percent}% of {substrate} converted in {material}"');
    expect(vessel).toContain('class="observation-status" role="status" aria-live="polite" aria-atomic="true"');
  });

  it("keeps additive gel, coating, and enzyme fields safe with old scene payloads", () => {
    const vessel = source("Vessel.svelte");
    expect(vessel).toContain("vessel.coatings ?? []");
    expect(vessel).toContain("{#if vessel.gel}");
    expect(vessel).toContain("enzymeReadouts(vessel)");
  });

  it("provides persistent coating and gel descriptions without event effects", () => {
    const vessel = source("Vessel.svelte");
    expect(vessel).toContain("<title>{t(coating.words)}</title>");
    expect(vessel).toContain('t("translucent cohesive gel")');
    expect(vessel).toContain("vessel.gel.gelled_fraction");
  });
});
