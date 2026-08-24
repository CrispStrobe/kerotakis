//! # kerotakis-safety
//!
//! The L0 reactivity screen: runs before any chemistry, on the *prospective*
//! vessel state (PLAN.md, L0).
//!
//! This is a pedagogical tool, so known hazards produce a **strong warning
//! and then proceed** — being precise about what would happen is the lesson,
//! and the virtual lab is the one place it can be watched safely. The actual
//! outcome (chloramine gas, chlorine gas) is computed by the curated
//! reaction layer; this screen's job is to make sure the warning always
//! comes first. `Veto` is reserved for the product-safety boundary.
//!
//! The matrix is our own reimplementation of the reactive-group
//! compatibility methodology published by NOAA's Office of Response and
//! Restoration (see PLAN.md for sourcing and the legal position). Every
//! entry is our own encoding of published, non-copyrightable reactivity
//! facts.
//!
//! CAP-11: every registry species has an explicit reactive-group row
//! (totality enforced by CI); the incompatibility matrix covers
//! the groups those species populate.

use kerotakis_core::{SafetyScreen, SafetyVerdict, Severity, Vessel};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReactiveGroup {
    AcidStrong,
    BaseStrong,
    OxidizerStrong,
    OxidizerHypochlorite,
    ReducingAgent,
    ActiveMetal,
    FlammableLiquid,
    FlammableGas,
    WaterReactive,
    AmmoniaAmines,
    Carbonate,
}

pub fn hazard_labels(species_key: &str) -> Vec<&'static str> {
    groups(species_key)
        .iter()
        .map(|g| match g {
            ReactiveGroup::AcidStrong => "corrosive",
            ReactiveGroup::BaseStrong => "corrosive",
            ReactiveGroup::OxidizerStrong => "oxidiser",
            ReactiveGroup::OxidizerHypochlorite => "oxidiser",
            ReactiveGroup::ReducingAgent => "reducing_agent",
            ReactiveGroup::ActiveMetal => "flammable_solid",
            ReactiveGroup::FlammableLiquid => "flammable",
            ReactiveGroup::FlammableGas => "flammable",
            ReactiveGroup::WaterReactive => "water_reactive",
            ReactiveGroup::AmmoniaAmines => "toxic",
            ReactiveGroup::Carbonate => "irritant",
        })
        .collect()
}

