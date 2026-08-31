import { describe, expect, it } from "vitest";
import { isNetIonic, latestNetIonic, spectatorPhrase, type NetIonic } from "./ionic";

const silverChloride: NetIonic = {
  vessel: 0,
  basis: "precipitation",
  reactants: [
    { species: "Ag+", label: "Ag⁺", coefficient: 1, charge: 1, phase: "aqueous" },
    { species: "Cl-", label: "Cl⁻", coefficient: 1, charge: -1, phase: "aqueous" },
  ],
  products: [
    { species: "AgCl", label: "AgCl", coefficient: 1, charge: 0, phase: "solid" },
  ],
  spectators: [
    { species: "Na+", label: "Na⁺", coefficient: 1, charge: 1, phase: "aqueous" },
    { species: "NO3-", label: "NO₃⁻", coefficient: 1, charge: -1, phase: "aqueous" },
  ],
  equation: "Ag⁺(aq) + Cl⁻(aq) → AgCl(s)",
  provenance: "PHREEQC (IPhreeqc) · wateq4f.dat",
};

describe("the ionic contract", () => {
  it("accepts the engine's shape and refuses anything short of it", () => {
    expect(isNetIonic(silverChloride)).toBe(true);
    expect(isNetIonic(null)).toBe(false);
    expect(isNetIonic({ ...silverChloride, equation: "" })).toBe(false);
    // A basis the shell does not know is not one it may render: the tag
    // is what tells a reader why the equation is trustworthy.
    expect(isNetIonic({ ...silverChloride, basis: "vibes" })).toBe(false);
    expect(isNetIonic({ ...silverChloride, reactants: undefined })).toBe(false);
  });

  it("pins the last well-formed entry across a batch of steps", () => {
    const water: NetIonic = {
      vessel: 0,
      basis: "neutralisation",
      reactants: [],
      products: [],
      spectators: [],
      equation: "H⁺(aq) + OH⁻(aq) → H₂O(l)",
    };
    const steps = [
      { ionic: [silverChloride] },
      { rendered: [] },
      { ionic: [water] },
    ];
    expect(latestNetIonic(steps)?.equation).toBe("H⁺(aq) + OH⁻(aq) → H₂O(l)");
  });

  it("is null where the engine derived nothing, and skips malformed entries", () => {
    expect(latestNetIonic([{}, { ionic: [] }])).toBeNull();
    expect(latestNetIonic([{ ionic: [{ equation: 42 }] }])).toBeNull();
    expect(latestNetIonic([{ ionic: "not an array" }])).toBeNull();
  });

  it("names the spectators, and says nothing when there were none", () => {
    expect(spectatorPhrase(silverChloride)).toBe("Na⁺, NO₃⁻");
    expect(spectatorPhrase({ ...silverChloride, spectators: [] })).toBeNull();
  });
});
