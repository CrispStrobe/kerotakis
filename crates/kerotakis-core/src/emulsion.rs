//! Bounded oil-in-water emulsion observable for household detergent.
//!
//! Mechanical stirring supplies the action; a recipe-declared surfactant dose
//! determines how much of a reviewed immiscible material is temporarily
//! dispersed. No molecular surfactant, CMC, droplet distribution, rheology, or
//! CFD trajectory is inferred.

use crate::material::{self, MaterialRole};
use crate::vessel::{EmulsionState, Vessel};

#[derive(Debug, Clone, PartialEq)]
pub struct EmulsionObservation {
    pub material: String,
    pub oil_recipe_id: String,
    pub dispersed_volume_l: f64,
    pub dispersed_fraction: f64,
    pub half_life_seconds: f64,
}

pub fn observe(vessel: &Vessel) -> Option<EmulsionObservation> {
    let state = vessel.emulsion.as_ref()?;
    let layer = material::immiscible_liquid_layers(vessel)
        .into_iter()
        .find(|layer| layer.recipe_id == state.oil_recipe_id)?;
    let dispersed_volume_l = state.dispersed_volume_l.clamp(0.0, layer.volume_l);
    (dispersed_volume_l > 1e-9).then_some(EmulsionObservation {
        material: layer.material,
        oil_recipe_id: layer.recipe_id,
        dispersed_volume_l,
        dispersed_fraction: (dispersed_volume_l / layer.volume_l).clamp(0.0, 1.0),
        half_life_seconds: state.half_life_seconds,
    })
}

/// Apply a completed mechanical stir. Returns an observation only when a
/// water phase, immiscible oil, and recipe-declared emulsifier are all present.
pub fn after_stir(vessel: &mut Vessel, mixing_fraction: f64) -> Option<EmulsionObservation> {
    if vessel.liquid_volume().0 <= 1e-9 {
        return None;
    }
    let oil = material::immiscible_liquid_layers(vessel)
        .into_iter()
        .next()?;
    let emulsifier = vessel
        .unresolved_materials
        .iter()
        .filter_map(|portion| {
            let recipe = material::lookup_versioned(&portion.recipe_id, portion.recipe_version)?;
            recipe.roles.into_iter().find_map(|role| match role {
                MaterialRole::AqueousEmulsifier {
                    saturation_amount,
                    max_dispersed_fraction,
                    half_life_seconds,
                } => Some((
                    (portion.amount / saturation_amount).clamp(0.0, 1.0),
                    max_dispersed_fraction,
                    half_life_seconds,
                )),
                _ => None,
            })
        })
        .max_by(|left, right| left.0.total_cmp(&right.0))?;
    let target_fraction =
        (emulsifier.0 * emulsifier.1 * mixing_fraction.clamp(0.0, 1.0)).clamp(0.0, 1.0);
    let target_volume_l = oil.volume_l * target_fraction;
    let existing = vessel
        .emulsion
        .as_ref()
        .filter(|state| state.oil_recipe_id == oil.recipe_id)
        .map(|state| state.dispersed_volume_l)
        .unwrap_or(0.0);
    vessel.emulsion = Some(EmulsionState {
        oil_recipe_id: oil.recipe_id,
        dispersed_volume_l: existing.max(target_volume_l),
        half_life_seconds: emulsifier.2,
    });
    observe(vessel)
}

/// Coalescence while the vessel rests. Event construction stays in the bench
/// loop so this module only mutates the persistent geometry state.
pub fn advance(vessel: &mut Vessel, seconds: f64) {
    let Some(before) = observe(vessel) else {
        return;
    };
    let decay = (-std::f64::consts::LN_2 * seconds.max(0.0) / before.half_life_seconds).exp();
    if let Some(state) = vessel.emulsion.as_mut() {
        state.dispersed_volume_l *= decay;
        if state.dispersed_volume_l <= 1e-9 {
            vessel.emulsion = None;
        }
    }
}
