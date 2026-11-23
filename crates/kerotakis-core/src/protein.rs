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
    // Whole milk: 3.15 g protein per 100 g (USDA FoodData Central, SR Legacy
    // NDB 01077, milk, whole, 3.25 % milkfat), out of the recipe's 12.20 %
    // unresolved remainder. Casein does not heat-set; the whey proteins
    // denature over the same window as cream's, and milk does not coagulate
    // on heating alone.
    ProteinProfile {
        recipe_id: "household/whole-milk-surrogate",
        protein_share_of_unresolved: 0.0315 / 0.122_014_9,
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

pub fn is_protein_recipe(recipe_id: &str) -> bool {
    PROFILES
        .iter()
        .any(|profile| profile.recipe_id == recipe_id)
}

/// Record the irreversible envelope of the existing temperature thresholds.
/// This is a qualitative history model, not a measured time-dependent rate law.
/// Call after physical state changes; observing a sample never mutates it.
pub fn record_heat_history(vessel: &mut Vessel) {
    let celsius = vessel.temperature.to_celsius();
    if !celsius.is_finite() {
        return;
    }
    // Enzyme lots (the catalase in dry yeast, a weighed-out protease) are
    // proteins too: past their profile's denaturation temperature they are
    // marked dead for good, which the kinetic catalyst reads.
    let kelvin = vessel.temperature.0;
    for lot in &mut vessel.lots {
        if crate::enzyme::profile(&lot.species.0)
            .is_some_and(|profile| kelvin > profile.denatures_above_k)
        {
            lot.heat_denatured = true;
        }
    }
    let chemical = chemical_fraction(vessel);
    for portion in &mut vessel.unresolved_materials {
        if let Some(profile) = PROFILES.iter().find(|p| p.recipe_id == portion.recipe_id) {
            portion.protein_denatured_fraction = portion
                .protein_denatured_fraction
                .max(temperature_fraction(profile, celsius))
                .max(chemical)
                .clamp(0.0, 1.0);
        }
    }
}

/// Ethanol volume fraction of the liquid at which dissolved protein is
/// treated as precipitated. EDITORIAL: cold-ethanol fractionation of plasma
/// proteins (Cohn et al., J. Am. Chem. Soc. 68 (1946) 459) precipitates
/// most protein between about 8 and 40 % ethanol depending on pH and
/// temperature; one threshold inside that range stands in for all of it.
const ETHANOL_PRECIPITATES: f64 = 0.25;

/// Heavy-metal cations that denature and precipitate protein by binding to
/// its side chains (the reason egg white is a first-aid antidote for them).
const HEAVY_METAL_IONS: &[&str] = &["Cu+2", "Ag+", "Pb+2", "Zn+2"];

/// Protein denatured by the solution itself rather than by heat: a
/// qualitative history like the heat envelope, not a rate law.
fn chemical_fraction(vessel: &Vessel) -> f64 {
    use crate::species::Phase;
    let litres = vessel.liquid_volume().0;
    if litres <= 0.0 {
        return 0.0;
    }
    let ethanol_litres: f64 = vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == "ethanol" && p.phase == Phase::Liquid)
        .filter_map(|p| {
            crate::species::lookup(&p.species)
                .map(|d| p.moles.0 * d.molar_mass / d.density / 1000.0)
        })
        .sum();
    if ethanol_litres / litres >= ETHANOL_PRECIPITATES {
        return 1.0;
    }
    // In strong alkali copper is held by the biuret chelate instead, which
    // leaves the solution clear and violet.
    if crate::reducing_sugar::alkaline(vessel) {
        return 0.0;
    }
    let metal: f64 = vessel
        .contents
        .iter()
        .filter(|p| p.phase == Phase::Aqueous && HEAVY_METAL_IONS.contains(&p.species.0.as_str()))
        .map(|p| p.moles.0)
        .sum();
    let sites = crate::biuret::peptide_sites(vessel);
    if metal <= 0.0 || sites <= 0.0 {
        return 0.0;
    }
    (metal / sites).clamp(0.0, 1.0)
}

fn temperature_fraction(profile: &ProteinProfile, celsius: f64) -> f64 {
    match (profile.denaturation_onset_c, profile.denaturation_full_c) {
        (Some(onset), Some(full)) => ((celsius - onset) / (full - onset)).clamp(0.0, 1.0),
        // Manufactured gelatine begins in the denatured state.
        _ => 1.0,
    }
}

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
            let denatured_fraction = portion
                .protein_denatured_fraction
                .max(temperature_fraction(profile, celsius))
                .clamp(0.0, 1.0);
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
