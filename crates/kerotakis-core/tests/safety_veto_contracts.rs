//! A veto refuses the command before solver, stock, receiver or spill mutation.
use kerotakis_core::authority::SpillDestination;
use kerotakis_core::*;
#[derive(Default)]
struct Counter(usize);
impl Equilibrator for Counter {
    fn name(&self) -> &'static str {
        "safety-veto-counter"
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.0 += 1;
        v.temperature.0 += 1.0;
        v.free_proton += 0.001;
        Ok(Vec::new())
    }
}
struct Veto;
impl SafetyScreen for Veto {
    fn assess(&self, _: &Vessel) -> SafetyVerdict {
        SafetyVerdict::Veto {
            reason: "frozen veto".to_string(),
        }
    }
}
fn bench() -> Bench {
    let mut b = Bench::new();
    b.vessels[0].deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
    for id in [1, 2] {
        b.vessels.push(Vessel::new(VesselId(id), "receiver"));
    }
    b.vessels[1].deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    b
}
fn physical(b: &Bench) -> serde_json::Value {
    let mut value = serde_json::to_value(b).unwrap();
    value.as_object_mut().unwrap().remove("log");
    value
}
fn refused(mut b: Bench, op: Operator) {
    let before = physical(&b);
    let entries = b.log.len();
    let mut counter = Counter::default();
    let events = b.step_with(op, &mut counter, &Veto).unwrap();
    assert_eq!(physical(&b), before);
    assert_eq!(counter.0, 0);
    assert_eq!(b.log.len(), entries + 1);
    assert_eq!(
        events.len(),
        1,
        "veto must not report physical changes: {events:?}"
    );
    assert!(matches!(&events[0], Event::SafetyVeto { reason } if reason=="frozen veto"));
    assert_eq!(
        serde_json::to_value(&events).unwrap(),
        serde_json::to_value(&b.log.last().unwrap().events).unwrap()
    );
}
#[test]
fn add_veto_is_atomic() {
    refused(
        bench(),
        Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new("water"),
            moles: Moles(0.25),
            at: None,
        },
    );
}
#[test]
fn material_add_veto_is_atomic() {
    refused(
        bench(),
        script::parse_op("add v1 olive_oil 1g").unwrap().unwrap(),
    );
}
#[test]
fn decant_veto_is_atomic() {
    refused(
        bench(),
        Operator::Decant {
            from: VesselId(0),
            to: VesselId(1),
            fraction: 0.5,
        },
    );
}
#[test]
fn decant_veto_removes_automatically_created_receiver() {
    refused(
        bench(),
        Operator::Decant {
            from: VesselId(0),
            to: VesselId(3),
            fraction: 0.5,
        },
    );
}
#[test]
fn mix_veto_is_atomic() {
    refused(
        bench(),
        Operator::Mix {
            a: VesselId(0),
            b: VesselId(1),
            into: VesselId(2),
            fraction_a: 0.5,
            fraction_b: 0.5,
        },
    );
}
#[test]
fn filter_veto_removes_automatically_created_receiver() {
    refused(
        bench(),
        Operator::Filter {
            from: VesselId(0),
            to: VesselId(3),
        },
    );
}
#[test]
fn deliberate_discard_veto_is_atomic() {
    refused(
        bench(),
        Operator::Discard {
            vessel: VesselId(0),
        },
    );
}
#[test]
fn spill_recovery_veto_is_atomic() {
    let mut b = bench();
    let destination = SpillDestination::Bench {
        zone: "audit".to_string(),
    };
    b.step_with(
        Operator::Spill {
            from: VesselId(0),
            destination: destination.clone(),
            fraction: 0.5,
            replay_seed: 1,
        },
        &mut SolverStack::new(vec![]),
        &PermissiveScreen,
    )
    .unwrap();
    refused(
        b,
        Operator::RecoverSpill {
            destination,
            to: VesselId(1),
            fraction: 0.5,
        },
    );
}
#[test]
fn extraction_veto_retains_its_atomic_contract() {
    refused(
        bench(),
        Operator::Extract {
            from: VesselId(0),
            to: VesselId(1),
            solvent: SpeciesId::new("hexane"),
            total_solvent: Moles(0.5),
            stages: 1,
        },
    );
}
#[test]
fn an_accidental_spill_still_happens_with_a_hazard() {
    let mut b = bench();
    let events = b
        .step_with(
            Operator::Spill {
                from: VesselId(0),
                destination: SpillDestination::Bench {
                    zone: "audit".to_string(),
                },
                fraction: 0.5,
                replay_seed: 1,
            },
            &mut SolverStack::new(vec![]),
            &Veto,
        )
        .unwrap();
    assert!(!events.iter().any(|e| matches!(e, Event::SafetyVeto { .. })));
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::SpillHazard { .. })));
    assert_eq!(b.vessels[0].moles_of(&SpeciesId::new("water")).0, 1.0);
    assert_eq!(
        b.spills[0].contents.iter().map(|p| p.moles.0).sum::<f64>(),
        1.0
    );
}
