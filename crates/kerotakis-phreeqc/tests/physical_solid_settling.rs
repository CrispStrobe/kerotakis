//! Actual aqueous readback must not turn historical stock lots into particles.
#![cfg(feature = "engine")]
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(DisplacementEquilibrator::wrapping(Box::new(
            PhreeqcEquilibrator::new().unwrap(),
        ))),
        Box::new(StateEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn step(bench: &mut Bench, stack: &mut SolverStack, operator: Operator) -> Vec<Event> {
    let events = bench.step_with(operator, stack, &PermissiveScreen).unwrap();
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::SolverFailed { .. })),
        "{events:?}"
    );
    events
}

fn add(bench: &mut Bench, stack: &mut SolverStack, species: &str, amount: f64) {
    step(
        bench,
        stack,
        Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new(species),
            moles: Moles(amount),
            at: None,
        },
    );
}

fn solid(vessel: &Vessel, species: &str) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == species && p.phase == Phase::Solid)
        .map(|p| p.moles.0)
        .sum()
}

#[test]
fn dissolved_copper_sulfate_lot_does_not_settle_after_real_zinc_displacement() {
    let mut solver = stack();
    let mut bench = Bench::new();
    add(&mut bench, &mut solver, "water", 5.534276991396059);
    add(&mut bench, &mut solver, "CuSO4", 0.001);
    assert!(bench.vessels[0].solution.is_some());
    assert_eq!(solid(&bench.vessels[0], "CuSO4"), 0.0);
    assert!(bench.vessels[0]
        .lots
        .iter()
        .any(|lot| lot.species.0 == "CuSO4" && lot.phase == Phase::Solid));
    add(&mut bench, &mut solver, "Zn", 0.001);
    assert!(solid(&bench.vessels[0], "Cu") > 0.0009);
    assert_eq!(solid(&bench.vessels[0], "CuSO4"), 0.0);
    let lot_before: Vec<_> = bench.vessels[0]
        .lots
        .iter()
        .filter(|lot| lot.species.0 == "CuSO4")
        .map(|lot| format!("{lot:?}"))
        .collect();
    let events = step(&mut bench, &mut solver, Operator::Wait { seconds: 10.0 });
    assert!(!events.iter().any(|event| matches!(event,
        Event::GravitySettled { separations, .. } if separations.iter().any(|s| s.species.0 == "CuSO4")
    )), "{events:?}");
    let lot_after: Vec<_> = bench.vessels[0]
        .lots
        .iter()
        .filter(|lot| lot.species.0 == "CuSO4")
        .map(|lot| format!("{lot:?}"))
        .collect();
    assert_eq!(
        lot_after, lot_before,
        "historical stock provenance remains intact"
    );
}

#[test]
fn real_residual_silver_chloride_particulates_still_settle_after_aqueous_readback() {
    let mut solver = stack();
    let mut bench = Bench::new();
    add(&mut bench, &mut solver, "water", 5.55);
    add(&mut bench, &mut solver, "AgCl", 0.007);
    assert!(solid(&bench.vessels[0], "AgCl") > 0.006);
    let events = step(&mut bench, &mut solver, Operator::Wait { seconds: 10.0 });
    assert!(events.iter().any(|event| matches!(event,
        Event::GravitySettled { separations, .. } if separations.iter().any(|s| s.species.0 == "AgCl" && s.separated_fraction > 0.0)
    )), "{events:?}");
}
