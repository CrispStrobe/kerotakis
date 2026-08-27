use kerotakis_core::ops::Event;
use kerotakis_core::scene::scene;
use kerotakis_core::script::parse_op;
use kerotakis_core::Bench;

fn step(bench: &mut Bench, command: &str) -> Vec<Event> {
    let op = parse_op(command)
        .unwrap_or_else(|error| panic!("parse {command}: {error}"))
        .unwrap_or_else(|| panic!("operator {command}"));
    bench
        .step(op)
        .unwrap_or_else(|error| panic!("step {command}: {error}"))
}

#[test]
fn pepper_then_dish_soap_computes_and_persists_surface_clearing() {
    let mut bench = Bench::new();
    step(&mut bench, "add v1 water 100mL");
    let pepper_events = step(&mut bench, "add v1 Pfeffer 0.08g");
    assert!(!pepper_events
        .iter()
        .any(|event| matches!(event, Event::SurfaceSpread { .. })));
    let before = bench.vessels[0]
        .surface_particles
        .as_ref()
        .expect("pepper surface state");
    assert_eq!(before.material, "ground black pepper");
    assert!((before.coverage_fraction - 1.0).abs() < 1e-12);
    assert_eq!(before.cleared_fraction, 0.0);

    let soap_events = step(&mut bench, "add v1 Spülmittel 1mL");
    let spread = soap_events.iter().find_map(|event| match event {
        Event::SurfaceSpread {
            from_cleared_fraction,
            to_cleared_fraction,
            coverage_fraction,
            ..
        } => Some((
            *from_cleared_fraction,
            *to_cleared_fraction,
            *coverage_fraction,
        )),
        _ => None,
    });
    let (from, to, coverage) = spread.expect("computed surface spread event");
    assert_eq!(from, 0.0);
    assert!((to - 0.9).abs() < 1e-12);
    assert!((coverage - 1.0).abs() < 1e-12);

    let rendered = scene(&bench);
    let particles = rendered.vessels[0]
        .surface_particles
        .as_ref()
        .expect("surface particles in scene contract");
    assert_eq!(particles.material, "ground black pepper");
    assert!((particles.cleared_fraction - to).abs() < 1e-12);
}

#[test]
fn soap_before_pepper_has_no_false_sudden_spread_event() {
    let mut bench = Bench::new();
    step(&mut bench, "add v1 water 100mL");
    step(&mut bench, "add v1 Spülmittel 1mL");
    let events = step(&mut bench, "add v1 Pfeffer 0.08g");
    assert!(!events
        .iter()
        .any(|event| matches!(event, Event::SurfaceSpread { .. })));
}
