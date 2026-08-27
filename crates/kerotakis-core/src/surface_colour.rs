//! Bounded surface-colour observable for the milk-and-detergent activity.
//!
//! This is not CFD. It conserves the resolved dye inventory while recording
//! whether that inventory is localized on an opaque colloid, spread across
//! its surface by a declared surfactant response, or homogenized by stirring.

use crate::material::{self, MaterialRecipe, MaterialRole};
use crate::species::SpeciesId;
use crate::units::Moles;
use crate::vessel::{SurfaceColourSpot, Vessel};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceColourObservation {
    pub from_spread_fraction: f64,
    pub to_spread_fraction: f64,
    pub spot_count: usize,
}

pub fn after_material_added(
    vessel: &mut Vessel,
    recipe: &MaterialRecipe,
    resolved_components: &[(SpeciesId, Moles)],
) -> Option<SurfaceColourObservation> {
    if vessel.liquid_volume().0 <= 1e-9 {
        return None;
    }

    if let Some(srgb) = recipe.roles.iter().find_map(|role| match role {
        MaterialRole::SurfaceColourant { srgb } => Some(*srgb),
        _ => None,
    }) {
        if has_opaque_colloid(vessel) {
            for (species, moles) in resolved_components {
                if moles.0 <= 0.0 || !has_visible_spectrum(species) {
                    continue;
                }
                vessel.surface_colours.push(SurfaceColourSpot {
                    material: recipe.name.clone(),
                    species: species.clone(),
                    moles: *moles,
                    srgb,
                    spread_fraction: reducer_spread_fraction(vessel),
                });
            }
        }
        return None;
    }

    if !recipe
        .roles
        .iter()
        .any(|role| matches!(role, MaterialRole::SurfaceTensionReducer { .. }))
    {
        return None;
    }
    let to = reducer_spread_fraction(vessel);
    let from = vessel
        .surface_colours
        .iter()
        .map(|spot| spot.spread_fraction)
        .fold(0.0, f64::max);
    for spot in &mut vessel.surface_colours {
        spot.spread_fraction = spot.spread_fraction.max(to);
    }
    (to > from + 1e-9 && !vessel.surface_colours.is_empty()).then_some(SurfaceColourObservation {
        from_spread_fraction: from,
        to_spread_fraction: to,
        spot_count: vessel.surface_colours.len(),
    })
}

pub fn sequestered_moles(vessel: &Vessel, species: &SpeciesId) -> f64 {
    vessel
        .surface_colours
        .iter()
        .filter(|spot| &spot.species == species)
        .map(|spot| spot.moles.0)
        .sum()
}

pub fn homogenize(vessel: &mut Vessel) -> usize {
    let count = vessel.surface_colours.len();
    vessel.surface_colours.clear();
    count
}

fn has_opaque_colloid(vessel: &Vessel) -> bool {
    vessel.unresolved_materials.iter().any(|portion| {
        material::all().into_iter().any(|recipe| {
            recipe.id == portion.recipe_id
                && recipe.version == portion.recipe_version
                && recipe
                    .roles
                    .iter()
                    .any(|role| matches!(role, MaterialRole::OpaqueLiquidColloid { .. }))
        })
    })
}

fn has_visible_spectrum(species: &SpeciesId) -> bool {
    crate::species::lookup(species)
        .and_then(|data| data.spectrum)
        .is_some()
}

fn reducer_spread_fraction(vessel: &Vessel) -> f64 {
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
