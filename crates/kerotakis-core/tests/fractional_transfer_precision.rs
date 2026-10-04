//! Scalar fractional transfers must account for both ends of the same split.
use kerotakis_core::*;
#[derive(Default)]
struct Counter(usize);
impl Equilibrator for Counter {
    fn name(&self) -> &'static str {
        "fractional-transfer-counter"
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.0 += 1;
        v.temperature.0 += 1.0;
        Ok(Vec::new())
    }
}
fn fixture(amount: f64, material_only: bool) -> Bench {
    let mut b = Bench::new();
    if material_only {
        let oil = material::lookup("olive_oil", None).unwrap();
        b.vessels[0]
            .unresolved_materials
            .push(vessel::UnresolvedMaterialPortion {
                material: oil.canonical_key.clone(),
                recipe_id: oil.id.clone(),
                recipe_version: oil.version,
                basis: oil.basis,
                amount,
                enzyme_hydrolysis: None,
                protein_denatured_fraction: 0.0,
            });
    } else {
        b.vessels[0].deposit(SpeciesId::new("water"), Moles(amount), Phase::Liquid);
    }
    b.vessels.push(Vessel::new(VesselId(1), "empty-source"));
    b.vessels.push(Vessel::new(VesselId(2), "receiver"));
    b
}
fn request(fraction: f64, mix: bool) -> Operator {
    if mix {
        Operator::Mix {
            a: VesselId(0),
            b: VesselId(1),
            into: VesselId(2),
            fraction_a: fraction,
            fraction_b: 0.0,
        }
    } else {
        Operator::Decant {
            from: VesselId(0),
            to: VesselId(2),
            fraction,
        }
    }
}
fn refused(mut b: Bench, op: Operator) {
    let before = serde_json::to_value(&b).unwrap();
    let mut counter = Counter::default();
    assert!(b.step_with(op, &mut counter, &PermissiveScreen).is_err());
    assert_eq!(serde_json::to_value(&b).unwrap(), before);
    assert_eq!(counter.0, 0);
}
fn refusal_matrix(mix: bool, material: bool) {
    for fraction in [1e-20, 1e-16, 1e-14] {
        refused(fixture(1.0, material), request(fraction, mix));
    }
}
fn quantum_matrix(mix: bool, material: bool) {
    let quantum = f64::from_bits(1);
    for (amount, fraction) in [(quantum, 0.5), (5.0 * quantum, 0.5), (22.0 * quantum, 0.25)] {
        refused(fixture(amount, material), request(fraction, mix));
    }
}
#[test]
fn decant_refuses_zero_or_inaccurate_donor_debits() {
    refusal_matrix(false, false);
}
#[test]
fn mix_refuses_zero_or_inaccurate_donor_debits() {
    refusal_matrix(true, false);
}
#[test]
fn decant_refuses_underflow_and_inaccurate_quantum_fractions() {
    quantum_matrix(false, false);
}
#[test]
fn mix_refuses_underflow_and_inaccurate_quantum_fractions() {
    quantum_matrix(true, false);
}
#[test]
fn unresolved_decant_refuses_zero_or_inaccurate_debits() {
    refusal_matrix(false, true);
}
#[test]
fn unresolved_mix_refuses_zero_or_inaccurate_debits() {
    refusal_matrix(true, true);
}
#[test]
fn unresolved_decant_refuses_underflow_and_quantum_errors() {
    quantum_matrix(false, true);
}
#[test]
fn unresolved_mix_refuses_underflow_and_quantum_errors() {
    quantum_matrix(true, true);
}
#[test]
fn second_source_refusal_restores_both_sources_and_receiver() {
    let mut b = fixture(2.0, false);
    b.vessels[1].deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    refused(
        b,
        Operator::Mix {
            a: VesselId(0),
            b: VesselId(1),
            into: VesselId(2),
            fraction_a: 0.5,
            fraction_b: 1e-20,
        },
    );
}
#[test]
fn later_component_refusal_is_atomic() {
    let mut b = fixture(2.0, false);
    b.vessels[0].deposit(
        SpeciesId::new("NaCl"),
        Moles(5.0 * f64::from_bits(1)),
        Phase::Aqueous,
    );
    refused(b, request(0.5, false));
}
#[test]
fn donor_refusal_removes_an_automatically_created_receiver() {
    let mut b = fixture(1.0, false);
    b.vessels.truncate(1);
    refused(b, request(1e-20, false));
}
fn stock(b: &Bench, index: usize, material: bool) -> f64 {
    if material {
        b.vessels[index]
            .unresolved_materials
            .iter()
            .map(|p| p.amount)
            .sum()
    } else {
        b.vessels[index].moles_of(&SpeciesId::new("water")).0
    }
}
#[test]
fn represented_splits_keep_requested_fraction_debit_and_residue() {
    for material in [false, true] {
        for mix in [false, true] {
            for (amount, fraction) in [
                (1.0, 0.25),
                (1e-100, 0.5),
                (8.0 * f64::from_bits(1), 0.5),
                (f64::from_bits(1), 1.0),
                (1.0, 0.0),
            ] {
                let mut b = fixture(amount, material);
                b.step_with(
                    request(fraction, mix),
                    &mut SolverStack::new(vec![]),
                    &PermissiveScreen,
                )
                .unwrap();
                assert_eq!(stock(&b, 2, material), amount * fraction);
                assert_eq!(stock(&b, 0, material), amount - amount * fraction);
            }
        }
    }
}
