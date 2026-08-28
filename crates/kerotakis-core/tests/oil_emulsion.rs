use kerotakis_core::ops::Event;
use kerotakis_core::script::parse_op;
use kerotakis_core::{observe, scene, Bench, VesselId};

fn run(bench: &mut Bench, command: &str) -> Vec<Event> {
    bench
        .step(parse_op(command).expect("valid command").expect("operator"))
        .expect("step succeeds")
}

fn prepared(with_soap: bool) -> Bench {
    let mut bench = Bench::new();
    run(&mut bench, "add v1 water 100mL");
    run(&mut bench, "add v1 vegetable_oil 50mL");
    if with_soap {
        run(&mut bench, "add v1 dish_soap 1mL");
    }
    bench
}

#[test]
fn detergent_and_stirring_make_a_cloudy_finite_emulsion() {
    let mut bench = prepared(true);
    let before = scene(&bench).vessels.remove(0);
    assert!(before.emulsion.is_none());
    assert!((before.layers.last().unwrap().volume_l - 0.050).abs() < 1e-9);

    let events = run(&mut bench, "stir v1 500rpm 10s");
    let changed = events.iter().find_map(|event| match event {
        Event::EmulsionChanged {
            from_dispersed_fraction,
            to_dispersed_fraction,
            ..
        } => Some((*from_dispersed_fraction, *to_dispersed_fraction)),
        _ => None,
    });
    let (from, to) = changed.expect("stir creates a computed emulsion event");
    assert_eq!(from, 0.0);
    assert!((to - 0.92).abs() < 1e-8);

    let after = scene(&bench).vessels.remove(0);
    let emulsion = after.emulsion.expect("persistent emulsion scene state");
    assert!((emulsion.dispersed_volume_l - 0.046).abs() < 1e-8);
    assert!(after.liquid.unwrap().cloudiness > 0.70);
    assert!((after.layers.last().unwrap().volume_l - 0.004).abs() < 1e-8);
    assert!(after.words.contains("cloudy droplets"));
    assert!(observe(bench.vessel(VesselId(0)).unwrap())
        .words
        .contains("cloudy"));
}

#[test]
fn resting_coalesces_droplets_and_restores_the_oil_layer() {
    let mut bench = prepared(true);
    run(&mut bench, "stir v1 500rpm 10s");
    let events = run(&mut bench, "wait 300s");
    assert!(events.iter().any(|event| matches!(
        event,
        Event::EmulsionChanged {
            from_dispersed_fraction,
            to_dispersed_fraction,
            ..
        } if (*from_dispersed_fraction - 0.92).abs() < 1e-8
            && (*to_dispersed_fraction - 0.46).abs() < 1e-8
    )));

    let after = scene(&bench).vessels.remove(0);
    assert!((after.emulsion.unwrap().dispersed_fraction - 0.46).abs() < 1e-8);
    assert!((after.layers.last().unwrap().volume_l - 0.027).abs() < 1e-8);
}

#[test]
fn stirring_without_detergent_does_not_invent_a_stable_emulsion() {
    let mut bench = prepared(false);
    let events = run(&mut bench, "stir v1 500rpm 10s");
    assert!(!events
        .iter()
        .any(|event| matches!(event, Event::EmulsionChanged { .. })));
    let after = scene(&bench).vessels.remove(0);
    assert!(after.emulsion.is_none());
    assert!((after.layers.last().unwrap().volume_l - 0.050).abs() < 1e-9);
}
