use kerotakis_core::ops::Event;
use kerotakis_core::script::parse_op;
use kerotakis_core::{Bench, Moles, SpeciesId};

fn run(bench: &mut Bench, command: &str) -> Vec<Event> {
    bench
        .step(parse_op(command).expect("valid command").expect("operator"))
        .expect("step succeeds")
}

fn prepared(yeast: Option<&str>) -> Bench {
    let mut bench = Bench::new();
    run(&mut bench, "add v1 water 100mL");
    run(&mut bench, "add v1 table_sugar 10g");
    if let Some(yeast) = yeast {
        run(&mut bench, &format!("add v1 {yeast}"));
    }
    bench
}

#[test]
fn hydrated_yeast_converts_finite_sucrose_with_balanced_stoichiometry() {
    let mut bench = prepared(Some("dry_yeast 1g"));
    let sucrose_before = bench.vessels[0].moles_of(&SpeciesId::new("sucrose")).0;
    let events = run(&mut bench, "wait 600s");
    let step = events.iter().find_map(|event| match event {
        Event::Fermented {
            sucrose_moles,
            ethanol_moles,
            carbon_dioxide_moles,
            active_yeast_grams,
            ..
        } => Some((
            sucrose_moles.0,
            ethanol_moles.0,
            carbon_dioxide_moles.0,
            *active_yeast_grams,
        )),
        _ => None,
    });
    let (sugar, ethanol, carbon_dioxide, active_yeast) = step.expect("fermentation event");
    assert!(sugar > 1e-4);
    assert!((ethanol - 4.0 * sugar).abs() < 1e-12);
    assert!((carbon_dioxide - 4.0 * sugar).abs() < 1e-12);
    assert!(active_yeast > 0.5 && active_yeast <= 1.0);
    assert!(
        (bench.vessels[0].moles_of(&SpeciesId::new("sucrose")).0 - (sucrose_before - sugar)).abs()
            < 1e-12
    );
    assert!((bench.vessels[0].moles_of(&SpeciesId::new("ethanol")).0 - ethanol).abs() < 1e-12);
    assert!(events.iter().any(|event| matches!(
        event,
        Event::GasProduced { reaction, species, moles, .. }
            if reaction == "yeast-sucrose-fermentation"
                && species.0 == "CO2"
                && (moles.0 - carbon_dioxide).abs() < 1e-12
    )));
}

#[test]
fn sugar_water_without_yeast_does_not_ferment() {
    let mut bench = prepared(None);
    let before = bench.vessels[0].moles_of(&SpeciesId::new("sucrose"));
    let events = run(&mut bench, "wait 600s");
    assert!(!events
        .iter()
        .any(|event| matches!(event, Event::Fermented { .. })));
    assert_eq!(
        bench.vessels[0].moles_of(&SpeciesId::new("sucrose")),
        before
    );
    assert_eq!(
        bench.vessels[0].moles_of(&SpeciesId::new("ethanol")),
        Moles(0.0)
    );
}

#[test]
fn already_hydrated_fresh_yeast_starts_faster_than_dry_yeast() {
    let produced = |yeast: &str| {
        let mut bench = prepared(Some(yeast));
        run(&mut bench, "wait 2s")
            .into_iter()
            .find_map(|event| match event {
                Event::Fermented { sucrose_moles, .. } => Some(sucrose_moles.0),
                _ => None,
            })
            .unwrap_or(0.0)
    };
    let dry = produced("dry_yeast 1g");
    let fresh = produced("fresh_yeast 3.333g");
    assert!(fresh > dry * 2.0, "dry={dry}, fresh={fresh}");
}
