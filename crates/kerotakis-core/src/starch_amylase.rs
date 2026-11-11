//! Alpha-amylase digestion of starch over bench time (2026-11-11,
//! batch3-diverse:en-001).
//!
//! The defect: the curated row `2 (C6H10O5) + H2O ->[amylase] C12H22O11`
//! completed the whole digestion the instant the enzyme was added, so a
//! "sample after 20 min" protocol and a "sample after 10 s" protocol read
//! the same. The row is retired; digestion now advances only while bench
//! time passes (`clock::EnzymeClock`), at a rate taken from a stated
//! enzyme UNIT DEFINITION and a stated specific activity:
//!
//! * Unit definition (Sigma-Aldrich A3176, alpha-amylase from porcine
//!   pancreas, Type VI-B; Bernfeld-type reducing-sugar assay): "One unit
//!   will liberate 1.0 mg of maltose from starch in 3 min at pH 6.9 at
//!   20 °C." Read on the product page (sigmaaldrich.com/product/sigma/a3176,
//!   mirrored at scientificlabs.com A3176-5MU, opened 2026-11-11).
//! * Specific activity: "≥5 units/mg solid" (same page, Specifications).
//!   The page's title says ≥10; the lower specification is used, so the
//!   modelled digestion is a LOWER BOUND on what a real lot does.
//! * Same page: "The pH range for activity is 5.5 to 8.0, with the pH
//!   optimum at 7" and "Chloride ions are necessary for activity and
//!   stability."
//!
//! Model (one generic mechanism, no fitted constants):
//! `maltose mg/s = (enzyme mg × 5 U/mg) × (1.0 mg / 180 s)`, zero order in
//! starch (the unit is an initial rate in substrate excess), capped by the
//! starch present. Each maltose consumes two anhydroglucose repeat units
//! and one water, the same aggregate stoichiometry the retired row used.
//!
//! Declared boundaries:
//! * The rate is the unit-definition rate (20 °C, pH 6.9). No Arrhenius
//!   speed-up of the catalytic rate is claimed, and none is invented.
//!
//! Heat inactivation (2026-11-11, curiosity bio-030 "Does boiling destroy
//! amylase activity?" — the bench used to digest starch at 100 °C):
//!
//! * Source: Miłek J., "Determination of Activation Energies and the
//!   Optimum Temperatures of Hydrolysis of Starch by α-Amylase from Porcine
//!   Pancreas", Molecules 26 (2021) 4117, doi:10.3390/molecules26144117
//!   (open access, PMC8306296). Read in full via Europe PMC: Eqs. (2)-(7)
//!   treat deactivation as first order, `da/dt = -k_d a`,
//!   `k_d(T) = B exp(-E_d/RT)`, `β = B t_a exp(-E_d/(R T_opt))`, R = 8.315
//!   J/(mol K). Table 1 row 1a (Sigma-Aldrich porcine pancreatic enzyme,
//!   pH 6.9 sodium phosphate, t_a = 3 min — the same supplier, pH and assay
//!   time as the A3176 unit definition above; data of Akhond et al., Appl.
//!   Biochem. Biotechnol. 180 (2016) 954-968, doi:10.1007/s12010-016-2145-1):
//!   T_opt = 311.06 K, E_d = 164.9 kJ/mol, β = 1.46. Hence
//!   `k_d(T) = (β/t_a) exp[(E_d/R)(1/T_opt - 1/T)]` per minute.
//! * Applied only on the falling limb, T > T_opt, which is where Eq. (6)
//!   attributes the measured activity loss to deactivation. Below T_opt
//!   the same fit is dominated by the catalytic Arrhenius term and its
//!   k_d is not constrained by the data, so no inactivation is claimed
//!   there (37 °C digestion is unchanged).
//! * The fitted data (Fig. 1a) end at 343 K. Above 343.15 K no extrapolation
//!   is claimed: the enzyme is booked as completely inactivated at once
//!   (the fit's own k_d there already exceeds 180 per minute, a half-life
//!   under a quarter of a second). Corroboration only, different organism:
//!   wheat α-amylase loses activity completely around 85 °C (Singh,
//!   Shandilya, Kundu, Kayastha, PLoS ONE 10 (2015) e0129203,
//!   doi:10.1371/journal.pone.0129203, Fig. 3A), and wheat amylase is the
//!   more heat-stable of the two.
//! * Irreversible: the inactivated moles are kept on the vessel
//!   (`Vessel::amylase_inactive_moles`); cooling does not restore them and
//!   freshly added enzyme is active. Not carried by a pour (declared).
//! * Starch present with no active enzyme left is reported with
//!   `not-modeled.amylase-heat-inactivated` and nothing is digested.
//! * A characterised solution with pH outside 5.5-8.0 is outside the
//!   source; digestion is then refused with
//!   `not-modeled.amylase-outside-unit-conditions` instead of run.
//! * An enzyme dose that cannot be converted to milligrams (no registry
//!   molar mass) is refused with `not-modeled.amylase-dose-not-convertible`.
//! * The chloride requirement is disclosed, not gated: the unit's assay
//!   buffer supplies it, and the bench does not know a chloride threshold.

