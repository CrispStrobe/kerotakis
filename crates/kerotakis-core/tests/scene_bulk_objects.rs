use kerotakis_core::scene::scene_vessel;
use kerotakis_core::script::parse_op;
use kerotakis_core::Bench;

fn scene_with(material: &str) -> kerotakis_core::scene::SceneVessel {
    let mut bench = Bench::new();
    for line in [
        "add v1 water 250mL".to_string(),
        format!("add v1 {material} 20g"),
    ] {
        let op = parse_op(&line).expect("parse").expect("bench operation");
        bench.step(op).expect("run");
    }
    scene_vessel(&bench.vessels[0])
}

#[test]
fn whole_object_bulk_density_reaches_the_scene_contract() {
    let apple = scene_with("apple");
    assert_eq!(apple.bulk_objects.len(), 1);
    assert_eq!(apple.bulk_objects[0].position, "floating");
    assert!((apple.bulk_objects[0].bulk_density_g_per_ml - 0.85).abs() < 1e-12);
    assert!(apple
        .solids
        .iter()
        .filter(|solid| solid.species == "cellulose")
        .all(|solid| solid.represented_by_bulk_object));

    let pumice = scene_with("pumice");
    assert_eq!(pumice.bulk_objects[0].position, "floating");

    let potato = scene_with("potato");
    assert_eq!(potato.bulk_objects[0].position, "sunk");
}
