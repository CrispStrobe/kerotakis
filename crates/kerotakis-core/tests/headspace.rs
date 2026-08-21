//! Finite headspace behavior that does not require a chemistry engine.

use kerotakis_core::*;

#[test]
fn old_vessels_deserialise_as_open() {
    let json = r#"{
        "elapsed_seconds": 0.0,
        "id": 0,
        "label": "beaker",
        "contents": [],
        "temperature": 298.15,
        "pressure": 101325.0,
        "thermal_mode": "adiabatic",
        "solute_charge": 0.0,
        "solution": null
    }"#;
    let vessel: Vessel = serde_json::from_str(json).expect("old vessel JSON remains readable");
    assert_eq!(vessel.headspace, Headspace::Open);
}

#[test]
fn script_parses_seal_and_open_with_explicit_units() {
    assert_eq!(
        script::parse_op("seal v1 250mL").unwrap(),
        Some(Operator::Seal {
            vessel: VesselId(0),
            headspace_volume: Liters(0.25),
        })
    );
    assert_eq!(
        script::parse_op("open v1").unwrap(),
        Some(Operator::Open {
            vessel: VesselId(0)
        })
    );
    assert!(script::parse_op("seal v1 0L").is_err());
    assert!(script::parse_op("seal v1 1g").is_err());
    assert_eq!(
        script::parse_op("regulate v1 1.5bar 250mL").unwrap(),
        Some(Operator::Regulate {
            vessel: VesselId(0),
            pressure: Pascal(150_000.0),
            initial_volume: Liters(0.25),
        })
    );
    assert_eq!(
        script::parse_op("sweep v1 90kPa").unwrap(),
        Some(Operator::Sweep {
            vessel: VesselId(0),
            pressure: Pascal(90_000.0),
        })
    );
    assert!(script::parse_op("regulate v1 0bar 1L").is_err());
    assert!(script::parse_op("sweep v1 1psi").is_err());
}

#[test]
fn sealing_traps_one_atmosphere_and_opening_releases_it() {
    let mut bench = Bench::new();
    let vessel = VesselId(0);
    let sealed = bench
        .step(Operator::Seal {
            vessel,
            headspace_volume: Liters(1.0),
        })
        .unwrap();
    assert!(sealed.iter().any(|event| matches!(
        event,
        Event::VesselSealed { trapped_air, .. } if trapped_air.0 > 0.04
    )));

    let state = bench.vessel(vessel).unwrap();
    assert!(state.is_sealed());
    assert!((state.pressure.0 - Pascal::ATMOSPHERIC.0).abs() < 2.0);
    assert!(state.gas_moles().0 > 0.04);
    let gas_mass = state.mass().0;

    let opened = bench.step(Operator::Open { vessel }).unwrap();
    let vented: f64 = opened
        .iter()
        .filter_map(|event| match event {
            Event::GasEvolved { moles, .. } => Some(moles.0),
            _ => None,
        })
        .sum();
    let state = bench.vessel(vessel).unwrap();
    assert_eq!(state.headspace, Headspace::Open);
    assert!((state.pressure.0 - Pascal::ATMOSPHERIC.0).abs() < 1e-9);
    assert!(state.gas_moles().0 < 1e-12);
    assert!(vented > 0.04);
    assert!(gas_mass > 1.0, "trapped air has mass, got {gas_mass} g");
}

#[test]
fn heating_a_sealed_gas_raises_pressure_in_proportion_to_temperature() {
    let mut bench = Bench::new();
    let vessel = VesselId(0);
    bench
        .step(Operator::Seal {
            vessel,
            headspace_volume: Liters(1.0),
        })
        .unwrap();
    let before = bench.vessel(vessel).unwrap().clone();
    bench
        .step(Operator::Heat {
            vessel,
            energy: Joules(50.0),
        })
        .unwrap();
    let after = bench.vessel(vessel).unwrap();
    let expected_temperature = before.temperature.0 + 50.0 / before.heat_capacity();
    assert!(
        (after.temperature.0 - expected_temperature).abs() < 1e-10,
        "a rigid gas uses Cv = Cp - R"
    );
    let pressure_ratio = after.pressure.0 / before.pressure.0;
    let temperature_ratio = after.temperature.0 / before.temperature.0;
    assert!((pressure_ratio - temperature_ratio).abs() < 1e-10);
    assert!(after.pressure.0 > before.pressure.0);
}

