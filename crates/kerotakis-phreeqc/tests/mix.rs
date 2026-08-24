//! PHREEQC MIX routing: mixing two solved solutions by fraction.

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;
use kerotakis_safety::ReactiveGroupScreen;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(CuratedEquilibrator),
        Box::new(PhreeqcEquilibrator::new().expect("engine")),
        Box::new(HonestyEquilibrator),
    ])
}

fn add(
    bench: &mut Bench,
    stack: &mut SolverStack,
    v: VesselId,
    key: &str,
    moles: f64,
) -> Vec<Event> {
    bench
        .step_with(
            Operator::Add {
                vessel: v,
                species: SpeciesId::new(key),
                moles: Moles(moles),
                at: None,
            },
            stack,
            &ReactiveGroupScreen,
        )
        .expect("add")
}

#[test]
fn mix_two_salt_solutions_conserves_mass() {
    let mut bench = Bench::new();
    let mut stack = stack();
    bench.step(Operator::NewVessel { kind: None }).unwrap(); // v2
    bench.step(Operator::NewVessel { kind: None }).unwrap(); // v3

    add(&mut bench, &mut stack, VesselId(0), "water", 27.75);
    add(&mut bench, &mut stack, VesselId(0), "NaCl", 0.1);

    add(&mut bench, &mut stack, VesselId(1), "water", 27.75);
    add(&mut bench, &mut stack, VesselId(1), "KCl", 0.1);

    let mass_before: f64 = bench.vessels.iter().map(|v| v.mass().0).sum();

    let events = bench
        .step_with(
            Operator::Mix {
                a: VesselId(0),
                b: VesselId(1),
                into: VesselId(2),
                fraction_a: 0.5,
                fraction_b: 0.5,
            },
            &mut stack,
            &ReactiveGroupScreen,
        )
        .expect("mix");

    let mass_after: f64 = bench.vessels.iter().map(|v| v.mass().0).sum();
    assert!(
        (mass_before - mass_after).abs() / mass_before < 1e-3,
        "mass must be conserved: {mass_before} vs {mass_after}"
    );

    assert!(
        events.iter().any(|e| matches!(e, Event::Mixed { .. })),
        "must emit a Mixed event"
    );

    // The mixed solution should be characterized.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::SolutionCharacterized { .. })),
        "mixed solution should be characterized"
    );
}

#[test]
fn mix_acid_and_base_produces_neutral_ph() {
    let mut bench = Bench::new();
    let mut stack = stack();
    bench.step(Operator::NewVessel { kind: None }).unwrap(); // v2
    bench.step(Operator::NewVessel { kind: None }).unwrap(); // v3

    add(&mut bench, &mut stack, VesselId(0), "water", 27.75);
    add(&mut bench, &mut stack, VesselId(0), "HCl", 0.01);

    add(&mut bench, &mut stack, VesselId(1), "water", 27.75);
    add(&mut bench, &mut stack, VesselId(1), "NaOH", 0.01);

    bench
        .step_with(
            Operator::Mix {
                a: VesselId(0),
                b: VesselId(1),
                into: VesselId(2),
                fraction_a: 1.0,
                fraction_b: 1.0,
            },
            &mut stack,
            &ReactiveGroupScreen,
        )
        .expect("mix");

    let v3 = bench.vessel(VesselId(2)).unwrap();
    let ph = v3.solution.as_ref().expect("solution").ph;
    assert!(
        (ph - 7.0).abs() < 0.5,
        "equimolar HCl + NaOH → near-neutral pH, got {ph:.2}"
    );
}

#[test]
fn hard_water_lesson_replays() {
    let mut bench = Bench::new();
    let mut stack = stack();
    bench.step(Operator::NewVessel { kind: None }).unwrap(); // v2
    bench.step(Operator::NewVessel { kind: None }).unwrap(); // v3
    bench.step(Operator::NewVessel { kind: None }).unwrap(); // v4
    bench.step(Operator::NewVessel { kind: None }).unwrap(); // v5

    // Hard water with dissolved calcium and magnesium.
    add(&mut bench, &mut stack, VesselId(2), "water", 27.75);
    add(&mut bench, &mut stack, VesselId(2), "CaCl2", 5e-3);
    add(&mut bench, &mut stack, VesselId(2), "MgSO4", 3e-3);

    // Soft water (just sodium chloride).
    add(&mut bench, &mut stack, VesselId(3), "water", 27.75);
    add(&mut bench, &mut stack, VesselId(3), "NaCl", 10e-3);

    // Mix them.
    let events = bench
        .step_with(
            Operator::Mix {
                a: VesselId(2),
                b: VesselId(3),
                into: VesselId(4),
                fraction_a: 0.5,
                fraction_b: 0.5,
            },
            &mut stack,
            &ReactiveGroupScreen,
        )
        .expect("mix");

    assert!(
        events.iter().any(|e| matches!(e, Event::Mixed { .. })),
        "must emit Mixed event"
    );

    let v5 = bench.vessel(VesselId(4)).unwrap();
    assert!(
        v5.solution.is_some(),
        "mixed solution should be characterized"
    );
}
