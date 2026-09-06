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

  // One catalogue, one card, one rail. Both promises now read the same
  // file and the same card, and they are kept separate because they still
  // regress independently: the rail's accessible pressed state and the
  // card's linked-learning report are different pieces of wiring.
  it("exposes named, pressed-state filters in one rail", () => {
    const catalog = source("Catalog.svelte");
    expect(catalog).toContain('role="group" aria-label={t("completion status")}');
    expect(catalog).toContain("aria-pressed={filters.progress === value}");
    expect(catalog).toContain('role="group" aria-label={t("level")}');
    expect(catalog).toContain("aria-pressed={filters.level === level}");
    expect(catalog).toContain("aria-pressed={filters.duration === band}");
    expect(catalog).toContain("aria-pressed={filters.shelfOnly}");
    // One row: the search box and the scrolling rail, never a second line
    // of chips pushing the experiments below the fold.
    expect(catalog).toContain('class="filter-row"');
    expect(catalog).toContain('class="filter-rail"');
  });

  it("keeps linked-learning progress and Continue/Replay actions on the card", () => {
    const catalog = source("Catalog.svelte");
    expect(catalog).toContain("data-progress={links.progress}");
    expect(catalog).toContain("{links.completedLearning}/{links.linkedLearning}");
    expect(catalog).toContain("runTargetLabel(item.run, item.done)");
    expect(catalog).toContain("codexLearningLabel(links.codexCompleted.includes(id))");
    // Every card carries the same completion word, whichever corpus it came
    // from — the headless gate reads exactly this.
    expect(catalog).toContain('class="completion"');
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
