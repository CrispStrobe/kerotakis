//! L0 sees the raw receiver inventory, geometry and temperature that the operation applies.
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
fn oil(amount: f64) -> vessel::UnresolvedMaterialPortion {
    let recipe = material::lookup("olive_oil", None).unwrap();
    vessel::UnresolvedMaterialPortion {
        material: recipe.canonical_key.clone(),
        recipe_id: recipe.id.clone(),
        recipe_version: recipe.version,
        basis: recipe.basis,
        amount,
        enzyme_hydrolysis: None,
        protein_denatured_fraction: 0.0,
    }
}
fn bench() -> Bench {
    let mut b = Bench::new();
    for line in [
        "add v1 water 2mol",
        "new",
        "add v2 whole_milk 10mL",
        "add v2 food_colour_red 1mL",
        "new",
        "add v3 water 1mol",
    ] {
        b.step(script::parse_op(line).unwrap().unwrap()).unwrap();
    }
    b.vessels[0].unresolved_materials.push(oil(0.125));
    b.vessels[0].temperature = Kelvin(323.15);
    b.vessels[2].temperature = Kelvin(303.15);
    b.vessels[1].thermal_mode = vessel::ThermalMode::Adiabatic;
    assert!(!b.vessels[1].surface_colours.is_empty());
    b
}
fn op(kind: usize, fraction: f64) -> Operator {
    match kind {
        0 => Operator::Decant {
            from: VesselId(0),
            to: VesselId(1),
            fraction,
        },
        1 => Operator::Mix {
            a: VesselId(0),
            b: VesselId(2),
            into: VesselId(1),
            fraction_a: fraction,
            fraction_b: fraction / 2.0,
        },
        _ => Operator::Filter {
            from: VesselId(0),
            to: VesselId(1),
        },
    }
}
fn observed(kind: usize) -> (Vessel, Vessel) {
    let mut b = bench();
    let recorder = Recorder::default();
    b.step_with(op(kind, 0.5), &mut SolverStack::new(vec![]), &recorder)
        .unwrap();
    let seen = recorder.0.borrow();
    assert_eq!(seen.len(), 1);
    (seen[0].clone(), b.vessels[1].clone())
}
fn surface(kind: usize) {
    let (probe, actual) = observed(kind);
    assert_eq!(probe.surface_colours, actual.surface_colours);
    assert!(probe.surface_colours.is_empty());
}
fn materials(kind: usize) {
    let (probe, actual) = observed(kind);
    assert_eq!(
        serde_json::to_value(&probe.unresolved_materials).unwrap(),
        serde_json::to_value(&actual.unresolved_materials).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&probe.contents).unwrap(),
        serde_json::to_value(&actual.contents).unwrap()
    );
}
fn temperature(kind: usize) {
    let (probe, actual) = observed(kind);
    assert_eq!(probe.temperature, actual.temperature);
}
#[test]
fn decant_probe_has_applied_surface_geometry() {
    surface(0);
}
#[test]
fn mix_probe_has_applied_surface_geometry() {
    surface(1);
}
#[test]
fn filter_probe_has_applied_surface_geometry() {
    surface(2);
}
#[test]
fn decant_probe_includes_incoming_unresolved_liquid() {
    materials(0);
}
#[test]
fn mix_probe_includes_incoming_unresolved_liquid() {
    materials(1);
}
#[test]
fn decant_probe_has_applied_adiabatic_temperature() {
    temperature(0);
}
#[test]
fn mix_probe_has_applied_adiabatic_temperature() {
    temperature(1);
}
#[test]
fn filter_probe_has_applied_adiabatic_temperature() {
    temperature(2);
}
#[test]
fn zero_pours_preserve_receiver_surface_geometry() {
    for kind in [0, 1] {
        let mut b = bench();
        let before = serde_json::to_value(&b.vessels[1].surface_colours).unwrap();
        let recorder = Recorder::default();
        b.step_with(op(kind, 0.0), &mut SolverStack::new(vec![]), &recorder)
            .unwrap();
        assert_eq!(
            serde_json::to_value(&recorder.0.borrow()[0].surface_colours).unwrap(),
            before
        );
        assert_eq!(
            serde_json::to_value(&b.vessels[1].surface_colours).unwrap(),
            before
        );
    }
}
#[test]
fn filtration_moves_declared_unresolved_liquid_and_keeps_solid() {
    let mut b = bench();
    b.vessels[0].deposit(SpeciesId::new("Fe"), Moles(0.25), Phase::Solid);
    b.step_with(op(2, 1.0), &mut SolverStack::new(vec![]), &PermissiveScreen)
        .unwrap();
    assert!(b.vessels[0].unresolved_materials.is_empty());
    assert_eq!(
        b.vessels[1]
            .unresolved_materials
            .iter()
            .filter(|p| p.material == "olive_oil")
            .map(|p| p.amount)
            .sum::<f64>(),
        0.125
    );
    assert_eq!(b.vessels[0].moles_of(&SpeciesId::new("Fe")).0, 0.25);
    assert_eq!(b.vessels[1].moles_of(&SpeciesId::new("Fe")).0, 0.0);
}
