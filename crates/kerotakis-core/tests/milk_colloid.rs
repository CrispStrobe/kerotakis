use kerotakis_core::ops::Operator;
use kerotakis_core::scene;
use kerotakis_core::script::parse_op;
use kerotakis_core::{Bench, VesselId};

fn run(bench: &mut Bench, script: &str) {
    let operator = parse_op(script)
        .expect("valid command")
        .expect("command produces an operator");
    bench.step(operator).expect("operator succeeds");
}

#[test]
fn localized_milk_recipe_conserves_water_and_unresolved_milk_solids() {
    let recipe = kerotakis_core::material::lookup("Vollmilch", Some("de"))
        .expect("localized whole-milk recipe");
    assert_eq!(recipe.canonical_key, "whole_milk");
    let expansion = recipe.expand(103.0, 0).expect("100 mL milk by mass");
    assert!((expansion.components[0].amount - 89.61).abs() < 1e-10);
    assert!((expansion.unresolved_amount - 12.761803).abs() < 1e-10);
    assert!(
        (expansion.components[0].amount + expansion.unresolved_amount - expansion.total_amount)
            .abs()
            < 1e-10
    );
}

#[test]
fn milk_is_opaque_dilutes_visibly_and_has_honest_mass_and_volume() {
    let mut full = Bench::new();
    run(&mut full, "add v1 whole_milk 100mL");
    let full_scene = scene(&full);
    let full_vessel = &full_scene.vessels[0];
    let full_liquid = full_vessel.liquid.as_ref().expect("milk is visible");
    assert_eq!(full_liquid.colour_word, "white");
    assert!(full_liquid.cloudiness > 0.99);
    assert!((full_vessel.mass_g - 103.0).abs() < 0.1);
    assert!((full_liquid.volume_l - 0.103).abs() < 0.002);
    assert!(full_vessel.words.contains("white"), "{}", full_vessel.words);
    assert!(full_vessel.words.contains("cannot see through"));

    let mut diluted = Bench::new();
    run(&mut diluted, "add v1 whole_milk 10mL");
    run(&mut diluted, "add v1 water 90mL");
    let diluted_cloudiness = scene(&diluted).vessels[0]
        .liquid
        .as_ref()
        .expect("diluted milk is visible")
        .cloudiness;
    assert!(diluted_cloudiness > 0.15);
    assert!(diluted_cloudiness < full_liquid.cloudiness);
}

#[test]
fn a_half_pour_conserves_milk_and_keeps_its_concentration() {
    let mut bench = Bench::new();
    run(&mut bench, "add v1 Milch 100mL");
    bench
        .step(Operator::NewVessel { kind: None })
        .expect("second vessel");
    bench
        .step(Operator::Decant {
            from: VesselId(0),
            to: VesselId(1),
            fraction: 0.5,
        })
        .expect("pour half the milk");

    let picture = scene(&bench);
    let cloudiness = picture.vessels[0]
        .liquid
        .as_ref()
        .expect("milk remains")
        .cloudiness;
    for vessel in &picture.vessels {
        let liquid = vessel.liquid.as_ref().expect("milk in both vessels");
        assert!((liquid.volume_l - 0.0515).abs() < 0.001);
        assert!((liquid.cloudiness - cloudiness).abs() < 1e-10);
    }
    let conserved_unresolved: f64 = bench
        .vessels
        .iter()
        .flat_map(|vessel| &vessel.unresolved_materials)
        .filter(|portion| portion.recipe_id == "household/whole-milk-surrogate")
        .map(|portion| portion.amount)
        .sum();
    assert!((conserved_unresolved - 12.761803).abs() < 1e-10);
}