use crate::ops::Event;
use crate::phrase::{Phrase, Slot};
use crate::species::{self, Phase};
use crate::vessel::Vessel;
use crate::{Moles, SpeciesId};

pub const ENZYME: &str = "amylase";
pub const SUBSTRATE: &str = "starch";
pub const PRODUCT: &str = "maltose";

/// Sigma-Aldrich A3176 specification, ≥5 units per mg solid (lower bound).
pub const SPECIFIC_ACTIVITY_U_PER_MG: f64 = 5.0;
/// One unit liberates 1.0 mg maltose in 3 min (180 s).
pub const UNIT_MALTOSE_MG: f64 = 1.0;
pub const UNIT_SECONDS: f64 = 180.0;
/// Source pH window for activity (same page).
pub const PH_RANGE: (f64, f64) = (5.5, 8.0);

pub const EQUATION: &str = "2 (C₆H₁₀O₅) + H₂O →[amylase] C₁₂H₂₂O₁₁";
/// Miłek 2021 (doi:10.3390/molecules26144117) Table 1 row 1a: optimum
/// temperature of the Sigma porcine pancreatic enzyme, K.
pub const INACTIVATION_T_OPT_K: f64 = 311.06;
/// Same row: activation energy of deactivation, J/mol (164.9 kJ/mol).
pub const INACTIVATION_E_D_J_PER_MOL: f64 = 164_900.0;
/// Same row: dimensionless β = B t_a exp(-E_d/(R T_opt)).
pub const INACTIVATION_BETA: f64 = 1.46;
/// Same row: assay time t_a, minutes.
pub const INACTIVATION_ASSAY_MINUTES: f64 = 3.0;
/// Gas constant as the paper uses it, J/(mol K).
pub const INACTIVATION_GAS_CONSTANT: f64 = 8.315;
/// Highest temperature in the fitted data (Fig. 1a), K. Above it the enzyme
/// is booked as completely inactivated rather than extrapolated.
pub const COMPLETE_LOSS_ABOVE_K: f64 = 343.15;
pub const INACTIVATION_SOURCE: &str = "Miłek, Molecules 26 (2021) 4117, doi:10.3390/molecules26144117, Table 1 row 1a (Sigma porcine pancreatic α-amylase, data of Akhond et al. 2016, doi:10.1007/s12010-016-2145-1): first-order deactivation, T_opt 311.06 K, E_d 164.9 kJ/mol, β 1.46, t_a 3 min";

pub const SOURCE: &str = "Sigma-Aldrich A3176 alpha-amylase (porcine pancreas, Type VI-B): 1 U liberates 1.0 mg maltose from starch in 3 min at pH 6.9, 20 °C; ≥5 U/mg solid";

fn total(vessel: &Vessel, key: &str) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == key)
        .map(|p| p.moles.0)
        .sum()
}

fn has_liquid_water(vessel: &Vessel) -> bool {
    vessel
        .contents
        .iter()
        .any(|p| p.species.0 == "water" && p.phase == Phase::Liquid && p.moles.0 > 0.0)
}

/// First-order heat-inactivation rate constant, per second, at `t_k`.
///
/// `Some(0.0)` at or below the fitted optimum (no inactivation claimed),
/// `Some(f64::INFINITY)` above `COMPLETE_LOSS_ABOVE_K` (complete loss), the
/// Miłek 2021 Eq. (5)/(7) value in between. `None` for a non-finite input.
pub fn inactivation_rate_per_s(t_k: f64) -> Option<f64> {
    if !t_k.is_finite() || t_k <= 0.0 {
        return None;
    }
    if t_k <= INACTIVATION_T_OPT_K {
        return Some(0.0);
    }
    if t_k > COMPLETE_LOSS_ABOVE_K {
        return Some(f64::INFINITY);
    }
    let per_minute = INACTIVATION_BETA / INACTIVATION_ASSAY_MINUTES
        * (INACTIVATION_E_D_J_PER_MOL / INACTIVATION_GAS_CONSTANT
            * (1.0 / INACTIVATION_T_OPT_K - 1.0 / t_k))
            .exp();
    Some(per_minute / 60.0)
}

