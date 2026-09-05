//! BRD-014.S05: sunscreen answers ultraviolet light by attenuating it.

use kerotakis_core::script::parse_op;
use kerotakis_core::*;

fn step(bench: &mut Bench, line: &str) -> Vec<Event> {
    bench
        .step(parse_op(line).expect("grammar").expect("an operator"))
        .expect("step")
}

fn transmitted(events: &[Event]) -> Option<(f64, String)> {
    events.iter().find_map(|e| match e {
        Event::UvAttenuated {
            transmitted_fraction,
            band,
            ..
        } => Some((*transmitted_fraction, band.clone())),
        _ => None,
    })
}

#[test]
fn sunscreen_lets_a_thirtieth_of_uv_b_and_a_tenth_of_uv_a_through() {
    let mut bench = Bench::new();
    step(&mut bench, "add v1 sunscreen 1g");
    let uvb = step(&mut bench, "irradiate v1 300nm 10W/m2");
    let (fraction, band) = transmitted(&uvb).expect("a UV-B reading");
    assert_eq!(band, "UV-B");
    assert!((fraction - 1.0 / 30.0).abs() < 1e-12, "{fraction}");
    let uva = step(&mut bench, "irradiate v1 350nm 10W/m2");
    let (fraction, band) = transmitted(&uva).expect("a UV-A reading");
    assert_eq!(band, "UV-A");
    assert!((fraction - 0.1).abs() < 1e-12, "{fraction}");
    // Visible light is not this model's to speak about.
    let visible = step(&mut bench, "irradiate v1 500nm 10W/m2");
    assert!(transmitted(&visible).is_none(), "{visible:?}");
}

#[test]
fn a_material_without_the_role_says_nothing_new_under_uv() {
    let mut bench = Bench::new();
    step(&mut bench, "add v1 water 100mL");
    let events = step(&mut bench, "irradiate v1 300nm 10W/m2");
    assert!(events.iter().any(|e| matches!(e, Event::Irradiated { .. })));
    assert!(transmitted(&events).is_none());
}

#[test]
fn the_reading_carries_its_mechanism_and_the_recipe_its_boundary() {
    let recipe = material::lookup("sunscreen", None).expect("the recipe resolves");
    let role = recipe
        .roles
        .iter()
        .find_map(|r| match r {
            material::MaterialRole::UvAttenuation {
                spf,
                boundary,
                source,
                ..
            } => Some((*spf, boundary.clone(), source.clone())),
            _ => None,
        })
        .expect("the UV role");
    assert_eq!(role.0, 30.0);
    assert!(role.1.contains("not a measured spectrum"));
    assert!(role.2.contains("PENDING REVIEW"));
}
