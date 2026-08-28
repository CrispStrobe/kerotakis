use kerotakis_core::ops::{Event, Operator};
use kerotakis_core::scene;
use kerotakis_core::script::parse_op;
use kerotakis_core::{Bench, VesselId};

fn run(bench: &mut Bench, script: &str) -> Vec<Event> {
    let operator = parse_op(script)
        .expect("valid command")
        .expect("command produces an operator");
    bench.step(operator).expect("operator succeeds")
}

#[test]
fn oil_forms_a_persistent_layer_above_coloured_water() {
    let mut bench = Bench::new();
    run(&mut bench, "add v1 water 100mL");
    run(&mut bench, "add v1 food_colour_blue 1mL");
    let events = run(&mut bench, "add v1 Pflanzenöl 50mL");

    assert!(events.iter().any(|event| matches!(
        event,
        Event::MaterialLayersFormed { upper_material, lower, .. }
            if upper_material == "vegetable oil" && lower.0 == "water"
    )));

    let vessel = &scene(&bench).vessels[0];
    let liquid = vessel
        .liquid
        .as_ref()
        .expect("oil and water are visible liquid");
    // The dye itself is dissolved solute, while its water carrier uses the
    // registry density conversion; the visible total remains about 151 mL.
    assert!((liquid.volume_l - 0.151).abs() < 1e-5);
    assert_eq!(vessel.layers.len(), 2);
    assert_eq!(vessel.layers[0].species, "solution");
    assert_eq!(vessel.layers[1].species, "vegetable_oil");
    assert!((vessel.layers[0].volume_l - 0.101).abs() < 1e-5);
    assert!((vessel.layers[1].volume_l - 0.050).abs() < 1e-9);
    assert_ne!(vessel.layers[0].srgb, vessel.layers[1].srgb);
    assert!(vessel.words.contains("separate pale yellow layer"));
    assert!((vessel.mass_g - 146.7).abs() < 0.2);
}

#[test]
fn oil_alone_is_visible_and_a_half_pour_conserves_it() {
    let mut bench = Bench::new();
    run(&mut bench, "add v1 vegetable_oil 50mL");
    bench
        .step(Operator::NewVessel { kind: None })
        .expect("second vessel");
    bench
        .step(Operator::Decant {
            from: VesselId(0),
            to: VesselId(1),
            fraction: 0.5,
        })
        .expect("pour oil");

    let picture = scene(&bench);
    for vessel in &picture.vessels {
        assert!((vessel.liquid.as_ref().expect("visible oil").volume_l - 0.025).abs() < 1e-9);
        assert_eq!(vessel.layers[0].species, "vegetable_oil");
    }
    let conserved: f64 = bench
        .vessels
        .iter()
        .flat_map(|vessel| &vessel.unresolved_materials)
        .filter(|portion| portion.recipe_id == "household/vegetable-oil-surrogate")
        .map(|portion| portion.amount)
        .sum();
    assert!((conserved - 46.0).abs() < 1e-9);
}
