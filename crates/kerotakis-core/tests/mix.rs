//! Integration tests for the Mix operator.

use kerotakis_core::*;

#[test]
fn mix_transfers_correct_fractions() {
    let mut bench = Bench::new();
    bench.step(Operator::NewVessel { kind: None }).unwrap(); // v2
    bench.step(Operator::NewVessel { kind: None }).unwrap(); // v3

    bench
        .step(Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new("water"),
            moles: Moles(10.0),
            at: Some(Kelvin(293.15)),
        })
        .unwrap();
    bench
        .step(Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new("NaCl"),
            moles: Moles(0.1),
            at: None,
        })
        .unwrap();

    bench
        .step(Operator::Add {
            vessel: VesselId(1),
            species: SpeciesId::new("water"),
            moles: Moles(20.0),
            at: Some(Kelvin(353.15)),
        })
        .unwrap();
    bench
        .step(Operator::Add {
            vessel: VesselId(1),
            species: SpeciesId::new("KCl"),
            moles: Moles(0.2),
            at: None,
        })
        .unwrap();

    let mass_before: f64 = bench.vessels.iter().map(|v| v.mass().0).sum();
    let source_a_temperature = bench.vessels[0].temperature.0;
    let source_b_temperature = bench.vessels[1].temperature.0;

    let events = bench
        .step(Operator::Mix {
            a: VesselId(0),
            b: VesselId(1),
            into: VesselId(2),
            fraction_a: 0.5,
            fraction_b: 0.5,
        })
        .unwrap();

    let mass_after: f64 = bench.vessels.iter().map(|v| v.mass().0).sum();
    assert!(
        (mass_before - mass_after).abs() < 1e-9,
        "mass must be conserved: {mass_before} vs {mass_after}"
    );

    assert!(
        events.iter().any(|e| matches!(e, Event::Mixed { .. })),
        "must emit a Mixed event"
    );
    let thermal_evidence = events.iter().find_map(|event| match event {
        Event::Mixed {
            temperature_a,
            temperature_b,
            temperature_into,
            ..
        } => Some((temperature_a.0, temperature_b.0, temperature_into.0)),
        _ => None,
    });
    let (ta, tb, result) = thermal_evidence.expect("mixed event carries thermal evidence");
    assert!((ta - source_a_temperature).abs() < 1e-9);
    assert!((tb - source_b_temperature).abs() < 1e-9);
    let low = ta.min(tb);
    let high = ta.max(tb);
    assert!(
        result > low && result < high,
        "computed result must lie between its sources"
    );

    // Source vessels should have half their original liquid remaining.
    let v1_water: f64 = bench.vessels[0]
        .contents
        .iter()
        .filter(|p| p.species.0 == "water")
        .map(|p| p.moles.0)
        .sum();
    assert!(
        (v1_water - 5.0).abs() < 1e-9,
        "v1 should retain 50% of water: {v1_water}"
    );
}

#[test]
fn mix_rejects_same_source_and_target() {
    let mut bench = Bench::new();
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    bench
        .step(Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new("water"),
            moles: Moles(1.0),
            at: None,
        })
        .unwrap();
    bench
        .step(Operator::Add {
            vessel: VesselId(1),
            species: SpeciesId::new("water"),
            moles: Moles(1.0),
            at: None,
        })
        .unwrap();

    let result = bench.step(Operator::Mix {
        a: VesselId(0),
        b: VesselId(1),
        into: VesselId(0),
        fraction_a: 0.5,
        fraction_b: 0.5,
    });
    assert!(result.is_err(), "mix into a source vessel must fail");
}

#[test]
fn mix_rejects_same_sources() {
    let mut bench = Bench::new();
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    bench
        .step(Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new("water"),
            moles: Moles(1.0),
            at: None,
        })
        .unwrap();

    let result = bench.step(Operator::Mix {
        a: VesselId(0),
        b: VesselId(0),
        into: VesselId(2),
        fraction_a: 0.5,
        fraction_b: 0.5,
    });
    assert!(result.is_err(), "mixing a vessel with itself must fail");
}

#[test]
fn mix_rejects_bad_fraction() {
    let mut bench = Bench::new();
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    bench
        .step(Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new("water"),
            moles: Moles(1.0),
            at: None,
        })
        .unwrap();
    bench
        .step(Operator::Add {
            vessel: VesselId(1),
            species: SpeciesId::new("water"),
            moles: Moles(1.0),
            at: None,
        })
        .unwrap();

    let result = bench.step(Operator::Mix {
        a: VesselId(0),
        b: VesselId(1),
        into: VesselId(2),
        fraction_a: 1.5,
        fraction_b: 0.5,
    });
    assert!(result.is_err(), "fraction > 1 must fail");
}

#[test]
fn mix_verb_parses() {
    let op = kerotakis_core::script::parse_op("mix v1 0.5 v2 0.5 into v3")
        .unwrap()
        .unwrap();
    assert_eq!(
        op,
        Operator::Mix {
            a: VesselId(0),
            b: VesselId(1),
            into: VesselId(2),
            fraction_a: 0.5,
            fraction_b: 0.5,
        }
    );
}

#[test]
fn mix_adiabatic_temperature_balance() {
    let mut bench = Bench::new();
    bench.step(Operator::NewVessel { kind: None }).unwrap(); // v2
    bench.step(Operator::NewVessel { kind: None }).unwrap(); // v3

    bench
        .step(Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new("water"),
            moles: Moles(10.0),
            at: Some(Kelvin::from_celsius(80.0)),
        })
        .unwrap();
    bench
        .step(Operator::Add {
            vessel: VesselId(1),
            species: SpeciesId::new("water"),
            moles: Moles(10.0),
            at: Some(Kelvin::from_celsius(20.0)),
        })
        .unwrap();

    bench
        .step(Operator::Mix {
            a: VesselId(0),
            b: VesselId(1),
            into: VesselId(2),
            fraction_a: 1.0,
            fraction_b: 1.0,
        })
        .unwrap();

    let t_mixed = bench.vessel(VesselId(2)).unwrap().temperature;
    // 50/50 by mass at 80°C and 20°C → 50°C.
    assert!(
        (t_mixed.to_celsius() - 50.0).abs() < 1.0,
        "mixed temperature should be ~50°C, got {:.2}°C",
        t_mixed.to_celsius()
    );
}
