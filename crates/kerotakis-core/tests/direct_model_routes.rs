use kerotakis_core::gas_tests::GasTest;
use kerotakis_core::*;

fn run(
    bench: &mut Bench,
    stack: &mut SolverStack,
    operator: Operator,
) -> Result<Vec<Event>, BenchError> {
    bench.step_with(operator, stack, &PermissiveScreen)
}

#[test]
fn successful_gas_tests_and_pressure_readings_record_their_real_routes() {
    let mut bench = Bench::new();
    let mut stack = SolverStack::new(vec![]);
    run(
        &mut bench,
        &mut stack,
        Operator::Seal {
            vessel: VesselId(0),
            headspace_volume: Liters(0.5),
        },
    )
    .unwrap();
    bench.vessels[0].retain_gas(SpeciesId::new("CO2"), Moles(0.01));

    run(
        &mut bench,
        &mut stack,
        Operator::TestGas {
            vessel: VesselId(0),
            test: GasTest::Limewater,
        },
    )
    .unwrap();
    assert!(stack.last_routes.iter().any(|route| {
        route.solver == "gas-test:limewater"
            && route.kind == SolverRouteKind::Curated
            && matches!(
                route.outcome,
                SolverRouteOutcome::Succeeded { event_count: 1 }
            )
    }));

    run(
        &mut bench,
        &mut stack,
        Operator::Measure {
            vessel: VesselId(0),
            instrument: Instrument::PressureGauge,
        },
    )
    .unwrap();
    assert!(stack.last_routes.iter().any(|route| {
        route.solver == "instrument:pressure-gauge"
            && route.kind == SolverRouteKind::Computed
            && matches!(
                route.outcome,
                SolverRouteOutcome::Succeeded { event_count: 1 }
            )
    }));
}

#[test]
fn curated_reaction_routes_require_a_reaction_result() {
    let mut bench = Bench::new();
    let mut stack = SolverStack::new(vec![]);
    for (species, moles) in [("CH3COOH", 0.02), ("ethanol", 0.02)] {
        bench.vessels[0].deposit(
            SpeciesId::new(species),
            Moles(moles),
            kerotakis_core::species::lookup_key(species)
                .unwrap()
                .standard_phase,
        );
    }

    run(
        &mut bench,
        &mut stack,
        Operator::React {
            vessel: VesselId(0),
            reaction: "esterification".into(),
        },
    )
    .unwrap();
    assert!(stack.last_routes.iter().any(|route| {
        route.solver == "curated-reaction:esterification"
            && route.kind == SolverRouteKind::Curated
            && route.chemistry
            && matches!(
                route.outcome,
                SolverRouteOutcome::Succeeded { event_count: 1 }
            )
    }));

    let mut open_bench = Bench::new();
    let mut open_stack = SolverStack::new(vec![]);
    let events = run(
        &mut open_bench,
        &mut open_stack,
        Operator::TestGas {
            vessel: VesselId(0),
            test: GasTest::Limewater,
        },
    )
    .unwrap();
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::NotYetModeled { .. })));
    assert!(open_stack
        .last_routes
        .iter()
        .all(|route| !route.solver.starts_with("gas-test:")));
}

/// `SolverRouteKind` records PROVENANCE — where the numbers a road runs on
/// came from — and not what the answer is made of. Decided and measured on
/// 2026-09-11; the measurement and the argument are in `PLAN.md` under "Is
/// a boil a curated route or a computed one?".
///
/// Both halves of the pair that raised the question are asserted together,
/// because the question only exists as a comparison between them.
///
/// - `phase-routes` is `Curated` and stays `Curated`. It melts, boils,
///   freezes, condenses, sublimes, deposits, dehydrates and softens
///   polymers, and every number it uses for any of that is a curated
///   table — `phase_route.rs`'s latent heats, or the registry's transition
///   temperatures and hydrate pairs. The arithmetic over them does not
///   make the road a computed one; if it did, `curated-reactions` (an
///   extent over a curated stoichiometry) and `reaction-families` (a
///   product set over a curated pattern) would be computed too, and
///   `Curated` would have no members left.
/// - `curated-combustion` is `Computed` NOT because anyone decided that:
///   `CombustionEquilibrator` has no `route_kind` and takes the trait
///   default. It is asserted here as a characterization, so the pair stays
///   visible and so that giving it a declaration is a deliberate act that
///   fails this test first. `PLAN.md` owns that open item, with the eight
///   corpus rows it would move already counted.
#[test]
fn route_kinds_record_where_the_numbers_came_from() {
    assert_eq!(
        kerotakis_core::PhaseRouteEquilibrator.route_kind(),
        SolverRouteKind::Curated,
        "phase-routes reads curated tables for every transition it runs; \
         see PLAN.md, 2026-09-11"
    );
    assert_eq!(
        kerotakis_core::combustion::CombustionEquilibrator.route_kind(),
        SolverRouteKind::Computed,
        "curated-combustion has no route_kind of its own and takes the \
         trait default; PLAN.md owns changing that"
    );
}
