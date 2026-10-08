//! Source-informed nuclide state boundary controls, frozen before validator repair.
//! Validation enforces serializable identity/amount domains, not nuclear
//! conservation or curated decay support. Legacy solver routes remain usable.
use kerotakis_core::delta::{StateDelta, ThermalDelta};
use kerotakis_core::nuclide::Nuclide;
use kerotakis_core::*;

#[derive(Clone, Copy)]
enum Bad {
    Negative,
    Nan,
    Infinite,
    UnknownElement,
    ZeroMass,
}
fn entry(bad: Bad) -> (Nuclide, f64) {
    match bad {
        Bad::Negative => (Nuclide::new("C", 14), -1e-12),
        Bad::Nan => (Nuclide::new("C", 14), f64::NAN),
        Bad::Infinite => (Nuclide::new("C", 14), f64::INFINITY),
        Bad::UnknownElement => (Nuclide::new("Xx", 14), 1e-12),
        Bad::ZeroMass => (Nuclide::new("C", 0), 1e-12),
    }
}
fn water() -> Vessel {
    let mut v = Vessel::new(VesselId(0), "tracer");
    v.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    v
}
fn invalid_state_refuses_delta(bad: Bad) {
    let mut v = water();
    let (key, amount) = entry(bad);
    v.nuclides.inventory.insert(key.clone(), amount);
    let before = format!("{v:?}");
    let result = StateDelta::new("nuclide-state-audit")
        .with_thermal(ThermalDelta::SetTemperature(Kelvin(310.0)))
        .commit(&mut v);
    assert!(result.is_err(), "invalid nuclide state accepted");
    assert_eq!(format!("{v:?}"), before);
    assert_eq!(v.nuclides.inventory[&key].to_bits(), amount.to_bits());
}
struct Corrupt(Bad);
impl Equilibrator for Corrupt {
    fn name(&self) -> &'static str {
        "nuclide-state-audit"
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        let (key, amount) = entry(self.0);
        v.nuclides.inventory.insert(key, amount);
        v.temperature = Kelvin(310.0);
        v.honesty_said.push("uncommitted tracer diagnosis".into());
        Ok(Vec::new())
    }
}
fn invalid_solver_rolls_back(bad: Bad) {
    let mut v = water();
    v.nuclides.deposit(Nuclide::new("C", 12), 0.0);
    let before = serde_json::to_value(&v).unwrap();
    let mut stack = SolverStack::new(vec![Box::new(Corrupt(bad))]);
    let events = stack.equilibrate(&mut v).unwrap();
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::SolverFailed { .. })));
    assert_eq!(serde_json::to_value(&v).unwrap(), before);
    assert!(matches!(
        stack.last_routes[0].outcome,
        SolverRouteOutcome::Failed
    ));
}
#[test]
fn negative_amount_refuses_delta() {
    invalid_state_refuses_delta(Bad::Negative);
}
#[test]
fn nan_amount_refuses_delta() {
    invalid_state_refuses_delta(Bad::Nan);
}
#[test]
fn infinite_amount_refuses_delta() {
    invalid_state_refuses_delta(Bad::Infinite);
}
#[test]
fn unknown_element_refuses_delta() {
    invalid_state_refuses_delta(Bad::UnknownElement);
}
#[test]
fn zero_mass_number_refuses_delta() {
    invalid_state_refuses_delta(Bad::ZeroMass);
}
#[test]
fn negative_amount_legacy_solver_rolls_back() {
    invalid_solver_rolls_back(Bad::Negative);
}
#[test]
fn nan_amount_legacy_solver_rolls_back() {
    invalid_solver_rolls_back(Bad::Nan);
}
#[test]
fn infinite_amount_legacy_solver_rolls_back() {
    invalid_solver_rolls_back(Bad::Infinite);
}
#[test]
fn unknown_element_legacy_solver_rolls_back() {
    invalid_solver_rolls_back(Bad::UnknownElement);
}
#[test]
fn zero_mass_number_legacy_solver_rolls_back() {
    invalid_solver_rolls_back(Bad::ZeroMass);
}

#[test]
fn positive_subnormal_and_zero_tracers_allow_delta_commit() {
    for amount in [1e-12, f64::from_bits(1), 0.0] {
        let mut v = water();
        let key = Nuclide::parse("Tc-99m").unwrap();
        v.nuclides.deposit(key.clone(), amount);
        StateDelta::new("nuclide-valid-audit")
            .with_thermal(ThermalDelta::SetTemperature(Kelvin(310.0)))
            .commit(&mut v)
            .unwrap();
        assert_eq!(v.temperature.0, 310.0);
        assert_eq!(v.nuclides.inventory[&key].to_bits(), amount.to_bits());
        assert!(serde_json::to_value(&v).is_ok());
    }
}
struct Warm;
impl Equilibrator for Warm {
    fn name(&self) -> &'static str {
        "nuclide-valid-audit"
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        v.temperature = Kelvin(310.0);
        Ok(Vec::new())
    }
}
#[test]
fn positive_subnormal_and_zero_tracers_allow_legacy_solver() {
    for amount in [1e-12, f64::from_bits(1), 0.0] {
        let mut v = water();
        let key = Nuclide::new("C", 14);
        v.nuclides.deposit(key.clone(), amount);
        let mut stack = SolverStack::new(vec![Box::new(Warm)]);
        let events = stack.equilibrate(&mut v).unwrap();
        assert!(!events
            .iter()
            .any(|e| matches!(e, Event::SolverFailed { .. })));
        assert_eq!(v.temperature.0, 310.0);
        assert_eq!(v.nuclides.inventory[&key].to_bits(), amount.to_bits());
        assert!(serde_json::to_value(&v).is_ok());
    }
}