/// Registry-key → reactive groups. Assignment is total over the registry;
/// the CI totality test (`tests/totality.rs`) fails if a species is added
/// without a row here. Inert species (solvents, spectator ions, salts,
/// indicators) get `&[]`.
///
/// Provenance: each assignment follows from the species' chemical identity
/// and the published NOAA reactive-group definitions. Acids dissociate
/// completely ⇒ AcidStrong; metals above hydrogen in the activity series
/// ⇒ ActiveMetal; etc. No database exports are used.
pub fn groups(species_key: &str) -> &'static [ReactiveGroup] {
    use ReactiveGroup::*;
    match species_key {
        // ── strong acids ──────────────────────────────────────────
        "HCl" => &[AcidStrong],
        "H2SO4" => &[AcidStrong],

        // ── strong bases ──────────────────────────────────────────
        "NaOH" => &[BaseStrong],
        "KOH" => &[BaseStrong],
        "OH-" => &[BaseStrong],
        "Ca(OH)2" => &[BaseStrong],

        // ── strong oxidizers ──────────────────────────────────────
        "H2O2" => &[OxidizerStrong],
        "KMnO4" => &[OxidizerStrong],
        "Cl2" => &[OxidizerStrong],
        "MnO4-" => &[OxidizerStrong],

        // ── hypochlorite (specific sub-class of oxidizer) ─────────
        "NaOCl" => &[OxidizerHypochlorite],

        // ── reducing agents ───────────────────────────────────────
        "Na2SO3" => &[ReducingAgent],
        "Na2S2O3" => &[ReducingAgent],

        // ── active metals (above H in activity series) ────────────
        "Al" => &[ActiveMetal],
        "Mg" => &[ActiveMetal],
        "Zn" => &[ActiveMetal],
        "Fe" => &[ActiveMetal],
        "Pb" => &[ActiveMetal],

        // ── flammable liquids ─────────────────────────────────────
        "ethanol" => &[FlammableLiquid],
        "methanol" => &[FlammableLiquid],
        "hexane" => &[FlammableLiquid],
        "propanone" => &[FlammableLiquid],
        "ethyl_acetate" => &[FlammableLiquid],

        // ── flammable gas ─────────────────────────────────────────
        "H2" => &[FlammableGas],

        // ── water-reactive ────────────────────────────────────────
        "CaO" => &[WaterReactive],

        // ── ammonia / amines ──────────────────────────────────────
        "NH3" => &[AmmoniaAmines],

        // ── carbonates (vigorous CO₂ release with strong acid) ────
        "CaCO3" => &[Carbonate],
        "Na2CO3" => &[Carbonate],
        "NaHCO3" => &[Carbonate],

        // ── inert: solvents, salts, oxides, ions, indicators ──────
        "water" | "NaCl" | "AgNO3" | "AgCl" | "catalase" | "MnO2" | "S" | "SO2" | "Cu(OH)2"
        | "CuO" | "Na+" | "Cl-" | "Ag+" | "NO3-" | "NH2Cl" | "CH3COOH" | "NaOAc" | "CH3COO-"
        | "CO2" | "HCO3-" | "H3PO4" | "H2PO4-" | "KCl" | "CaCl2" | "MgSO4" | "gypsum" | "K+"
        | "Ca+2" | "Mg+2" | "Sr+2" | "SO4-2" | "Cu" | "Ag" | "MgO" | "C" | "O2" | "N2"
        | "CuSO4" | "Cu+2" | "FeSO4" | "Fe+2" | "Fe+3" | "Cu+1" | "Mn+2" | "MnO4-2" | "Mn+3"
        | "phenolphthalein" | "methyl_orange" | "bromothymol_blue" | "Zn+2" | "ZnSO4" | "Pb+2"
        | "Pb(NO3)2" | "PE" | "PP" | "PET" | "PS" | "betanin" | "betanin_ox"
        | "curcumin" | "curcumin_ox" | "indigo_carmine" | "indigo_carmine_ox" => &[],

        _ => &[],
    }
}

/// Every species key covered by `groups()`. The totality test checks
/// this set against `species::registry()` — a new species without an
/// entry here fails CI.
pub const COVERED_KEYS: &[&str] = &[
    "Ag",
    "Ag+",
    "AgCl",
    "Al",
    "betanin",
    "betanin_ox",
    "AgNO3",
    "C",
    "CO2",
    "Ca(OH)2",
    "Ca+2",
    "CaCO3",
    "CaCl2",
    "CaO",
    "CH3COO-",
    "CH3COOH",
    "Cl-",
    "Cl2",
    "Cu",
    "Cu+1",
    "curcumin",
    "curcumin_ox",
    "Cu+2",
    "CuO",
    "Cu(OH)2",
    "CuSO4",
    "Fe",
    "Fe+2",
    "Fe+3",
    "FeSO4",
    "H2",
    "H2O2",
    "H2PO4-",
    "H2SO4",
    "H3PO4",
    "HCl",
    "HCO3-",
    "K+",
    "KCl",
    "KMnO4",
    "Mg",
    "Mg+2",
    "MgO",
    "MgSO4",
    "Mn+2",
    "Mn+3",
    "MnO2",
    "MnO4-",
    "MnO4-2",
    "N2",
    "NH2Cl",
    "NH3",
    "NO3-",
    "Na+",
    "Na2CO3",
    "Na2S2O3",
    "Na2SO3",
    "NaCl",
    "NaHCO3",
    "NaOAc",
    "NaOCl",
    "KOH",
    "NaOH",
    "OH-",
    "O2",
    "Pb",
    "Pb(NO3)2",
    "Pb+2",
    "PE",
    "PET",
    "PP",
    "PS",
    "S",
    "SO2",
    "SO4-2",
    "Sr+2",
    "Zn",
    "Zn+2",
    "ZnSO4",
    "bromothymol_blue",
    "catalase",
    "ethanol",
    "ethyl_acetate",
    "gypsum",
    "hexane",
    "indigo_carmine",
    "indigo_carmine_ox",
    "methanol",
    "methyl_orange",
    "phenolphthalein",
    "propanone",
    "water",
];

