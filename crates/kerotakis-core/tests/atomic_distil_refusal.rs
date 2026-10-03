//! An unsupported still operation cannot trigger a second physical mutation.
use kerotakis_core::*;

#[derive(Default)]
struct CountingMutator {
    calls: Vec<VesselId>,
}

impl Equilibrator for CountingMutator {
    fn name(&self) -> &'static str {
        "deliberately-mutating-test-solver"
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.calls.push(vessel.id);
        vessel.temperature.0 += 1.0;
        vessel.free_proton += 0.001;
        Ok(Vec::new())
    }
}

fn prepared(solute: &str, scale: f64) -> Bench {
    let mut bench = Bench::new();
    bench.vessels[0].deposit(SpeciesId::new("water"), Moles(3.0 * scale), Phase::Liquid);
    bench.vessels[0].deposit(SpeciesId::new(solute), Moles(0.003 * scale), Phase::Aqueous);
    bench.vessels[0].solution = Some(SolutionInfo {
        solvent_activity: None,
        scope: Default::default(),
        solvent_kg: Some(0.054 * scale),
        pe: None,
        redox: vec![],
        ph: 10.0,
        ionic_strength: 0.001,
        species: vec![],
        provenance: None,
    });
    let mut receiver = Vessel::new(VesselId(1), "receiver");
    receiver.deposit(SpeciesId::new("water"), Moles(scale), Phase::Liquid);
    bench.vessels.push(receiver);
    bench
}

fn operation(fraction: Option<f64>, energy: Option<Joules>) -> Operator {
    Operator::Distil {
        from: VesselId(0),
        to: VesselId(1),
        fraction,
        energy,
        stages: 1,
    }
}

fn physical(bench: &Bench) -> serde_json::Value {
    let mut state = serde_json::to_value(bench).unwrap();
    state.as_object_mut().unwrap().remove("log");
    state
}

#[test]
fn unsupported_distillation_preserves_the_entire_bench_except_its_diagnostic_log() {
    for scale in [0.1, 1.0, 10.0] {
        for op in [
            operation(Some(0.2), None),
            operation(None, Some(Joules(500.0 * scale))),
        ] {
            let mut bench = prepared("NH3", scale);
            let mut solver = CountingMutator::default();
            let before = physical(&bench);
            let previous_log = bench.log.len();
            let events = bench.step_with(op, &mut solver, &PermissiveScreen).unwrap();
            assert_eq!(physical(&bench), before);
            assert!(solver.calls.is_empty());
            assert_eq!(bench.log.len(), previous_log + 1);
            assert_eq!(
                serde_json::to_value(&bench.log.last().unwrap().events).unwrap(),
                serde_json::to_value(&events).unwrap()
            );
            assert_eq!(events.len(), 1);
            assert!(matches!(
                events[0],
                Event::NotYetModeled {
                    cause: ops::NotModelledCause::ModelBoundary,
                    ..
                }
            ));
        }
    }
}

#[test]
fn supported_distillation_still_settles_both_affected_vessels() {
    for solute in ["ethanol", "methanol"] {
        let mut bench = prepared(solute, 1.0);
        let mut solver = CountingMutator::default();
        let events = bench
            .step_with(operation(Some(0.2), None), &mut solver, &PermissiveScreen)
            .unwrap();
        assert_eq!(solver.calls, [VesselId(0), VesselId(1)]);
        assert!(
            events.iter().any(|e| matches!(e, Event::Distilled { .. })),
            "{events:?}"
        );
        assert!(bench.vessels[1]
            .contents
            .iter()
            .any(|p| p.species.0 == solute && p.moles.0 > 0.0));
        assert_eq!(bench.log.len(), 1);
    }
}

#[test]
fn intermediate_binary_fit_boundary_refuses_the_complete_cut_without_settling() {
    for stages in [1, 2] {
        let mut bench = Bench::new();
        bench.vessels[0].deposit(SpeciesId::new("water"), Moles(0.5), Phase::Liquid);
        bench.vessels[0].deposit(SpeciesId::new("ethanol"), Moles(0.5), Phase::Liquid);
        bench.vessels.push(Vessel::new(VesselId(1), "receiver"));
        let before = physical(&bench);
        let mut solver = CountingMutator::default();
        let events = bench
            .step_with(
                Operator::Distil {
                    from: VesselId(0),
                    to: VesselId(1),
                    fraction: Some(0.1),
                    energy: None,
                    stages,
                },
                &mut solver,
                &PermissiveScreen,
            )
            .unwrap();
        assert_eq!(physical(&bench), before);
        assert!(solver.calls.is_empty());
        assert!(!events
            .iter()
            .any(|event| matches!(event, Event::Distilled { .. })));
        assert!(events.iter().any(|event| matches!(event, Event::NotYetModeled { cause: ops::NotModelledCause::ModelBoundary, what, .. } if what.contains("No cut was transferred"))));
    }
}

