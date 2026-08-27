//! Bounded baker's-yeast fermentation of dissolved sucrose.
//!
//! The chemistry ledger follows the aggregate balanced reaction
//! C12H22O11 + H2O -> 4 C2H5OH + 4 CO2. The rate is deliberately a
//! recipe-level classroom response: finite sugar, yeast dose, hydration and a
//! smooth temperature envelope matter, but cell growth, oxygen switching,
//! inhibition, strain variation and secondary metabolites are not claimed.

use crate::material::{self, MaterialRole};
use crate::species::{Phase, SpeciesId};
use crate::units::Moles;
use crate::vessel::Vessel;

const SUCROSE: &str = "sucrose";
const WATER: &str = "water";
const ETHANOL: &str = "ethanol";
const CARBON_DIOXIDE: &str = "CO2";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FermentationStep {
    pub sucrose_moles: f64,
    pub ethanol_moles: f64,
    pub carbon_dioxide_moles: f64,
    pub active_yeast_grams: f64,
}

pub fn advance(vessel: &mut Vessel, seconds: f64) -> Option<FermentationStep> {
    if seconds <= 0.0 || vessel.liquid_volume().0 <= 1e-9 {
        return None;
    }
    let dissolved_sucrose = phase_moles(vessel, SUCROSE, Phase::Aqueous);
    if dissolved_sucrose <= 0.0 {
        return None;
    }

    let mut effective_rate = 0.0;
    let mut active_yeast_grams = 0.0;
    for portion in &vessel.unresolved_materials {
        let Some(recipe) = material::lookup_versioned(&portion.recipe_id, portion.recipe_version)
        else {
            continue;
        };
        let Some((reference_rate, optimum, width, requires_hydration)) =
            recipe.roles.iter().find_map(|role| match role {
                MaterialRole::FermentationCulture {
                    reference_rate_per_second_per_gram,
                    optimum_temperature_k,
                    temperature_width_k,
                    requires_hydration,
                } => Some((
                    *reference_rate_per_second_per_gram,
                    *optimum_temperature_k,
                    *temperature_width_k,
                    *requires_hydration,
                )),
                _ => None,
            })
        else {
            continue;
        };
        let hydration = if requires_hydration {
            dry_yeast_hydration(vessel, &recipe.id)
        } else {
            1.0
        };
        let temperature = (-((vessel.temperature.0 - optimum) / width).powi(2)).exp();
        let active = portion.amount * hydration * temperature;
        active_yeast_grams += active;
        effective_rate += reference_rate * active;
    }
    if effective_rate <= 0.0 {
        return None;
    }

    let wanted = dissolved_sucrose * (1.0 - (-effective_rate * seconds).exp());
    let water_limit = phase_moles(vessel, WATER, Phase::Liquid);
    let extent = wanted.min(water_limit).max(0.0);
    if extent <= 0.0 {
        return None;
    }

    withdraw_phase(vessel, SUCROSE, Phase::Aqueous, extent);
    withdraw_phase(vessel, WATER, Phase::Liquid, extent);
    vessel.deposit(SpeciesId::new(ETHANOL), Moles(4.0 * extent), Phase::Aqueous);
    vessel.deposit(
        SpeciesId::new(CARBON_DIOXIDE),
        Moles(4.0 * extent),
        Phase::Gas,
    );
    vessel.resolved.invalidate();
    Some(FermentationStep {
        sucrose_moles: extent,
        ethanol_moles: 4.0 * extent,
        carbon_dioxide_moles: 4.0 * extent,
        active_yeast_grams,
    })
}

fn phase_moles(vessel: &Vessel, species: &str, phase: Phase) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|portion| portion.species.0 == species && portion.phase == phase)
        .map(|portion| portion.moles.0)
        .sum()
}

fn withdraw_phase(vessel: &mut Vessel, species: &str, phase: Phase, moles: f64) {
    let mut remaining = moles;
    for portion in &mut vessel.contents {
        if portion.species.0 == species && portion.phase == phase && remaining > 0.0 {
            let take = portion.moles.0.min(remaining);
            portion.moles.0 -= take;
            remaining -= take;
        }
    }
    vessel.contents.retain(|portion| portion.moles.0 > 1e-15);
}

fn dry_yeast_hydration(vessel: &Vessel, recipe_id: &str) -> f64 {
    let source = format!("material recipe {recipe_id}");
    let hydrated_seconds = vessel
        .lots
        .iter()
        .filter(|lot| lot.source.as_deref() == Some(source.as_str()))
        .filter_map(|lot| lot.hydrated_at)
        .map(|started| (vessel.elapsed_seconds - started).max(0.0))
        .fold(0.0, f64::max);
    let tau = (6.0 * 2_f64.powf((298.15 - vessel.temperature.0) / 10.0)).clamp(2.0, 30.0);
    (1.0 - (-hydrated_seconds / tau).exp()).clamp(0.0, 1.0)
}
