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

/// KID-2: how much of the declared acid is in this vessel, whatever form
/// the solver left it in.
///
/// The dose used to be a sum over vessel contents whose species id equalled
/// the recipe's `acid_species` — `CH3COOH`. That works only when no aqueous
/// solver has run. With the engine linked, adding vinegar leaves the ledger
/// holding `CH3COO-` and no `CH3COOH` at all, so the dose was always zero,
/// the curdling event never fired, and `lessons/milk-curds.lab` did not
/// demonstrate its own headline claim on the shipped bench. The unit test
/// passed because it drove `Bench::step` without the solver behind it.
///
/// A Brønsted acid and its conjugate base differ only in hydrogen count and
/// charge, so the family is exactly the set of species sharing the acid's
/// non-hydrogen composition. Summing over that family restores the number
/// the recipe was calibrated against — 10 mL of 5% vinegar is 0.008376 mol
/// of acetate-equivalent whether the solver has deprotonated it or not.
///
/// **Stated boundary.** This counts acid *inventory*, not acidity. Adding
/// sodium acetate to milk would read as a dose here, and real milk would
/// not curdle: casein aggregates when the pH reaches its isoelectric point,
/// not when acetate is present. Making the response pH-driven needs a
/// reviewed isoelectric datum on the recipe and is tracked separately in
/// `KIDS.md`; what this function fixes is the model reading a ledger the
/// solver had already rewritten.
fn acid_inventory(vessel: &Vessel, acid_species: &str) -> f64 {
    let Some(acid) = crate::stoich::parse_formula(
        crate::species::lookup(&crate::species::SpeciesId::new(acid_species))
            .map(|data| data.formula)
            .unwrap_or(acid_species),
    )
    .ok() else {
        return 0.0;
    };
    let skeleton = |formula: &crate::stoich::Formula| -> Vec<(String, f64)> {
        formula
            .counts
            .iter()
            .filter(|(element, _)| element.as_str() != "H")
            .map(|(element, count)| (element.clone(), *count))
            .collect()
    };
    let wanted = skeleton(&acid);
    vessel
        .contents
        .iter()
        .filter(|item| {
            if item.species.0.as_str() == acid_species {
                return true;
            }
            crate::species::lookup(&item.species)
                .and_then(|data| crate::stoich::parse_formula(data.formula).ok())
                .is_some_and(|formula| skeleton(&formula) == wanted)
        })
        .map(|item| item.moles.0)
        .sum()
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
            let acid_moles = acid_inventory(vessel, acid_species);
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
