//! Frozen native/stack refusal controls use complete cached output, no native solve.
use super::*;
use kerotakis_core::solve::{SolverRouteOutcome, SolverStack};

fn overdraw_fixture() -> (PhreeqcEquilibrator, Vessel) {
    let mut solver = PhreeqcEquilibrator::new().expect("construct native adapter");
    let mut vessel = Vessel::new(VesselId(41), "preserve full rejected native state");
    vessel.deposit(SpeciesId::new("water"), Moles(55.51), Phase::Liquid);
    vessel
        .solid_solutions
        .push(SolidSolution::aragonite_strontianite(
            "owned carbonate",
            Moles(0.003),
            Moles(0.004),
        ));
    vessel.temperature = Kelvin(303.15);
    vessel.free_proton = 1e-8;
    vessel.free_hydroxide = 1e-6;
    let setup = solver
        .setup_problem(&vessel)
        .expect("supported setup")
        .expect("aqueous problem");
    let mut values = BTreeMap::<String, f64>::new();
    for column in setup
        .problem
        .elements
        .iter()
        .cloned()
        .chain(valence_totals(&setup.problem, setup.db_tag))
    {
        values.insert(column, 0.0);
    }
    for (phase, _, _) in &setup.problem.phases {
        values.insert(phase.clone(), 0.0);
    }
    for (phase, _, _) in &setup.problem.gases {
        values.insert(format!("g_{phase}"), 0.0);
    }
    values.extend([
        ("mass_H2O".into(), setup.problem.kgw),
        ("pH".into(), 7.0),
        ("mu".into(), 0.01),
        ("pe".into(), 4.0),
        // Available typed calcium .003; fabricated final .006 overdraws it.
        ("s_Aragonite".into(), 0.006),
        ("s_Strontianite".into(), 0.004),
    ]);
    let rows = vec![
        values.keys().cloned().collect(),
        values.values().map(ToString::to_string).collect(),
    ];
    solver.cache.insert(
        setup.key,
        Rc::new(CachedSolve {
            rows,
            speciation: Vec::new(),
            saturation: Vec::new(),
            redox_adjusted: false,
            pe_determined: true,
        }),
    );
    (solver, vessel)
}

#[test]
fn native_overdraw_refusal_preserves_complete_serialized_vessel() {
    let (mut solver, mut vessel) = overdraw_fixture();
    let before = serde_json::to_value(&vessel).expect("serialize full input");
    let error = solver
        .equilibrate(&mut vessel)
        .expect_err("native phase overdraw must refuse");
    assert!(
        error.to_string().contains("Ca phase overdraw"),
        "must reach the allocation guard: {error}"
    );
    assert_eq!(
        solver.cache_hits, 1,
        "fixture must use its complete cached answer"
    );
    assert_eq!(serde_json::to_value(&vessel).unwrap(), before);
}

#[test]
fn stack_records_failed_native_route_and_preserves_complete_vessel() {
    let (solver, mut vessel) = overdraw_fixture();
    let before = serde_json::to_value(&vessel).expect("serialize full input");
    let mut stack = SolverStack::new(vec![Box::new(solver)]);
    let events = stack
        .equilibrate(&mut vessel)
        .expect("stack reports route refusal as an event");
    assert_eq!(stack.last_routes.len(), 1);
    assert!(matches!(
        stack.last_routes[0].outcome,
        SolverRouteOutcome::Failed
    ));
    assert!(stack.last_routes[0]
        .reason
        .as_deref()
        .unwrap_or("")
        .contains("Ca phase overdraw"));
    assert!(events.iter().any(|event| matches!(event, Event::SolverFailed { detail, .. } if detail.contains("Ca phase overdraw"))));
    assert_eq!(serde_json::to_value(&vessel).unwrap(), before);
}