#[test]
fn trace_condensate_that_receiver_cannot_retain_refuses_without_mutation() {
    for stages in [1, 4] {
        let mut bench = Bench::new();
        bench.vessels[0].deposit(SpeciesId::new("water"), Moles(1e-20), Phase::Liquid);
        bench.vessels[0].deposit(SpeciesId::new("ethanol"), Moles(1.0), Phase::Liquid);
        let mut receiver = Vessel::new(VesselId(1), "receiver");
        receiver.deposit(SpeciesId::new("water"), Moles(0.2), Phase::Liquid);
        bench.vessels.push(receiver);
        let before = physical(&bench);
        let mut solver = CountingMutator::default();
        let events = bench
            .step_with(
                Operator::Distil {
                    from: VesselId(0),
                    to: VesselId(1),
                    fraction: Some(0.01),
                    energy: None,
                    stages,
                },
                &mut solver,
                &PermissiveScreen,
            )
            .unwrap();
        assert_eq!(physical(&bench), before);
        assert!(solver.calls.is_empty());
        assert!(events.iter().any(|e| matches!(
            e,
            Event::NotYetModeled {
                cause: ops::NotModelledCause::ModelBoundary,
                ..
            }
        )));
        assert!(!events.iter().any(|e| matches!(e, Event::Distilled { .. })));
    }
}

fn held(bench: &Bench, vessel: usize, species: &str) -> f64 {
    bench.vessels[vessel]
        .contents
        .iter()
        .filter(|p| p.species.0 == species)
        .map(|p| p.moles.0)
        .sum()
}

#[test]
fn paired_trace_cut_into_empty_receiver_preserves_its_component_budget() {
    for stages in [1, 4] {
        let mut bench = Bench::new();
        bench.vessels[0].deposit(SpeciesId::new("water"), Moles(1e-20), Phase::Liquid);
        bench.vessels[0].deposit(SpeciesId::new("ethanol"), Moles(1.0), Phase::Liquid);
        bench.vessels.push(Vessel::new(VesselId(1), "receiver"));
        let mut solver = CountingMutator::default();
        let events = bench
            .step_with(
                Operator::Distil {
                    from: VesselId(0),
                    to: VesselId(1),
                    fraction: Some(0.01),
                    energy: None,
                    stages,
                },
                &mut solver,
                &PermissiveScreen,
            )
            .unwrap();
        assert_eq!(solver.calls, [VesselId(0), VesselId(1)]);
        assert!(events.iter().any(|e| matches!(e, Event::Distilled { .. })));
        for (name, initial) in [("water", 1e-20), ("ethanol", 1.0)] {
            assert!(held(&bench, 0, name) > 0.0 && held(&bench, 1, name) > 0.0);
            assert!(
                ((held(&bench, 0, name) + held(&bench, 1, name)) / initial - 1.0).abs() < 1e-10
            );
        }
    }
}

#[test]
fn receiver_loss_rounding_and_overflow_refuse_before_any_settlement() {
    for (name, amount, existing, fraction) in [
        ("water", 1e-20, 0.2, 0.01),
        ("methanol", 1e-20, 0.2, 0.01),
        ("water", 0.75 * f64::EPSILON, 1.0, 1.0),
        ("ethanol", 1e306, f64::MAX, 0.01),
    ] {
        let mut bench = Bench::new();
        bench.vessels[0].deposit(SpeciesId::new(name), Moles(amount), Phase::Liquid);
        let mut receiver = Vessel::new(VesselId(1), "receiver");
        receiver.deposit(SpeciesId::new(name), Moles(existing), Phase::Liquid);
        bench.vessels.push(receiver);
        let before = physical(&bench);
        let mut solver = CountingMutator::default();
        let events = bench
            .step_with(
                operation(Some(fraction), None),
                &mut solver,
                &PermissiveScreen,
            )
            .unwrap();
        assert_eq!(physical(&bench), before);
        assert!(solver.calls.is_empty());
        assert!(events
            .iter()
            .any(|e| matches!(e, Event::NotYetModeled { what, .. }
            if what.contains("receiver") && what.contains("No cut was transferred"))));
        assert!(!events.iter().any(|e| matches!(e, Event::Distilled { .. })));
    }
}

#[test]
fn representable_addition_into_populated_receiver_remains_supported() {
    let mut bench = Bench::new();
    bench.vessels[0].deposit(SpeciesId::new("water"), Moles(1e-6), Phase::Liquid);
    let mut receiver = Vessel::new(VesselId(1), "receiver");
    receiver.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    bench.vessels.push(receiver);
    let mut solver = CountingMutator::default();
    let events = bench
        .step_with(operation(Some(1.0), None), &mut solver, &PermissiveScreen)
        .unwrap();
    assert_eq!(solver.calls, [VesselId(0), VesselId(1)]);
    assert!(events.iter().any(|e| matches!(e, Event::Distilled { .. })));
    assert_eq!(held(&bench, 0, "water"), 0.0);
    assert!(((held(&bench, 1, "water") - 1.0) / 1e-6 - 1.0).abs() < 1e-8);
}

#[test]
fn dissolved_receiver_stock_cannot_hide_a_positive_trace_condensate() {
    for name in ["ethanol", "methanol"] {
        let mut bench = Bench::new();
        bench.vessels[0].deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
        bench.vessels[0].deposit(SpeciesId::new(name), Moles(1e-20), Phase::Aqueous);
        let mut receiver = Vessel::new(VesselId(1), "receiver");
        receiver.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
        receiver.deposit(SpeciesId::new(name), Moles(0.2), Phase::Aqueous);
        bench.vessels.push(receiver);
        let before = physical(&bench);
        let mut solver = CountingMutator::default();
        let events = bench
            .step_with(operation(Some(1e-4), None), &mut solver, &PermissiveScreen)
            .unwrap();
        assert_eq!(physical(&bench), before);
        assert!(solver.calls.is_empty());
        assert!(events
            .iter()
            .any(|e| matches!(e, Event::NotYetModeled { what, .. }
            if what.contains("receiver") && what.contains("No cut was transferred"))));
    }
}
