//! Typed crystal shape controls frozen before general state validation repair.
//! Invalid owner shape must reject both transactional delta and legacy solver
//! trial paths; shape validity is separate from formula-unit conservation.
use kerotakis_core::delta::{StateDelta, ThermalDelta};
use kerotakis_core::*;
fn crystal(ca: f64, sr: f64) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "crystal");
    v.deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
    v.solid_solutions
        .push(SolidSolution::aragonite_strontianite(
            "mixed",
            Moles(ca),
            Moles(sr),
        ));
    v
}
struct Warm;
impl Equilibrator for Warm {
    fn name(&self) -> &'static str {
        "crystal-shape-audit"
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        v.temperature = Kelvin(310.0);
        Ok(Vec::new())
    }
}
fn invalid_shape_refuses_all_paths(mut v: Vessel) {
    let before = format!("{v:?}");
    assert!(StateDelta::new("crystal-shape-audit")
        .with_thermal(ThermalDelta::SetTemperature(Kelvin(310.0)))
        .commit(&mut v)
        .is_err());
    assert_eq!(format!("{v:?}"), before);
    let mut legacy = SolverStack::new(vec![Box::new(Warm)]);
    let events = legacy.equilibrate(&mut v).unwrap();
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::SolverFailed { .. })));
    assert_eq!(format!("{v:?}"), before);
    let mut required = SolverStack::with_required_conservation(vec![Box::new(Warm)], 1e-8).unwrap();
    let events = required.equilibrate(&mut v).unwrap();
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::SolverFailed { .. })));
    assert_eq!(format!("{v:?}"), before);
}
#[test]
fn duplicate_component_entries_are_invalid_not_additive_ownership() {
    let mut v = crystal(0.5, 0.5);
    v.solid_solutions[0].components[1].component = SolidSolutionComponent::CalciumCarbonate;
    invalid_shape_refuses_all_paths(v);
}
#[test]
fn missing_component_entry_is_invalid_even_if_remaining_amount_finite() {
    let mut v = crystal(0.5, 0.5);
    v.solid_solutions[0].components.pop();
    invalid_shape_refuses_all_paths(v);
}
#[test]
fn blank_crystal_label_is_invalid() {
    let mut v = crystal(0.5, 0.5);
    v.solid_solutions[0].label = " ".into();
    invalid_shape_refuses_all_paths(v);
}
#[test]
fn duplicate_crystal_labels_are_ambiguous_owned_phase_identity() {
    let mut v = crystal(0.5, 0.5);
    v.solid_solutions.push(v.solid_solutions[0].clone());
    invalid_shape_refuses_all_paths(v);
}
#[test]
fn negative_component_amount_is_invalid() {
    invalid_shape_refuses_all_paths(crystal(-1e-200, 0.0));
}
#[test]
fn nonfinite_component_amount_is_invalid() {
    for n in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        invalid_shape_refuses_all_paths(crystal(n, 0.0));
    }
}

#[test]
fn empty_component_vector_is_invalid_unrepresented_phase() {
    let mut v = crystal(0.0, 0.0);
    v.solid_solutions[0].components.clear();
    invalid_shape_refuses_all_paths(v);
}
#[test]
fn extra_component_entry_is_invalid_even_when_zero() {
    let mut v = crystal(0.5, 0.5);
    let mut duplicate = v.solid_solutions[0].components[0].clone();
    duplicate.moles = Moles(0.0);
    v.solid_solutions[0].components.push(duplicate);
    invalid_shape_refuses_all_paths(v);
}
