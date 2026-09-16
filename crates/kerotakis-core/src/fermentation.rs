//! Bounded food fermentation by declared cultures.
//!
//! Every route here is one balanced aggregate reaction on a disaccharide,
//! or on ethanol, and each conserves mass exactly:
//!
//! * alcoholic    C12H22O11 + H2O -> 4 C2H5OH + 4 CO2
//! * homolactic   C12H22O11 + H2O -> 4 C3H6O3
//! * heterolactic C12H22O11 + H2O -> 2 C3H6O3 + 2 C2H5OH + 2 CO2
//! * acetic       C2H5OH + O2 -> CH3COOH + H2O
//!
//! The rate is deliberately a recipe-level classroom response: finite
//! substrate, culture CONCENTRATION, hydration and a smooth temperature
//! envelope matter, but cell growth, oxygen switching, inhibition, pH
//! inhibition, strain variation and secondary metabolites are not claimed.
//! Neither is anything a fermented food is actually judged by — no flavour,
//! no aroma, no texture, no coagulation into a curd, and NO food safety:
//! nothing here models a pathogen, a spoilage organism or a competing
//! culture, so a finished run says an acid was made and never that the food
//! is safe.

use crate::material::{self, CultureMetabolism, MaterialRole};
use crate::species::{Phase, SpeciesId};
use crate::units::Moles;
use crate::vessel::Vessel;

const SUCROSE: &str = "sucrose";
const WATER: &str = "water";
const ETHANOL: &str = "ethanol";
const CARBON_DIOXIDE: &str = "CO2";
const LACTIC_ACID: &str = "lactic_acid";
const ACETIC_ACID: &str = "CH3COOH";
const OXYGEN: &str = "O2";

/// The liquid volume a culture's declared rate is quoted at: **one litre**.
///
/// A fermentation is first order in the culture's CONCENTRATION, not in the
/// grams of culture somebody weighed out. Until 2026-09-16 this file
/// multiplied `reference_rate * grams` with nothing in the denominator, so
/// doubling the milk AND the culture doubled both the rate and the
/// substrate and the product came out four times larger — measured on
/// `bio-070`, where 2.716e-5 mol of total lactic acid became 1.086e-4 when
/// the experiment was doubled. Dividing by the vessel's liquid volume is
/// what makes "twice the batch" mean twice the yoghurt.
///
/// A concentration needs a volume to be quoted against, and this is it,
/// stated rather than implied. A recipe's
/// `reference_rate_per_second_per_gram_per_litre` is the first-order rate
/// constant of ONE GRAM of that culture in ONE LITRE of liquid at its
/// declared optimum temperature; the same dose in half the liquid runs
/// twice as fast.
///
/// ONE LITRE IS A DECLARED BASIS AND NOT A FIT. The owner's decision of
/// 2026-09-16 was for a stated, citable calibration volume rather than one
/// back-solved to keep today's outputs where they were, and the difference
/// is visible: every bench script here pours 50-200 mL, so every
/// fermentation number in the repository moved up by the ratio of one litre
/// to the liquid actually in the beaker — about eleven-fold for 100 mL of
/// milk.
///
/// THE MAGNITUDE QUESTION THIS CONSTANT DOES NOT ANSWER WAS ANSWERED THE
/// SAME DAY, separately, and this constant is not what answers it. The four
/// rates were editorial classroom timescales with no measured activity
/// behind them; two are now FITTED to published fermentations and two are
/// carried across from the lactic fit, and each role says in its own
/// `rate_evidence` which it is. The lactic constant fell 51-fold and the
/// alcoholic one 23.6-fold, so `bio-069`'s eight-hour counter-top yoghurt
/// converts 1.48% of the milk's lactose where it briefly converted 53.2%.
/// `kerotakis/fermentation-rate-calibration-v1` in the registry carries the
/// sources, the arithmetic and the five things the fit does not buy.
///
/// THE pH WAS NOT FITTED, and that is the load-bearing one. Milk's casein
/// and colloidal calcium phosphate are about 60% of its buffer capacity and
/// are modelled by nothing, so a computed yoghurt pH is a LOWER BOUND at any
/// acid dose. A rate chosen to make 4.5 appear would have cancelled a rate
/// error against a buffer error and left both defects in place.
const REFERENCE_VOLUME_LITRES: f64 = 1.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FermentationStep {
    /// Sucrose consumed by the routes that make gas and alcohol, and only
    /// those. The event the clock builds from this field names sucrose,
    /// ethanol and carbon dioxide, so nothing else may be counted into it.
    pub sucrose_moles: f64,
    pub ethanol_moles: f64,
    pub carbon_dioxide_moles: f64,
    pub active_yeast_grams: f64,
    /// Lactic acid deposited by the two lactic routes.
    pub lactic_acid_moles: f64,
    /// Acetic acid deposited by the acetic route.
    pub acetic_acid_moles: f64,
    /// Milk lactose taken out of conserved unresolved material, in grams.
    pub unresolved_lactose_grams: f64,
}

