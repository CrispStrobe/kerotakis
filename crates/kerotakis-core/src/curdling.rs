//! Bounded acid-driven curd formation for recipe-level colloids.
//!
//! Milk remains a conserved named aggregate: this module does not invent a
//! casein molecule or alter the chemical ledger. It maps the actual amount of
//! a declared acid species per gram of unresolved milk material to the visible
//! separation demonstrated in classroom curds-and-whey activities.

use crate::material::{self, MaterialBasis, MaterialRole};
use crate::vessel::Vessel;

#[derive(Debug, Clone, PartialEq)]
pub struct CurdlingObservation {
    pub material: String,
    pub recipe_id: String,
    pub formed_fraction: f64,
    pub separation_progress: f64,
    pub opacity_reduction: f64,
    pub curd_solids_mass_g: f64,
    pub curd_srgb: [u8; 3],
    pub acid_species: String,
    pub acid_moles: f64,
}

pub fn observe(vessel: &Vessel) -> Option<CurdlingObservation> {
    vessel
        .unresolved_materials
        .iter()
        .filter_map(|portion| {
            let recipe = material::lookup_versioned(&portion.recipe_id, portion.recipe_version)?;
            let (acid_species, onset, full, maximum, max_opacity_reduction, curd_srgb) =
                recipe.roles.iter().find_map(|role| match role {
                    MaterialRole::AcidCurdlingColloid {
                        acid_species,
                        onset_moles_per_gram,
                        full_moles_per_gram,
                        max_curdled_fraction,
                        max_opacity_reduction,
                        curd_srgb,
                    } => Some((
                        acid_species,
                        *onset_moles_per_gram,
                        *full_moles_per_gram,
                        *max_curdled_fraction,
                        *max_opacity_reduction,
                        *curd_srgb,
                    )),
                    _ => None,
                })?;
            let unresolved_mass_g = match portion.basis {
                MaterialBasis::MassFraction => portion.amount,
                MaterialBasis::VolumeFraction => recipe
                    .bulk_density
                    .as_ref()
                    .map(|density| portion.amount * density.value)
                    .unwrap_or(0.0),
                MaterialBasis::MoleFraction => 0.0,
            };
            if unresolved_mass_g <= 1e-12 {
                return None;
            }
            let acid_moles = vessel
                .contents
                .iter()
                .filter(|item| item.species.0.as_str() == acid_species.as_str())
                .map(|item| item.moles.0)
                .sum::<f64>();
            let dose = acid_moles / unresolved_mass_g;
            let separation_progress = ((dose - onset) / (full - onset)).clamp(0.0, 1.0);
            let formed_fraction = (separation_progress * maximum).clamp(0.0, 1.0);
            (formed_fraction > 1e-9).then_some(CurdlingObservation {
                material: recipe.name,
                recipe_id: recipe.id,
                formed_fraction,
                separation_progress,
                opacity_reduction: separation_progress * max_opacity_reduction,
                curd_solids_mass_g: unresolved_mass_g * formed_fraction,
                curd_srgb,
                acid_species: acid_species.clone(),
                acid_moles,
            })
        })
        .max_by(|left, right| left.formed_fraction.total_cmp(&right.formed_fraction))
}
