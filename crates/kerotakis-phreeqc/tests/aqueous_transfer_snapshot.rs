#![cfg(feature = "engine")]
//! MODEL-008: filtration transports an existing aqueous thermal baseline.
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn stack() -> SolverStack {
    SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
        PhreeqcEquilibrator::new().expect("native engine"),
    )]))
}
fn step(bench: &mut Bench, solver: &mut SolverStack, line: &str) -> Vec<Event> {
    let op = kerotakis_core::script::parse_op(line).unwrap().unwrap();
    let events = bench.step_with(op, solver, &PermissiveScreen).unwrap();
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::SolverFailed { .. })),
        "{line}: {events:?}"
    );
    events
}
fn amount(v: &Vessel, key: &str) -> f64 {
    v.moles_of(&SpeciesId::new(key)).0
}

#[test]
fn unchanged_ab029_obtains_fresh_filtrate_ph_and_preserves_capacity_contrast() {
    let mut bench = Bench::new();
    let mut solver = stack();
    let mut compared = false;
    // Exact originally submitted native script, including observations and
    // rendering directive; parsing skips that directive rather than deleting it.
    for line in include_str!("fixtures/ab029-filter-suspension.lab").lines() {
        let Some(op) = kerotakis_core::script::parse_op(line).unwrap() else {
            continue;
        };
        let filtering = matches!(op, Operator::Filter { .. });
        let before = filtering.then(|| bench.vessel(VesselId(1)).unwrap().clone());
        let events = bench.step_with(op, &mut solver, &PermissiveScreen).unwrap();
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::SolverFailed { .. })),
            "{line}: {events:?}"
        );
        if let Some(before) = before {
            let filtrate = bench.vessel(VesselId(2)).unwrap();
            assert!(
                filtrate.solution.is_some(),
                "filtrate needs a fresh computed answer"
            );
            assert_eq!(
                filtrate.solution.as_ref().unwrap().scope,
                SolutionScope::Complete
            );
            assert!(
                (filtrate.temperature.0 - before.temperature.0).abs() < 1e-6,
                "an unchanged aliquot cannot manufacture the lost hydroxide reference heat"
            );
            assert!(
                (filtrate.solution.as_ref().unwrap().ph - before.solution.as_ref().unwrap().ph)
                    .abs()
                    < 1e-4
            );
            assert!((amount(filtrate, "water") - amount(&before, "water")).abs() < 1e-9);
            assert!(
                amount(filtrate, "Mg(OH)2") < 1e-9,
                "solid reserve remains at source"
            );
            assert!(amount(bench.vessel(VesselId(1)).unwrap(), "Mg(OH)2") > 0.001);
            compared = true;
        }
    }
    assert!(compared);
    let suspension = bench.vessel(VesselId(0)).unwrap();
    let filtrate = bench.vessel(VesselId(2)).unwrap();
    let suspension_ph = suspension.solution.as_ref().unwrap().ph;
    let filtrate_ph = filtrate.solution.as_ref().unwrap().ph;
    assert!(suspension_ph > 8.0 && filtrate_ph < 3.0 && suspension_ph - filtrate_ph > 5.0);
}

#[test]
fn already_solved_basic_aliquot_does_not_create_repeated_cooling_on_pours() {
    let mut bench = Bench::new();
    let mut solver = stack();
    for line in [
        "add v1 water 100mL @ 25C",
        "add v1 NaOH 0.001mol",
        "new",
        "new",
    ] {
        step(&mut bench, &mut solver, line);
    }
    let initial = bench.vessel(VesselId(0)).unwrap().clone();
    step(&mut bench, &mut solver, "decant v1 v2 0.25");
    for id in [VesselId(0), VesselId(1)] {
        let v = bench.vessel(id).unwrap();
        assert!((v.temperature.0 - initial.temperature.0).abs() < 1e-6);
        assert!(
            (v.solution.as_ref().unwrap().ph - initial.solution.as_ref().unwrap().ph).abs() < 1e-4
        );
    }
    step(&mut bench, &mut solver, "filter v2 v3");
    let final_aliquot = bench.vessel(VesselId(2)).unwrap();
    assert!((final_aliquot.temperature.0 - initial.temperature.0).abs() < 1e-6);
    assert!(
        (final_aliquot.solution.as_ref().unwrap().ph - initial.solution.as_ref().unwrap().ph).abs()
            < 1e-4
    );
    let water: f64 = bench.vessels.iter().map(|v| amount(v, "water")).sum();
    let sodium: f64 = bench.vessels.iter().map(|v| amount(v, "Na+")).sum();
    assert!((water - amount(&initial, "water")).abs() < 1e-9);
    assert!((sodium - 0.001).abs() < 1e-10);
}
