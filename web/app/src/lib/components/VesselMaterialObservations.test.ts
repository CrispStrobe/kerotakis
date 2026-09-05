import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

describe("persistent material observations", () => {
  const source = readFileSync(new URL("./Vessel.svelte", import.meta.url), "utf8");

  it("draws instant snow from scene swelling values", () => {
    expect(source).toContain("vessel.swelling.swelling_ratio_g_per_g");
    expect(source).toContain('class="swollen-snow"');
    expect(source).toContain("vessel.swelling.retained_water_g.toFixed(1)");
  });

  it("draws blue light from persistent relative intensity", () => {
    expect(source).toContain("vessel.chemiluminescence?.relative_intensity");
    expect(source).toContain('class="computed-glow"');
    expect(source).toContain('stop-color="#5de8ff"');
  });

  it("keeps decorative light out of the accessibility tree", () => {
    expect(source).toContain('class="computed-glow" aria-hidden="true"');
  });

  it("draws a persistent translucent gel from scene state", () => {
    expect(source).toContain("vessel.gel.gelled_fraction");
    expect(source).toContain('class="gel-body"');
    expect(source).toContain('class="gel-status"');
    expect(source).toContain('t("translucent cohesive gel")');
    expect(source).toContain('t("of polymer gelled")');
  });

  it("draws only persistent source-backed protective coatings", () => {
    expect(source).toContain("vessel.coatings ?? []");
    expect(source).toContain("coating.recipe_id === object.recipe_id");
    expect(source).toContain('class="persistent-coating"');
    expect(source).toContain("t(coating.words)");
  });
});
