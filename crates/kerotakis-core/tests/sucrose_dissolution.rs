use kerotakis_core::ops::Event;
use kerotakis_core::script::parse_op;
use kerotakis_core::{Bench, Phase, SpeciesId};

fn step(bench: &mut Bench, command: &str) -> Vec<Event> {
    let operator = parse_op(command)
        .unwrap_or_else(|error| panic!("parse {command}: {error}"))
        .unwrap_or_else(|| panic!("operator {command}"));
    bench
        .step(operator)
        .unwrap_or_else(|error| panic!("step {command}: {error}"))
}

fn phase_moles(bench: &Bench, phase: Phase) -> f64 {
    bench.vessels[0]
        .contents
        .iter()
        .filter(|portion| portion.species == SpeciesId::new("sucrose") && portion.phase == phase)
        .map(|portion| portion.moles.0)
        .sum()
}

#[test]
fn household_sugar_dissolves_and_conserves_sucrose() {
    let mut bench = Bench::new();
    step(&mut bench, "add v1 water 100mL");
    let events = step(&mut bench, "add v1 Haushaltszucker 50g");
    let dissolved = events.iter().find_map(|event| match event {
        Event::Dissolved { species, moles, .. } if species.0 == "sucrose" => Some(moles.0),
        _ => None,
    });
    let expected = 50.0 / 342.2965;
    assert!((dissolved.expect("computed dissolution") - expected).abs() < 1e-12);
    assert!((phase_moles(&bench, Phase::Aqueous) - expected).abs() < 1e-12);
    assert!(phase_moles(&bench, Phase::Solid) < 1e-12);
}

#[test]
fn room_temperature_capacity_leaves_excess_sugar_as_crystals() {
    // KID-7: the limit is now read at the vessel's own temperature, so this
    // says which temperature it means. The reviewed 20 °C figure is 200 g
    // per 100 mL; a vessel that is not at 20 °C gets a different number, and
    // that is the point rather than a regression.
    let mut bench = Bench::new();
    step(&mut bench, "add v1 water 10mL @ 20C");
    step(&mut bench, "add v1 table_sugar 30g @ 20C");
    let dissolved_g = phase_moles(&bench, Phase::Aqueous) * 342.2965;
    let solid_g = phase_moles(&bench, Phase::Solid) * 342.2965;
    assert!(
        (dissolved_g - 20.0).abs() < 1e-9,
        "dissolved {dissolved_g} g"
    );
    assert!((solid_g - 10.0).abs() < 1e-9, "solid {solid_g} g");
    assert!((dissolved_g + solid_g - 30.0).abs() < 1e-9);
}

/// KID-7: hot water holds more, and the difference is the experiment.
#[test]
fn the_same_sugar_and_the_same_water_go_further_when_hot() {
    let capacity_at = |celsius: &str| -> f64 {
        let mut bench = Bench::new();
        step(&mut bench, &format!("add v1 water 10mL @ {celsius}C"));
        step(&mut bench, &format!("add v1 table_sugar 60g @ {celsius}C"));
        phase_moles(&bench, Phase::Aqueous) * 342.2965
    };
    let cold = capacity_at("20");
    let hot = capacity_at("100");
    assert!(
        (cold - 20.0).abs() < 1e-9,
        "20 °C holds the reviewed 20 g: {cold}"
    );
    assert!(
        (hot - 48.7).abs() < 1e-9,
        "100 °C holds the reviewed 48.7 g: {hot}"
    );
    // Nothing is lost either way: what does not dissolve is still there.
    for celsius in ["20", "100"] {
        let mut bench = Bench::new();
        step(&mut bench, &format!("add v1 water 10mL @ {celsius}C"));
        step(&mut bench, &format!("add v1 table_sugar 60g @ {celsius}C"));
        let total =
            (phase_moles(&bench, Phase::Aqueous) + phase_moles(&bench, Phase::Solid)) * 342.2965;
        assert!((total - 60.0).abs() < 1e-9, "at {celsius} °C: {total} g");
    }
}

/// KID-7: cooled past its limit with nothing to grow on, the solution stays
/// put and says so. That is the state a rock-candy jar sits in.
#[test]
fn a_cooled_syrup_reports_itself_supersaturated_rather_than_precipitating() {
    let mut bench = Bench::new();
    step(&mut bench, "add v1 water 10mL @ 100C");
    step(&mut bench, "add v1 table_sugar 40g @ 100C");
    assert!(
        phase_moles(&bench, Phase::Solid) < 1e-12,
        "all of it dissolves hot"
    );
    let events = step(&mut bench, "cool v1 3kJ");
    let reported = events.iter().any(
        |event| matches!(event, Event::Supersaturated { species, .. } if species.0 == "sucrose"),
    );
    assert!(reported, "cooling must report the state: {events:?}");
    assert!(
        phase_moles(&bench, Phase::Solid) < 1e-12,
        "and must not precipitate it without a seed"
    );

    // A crystal of the same sugar is all it takes.
    let events = step(&mut bench, "add v1 table_sugar 1g");
    assert!(
        events.iter().any(|event| matches!(
            event,
            Event::Precipitated { species, .. } if species.0 == "sucrose"
        )),
        "a seed must bring it down: {events:?}"
    );
    assert!(phase_moles(&bench, Phase::Solid) > 1e-3);
}

#[test]
fn sugar_without_water_does_not_claim_dissolution() {
    let mut bench = Bench::new();
    let events = step(&mut bench, "add v1 Kristallzucker 5g");
    assert!(!events
        .iter()
        .any(|event| matches!(event, Event::Dissolved { species, .. } if species.0 == "sucrose")));
    assert!(phase_moles(&bench, Phase::Solid) > 0.0);
}
