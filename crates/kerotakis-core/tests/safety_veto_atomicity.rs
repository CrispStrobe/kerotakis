//! Source-informed refusal forecasts frozen before implementation and execution.
use kerotakis_core::authority::SpillDestination;
use kerotakis_core::spill::SpillCompartment;
use kerotakis_core::*;
use std::cell::Cell;

#[derive(Clone, Copy, Debug)]
enum Case {
    Add,
    Material,
    Decant,
    Mix,
    Filter,
    Discard,
    Recover,
}

#[derive(Default)]
struct Veto(Cell<usize>);
impl SafetyScreen for Veto {
    fn assess(&self, _: &Vessel) -> SafetyVerdict {
        self.0.set(self.0.get() + 1);
        SafetyVerdict::Veto {
            reason: "frozen deterministic veto".into(),
        }
    }
}

#[derive(Default)]
struct CountingMutator(Vec<VesselId>);
impl Equilibrator for CountingMutator {
    fn name(&self) -> &'static str {
        "frozen-veto-mutator"
    }
    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.0.push(vessel.id);
        vessel.temperature.0 += 1.0;
        vessel.free_proton += 0.001;
        Ok(vec![])
    }
}

fn destination() -> SpillDestination {
    SpillDestination::Tray {
        tray: "frozen-veto".into(),
    }
}

fn prepared() -> Bench {
    let mut bench = Bench::new();
    bench
        .vessels
        .push(Vessel::new(VesselId(1), "existing second vessel"));
    bench
        .vessels
        .push(Vessel::new(VesselId(2), "existing receiver"));
    for vessel in &mut bench.vessels {
        vessel.deposit(SpeciesId::new("water"), Moles(4.0), Phase::Liquid);
        vessel.free_proton = 1e-7;
        vessel.solution = Some(SolutionInfo {
            solvent_activity: None,
            scope: Default::default(),
            solvent_kg: Some(0.072),
            pe: None,
            redox: vec![],
            ph: 7.0,
            ionic_strength: 0.001,
            species: vec![],
            provenance: None,
        });
    }
    let mut spill = SpillCompartment::new(destination(), Kelvin::STANDARD);
    spill.contents.push(Portion {
        species: SpeciesId::new("water"),
        moles: Moles(2.0),
        phase: Phase::Liquid,
    });
    spill.sources.push(VesselId(1));
    bench.spills.push(spill);
    bench
}

fn operation(case: Case) -> Operator {
    match case {
        Case::Add => Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new("water"),
            moles: Moles(0.5),
            at: None,
        },
        Case::Material => {
            let recipe = material::lookup("whole_milk", None).expect("registered material");
            Operator::AddMaterial {
                vessel: VesselId(0),
                material: recipe.canonical_key.clone(),
                recipe_id: recipe.id.clone(),
                recipe_version: recipe.version,
                total_amount: 1.0,
                basis: recipe.basis,
                sample_seed: 41,
                at: None,
            }
        }
        Case::Decant => Operator::Decant {
            from: VesselId(0),
            to: VesselId(2),
            fraction: 0.5,
        },
        Case::Mix => Operator::Mix {
            a: VesselId(0),
            b: VesselId(1),
            into: VesselId(2),
            fraction_a: 0.25,
            fraction_b: 0.5,
        },
        Case::Filter => Operator::Filter {
            from: VesselId(0),
            to: VesselId(2),
        },
        Case::Discard => Operator::Discard {
            vessel: VesselId(0),
        },
        Case::Recover => Operator::RecoverSpill {
            destination: destination(),
            to: VesselId(0),
            fraction: 0.5,
        },
    }
}

fn physical(bench: &Bench) -> serde_json::Value {
    let mut state = serde_json::to_value(bench).unwrap();
    state.as_object_mut().unwrap().remove("log");
    state
}