/// Active (not heat-inactivated) amylase, mol.
pub fn active_enzyme_moles(vessel: &Vessel) -> f64 {
    (total(vessel, ENZYME) - vessel.amylase_inactive_moles.unwrap_or(0.0)).max(0.0)
}

/// Whether amylase can digest starch here (enzyme, substrate and liquid
/// water present). Used by the insolubility remark so it does not tell a
/// learner "starch does not react" while the clock is digesting it.
pub fn consumes(vessel: &Vessel, species: &SpeciesId) -> bool {
    species.0 == SUBSTRATE
        && active_enzyme_moles(vessel) > 0.0
        && total(vessel, SUBSTRATE) > 0.0
        && has_liquid_water(vessel)
}

/// Active enzyme milligrams, or `None` when the dose has no registry molar
/// mass to convert it.
pub fn enzyme_mg(vessel: &Vessel) -> Option<f64> {
    let data = species::lookup_key(ENZYME)?;
    (data.molar_mass.is_finite() && data.molar_mass > 0.0)
        .then(|| active_enzyme_moles(vessel) * data.molar_mass * 1000.0)
}

/// Book first-order inactivation over `seconds` at the vessel temperature.
/// Returns the enzyme-weighted effective digestion time: the seconds of
/// full activity the surviving enzyme delivers, `(1 - e^{-k t}) / k`.
fn inactivate(vessel: &mut Vessel, seconds: f64) -> f64 {
    let active = active_enzyme_moles(vessel);
    let k = inactivation_rate_per_s(vessel.temperature.0).unwrap_or(0.0);
    if active <= 0.0 || k <= 0.0 {
        return seconds;
    }
    let (lost, effective) = if k.is_infinite() {
        (active, 0.0)
    } else {
        let surviving = (-k * seconds).exp();
        (active * (1.0 - surviving), (1.0 - surviving) / k)
    };
    vessel.amylase_inactive_moles = Some(vessel.amylase_inactive_moles.unwrap_or(0.0) + lost);
    effective
}

fn heat_inactivated_event(vessel: &Vessel) -> Event {
    let celsius = vessel.temperature.0 - 273.15;
    Event::not_modeled(
        vessel.id,
        crate::ops::NotModelledCause::ModelBoundary,
        Phrase::new(
            "not-modeled.amylase-heat-inactivated",
            "starch digestion by amylase at {celsius} °C: the enzyme has been irreversibly heat-inactivated (first-order loss of activity above its 38 °C optimum, complete above 70 °C), so no digestion is booked and the starch is left as it is",
            vec![("celsius".to_string(), Slot::number(format!("{celsius:.0}")))],
        ),
    )
}

/// Maltose (mol) the stated activity liberates in `seconds`, before the
/// starch cap.
pub fn maltose_capacity_moles(enzyme_mg: f64, seconds: f64) -> Option<f64> {
    let maltose = species::lookup_key(PRODUCT)?;
    if maltose.molar_mass <= 0.0 {
        return None;
    }
    let mg = enzyme_mg * SPECIFIC_ACTIVITY_U_PER_MG * UNIT_MALTOSE_MG * seconds / UNIT_SECONDS;
    Some(mg / 1000.0 / maltose.molar_mass)
}

