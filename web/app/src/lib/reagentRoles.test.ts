/**
 * GUI-093. The point of these is not that "NaOH is a base" — it is that
 * nowhere in `reagentRoles.ts` does the string "NaOH" appear. Each case
 * below feeds the exact fields the engine bridge sends for that species
 * (reactive-group rows from `kerotakis_safety::groups`, element counts and
 * charge from `kerotakis_core::stoich::parse_formula` over the registry
 * formula) and asserts the role that falls out of them, so a rule that
 * starts guessing shows up here rather than on the shelf.
 */
import { describe, expect, it } from "vitest";
import {
  REAGENT_ROLES,
  ROLE_LABELS,
  deriveShelfRoles,
  isMetalSymbol,
  rolesForSpecies,
  type RoleInput,
} from "./reagentRoles";

/** A shelf item as the bridge sends it; only the derivation inputs matter. */
const item = (
  key: string,
  formula: string,
  extra: Partial<RoleInput> = {},
): RoleInput => ({
  key,
  formula,
  hazards: [],
  reactive_groups: [],
  charge: 0,
  indicator: false,
  solvent: false,
  ...extra,
});

describe("roles from the safety matrix", () => {
  it("splits acid from base, which the hazard labels cannot", () => {
    // Both render "corrosive" in `hazards`; the unflattened rows do not.
    const acid = item("HCl", "HCl", {
      reactive_groups: ["acid_strong"],
      hazards: ["corrosive"],
      elements: { H: 1, Cl: 1 },
    });
    const base = item("NaOH", "NaOH", {
      reactive_groups: ["base_strong"],
      hazards: ["corrosive"],
      elements: { Na: 1, O: 1, H: 1 },
    });
    expect(rolesForSpecies(acid)).toContain("acid");
    expect(rolesForSpecies(acid)).not.toContain("base");
    expect(rolesForSpecies(base)).toContain("base");
    expect(rolesForSpecies(base)).not.toContain("acid");
  });

  it("reads the redox rows and the ammonia row", () => {
    expect(
      rolesForSpecies(item("KMnO4", "KMnO4", {
        reactive_groups: ["oxidizer_strong"],
        elements: { K: 1, Mn: 1, O: 4 },
      })),
    ).toEqual(["salt", "oxidiser"]);
    expect(
      rolesForSpecies(item("Na2S2O3", "Na2S2O3", {
        reactive_groups: ["reducing_agent"],
        elements: { Na: 2, S: 2, O: 3 },
      })),
    ).toEqual(["salt", "reducer"]);
    // Ammonia is the matrix's weak base, and reads as one.
    expect(
      rolesForSpecies(item("NH3", "NH3(aq)", {
        reactive_groups: ["ammonia_amines"],
        elements: { N: 1, H: 3 },
      })),
    ).toEqual(["base"]);
  });

  it("falls back to the flattened labels when an older bridge sends no rows", () => {
    const old: RoleInput = {
      key: "KMnO4",
      formula: "KMnO4",
      hazards: ["oxidiser"],
      // reactive_groups, elements and charge all absent.
    };
    expect(rolesForSpecies(old)).toEqual(["oxidiser"]);
    // "corrosive" is deliberately not mapped: it cannot say which of the
    // two roles it means, and a coin-flip is worse than the phase filter.
    expect(rolesForSpecies({ key: "HCl", formula: "HCl", hazards: ["corrosive"] }))
      .toEqual(["unsorted"]);
  });
});

describe("roles from composition", () => {
  it("calls a hydroxide a base with no safety row behind it", () => {
    // Copper(II) hydroxide has no NOAA reactive group at all — the
    // composition is the whole reason it is filed under bases.
    expect(
      rolesForSpecies(item("Cu(OH)2", "CuH2O2", { elements: { Cu: 1, H: 2, O: 2 } })),
    ).toEqual(["base"]);
    expect(
      rolesForSpecies(item("Fe(OH)3", "FeH3O3", { elements: { Fe: 1, H: 3, O: 3 } })),
    ).toEqual(["base"]);
  });

  it("separates oxides from salts", () => {
    expect(rolesForSpecies(item("Fe2O3", "Fe2O3", { elements: { Fe: 2, O: 3 } })))
      .toEqual(["oxide"]);
    expect(rolesForSpecies(item("SiO2", "SiO2", { elements: { Si: 1, O: 2 } })))
      .toEqual(["oxide"]);
    expect(rolesForSpecies(item("CuSO4", "CuSO4", { elements: { Cu: 1, S: 1, O: 4 } })))
      .toEqual(["salt"]);
    // An ammonium salt has no metal in it and is a salt regardless.
    expect(rolesForSpecies(item("NH4Cl", "NH4Cl", { elements: { N: 1, H: 4, Cl: 1 } })))
      .toEqual(["salt"]);
  });

  it("reads the H+ donor the way a chemist writes one", () => {
    expect(rolesForSpecies(item("H3PO4", "H3PO4", { elements: { H: 3, P: 1, O: 4 } })))
      .toEqual(["acid"]);
    // Acetic acid leads with carbon and still closes a carboxyl group.
    expect(
      rolesForSpecies(item("CH3COOH", "CH3COOH", { elements: { C: 2, H: 4, O: 2 } })),
    ).toEqual(["acid", "organic"]);
    // Water and hydrogen peroxide both lead with H and are not acids:
    // nothing beyond hydrogen and oxygen is in either of them.
    expect(
      rolesForSpecies(item("water", "H2O", { elements: { H: 2, O: 1 }, solvent: true })),
    ).toEqual(["solvent"]);
    expect(
      rolesForSpecies(item("H2O2", "H2O2", {
        reactive_groups: ["oxidizer_strong"],
        elements: { H: 2, O: 2 },
      })),
    ).toEqual(["oxidiser"]);
  });

  it("keeps carbonates out of the organics", () => {
    // Both carry carbon and hydrogen; only one has a C–H skeleton.
    expect(
      rolesForSpecies(item("NaHCO3", "NaHCO3", {
        reactive_groups: ["carbonate"],
        elements: { Na: 1, H: 1, C: 1, O: 3 },
      })),
    ).toEqual(["salt"]);
    expect(rolesForSpecies(item("methanol", "CH3OH", { elements: { C: 1, H: 4, O: 1 } })))
      .toEqual(["organic"]);
    // A hydrate parses to the sum of its parts and stays a salt.
    expect(
      rolesForSpecies(item("epsomite", "MgSO4·7H2O", {
        elements: { Mg: 1, S: 1, O: 11, H: 14 },
      })),
    ).toEqual(["salt"]);
  });

  it("distinguishes the metal from its dissolved ion", () => {
    expect(rolesForSpecies(item("Cu", "Cu", { elements: { Cu: 1 } }))).toEqual(["metal"]);
    expect(
      rolesForSpecies(item("Cu+2", "Cu+2", { elements: { Cu: 1 }, charge: 2 })),
    ).toEqual(["ion"]);
    expect(
      rolesForSpecies(item("MnO4-", "MnO4-", {
        reactive_groups: ["oxidizer_strong"],
        elements: { Mn: 1, O: 4 },
        charge: -1,
      })),
    ).toEqual(["oxidiser", "ion"]);
    // An elemental non-metal is honestly none of these roles.
    expect(rolesForSpecies(item("S", "S", { elements: { S: 1 } }))).toEqual(["unsorted"]);
  });

  it("takes metals from the periodic table rather than a list here", () => {
    expect(isMetalSymbol("Fe")).toBe(true);
    expect(isMetalSymbol("Al")).toBe(true);
    expect(isMetalSymbol("Si")).toBe(false);
    expect(isMetalSymbol("C")).toBe(false);
  });
});