/// One culture's contribution, already reduced to a first-order rate.
struct ActiveCulture {
    metabolism: CultureMetabolism,
    rate_per_second: f64,
    active_grams: f64,
}

pub fn advance(vessel: &mut Vessel, seconds: f64) -> Option<FermentationStep> {
    if seconds <= 0.0 || vessel.liquid_volume().0 <= 1e-9 {
        return None;
    }
    let cultures = active_cultures(vessel);
    if cultures.is_empty() {
        return None;
    }

    let mut step = FermentationStep {
        sucrose_moles: 0.0,
        ethanol_moles: 0.0,
        carbon_dioxide_moles: 0.0,
        active_yeast_grams: 0.0,
        lactic_acid_moles: 0.0,
        acetic_acid_moles: 0.0,
        unresolved_lactose_grams: 0.0,
    };
    for culture in &cultures {
        let extent = 1.0 - (-culture.rate_per_second * seconds).exp();
        if extent <= 0.0 {
            continue;
        }
        match culture.metabolism {
            CultureMetabolism::Alcoholic => {
                let moles = consume_sucrose(vessel, extent);
                if moles > 0.0 {
                    step.sucrose_moles += moles;
                    step.ethanol_moles += 4.0 * moles;
                    step.carbon_dioxide_moles += 4.0 * moles;
                    step.active_yeast_grams += culture.active_grams;
                    deposit(vessel, ETHANOL, 4.0 * moles, Phase::Aqueous);
                    deposit(vessel, CARBON_DIOXIDE, 4.0 * moles, Phase::Gas);
                }
            }
            CultureMetabolism::Homolactic => {
                // Table sugar in the beaker, and the lactose the milk
                // recipe conserves as unresolved solids. Both are
                // disaccharides and both give four lactic acids.
                let moles = consume_sucrose(vessel, extent)
                    + consume_unresolved_lactose(vessel, extent, &mut step);
                if moles > 0.0 {
                    step.lactic_acid_moles += 4.0 * moles;
                    deposit(vessel, LACTIC_ACID, 4.0 * moles, Phase::Aqueous);
                }
            }
            CultureMetabolism::Heterolactic => {
                // Deliberately dissolved sucrose only. This route makes
                // gas, the gas is announced through the sucrose count, and
                // there is no such count for the unresolved milk lactose —
                // so a sourdough starter in milk ferments nothing here
                // rather than making carbon dioxide nothing reports.
                let moles = consume_sucrose(vessel, extent);
                if moles > 0.0 {
                    step.sucrose_moles += moles;
                    step.ethanol_moles += 2.0 * moles;
                    step.carbon_dioxide_moles += 2.0 * moles;
                    step.lactic_acid_moles += 2.0 * moles;
                    step.active_yeast_grams += culture.active_grams;
                    deposit(vessel, ETHANOL, 2.0 * moles, Phase::Aqueous);
                    deposit(vessel, CARBON_DIOXIDE, 2.0 * moles, Phase::Gas);
                    deposit(vessel, LACTIC_ACID, 2.0 * moles, Phase::Aqueous);
                }
            }
            CultureMetabolism::Acetic => {
                let moles = consume_ethanol_and_oxygen(vessel, extent);
                if moles > 0.0 {
                    step.acetic_acid_moles += moles;
                    deposit(vessel, ACETIC_ACID, moles, Phase::Aqueous);
                    deposit(vessel, WATER, moles, Phase::Liquid);
                }
            }
        }
    }

    let changed = step.sucrose_moles > 0.0
        || step.lactic_acid_moles > 0.0
        || step.acetic_acid_moles > 0.0
        || step.unresolved_lactose_grams > 0.0;
    if !changed {
        return None;
    }
    vessel.resolved.invalidate();
    Some(step)
}

