use kerotakis_core::script::parse_op;
use kerotakis_core::{observe, Bench, VesselId};

fn paint_mix(parts: &[(&str, f64)]) -> ([u8; 3], f64) {
    let mut bench = Bench::new();
    for (material, millilitres) in parts {
        let command = format!("add v1 {material} {millilitres}mL");
        bench
            .step(
                parse_op(&command)
                    .expect("paint command")
                    .expect("paint operator"),
            )
            .expect("add paint");
    }
    let appearance = observe(bench.vessel(VesselId(0)).expect("vessel"));
    let colour = appearance.liquid.expect("computed paint colour");
    ([colour.r, colour.g, colour.b], appearance.cloudiness)
}

fn brightness(rgb: [u8; 3]) -> u16 {
    rgb.into_iter().map(u16::from).sum()
}

#[test]
fn installed_acrylic_primaries_are_computed_and_opaque() {
    let (red, red_opacity) = paint_mix(&[("Acrylfarbe_rot", 1.0)]);
    let (yellow, _) = paint_mix(&[("Acrylfarbe_gelb", 1.0)]);
    let (blue, _) = paint_mix(&[("Acrylfarbe_blau", 1.0)]);
    assert!(red[0] > red[1] && red[0] > red[2], "red: {red:?}");
    assert!(
        yellow[0] > yellow[2] && yellow[1] > yellow[2],
        "yellow: {yellow:?}"
    );
    assert!(blue[2] > blue[0] && blue[2] > blue[1], "blue: {blue:?}");
    assert_eq!(red_opacity, 1.0);
}

#[test]
fn blue_and_yellow_paint_mix_subtractively_and_in_any_order() {
    let (yellow_then_blue, _) = paint_mix(&[("yellow_acrylic", 1.0), ("blue_acrylic", 1.0)]);
    let (blue_then_yellow, _) = paint_mix(&[("blue_acrylic", 1.0), ("yellow_acrylic", 1.0)]);
    assert_eq!(yellow_then_blue, blue_then_yellow);
    assert!(
        yellow_then_blue[1] > yellow_then_blue[0] && yellow_then_blue[1] > yellow_then_blue[2],
        "subtractive mixture should be green: {yellow_then_blue:?}"
    );
}

#[test]
fn white_paint_lightens_red_without_rgb_averaging() {
    let (red, _) = paint_mix(&[("red_acrylic", 1.0)]);
    let (tint, _) = paint_mix(&[("red_acrylic", 1.0), ("white_acrylic", 3.0)]);
    assert!(brightness(tint) > brightness(red), "{red:?} -> {tint:?}");
    assert!(tint[0] > tint[1] && tint[0] > tint[2], "{tint:?}");
}

#[test]
fn generic_acrylic_name_is_not_silently_guessed() {
    assert!(kerotakis_core::material::lookup("Acrylfarbe", Some("de")).is_none());
    assert!(kerotakis_core::material::lookup("acrylic paint", Some("en")).is_none());
}
