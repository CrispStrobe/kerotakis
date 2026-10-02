use kerotakis_core::*;
fn run(b: &mut Bench, line: &str) -> Vec<Event> {
    b.step(script::parse_op(line).unwrap().unwrap()).unwrap()
}
#[test]
fn visible_oil_layer_drains_water_and_conserves_oil_and_salt() {
    let mut b = Bench::new();
    for line in ["add v1 water 50mL", "add v1 vegetable_oil 50mL", "new"] {
        run(&mut b, line);
    }
    b.vessels[0].deposit(SpeciesId::new("Na+"), Moles(0.001), Phase::Aqueous);
    b.vessels[0].deposit(SpeciesId::new("Cl-"), Moles(0.001), Phase::Aqueous);
    assert!(render_vessel(&b.vessels[0], Register(3))
        .join("\n")
        .contains("100.0 mL"));
    let mass_before: f64 = b.vessels.iter().map(|v| v.mass().0).sum();
    let events = run(&mut b, "drain v1 v2");
    assert!(
        events.iter().any(|e| matches!(e, Event::Drained { .. })),
        "{events:?}"
    );
    let src = b.vessel(VesselId(0)).unwrap();
    let dst = b.vessel(VesselId(1)).unwrap();
    assert!(!src.contents.iter().any(|p| p.species.0 == "water"));
    assert!(dst
        .contents
        .iter()
        .any(|p| p.species.0 == "Na+" && p.moles.0 == 0.001));
    assert_eq!(src.unresolved_materials.len(), 1);
    assert!(dst.unresolved_materials.is_empty());
    assert!((b.vessels.iter().map(|v| v.mass().0).sum::<f64>() - mass_before).abs() < 1e-8);
}
#[test]
fn oil_drain_does_not_guess_neutral_solute_partition() {
    let mut b = Bench::new();
    for line in [
        "add v1 water 50mL",
        "add v1 vegetable_oil 50mL",
        "add v1 ethanol 1mL",
        "new",
    ] {
        run(&mut b, line);
    }
    let before = b.vessel(VesselId(0)).unwrap().clone();
    let events = run(&mut b, "drain v1 v2");
    assert_eq!(
        serde_json::to_value(b.vessel(VesselId(0)).unwrap()).unwrap(),
        serde_json::to_value(&before).unwrap()
    );
    assert!(
        events.iter().any(
            |e| matches!(e, Event::NotYetModeled { reason: Some(reason), .. }
        if reason.key == "not-modeled.material-layer-partition")
        ),
        "{events:?}"
    );
}
#[test]
fn incomplete_temperature_survives_serialization_and_a_pour() {
    let mut b = Bench::new();
    run(&mut b, "add v1 water 50mL");
    run(&mut b, "new");
    b.vessels[0]
        .unpriced_heat
        .push(SpeciesId::new("chalcanthite"));
    assert!(render_vessel(&b.vessels[0], Register(3))
        .join("\n")
        .contains("incomplete"));
    let encoded = serde_json::to_string(&b).unwrap();
    let mut b: Bench = serde_json::from_str(&encoded).unwrap();
    run(&mut b, "decant v1 v2 0.5");
    let events = run(&mut b, "measure v2 thermometer");
    for level in [1, 2, 3] {
        assert!(render_events_in(&events, Register(level), Locale::EN)
            .join("\n")
            .contains("incomplete"));
    }
}
#[test]
fn layered_ph_and_conductivity_readings_name_the_aqueous_layer() {
    let mut b = Bench::new();
    for line in ["add v1 water 50mL", "add v1 vegetable_oil 50mL"] {
        run(&mut b, line);
    }
    // The core-only bench has no aqueous engine; supply a characterized
    // solution to exercise measurement scope rather than solver capability.
    b.vessels[0].solution = Some(
        serde_json::from_value(serde_json::json!({
            "ph": 7.0, "ionic_strength": 1e-7, "pe": null
        }))
        .unwrap(),
    );
    for instrument in ["ph", "conductivity"] {
        let events = run(&mut b, &format!("measure v1 {instrument}"));
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::Measured { note: Some(note), .. }
            if note.contains("aqueous layer"))),
            "{events:?}"
        );
    }
}

#[test]
fn transactional_commit_preserves_temperature_limitations() {
    let before = Vessel::new(VesselId(0), "beaker");
    let mut after = before.clone();
    after.unpriced_heat.push(SpeciesId::new("chalcanthite"));
    let delta = orchestrator::diff_vessels(&before, &after, "heat audit");
    assert!(!delta.is_empty());
    let mut committed = before;
    delta.apply(&mut committed);
    assert_eq!(committed.unpriced_heat, after.unpriced_heat);
}
#[test]
fn spill_recovery_preserves_temperature_limitations() {
    let mut b = Bench::new();
    run(&mut b, "add v1 water 50mL");
    run(&mut b, "new");
    b.vessels[0]
        .unpriced_heat
        .push(SpeciesId::new("chalcanthite"));
    let destination = authority::SpillDestination::Bench {
        zone: "heat audit".into(),
    };
    b.step(Operator::Spill {
        from: VesselId(0),
        destination: destination.clone(),
        fraction: 0.5,
        replay_seed: 73,
    })
    .unwrap();
    b.step(Operator::RecoverSpill {
        destination,
        to: VesselId(1),
        fraction: 1.0,
    })
    .unwrap();
    assert!(b.vessels[1].temperature_limitation().is_some());
    b.vessels[1].thermal_mode = ThermalMode::Thermostatted(Kelvin::STANDARD);
    assert!(b.vessels[1].temperature_limitation().is_none());
}