#[test]
fn a_pressure_controller_expands_the_headspace_when_heated() {
    let mut bench = Bench::new();
    let vessel = VesselId(0);
    bench
        .step(Operator::Regulate {
            vessel,
            pressure: Pascal(200_000.0),
            initial_volume: Liters(1.0),
        })
        .unwrap();
    let before = bench.vessel(vessel).unwrap().clone();
    bench
        .step(Operator::Heat {
            vessel,
            energy: Joules(50.0),
        })
        .unwrap();
    let after = bench.vessel(vessel).unwrap();
    let expected_temperature = before.temperature.0 + 50.0 / before.heat_capacity();
    assert!(
        (after.temperature.0 - expected_temperature).abs() < 1e-10,
        "a moving constant-pressure boundary uses Cp"
    );
    let (before_volume, after_volume) = (
        before.headspace_volume().unwrap().0,
        after.headspace_volume().unwrap().0,
    );
    assert!((after.pressure.0 - 200_000.0).abs() < 1e-9);
    assert!((after.gas_moles().0 - before.gas_moles().0).abs() < 1e-12);
    assert!(after_volume > before_volume);
    assert!(
        (after_volume / before_volume - after.temperature.0 / before.temperature.0).abs() < 1e-10
    );

    // The controller's expansion is the mechanical work that distinguishes
    // this path from rigid heating. P[Pa] * dV[m^3] = n R dT[J].
    let expansion_work = after.pressure.0 * (after_volume - before_volume) / 1000.0;
    let ideal_gas_work =
        before.gas_moles().0 * 8.314_462_618 * (after.temperature.0 - before.temperature.0);
    assert!((expansion_work - ideal_gas_work).abs() < 1e-10);
}

#[test]
fn trapped_air_takes_part_of_the_heat_that_an_open_liquid_keeps() {
    fn water_bench() -> Bench {
        let mut bench = Bench::new();
        bench
            .step(Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new("water"),
                moles: Moles(1.0),
                at: None,
            })
            .unwrap();
        bench
    }

    let vessel = VesselId(0);
    let mut open = water_bench();
    let mut sealed = water_bench();
    sealed
        .step(Operator::Seal {
            vessel,
            headspace_volume: Liters(10.0),
        })
        .unwrap();

    let open_capacity = open.vessel(vessel).unwrap().heat_capacity();
    let sealed_capacity = sealed.vessel(vessel).unwrap().heat_capacity();
    assert!(
        sealed_capacity > open_capacity,
        "the trapped gas must carry sensible energy"
    );

    for bench in [&mut open, &mut sealed] {
        bench
            .step(Operator::Heat {
                vessel,
                energy: Joules(1_000.0),
            })
            .unwrap();
    }
    let open_state = open.vessel(vessel).unwrap();
    let sealed_state = sealed.vessel(vessel).unwrap();
    assert!(
        sealed_state.temperature.0 < open_state.temperature.0,
        "the same heat warms liquid plus trapped air less than open liquid"
    );
    assert!((open_state.enthalpy().0 - 1_000.0).abs() < 1e-9);
    assert!((sealed_state.enthalpy().0 - 1_000.0).abs() < 1e-9);
    assert!(sealed_state.pressure.0 > Pascal::ATMOSPHERIC.0);
}

