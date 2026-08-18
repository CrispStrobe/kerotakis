//! # kerotakis-safety
//!
//! The L0 reactivity screen: runs before any chemistry, on the *prospective*
//! vessel state, and can veto the operation (PLAN.md, L0).
//!
//! This is the seed of our own reimplementation of the reactive-group
//! compatibility methodology published by NOAA's Office of Response and
//! Restoration (open-access CRW papers; see PLAN.md for the exact sourcing
//! and legal position). The matrix below is a hand-verified starter set of
//! textbook incompatibilities — the full group set and SMARTS-driven group
//! assignment arrive with the Indigo integration. Every entry is our own
//! encoding of published, non-copyrightable reactivity facts.

use kerotakis_core::{SafetyScreen, Vessel};

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

/// The seed incompatibility matrix. Symmetric; checked both ways.
/// (Reason strings are shown verbatim to the user by the veto event.)
const INCOMPATIBLE: &[(ReactiveGroup, ReactiveGroup, &str)] = &[
    (
        ReactiveGroup::OxidizerHypochlorite,
        ReactiveGroup::AmmoniaAmines,
        "mixing bleach with ammonia releases toxic chloramine gases",
    ),
    (
        ReactiveGroup::OxidizerHypochlorite,
        ReactiveGroup::AcidStrong,
        "mixing bleach with acid releases toxic chlorine gas",
    ),
];

/// The L0 screen: vetoes any state whose species carry incompatible
/// reactive groups.
pub struct ReactiveGroupScreen;

impl SafetyScreen for ReactiveGroupScreen {
    fn veto(&self, vessel: &Vessel) -> Option<String> {
        let present: Vec<(&str, ReactiveGroup)> = vessel
            .contents
            .iter()
            .flat_map(|p| {
                let key = p.species.0.as_str();
                groups(key).iter().map(move |g| (key, *g))
            })
            .collect();
        for (i, (_, a)) in present.iter().enumerate() {
            for (_, b) in present.iter().skip(i + 1) {
                for (x, y, reason) in INCOMPATIBLE {
                    if (a == x && b == y) || (a == y && b == x) {
                        return Some((*reason).to_string());
                    }
                }
            }
        }
        None
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
    fn bleach_and_ammonia_is_vetoed() {
        let v = vessel_with(&["water", "NaOCl", "NH3"]);
        let veto = ReactiveGroupScreen.veto(&v);
        assert!(veto.is_some_and(|r| r.contains("chloramine")));
    }

    #[test]
    fn bleach_and_acid_is_vetoed() {
        let v = vessel_with(&["NaOCl", "HCl"]);
        let veto = ReactiveGroupScreen.veto(&v);
        assert!(veto.is_some_and(|r| r.contains("chlorine")));
    }

    #[test]
    fn acid_and_base_is_allowed() {
        // Neutralisation is chemistry, not a hazard veto.
        let v = vessel_with(&["water", "HCl", "NaOH"]);
        assert!(ReactiveGroupScreen.veto(&v).is_none());
    }

    #[test]
    fn benign_mixtures_pass() {
        let v = vessel_with(&["water", "NaCl", "AgNO3"]);
        assert!(ReactiveGroupScreen.veto(&v).is_none());
    }
}
