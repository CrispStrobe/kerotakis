//! Bounded hydration model for a declared superabsorbent-polymer material.
//!
//! Water is retained in a cross-linked polyelectrolyte network; it is not
//! consumed and this module never edits the mole ledger. The shipped material
//! remains unresolved because instant-snow powders vary by formulation.

use crate::material::MaterialBasis;
use crate::species;
use crate::vessel::Vessel;

pub const RECIPE_ID: &str = "school/sodium-polyacrylate-instant-snow";

#[derive(Debug, Clone, PartialEq)]
pub struct SwellingObservation {
    pub dry_polymer_g: f64,
    pub available_water_g: f64,
    pub retained_water_g: f64,
    pub swelling_ratio_g_per_g: f64,
    pub capacity_g_per_g: f64,
    pub saturated: bool,
}

/// Observe equilibrium uptake without moving or creating matter.
///
/// `100 g/g` is a conservative teaching capacity, not a product
/// specification. Real sodium-polyacrylate gels depend strongly on crosslink
/// density and ionic environment (Mussel, Basser & Horkay, Soft Matter 2019,
/// DOI 10.1039/C9SM00464E).
pub fn observe(vessel: &Vessel) -> Option<SwellingObservation> {
    const CAPACITY_G_PER_G: f64 = 100.0;
    let dry_polymer_g: f64 = vessel
        .unresolved_materials
        .iter()
        .filter(|p| p.recipe_id == RECIPE_ID && p.basis == MaterialBasis::MassFraction)
        .map(|p| p.amount)
        .sum();
    if dry_polymer_g <= 1e-9 {
        return None;
    }
    let available_water_g: f64 = vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == "water")
        .map(|p| p.moles.0 * species::lookup_key("water").map_or(0.0, |d| d.molar_mass))
        .sum();
    let retained_water_g = available_water_g.min(dry_polymer_g * CAPACITY_G_PER_G);
    Some(SwellingObservation {
        dry_polymer_g,
        available_water_g,
        retained_water_g,
        swelling_ratio_g_per_g: retained_water_g / dry_polymer_g,
        capacity_g_per_g: CAPACITY_G_PER_G,
        saturated: retained_water_g + 1e-9 >= dry_polymer_g * CAPACITY_G_PER_G,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vessel::{UnresolvedMaterialPortion, VesselId};
    use crate::{Phase, SpeciesId};

    #[test]
    fn uptake_is_water_limited_and_conserves_the_ledger() {
        let mut vessel = Vessel::new(VesselId(0), "beaker");
        vessel.unresolved_materials.push(UnresolvedMaterialPortion {
            material: "instant snow".into(),
            recipe_id: RECIPE_ID.into(),
            recipe_version: 1,
            basis: MaterialBasis::MassFraction,
            amount: 1.0,
            enzyme_hydrolysis: None,
        });
        vessel.deposit(
            SpeciesId::new("water"),
            crate::Moles(50.0 / 18.01528),
            Phase::Liquid,
        );
        let water_before: f64 = vessel.contents.iter().map(|p| p.moles.0).sum();
        let seen = observe(&vessel).unwrap();
        assert!((seen.swelling_ratio_g_per_g - 50.0).abs() < 0.01);
        assert!(!seen.saturated);
        assert_eq!(
            water_before,
            vessel.contents.iter().map(|p| p.moles.0).sum::<f64>()
        );
    }
}