struct Incompatibility {
    a: ReactiveGroup,
    b: ReactiveGroup,
    severity: Severity,
    hazard: &'static str,
    real_world: &'static str,
}

/// The incompatibility matrix. Symmetric; checked both ways.
const INCOMPATIBLE: &[Incompatibility] = &[
    // ── toxic-gas producers ───────────────────────────────────────
    Incompatibility {
        a: ReactiveGroup::OxidizerHypochlorite,
        b: ReactiveGroup::AmmoniaAmines,
        severity: Severity::Danger,
        hazard: "mixing bleach with ammonia makes chloramine, a toxic gas",
        real_world: "People are hospitalised every year from mixing \
                     these two household cleaners.",
    },
    Incompatibility {
        a: ReactiveGroup::OxidizerHypochlorite,
        b: ReactiveGroup::AcidStrong,
        severity: Severity::Danger,
        hazard: "mixing bleach with acid releases chlorine, a toxic gas",
        real_world: "Chlorine gas was used as a chemical weapon; even \
                     small amounts damage lungs.",
    },
    // ── fire / explosion risks ────────────────────────────────────
    Incompatibility {
        a: ReactiveGroup::OxidizerStrong,
        b: ReactiveGroup::FlammableLiquid,
        severity: Severity::Danger,
        hazard: "a strong oxidizer mixed with a flammable liquid \
                 can ignite or explode",
        real_world: "Potassium permanganate and glycerol ignite on \
                     contact; similar mixtures cause laboratory fires.",
    },
    Incompatibility {
        a: ReactiveGroup::OxidizerStrong,
        b: ReactiveGroup::FlammableGas,
        severity: Severity::Danger,
        hazard: "a strong oxidizer in the presence of a flammable gas \
                 creates an explosion risk",
        real_world: "Hydrogen and chlorine mixtures can detonate when \
                     exposed to light or a spark.",
    },
    Incompatibility {
        a: ReactiveGroup::OxidizerStrong,
        b: ReactiveGroup::ReducingAgent,
        severity: Severity::Danger,
        hazard: "mixing a strong oxidizer with a reducing agent can \
                 cause a violent, potentially explosive reaction",
        real_world: "Permanganate and sulfite solutions react vigorously; \
                     at scale, such mixtures can detonate.",
    },
    // ── acid + metal → hydrogen gas ───────────────────────────────
    Incompatibility {
        a: ReactiveGroup::AcidStrong,
        b: ReactiveGroup::ActiveMetal,
        severity: Severity::Caution,
        hazard: "strong acid dissolves this metal, releasing hydrogen \
                 gas which is flammable",
        real_world: "Magnesium ribbon in hydrochloric acid produces \
                     enough hydrogen to pop with a lit splint — a \
                     familiar school demo, but the gas is genuinely \
                     flammable.",
    },
    // ── acid + carbonate → CO₂ ────────────────────────────────────
    Incompatibility {
        a: ReactiveGroup::AcidStrong,
        b: ReactiveGroup::Carbonate,
        severity: Severity::Caution,
        hazard: "strong acid and carbonate fizz vigorously, releasing \
                 carbon dioxide — the mixture can spatter",
        real_world: "Adding acid to chalk or baking soda foams over \
                     if the vessel is too small. CO₂ displaces air in \
                     enclosed spaces.",
    },
    // ── water-reactive ────────────────────────────────────────────
    Incompatibility {
        a: ReactiveGroup::WaterReactive,
        b: ReactiveGroup::BaseStrong,
        severity: Severity::Caution,
        hazard: "this water-reactive substance already reacted with \
                 water; combining with a strong base adds further heat",
        real_world: "Quicklime (CaO) generates enough heat on contact \
                     with water to cause burns; adding caustic soda \
                     on top is reckless.",
    },
];

