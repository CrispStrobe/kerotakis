//! Bounded curated acid/carbonate gas partition, distinct from film transfer.
//!
//! The classroom acid + bicarbonate reaction is established qualitatively:
//! RSC, Bubble volcanoes, https://edu.rsc.org/resources/bubble-volcanoes/2026.article.
//! This is an explicit endpoint approximation, not an experimental nucleation
//! rate or a claim that the whole supersaturated inventory escapes instantly.
//! Native speciation establishes newly formed neutral CO2; only that parcel
//! may be offered to the already conservative external-carbon owner.

use crate::vessel::SolutionScope;
use crate::{Event, Phase, Vessel};

#[derive(Clone, Copy)]
pub(crate) struct Dose {
    neutral_before: f64,
    capacity: f64,
}

fn pool(v: &Vessel, key: &str) -> f64 {
    v.contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase != Phase::Gas)
        .map(|p| p.moles.0)
        .sum()
}

fn literal_acid(v: &Vessel) -> f64 {
    // HCl's proton_eq is zero to prevent duplicate generic reserve spending.
    // Here the native first pass consumes the acid; this only bounds a fresh
    // dose's possible gas parcel and never withdraws a second acid reserve.
    crate::strong_reserve::CARRIERS
        .iter()
        .filter(|carrier| carrier.conjugate.is_some())
        .map(|carrier| {
            v.contents
                .iter()
                .filter(|p| p.species.0 == carrier.key && carrier.admit.admits(p.phase))
                .map(|p| p.moles.0)
                .sum::<f64>()
        })
        .sum::<f64>()
        + pool(v, "CH3COOH")
}

fn carbonate(v: &Vessel) -> f64 {
    ["NaHCO3", "Na2CO3", "KHCO3", "K2CO3", "HCO3-", "CO3-2"]
        .iter()
        .map(|key| pool(v, key))
        .sum()
}

fn medium(v: &Vessel) -> bool {
    v.unresolved_materials.is_empty() && v.material_objects.is_empty()
        && v.surfaces.is_empty() && v.exchanges.is_empty()
        && v.solid_solutions.is_empty() && v.electrodes.is_empty()
        && v.contents.iter().all(|p| p.phase != Phase::Gas && matches!(p.species.0.as_str(),
            "water" | "Na+" | "K+" | "H+" | "OH-" | "base_equivalents"
            | "Cl-" | "Br-" | "I-" | "NO3-"
            | "HCl" | "HBr" | "HI" | "HNO3"
            | "NaHCO3" | "Na2CO3" | "KHCO3" | "K2CO3"
            | "HCO3-" | "CO3-2" | "CO2(aq)"
            | "CH3COOH" | "CH3COO-" | "NaOAc"))
        // Nitric acid / iodide or bromide is a redox problem, not this
        // acid/base-only endpoint. Do not infer slow spectator ownership.
        && !((pool(v, "NO3-") + pool(v, "HNO3") > 0.0)
            && (pool(v, "I-") + pool(v, "Br-") + pool(v, "HI") + pool(v, "HBr") > 0.0))
}

pub(crate) fn prepare(before: &Vessel, supplied: &Vessel) -> Option<Dose> {
    if !matches!(supplied.headspace, crate::vessel::Headspace::Open)
        || !medium(before)
        || !medium(supplied)
        || before.pending_co2_transfer_mol != 0.0
        || supplied.pending_co2_transfer_mol != 0.0
        || before.solution.as_ref()?.scope != SolutionScope::Complete
    {
        return None;
    }
    let acid_added = (literal_acid(supplied) - literal_acid(before)).max(0.0);
    let carbonate_added = (carbonate(supplied) - carbonate(before)).max(0.0);
    let prior_capacity = literal_acid(before) + before.free_proton;
    let prior_acid = before.solution.as_ref()?.ph
        < crate::bench::carbonic_acid_pk1(before.temperature.0)
        && prior_capacity > crate::OBSERVABLE_MOLES;
    // Plain bicarbonate in water has a small neutral-carbon population too;
    // without an acid reaction it belongs to the finite-time film clock.
    let capacity = if acid_added > 0.0 && carbonate(before) > 0.0 {
        acid_added
    } else if carbonate_added > 0.0 && prior_acid {
        carbonate_added.min(prior_capacity)
    } else {
        return None;
    };
    Some(Dose {
        neutral_before: pool(before, "CO2(aq)"),
        capacity,
    })
}

impl Dose {
    pub(crate) fn extent(self, solved: &Vessel, events: &[Event]) -> Option<f64> {
        let info = solved.solution.as_ref()?;
        if info.scope != SolutionScope::Complete
            || !medium(solved)
            || !info.ionic_strength.is_finite()
            || info.ionic_strength > 1.0
            || solved.pending_co2_transfer_mol != 0.0
            || solved.co2_partial_pressure_atm? <= crate::clock::ATMOSPHERIC_CO2_ATM
            || events
                .iter()
                .any(|e| matches!(e, Event::SolverFailed { .. } | Event::NotYetModeled { .. }))
        {
            return None;
        }
        let already_exported: f64 = events
            .iter()
            .filter_map(|e| match e {
                Event::GasEvolved { species, moles, .. } if species.0 == "CO2" => Some(moles.0),
                _ => None,
            })
            .sum();
        let amount = (pool(solved, "CO2(aq)") - self.neutral_before)
            .max(0.0)
            .min((self.capacity - already_exported).max(0.0));
        (amount.is_finite() && amount > crate::OBSERVABLE_MOLES).then_some(amount)
    }
}

pub(crate) fn notice(vessel: crate::VesselId) -> Event {
    Event::not_modeled(vessel, crate::ops::NotModelledCause::ModelBoundary,
        crate::phrase::Phrase::bare("not-modeled.carbonate-fast-endpoint",
            "this acid/carbonate gas parcel is a bounded curated endpoint from newly formed dissolved CO2. Acid-front nucleation and escape times are not modelled; this is not a calibrated instantaneous gas yield"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Moles, SpeciesId, VesselId};

    #[test]
    fn bounding_a_literal_hcl_dose_does_not_change_generic_reserve_ownership() {
        let mut v = Vessel::new(VesselId(0), "literal acid");
        v.deposit(SpeciesId::new("HCl"), Moles(0.001), Phase::Aqueous);
        assert_eq!(literal_acid(&v), 0.001);
        let carrier = crate::strong_reserve::CARRIERS
            .iter()
            .find(|carrier| carrier.key == "HCl")
            .unwrap();
        assert_eq!(carrier.proton_eq, 0.0);
    }

    #[test]
    fn acid_base_endpoint_does_not_claim_reactive_nitrate_halide_pairs() {
        for (a, b) in [
            ("NO3-", "I-"),
            ("NO3-", "Br-"),
            ("HNO3", "HI"),
            ("HNO3", "HBr"),
        ] {
            let mut v = Vessel::new(VesselId(0), "redox exclusion");
            v.deposit(SpeciesId::new(a), Moles(0.001), Phase::Aqueous);
            v.deposit(SpeciesId::new(b), Moles(0.001), Phase::Aqueous);
            assert!(!medium(&v), "{a} with {b}");
        }
    }

    #[test]
    fn unreviewed_proton_buffers_stay_outside_the_endpoint_domain() {
        for key in ["H2PO4-", "NH4+", "boric_acid", "lactic_acid"] {
            let mut v = Vessel::new(VesselId(0), "buffer exclusion");
            v.deposit(SpeciesId::new(key), Moles(0.001), Phase::Aqueous);
            assert!(!medium(&v), "{key}");
        }
    }
}