fn refused(case: Case) {
    let mut bench = prepared();
    let before = physical(&bench);
    let mut solver = CountingMutator::default();
    let screen = Veto::default();
    let events = bench
        .step_with(operation(case), &mut solver, &screen)
        .expect("veto is a diagnostic refusal");
    assert!(
        screen.0.get() > 0,
        "{case:?} must actually reach its safety screen"
    );
    assert!(
        solver.0.is_empty(),
        "{case:?} refusal must not run solvers: {:?}",
        solver.0
    );
    assert_eq!(
        physical(&bench),
        before,
        "{case:?} must preserve the complete bench except its refusal log"
    );
    assert_eq!(
        events.len(),
        1,
        "{case:?} must retain only its refusal diagnostic"
    );
    assert!(
        matches!(&events[0], Event::SafetyVeto { reason } if reason == "frozen deterministic veto")
    );
    assert_eq!(bench.log.len(), 1);
    assert_eq!(
        serde_json::to_value(&bench.log[0].events).unwrap(),
        serde_json::to_value(&events).unwrap()
    );
}

fn accepted(case: Case) {
    let mut bench = prepared();
    let before = physical(&bench);
    let mut solver = CountingMutator::default();
    let events = bench
        .step_with(operation(case), &mut solver, &PermissiveScreen)
        .expect("same supported operation accepts without veto");
    assert!(
        !solver.0.is_empty(),
        "{case:?} accepted operation must retain solver settlement"
    );
    assert_ne!(physical(&bench), before);
    assert!(!events
        .iter()
        .any(|event| matches!(event, Event::SafetyVeto { .. })));
    let has_success = events.iter().any(|event| {
        matches!(
            (case, event),
            (Case::Add, Event::Added { .. })
                | (Case::Material, Event::MaterialAdded { .. })
                | (Case::Decant, Event::Transferred { .. })
                | (Case::Mix, Event::Mixed { .. })
                | (Case::Filter, Event::Filtered { .. })
                | (Case::Discard, Event::Discarded { .. })
                | (Case::Recover, Event::SpillRecovered { .. })
        )
    });
    assert!(has_success, "{case:?} retains its accepted operator event");
}

macro_rules! contracts {
    ($refused:ident, $accepted:ident, $case:ident) => {
        #[test]
        fn $refused() {
            refused(Case::$case);
        }
        #[test]
        fn $accepted() {
            accepted(Case::$case);
        }
    };
}
contracts!(
    vetoed_add_preserves_full_state_without_solving,
    accepted_add_still_settles,
    Add
);
contracts!(
    vetoed_material_preserves_full_state_without_solving,
    accepted_material_still_settles,
    Material
);
contracts!(
    vetoed_decant_preserves_full_state_without_solving,
    accepted_decant_still_settles,
    Decant
);
contracts!(
    vetoed_mix_preserves_full_state_without_solving,
    accepted_mix_still_settles,
    Mix
);
contracts!(
    vetoed_filter_preserves_full_state_without_solving,
    accepted_filter_still_settles,
    Filter
);
contracts!(
    vetoed_discard_preserves_full_state_without_solving,
    accepted_discard_still_settles,
    Discard
);
contracts!(
    vetoed_recovery_preserves_full_state_without_solving,
    accepted_recovery_still_settles,
    Recover
);

#[test]
fn accidental_spill_proceeds_with_hazard_despite_veto() {
    let mut bench = prepared();
    let mut solver = CountingMutator::default();
    let screen = Veto::default();
    let events = bench
        .step_with(
            Operator::Spill {
                from: VesselId(0),
                destination: destination(),
                fraction: 0.5,
                replay_seed: 41,
            },
            &mut solver,
            &screen,
        )
        .unwrap();
    assert!(screen.0.get() > 0);
    assert!(
        !solver.0.is_empty(),
        "an actual spill still settles its physical source"
    );
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::SpillHazard { .. })));
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::SpillCreated { .. })));
    assert!(!events
        .iter()
        .any(|event| matches!(event, Event::SafetyVeto { .. })));
    assert_eq!(bench.vessels[0].moles_of(&SpeciesId::new("water")).0, 2.0);
    let spilled: f64 = bench
        .spills
        .iter()
        .filter(|spill| spill.destination == destination())
        .flat_map(|spill| &spill.contents)
        .filter(|portion| portion.species.0 == "water")
        .map(|portion| portion.moles.0)
        .sum();
    assert_eq!(
        spilled, 4.0,
        "existing two moles plus the actual two-mole spill"
    );
}

