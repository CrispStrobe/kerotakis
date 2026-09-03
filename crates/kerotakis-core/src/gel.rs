//! KID-14: borate-crosslinked polymer gel — the slime observable.
//!
//! Slime is not a reaction with a stoichiometry. Borate ions bridge the
//! hydroxyl groups on neighbouring poly(vinyl alcohol) chains, and the
//! bridges keep breaking and re-forming — which is why the result flows
//! slowly and tears quickly, and why nothing is consumed. So this module
//! does what [`crate::curdling`] does for milk: it maps the dose of
//! crosslinker per gram of polymer actually in the vessel onto a bounded
//! visible change, and creates no matter.
//!
//! The dose is per gram of *polymer*, not per gram of glue, so a thinner
//! glue needs proportionally less borax — which is the thing a child
//! discovers by getting it wrong twice.

use crate::species::{self, SpeciesId};
use crate::vessel::Vessel;

/// One curated polymer/crosslinker pair.
pub struct GelPair {
    pub polymer: &'static str,
    pub crosslinker: &'static str,
    /// Moles of crosslinker per gram of polymer at which the mixture first
    /// stops behaving like a liquid.
    pub onset_moles_per_gram: f64,
    /// Where the response saturates.
    pub full_moles_per_gram: f64,
    /// Ceiling on the fraction of polymer bound into the gel.
    pub max_gelled_fraction: f64,
    pub provenance: &'static str,
}

pub const GEL_PAIRS: &[GelPair] = &[GelPair {
    polymer: "PVA",
    crosslinker: "Na2B4O7",
    // Calibrated to the familiar classroom ratio — roughly 50 mL of school
    // glue to a few millilitres of dilute borax solution — so that the
    // amount a recipe card gives lands inside the response rather than at
    // one end of it.
    onset_moles_per_gram: 3.0e-5,
    full_moles_per_gram: 2.0e-4,
    max_gelled_fraction: 0.95,
    provenance: "Borate crosslinking of poly(vinyl alcohol): borate diesters bridge hydroxyls on neighbouring chains and exchange continuously, which is why the gel is viscoelastic and why the crosslinker is not consumed. Editorial judgement (Kerotakis): the onset and saturation doses are a bounded teaching response calibrated to the familiar 50 mL-glue classroom ratio, not measured rheology. No claim is made about modulus, relaxation time, the degree of hydrolysis that decides whether a given glue gels at all, or about poly(vinyl acetate) glues, which do not gel this way",
}];

/// What the gel looks like right now, if a curated pair is present.
#[derive(Debug, Clone, PartialEq)]
pub struct GelObservation {
    pub polymer: &'static str,
    pub crosslinker: &'static str,
    pub gelled_fraction: f64,
    pub polymer_grams: f64,
    pub crosslinker_moles: f64,
}

pub fn observe(vessel: &Vessel) -> Option<GelObservation> {
    GEL_PAIRS.iter().find_map(|pair| {
        let grams = |key: &str| -> f64 {
            let data = species::lookup_key(key);
            vessel
                .contents
                .iter()
                .filter(|p| p.species.0 == key)
                .map(|p| p.moles.0 * data.map_or(0.0, |d| d.molar_mass))
                .sum()
        };
        let moles = |key: &str| -> f64 {
            vessel
                .contents
                .iter()
                .filter(|p| p.species.0 == key)
                .map(|p| p.moles.0)
                .sum()
        };
        let polymer_grams = grams(pair.polymer);
        if polymer_grams <= 1e-9 {
            return None;
        }
        let crosslinker_moles = moles(pair.crosslinker);
        let dose = crosslinker_moles / polymer_grams;
        let progress = ((dose - pair.onset_moles_per_gram)
            / (pair.full_moles_per_gram - pair.onset_moles_per_gram))
            .clamp(0.0, 1.0);
        let gelled_fraction = progress * pair.max_gelled_fraction;
        (gelled_fraction > 1e-9).then_some(GelObservation {
            polymer: pair.polymer,
            crosslinker: pair.crosslinker,
            gelled_fraction,
            polymer_grams,
            crosslinker_moles,
        })
    })
}

/// The species a gel pair binds, so callers can name them.
pub fn crosslinker_of(polymer: &str) -> Option<SpeciesId> {
    GEL_PAIRS
        .iter()
        .find(|pair| pair.polymer == polymer)
        .map(|pair| SpeciesId::new(pair.crosslinker))
}
