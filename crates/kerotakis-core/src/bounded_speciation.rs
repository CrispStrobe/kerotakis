//! Bounded salts whose dissolved part is speciated.
//!
//! A soluble solid with a curated room-temperature solubility and no
//! database mineral is bounded by [`crate::solve::saturation_moves`]: only
//! as much of it as the reviewed figure allows moves into solution. Until
//! 2026-11-26 the dissolved part of every such solid then stayed in the
//! ledger as the undissociated formula unit, because the aqueous engine's
//! `Dissolves` role is phase-blind and would have dissolved the whole solid
//! however much of it sat on the bottom.
//!
//! The rows of [`BOUNDED_SPECIATING_SOLIDS`] take the better route the
//! engine's own comment asked for: the SOLID portion is held outside the
//! aqueous solve (exactly like crystals held by the crystallisation
//! ledger), only the dissolved part is handed to the engine, and the
//! saturation pass counts what the engine booked onto ions as already
//! dissolved, so the bound survives the speciation.
//!
//! A row is admitted only when the aqueous engine derives the salt's
//! dissolved form from its formula into a database it routes to, so the
//! readback names real ions (potassium dichromate and chromate: K⁺ and
//! minteq.v4's chromium(VI) species CrO₄²⁻, HCrO₄⁻ and Cr₂O₇²⁻).
//! Adding a salt is adding a row; the mechanism does not change.

use crate::species::{self, Phase};
use crate::vessel::Vessel;

/// Registry keys of bounded solids whose dissolved part is speciated.
pub const BOUNDED_SPECIATING_SOLIDS: &[&str] = &["K2Cr2O7", "K2CrO4"];

/// Whether this registry key is a bounded solid whose dissolved part the
/// aqueous engine speciates.
pub fn speciates(key: &str) -> bool {
    BOUNDED_SPECIATING_SOLIDS.contains(&key)
}

/// Formula units of `key` already dissolved as ions: for each element of
/// the formula other than H and O, the moles of that element carried by
/// dissolved CHARGED species, divided by its count in the formula; the
/// smallest wins (the limiting ion). With no other source of those ions
/// this is exactly what was dissolved; a shared ion from another salt can
/// only make it larger, so the bound is never exceeded through it.
pub fn dissolved_as_ions(vessel: &Vessel, key: &str) -> f64 {
    let Some(formula) =
        species::lookup_key(key).and_then(|d| crate::stoich::parse_formula(d.formula).ok())
    else {
        return 0.0;
    };
    let mut least = f64::INFINITY;
    for (el, n) in &formula.counts {
        if el == "H" || el == "O" || *n <= 0.0 {
            continue;
        }
        let carried: f64 = vessel
            .contents
            .iter()
            .filter(|p| p.phase == Phase::Aqueous)
            .filter_map(|p| {
                let data = species::lookup(&p.species)?;
                let f = crate::stoich::parse_formula(data.formula).ok()?;
                (f.charge != 0.0).then(|| p.moles.0 * f.counts.get(el).copied().unwrap_or(0.0))
            })
            .sum();
        least = least.min(carried / n);
    }
    if least.is_finite() {
        least.max(0.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Moles, SpeciesId, VesselId};

    #[test]
    fn every_row_is_a_bounded_registry_solid() {
        for key in BOUNDED_SPECIATING_SOLIDS {
            let d = species::lookup_key(key).unwrap_or_else(|| panic!("{key} registered"));
            assert_eq!(d.standard_phase, Phase::Solid, "{key}");
            assert!(d.aqueous_solubility_g_per_100_ml.is_some(), "{key}");
        }
    }

    #[test]
    fn the_limiting_ion_counts_formula_units() {
        let mut v = Vessel::new(VesselId(0), "beaker");
        v.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
        v.deposit(SpeciesId::new("K+"), Moles(0.004), Phase::Aqueous);
        v.deposit(SpeciesId::new("Cr2O7-2"), Moles(0.0005), Phase::Aqueous);
        v.deposit(SpeciesId::new("HCrO4-"), Moles(0.001), Phase::Aqueous);
        // Cr: 2·0.0005 + 0.001 = 0.002 → 0.001 K2Cr2O7; K: 0.004/2 = 0.002.
        assert!((dissolved_as_ions(&v, "K2Cr2O7") - 0.001).abs() < 1e-15);
        // Nothing dissolved: zero, never negative or infinite.
        let empty = Vessel::new(VesselId(1), "beaker");
        assert_eq!(dissolved_as_ions(&empty, "K2Cr2O7"), 0.0);
    }
}
