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
    [
        "NaHCO3", "Na2CO3", "KHCO3", "K2CO3", "CaCO3", "HCO3-", "CO3-2",
    ]
    .iter()
    .map(|key| pool(v, key))
    .sum()
}

fn medium(v: &Vessel) -> bool {
    v.unresolved_materials.is_empty() && v.material_objects.is_empty()
        && v.surfaces.is_empty() && v.exchanges.is_empty()
        && v.solid_solutions.is_empty() && v.electrodes.is_empty()
        && v.contents.iter().all(|p| p.phase != Phase::Gas && matches!(p.species.0.as_str(),
            "water" | "Na+" | "K+" | "Ca+2" | "H+" | "OH-" | "base_equivalents"
            | "Cl-" | "Br-" | "I-" | "NO3-"
            | "HCl" | "HBr" | "HI" | "HNO3"
            | "NaHCO3" | "Na2CO3" | "KHCO3" | "K2CO3" | "CaCO3"
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

fn refused(detail: &str) -> crate::SolveError {
    crate::SolveError::NotConverged {
        solver: "carbonate-endpoint".into(),
        reason: Some(Box::new(crate::phrase::Phrase::bare(
            "solver-reason.continuation.answer-incomplete",
            "The trusted adapter did not supply a finite complete aqueous answer.",
        ))),
        detail: detail.into(),
    }
}

fn fresh_readback(
    vessel: &mut Vessel,
    aqueous: &mut dyn crate::aqueous_continuation::TrustedAqueousReadback,
) -> Result<Vec<Event>, crate::SolveError> {
    // Invalidate before entering even an opted-in adapter. A no-op cannot
    // inherit the previous complete characterization or certify another pose.
    vessel.solution = None;
    vessel.resolved.invalidate();
    vessel.free_proton = 0.0;
    vessel.free_hydroxide = 0.0;
    let question = vessel.clone();
    let (events, receipt) = aqueous.characterize_current(vessel)?;
    if !receipt.certifies(&question, vessel)
        || question.id != vessel.id
        || question.label != vessel.label
        || question.headspace != vessel.headspace
        || question.thermal_mode != vessel.thermal_mode
        || question.heat_input != vessel.heat_input
        || question.aqueous_heat != vessel.aqueous_heat
        || question.zero_time_dose != vessel.zero_time_dose
        || question.co2_film_boundary != vessel.co2_film_boundary
        || !medium(vessel)
        || !vessel
            .solution
            .as_ref()
            .is_some_and(|info| info.scope == SolutionScope::Complete && info.provenance.is_some())
        || events.iter().any(|event| {
            matches!(
                event,
                Event::SolverFailed { .. } | Event::NotYetModeled { .. }
            )
        })
        || !crate::delta::StateDelta::validate_state(vessel).is_empty()
        || !crate::delta::StateDelta::validate_conservation(&question, vessel, &events, 1e-10)
            .is_empty()
        || (crate::ledger::ConservedLedger::from_vessel(&question).charge
            - crate::ledger::ConservedLedger::from_vessel(vessel).charge)
            .abs()
            > 1e-10
    {
        return Err(refused(
            "carbonate endpoint lacked a fresh complete conservative readback",
        ));
    }
    Ok(events)
}

pub(crate) fn run(
    vessel: &mut Vessel,
    dose: Dose,
    aqueous: &mut dyn crate::aqueous_continuation::TrustedAqueousReadback,
    first_events: &[Event],
) -> Result<Option<Vec<Event>>, crate::SolveError> {
    // Avoid an extra characterization unless the first result identifies a
    // possible endpoint. Admission is then repeated on a real fresh receipt.
    if dose.extent(vessel, first_events).is_none() {
        return Ok(None);
    }
    let mut candidate = vessel.clone();
    candidate.step_start = Some(crate::vessel::StepStart::capture(vessel));
    candidate.heat_input = None;
    let mut events = fresh_readback(&mut candidate, aqueous)?;
    let mut observed = first_events.to_vec();
    observed.extend(events.iter().cloned());
    let Some(amount) = dose.extent(&candidate, &observed) else {
        return Ok(None);
    };
    candidate.step_start = Some(crate::vessel::StepStart::capture(&candidate));
    candidate.pending_co2_transfer_mol = -amount;
    candidate.zero_time_dose = true;
    candidate.co2_film_boundary = true;
    let mut transferred = fresh_readback(&mut candidate, aqueous)?;
    let exported: f64 = transferred
        .iter()
        .filter_map(|event| match event {
            Event::GasEvolved { species, moles, .. } if species.0 == "CO2" => Some(moles.0),
            _ => None,
        })
        .sum();
    if candidate.pending_co2_transfer_mol != 0.0
        || !exported.is_finite()
        || (exported - amount).abs() > 1e-12
        || transferred.iter().any(|event| {
            matches!(event,
            Event::GasAbsorbed { species, .. } if species.0 == "CO2")
        })
    {
        return Err(refused(
            "carbonate endpoint did not spend and announce its exact owned parcel",
        ));
    }
    events.append(&mut transferred);
    events.push(notice(vessel.id));
    *vessel = candidate;
    Ok(Some(events))
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
    fn calcite_and_its_native_calcium_readback_share_the_bounded_domain() {
        let mut before = boundary_fixture();
        before.deposit(SpeciesId::new("CaCO3"), Moles(0.01), Phase::Solid);
        before.deposit(SpeciesId::new("Ca+2"), Moles(0.0001), Phase::Aqueous);
        let mut supplied = before.clone();
        supplied.deposit(SpeciesId::new("HCl"), Moles(0.001), Phase::Aqueous);
        let dose = prepare(&before, &supplied).expect("calcite acid dose admitted");
        let mut solved = before.clone();
        solved.deposit(SpeciesId::new("CO2(aq)"), Moles(0.0003), Phase::Aqueous);
        assert!((dose.extent(&solved, &[]).unwrap() - 0.0003).abs() < 1e-12);
        assert!(
            prepare(&before, &before).is_none(),
            "calcite alone is not an acid dose"
        );
        for unreviewed in ["MgCO3", "Ca(OH)2", "gypsum"] {
            let mut other = supplied.clone();
            other.deposit(SpeciesId::new(unreviewed), Moles(0.001), Phase::Solid);
            assert!(prepare(&before, &other).is_none(), "{unreviewed}");
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

    // Synthetic adapters here test refusal/atomicity only. These arbitrary
    // coordinates are never asserted as a chemical prediction or native run.
    fn boundary_fixture() -> Vessel {
        let mut v = Vessel::new(VesselId(0), "boundary fixture");
        v.deposit(SpeciesId::new("water"), Moles(10.0), Phase::Liquid);
        v.deposit(SpeciesId::new("CO2(aq)"), Moles(0.0005), Phase::Aqueous);
        v.solution = Some(
            serde_json::from_value(serde_json::json!({
                "ph": 6.5, "ionic_strength": 0.01,
                "provenance": {"engine": "test double", "dataset": "test double",
                    "model": "test double", "routing": "test double"}
            }))
            .unwrap(),
        );
        v.co2_partial_pressure_atm = Some(0.1);
        v.co2_film_boundary = true;
        v.zero_time_dose = true;
        v
    }

    enum BrokenReadback {
        NoOp,
        Partial,
        StaleReceipt,
        ClearsParcel,
    }

    impl crate::aqueous_continuation::TrustedAqueousReadback for BrokenReadback {
        fn characterize_current(
            &mut self,
            v: &mut Vessel,
        ) -> Result<
            (
                Vec<Event>,
                crate::aqueous_continuation::CurrentAqueousAnswer,
            ),
            crate::SolveError,
        > {
            use crate::aqueous_continuation::CurrentAqueousAnswer;
            let input = v.clone();
            if matches!(self, Self::NoOp) {
                return CurrentAqueousAnswer::attest_computed(&input, v).map(|r| (vec![], r));
            }
            v.solution = boundary_fixture().solution;
            if matches!(self, Self::Partial) {
                v.solution.as_mut().unwrap().scope = SolutionScope::SolventOnly;
            }
            if matches!(self, Self::ClearsParcel) {
                v.pending_co2_transfer_mol = 0.0;
            }
            let receipt = CurrentAqueousAnswer::attest_computed(&input, v)?;
            if matches!(self, Self::StaleReceipt) {
                v.solution.as_mut().unwrap().ph += 0.1;
            }
            Ok((vec![], receipt))
        }
    }

    #[test]
    fn stale_partial_noop_and_unannounced_parcel_readbacks_never_commit() {
        for mut adapter in [
            BrokenReadback::NoOp,
            BrokenReadback::Partial,
            BrokenReadback::StaleReceipt,
            BrokenReadback::ClearsParcel,
        ] {
            let mut v = boundary_fixture();
            let original = format!("{v:?}");
            let dose = Dose {
                neutral_before: 0.0,
                capacity: 0.001,
            };
            assert!(run(&mut v, dose, &mut adapter, &[]).is_err());
            assert_eq!(
                format!("{v:?}"),
                original,
                "a rejected readback changed the original vessel"
            );
        }
    }
}
