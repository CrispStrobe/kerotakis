//! Source-informed stagewise forecasts, frozen before rewriting production Extract.
use kerotakis_core::stock::StockUnit;
use kerotakis_core::*;
use std::cell::{Cell, RefCell};

fn fixture() -> Bench {
    let mut b = Bench::new();
    b.vessels[0].deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
    b.vessels[0].deposit(SpeciesId::new("I2"), Moles(0.001), Phase::Aqueous);
    b.vessels[0].thermal_mode = vessel::ThermalMode::Adiabatic;
    b.vessels[0].temperature = Kelvin(298.6);
    let mut receiver = Vessel::new(VesselId(1), "beaker");
    receiver.thermal_mode = vessel::ThermalMode::Adiabatic;
    b.vessels.push(receiver);
    b
}
fn op(total: f64, stages: u32) -> Operator {
    Operator::Extract {
        from: VesselId(0),
        to: VesselId(1),
        solvent: SpeciesId::new("hexane"),
        total_solvent: Moles(total),
        stages,
    }
}
fn run(b: &mut Bench, total: f64, stages: u32, screen: &dyn SafetyScreen) -> Vec<Event> {
    b.step_with(op(total, stages), &mut SolverStack::new(vec![]), screen)
        .unwrap()
}
fn physical(b: &Bench) -> serde_json::Value {
    let mut value = serde_json::to_value(b).unwrap();
    value.as_object_mut().unwrap().remove("log");
    value
}
fn near(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-8, "{a} != {b}");
}
fn iodine(v: &Vessel) -> f64 {
    v.moles_of(&SpeciesId::new("I2")).0
}
fn hexane(v: &Vessel) -> f64 {
    v.moles_of(&SpeciesId::new("hexane")).0
}
#[derive(Default)]
struct Record(RefCell<Vec<(Vessel, Vessel)>>);
impl SafetyScreen for Record {
    fn assess(&self, _: &Vessel) -> SafetyVerdict {
        SafetyVerdict::Allow
    }
    fn assess_pour(&self, before: &Vessel, after: &Vessel) -> SafetyVerdict {
        self.0.borrow_mut().push((before.clone(), after.clone()));
        SafetyVerdict::Allow
    }
}
fn source_contacts(record: &Record) -> Vec<(Vessel, Vessel)> {
    record
        .0
        .borrow()
        .iter()
        .filter(|(before, after)| after.id == VesselId(0) && hexane(after) > hexane(before))
        .cloned()
        .collect()
}
fn receiver_contacts(record: &Record) -> Vec<(Vessel, Vessel)> {
    record
        .0
        .borrow()
        .iter()
        .filter(|(before, after)| after.id == VesselId(1) && hexane(after) > hexane(before))
        .cloned()
        .collect()
}
fn repeated(mut b: Bench, total: f64, stages: u32) -> Bench {
    for _ in 0..stages {
        run(&mut b, total / f64::from(stages), 1, &PermissiveScreen);
    }
    b
}

