//! Source-informed spill/discard forecasts frozen before execution.
use kerotakis_core::authority::SpillDestination;
use kerotakis_core::spill::SpillCompartment;
use kerotakis_core::*;
use std::cell::Cell;
fn destination() -> SpillDestination {
    SpillDestination::Tray {
        tray: "precision".into(),
    }
}
fn oil(amount: f64) -> vessel::UnresolvedMaterialPortion {
    let r = material::lookup("olive_oil", None).unwrap();
    vessel::UnresolvedMaterialPortion {
        material: r.canonical_key.clone(),
        recipe_id: r.id.clone(),
        recipe_version: r.version,
        basis: r.basis,
        amount,
        enzyme_hydrolysis: None,
        protein_denatured_fraction: 0.0,
    }
}
fn fixture(amount: f64, unresolved: bool) -> Bench {
    let mut b = Bench::new();
    if unresolved {
        b.vessels[0].unresolved_materials.push(oil(amount));
    } else {
        b.vessels[0].deposit(SpeciesId::new("water"), Moles(amount), Phase::Liquid);
    }
    b
}
fn spill(fraction: f64) -> Operator {
    Operator::Spill {
        from: VesselId(0),
        destination: destination(),
        fraction,
        replay_seed: 71,
    }
}
#[derive(Default)]
struct Calls(Cell<usize>);
impl SafetyScreen for Calls {
    fn assess(&self, _: &Vessel) -> SafetyVerdict {
        self.0.set(self.0.get() + 1);
        SafetyVerdict::Allow
    }
}
#[derive(Default)]
struct Solver(usize);
impl Equilibrator for Solver {
    fn name(&self) -> &'static str {
        "spill-quantity-counter"
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.0 += 1;
        v.temperature.0 += 1.0;
        Ok(vec![])
    }
}
fn refuse(mut b: Bench, op: Operator) {
    let before = serde_json::to_value(&b).unwrap();
    let mut solver = Solver::default();
    let screen = Calls::default();
    assert!(b.step_with(op, &mut solver, &screen).is_err());
    assert_eq!(serde_json::to_value(&b).unwrap(), before);
    assert_eq!(solver.0, 0);
    assert_eq!(screen.0.get(), 0);
}
fn debit_matrix(material: bool) {
    for f in [1e-20, 1e-16, 1e-14] {
        refuse(fixture(1.0, material), spill(f));
    }
}
#[test]
fn molecular_spill_requires_accurate_donor_debit() {
    debit_matrix(false);
}
#[test]
fn unresolved_spill_requires_accurate_donor_debit() {
    debit_matrix(true);
}
fn quantum_matrix(material: bool) {
    let q = f64::from_bits(1);
    for (n, f) in [(q, 0.5), (5.0 * q, 0.5), (22.0 * q, 0.25)] {
        refuse(fixture(n, material), spill(f));
    }
}
#[test]
fn molecular_spill_rejects_quantum_loss_and_distortion() {
    quantum_matrix(false);
}
#[test]
fn unresolved_spill_rejects_quantum_loss_and_distortion() {
    quantum_matrix(true);
}
fn receiver(before: f64, incoming: f64, phase: Phase, waste: bool) {
    let mut b = fixture(incoming, false);
    let d = if waste {
        SpillDestination::Waste
    } else {
        destination()
    };
    let mut old = SpillCompartment::new(d, Kelvin::STANDARD);
    let mut v = Vessel::new(VesselId(2), "old-spill");
    v.deposit(SpeciesId::new("water"), Moles(before), phase);
    old.contents = v.contents;
    b.spills.push(old);
    refuse(
        b,
        if waste {
            Operator::Discard {
                vessel: VesselId(0),
            }
        } else {
            spill(1.0)
        },
    );
}
#[test]
fn spill_receiver_increment_cannot_disappear() {
    receiver(1e30, 1.0, Phase::Liquid, false);
}
#[test]
fn spill_receiver_nonzero_increment_requires_accuracy() {
    receiver(1.0, 1e-14, Phase::Liquid, false);
}
#[test]
fn spill_combined_condensed_inventory_cannot_hide_increment() {
    receiver(1e30, 1.0, Phase::Aqueous, false);
}
#[test]
fn discard_receiver_increment_cannot_disappear() {
    receiver(1e30, 1.0, Phase::Liquid, true);
}
#[test]
fn discard_receiver_nonzero_increment_requires_accuracy() {
    receiver(1.0, 1e-14, Phase::Liquid, true);
}
#[test]
fn discard_combined_condensed_inventory_cannot_hide_increment() {
    receiver(1e30, 1.0, Phase::Aqueous, true);
}
#[test]
fn later_component_precision_refusal_restores_entire_spill() {
    let mut b = fixture(2.0, false);
    b.vessels[0].deposit(
        SpeciesId::new("NaCl"),
        Moles(5.0 * f64::from_bits(1)),
        Phase::Aqueous,
    );
    refuse(b, spill(0.5));
}
#[test]
fn zero_spill_skips_screen_and_settlement() {
    let mut b = fixture(1.0, false);
    let before = serde_json::to_value(&b).unwrap();
    let mut solver = Solver::default();
    let screen = Calls::default();
    let events = b.step_with(spill(0.0), &mut solver, &screen).unwrap();
    let mut after = serde_json::to_value(&b).unwrap();
    after["log"] = before["log"].clone();
    assert_eq!(after, before);
    assert!(events.is_empty());
    assert_eq!(solver.0, 0);
    assert_eq!(screen.0.get(), 0);
}
#[test]
fn ordinary_spill_closes_both_inventories() {
    let mut b = fixture(2.0, false);
    b.vessels[0].unresolved_materials.push(oil(0.125));
    b.step_with(
        spill(0.25),
        &mut SolverStack::new(vec![]),
        &PermissiveScreen,
    )
    .unwrap();
    assert_eq!(b.vessels[0].contents[0].moles.0, 1.5);
    assert_eq!(b.spills[0].contents[0].moles.0, 0.5);
    assert_eq!(b.vessels[0].unresolved_materials[0].amount, 0.09375);
    assert_eq!(b.spills[0].unresolved_materials[0].amount, 0.03125);
}
#[test]
fn exact_full_quantum_spill_and_discard_remain_supported() {
    for material in [false, true] {
        for waste in [false, true] {
            let q = f64::from_bits(1);
            let mut b = fixture(q, material);
            let op = if waste {
                Operator::Discard {
                    vessel: VesselId(0),
                }
            } else {
                spill(1.0)
            };
            b.step_with(op, &mut SolverStack::new(vec![]), &PermissiveScreen)
                .unwrap();
            assert!(
                b.vessels[0].contents.is_empty() && b.vessels[0].unresolved_materials.is_empty()
            );
            if material {
                assert_eq!(b.spills[0].unresolved_materials[0].amount, q);
            } else {
                assert_eq!(b.spills[0].contents[0].moles.0, q);
            }
        }
    }
}
#[test]
fn accidental_spill_cannot_be_safety_vetoed() {
    struct Veto;
    impl SafetyScreen for Veto {
        fn assess(&self, _: &Vessel) -> SafetyVerdict {
            SafetyVerdict::Veto {
                reason: "accidental hazard".into(),
            }
        }
    }
    let mut b = fixture(2.0, false);
    let events = b
        .step_with(spill(0.5), &mut SolverStack::new(vec![]), &Veto)
        .unwrap();
    assert_eq!(b.spills[0].contents[0].moles.0, 1.0);
    assert_eq!(b.vessels[0].contents[0].moles.0, 1.0);
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::SpillHazard { .. })));
    assert!(!events.iter().any(|e| matches!(e, Event::SafetyVeto { .. })));
}
#[test]
fn deliberate_discard_veto_is_atomic() {
    struct Veto;
    impl SafetyScreen for Veto {
        fn assess(&self, _: &Vessel) -> SafetyVerdict {
            SafetyVerdict::Veto {
                reason: "discard veto".into(),
            }
        }
    }
    let mut b = fixture(2.0, false);
    let before = serde_json::to_value(&b).unwrap();
    let mut solver = Solver::default();
    let events = b
        .step_with(
            Operator::Discard {
                vessel: VesselId(0),
            },
            &mut solver,
            &Veto,
        )
        .unwrap();
    let mut after = serde_json::to_value(&b).unwrap();
    after["log"] = before["log"].clone();
    assert_eq!(after, before);
    assert_eq!(solver.0, 0);
    assert!(matches!(&events[..], [Event::SafetyVeto { .. }]));
}
