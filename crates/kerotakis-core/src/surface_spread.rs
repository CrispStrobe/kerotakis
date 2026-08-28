//! Bounded pepper-and-soap surface observable (BRD-014).
//!
//! This module intentionally models one familiar ordering: unresolved floating
//! grains are placed on quiet liquid, then a recipe-declared surfactant dose
//! clears a central region. It does not supply a universal surface-tension
//! coefficient, Marangoni flow field, or CFD particle trajectory.

use crate::material::{self, MaterialRecipe, MaterialRole};
use crate::vessel::{SurfaceParticleState, Vessel};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceSpreadObservation {
    pub from_cleared_fraction: f64,
    pub to_cleared_fraction: f64,
    pub coverage_fraction: f64,
}

/// Update the persistent surface layer after one named material addition.
/// Returns an observation only when a newly added reducer actually pushes an
/// existing floating layer outward.
pub fn after_material_added(
    vessel: &mut Vessel,
    added_recipe: &MaterialRecipe,
) -> Option<SurfaceSpreadObservation> {
    if vessel.liquid_volume().0 <= 1e-9 {
        return None;
    }

    if let Some(saturation) = added_recipe.roles.iter().find_map(|role| match role {
        MaterialRole::SurfaceFloater { saturation_amount } => Some(*saturation_amount),
        _ => None,
    }) {
        let amount = vessel
            .unresolved_materials
            .iter()
            .filter(|portion| {
                portion.recipe_id == added_recipe.id
                    && portion.recipe_version == added_recipe.version
            })
            .map(|portion| portion.amount)
            .sum::<f64>();
        let prior_clear = reducer_clear_fraction(vessel);
        vessel.surface_particles = Some(SurfaceParticleState {
            material: added_recipe.name.clone(),
            coverage_fraction: (amount / saturation).clamp(0.0, 1.0),
            // If detergent was already present there is no sudden gradient;
            // retain the equilibrium edge-clearing state without an event.
            cleared_fraction: prior_clear,
        });
        return None;
    }

    let is_reducer = added_recipe
        .roles
        .iter()
        .any(|role| matches!(role, MaterialRole::SurfaceTensionReducer { .. }));
    if !is_reducer {
        return None;
    }
    let to = reducer_clear_fraction(vessel);
    let particles = vessel.surface_particles.as_mut()?;
    let from = particles.cleared_fraction;
    particles.cleared_fraction = particles.cleared_fraction.max(to);
    (particles.cleared_fraction > from + 1e-9).then_some(SurfaceSpreadObservation {
        from_cleared_fraction: from,
        to_cleared_fraction: particles.cleared_fraction,
        coverage_fraction: particles.coverage_fraction,
    })
}

fn reducer_clear_fraction(vessel: &Vessel) -> f64 {
    vessel
        .unresolved_materials
        .iter()
        .filter_map(|portion| {
            let recipe = material::all().into_iter().find(|recipe| {
                recipe.id == portion.recipe_id && recipe.version == portion.recipe_version
            })?;
            recipe.roles.into_iter().find_map(|role| match role {
                MaterialRole::SurfaceTensionReducer {
                    saturation_amount,
                    max_cleared_fraction,
                } => Some(
                    (portion.amount / saturation_amount).clamp(0.0, 1.0) * max_cleared_fraction,
                ),
                _ => None,
            })
        })
        .fold(0.0, f64::max)
}