#[test]
fn each_source_contact_contains_only_one_fresh_portion() {
    let mut b = fixture();
    let r = Record::default();
    run(&mut b, 0.4, 4, &r);
    let seen = source_contacts(&r);
    assert_eq!(seen.len(), 4);
    for (before, after) in seen {
        near(hexane(&before), 0.0);
        near(hexane(&after), 0.1);
    }
}
#[test]
fn source_contact_thermal_sequence_matches_separate_one_stage_operations() {
    let mut staged = fixture();
    let a = Record::default();
    run(&mut staged, 0.4, 4, &a);
    let mut explicit = fixture();
    let b = Record::default();
    for _ in 0..4 {
        run(&mut explicit, 0.1, 1, &b);
    }
    let actual = source_contacts(&a);
    let expected = source_contacts(&b);
    assert_eq!(actual.len(), expected.len());
    for ((ab, aa), (eb, ea)) in actual.iter().zip(expected.iter()) {
        near(ab.temperature.0, eb.temperature.0);
        near(aa.temperature.0, ea.temperature.0);
        near(iodine(ab), iodine(eb));
        near(iodine(aa), iodine(ea));
    }
}
#[test]
fn donor_temperature_evolves_between_fresh_contacts() {
    let mut b = fixture();
    let r = Record::default();
    run(&mut b, 0.4, 4, &r);
    let seen = source_contacts(&r);
    assert_eq!(seen.len(), 4);
    for pair in seen.windows(2) {
        assert!(pair[1].0.temperature.0 < pair[0].0.temperature.0);
        near(pair[1].0.temperature.0, pair[0].1.temperature.0);
        assert!(iodine(&pair[1].0) < iodine(&pair[0].0));
    }
}
#[test]
fn donor_final_temperature_matches_explicit_fresh_stages() {
    let initial = fixture();
    let explicit = repeated(initial.clone(), 0.4, 4);
    let mut staged = initial;
    run(&mut staged, 0.4, 4, &PermissiveScreen);
    near(
        staged.vessels[0].temperature.0,
        explicit.vessels[0].temperature.0,
    );
}
#[test]
fn collector_accumulates_streams_at_their_actual_stage_temperatures() {
    let mut initial = fixture();
    initial.vessels[1].deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    initial.vessels[1].temperature = Kelvin(350.0);
    let explicit = repeated(initial.clone(), 0.4, 4);
    let mut staged = initial;
    run(&mut staged, 0.4, 4, &PermissiveScreen);
    near(
        staged.vessels[1].temperature.0,
        explicit.vessels[1].temperature.0,
    );
    near(staged.vessels[1].mass().0, explicit.vessels[1].mass().0);
}
#[test]
fn receiver_context_before_is_the_previous_accepted_stage() {
    let mut b = fixture();
    let r = Record::default();
    run(&mut b, 0.4, 4, &r);
    let seen = receiver_contacts(&r);
    assert_eq!(seen.len(), 4);
    for (stage, (before, after)) in seen.iter().enumerate() {
        near(hexane(before), 0.1 * stage as f64);
        near(hexane(after), 0.1 * (stage + 1) as f64);
        if stage > 0 {
            assert_eq!(before.contents, seen[stage - 1].1.contents);
            near(before.temperature.0, seen[stage - 1].1.temperature.0);
        }
    }
    assert_eq!(seen.last().unwrap().1.contents, b.vessels[1].contents);
}
#[test]
fn source_geometry_is_not_a_provisional_all_solvent_dump() {
    struct Geometry;
    impl SafetyScreen for Geometry {
        fn assess(&self, _: &Vessel) -> SafetyVerdict {
            SafetyVerdict::Allow
        }
        fn assess_pour(&self, _: &Vessel, after: &Vessel) -> SafetyVerdict {
            if after.id == VesselId(0) && after.liquid_volume().0 > 0.065 {
                SafetyVerdict::Veto {
                    reason: "source contact overflows bounded geometry".into(),
                }
            } else {
                SafetyVerdict::Allow
            }
        }
    }
    let mut b = fixture();
    let events = run(&mut b, 0.4, 4, &Geometry);
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::Extracted { stages: 4, .. })));
    assert!(!events.iter().any(|e| matches!(e, Event::SafetyVeto { .. })));
}
#[test]
fn unsupported_first_stage_temperature_cannot_be_hidden_by_bulk_cooling() {
    let mut b = fixture();
    b.vessels[0].temperature = Kelvin(300.0);
    b.stock.stock("hexane", 10.0, StockUnit::Mole);
    let before = physical(&b);
    let events = run(&mut b, 4.0, 16, &PermissiveScreen);
    assert_eq!(physical(&b), before);
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::NotYetModeled { .. })));
    assert!(!events.iter().any(|e| matches!(e, Event::Extracted { .. })));
}
#[test]
fn second_source_contact_veto_rolls_back_every_provisional_stage() {
    struct Veto(Cell<usize>);
    impl SafetyScreen for Veto {
        fn assess(&self, _: &Vessel) -> SafetyVerdict {
            SafetyVerdict::Allow
        }
        fn assess_pour(&self, before: &Vessel, after: &Vessel) -> SafetyVerdict {
            if after.id == VesselId(0) && hexane(after) > hexane(before) {
                let n = self.0.get() + 1;
                self.0.set(n);
                if n == 2 {
                    return SafetyVerdict::Veto {
                        reason: "second contact veto".into(),
                    };
                }
            }
            SafetyVerdict::Allow
        }
    }
    let mut b = fixture();
    b.stock.stock("hexane", 1.0, StockUnit::Mole);
    let before = physical(&b);
    let veto = Veto(Cell::new(0));
    let events = run(&mut b, 0.4, 4, &veto);
    assert_eq!(veto.0.get(), 2);
    assert_eq!(physical(&b), before);
    assert!(matches!(&events[..],[Event::SafetyVeto{reason}] if reason=="second contact veto"));
}
#[test]
fn late_collector_veto_rolls_back_source_receiver_stock_and_history_atomically() {
    struct Veto;
    impl SafetyScreen for Veto {
        fn assess(&self, _: &Vessel) -> SafetyVerdict {
            SafetyVerdict::Allow
        }
        fn assess_pour(&self, _: &Vessel, after: &Vessel) -> SafetyVerdict {
            if after.id == VesselId(1) && hexane(after) > 0.25 {
                SafetyVerdict::Veto {
                    reason: "late collector veto".into(),
                }
            } else {
                SafetyVerdict::Allow
            }
        }
    }
    let mut b = fixture();
    b.stock.stock("hexane", 1.0, StockUnit::Mole);
    let before = physical(&b);
    let events = run(&mut b, 0.4, 4, &Veto);
    assert_eq!(physical(&b), before);
    assert!(matches!(&events[..], [Event::SafetyVeto { .. }]));
    assert_eq!(b.log.len(), 1);
}
#[test]
fn provisional_warning_from_first_stage_does_not_survive_later_veto() {
    struct WarningThenVeto(Cell<usize>);
    impl SafetyScreen for WarningThenVeto {
        fn assess(&self, _: &Vessel) -> SafetyVerdict {
            SafetyVerdict::Allow
        }
        fn assess_pour(&self, before: &Vessel, after: &Vessel) -> SafetyVerdict {
            if after.id == VesselId(0) && hexane(after) > hexane(before) {
                let n = self.0.get() + 1;
                self.0.set(n);
                if n == 2 {
                    return SafetyVerdict::Veto {
                        reason: "later veto".into(),
                    };
                }
                return SafetyVerdict::Warn {
                    severity: Severity::Danger,
                    rule: "first-contact".into(),
                    hazard: "provisional".into(),
                    real_world: "test".into(),
                };
            }
            SafetyVerdict::Allow
        }
    }
    let mut b = fixture();
    let before = physical(&b);
    let events = run(&mut b, 0.4, 4, &WarningThenVeto(Cell::new(0)));
    assert_eq!(physical(&b), before);
    assert!(matches!(&events[..], [Event::SafetyVeto { .. }]));
}
#[test]
fn successful_stage_warning_is_reported_once_without_losing_stages() {
    struct Warn;
    impl SafetyScreen for Warn {
        fn assess(&self, _: &Vessel) -> SafetyVerdict {
            SafetyVerdict::Warn {
                severity: Severity::Danger,
                rule: "repeated-warning".into(),
                hazard: "test".into(),
                real_world: "test".into(),
            }
        }
    }
    let mut b = fixture();
    let events = run(&mut b, 0.4, 4, &Warn);
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e,Event::HazardWarning{rule,..} if rule=="repeated-warning"))
            .count(),
        1
    );
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::Extracted { stages: 4, .. })));
}
#[test]
fn equal_temperature_control_preserves_prior_repeated_yield() {
    let mut initial = fixture();
    initial.vessels[0].temperature = Kelvin::STANDARD;
    let expected = repeated(initial.clone(), 0.4, 4);
    let mut actual = initial;
    run(&mut actual, 0.4, 4, &PermissiveScreen);
    near(iodine(&actual.vessels[0]), iodine(&expected.vessels[0]));
    near(iodine(&actual.vessels[1]), iodine(&expected.vessels[1]));
    near(actual.vessels[0].temperature.0, Kelvin::STANDARD.0);
}
#[test]
fn every_species_and_fresh_solvent_stock_remains_conserved() {
    let mut b = fixture();
    b.stock.stock("hexane", 1.0, StockUnit::Mole);
    run(&mut b, 0.4, 4, &PermissiveScreen);
    near(iodine(&b.vessels[0]) + iodine(&b.vessels[1]), 0.001);
    near(hexane(&b.vessels[0]), 0.0);
    near(hexane(&b.vessels[1]), 0.4);
    near(b.vessels[0].moles_of(&SpeciesId::new("water")).0, 2.0);
    near(b.stock.remaining("hexane").unwrap().amount, 0.6);
}
#[test]
fn stagewise_collector_carries_one_fresh_lot_per_contact() {
    let mut b = fixture();
    run(&mut b, 0.4, 4, &PermissiveScreen);
    let lots: Vec<_> = b.vessels[1]
        .lots
        .iter()
        .filter(|l| l.source.as_deref() == Some("fresh extracting solvent"))
        .collect();
    assert_eq!(lots.len(), 4);
    for lot in lots {
        near(lot.moles.0, 0.1);
    }
}
#[test]
fn save_reload_between_extractions_preserves_continuation() {
    let mut direct = fixture();
    run(&mut direct, 0.2, 2, &PermissiveScreen);
    let mut resumed: Bench =
        serde_json::from_value(serde_json::to_value(&direct).unwrap()).unwrap();
    run(&mut direct, 0.2, 2, &PermissiveScreen);
    run(&mut resumed, 0.2, 2, &PermissiveScreen);
    assert_eq!(physical(&direct), physical(&resumed));
    let expected = repeated(fixture(), 0.4, 4);
    near(
        direct.vessels[0].temperature.0,
        expected.vessels[0].temperature.0,
    );
    near(
        direct.vessels[1].temperature.0,
        expected.vessels[1].temperature.0,
    );
}
#[test]
fn underflowed_positive_stage_portion_is_atomic_before_engine_calls() {
    struct NoScreen;
    impl SafetyScreen for NoScreen {
        fn assess(&self, _: &Vessel) -> SafetyVerdict {
            panic!("invalid stage amount must refuse before safety")
        }
    }
    let mut b = fixture();
    let before = serde_json::to_value(&b).unwrap();
    assert!(b
        .step_with(
            op(f64::from_bits(1), 2),
            &mut SolverStack::new(vec![]),
            &NoScreen
        )
        .is_err());
    assert_eq!(serde_json::to_value(&b).unwrap(), before);
}
#[test]
fn excessive_stage_count_refuses_without_unbounded_work() {
    struct NoScreen;
    impl SafetyScreen for NoScreen {
        fn assess(&self, _: &Vessel) -> SafetyVerdict {
            panic!("unbounded stage count must refuse before safety")
        }
    }
    let mut b = fixture();
    let before = physical(&b);
    let result = b.step_with(op(0.4, u32::MAX), &mut SolverStack::new(vec![]), &NoScreen);
    assert!(
        result.is_err()
            || result.as_ref().is_ok_and(|events| events
                .iter()
                .any(|e| matches!(e, Event::NotYetModeled { .. })))
    );
    assert_eq!(physical(&b), before);
}
#[test]
fn crystalline_first_contact_capacity_is_not_the_whole_solvent_capacity() {
    let mut b = fixture();
    b.vessels[0].contents.retain(|p| p.species.0 != "I2");
    b.vessels[0].deposit(SpeciesId::new("I2"), Moles(0.0005), Phase::Solid);
    b.vessels[0].temperature = Kelvin::STANDARD;
    let before = physical(&b);
    let events = run(&mut b, 0.1, 4, &PermissiveScreen);
    assert_eq!(physical(&b), before);
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::NotYetModeled { .. })));
    assert!(!events.iter().any(|e| matches!(e, Event::Extracted { .. })));
}
#[test]
fn later_stage_veto_does_not_create_an_absent_receiver() {
    let mut b = fixture();
    b.vessels.pop();
    let before = physical(&b);
    struct Veto(Cell<usize>);
    impl SafetyScreen for Veto {
        fn assess(&self, _: &Vessel) -> SafetyVerdict {
            SafetyVerdict::Allow
        }
        fn assess_pour(&self, before: &Vessel, after: &Vessel) -> SafetyVerdict {
            if after.id == VesselId(0) && hexane(after) > hexane(before) {
                let n = self.0.get() + 1;
                self.0.set(n);
                if n == 2 {
                    return SafetyVerdict::Veto {
                        reason: "later".into(),
                    };
                }
            }
            SafetyVerdict::Allow
        }
    }
    let events = run(&mut b, 0.4, 4, &Veto(Cell::new(0)));
    assert_eq!(physical(&b), before);
    assert!(matches!(&events[..], [Event::SafetyVeto { .. }]));
}