/// The L0 screen: warns (strongly, precisely) on states whose species carry
/// incompatible reactive groups; the simulation then shows what happens.
///
/// Water-reactive species are also checked against the presence of water
/// in the vessel — this is not a group-vs-group rule but a special case.
pub struct ReactiveGroupScreen;

impl SafetyScreen for ReactiveGroupScreen {
    fn assess(&self, vessel: &Vessel) -> SafetyVerdict {
        let present: Vec<ReactiveGroup> = vessel
            .contents
            .iter()
            .flat_map(|p| groups(p.species.0.as_str()).iter().copied())
            .collect();

        // Water-reactive + water is a special case: the hazard is the
        // exothermic slaking, not a group-vs-group incompatibility.
        let has_water = vessel
            .contents
            .iter()
            .any(|p| p.species.0 == "water" && p.moles.0 > 1e-12);
        if has_water && present.contains(&ReactiveGroup::WaterReactive) {
            return SafetyVerdict::Warn {
                severity: Severity::Caution,
                hazard: "this substance reacts violently with water, \
                         releasing a large amount of heat"
                    .to_string(),
                real_world: "Quicklime (CaO) in water can reach 100 °C \
                             and cause severe burns. Always add the solid \
                             to the water slowly, never the reverse."
                    .to_string(),
            };
        }

        for (i, a) in present.iter().enumerate() {
            for b in present.iter().skip(i + 1) {
                for inc in INCOMPATIBLE {
                    if (*a == inc.a && *b == inc.b) || (*a == inc.b && *b == inc.a) {
                        return SafetyVerdict::Warn {
                            severity: inc.severity,
                            hazard: inc.hazard.to_string(),
                            real_world: inc.real_world.to_string(),
                        };
                    }
                }
            }
        }
        SafetyVerdict::Allow
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kerotakis_core::*;

    fn vessel_with(keys: &[&str]) -> Vessel {
        let mut v = Vessel::new(VesselId(0), "test");
        for k in keys {
            v.deposit(SpeciesId::new(k), Moles(0.1), Phase::Liquid);
        }
        v
    }

    // ── existing tests ────────────────────────────────────────────

    #[test]
    fn bleach_and_ammonia_warns_and_proceeds() {
        let v = vessel_with(&["water", "NaOCl", "NH3"]);
        match ReactiveGroupScreen.assess(&v) {
            SafetyVerdict::Warn { hazard, .. } => assert!(hazard.contains("chloramine")),
            other => panic!("expected Warn, got {other:?}"),
        }
    }

    #[test]
    fn bleach_and_acid_warns() {
        let v = vessel_with(&["NaOCl", "HCl"]);
        match ReactiveGroupScreen.assess(&v) {
            SafetyVerdict::Warn { hazard, .. } => assert!(hazard.contains("chlorine")),
            other => panic!("expected Warn, got {other:?}"),
        }
    }

    #[test]
    fn acid_and_base_is_allowed() {
        let v = vessel_with(&["water", "HCl", "NaOH"]);
        assert_eq!(ReactiveGroupScreen.assess(&v), SafetyVerdict::Allow);
    }

    #[test]
    fn benign_mixtures_pass() {
        let v = vessel_with(&["water", "NaCl", "AgNO3"]);
        assert_eq!(ReactiveGroupScreen.assess(&v), SafetyVerdict::Allow);
    }

    // ── CAP-11 new rules ──────────────────────────────────────────

    #[test]
    fn permanganate_and_ethanol_warns() {
        let v = vessel_with(&["KMnO4", "ethanol"]);
        match ReactiveGroupScreen.assess(&v) {
            SafetyVerdict::Warn {
                severity, hazard, ..
            } => {
                assert_eq!(severity, Severity::Danger);
                assert!(hazard.contains("oxidizer"));
            }
            other => panic!("expected Danger, got {other:?}"),
        }
    }

    #[test]
    fn peroxide_and_hexane_warns() {
        let v = vessel_with(&["H2O2", "hexane"]);
        match ReactiveGroupScreen.assess(&v) {
            SafetyVerdict::Warn {
                severity, hazard, ..
            } => {
                assert_eq!(severity, Severity::Danger);
                assert!(hazard.contains("flammable"));
            }
            other => panic!("expected Danger, got {other:?}"),
        }
    }

    #[test]
    fn chlorine_and_hydrogen_warns() {
        let v = vessel_with(&["Cl2", "H2"]);
        match ReactiveGroupScreen.assess(&v) {
            SafetyVerdict::Warn {
                severity, hazard, ..
            } => {
                assert_eq!(severity, Severity::Danger);
                assert!(hazard.contains("explosion"));
            }
            other => panic!("expected Danger, got {other:?}"),
        }
    }

    #[test]
    fn oxidizer_and_reducing_agent_warns() {
        let v = vessel_with(&["KMnO4", "Na2SO3"]);
        match ReactiveGroupScreen.assess(&v) {
            SafetyVerdict::Warn {
                severity, hazard, ..
            } => {
                assert_eq!(severity, Severity::Danger);
                assert!(hazard.contains("violent"));
            }
            other => panic!("expected Danger, got {other:?}"),
        }
    }

    #[test]
    fn acid_and_metal_warns() {
        let v = vessel_with(&["HCl", "Mg"]);
        match ReactiveGroupScreen.assess(&v) {
            SafetyVerdict::Warn {
                severity, hazard, ..
            } => {
                assert_eq!(severity, Severity::Caution);
                assert!(hazard.contains("hydrogen"));
            }
            other => panic!("expected Caution, got {other:?}"),
        }
    }

    #[test]
    fn acid_and_carbonate_warns() {
        let v = vessel_with(&["HCl", "CaCO3"]);
        match ReactiveGroupScreen.assess(&v) {
            SafetyVerdict::Warn {
                severity, hazard, ..
            } => {
                assert_eq!(severity, Severity::Caution);
                assert!(hazard.contains("carbonate"));
            }
            other => panic!("expected Caution, got {other:?}"),
        }
    }

    #[test]
    fn quicklime_and_water_warns() {
        let v = vessel_with(&["CaO", "water"]);
        match ReactiveGroupScreen.assess(&v) {
            SafetyVerdict::Warn {
                severity, hazard, ..
            } => {
                assert_eq!(severity, Severity::Caution);
                assert!(hazard.contains("water"));
            }
            other => panic!("expected Caution, got {other:?}"),
        }
    }

    #[test]
    fn all_inert_species_allow() {
        let v = vessel_with(&["water", "NaCl", "KCl", "CaCl2", "phenolphthalein", "N2"]);
        assert_eq!(ReactiveGroupScreen.assess(&v), SafetyVerdict::Allow);
    }

    #[test]
    fn totality_of_covered_keys() {
        let registry_keys: std::collections::HashSet<&str> = kerotakis_core::species::registry()
            .iter()
            .map(|s| s.key)
            .collect();
        let safety_keys: std::collections::HashSet<&str> = COVERED_KEYS.iter().copied().collect();

        let missing: Vec<&&str> = registry_keys.difference(&safety_keys).collect();
        assert!(
            missing.is_empty(),
            "registry species without safety row: {missing:?}"
        );

        let extra: Vec<&&str> = safety_keys.difference(&registry_keys).collect();
        assert!(
            extra.is_empty(),
            "safety rows for species not in registry: {extra:?}"
        );
    }
}
