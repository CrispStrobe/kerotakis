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
//! * The rate is the unit-definition rate (20 °C, pH 6.9). No temperature
//!   or pH dependence is claimed inside the source's pH window, and none is
//!   invented: the event says so.
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

/// Whether amylase can digest starch here (enzyme, substrate and liquid
/// water present). Used by the insolubility remark so it does not tell a
/// learner "starch does not react" while the clock is digesting it.
pub fn consumes(vessel: &Vessel, species: &SpeciesId) -> bool {
    species.0 == SUBSTRATE
        && total(vessel, ENZYME) > 0.0
        && total(vessel, SUBSTRATE) > 0.0
        && has_liquid_water(vessel)
}

/// Enzyme milligrams from the ledger, or `None` when the dose has no
/// registry molar mass to convert it.
pub fn enzyme_mg(vessel: &Vessel) -> Option<f64> {
    let data = species::lookup_key(ENZYME)?;
    (data.molar_mass.is_finite() && data.molar_mass > 0.0)
        .then(|| total(vessel, ENZYME) * data.molar_mass * 1000.0)
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
    if seconds <= 0.0
        || total(vessel, ENZYME) <= 0.0
        || total(vessel, SUBSTRATE) <= 0.0
        || !has_liquid_water(vessel)
    {
        return events;
    }
    let Some(mg) = enzyme_mg(vessel) else {
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
    let Some(capacity) = maltose_capacity_moles(mg, seconds) else {
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
