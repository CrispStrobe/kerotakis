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

  it("draws adsorption only from the standing scene ledger", () => {
    expect(source).toContain("adsorptionReadouts(vessel.adsorption)");
    expect(source).toContain('class="adsorption-marker"');
    expect(source).toContain("data-loading-fraction={adsorption.loadingFraction.toFixed(4)}");
    expect(source).toContain('class="persistent-readout adsorption-readout"');
    expect(source).toContain("progress.still_dissolved_mg.toFixed(1)");
    expect(source).toContain("progress.boundary");
    expect(source).not.toContain("Adsorbed");
  });

  it("exposes engine-owned liquid and solid quantities to semantic DOM goldens", () => {
    expect(source).toContain('class="scene-liquid-layer"');
    expect(source).toContain("data-volume-l={layer.volume_l.toFixed(6)}");
    expect(source).toContain('class="scene-solid"');
    expect(source).toContain("data-moles={solid.moles.toFixed(6)}");
  });

  it("draws a partition from standing scene equilibrium, not an event", () => {
    expect(source).toContain("partitionReadouts(vessel.partition)");
    expect(source).toContain('class="partition-marker"');
    expect(source).toContain("22 * split.fractionLower");
    expect(source).toContain('class="persistent-readout partition-readout"');
    expect(source).toContain("split.boundary");
    expect(source).not.toContain("Partitioned");
  });
});
