//! Bounded enzyme activity inside explicitly unresolved food materials.
//!
//! The registry does not yet carry honest molecular ledgers for milk sugar,
//! triglyceride mixtures, or polydisperse proteins and their products.  This
//! module therefore records only how much of a reviewed substrate fraction
//! was hydrolysed.  The material mass stays conserved and unresolved.

use crate::enzyme::EnzymeFamily;
use crate::species;
use crate::vessel::{EnzymeHydrolysisState, Vessel};

#[derive(Debug, Clone, PartialEq)]
pub struct EnzymeActivityStep {
    pub family: EnzymeFamily,
    pub material: String,
    pub substrate: &'static str,
    pub hydrolysed_mass_g: f64,
    pub converted_fraction: f64,
}

#[derive(Clone, Copy)]
struct SubstrateProfile {
    recipe_id: &'static str,
    family: EnzymeFamily,
    substrate: &'static str,
    /// Share of the recipe's unresolved portion that is the named substrate.
    substrate_share: f64,
    optimum_k: f64,
    width_k: f64,
}

const PROFILES: &[SubstrateProfile] = &[
    SubstrateProfile {
        recipe_id: "household/whole-milk-surrogate",
        family: EnzymeFamily::Lactase,
        substrate: "lactose in milk",
        // Representative 4.8% lactose inside the recipe's 13% unresolved solids.
        substrate_share: 0.048 / 0.13,
        optimum_k: 310.15,
        width_k: 16.0,
    },
    SubstrateProfile {
        recipe_id: "food/gelatin",
        family: EnzymeFamily::Protease,
        substrate: "gelatine protein",
        substrate_share: 1.0,
        optimum_k: 310.15,
        width_k: 18.0,
    },
    SubstrateProfile {
        recipe_id: "food/albumin",
        family: EnzymeFamily::Protease,
        substrate: "albumin protein",
        substrate_share: 1.0,
        optimum_k: 310.15,
        width_k: 18.0,
    },
    SubstrateProfile {
        recipe_id: "household/vegetable-oil-surrogate",
        family: EnzymeFamily::Lipase,
        substrate: "triglycerides in vegetable oil",
        substrate_share: 1.0,
        optimum_k: 310.15,
        width_k: 18.0,
    },
    // bio-051: a raw cut of muscle. Roughly three quarters of the recipe's
    // conserved 28% remainder is protein and the rest is fat and connective
    // tissue, so the substrate share is 0.75 rather than 1.0 — a protease
    // does not eat the fat. What this reports is hydrolysed protein mass and
    // never texture: tenderness is a mechanical property of collagen, and
    // this is a chemical claim about peptide bonds.
    SubstrateProfile {
        recipe_id: "food/meat",
        family: EnzymeFamily::Protease,
        substrate: "muscle protein",
        substrate_share: 0.75,
        optimum_k: 310.15,
        width_k: 18.0,
    },
];

// Editorial teaching correlation, not a measured universal rate constant.
// At the reference temperature it converts roughly 97% of 1 g substrate in
// an hour with 0.1 g enzyme. Dose ordering and the bounded asymptote are the
// claims; product-specific kinetics, inhibition, and irreversible denaturing
// remain outside this model.
const REFERENCE_RATE_PER_SECOND: f64 = 0.01;

fn catalyst_mass_g(vessel: &Vessel, family: EnzymeFamily) -> f64 {
    let Some(profile) = crate::enzyme::FAMILIES.iter().find(|p| p.family == family) else {
        return 0.0;
    };
    let id = crate::SpeciesId::new(profile.species);
    let Some(data) = species::lookup(&id) else {
        return 0.0;
    };
    vessel.moles_of(&id).0 * data.molar_mass
}

/// Advance every supported substrate during one shared bench-time interval.
pub fn advance(vessel: &mut Vessel, seconds: f64) -> Vec<EnzymeActivityStep> {
    if seconds <= 0.0 || vessel.liquid_volume().0 <= 0.0 {
        return Vec::new();
    }
    let temperature = vessel.temperature.0;
    let catalyst_masses = [
        (
            EnzymeFamily::Lactase,
            catalyst_mass_g(vessel, EnzymeFamily::Lactase),
        ),
        (
            EnzymeFamily::Protease,
            catalyst_mass_g(vessel, EnzymeFamily::Protease),
        ),
        (
            EnzymeFamily::Lipase,
            catalyst_mass_g(vessel, EnzymeFamily::Lipase),
        ),
    ];
    let mut steps = Vec::new();
    for portion in &mut vessel.unresolved_materials {
        let Some(profile) = PROFILES.iter().find(|p| p.recipe_id == portion.recipe_id) else {
            continue;
        };
        let enzyme_g = catalyst_masses
            .iter()
            .find(|(family, _)| *family == profile.family)
            .map(|(_, mass)| *mass)
            .unwrap_or(0.0);
        let substrate_g = portion.amount * profile.substrate_share;
        if enzyme_g <= 0.0 || substrate_g <= 0.0 {
            continue;
        }
        let before = portion
            .enzyme_hydrolysis
            .as_ref()
            .filter(|state| state.family == profile.family)
            .map(|state| state.converted_fraction)
            .unwrap_or(0.0)
            .clamp(0.0, 1.0);
        let z = (temperature - profile.optimum_k) / profile.width_k;
        let temperature_factor = (-0.5 * z * z).exp();
        let exponent =
            REFERENCE_RATE_PER_SECOND * enzyme_g / substrate_g * temperature_factor * seconds;
        let after = 1.0 - (1.0 - before) * (-exponent).exp();
        let delta = (after - before).max(0.0);
        if delta * substrate_g <= 1e-9 {
            continue;
        }
        portion.enzyme_hydrolysis = Some(EnzymeHydrolysisState {
            family: profile.family,
            converted_fraction: after,
        });
        steps.push(EnzymeActivityStep {
            family: profile.family,
            material: crate::material::lookup_versioned(&portion.recipe_id, portion.recipe_version)
                .map(|recipe| recipe.name)
                .unwrap_or_else(|| portion.material.clone()),
            substrate: profile.substrate,
            hydrolysed_mass_g: delta * substrate_g,
            converted_fraction: after,
        });
    }
    steps
}
