//! Source-informed typed crystal ownership contracts frozen before support.
//! Formula-unit conservation is distinct from phase equilibrium, kinetics and
//! reaction heat. SrCO3 support is scoped to the closed typed component enum;
//! these tests do not authorize arbitrary unknown bulk species.
use kerotakis_core::*;

#[derive(Clone, Copy)]
enum Action {
    Warm,
    SetComponent(SolidSolutionComponent, f64),
    SwapCation,
    DissolveCalcium(bool),
    Redistribute,
    CalciumToBulk,
}
struct Route(Action);
fn set_component(v: &mut Vessel, component: SolidSolutionComponent, moles: f64) {
    v.solid_solutions[0]
        .components
        .iter_mut()
        .find(|c| c.component == component)
        .unwrap()
        .moles = Moles(moles);
}
impl Equilibrator for Route {
    fn name(&self) -> &'static str {
        "typed-crystal-owner-audit"
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        match self.0 {
            Action::Warm => v.temperature = Kelvin(310.0),
            Action::SetComponent(c, n) => set_component(v, c, n),
            Action::SwapCation => {
                let ca = v.solid_solutions[0]
                    .moles_of(SolidSolutionComponent::CalciumCarbonate)
                    .0;
                let sr = v.solid_solutions[0]
                    .moles_of(SolidSolutionComponent::StrontiumCarbonate)
                    .0;
                set_component(v, SolidSolutionComponent::CalciumCarbonate, sr);
                set_component(v, SolidSolutionComponent::StrontiumCarbonate, ca);
            }
            Action::DissolveCalcium(include_carbonate) => {
                let amount = v.solid_solutions[0]
                    .moles_of(SolidSolutionComponent::CalciumCarbonate)
                    .0;
                set_component(v, SolidSolutionComponent::CalciumCarbonate, 0.0);
                v.deposit(SpeciesId::new("Ca+2"), Moles(amount), Phase::Aqueous);
                if include_carbonate {
                    v.deposit(SpeciesId::new("CO3-2"), Moles(amount), Phase::Aqueous);
                }
            }
            Action::Redistribute => {
                v.solid_solutions = vec![
                    SolidSolution::aragonite_strontianite("phase-a", Moles(0.25), Moles(0.125)),
                    SolidSolution::aragonite_strontianite("phase-b", Moles(0.75), Moles(0.375)),
                ];
            }
            Action::CalciumToBulk => {
                let amount = v.solid_solutions[0]
                    .moles_of(SolidSolutionComponent::CalciumCarbonate)
                    .0;
                set_component(v, SolidSolutionComponent::CalciumCarbonate, 0.0);
                v.deposit(SpeciesId::new("CaCO3"), Moles(amount), Phase::Solid);
            }
        }
        Ok(Vec::new())
    }
}
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
fn strict(mut v: Vessel, action: Action, accept: bool) {
    let before = serde_json::to_value(&v).unwrap();
    let mut stack =
        SolverStack::with_required_conservation(vec![Box::new(Route(action))], 1e-8).unwrap();
    let events = stack.equilibrate(&mut v).unwrap();
    let failed = events
        .iter()
        .any(|e| matches!(e, Event::SolverFailed { .. }));
    assert_eq!(failed, !accept);
    if accept {
        assert!(matches!(
            stack.last_routes[0].outcome,
            SolverRouteOutcome::Succeeded { .. }
        ));
        if matches!(action, Action::Warm) {
            assert_eq!(v.temperature.0, 310.0);
        }
    } else {
        assert_eq!(serde_json::to_value(&v).unwrap(), before);
        assert!(matches!(
            stack.last_routes[0].outcome,
            SolverRouteOutcome::Failed
        ));
    }
}
#[test]
fn unchanged_calcium_crystal_is_represented() {
    strict(crystal(1.0, 0.0), Action::Warm, true);
}
#[test]
fn unchanged_strontium_crystal_is_represented_without_bulk_registry_alias() {
    strict(crystal(0.0, 1.0), Action::Warm, true);
}
#[test]
fn empty_well_shaped_crystal_is_valid() {
    strict(crystal(0.0, 0.0), Action::Warm, true);
}
#[test]
fn saved_reloaded_mixed_crystal_remains_represented() {
    let original = crystal(0.25, 0.75);
    let value = serde_json::to_value(&original).unwrap();
    let copy: Vessel = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(&copy).unwrap(), value);
    strict(copy, Action::Warm, true);
}
fn background(ca: f64, sr: f64) -> Vessel {
    let mut v = crystal(ca, sr);
    v.deposit(SpeciesId::new("CaCO3"), Moles(2.0), Phase::Solid);
    v
}
#[test]
fn trace_calcium_loss_not_hidden_by_bulk_calcium_background() {
    strict(
        background(1e-200, 0.0),
        Action::SetComponent(SolidSolutionComponent::CalciumCarbonate, 0.0),
        false,
    );
}
#[test]
fn trace_strontium_loss_not_hidden_by_unchanged_calcium_background() {
    strict(
        background(0.0, 1e-200),
        Action::SetComponent(SolidSolutionComponent::StrontiumCarbonate, 0.0),
        false,
    );
}
#[test]
fn trace_calcium_creation_not_hidden_by_bulk_calcium_background() {
    strict(
        background(0.0, 0.0),
        Action::SetComponent(SolidSolutionComponent::CalciumCarbonate, 1e-200),
        false,
    );
}
#[test]
fn trace_strontium_creation_not_hidden_by_unchanged_calcium_background() {
    strict(
        background(0.0, 0.0),
        Action::SetComponent(SolidSolutionComponent::StrontiumCarbonate, 1e-200),
        false,
    );
}
#[test]
fn calcium_to_strontium_swap_refuses_even_with_equal_carbon_oxygen() {
    strict(crystal(1.0, 0.0), Action::SwapCation, false);
}
#[test]
fn strontium_to_calcium_swap_refuses_even_with_equal_carbon_oxygen() {
    strict(crystal(0.0, 1.0), Action::SwapCation, false);
}
#[test]
fn neutral_balanced_calcium_carbonate_dissolution_closes() {
    strict(crystal(0.001, 0.0), Action::DissolveCalcium(true), true);
}
#[test]
fn calcium_only_dissolution_cannot_lose_carbonate_inventory() {
    strict(crystal(0.001, 0.0), Action::DissolveCalcium(false), false);
}
#[test]
fn redistribution_across_distinct_typed_phases_conserves_formula_units() {
    strict(crystal(1.0, 0.5), Action::Redistribute, true);
}
#[test]
fn typed_calcium_to_registered_bulk_phase_is_same_owned_matter() {
    strict(crystal(0.25, 0.5), Action::CalciumToBulk, true);
}
#[test]
fn generic_unregistered_strontium_carbonate_bulk_still_refuses() {
    let mut v = crystal(0.0, 0.0);
    v.deposit(SpeciesId::new("SrCO3"), Moles(0.25), Phase::Solid);
    strict(v, Action::Warm, false);
}
#[test]
fn typed_crystal_does_not_certify_unrelated_unknown_bulk() {
    let mut v = crystal(0.5, 0.5);
    v.deposit(
        SpeciesId::new("opaque-crystal-audit"),
        Moles(0.25),
        Phase::Solid,
    );
    strict(v, Action::Warm, false);
}
