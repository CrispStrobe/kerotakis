use kerotakis_core::*;

fn run(bench: &mut Bench, line: &str) -> Vec<Event> {
    bench
        .step(script::parse_op(line).unwrap().unwrap())
        .unwrap()
}

#[test]
fn olive_oil_is_a_distinct_unresolved_material_with_documented_reference_density() {
    let olive = material::lookup("olive_oil", None).unwrap();
    let generic = material::lookup("vegetable_oil", None).unwrap();
    assert_ne!(olive.id, generic.id);
    for (query, language) in [
        ("olive oil", "en"),
        ("Olivenöl", "de"),
        ("huile d'olive", "fr"),
    ] {
        assert_eq!(
            material::lookup(query, Some(language)).unwrap().id,
            olive.id
        );
    }
    assert!(olive.components.is_empty());
    let density = olive.bulk_density.unwrap();
    assert_eq!(density.value, 0.9161);
    assert_eq!(density.conditions.temperature.unwrap().lower, 293.15);
    assert_eq!(density.source_id, "literature/odeh-2015-olive-oil-density");
    assert!(olive
        .lot_assumptions
        .iter()
        .any(|note| note.contains("heat capacity are not modelled")));
}

#[test]
fn original_oil_water_and_drain_inputs_preserve_mass_and_named_oil_identity() {
    let mut bench = Bench::new();
    run(&mut bench, "add v1 water 50mL");
    run(&mut bench, "add v1 olive_oil 50mL");
    run(&mut bench, "new");
    let source = &bench.vessels[0];
    assert_eq!(source.unresolved_materials.len(), 1);
    assert_eq!(
        source.unresolved_materials[0].recipe_id,
        "household/olive-oil-surrogate"
    );
    assert!((source.unresolved_materials[0].amount - 45.805).abs() < 1e-10);
    let scene = scene_vessel(source);
    assert_eq!(scene.layers.len(), 2);
    assert_eq!(scene.layers[0].species, "solution");
    assert!((scene.layers[1].volume_l - 0.05).abs() < 1e-10);
    assert!(render_vessel(source, Register(3))
        .join("\n")
        .contains("100.0 mL"));
    let total_before: f64 = bench.vessels.iter().map(|v| v.mass().0).sum();
    let events = run(&mut bench, "drain v1 v2");
    assert!(events.iter().any(|e| matches!(e, Event::Drained { .. })));
    assert_eq!(bench.vessels[0].unresolved_materials.len(), 1);
    assert!(bench.vessels[1].unresolved_materials.is_empty());
    assert!(bench.vessels[1].moles_of(&SpeciesId::new("water")).0 > 0.0);
    assert!((bench.vessels.iter().map(|v| v.mass().0).sum::<f64>() - total_before).abs() < 1e-8);
    let restored: Bench = serde_json::from_str(&serde_json::to_string(&bench).unwrap()).unwrap();
    assert_eq!(
        scene_vessel(&restored.vessels[0]),
        scene_vessel(&bench.vessels[0])
    );
}

#[test]
fn olive_oil_does_not_invent_calorimetry_spectrum_or_neutral_partition_models() {
    let mut bench = Bench::new();
    for line in [
        "add v1 water 50mL",
        "add v1 olive_oil 50mL",
        "add v1 ethanol 1mL",
        "new",
    ] {
        run(&mut bench, line);
    }
    assert_eq!(
        observable_support(&bench.vessels[0], "temperature").status,
        ObservableStatus::Incomplete
    );
    assert!(observable_support(&bench.vessels[0], "temperature")
        .reasons
        .iter()
        .any(|reason| reason == "missing-material-heat-capacity"));
    let before = serde_json::to_value(&bench.vessels[0]).unwrap();
    let events = run(&mut bench, "drain v1 v2");
    assert_eq!(before, serde_json::to_value(&bench.vessels[0]).unwrap());
    assert!(events.iter().any(|e| matches!(e, Event::NotYetModeled { reason: Some(reason), .. } if reason.key == "not-modeled.material-layer-partition")));
}
