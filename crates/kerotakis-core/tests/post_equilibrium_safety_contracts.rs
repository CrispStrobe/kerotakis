//! Frozen contracts for a separate post-equilibrium safety acceptance hook.
use kerotakis_core::*;
use std::cell::RefCell;
struct Warm;
impl Equilibrator for Warm {
    fn name(&self) -> &'static str {
        "warm-test"
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        v.temperature = Kelvin(330.0);
        Ok(vec![])
    }
}
struct FinalScreen {
    seen: RefCell<Vec<(Vessel, Vessel)>>,
    veto: bool,
}
impl SafetyScreen for FinalScreen {
    fn assess(&self, _: &Vessel) -> SafetyVerdict {
        SafetyVerdict::Allow
    }
    fn assess_equilibrated(&self, before: &Vessel, after: &Vessel) -> SafetyVerdict {
        self.seen.borrow_mut().push((before.clone(), after.clone()));
        if self.veto {
            SafetyVerdict::Veto {
                reason: "post-equilibrium-temperature".into(),
            }
        } else {
            SafetyVerdict::Allow
        }
    }
}
fn screen(veto: bool) -> FinalScreen {
    FinalScreen {
        seen: RefCell::new(vec![]),
        veto,
    }
}
fn fixture() -> Bench {
    let mut b = Bench::new();
    b.vessels[0].deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
    b.vessels.push(Vessel::new(VesselId(1), "beaker"));
    b
}
fn pour() -> Operator {
    Operator::Decant {
        from: VesselId(0),
        to: VesselId(1),
        fraction: 0.5,
    }
}
#[test]
fn hook_observes_pre_solver_and_final_solver_state() {
    let mut b = fixture();
    let s = screen(false);
    b.step_with(pour(), &mut SolverStack::new(vec![Box::new(Warm)]), &s)
        .unwrap();
    let seen = s.seen.borrow();
    assert_eq!(seen.len(), 2);
    for (before, after) in seen.iter() {
        assert_eq!(before.temperature, Kelvin::STANDARD);
        assert_eq!(after.temperature, Kelvin(330.0));
        assert_eq!(
            serde_json::to_value(after).unwrap(),
            serde_json::to_value(b.vessel(after.id).unwrap()).unwrap()
        );
    }
}
#[test]
fn final_veto_restores_whole_transfer_and_emits_only_veto() {
    let mut b = fixture();
    let before = serde_json::to_value(&b).unwrap();
    let s = screen(true);
    let e = b
        .step_with(pour(), &mut SolverStack::new(vec![Box::new(Warm)]), &s)
        .unwrap();
    assert!(matches!(e.as_slice(), [Event::SafetyVeto { .. }]));
    let mut after = serde_json::to_value(&b).unwrap();
    after["log"] = before["log"].clone();
    assert_eq!(after, before);
}
#[test]
fn final_veto_removes_automatic_receiver() {
    let mut b = fixture();
    b.vessels.pop();
    let s = screen(true);
    let e = b
        .step_with(pour(), &mut SolverStack::new(vec![Box::new(Warm)]), &s)
        .unwrap();
    assert!(matches!(e.as_slice(), [Event::SafetyVeto { .. }]));
    assert_eq!(b.vessels.len(), 1);
    assert_eq!(b.vessels[0].contents[0].moles, Moles(2.0));
}
#[test]
fn final_veto_restores_stock_draw() {
    let mut b = fixture();
    b.stock.stock("water", 1.0, stock::StockUnit::Mole);
    let before = serde_json::to_value(&b).unwrap();
    let s = screen(true);
    b.step_with(
        Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new("water"),
            moles: Moles(0.5),
            at: None,
        },
        &mut SolverStack::new(vec![Box::new(Warm)]),
        &s,
    )
    .unwrap();
    let mut after = serde_json::to_value(&b).unwrap();
    after["log"] = before["log"].clone();
    assert_eq!(after, before);
}
#[test]
fn unchanged_operation_skips_final_hook() {
    let mut b = fixture();
    let s = screen(true);
    b.step_with(
        Operator::Decant {
            from: VesselId(0),
            to: VesselId(1),
            fraction: 0.0,
        },
        &mut SolverStack::new(vec![Box::new(Warm)]),
        &s,
    )
    .unwrap();
    assert!(s.seen.borrow().is_empty());
}
#[test]
fn final_warning_survives_without_duplicate_rule() {
    struct Warn;
    impl SafetyScreen for Warn {
        fn assess(&self, _: &Vessel) -> SafetyVerdict {
            SafetyVerdict::Allow
        }
        fn assess_equilibrated(&self, _: &Vessel, _: &Vessel) -> SafetyVerdict {
            SafetyVerdict::Warn {
                severity: Severity::Danger,
                rule: "final-state".into(),
                hazard: "test".into(),
                real_world: "test".into(),
            }
        }
    }
    let mut b = fixture();
    let e = b
        .step_with(pour(), &mut SolverStack::new(vec![Box::new(Warm)]), &Warn)
        .unwrap();
    assert_eq!(
        e.iter()
            .filter(|e| matches!(e,Event::HazardWarning {rule,..} if rule=="final-state"))
            .count(),
        1
    );
    assert_eq!(b.vessels[0].temperature, Kelvin(330.0));
}
