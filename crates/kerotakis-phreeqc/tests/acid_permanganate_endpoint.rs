#![cfg(feature = "engine")]
//! Actual native wrapper/cache expectations; no rate or calorimetry calibration.
use kerotakis_core::aqueous_continuation::TrustedAqueousReadback;
use kerotakis_core::*;
use kerotakis_phreeqc::{acid_endpoint_acceptance, PhreeqcEquilibrator};

#[test]
fn actual_native_wrapper_commits_open_and_sealed_endpoints() {
    let mut wrapper =
        DisplacementEquilibrator::wrapping(Box::new(PhreeqcEquilibrator::new().unwrap()));
    let report = acid_endpoint_acceptance::run_wrapper(&mut wrapper, false);
    assert!(
        report.passed(),
        "{}",
        serde_json::to_string_pretty(&report).unwrap()
    );
}
#[test]
fn actual_exported_endpoint_questions_replay_without_engine_calls() {
    let mut native = PhreeqcEquilibrator::new().unwrap();
    let original = acid_endpoint_acceptance::run_aqueous(&mut native);
    assert!(
        original.passed(),
        "{}",
        serde_json::to_string_pretty(&original).unwrap()
    );
    let cache = native.export_cache();
    let bytes = postcard::to_allocvec(&cache).unwrap();
    let mut replay = PhreeqcEquilibrator::new().unwrap();
    replay.import_cache(postcard::from_bytes(&bytes).unwrap());
    let calls = replay.engine_calls();
    let repeated = acid_endpoint_acceptance::run_aqueous(&mut replay);
    assert!(
        repeated.passed(),
        "{}",
        serde_json::to_string_pretty(&repeated).unwrap()
    );
    assert_eq!(
        replay.engine_calls(),
        calls,
        "the exported questions must answer without native solves"
    );
}
#[test]
fn incomplete_bare_reagent_problem_retains_the_ordinary_adapter_answer() {
    let mut initial = Vessel::new(VesselId(0), "incomplete stock boundary");
    initial.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
    initial.deposit(SpeciesId::new("MnO4-"), Moles(0.002), Phase::Aqueous);
    initial.deposit(SpeciesId::new("H2O2"), Moles(0.005), Phase::Liquid);
    let mut direct = initial.clone();
    let mut ordinary = PhreeqcEquilibrator::new().unwrap();
    let expected = ordinary.equilibrate(&mut direct).unwrap();
    let mut actual = initial;
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    let events = displacement::over(&mut solver, &mut actual).unwrap();
    assert_eq!(format!("{actual:?}"), format!("{direct:?}"));
    for event in expected {
        assert!(events
            .iter()
            .any(|e| format!("{e:?}") == format!("{event:?}")));
    }
    assert!(!events
        .iter()
        .any(|e| matches!(e, Event::ReactionOccurred { .. })));
}
#[test]
fn optional_characterization_preserves_solvent_only_scope() {
    let mut v = Vessel::new(VesselId(0), "ordinary partial solvent answer");
    v.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
    v.deposit(SpeciesId::new("H2O2"), Moles(0.005), Phase::Liquid);
    let mut ordinary = v.clone();
    let mut direct = PhreeqcEquilibrator::new().unwrap();
    let expected = direct.equilibrate(&mut ordinary).unwrap();
    let mut optional = PhreeqcEquilibrator::new().unwrap();
    let (events, receipt) = optional.characterize_optional_current(&mut v).unwrap();
    assert_eq!(format!("{v:?}"), format!("{ordinary:?}"));
    assert_eq!(format!("{events:?}"), format!("{expected:?}"));
    assert!(
        receipt.is_none(),
        "an unspeciated neutral spectator is not a complete answer"
    );
}

#[test]
fn complete_stock_orderings_reach_the_endpoint_through_the_standard_stack() {
    let stocks = [("H2SO4", 0.006), ("KMnO4", 0.002), ("H2O2", 0.005)];
    for order in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        let mut bench = Bench::new();
        let mut stack = SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
            DisplacementEquilibrator::wrapping(Box::new(PhreeqcEquilibrator::new().unwrap())),
        )]));
        let mut all = vec![];
        for (key, n) in std::iter::once(("water", 5.55)).chain(order.map(|i| stocks[i])) {
            let events = bench
                .step_with(
                    Operator::Add {
                        vessel: VesselId(0),
                        species: SpeciesId::new(key),
                        moles: Moles(n),
                        at: None,
                    },
                    &mut stack,
                    &kerotakis_safety::ReactiveGroupScreen,
                )
                .unwrap();
            all.extend(events);
        }
        let v = bench.vessel(VesselId(0)).unwrap();
        assert!(
            all.iter().any(
                |e| matches!(e,Event::ReactionOccurred{equation,..} if equation == kerotakis_core::curated::REACTIONS.iter()
                .find(|r| r.post_aqueous == Some(kerotakis_core::curated::PostAqueousEndpoint::AcidicPermanganatePeroxide))
                .unwrap().equation)
            ),
            "{order:?}: {all:?}"
        );
        assert!(
            v.moles_of(&SpeciesId::new("Mn+2")).0 > 0.0019,
            "{order:?}: {v:?}"
        );
        assert!(v.solution.as_ref().is_some_and(|s| s.ph.is_finite()));
    }
}
