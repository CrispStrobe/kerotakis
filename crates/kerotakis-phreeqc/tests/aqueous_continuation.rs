#![cfg(feature = "engine")]
//! Integration controls for the thermal continuation primitive. The artificial
//! material transform is a test callback, not a production reaction route.
use kerotakis_core::aqueous_continuation::{
    continue_without_reaction_heat, whole_step_heat_available, TrustedAqueousReadback,
};
use kerotakis_core::vessel::StepStart;
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

#[test]
fn real_native_wrapper_exposes_shared_mechanical_acceptance() {
    let mut wrapper =
        DisplacementEquilibrator::wrapping(Box::new(PhreeqcEquilibrator::new().unwrap()));
    let report = kerotakis_phreeqc::continuation_acceptance::run_continuation_acceptance(
        &mut wrapper,
        false,
    );
    assert!(
        report.passed(),
        "{}",
        serde_json::to_string_pretty(&report).unwrap()
    );
}

fn release_peroxide(candidate: &mut Vessel) -> Result<Vec<Event>, SolveError> {
    let peroxide = candidate.moles_of(&SpeciesId::new("H2O2")).0;
    candidate.withdraw(&SpeciesId::new("H2O2"), Moles(peroxide));
    candidate.deposit(SpeciesId::new("water"), Moles(peroxide), Phase::Liquid);
    let species = SpeciesId::new("O2");
    let moles = Moles(peroxide / 2.0);
    Ok(vec![if candidate.retain_gas(species.clone(), moles) {
        Event::GasContained {
            vessel: candidate.id,
            species,
            moles,
        }
    } else {
        Event::GasEvolved {
            vessel: candidate.id,
            species,
            moles,
        }
    }])
}

#[test]
fn concrete_continuation_replays_exported_cache_without_native_solves() {
    let mut live = PhreeqcEquilibrator::new().unwrap();
    let mut initial = Vessel::new(VesselId(0), "cache continuation control");
    initial.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
    initial.deposit(SpeciesId::new("HCl"), Moles(0.005), Phase::Liquid);
    initial.deposit(SpeciesId::new("H2O2"), Moles(0.002), Phase::Liquid);
    initial.step_start = Some(StepStart::capture(&initial));
    let (_, receipt) = live.characterize_current(&mut initial).unwrap();
    let mut expected = initial.clone();
    let expected_events =
        continue_without_reaction_heat(&mut expected, &receipt, &mut live, release_peroxide)
            .unwrap();
    let exported = live.export_cache();
    assert!(!exported.entries.is_empty());
    // The same serialized cache transport is used by shipped-data clients.
    let bytes = postcard::to_allocvec(&exported).unwrap();
    let mut replay = PhreeqcEquilibrator::new().unwrap();
    replay.import_cache(postcard::from_bytes(&bytes).unwrap());
    let calls = replay.engine_calls();
    let hits = replay.cache_hits();
    let mut actual = initial;
    let events =
        continue_without_reaction_heat(&mut actual, &receipt, &mut replay, release_peroxide)
            .unwrap();
    assert_eq!(
        replay.engine_calls(),
        calls,
        "replay must not solve natively"
    );
    assert!(replay.cache_hits() > hits);
    assert_eq!(format!("{actual:?}"), format!("{expected:?}"));
    assert_eq!(format!("{events:?}"), format!("{expected_events:?}"));
}

#[test]
fn native_readback_and_reentry_preserve_unknown_heat_ownership_and_temperature() {
    for sealed in [false, true] {
        let mut aqueous = PhreeqcEquilibrator::new().unwrap();
        let mut vessel = Vessel::new(VesselId(0), "native continuation control");
        vessel.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
        vessel.deposit(SpeciesId::new("HCl"), Moles(0.005), Phase::Liquid);
        vessel.deposit(SpeciesId::new("H2O2"), Moles(0.002), Phase::Liquid);
        if sealed {
            vessel.headspace = Headspace::Sealed {
                volume: Liters(1.0),
            };
        }
        vessel.step_start = Some(StepStart::capture(&vessel));
        let (_, initial_answer) = aqueous.characterize_current(&mut vessel).unwrap();
        assert!(vessel.solution.as_ref().unwrap().ph < 3.0);
        let original = vessel.step_start.clone();
        let temperature = vessel.temperature;
        let peroxide = vessel.moles_of(&SpeciesId::new("H2O2")).0;
        assert!(peroxide > 0.0019);
        let events = continue_without_reaction_heat(
            &mut vessel,
            &initial_answer,
            &mut aqueous,
            |candidate| {
                candidate.withdraw(&SpeciesId::new("H2O2"), Moles(peroxide));
                candidate.deposit(SpeciesId::new("water"), Moles(peroxide), Phase::Liquid);
                let species = SpeciesId::new("O2");
                let moles = Moles(peroxide / 2.0);
                Ok(vec![if candidate.retain_gas(species.clone(), moles) {
                    Event::GasContained {
                        vessel: candidate.id,
                        species,
                        moles,
                    }
                } else {
                    Event::GasEvolved {
                        vessel: candidate.id,
                        species,
                        moles,
                    }
                }])
            },
        )
        .unwrap();
        assert!(!whole_step_heat_available(&vessel));
        assert_eq!(vessel.step_start, original);
        assert_eq!(vessel.temperature, temperature);
        assert!(vessel.solution.as_ref().unwrap().ph < 3.0);
        assert!(events
            .iter()
            .any(|event| matches!(event, Event::NotYetModeled { .. })));
        for _ in 0..2 {
            let mut wrapped =
                DisplacementEquilibrator::wrapping(Box::new(PhreeqcEquilibrator::new().unwrap()));
            equilibrate_phase_coupled(&mut wrapped, &mut vessel).unwrap();
            assert_eq!(vessel.temperature, temperature);
            assert!(!whole_step_heat_available(&vessel));
        }
        let source = vessel.clone();
        assert!(aqueous
            .mix(&mut vessel, &source, 0.5, &source, 0.5)
            .is_none());
    }
}
