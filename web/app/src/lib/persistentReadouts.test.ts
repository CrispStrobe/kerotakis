import { describe, expect, it } from "vitest";
import { enzymeReadouts } from "./persistentReadouts";

describe("persistent vessel readouts", () => {
  it("is absent for an older scene-v1 vessel", () => {
    expect(enzymeReadouts({})).toEqual([]);
  });

  it("formats and bounds the serialized conversion fraction", () => {
    expect(enzymeReadouts({ enzyme_hydrolysis: [{
      family: "lactase", material: "whole milk", substrate: "lactose in milk", converted_fraction: 0.625,
    }] })).toEqual([{ family: "lactase", material: "whole milk", substrate: "lactose in milk", percent: 63 }]);
    expect(enzymeReadouts({ enzyme_hydrolysis: [{
      family: "protease", material: "gelatine", substrate: "gelatine protein", converted_fraction: 1.2,
    }] })[0]?.percent).toBe(100);
  });
});
