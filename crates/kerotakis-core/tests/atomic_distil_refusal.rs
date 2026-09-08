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
