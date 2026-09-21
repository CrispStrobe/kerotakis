import { describe, expect, it } from "vitest";
import {
  completeIonic,
  isNetIonic,
  latestNetIonic,
  spectatorPhrase,
  writtenTerm,
  type NetIonic,
} from "./ionic";

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

/**
 * The complete ionic equation (GUI-092, second slice).
 *
 * Barium chloride met by sodium sulfate, because a suite that only ever
 * tests 1:1 salts passes on the bug this feature fixes: the engine used to
 * hand every spectator `coefficient: 1` unconditionally, and against a
 * 1:1 salt that number happens to be right.
 */
const bariumSulfate: NetIonic = {
  vessel: 0,
  basis: "precipitation",
  reactants: [
    { species: "Ba+2", label: "Ba²⁺", coefficient: 1, charge: 2, phase: "aqueous" },
    { species: "SO4-2", label: "SO₄²⁻", coefficient: 1, charge: -2, phase: "aqueous" },
  ],
  products: [
    { species: "BaSO4", label: "BaSO₄", coefficient: 1, charge: 0, phase: "solid" },
  ],
  spectators: [
    { species: "Na+", label: "Na⁺", coefficient: 1, charge: 1, phase: "aqueous" },
    { species: "Cl-", label: "Cl⁻", coefficient: 1, charge: -1, phase: "aqueous" },
  ],
  equation: "Ba²⁺(aq) + SO₄²⁻(aq) → BaSO₄(s)",
  complete: {
    reactants: [
      { species: "Ba+2", label: "Ba²⁺", coefficient: 1, charge: 2, phase: "aqueous" },
      { species: "SO4-2", label: "SO₄²⁻", coefficient: 1, charge: -2, phase: "aqueous" },
      {
        species: "Na+", label: "Na⁺", coefficient: 2, charge: 1,
        phase: "aqueous", spectator: true,
      },
      {
        species: "Cl-", label: "Cl⁻", coefficient: 2, charge: -1,
        phase: "aqueous", spectator: true,
      },
    ],
    products: [
      { species: "BaSO4", label: "BaSO₄", coefficient: 1, charge: 0, phase: "solid" },
      {
        species: "Na+", label: "Na⁺", coefficient: 2, charge: 1,
        phase: "aqueous", spectator: true,
      },
      {
        species: "Cl-", label: "Cl⁻", coefficient: 2, charge: -1,
        phase: "aqueous", spectator: true,
      },
    ],
    equation:
      "Ba²⁺(aq) + SO₄²⁻(aq) + 2 Na⁺(aq) + 2 Cl⁻(aq) → " +
      "BaSO₄(s) + 2 Na⁺(aq) + 2 Cl⁻(aq)",
  },
};

describe("the complete ionic equation", () => {
  it("passes the engine's line through, coefficients and flags intact", () => {
    const complete = completeIonic(bariumSulfate);
    expect(complete).not.toBeNull();
    // Two of each, and neither number was worked out here.
    expect(
      complete?.reactants.filter((t) => t.spectator).map((t) => t.coefficient),
    ).toEqual([2, 2]);
    expect(complete?.equation).toContain("2 Na⁺(aq)");
  });

  it("is null where the engine drew none, and the shell writes no replacement", () => {
    // The refusal case, seen from the shell. `silverChloride` has
    // spectators and no `complete` — which is exactly the state an engine
    // reaches when the counter-ion is ambiguous — and the answer here must
    // be null rather than an equation assembled out of that list, whose
    // coefficients are selected by abundance and mean nothing.
    expect(silverChloride.spectators).toHaveLength(2);
    expect(completeIonic(silverChloride)).toBeNull();
  });

  it("refuses a side that carries net charge, however well-formed", () => {
    // Each side of a complete ionic equation is a statement about bottles,
    // and a bottle does not carry a charge. A line that fails this is a
    // line the engine should not have emitted, and repairing it here would
    // be the shell assembling chemistry.
    const lopsided: NetIonic = {
      ...bariumSulfate,
      complete: {
        ...bariumSulfate.complete!,
        reactants: bariumSulfate.complete!.reactants.filter(
          (t) => t.species !== "Cl-",
        ),
      },
    };
    expect(completeIonic(lopsided)).toBeNull();
  });

  it("refuses a complete equation with nothing struck through", () => {
    // Then it is the net equation with extra characters, not a second line.
    const nothingCancels: NetIonic = {
      ...bariumSulfate,
      complete: {
        ...bariumSulfate.complete!,
        reactants: bariumSulfate.complete!.reactants.map((t) => ({
          ...t,
          spectator: false,
        })),
      },
    };
    expect(completeIonic(nothingCancels)).toBeNull();
  });

  it("writes a term the way it is read", () => {
    expect(
      writtenTerm({
        species: "OH-", label: "OH⁻", coefficient: 2, charge: -1, phase: "aqueous",
      }),
    ).toBe("2 OH⁻(aq)");
    // A coefficient of one is not written, which is how chemistry writes it.
    expect(
      writtenTerm({
        species: "AgCl", label: "AgCl", coefficient: 1, charge: 0, phase: "solid",
      }),
    ).toBe("AgCl(s)");
    expect(
      writtenTerm({
        species: "H2O", label: "H₂O", coefficient: 1, charge: 0, phase: "liquid",
      }),
    ).toBe("H₂O(l)");
  });
});
