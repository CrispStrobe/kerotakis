//! Bounded enzyme activity inside explicitly unresolved food materials.
//!
//! The registry does not yet carry honest molecular ledgers for milk sugar,
//! triglyceride mixtures, or polydisperse proteins and their products.  This
//! module therefore records only how much of a reviewed substrate fraction
//! was hydrolysed.  The material mass stays conserved and unresolved.

use crate::enzyme::{EnzymeFamily, EnzymeProfile, SubstrateClass};
use crate::material::{self, MaterialRole};
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
    class: SubstrateClass,
    substrate: &'static str,
    /// Share of the recipe's unresolved portion that is the named substrate.
    substrate_share: f64,
}

const PROFILES: &[SubstrateProfile] = &[
    SubstrateProfile {
        recipe_id: "household/whole-milk-surrogate",
        class: SubstrateClass::Lactose,
        substrate: "lactose in milk",
        // Representative 4.8% lactose inside the recipe's 13% unresolved solids.
        substrate_share: 0.048 / 0.13,
    },
    SubstrateProfile {
        recipe_id: "food/gelatin",
        class: SubstrateClass::Protein,
        substrate: "gelatine protein",
        substrate_share: 1.0,
    },
    SubstrateProfile {
        recipe_id: "food/albumin",
        class: SubstrateClass::Protein,
        substrate: "albumin protein",
        substrate_share: 1.0,
    },
    SubstrateProfile {
        recipe_id: "household/vegetable-oil-surrogate",
        class: SubstrateClass::Triglyceride,
        substrate: "triglycerides in vegetable oil",
        substrate_share: 1.0,
    },
    // bio-051: a raw cut of muscle. Roughly three quarters of the recipe's
    // conserved 28% remainder is protein and the rest is fat and connective
    // tissue, so the substrate share is 0.75 rather than 1.0 — a protease
    // does not eat the fat. What this reports is hydrolysed protein mass and
    // never texture: tenderness is a mechanical property of collagen, and
    // this is a chemical claim about peptide bonds.
    SubstrateProfile {
        recipe_id: "food/meat",
        class: SubstrateClass::Protein,
        substrate: "muscle protein",
        substrate_share: 0.75,
    },
];

/// The share of a recipe's conserved remainder that is milk sugar.
///
/// One number, two consumers: the lactase activity model hydrolyses it and
/// the lactic fermentation route eats it. It lives here because the
/// substrate table is here, and it is exposed rather than copied so the
/// two engines can never disagree about how much lactose a gram of milk
/// solids is.
pub fn unresolved_lactose_share(recipe_id: &str) -> Option<f64> {
    PROFILES
        .iter()
        .find(|profile| profile.recipe_id == recipe_id && profile.class == SubstrateClass::Lactose)
        .map(|profile| profile.substrate_share)
}

// Editorial teaching correlation, not a measured universal rate constant.
// At the reference temperature it converts roughly 97% of 1 g substrate in
// an hour with 0.1 g enzyme. Dose ordering and the bounded asymptote are the
// claims; product-specific kinetics, inhibition, and irreversible denaturing
// remain outside this model.
const REFERENCE_RATE_PER_SECOND: f64 = 0.01;

/// One catalyst dose available in a vessel, already reduced to grams.
struct Catalyst {
    profile: &'static EnzymeProfile,
    grams: f64,
}

fn dissolved_catalyst_g(vessel: &Vessel, profile: &EnzymeProfile) -> f64 {
    let id = crate::SpeciesId::new(profile.species);
    let Some(data) = species::lookup(&id) else {
        return 0.0;
    };
    vessel.moles_of(&id).0 * data.molar_mass
}

/// The enzyme-source role of an unresolved portion, resolved against the
/// catalyst catalogue, with the dose the portion's dispensed amount implies.
///
/// `portion.amount` is the conserved remainder, not what the operator
/// weighed out, so the dispensed mass is recovered through the recipe's own
/// unresolved fraction. That keeps the role's number readable as "per gram
/// of pineapple" rather than "per gram of whatever pineapple this bench
/// could not resolve".
fn carried_catalyst(
    portion: &crate::vessel::UnresolvedMaterialPortion,
) -> Option<(&'static EnzymeProfile, f64, f64)> {
    let recipe = material::lookup_versioned(&portion.recipe_id, portion.recipe_version)?;
    let (enzyme, per_gram, denatures_above_k) =
        recipe.roles.iter().find_map(|role| match role {
            MaterialRole::EnzymeSource {
                enzyme,
                catalyst_equivalent_per_gram,
                denatures_above_k,
            } => Some((
                enzyme.clone(),
                *catalyst_equivalent_per_gram,
                *denatures_above_k,
            )),
            _ => None,
        })?;
    let profile = crate::enzyme::profile(&enzyme)?;
    let unresolved = recipe
        .unresolved_fraction
        .map(|fraction| fraction.lower.max(fraction.upper))
        .filter(|share| *share > 0.0)?;
    let dispensed_g = portion.amount / unresolved;
    Some((profile, dispensed_g * per_gram, denatures_above_k))
}