#[derive(Default)]
struct Warn;
impl SafetyScreen for Warn {
    fn assess(&self, _: &Vessel) -> SafetyVerdict {
        SafetyVerdict::Warn {
            severity: Severity::Caution,
            rule: "frozen-warning".into(),
            hazard: "accepted warning".into(),
            real_world: "frozen test control".into(),
        }
    }
}

fn fresh_receiver(mut bench: Bench) -> Bench {
    bench.vessels.retain(|vessel| vessel.id != VesselId(2));
    bench
}

fn fresh_refused(case: Case) {
    let mut bench = fresh_receiver(prepared());
    let before = physical(&bench);
    let screen = Veto::default();
    let mut solver = CountingMutator::default();
    let events = bench
        .step_with(operation(case), &mut solver, &screen)
        .expect("vetoed transfer remains diagnostic");
    assert!(screen.0.get() > 0);
    assert!(solver.0.is_empty());
    assert_eq!(
        physical(&bench),
        before,
        "{case:?} refusal must not create its proposed receiver"
    );
    assert_eq!(events.len(), 1);
    assert!(matches!(&events[0], Event::SafetyVeto { .. }));
    assert_eq!(bench.log.len(), 1);
}

fn fresh_warned(case: Case) {
    let mut bench = fresh_receiver(prepared());
    let mut solver = CountingMutator::default();
    let events = bench
        .step_with(operation(case), &mut solver, &Warn)
        .expect("warning permits a supported transfer into a new receiver");
    assert!(!solver.0.is_empty());
    let created = events
        .iter()
        .position(
            |event| matches!(event, Event::VesselCreated { vessel } if *vessel == VesselId(2)),
        )
        .expect("accepted transfer creates receiver");
    let warning = events
        .iter()
        .position(
            |event| matches!(event, Event::HazardWarning { rule, .. } if rule == "frozen-warning"),
        )
        .expect("warning retained");
    let success = events
        .iter()
        .position(|event| {
            matches!(
                (case, event),
                (Case::Decant, Event::Transferred { .. }) | (Case::Filter, Event::Filtered { .. })
            )
        })
        .expect("transfer success retained");
    assert!(
        created < warning && warning < success,
        "preserve existing accepted creation/warning/success order"
    );
    assert!(!events
        .iter()
        .any(|event| matches!(event, Event::SafetyVeto { .. })));
    assert_eq!(
        bench
            .vessels
            .iter()
            .filter(|vessel| vessel.id == VesselId(2))
            .count(),
        1
    );
    let receiver = bench
        .vessels
        .iter()
        .find(|vessel| vessel.id == VesselId(2))
        .unwrap();
    assert_eq!(
        receiver.moles_of(&SpeciesId::new("water")).0,
        if matches!(case, Case::Decant) {
            2.0
        } else {
            4.0
        }
    );
}

#[test]
fn fresh_decant_veto_does_not_create_receiver() {
    fresh_refused(Case::Decant);
}
#[test]
fn fresh_filter_veto_does_not_create_receiver() {
    fresh_refused(Case::Filter);
}
#[test]
fn fresh_decant_warning_preserves_creation_and_event_order() {
    fresh_warned(Case::Decant);
}
#[test]
fn fresh_filter_warning_preserves_creation_and_event_order() {
    fresh_warned(Case::Filter);
}
#[test]
fn missing_mix_receiver_keeps_existing_error_and_full_state() {
    let mut bench = fresh_receiver(prepared());
    let before = serde_json::to_value(&bench).unwrap();
    let screen = Veto::default();
    let mut solver = CountingMutator::default();
    let result = bench.step_with(operation(Case::Mix), &mut solver, &screen);
    assert!(matches!(result, Err(BenchError::NoSuchVessel(VesselId(2)))));
    assert_eq!(screen.0.get(), 0);
    assert!(solver.0.is_empty());
    assert_eq!(serde_json::to_value(&bench).unwrap(), before);
}
