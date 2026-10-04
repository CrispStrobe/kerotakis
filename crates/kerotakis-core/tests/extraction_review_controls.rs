//! Supplementary review controls frozen before correction; no separate old-source baseline.
use kerotakis_core::stock::StockUnit;
use kerotakis_core::*;
use std::cell::RefCell;
fn fixture() -> Bench {
    let mut b = Bench::new();
    b.vessels[0].deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
    b.vessels[0].deposit(SpeciesId::new("I2"), Moles(0.001), Phase::Aqueous);
    b.vessels.push(Vessel::new(VesselId(1), "beaker"));
    b
}
fn op() -> Operator {
    Operator::Extract {
        from: VesselId(0),
        to: VesselId(1),
        solvent: SpeciesId::new("hexane"),
        total_solvent: Moles(0.1),
        stages: 1,
    }
}
#[test]
fn screened_receiver_includes_propagated_unpriced_heat() {
    struct Record(RefCell<Vec<Vessel>>);
    impl SafetyScreen for Record {
        fn assess(&self, v: &Vessel) -> SafetyVerdict {
            self.0.borrow_mut().push(v.clone());
            SafetyVerdict::Allow
        }
    }
    let mut b = fixture();
    b.vessels[0].unpriced_heat.push(SpeciesId::new("I2"));
    b.vessels[1].thermal_mode = vessel::ThermalMode::Adiabatic;
    let screen = Record(RefCell::new(vec![]));
    b.step_with(op(), &mut SolverStack::new(vec![]), &screen)
        .unwrap();
    let seen = screen.0.borrow();
    assert_eq!(seen.len(), 2);
    assert_eq!(seen[1].unpriced_heat, b.vessels[1].unpriced_heat);
    assert!(seen[1].unpriced_heat.contains(&SpeciesId::new("I2")));
}
#[test]
fn stock_refusal_drops_provisional_warnings_and_moved_diagnostics() {
    struct Warn;
    impl SafetyScreen for Warn {
        fn assess(&self, _: &Vessel) -> SafetyVerdict {
            SafetyVerdict::Warn {
                severity: Severity::Danger,
                rule: "provisional-extraction".into(),
                hazard: "provisional".into(),
                real_world: "test".into(),
            }
        }
    }
    let mut b = fixture();
    b.vessels[0].deposit(SpeciesId::new("NaCl"), Moles(0.1), Phase::Aqueous);
    b.stock.stock("hexane", 0.01, StockUnit::Mole);
    let before = serde_json::to_value(&b).unwrap();
    let events = b
        .step_with(op(), &mut SolverStack::new(vec![]), &Warn)
        .unwrap();
    let mut after = serde_json::to_value(&b).unwrap();
    after["log"] = before["log"].clone();
    assert_eq!(after, before);
    assert!(matches!(&events[..], [Event::StockExhausted { .. }]));
}