/// How far off the optimum the beaker is, as a factor in 0..=1.
fn envelope(value: f64, optimum: f64, width: f64) -> f64 {
    let z = (value - optimum) / width;
    (-0.5 * z * z).exp()
}

/// Advance every supported substrate during one shared bench-time interval.
pub fn advance(vessel: &mut Vessel, seconds: f64) -> Vec<EnzymeActivityStep> {
    if seconds <= 0.0 || vessel.liquid_volume().0 <= 0.0 {
        return Vec::new();
    }
    let temperature = vessel.temperature.0;
    // A vessel no aqueous solver has characterised has no acidity to read.
    // Inventing a neutral one would silently switch pepsin off in exactly
    // the beakers where nothing has looked, so the pH term is left at full
    // strength and the temperature term carries the whole envelope — which
    // is the behaviour every reviewed pair had before pH existed here.
    let ph = vessel.solution.as_ref().map(|solution| solution.ph);

    // ── survey: which catalysts are in the beaker, and in what dose ──
    let mut doses: Vec<Catalyst> = crate::enzyme::FAMILIES
        .iter()
        .filter(|profile| profile.substrate.is_some())
        .filter_map(|profile| {
            let grams = dissolved_catalyst_g(vessel, profile);
            (grams > 0.0).then_some(Catalyst { profile, grams })
        })
        .collect();
    let mut newly_denatured: Vec<(usize, EnzymeFamily)> = Vec::new();
    for (index, portion) in vessel.unresolved_materials.iter().enumerate() {
        let Some((profile, grams, denatures_above_k)) = carried_catalyst(portion) else {
            continue;
        };
        let already = portion
            .enzyme_hydrolysis
            .as_ref()
            .is_some_and(|state| state.carried_enzyme_denatured);
        if already {
            continue;
        }
        if temperature > denatures_above_k {
            newly_denatured.push((index, profile.family));
            continue;
        }
        if grams > 0.0 && profile.substrate.is_some() {
            doses.push(Catalyst { profile, grams });
        }
    }
    for (index, family) in newly_denatured {
        let portion = &mut vessel.unresolved_materials[index];
        match portion.enzyme_hydrolysis.as_mut() {
            Some(state) => state.carried_enzyme_denatured = true,
            None => {
                portion.enzyme_hydrolysis = Some(EnzymeHydrolysisState {
                    family,
                    converted_fraction: 0.0,
                    carried_enzyme_denatured: true,
                })
            }
        }
    }
    if doses.is_empty() {
        return Vec::new();
    }

    // ── act: every catalyst that cuts this class works the same pool ──
    let mut steps = Vec::new();
    for portion in &mut vessel.unresolved_materials {
        let Some(profile) = PROFILES.iter().find(|p| p.recipe_id == portion.recipe_id) else {
            continue;
        };
        let substrate_g = portion.amount * profile.substrate_share;
        if substrate_g <= 0.0 {
            continue;
        }
        // Rates add because the catalysts share one substrate pool; the
        // event is attributed to the one doing most of the cutting, which
        // is also the family the progress state records.
        let mut exponent = 0.0;
        let mut leader: Option<(&'static EnzymeProfile, f64)> = None;
        for dose in &doses {
            if dose.profile.substrate != Some(profile.class) {
                continue;
            }
            let temperature_factor = envelope(
                temperature,
                dose.profile.optimum_temperature_k,
                dose.profile.temperature_width_k,
            );
            let acidity_factor = ph.map_or(1.0, |ph| {
                envelope(ph, dose.profile.optimum_ph, dose.profile.ph_width)
            });
            let share = REFERENCE_RATE_PER_SECOND * dose.grams / substrate_g
                * temperature_factor
                * acidity_factor
                * seconds;
            if share <= 0.0 {
                continue;
            }
            exponent += share;
            if leader.is_none_or(|(_, best)| share > best) {
                leader = Some((dose.profile, share));
            }
        }
        let Some((acting, _)) = leader else {
            continue;
        };
        let before = portion
            .enzyme_hydrolysis
            .as_ref()
            .map_or(0.0, |state| state.converted_fraction)
            .clamp(0.0, 1.0);
        let after = 1.0 - (1.0 - before) * (-exponent).exp();
        let delta = (after - before).max(0.0);
        if delta * substrate_g <= 1e-9 {
            continue;
        }
        let denatured = portion
            .enzyme_hydrolysis
            .as_ref()
            .is_some_and(|state| state.carried_enzyme_denatured);
        portion.enzyme_hydrolysis = Some(EnzymeHydrolysisState {
            family: acting.family,
            converted_fraction: after,
            carried_enzyme_denatured: denatured,
        });
        steps.push(EnzymeActivityStep {
            family: acting.family,
            material: material::lookup_versioned(&portion.recipe_id, portion.recipe_version)
                .map(|recipe| recipe.name)
                .unwrap_or_else(|| portion.material.clone()),
            substrate: profile.substrate,
            hydrolysed_mass_g: delta * substrate_g,
            converted_fraction: after,
        });
    }
    steps
}