#[test]
fn rigid_and_pressure_controlled_gas_spend_the_same_heat_differently() {
    let vessel = VesselId(0);
    let mut sealed = Bench::new();
    let mut regulated = Bench::new();
    sealed
        .step(Operator::Seal {
            vessel,
            headspace_volume: Liters(1.0),
        })
        .unwrap();
    regulated
        .step(Operator::Regulate {
            vessel,
            pressure: Pascal::ATMOSPHERIC,
            initial_volume: Liters(1.0),
        })
        .unwrap();

    let sealed_before = sealed.vessel(vessel).unwrap().clone();
    let regulated_before = regulated.vessel(vessel).unwrap().clone();
    assert!((sealed_before.gas_moles().0 - regulated_before.gas_moles().0).abs() < 1e-12);
    assert!(sealed_before.heat_capacity() < regulated_before.heat_capacity());
    let capacity_gap = regulated_before.heat_capacity() - sealed_before.heat_capacity();
    let expected_gap = sealed_before.gas_moles().0 * 8.314_462_618;
    assert!(
        (capacity_gap - expected_gap).abs() < 1e-12,
        "Cp - Cv must equal nR for the trapped ideal gas"
    );

    for bench in [&mut sealed, &mut regulated] {
        bench
            .step(Operator::Heat {
                vessel,
                energy: Joules(10.0),
            })
            .unwrap();
    }
    let sealed_after = sealed.vessel(vessel).unwrap();
    let regulated_after = regulated.vessel(vessel).unwrap();
    assert!(sealed_after.temperature.0 > regulated_after.temperature.0);
    assert_eq!(
        sealed_after.headspace_volume(),
        sealed_before.headspace_volume(),
        "a sealed boundary is rigid"
    );
    assert!(
        regulated_after.headspace_volume().unwrap().0
            > regulated_before.headspace_volume().unwrap().0,
        "a pressure controller pays expansion work"
    );
}

#[test]
fn a_nitrogen_sweep_vents_owned_gas_and_holds_external_pressure() {
    let mut bench = Bench::new();
    let vessel = VesselId(0);
    bench
        .step(Operator::Seal {
            vessel,
            headspace_volume: Liters(1.0),
        })
        .unwrap();
    let events = bench
        .step(Operator::Sweep {
            vessel,
            pressure: Pascal(80_000.0),
        })
        .unwrap();
    let state = bench.vessel(vessel).unwrap();
    assert_eq!(
        state.headspace,
        Headspace::Swept {
            pressure: Pascal(80_000.0)
        }
    );
    assert_eq!(state.gas_moles(), Moles(0.0));
    assert_eq!(state.pressure, Pascal(80_000.0));
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::GasEvolved { .. })));
}

#[test]
fn a_curated_gas_product_stays_under_the_lid() {
    let mut bench = Bench::new();
    let vessel = VesselId(0);
    let mut solver = CuratedEquilibrator;
    bench
        .step_with(
            Operator::Seal {
                vessel,
                headspace_volume: Liters(1.0),
            },
            &mut solver,
            &PermissiveScreen,
        )
        .unwrap();
    for (species, moles) in [("NH3", 0.01), ("NaOCl", 0.01)] {
        let events = bench
            .step_with(
                Operator::Add {
                    vessel,
                    species: SpeciesId::new(species),
                    moles: Moles(moles),
                    at: None,
                },
                &mut solver,
                &PermissiveScreen,
            )
            .unwrap();
        if species == "NaOCl" {
            assert!(events.iter().any(|event| matches!(
                event,
                Event::GasContained { species, .. } if species.0 == "NH2Cl"
            )));
            assert!(!events.iter().any(|event| matches!(
                event,
                Event::GasEvolved { species, .. } if species.0 == "NH2Cl"
            )));
        }
    }
    let state = bench.vessel(vessel).unwrap();
    assert!(state.contents.iter().any(|portion| {
        portion.phase == Phase::Gas
            && portion.species.0 == "NH2Cl"
            && (portion.moles.0 - 0.01).abs() < 1e-12
    }));
}