fn active_cultures(vessel: &Vessel) -> Vec<ActiveCulture> {
    let mut cultures: Vec<ActiveCulture> = Vec::new();
    // No liquid, no concentration, and nothing to divide by. `advance`
    // already refuses a dry vessel; this repeats the guard because the
    // division below is the only place it would show up as a nonsense
    // rather than as a refusal.
    let litres = vessel.liquid_volume().0;
    if litres <= 1e-9 {
        return cultures;
    }
    for portion in &vessel.unresolved_materials {
        let Some(recipe) = material::lookup_versioned(&portion.recipe_id, portion.recipe_version)
        else {
            continue;
        };
        let Some((reference_rate, optimum, width, requires_hydration, metabolism)) =
            recipe.roles.iter().find_map(|role| match role {
                MaterialRole::FermentationCulture {
                    reference_rate_per_second_per_gram_per_litre,
                    optimum_temperature_k,
                    temperature_width_k,
                    requires_hydration,
                    metabolism,
                    ..
                } => Some((
                    *reference_rate_per_second_per_gram_per_litre,
                    *optimum_temperature_k,
                    *temperature_width_k,
                    *requires_hydration,
                    *metabolism,
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
        // The whole of "yoghurt sets overnight on the counter and not in
        // the refrigerator" is this factor: a Gaussian in temperature
        // around the culture's own declared optimum.
        let temperature = (-((vessel.temperature.0 - optimum) / width).powi(2)).exp();
        let active = portion.amount * hydration * temperature;
        if active <= 0.0 {
            continue;
        }
        // THE RATE READS A CONCENTRATION. `reference_rate` belongs to
        // `active` grams dissolved in `REFERENCE_VOLUME_LITRES`, so the
        // dose that drives this vessel is the one that would give this
        // vessel's concentration in that reference volume. Multiplying by
        // the grams alone — which is what this line did until 2026-09-16 —
        // made the extent grow with the batch, and since the extent is
        // spent on a substrate that also grows with the batch, the product
        // went as the SQUARE of the experiment.
        let dose_at_reference_volume = active * REFERENCE_VOLUME_LITRES / litres;
        match cultures
            .iter_mut()
            .find(|existing| existing.metabolism == metabolism)
        {
            Some(existing) => {
                existing.rate_per_second += reference_rate * dose_at_reference_volume;
                existing.active_grams += active;
            }
            None => cultures.push(ActiveCulture {
                metabolism,
                rate_per_second: reference_rate * dose_at_reference_volume,
                active_grams: active,
            }),
        }
    }
    cultures.retain(|culture| culture.rate_per_second > 0.0);
    cultures
}

/// Take a share of the dissolved sucrose and the water the hydrolysis
/// needs, and report the moles actually taken.
fn consume_sucrose(vessel: &mut Vessel, extent: f64) -> f64 {
    let available = phase_moles(vessel, SUCROSE, Phase::Aqueous);
    if available <= 0.0 {
        return 0.0;
    }
    let water = phase_moles(vessel, WATER, Phase::Liquid);
    let moles = (available * extent).min(water).max(0.0);
    if moles <= 0.0 {
        return 0.0;
    }
    withdraw_phase(vessel, SUCROSE, Phase::Aqueous, moles);
    withdraw_phase(vessel, WATER, Phase::Liquid, moles);
    moles
}

/// The milk sugar the whole-milk recipe conserves as unresolved solids.
///
/// It is not a registry species, so it cannot be withdrawn from the
/// inventory; it is withdrawn from the conserved material's own mass
/// instead, and the mass that leaves plus the water the hydrolysis takes
/// is exactly the mass of lactic acid deposited.
fn consume_unresolved_lactose(
    vessel: &mut Vessel,
    extent: f64,
    step: &mut FermentationStep,
) -> f64 {
    let lactose_molar_mass = lactose_equivalent_molar_mass();
    if lactose_molar_mass <= 0.0 {
        return 0.0;
    }
    let mut available_g = 0.0;
    let mut sources: Vec<(usize, f64)> = Vec::new();
    for (index, portion) in vessel.unresolved_materials.iter().enumerate() {
        let Some(share) = crate::enzyme_activity::unresolved_lactose_share(&portion.recipe_id)
        else {
            continue;
        };
        let grams = portion.amount * share;
        if grams > 0.0 {
            available_g += grams;
            sources.push((index, grams));
        }
    }
    if available_g <= 0.0 {
        return 0.0;
    }
    let water = phase_moles(vessel, WATER, Phase::Liquid);
    let wanted_g = available_g * extent;
    let moles = (wanted_g / lactose_molar_mass).min(water).max(0.0);
    if moles <= 0.0 {
        return 0.0;
    }
    let taken_g = moles * lactose_molar_mass;
    for (index, grams) in sources {
        let share = grams / available_g;
        vessel.unresolved_materials[index].amount -= taken_g * share;
    }
    vessel
        .unresolved_materials
        .retain(|portion| portion.amount > 1e-12);
    withdraw_phase(vessel, WATER, Phase::Liquid, moles);
    step.unresolved_lactose_grams += taken_g;
    moles
}

/// Lactose is not an installed species, so its mass is derived from the
/// products of the reaction that consumes it. This keeps the vessel's mass
/// exactly conserved whatever the registry's own rounding is.
fn lactose_equivalent_molar_mass() -> f64 {
    let lactic = molar_mass(LACTIC_ACID);
    let water = molar_mass(WATER);
    if lactic <= 0.0 || water <= 0.0 {
        return 0.0;
    }
    4.0 * lactic - water
}

fn molar_mass(key: &str) -> f64 {
    crate::species::lookup_key(key)
        .map(|data| data.molar_mass)
        .unwrap_or(0.0)
}

/// Ethanol plus dissolved or headspace oxygen, one mole each.
fn consume_ethanol_and_oxygen(vessel: &mut Vessel, extent: f64) -> f64 {
    let ethanol =
        phase_moles(vessel, ETHANOL, Phase::Aqueous) + phase_moles(vessel, ETHANOL, Phase::Liquid);
    let oxygen =
        phase_moles(vessel, OXYGEN, Phase::Gas) + phase_moles(vessel, OXYGEN, Phase::Aqueous);
    if ethanol <= 0.0 || oxygen <= 0.0 {
        return 0.0;
    }
    // Oxygen is the reason a vinegar jar is covered with cloth and not a
    // lid: this is an oxidation, and it stops when the air does.
    let moles = (ethanol * extent).min(oxygen).max(0.0);
    if moles <= 0.0 {
        return 0.0;
    }
    withdraw_across_phases(vessel, ETHANOL, &[Phase::Aqueous, Phase::Liquid], moles);
    withdraw_across_phases(vessel, OXYGEN, &[Phase::Gas, Phase::Aqueous], moles);
    moles
}

fn deposit(vessel: &mut Vessel, species: &str, moles: f64, phase: Phase) {
    if moles <= 0.0 {
        return;
    }
    vessel.deposit(SpeciesId::new(species), Moles(moles), phase);
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

fn withdraw_across_phases(vessel: &mut Vessel, species: &str, phases: &[Phase], moles: f64) {
    let mut remaining = moles;
    for phase in phases {
        if remaining <= 0.0 {
            break;
        }
        let available = phase_moles(vessel, species, *phase);
        let take = available.min(remaining);
        if take > 0.0 {
            withdraw_phase(vessel, species, *phase, take);
            remaining -= take;
        }
    }
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
