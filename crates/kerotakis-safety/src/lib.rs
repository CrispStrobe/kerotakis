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
//! The matrix is the seed of our own reimplementation of the reactive-group
//! compatibility methodology published by NOAA's Office of Response and
//! Restoration (see PLAN.md for sourcing and the legal position). Every
//! entry is our own encoding of published, non-copyrightable reactivity
//! facts.

use kerotakis_core::{SafetyScreen, SafetyVerdict, Severity, Vessel};

/// Seed reactive groups. Grows toward the full published group set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReactiveGroup {
    AcidStrong,
    BaseStrong,
    OxidizerHypochlorite,
    AmmoniaAmines,
}

/// Registry-key → reactive groups. Until SMARTS-based assignment lands
/// (Indigo, P1 completion), assignment is by curated table over the seed
/// registry.
pub fn groups(species_key: &str) -> &'static [ReactiveGroup] {
    match species_key {
        "HCl" => &[ReactiveGroup::AcidStrong],
        "NaOH" => &[ReactiveGroup::BaseStrong],
        "NaOCl" => &[ReactiveGroup::OxidizerHypochlorite],
        "NH3" => &[ReactiveGroup::AmmoniaAmines],
        _ => &[],
    }
}

struct Incompatibility {
    a: ReactiveGroup,
    b: ReactiveGroup,
    severity: Severity,
    hazard: &'static str,
    real_world: &'static str,
}

/// The seed incompatibility matrix. Symmetric; checked both ways.
const INCOMPATIBLE: &[Incompatibility] = &[
    Incompatibility {
        a: ReactiveGroup::OxidizerHypochlorite,
        b: ReactiveGroup::AmmoniaAmines,
        severity: Severity::Danger,
        hazard: "mixing bleach with ammonia makes chloramine, a toxic gas",
        real_world: "People are hospitalised every year from mixing these two household cleaners.",
    },
    Incompatibility {
        a: ReactiveGroup::OxidizerHypochlorite,
        b: ReactiveGroup::AcidStrong,
        severity: Severity::Danger,
        hazard: "mixing bleach with acid releases chlorine, a toxic gas",
        real_world: "Chlorine gas was used as a chemical weapon; even small amounts damage lungs.",
    },
];

/// The L0 screen: warns (strongly, precisely) on states whose species carry
/// incompatible reactive groups; the simulation then shows what happens.
pub struct ReactiveGroupScreen;

impl SafetyScreen for ReactiveGroupScreen {
    fn assess(&self, vessel: &Vessel) -> SafetyVerdict {
        let present: Vec<ReactiveGroup> = vessel
            .contents
            .iter()
            .flat_map(|p| groups(p.species.0.as_str()).iter().copied())
            .collect();
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
        // Neutralisation is chemistry, not a hazard.
        let v = vessel_with(&["water", "HCl", "NaOH"]);
        assert_eq!(ReactiveGroupScreen.assess(&v), SafetyVerdict::Allow);
    }

    #[test]
    fn benign_mixtures_pass() {
        let v = vessel_with(&["water", "NaCl", "AgNO3"]);
        assert_eq!(ReactiveGroupScreen.assess(&v), SafetyVerdict::Allow);
    }
}
