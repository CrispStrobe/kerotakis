//! Supported counterparts of the frozen receiver refusal contracts.
use kerotakis_core::refusal::Refuses;
use kerotakis_core::*;

fn bench(species: &str, phase: Phase, amount: f64, bulk: f64) -> Bench {
    let mut b = Bench::new();
    b.vessels[0].deposit(SpeciesId::new(species), Moles(amount), phase);
    b.vessels.push(Vessel::new(VesselId(1), "receiver"));
    b.vessels[1].deposit(SpeciesId::new(species), Moles(bulk), phase);
    b
}
fn transfer(b: &mut Bench, op: Operator) {
    b.step_with(op, &mut SolverStack::new(vec![]), &PermissiveScreen)
        .unwrap();
}
#[test]
fn magnetic_and_filtered_traces_survive_empty_receivers() {
    for amount in [1e-100, 1e-20, 0.25] {
        for (species, phase, op) in [
            (
                "Fe",
                Phase::Solid,
                Operator::Magnet {
                    from: VesselId(0),
                    to: VesselId(1),
                },
            ),
            (
                "NaCl",
                Phase::Aqueous,
                Operator::Filter {
                    from: VesselId(0),
                    to: VesselId(1),
                },
            ),
        ] {
            let mut b = bench(species, phase, amount, 0.0);
            transfer(&mut b, op);
            assert_eq!(b.vessels[1].moles_of(&SpeciesId::new(species)).0, amount);
            assert_eq!(b.vessels[0].moles_of(&SpeciesId::new(species)).0, 0.0);
        }
    }
}
#[test]
fn accurate_preloaded_magnetic_and_filtered_receivers_work() {
    for (species, phase, op) in [
        (
            "Fe",
            Phase::Solid,
            Operator::Magnet {
                from: VesselId(0),
                to: VesselId(1),
            },
        ),
        (
            "NaCl",
            Phase::Aqueous,
            Operator::Filter {
                from: VesselId(0),
                to: VesselId(1),
            },
        ),
    ] {
        let mut b = bench(species, phase, 0.25, 1.0);
        transfer(&mut b, op);
        assert_eq!(b.vessels[1].moles_of(&SpeciesId::new(species)).0, 1.25);
    }
}
#[test]
fn mixed_equal_traces_are_representable() {
    let mut b = bench("water", Phase::Liquid, 1e-20, 1e-20);
    b.vessels.push(Vessel::new(VesselId(2), "receiver"));
    transfer(
        &mut b,
        Operator::Mix {
            a: VesselId(0),
            b: VesselId(1),
            into: VesselId(2),
            fraction_a: 1.0,
            fraction_b: 1.0,
        },
    );
    assert_eq!(b.vessels[2].moles_of(&SpeciesId::new("water")).0, 2e-20);
}
#[test]
fn draining_retains_trace_solute_and_supported_bulk_increment() {
    for (amount, bulk) in [(1e-20, 0.0), (0.25, 1.0)] {
        let mut b = bench("NaCl", Phase::Aqueous, amount, bulk);
        b.vessels[0].deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
        b.vessels[0].deposit(SpeciesId::new("hexane"), Moles(1.0), Phase::Liquid);
        transfer(
            &mut b,
            Operator::Drain {
                from: VesselId(0),
                to: VesselId(1),
            },
        );
        assert_eq!(
            b.vessels[1].moles_of(&SpeciesId::new("NaCl")).0,
            bulk + amount
        );
        assert_eq!(b.vessels[0].moles_of(&SpeciesId::new("hexane")).0, 1.0);
    }
}
#[test]
fn precision_error_has_structured_species_and_readable_explanation() {
    let mut b = bench("Fe", Phase::Solid, 1e-20, 1.0);
    let error = b
        .step_with(
            Operator::Magnet {
                from: VesselId(0),
                to: VesselId(1),
            },
            &mut SolverStack::new(vec![]),
            &PermissiveScreen,
        )
        .unwrap_err();
    assert!(matches!(error, BenchError::TransferPrecision { .. }));
    assert_eq!(error.refusal().key, "error.transfer-precision");
    assert!(error.to_string().contains("Fe"));
}
