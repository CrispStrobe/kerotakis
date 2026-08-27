use kerotakis_core::ops::Event;
use kerotakis_core::scene;
use kerotakis_core::script::parse_op;
use kerotakis_core::Bench;

fn run(bench: &mut Bench, script: &str) -> Vec<Event> {
    let operator = parse_op(script)
        .expect("valid command")
        .expect("command produces an operator");
    bench.step(operator).expect("operator succeeds")
}

#[test]
fn household_vinegar_separates_milk_into_computed_curds_and_whey() {
    let mut bench = Bench::new();
    run(&mut bench, "add v1 Milch 100mL");
    let before = scene(&bench).vessels[0]
        .liquid
        .as_ref()
        .expect("milk liquid")
        .cloudiness;

    let events = run(&mut bench, "add v1 Essig 10mL");
    let event = events
        .iter()
        .find_map(|event| match event {
            Event::CurdlingChanged {
                from_formed_fraction,
                to_formed_fraction,
                curd_solids_mass_g,
                acid_moles,
                ..
            } => Some((
                *from_formed_fraction,
                *to_formed_fraction,
                *curd_solids_mass_g,
                acid_moles.0,
            )),
            _ => None,
        })
        .expect("curdling event");
    assert!(event.0.abs() < 1e-12);
    assert!((event.1 - 0.28).abs() < 1e-12);
    assert!((event.2 - 3.7492).abs() < 1e-4);
    assert!((event.3 - 0.008376).abs() < 2e-6);

    let picture = scene(&bench);
    let vessel = &picture.vessels[0];
    let curds = vessel.curds.as_ref().expect("drawable curds");
    assert!((curds.formed_fraction - 0.28).abs() < 1e-12);
    assert!((curds.separation_progress - 1.0).abs() < 1e-12);
    assert!((curds.solids_mass_g - 3.7492).abs() < 1e-4);
    assert_eq!(curds.srgb, [250, 248, 230]);
    assert!(vessel.words.contains("Soft curds"), "{}", vessel.words);
    assert!(vessel.words.contains("cloudy whey"), "{}", vessel.words);
    assert!(
        vessel.liquid.as_ref().expect("whey liquid").cloudiness < before,
        "dispersed colloid must decrease when solids join the curds"
    );
    assert!((vessel.mass_g - 113.06).abs() < 0.15);
}

#[test]
fn vinegar_without_milk_and_a_trace_dose_do_not_invent_curds() {
    let mut vinegar = Bench::new();
    let events = run(&mut vinegar, "add v1 Essig 10mL");
    assert!(!events
        .iter()
        .any(|event| matches!(event, Event::CurdlingChanged { .. })));
    assert!(scene(&vinegar).vessels[0].curds.is_none());

    let mut trace = Bench::new();
    run(&mut trace, "add v1 Milch 100mL");
    let events = run(&mut trace, "add v1 Essig 1mL");
    assert!(!events
        .iter()
        .any(|event| matches!(event, Event::CurdlingChanged { .. })));
    assert!(scene(&trace).vessels[0].curds.is_none());
}
