use kerotakis_core::*;

fn bench_with(ops: &[Operator]) -> (Bench, Vec<Event>) {
    let mut bench = Bench::new();
    let mut events = Vec::new();
    for op in ops {
        events.extend(bench.step(op.clone()).expect("operator applies"));
    }
    (bench, events)
}

fn moles_in(bench: &Bench, vessel: usize, key: &str) -> f64 {
    bench.vessels[vessel]
        .contents
        .iter()
        .filter(|p| p.species.0 == key)
        .map(|p| p.moles.0)
        .sum()
}

#[test]
fn dilute_adds_water_by_volume() {
    let (bench, events) = bench_with(&[
        Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new("NaCl"),
            moles: Moles(0.1),
            at: None,
        },
        Operator::Dilute {
            vessel: VesselId(0),
            volume: Liters(0.1),
        },
    ]);
    let water = moles_in(&bench, 0, "water");
    assert!(
        water > 5.0,
        "100 mL should deposit several moles of water, got {water}"
    );
    assert!(
        events.iter().any(|e| matches!(e, Event::Diluted { .. })),
        "expected a Diluted event"
    );
}

#[test]
fn dilute_is_monotone_in_water() {
    let mut bench = Bench::new();
    bench
        .step(Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new("NaCl"),
            moles: Moles(0.1),
            at: None,
        })
        .unwrap();

    let before = moles_in(&bench, 0, "water");
    bench
        .step(Operator::Dilute {
            vessel: VesselId(0),
            volume: Liters(0.05),
        })
        .unwrap();
    let after = moles_in(&bench, 0, "water");
    assert!(
        after > before,
        "dilute must increase water: {before} → {after}"
    );

    bench
        .step(Operator::Dilute {
            vessel: VesselId(0),
            volume: Liters(0.1),
        })
        .unwrap();
    let final_w = moles_in(&bench, 0, "water");
    assert!(
        final_w > after,
        "second dilute must further increase water: {after} → {final_w}"
    );
}

#[test]
fn dilute_conserves_solute() {
    let mut bench = Bench::new();
    bench
        .step(Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new("NaCl"),
            moles: Moles(0.5),
            at: None,
        })
        .unwrap();

    let salt_before = moles_in(&bench, 0, "NaCl");
    bench
        .step(Operator::Dilute {
            vessel: VesselId(0),
            volume: Liters(0.2),
        })
        .unwrap();
    let salt_after = moles_in(&bench, 0, "NaCl");
    assert!(
        (salt_after - salt_before).abs() < 1e-12,
        "dilute must not change solute: {salt_before} → {salt_after}"
    );
}

#[test]
fn dilute_parsed_from_script() {
    let op = kerotakis_core::script::parse_op("dilute v1 100mL")
        .unwrap()
        .unwrap();
    match op {
        Operator::Dilute { vessel, volume } => {
            assert_eq!(vessel, VesselId(0));
            assert!((volume.0 - 0.1).abs() < 1e-12, "100 mL = 0.1 L");
        }
        other => panic!("expected Dilute, got {other:?}"),
    }
}

#[test]
fn titrate_parsed_from_script() {
    let op = kerotakis_core::script::parse_op("titrate v1 NaOH 1mL until ph 7")
        .unwrap()
        .unwrap();
    match op {
        Operator::Titrate {
            vessel,
            titrant,
            concentration,
            step,
            target_ph,
            max_steps,
            endpoint,
        } => {
            assert_eq!(
                endpoint,
                kerotakis_core::ops::Endpoint::Ph,
                "CAP-12's line still means the pH endpoint"
            );
            assert_eq!(vessel, VesselId(0));
            assert_eq!(titrant, SpeciesId::new("NaOH"));
            assert!(
                (concentration - 1.0).abs() < 1e-12,
                "no stated molarity means the 1 mol/L standard"
            );
            assert!((step.0 - 0.001).abs() < 1e-12, "1 mL = 0.001 L");
            assert!((target_ph - 7.0).abs() < 1e-12);
            assert_eq!(max_steps, 100);
        }
        other => panic!("expected Titrate, got {other:?}"),
    }
}

#[test]
fn titrate_parsed_with_max() {
    let op = kerotakis_core::script::parse_op("titrate v2 HCl 0.5mL until ph 3 max 50")
        .unwrap()
        .unwrap();
    match op {
        Operator::Titrate {
            vessel,
            titrant,
            max_steps,
            ..
        } => {
            assert_eq!(vessel, VesselId(1));
            assert_eq!(titrant, SpeciesId::new("HCl"));
            assert_eq!(max_steps, 50);
        }
        other => panic!("expected Titrate, got {other:?}"),
    }
}
