//! Source-informed recovery forecasts frozen before baseline execution.
use kerotakis_core::authority::SpillDestination;
use kerotakis_core::spill::SpillCompartment;
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
        "recovery-counter"
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.0 += 1;
        v.temperature.0 += 1.0;
        Ok(vec![])
    }
}
fn destination() -> SpillDestination {
    SpillDestination::Tray {
        tray: "contract".into(),
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
fn fixture(amount: f64, unresolved: bool) -> Bench {
    let mut b = Bench::new();
    let mut spill = SpillCompartment::new(destination(), Kelvin(333.15));
    if unresolved {
        spill.unresolved_materials.push(oil(amount));
    } else {
        let mut v = Vessel::new(VesselId(1), "spill-source");
        v.deposit(SpeciesId::new("water"), Moles(amount), Phase::Liquid);
        spill.contents = v.contents;
    }
    b.spills.push(spill);
    b
}
fn op(fraction: f64) -> Operator {
    Operator::RecoverSpill {
        destination: destination(),
        to: VesselId(0),
        fraction,
    }
}
fn refused(mut b: Bench, fraction: f64) {
    let before = serde_json::to_value(&b).unwrap();
    let mut counter = Counter::default();
    let screen = Recorder::default();
    assert!(b.step_with(op(fraction), &mut counter, &screen).is_err());
    assert_eq!(serde_json::to_value(&b).unwrap(), before);
    assert_eq!(counter.0, 0);
    assert!(
        screen.0.borrow().is_empty(),
        "invalid quantities must not reach screening"
    );
}
fn observed(mut b: Bench) -> (Vessel, Vessel) {
    let screen = Recorder::default();
    b.step_with(op(0.5), &mut SolverStack::new(vec![]), &screen)
        .unwrap();
    let seen = screen.0.borrow();
    assert_eq!(seen.len(), 1);
    (seen[0].clone(), b.vessels[0].clone())
}
#[test]
fn screen_includes_recovered_unresolved_material() {
    let (probe, actual) = observed(fixture(0.125, true));
    assert_eq!(probe.unresolved_materials, actual.unresolved_materials);
    assert_eq!(probe.unresolved_materials[0].amount, 0.0625);
}
#[test]
fn screen_has_applied_adiabatic_temperature() {
    let mut b = fixture(2.0, false);
    b.vessels[0].deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    b.vessels[0].thermal_mode = vessel::ThermalMode::Adiabatic;
    let (probe, actual) = observed(b);
    assert!(actual.temperature.0 > Kelvin::STANDARD.0);
    assert_eq!(probe.temperature, actual.temperature);
}
#[test]
fn recovery_disturbs_receiver_surface_in_screen_and_commit() {
    let mut b = fixture(2.0, false);
    for line in ["add v1 whole_milk 10mL", "add v1 food_colour_red 1mL"] {
        b.step(script::parse_op(line).unwrap().unwrap()).unwrap();
    }
    assert!(!b.vessels[0].surface_colours.is_empty());
    let (probe, actual) = observed(b);
    assert!(probe.surface_colours.is_empty());
    assert!(actual.surface_colours.is_empty());
}
#[test]
fn screen_includes_committed_unpriced_heat_markers() {
    let mut b = fixture(2.0, false);
    b.vessels[0].thermal_mode = vessel::ThermalMode::Adiabatic;
    b.spills[0].unpriced_heat.push(SpeciesId::new("water"));
    let (probe, actual) = observed(b);
    assert_eq!(probe.unpriced_heat, actual.unpriced_heat);
    assert!(probe.unpriced_heat.contains(&SpeciesId::new("water")));
}
#[test]
fn molecular_debit_cannot_disappear_or_round_inaccurately() {
    for fraction in [1e-20, 1e-16, 1e-14] {
        refused(fixture(1.0, false), fraction);
    }
}
#[test]
fn unresolved_debit_cannot_disappear_or_round_inaccurately() {
    for fraction in [1e-20, 1e-16, 1e-14] {
        refused(fixture(1.0, true), fraction);
    }
}
fn quantum_refusals(unresolved: bool) {
    let q = f64::from_bits(1);
    for (amount, fraction) in [(q, 0.5), (5.0 * q, 0.5), (22.0 * q, 0.25)] {
        refused(fixture(amount, unresolved), fraction);
    }
}
#[test]
fn molecular_quantum_fraction_must_be_representable() {
    quantum_refusals(false);
}
#[test]
fn unresolved_quantum_fraction_must_be_representable() {
    quantum_refusals(true);
}
#[test]
fn receiver_increment_cannot_be_swallowed() {
    let mut b = fixture(1.0, false);
    b.vessels[0].deposit(SpeciesId::new("water"), Moles(1e30), Phase::Liquid);
    refused(b, 1.0);
}
#[test]
fn receiver_nonzero_increment_requires_accuracy() {
    let mut b = fixture(1e-14, false);
    b.vessels[0].deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    refused(b, 1.0);
}
#[test]
fn receiver_combined_condensed_inventory_cannot_hide_increment() {
    let mut b = fixture(1.0, false);
    b.spills[0].contents[0].phase = Phase::Aqueous;
    b.vessels[0].deposit(SpeciesId::new("water"), Moles(1e30), Phase::Liquid);
    refused(b, 1.0);
}
#[test]
fn ordinary_partial_recovery_accounts_for_both_inventories() {
    let mut b = fixture(2.0, false);
    b.spills[0].unresolved_materials.push(oil(0.125));
    let events = b
        .step_with(op(0.25), &mut SolverStack::new(vec![]), &PermissiveScreen)
        .unwrap();
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::SpillRecovered { .. })));
    assert_eq!(b.spills[0].contents[0].moles.0, 1.5);
    assert_eq!(b.vessels[0].contents[0].moles.0, 0.5);
    assert_eq!(b.spills[0].unresolved_materials[0].amount, 0.09375);
    assert_eq!(b.vessels[0].unresolved_materials[0].amount, 0.03125);
}
#[test]
fn exact_full_quantum_recovery_remains_supported() {
    for unresolved in [false, true] {
        let mut b = fixture(f64::from_bits(1), unresolved);
        b.step_with(op(1.0), &mut SolverStack::new(vec![]), &PermissiveScreen)
            .unwrap();
        assert!(b.spills.is_empty());
        if unresolved {
            assert_eq!(
                b.vessels[0].unresolved_materials[0].amount,
                f64::from_bits(1)
            );
        } else {
            assert_eq!(b.vessels[0].contents[0].moles.0, f64::from_bits(1));
        }
    }
}
#[test]
fn zero_recovery_skips_screen_and_settlement() {
    let mut b = fixture(1.0, false);
    let before = serde_json::to_value(&b).unwrap();
    let mut counter = Counter::default();
    let screen = Recorder::default();
    let events = b.step_with(op(0.0), &mut counter, &screen).unwrap();
    let mut after = serde_json::to_value(&b).unwrap();
    after["log"] = before["log"].clone();
    assert_eq!(after, before);
    assert!(events.is_empty());
    assert_eq!(counter.0, 0);
    assert!(screen.0.borrow().is_empty());
}
#[test]
fn recovery_veto_is_atomic_and_skips_settlement() {
    struct Veto;
    impl SafetyScreen for Veto {
        fn assess(&self, _: &Vessel) -> SafetyVerdict {
            SafetyVerdict::Veto {
                reason: "recovery veto".into(),
            }
        }
    }
    let mut b = fixture(2.0, false);
    let before = serde_json::to_value(&b).unwrap();
    let mut counter = Counter::default();
    let events = b.step_with(op(0.5), &mut counter, &Veto).unwrap();
    let mut after = serde_json::to_value(&b).unwrap();
    after["log"] = before["log"].clone();
    assert_eq!(after, before);
    assert_eq!(counter.0, 0);
    assert!(matches!(&events[..], [Event::SafetyVeto { .. }]));
}