describe("the honest gap", () => {
  it("says unsorted rather than guessing, and only when nothing derived", () => {
    // Chloramine: no reactive-group row, no metal, no carbon skeleton.
    const chloramine = item("NH2Cl", "NH2Cl", { elements: { N: 1, H: 2, Cl: 1 } });
    expect(rolesForSpecies(chloramine)).toEqual(["unsorted"]);
    // Enzyme identity comes from the engine, not its stand-in formula.
    expect(rolesForSpecies(item("catalase", "C", {
      elements: { C: 1 }, enzyme_family: "catalase",
    }))).toEqual(["enzyme"]);
    // A species with no composition at all is unsorted, not mis-sorted.
    expect(rolesForSpecies({ key: "mystery", formula: "???" })).toEqual(["unsorted"]);
  });

  it("never pairs unsorted with a real role", () => {
    const roles = rolesForSpecies(item("NaCl", "NaCl", { elements: { Na: 1, Cl: 1 } }));
    expect(roles).toEqual(["salt"]);
    expect(roles).not.toContain("unsorted");
  });

  it("exposes engine-declared protein materials", () => {
    expect(rolesForSpecies(item("egg_white", "H2O", {
      elements: { H: 2, O: 1 }, protein: true,
    }))).toEqual(["protein"]);
  });

  it("labels and orders every role it can return", () => {
    for (const role of REAGENT_ROLES) expect(ROLE_LABELS[role]).toBeTruthy();
    // Roles come back in REAGENT_ROLES order so the chips never reshuffle.
    const many = rolesForSpecies(item("FeCl3", "FeCl3", {
      reactive_groups: ["acidic_salt"],
      elements: { Fe: 1, Cl: 3 },
    }));
    expect(many).toEqual(["acid", "salt"]);
  });
});

describe("materials take the roles of what is in them", () => {
  const shelf: RoleInput[] = [
    item("water", "H2O", { elements: { H: 2, O: 1 }, solvent: true }),
    item("Fe", "Fe", { elements: { Fe: 1 }, reactive_groups: ["active_metal"] }),
    item("NaHCO3", "NaHCO3", {
      reactive_groups: ["carbonate"],
      elements: { Na: 1, H: 1, C: 1, O: 3 },
    }),
    item("starch", "C6H10O5", { elements: { C: 6, H: 10, O: 5 } }),
    item("catalase", "C", { elements: { C: 1 } }),
    { key: "iron_filings", formula: "Fe", components: ["Fe"] },
    { key: "baking_powder", formula: "NaHCO3 + C6H10O5", components: ["NaHCO3", "starch"] },
    { key: "dish_soap", formula: "H2O", components: ["water"] },
    { key: "dry_yeast", formula: "C", components: ["catalase"] },
    { key: "ground_black_pepper", formula: "", components: [] },
  ];
  const roles = deriveShelfRoles(shelf);

  it("classifies a mixture by its solutes", () => {
    expect(roles.get("iron_filings")).toEqual(["metal"]);
    expect(roles.get("baking_powder")).toEqual(["salt", "organic"]);
  });

  it("does not let the solvent decide what the bottle is for", () => {
    // Dish soap's only modelled component is water. Calling it a solvent
    // would be a category error dressed up as an answer.
    expect(roles.get("dish_soap")).toEqual(["unsorted"]);
    expect(roles.get("ground_black_pepper")).toEqual(["unsorted"]);
  });

  it("inherits the gap when its components are the gap", () => {
    expect(roles.get("dry_yeast")).toEqual(["unsorted"]);
  });

  it("still classifies the pure species alongside them", () => {
    expect(roles.get("water")).toEqual(["solvent"]);
    expect(roles.get("Fe")).toEqual(["metal"]);
    expect(roles.size).toBe(shelf.length);
  });
});
