import { describe, expect, it } from "vitest";
import { partitionReadouts } from "./partitionReadouts";

describe("persistent partition readouts", () => {
  it("is absent for older scene-v1 payloads", () => {
    expect(partitionReadouts()).toEqual([]);
  });

  it("keeps complementary lower and upper shares", () => {
    const row = partitionReadouts([{
      species: "ethanol", lower_solvent: "water", upper_solvent: "hexane",
      total_moles: 0.2, lower_moles: 0.168, upper_moles: 0.032,
      fraction_lower: 0.84, boundary: "instant equilibrium, no rate",
      provenance: "UNIFAC 1975",
    }])[0]!;
    expect(row).toMatchObject({ lowerPercent: 84, upperPercent: 16 });
    expect(row.fractionLower + row.fractionUpper).toBe(1);
  });

  it("contains a malformed fraction at the display boundary", () => {
    const base = {
      species: "x", lower_solvent: "water", upper_solvent: "hexane",
      total_moles: 1, lower_moles: 1, upper_moles: 0,
      boundary: "bounded", provenance: "source",
    };
    expect(partitionReadouts([{ ...base, fraction_lower: 2 }])[0]!.lowerPercent).toBe(100);
    expect(partitionReadouts([{ ...base, fraction_lower: Number.NaN }])[0]!.lowerPercent).toBe(0);
  });
});
