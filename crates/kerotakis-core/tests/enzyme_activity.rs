use kerotakis_core::enzyme::EnzymeFamily;
use kerotakis_core::script::parse_op;
use kerotakis_core::units::Kelvin;
use kerotakis_core::{Bench, Event};

fn run(lines: &[&str]) -> (Bench, Vec<Event>) {
    let mut bench = Bench::new();
    let mut events = Vec::new();
    for line in lines {
        let op = parse_op(line).expect("parse").expect("operator");
        events.extend(bench.step(op).expect("step"));
    }
    (bench, events)
}

fn hydrolysed(events: &[Event], family: EnzymeFamily) -> f64 {
    events
        .iter()
        .filter_map(|event| match event {
            Event::EnzymeHydrolysed {
                family: seen,
                hydrolysed_mass_g,
                ..
            } if *seen == family => Some(*hydrolysed_mass_g),
            _ => None,
        })
        .sum()
}

#[test]
fn each_family_acts_only_on_its_reviewed_food_substrate() {
    let cases = [
        ("whole_milk", "lactase", EnzymeFamily::Lactase),
        ("gelatin", "protease", EnzymeFamily::Protease),
        ("vegetable_oil", "lipase", EnzymeFamily::Lipase),
    ];
    for (material, enzyme, family) in cases {
        let (_, active) = run(&[
            "add v1 water 50mL",
            &format!("add v1 {material} 10g"),
            &format!("add v1 {enzyme} 0.1g"),
            "wait 1h",
        ]);
        assert!(hydrolysed(&active, family) > 0.0, "{material}: {active:?}");

        let (_, control) = run(&[
            "add v1 water 50mL",
            &format!("add v1 {material} 10g"),
            "wait 1h",
        ]);
        assert_eq!(hydrolysed(&control, family), 0.0, "{material}: {control:?}");
    }
}

#[test]
fn wrong_enzyme_is_a_silent_negative_control() {
    let (_, events) = run(&[
        "add v1 water 50mL",
        "add v1 whole_milk 10g",
        "add v1 lipase 0.1g",
        "wait 1h",
    ]);
    assert!(!events
        .iter()
        .any(|event| matches!(event, Event::EnzymeHydrolysed { .. })));
}

#[test]
fn progress_is_bounded_dose_ordered_and_mass_conserving() {
    let (low, low_events) = run(&[
        "add v1 water 50mL",
        "add v1 gelatin 10g",
        "add v1 protease 0.01g",
        "wait 1h",
    ]);
    let (high, high_events) = run(&[
        "add v1 water 50mL",
        "add v1 gelatin 10g",
        "add v1 protease 0.1g",
        "wait 1h",
    ]);
    let (low_before, _) = run(&[
        "add v1 water 50mL",
        "add v1 gelatin 10g",
        "add v1 protease 0.01g",
    ]);
    let (high_before, _) = run(&[
        "add v1 water 50mL",
        "add v1 gelatin 10g",
        "add v1 protease 0.1g",
    ]);
    assert!(
        hydrolysed(&high_events, EnzymeFamily::Protease)
            > hydrolysed(&low_events, EnzymeFamily::Protease)
    );
    assert!((low.vessels[0].mass().0 - low_before.vessels[0].mass().0).abs() < 1e-9);
    assert!((high.vessels[0].mass().0 - high_before.vessels[0].mass().0).abs() < 1e-9);
    let state = high.vessels[0]
        .unresolved_materials
        .iter()
        .find_map(|portion| portion.enzyme_hydrolysis.as_ref())
        .expect("progress state");
    assert!(state.converted_fraction > 0.0 && state.converted_fraction <= 1.0);
}

#[test]
fn split_wait_matches_one_interval() {
    let (_, one) = run(&[
        "add v1 water 50mL",
        "add v1 whole_milk 10g",
        "add v1 lactase 0.1g",
        "wait 1h",
    ]);
    let (_, two) = run(&[
        "add v1 water 50mL",
        "add v1 whole_milk 10g",
        "add v1 lactase 0.1g",
        "wait 30min",
        "wait 30min",
    ]);
    assert!(
        (hydrolysed(&one, EnzymeFamily::Lactase) - hydrolysed(&two, EnzymeFamily::Lactase)).abs()
            < 1e-9
    );
}

#[test]
fn activity_is_reduced_away_from_the_temperature_envelope() {
    let activity_at = |kelvin| {
        let (mut bench, _) = run(&[
            "add v1 water 50mL",
            "add v1 whole_milk 10g",
            "add v1 lactase 0.1g",
        ]);
        bench.vessels[0].temperature = Kelvin(kelvin);
        let events = bench
            .step(parse_op("wait 10min").unwrap().unwrap())
            .expect("wait");
        hydrolysed(&events, EnzymeFamily::Lactase)
    };
    let near_optimum = activity_at(310.15);
    assert!(near_optimum > activity_at(275.15));
    assert!(near_optimum > activity_at(345.15));
}

#[test]
fn decant_carries_progress_and_catalyst_proportionally() {
    let (mut bench, _) = run(&[
        "add v1 whole_milk 20g",
        "add v1 lactase 0.1g",
        "wait 10min",
        "new beaker",
        "decant v1 v2 0.5",
    ]);
    let states: Vec<_> = bench
        .vessels
        .iter()
        .map(|vessel| {
            vessel.unresolved_materials[0]
                .enzyme_hydrolysis
                .as_ref()
                .expect("transferred progress")
                .converted_fraction
        })
        .collect();
    assert!((states[0] - states[1]).abs() < 1e-12);
    let mass_before: f64 = bench.vessels.iter().map(|vessel| vessel.mass().0).sum();
    let events = bench
        .step(parse_op("wait 10min").unwrap().unwrap())
        .expect("wait");
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, Event::EnzymeHydrolysed { .. }))
            .count(),
        2
    );
    let mass_after: f64 = bench.vessels.iter().map(|vessel| vessel.mass().0).sum();
    assert!((mass_after - mass_before).abs() < 1e-9);
}
