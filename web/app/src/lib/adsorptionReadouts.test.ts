import { describe, expect, it } from "vitest";
import { adsorptionReadouts } from "./adsorptionReadouts";

describe("persistent adsorption readouts", () => {
  it("is absent for an older scene-v1 vessel", () => {
    expect(adsorptionReadouts()).toEqual([]);
  });

  it("keeps both sides of the engine-owned split", () => {
    expect(adsorptionReadouts([{
      sorbent: "activated_charcoal",
      sorbate: "methyl_orange",
      held_mg: 190.25,
      still_dissolved_mg: 136.75,
      held_fraction: 190.25 / 327,
      loading_mg_per_g: 190.25,
      loading_fraction: 0.95125,
      boundary: "equilibrium, not rate",
      provenance: "parameters pending review",
    }])[0]).toMatchObject({
      heldPercent: 58,
      held_mg: 190.25,
      still_dissolved_mg: 136.75,
      loadingFraction: 0.95125,
    });
  });

  it("contains malformed optional numbers without inventing chemistry", () => {
    const row = adsorptionReadouts([{
      sorbent: "carbon", sorbate: "dye", held_mg: Number.NaN,
      still_dissolved_mg: -2, held_fraction: 4, loading_fraction: Number.NaN,
      boundary: "bounded", provenance: "pending",
    }])[0]!;
    expect(row).toMatchObject({ held_mg: 0, still_dissolved_mg: 0, heldPercent: 100, loadingFraction: 0 });
  });
});
