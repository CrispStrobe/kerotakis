use kerotakis_core::script::parse_op;
use kerotakis_core::{observe, Bench, VesselId};

fn coloured_water(colours: &[(&str, f64)]) -> [u8; 3] {
    let mut bench = Bench::new();
    bench
        .step(
            parse_op("add v1 water 100mL")
                .expect("water command")
                .expect("water operator"),
        )
        .expect("add water");
    for (material, millilitres) in colours {
        let command = format!("add v1 {material} {millilitres}mL");
        bench
            .step(
                parse_op(&command)
                    .expect("colour command")
                    .expect("colour operator"),
            )
            .expect("add food colour");
    }
    let colour = observe(bench.vessel(VesselId(0)).expect("vessel"))
        .liquid
        .expect("liquid colour");
    [colour.r, colour.g, colour.b]
}

fn brightness(rgb: [u8; 3]) -> u16 {
    rgb.into_iter().map(u16::from).sum()
}

#[test]
fn food_colour_intensity_tracks_amount_through_beer_lambert() {
    let dilute = coloured_water(&[("blue_food_color", 0.1)]);
    let strong = coloured_water(&[("blue_food_color", 1.0)]);
    assert!(
        brightness(strong) < brightness(dilute),
        "more dye must transmit less light: dilute={dilute:?}, strong={strong:?}"
    );
    assert!(
        strong[2] > strong[0],
        "blue surrogate should remain blue: {strong:?}"
    );
}

#[test]
fn transparent_dye_mixing_is_order_independent() {
    let red_then_blue = coloured_water(&[("red_food_color", 0.5), ("blue_food_color", 0.5)]);
    let blue_then_red = coloured_water(&[("blue_food_color", 0.5), ("red_food_color", 0.5)]);
    assert_eq!(red_then_blue, blue_then_red);
}

#[test]
fn generic_food_colour_is_not_silently_guessed() {
    assert!(kerotakis_core::material::lookup("Lebensmittelfarbe", Some("de")).is_none());
    for (name, expected) in [
        ("Lebensmittelfarbe_rot", "food_colour_red"),
        ("Lebensmittelfarbe_gelb", "food_colour_yellow"),
        ("Lebensmittelfarbe_blau", "food_colour_blue"),
    ] {
        assert_eq!(
            kerotakis_core::material::lookup(name, Some("de"))
                .expect(name)
                .canonical_key,
            expected
        );
    }
}
