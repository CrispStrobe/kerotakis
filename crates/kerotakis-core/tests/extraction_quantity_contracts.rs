//! Source-informed extraction forecasts frozen before baseline execution.
use kerotakis_core::stock::StockUnit;
use kerotakis_core::*;
use std::cell::RefCell;
#[derive(Default)]
struct Recorder(RefCell<Vec<Vessel>>);
impl SafetyScreen for Recorder {
    fn assess(&self, v: &Vessel) -> SafetyVerdict {
        self.0.borrow_mut().push(v.clone());
        SafetyVerdict::Allow
    }
}
#[derive(Default)]
struct Counter(usize);
impl Equilibrator for Counter {
    fn name(&self) -> &'static str {
        "extraction-counter"
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.0 += 1;
        v.temperature.0 += 1.0;
        Ok(vec![])
    }
}
fn fixture(receiver: bool) -> Bench {
    let mut b = Bench::new();
    b.vessels[0].deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
    b.vessels[0].deposit(SpeciesId::new("I2"), Moles(0.001), Phase::Aqueous);
    if receiver {
        b.vessels.push(Vessel::new(VesselId(1), "beaker"));
    }
    b
}
fn op(amount: f64, stages: u32) -> Operator {
    Operator::Extract {
        from: VesselId(0),
        to: VesselId(1),
        solvent: SpeciesId::new("hexane"),
        total_solvent: Moles(amount),
        stages,
    }
}
fn physical(b: &Bench) -> serde_json::Value {
    let mut v = serde_json::to_value(b).unwrap();
    v.as_object_mut().unwrap().remove("log");
    v
}
fn refused(mut b: Bench, amount: f64) {
    let before = serde_json::to_value(&b).unwrap();
    let mut solver = Counter::default();
    let screen = Recorder::default();
    assert!(b.step_with(op(amount, 1), &mut solver, &screen).is_err());
    assert_eq!(serde_json::to_value(&b).unwrap(), before);
    assert_eq!(solver.0, 0);
    assert!(screen.0.borrow().is_empty());
}
fn observed(mut b: Bench) -> (Bench, Vec<Vessel>) {
    let screen = Recorder::default();
    b.step_with(op(0.1, 1), &mut SolverStack::new(vec![]), &screen)
        .unwrap();
    let seen = screen.0.borrow().clone();
    (b, seen)
}
fn receiver_veto(existing: bool) {
    struct Veto;
    impl SafetyScreen for Veto {
        fn assess(&self, v: &Vessel) -> SafetyVerdict {
            if v.id == VesselId(1) {
                SafetyVerdict::Veto {
                    reason: "receiver extraction veto".into(),
                }
            } else {
                SafetyVerdict::Allow
            }
        }
    }
    let mut b = fixture(existing);
    b.stock.stock("hexane", 1.0, StockUnit::Mole);
    let before = physical(&b);
    let mut solver = Counter::default();
    let events = b.step_with(op(0.1, 1), &mut solver, &Veto).unwrap();
    assert_eq!(physical(&b), before);
    assert_eq!(solver.0, 0);
    assert!(matches!(&events[..], [Event::SafetyVeto { .. }]));
}
#[test]
fn absent_receiver_is_screened_before_creation() {
    receiver_veto(false);
}
#[test]
fn existing_receiver_veto_remains_atomic() {
    receiver_veto(true);
}
fn hook_veto(target: usize) {
    struct Hook(usize);
    impl SafetyScreen for Hook {
        fn assess(&self, _: &Vessel) -> SafetyVerdict {
            SafetyVerdict::Allow
        }
        fn assess_pour(&self, before: &Vessel, after: &Vessel) -> SafetyVerdict {
            assert_eq!(before.id, after.id);
            assert_eq!(before.moles_of(&SpeciesId::new("hexane")).0, 0.0);
            assert_eq!(after.moles_of(&SpeciesId::new("hexane")).0, 0.1);
            if after.id == VesselId(self.0) {
                SafetyVerdict::Veto {
                    reason: "extraction pour veto".into(),
                }
            } else {
                SafetyVerdict::Allow
            }
        }
    }
    let mut b = fixture(true);
    let before = physical(&b);
    let mut solver = Counter::default();
    let events = b.step_with(op(0.1, 1), &mut solver, &Hook(target)).unwrap();
    assert_eq!(physical(&b), before);
    assert_eq!(solver.0, 0);
    assert!(matches!(&events[..], [Event::SafetyVeto { .. }]));
}
#[test]
fn source_uses_contextual_pour_hook() {
    hook_veto(0);
}
#[test]
fn receiver_uses_contextual_pour_hook() {
    hook_veto(1);
}
#[test]
fn source_screen_observes_applied_contact_temperature() {
    let mut b = fixture(true);
    b.vessels[0].temperature = Kelvin(298.6);
    b.vessels[0].thermal_mode = vessel::ThermalMode::Adiabatic;
    let (b, seen) = observed(b);
    assert_eq!(seen.len(), 2);
    assert!(b.vessels[0].temperature.0 < 298.6);
    assert_eq!(seen[0].temperature, b.vessels[0].temperature);
}
#[test]
fn receiver_screen_observes_applied_adiabatic_temperature() {
    let mut b = fixture(true);
    b.vessels[1].deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    b.vessels[1].temperature = Kelvin(350.0);
    b.vessels[1].thermal_mode = vessel::ThermalMode::Adiabatic;
    let (b, seen) = observed(b);
    assert_eq!(seen.len(), 2);
    assert!(b.vessels[1].temperature.0 < 350.0);
    assert_eq!(seen[1].temperature, b.vessels[1].temperature);
}
#[test]
fn receiver_surface_is_disturbed_before_screen_and_commit() {
    let mut b = fixture(true);
    for line in ["add v2 whole_milk 10mL", "add v2 food_colour_red 1mL"] {
        b.step(script::parse_op(line).unwrap().unwrap()).unwrap();
    }
    assert!(!b.vessels[1].surface_colours.is_empty());
    let (b, seen) = observed(b);
    assert!(seen[1].surface_colours.is_empty());
    assert!(b.vessels[1].surface_colours.is_empty());
}
#[test]
fn absent_receiver_allow_screens_exact_committed_inventory() {
    let (b, seen) = observed(fixture(false));
    assert_eq!(seen.len(), 2);
    assert_eq!(seen[1].id, VesselId(1));
    assert_eq!(seen[1].contents, b.vessels[1].contents);
    assert_eq!(b.vessels.len(), 2);
}
#[test]
fn existing_receiver_screen_retains_and_adds_exact_inventory() {
    let mut b = fixture(true);
    b.vessels[1].deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    let (b, seen) = observed(b);
    assert_eq!(seen.len(), 2);
    assert_eq!(seen[1].contents, b.vessels[1].contents);
}
#[test]
fn positive_tiny_solvent_yield_cannot_have_zero_donor_debit() {
    refused(fixture(false), 1e-20);
}
#[test]
fn receiver_cannot_swallow_extracted_solute() {
    let mut b = fixture(true);
    b.vessels[1].deposit(SpeciesId::new("I2"), Moles(1e20), Phase::Aqueous);
    refused(b, 0.1);
}
#[test]
fn receiver_nonzero_solute_increment_requires_accuracy() {
    let mut b = fixture(true);
    b.vessels[0]
        .contents
        .iter_mut()
        .find(|p| p.species.0 == "I2")
        .unwrap()
        .moles = Moles(1e-14);
    b.vessels[1].deposit(SpeciesId::new("I2"), Moles(1.0), Phase::Aqueous);
    refused(b, 0.1);
}
#[test]
fn combined_condensed_solute_cannot_hide_increment() {
    let mut b = fixture(true);
    b.vessels[1].deposit(SpeciesId::new("I2"), Moles(1e20), Phase::Liquid);
    refused(b, 0.1);
}
#[test]
fn receiver_cannot_swallow_fresh_solvent() {
    let mut b = fixture(true);
    b.vessels[1].deposit(SpeciesId::new("hexane"), Moles(1e20), Phase::Liquid);
    refused(b, 0.1);
}
#[test]
fn receiver_nonzero_solvent_increment_requires_accuracy() {
    let mut b = fixture(true);
    b.vessels[1].deposit(SpeciesId::new("hexane"), Moles(1e13), Phase::Liquid);
    refused(b, 0.1);
}
fn stock_refusal(balance: f64) {
    let mut b = fixture(false);
    b.stock.stock("hexane", balance, StockUnit::Mole);
    let before = physical(&b);
    let mut solver = Counter::default();
    let events = b
        .step_with(op(0.1, 1), &mut solver, &PermissiveScreen)
        .unwrap();
    assert_eq!(physical(&b), before);
    assert_eq!(solver.0, 0);
    assert!(events.iter().any(|e| matches!(
        e,
        Event::StockExhausted { .. } | Event::NotYetModeled { .. }
    )));
    assert!(!events.iter().any(|e| matches!(e, Event::Extracted { .. })));
}
#[test]
fn stock_shortage_is_atomic_without_receiver_creation() {
    stock_refusal(0.01);
}
#[test]
fn stock_precision_refusal_is_atomic_without_receiver_creation() {
    stock_refusal(1e30);
}
#[test]
fn accepted_extraction_debits_fresh_solvent_and_keeps_provenance() {
    let mut b = fixture(false);
    b.stock.stock("hexane", 0.5, StockUnit::Mole);
    b.step_with(op(0.1, 1), &mut SolverStack::new(vec![]), &PermissiveScreen)
        .unwrap();
    assert!((b.stock.remaining("hexane").unwrap().amount - 0.4).abs() < 1e-15);
    assert!(b.vessels[1]
        .lots
        .iter()
        .any(|l| l.source.as_deref() == Some("fresh extracting solvent")));
}
#[test]
fn exact_tiny_loading_with_ordinary_solvent_is_supported() {
    let mut b = fixture(false);
    b.vessels[0]
        .contents
        .iter_mut()
        .find(|p| p.species.0 == "I2")
        .unwrap()
        .moles = Moles(1e-200);
    b.step_with(op(0.1, 1), &mut SolverStack::new(vec![]), &PermissiveScreen)
        .unwrap();
    let initial = 1e-200;
    let donor = b.vessels[0].moles_of(&SpeciesId::new("I2")).0;
    let receiver = b.vessels[1].moles_of(&SpeciesId::new("I2")).0;
    assert!(donor > 0.0 && receiver > 0.0);
    assert!(((donor + receiver) / initial - 1.0).abs() < 1e-8);
}
#[test]
fn ordinary_stages_improve_yield_and_conserve_inventory() {
    let run = |stages| {
        let mut b = fixture(false);
        b.step_with(
            op(0.1, stages),
            &mut SolverStack::new(vec![]),
            &PermissiveScreen,
        )
        .unwrap();
        b
    };
    let single = run(1);
    let repeated = run(4);
    let id = SpeciesId::new("I2");
    assert!(repeated.vessels[1].moles_of(&id).0 > single.vessels[1].moles_of(&id).0);
    for b in [single, repeated] {
        assert!(
            (b.vessels.iter().map(|v| v.moles_of(&id).0).sum::<f64>() / 0.001 - 1.0).abs() < 1e-8
        );
    }
}
#[test]
fn invalid_inputs_are_atomic_without_engine_calls() {
    for (amount, stages) in [
        (0.0, 1),
        (-1.0, 1),
        (f64::NAN, 1),
        (f64::INFINITY, 1),
        (0.1, 0),
    ] {
        let mut b = fixture(false);
        let before = serde_json::to_value(&b).unwrap();
        let mut solver = Counter::default();
        assert!(b
            .step_with(op(amount, stages), &mut solver, &PermissiveScreen)
            .is_err());
        assert_eq!(serde_json::to_value(&b).unwrap(), before);
        assert_eq!(solver.0, 0);
    }
}
#[test]
fn broken_receiver_is_refused_before_stock_and_settlement() {
    for existing in [false, true] {
        let mut b = fixture(existing);
        b.broken_vessels.push(VesselId(1));
        b.stock.stock("hexane", 0.5, StockUnit::Mole);
        let before = serde_json::to_value(&b).unwrap();
        let mut solver = Counter::default();
        assert!(b
            .step_with(op(0.1, 1), &mut solver, &PermissiveScreen)
            .is_err());
        assert_eq!(serde_json::to_value(&b).unwrap(), before);
        assert_eq!(solver.0, 0);
    }
}
