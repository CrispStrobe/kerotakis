use kerotakis_core::ops::Event;
use kerotakis_core::script::parse_op;
use kerotakis_core::{scene, Bench};

fn run(bench: &mut Bench, command: &str) -> Vec<Event> {
    bench
        .step(parse_op(command).expect("valid command").expect("operator"))
        .expect("step succeeds")
}

#[test]
fn food_colours_stay_localized_until_soap_spreads_them() {
    let mut bench = Bench::new();
    run(&mut bench, "add v1 Vollmilch 100mL");
    run(&mut bench, "add v1 Lebensmittelfarbe_rot 1mL");
    run(&mut bench, "add v1 Lebensmittelfarbe_blau 1mL");

    let before = scene(&bench).vessels.remove(0);
    assert_eq!(before.surface_colours.len(), 2);
    assert!(before
        .surface_colours
        .iter()
        .all(|spot| spot.spread_fraction == 0.0));
    let liquid = before.liquid.expect("milk liquid");
    assert!(liquid.srgb[0] > 220 && liquid.srgb[1] > 220 && liquid.srgb[2] > 220);
    assert!(before.words.contains("drops are resting"));

    let events = run(&mut bench, "add v1 Spülmittel 1mL");
    assert!(events.iter().any(|event| matches!(
        event,
        Event::SurfaceColourSpread {
            from_spread_fraction,
            to_spread_fraction,
            spot_count: 2,
            ..
        } if *from_spread_fraction == 0.0 && (*to_spread_fraction - 0.9).abs() < 1e-12
    )));
    let after = scene(&bench).vessels.remove(0);
    assert!(after
        .surface_colours
        .iter()
        .all(|spot| (spot.spread_fraction - 0.9).abs() < 1e-12));
    assert!(after.words.contains("streaks have spread"));
}

#[test]
fn stirring_homogenizes_surface_dye_into_bulk_colour() {
    let mut bench = Bench::new();
    run(&mut bench, "add v1 whole_milk 100mL");
    run(&mut bench, "add v1 food_colour_blue 1mL");
    let localized = scene(&bench).vessels.remove(0).liquid.unwrap().srgb;

    let events = run(&mut bench, "stir v1 300rpm 2s");
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::SurfaceColourMixed { spot_count: 1, .. })));
    let mixed = scene(&bench).vessels.remove(0);
    assert!(mixed.surface_colours.is_empty());
    assert_ne!(mixed.liquid.unwrap().srgb, localized);
}

#[test]
fn food_colour_in_plain_water_uses_normal_homogeneous_optics() {
    let mut bench = Bench::new();
    run(&mut bench, "add v1 water 100mL");
    run(&mut bench, "add v1 food_colour_red 1mL");
    let rendered = scene(&bench).vessels.remove(0);
    assert!(rendered.surface_colours.is_empty());
    let rgb = rendered.liquid.unwrap().srgb;
    assert!(rgb[0] > rgb[1] && rgb[0] > rgb[2]);
}
