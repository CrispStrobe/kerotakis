//! Supplementary controls isolate donor refusals from the earlier receiver guard.
use kerotakis_core::refusal::Refuses;
use kerotakis_core::*;
#[derive(Default)]
struct Counter(usize);
impl Equilibrator for Counter {
    fn name(&self) -> &'static str {
        "fractional-independent-counter"
    }
    fn equilibrate(&mut self, _: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.0 += 1;
        Ok(Vec::new())
    }
}
fn bench() -> Bench {
    let mut b = Bench::new();
    b.vessels[0].deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
    b.vessels.push(Vessel::new(VesselId(1), "second-source"));
    b.vessels.push(Vessel::new(VesselId(2), "receiver"));
    b
}
#[test]
fn different_species_second_source_refuses_before_any_commit() {
    let mut b = bench();
    b.vessels[1].deposit(SpeciesId::new("NaCl"), Moles(1.0), Phase::Aqueous);
    let before = serde_json::to_value(&b).unwrap();
    let mut counter = Counter::default();
    let error = b
        .step_with(
            Operator::Mix {
                a: VesselId(0),
                b: VesselId(1),
                into: VesselId(2),
                fraction_a: 0.5,
                fraction_b: 1e-20,
            },
            &mut counter,
            &PermissiveScreen,
        )
        .unwrap_err();
    assert!(matches!(error, BenchError::TransferDonorPrecision { .. }));
    assert_eq!(error.refusal().key, "error.transfer-donor-precision");
    assert!(error.to_string().contains("NaCl"));
    assert_eq!(serde_json::to_value(&b).unwrap(), before);
    assert_eq!(counter.0, 0);
}
#[test]
fn near_full_splits_require_a_representable_requested_tail() {
    let fraction = 1.0 - f64::EPSILON;
    let mut b = bench();
    b.vessels[0].contents[0].moles = Moles(1.1);
    let before = serde_json::to_value(&b).unwrap();
    let mut counter = Counter::default();
    let op = Operator::Decant {
        from: VesselId(0),
        to: VesselId(2),
        fraction,
    };
    assert!(matches!(
        b.step_with(op, &mut counter, &PermissiveScreen),
        Err(BenchError::TransferDonorPrecision { .. })
    ));
    assert_eq!(serde_json::to_value(&b).unwrap(), before);
    assert_eq!(counter.0, 0);
    b.vessels[0].contents[0].moles = Moles(1.0);
    b.step_with(
        Operator::Decant {
            from: VesselId(0),
            to: VesselId(2),
            fraction,
        },
        &mut SolverStack::new(vec![]),
        &PermissiveScreen,
    )
    .unwrap();
    assert_eq!(
        b.vessels[0].moles_of(&SpeciesId::new("water")).0,
        f64::EPSILON
    );
    assert_eq!(b.vessels[2].moles_of(&SpeciesId::new("water")).0, fraction);
}
