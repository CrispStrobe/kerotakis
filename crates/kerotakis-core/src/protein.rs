//! Minimal recipe-level protein state for named biological materials.
//!
//! Proteins are polydisperse macromolecules, not honest small-molecule
//! species with one formula and molar mass. This bridge therefore keeps mass
//! in the conserved material portion and exposes only reviewed, bounded
//! material behaviour: protein inventory and heat denaturation.

use crate::material::{self, MaterialBasis};
use crate::vessel::Vessel;

#[derive(Debug, Clone, PartialEq)]
pub struct ProteinObservation {
    pub material: String,
    pub recipe_id: String,
    pub protein_mass_g: f64,
    pub denatured_fraction: f64,
    pub coagulated: bool,
}

#[derive(Clone, Copy)]
struct ProteinProfile {
    recipe_id: &'static str,
    protein_share_of_unresolved: f64,
    denaturation_onset_c: Option<f64>,
    denaturation_full_c: Option<f64>,
    coagulates: bool,
}

const PROFILES: &[ProteinProfile] = &[
    ProteinProfile {
        recipe_id: "food/egg-white",
        protein_share_of_unresolved: 0.95,
        denaturation_onset_c: Some(62.0),
        denaturation_full_c: Some(70.0),
        coagulates: true,
    },
    ProteinProfile {
        recipe_id: "food/gelatin",
        protein_share_of_unresolved: 1.0,
        // Gelatine is collagen that has already been denatured during
        // manufacture; cooling gelation is a separate structural transition.
        denaturation_onset_c: None,
        denaturation_full_c: None,
        coagulates: false,
    },
    ProteinProfile {
        recipe_id: "food/cream",
        protein_share_of_unresolved: 0.071,
        denaturation_onset_c: Some(70.0),
        denaturation_full_c: Some(85.0),
        coagulates: false,
    },
    ProteinProfile {
        recipe_id: "food/albumin",
        protein_share_of_unresolved: 1.0,
        denaturation_onset_c: Some(60.0),
        denaturation_full_c: Some(75.0),
        coagulates: true,
    },
];

pub fn observe(vessel: &Vessel) -> Vec<ProteinObservation> {
    let celsius = vessel.temperature.to_celsius();
    vessel
        .unresolved_materials
        .iter()
        .filter_map(|portion| {
            let profile = PROFILES
                .iter()
                .find(|profile| profile.recipe_id == portion.recipe_id)?;
            let recipe = material::lookup_versioned(&portion.recipe_id, portion.recipe_version)?;
            let unresolved_mass_g = match portion.basis {
                MaterialBasis::MassFraction => portion.amount,
                MaterialBasis::VolumeFraction => portion.amount * recipe.bulk_density?.value,
                MaterialBasis::MoleFraction => return None,
            };
            let denatured_fraction =
                match (profile.denaturation_onset_c, profile.denaturation_full_c) {
                    (Some(onset), Some(full)) => {
                        ((celsius - onset) / (full - onset)).clamp(0.0, 1.0)
                    }
                    // Manufactured gelatine begins in the denatured state.
                    _ => 1.0,
                };
            Some(ProteinObservation {
                material: recipe.name,
                recipe_id: recipe.id,
                protein_mass_g: unresolved_mass_g * profile.protein_share_of_unresolved,
                denatured_fraction,
                coagulated: profile.coagulates && denatured_fraction >= 0.5,
            })
        })
        .filter(|observation| observation.protein_mass_g > 1e-12)
        .collect()
}