/// Advance the digestion over `seconds`. Returns the events to report.
pub fn advance(vessel: &mut Vessel, seconds: f64) -> Vec<Event> {
    let mut events = Vec::new();
    if seconds <= 0.0 || total(vessel, ENZYME) <= 0.0 || !has_liquid_water(vessel) {
        return events;
    }
    // Inactivation is booked whether or not starch is present: a boiled
    // enzyme stays dead when the starch arrives later.
    // The dose is read BEFORE this interval's inactivation: the effective
    // time below already weights it by the surviving fraction.
    let had_active = active_enzyme_moles(vessel) > 0.0;
    let dose_mg = enzyme_mg(vessel);
    let effective_seconds = inactivate(vessel, seconds);
    if total(vessel, SUBSTRATE) <= 0.0 {
        return events;
    }
    if !had_active || effective_seconds <= 0.0 {
        events.push(heat_inactivated_event(vessel));
        return events;
    }
    let Some(mg) = dose_mg else {
        events.push(Event::not_modeled(
            vessel.id,
            crate::ops::NotModelledCause::NoReviewedDatum,
            Phrase::bare(
                "not-modeled.amylase-dose-not-convertible",
                "starch digestion by amylase: the enzyme amount cannot be converted to milligrams, so its activity in units is unknown and no digestion is booked",
            ),
        ));
        return events;
    };
    if let Some(ph) = vessel
        .solution
        .as_ref()
        .map(|s| s.ph)
        .filter(|ph| ph.is_finite())
    {
        if ph < PH_RANGE.0 || ph > PH_RANGE.1 {
            events.push(Event::not_modeled(
                vessel.id,
                crate::ops::NotModelledCause::ModelBoundary,
                Phrase::new(
                    "not-modeled.amylase-outside-unit-conditions",
                    "starch digestion by amylase at pH {ph}: the cited activity holds only between pH 5.5 and 8.0, so no digestion rate is claimed and the starch is left as it is",
                    vec![("ph".to_string(), Slot::number(format!("{ph:.2}")))],
                ),
            ));
            return events;
        }
    }
    let Some(capacity) = maltose_capacity_moles(mg, effective_seconds) else {
        return events;
    };
    let starch = total(vessel, SUBSTRATE);
    let water = vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == "water" && p.phase == Phase::Liquid)
        .map(|p| p.moles.0)
        .sum::<f64>();
    let extent = capacity.min(starch / 2.0).min(water);
    if extent <= 0.0 {
        return events;
    }
    // Starch is the whole repeat-unit pool, whatever phase it sits in.
    let mut take = 2.0 * extent;
    for phase in [Phase::Aqueous, Phase::Solid, Phase::Liquid] {
        if take <= 0.0 {
            break;
        }
        take -= vessel
            .withdraw_phase(&SpeciesId::new(SUBSTRATE), Moles(take), phase)
            .0;
    }
    let digested = 2.0 * extent - take.max(0.0);
    let extent = digested / 2.0;
    vessel.withdraw_phase(&SpeciesId::new("water"), Moles(extent), Phase::Liquid);
    vessel.deposit(SpeciesId::new(PRODUCT), Moles(extent), Phase::Aqueous);
    events.push(Event::ReactionOccurred {
        vessel: vessel.id,
        equation: EQUATION.to_string(),
    });
    events
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digestion_vessel(enzyme_g: f64) -> Vessel {
        let mut v = Vessel::new(crate::VesselId(0), "beaker");
        v.deposit(SpeciesId::new("water"), Moles(2.767), Phase::Liquid);
        // 0.20 g starch as anhydroglucose repeat units (162.14 g/mol).
        v.deposit(
            SpeciesId::new(SUBSTRATE),
            Moles(0.20 / 162.14),
            Phase::Solid,
        );
        let m = species::lookup_key(ENZYME).unwrap().molar_mass;
        v.deposit(SpeciesId::new(ENZYME), Moles(enzyme_g / m), Phase::Aqueous);
        v
    }

    #[test]
    fn the_unit_definition_sets_the_rate() {
        // 10 mg enzyme × 5 U/mg = 50 U → 50 mg maltose per 3 min.
        let moles = maltose_capacity_moles(10.0, 180.0).unwrap();
        let mg = moles * species::lookup_key(PRODUCT).unwrap().molar_mass * 1000.0;
        assert!((mg - 50.0).abs() < 1e-9, "{mg}");
    }

    #[test]
    fn digestion_is_partial_after_a_short_wait_and_complete_later() {
        let mut v = digestion_vessel(0.010);
        let starch0 = total(&v, SUBSTRATE);
        advance(&mut v, 60.0);
        let after_minute = total(&v, SUBSTRATE);
        assert!(after_minute > 0.0 && after_minute < starch0);
        advance(&mut v, 1140.0);
        assert!(total(&v, SUBSTRATE) < 1e-15);
        assert!((total(&v, PRODUCT) - starch0 / 2.0).abs() < 1e-12);
    }

    #[test]
    fn digestion_conserves_carbon_hydrogen_oxygen() {
        let mut v = digestion_vessel(0.002);
        let count = |v: &Vessel, el: &str| -> f64 {
            v.contents
                .iter()
                .map(|p| {
                    let f =
                        crate::stoich::parse_formula(species::lookup(&p.species).unwrap().formula)
                            .unwrap();
                    p.moles.0 * f.counts.get(el).copied().unwrap_or(0.0)
                })
                .sum()
        };
        let before: Vec<f64> = ["C", "H", "O"].iter().map(|e| count(&v, e)).collect();
        advance(&mut v, 300.0);
        for (i, el) in ["C", "H", "O"].iter().enumerate() {
            assert!((count(&v, el) - before[i]).abs() < 1e-12, "{el}");
        }
    }

    #[test]
    fn no_time_no_digestion() {
        let mut v = digestion_vessel(0.010);
        let starch0 = total(&v, SUBSTRATE);
        assert!(advance(&mut v, 0.0).is_empty());
        assert_eq!(total(&v, SUBSTRATE), starch0);
    }
}
