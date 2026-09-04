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
