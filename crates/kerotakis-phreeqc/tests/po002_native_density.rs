//! The unchanged PO002 original: density feasibility, not particle separation.
#![cfg(feature = "engine")]
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn emit(name: &str, value: &serde_json::Value) {
    if let Some(dir) = std::env::var_os("KEROTAKIS_PO002_EVIDENCE_DIR") {
        std::fs::write(
            std::path::PathBuf::from(dir).join(name),
            serde_json::to_vec_pretty(value).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn unchanged_original_compares_water_and_brine_with_ps_and_pet() {
    let original: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/po002-original-20261016.json")).unwrap();
    let mut bench = Bench::new();
    let mut stack = kerotakis_stack::standard_stack(vec![Box::new(PhaseEquilibrator::wrapping(
        Box::new(PhreeqcEquilibrator::new().unwrap()),
    ))]);
    let mut frames = Vec::new();
    let mut measurements = Vec::new();
    let mut solver_failed = false;
    for line in original["script"].as_str().unwrap().lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let command = script::parse_command(line, Locale::EN).unwrap();
        let before = serde_json::to_value(&bench.vessels).unwrap();
        let events = match command.operator {
            Some(op) => bench.step_with(op, &mut stack, &PermissiveScreen).unwrap(),
            None => {
                assert!(matches!(
                    command.canonical.as_str(),
                    "register lv3" | "inspect"
                ));
                Vec::new()
            }
        };
        solver_failed |= events
            .iter()
            .any(|e| matches!(e, Event::SolverFailed { .. }));
        for event in &events {
            if let Event::Measured { value, unit, .. } = event {
                if unit == "g/mL" {
                    measurements.push(*value);
                }
            }
        }
        frames.push(serde_json::json!({"command":line,"canonical":command.canonical,"before":before,"after":bench.vessels,"events":events}));
    }
    emit(
        "original.json",
        &serde_json::json!({"original":original,"frames":frames,"measurements":measurements,"after":bench.vessels}),
    );
    assert_eq!(measurements.len(), 2);
    assert!(!solver_failed, "native solver failure is not qualification");
    let ps = species::lookup(&SpeciesId::new("PS")).unwrap().density;
    let pet = species::lookup(&SpeciesId::new("PET")).unwrap().density;
    assert!(
        measurements[0] < ps && ps < measurements[1] && measurements[1] < pet,
        "water={}, PS={ps}, brine={}, PET={pet}",
        measurements[0],
        measurements[1]
    );
    assert!(!buoyancy::ionic_volume_unaccounted(&bench.vessels[1]));
    let native =
        buoyancy::native_liquid_density(&bench.vessels[1]).expect("native volume authority");
    assert!(native.dataset.contains("pitzer"));
    for (i, expected_ps_float) in [(0, false), (1, true)] {
        assert_eq!(
            buoyancy::floats_in(&SpeciesId::new("PS"), measurements[i]),
            Some(expected_ps_float)
        );
        assert_eq!(
            buoyancy::floats_in(&SpeciesId::new("PET"), measurements[i]),
            Some(false)
        );
        assert!(
            (bench.vessels[i].moles_of(&SpeciesId::new("PS")).0
                * species::lookup(&SpeciesId::new("PS")).unwrap().molar_mass
                - 0.5)
                .abs()
                < 1e-10
        );
        assert!(
            (bench.vessels[i].moles_of(&SpeciesId::new("PET")).0
                * species::lookup(&SpeciesId::new("PET")).unwrap().molar_mass
                - 0.5)
                .abs()
                < 1e-10
        );
    }
    // Scientific readouts do not move the exact retained material.
    for frame in &frames {
        if frame["command"].as_str().unwrap().starts_with("measure")
            || frame["command"].as_str().unwrap().starts_with("look")
            || frame["command"] == "inspect"
        {
            assert_eq!(frame["before"], frame["after"]);
        }
    }
    let v = &bench.vessels[1];
    let mut changed = v.clone();
    changed.temperature.0 += 1.0;
    assert!(buoyancy::native_liquid_density(&changed).is_none());
    assert!(buoyancy::ionic_volume_unaccounted(&changed));
    changed = v.clone();
    changed.pressure.0 += 1.0;
    assert!(buoyancy::native_liquid_density(&changed).is_none());
    let mut changed = v.clone();
    changed
        .contents
        .iter_mut()
        .find(|p| p.species.0 == "water")
        .unwrap()
        .moles
        .0 += 0.01;
    assert!(buoyancy::native_liquid_density(&changed).is_none());
    let mut changed = v.clone();
    changed.contents.push(vessel::Portion {
        species: SpeciesId::new("sucrose"),
        moles: Moles(0.01),
        phase: Phase::Aqueous,
    });
    assert!(buoyancy::native_liquid_density(&changed).is_none());
    assert!(
        buoyancy::NativeLiquidDensity::capture(v, f64::NAN, "synthetic negative".into()).is_none()
    );
    assert!(buoyancy::NativeLiquidDensity::capture(
        &changed,
        native.kg_per_l,
        native.dataset.clone()
    )
    .is_none());
}

#[test]
fn other_electrolytes_do_not_inherit_sodium_chloride_volume_authority() {
    let mut bench = Bench::new();
    let mut stack =
        kerotakis_stack::standard_stack(vec![Box::new(PhreeqcEquilibrator::new().unwrap())]);
    let mut frames = Vec::new();
    for line in ["add v1 water 50mL", "add v1 KCl 8g", "measure v1 density"] {
        let op = script::parse_op(line).unwrap().unwrap();
        let events = bench.step_with(op, &mut stack, &PermissiveScreen).unwrap();
        assert!(!events
            .iter()
            .any(|e| matches!(e, Event::SolverFailed { .. })));
        frames.push(serde_json::json!({"command":line,"events":events,"after":bench.vessels}));
    }
    emit(
        "unsupported-electrolyte.json",
        &serde_json::json!({"frames":frames}),
    );
    assert!(buoyancy::native_liquid_density(&bench.vessels[0]).is_none());
    assert!(buoyancy::ionic_volume_unaccounted(&bench.vessels[0]));
}
