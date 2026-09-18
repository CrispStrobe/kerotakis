use kerotakis_core::script::parse_op;
use kerotakis_core::{appearance, protein, Bench, Kelvin};

fn bench_with(material: &str) -> Bench {
    let mut bench = Bench::new();
    let op = parse_op(&format!("add v1 {material} 20g"))
        .expect("parse")
        .expect("operator");
    bench.step(op).expect("add material");
    bench
}

#[test]
fn bench_heated_egg_white_stays_denatured_after_cooling() {
    let mut bench = bench_with("egg_white");
    for command in ["heat v1 5kJ", "cool v1 5kJ"] {
        bench.step(parse_op(command).unwrap().unwrap()).unwrap();
    }
    assert!(bench.vessels[0].temperature.to_celsius() < 62.0);
    let observation = protein::observe(&bench.vessels[0]);
    assert_eq!(observation[0].denatured_fraction, 1.0);
    assert!(observation[0].coagulated);
}

#[test]
fn heat_history_survives_snapshots_and_old_portions_default_to_raw() {
    let mut bench = bench_with("egg_white");
    for command in ["heat v1 5kJ", "cool v1 5kJ"] {
        bench.step(parse_op(command).unwrap().unwrap()).unwrap();
    }
    let saved = serde_json::to_string(&bench).unwrap();
    let restored: Bench = serde_json::from_str(&saved).unwrap();
    assert_eq!(
        protein::observe(&restored.vessels[0])[0].denatured_fraction,
        1.0
    );

    let portion = &bench.vessels[0].unresolved_materials[0];
    let mut old = serde_json::to_value(portion).unwrap();
    old.as_object_mut()
        .unwrap()
        .remove("protein_denatured_fraction");
    let restored: kerotakis_core::vessel::UnresolvedMaterialPortion =
        serde_json::from_value(old).unwrap();
    assert_eq!(restored.protein_denatured_fraction, 0.0);
}

#[test]
fn adding_raw_material_does_not_inherit_cooked_portions_history() {
    let mut bench = bench_with("egg_white");
    for command in ["heat v1 5kJ", "cool v1 5kJ", "add v1 egg_white 20g"] {
        bench.step(parse_op(command).unwrap().unwrap()).unwrap();
    }
    let observations = protein::observe(&bench.vessels[0]);
    assert_eq!(observations.len(), 2);
    assert_eq!(observations[0].denatured_fraction, 1.0);
    assert_eq!(observations[1].denatured_fraction, 0.0);
    assert!((observations[0].protein_mass_g - observations[1].protein_mass_g).abs() < 1e-10);
    let before = serde_json::to_string(&bench.vessels[0]).unwrap();
    let _ = protein::observe(&bench.vessels[0]);
    assert_eq!(serde_json::to_string(&bench.vessels[0]).unwrap(), before);
}

#[test]
fn the_four_named_materials_expose_real_protein_mass() {
    for material in ["egg_white", "gelatin", "cream", "albumin"] {
        let bench = bench_with(material);
        let observations = protein::observe(&bench.vessels[0]);
        assert_eq!(observations.len(), 1, "{material}: {observations:?}");
        assert!(observations[0].protein_mass_g > 0.0);
    }
}

#[test]
fn heating_egg_white_makes_denaturation_visible() {
    let mut bench = bench_with("egg_white");
    let raw = appearance::observe(&bench.vessels[0]);
    assert!(!raw.words.contains("denatured"), "{}", raw.words);

    bench.vessels[0].temperature = Kelvin::from_celsius(70.0);
    let cooked = appearance::observe(&bench.vessels[0]);
    assert!(cooked.cloudiness > 0.9, "{cooked:?}");
    assert!(cooked.words.contains("denatured"), "{}", cooked.words);
    assert!(
        cooked.words.contains("opaque white solid"),
        "{}",
        cooked.words
    );
}
