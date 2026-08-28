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
    let mut bench = Bench::new();
    step(&mut bench, "add v1 water 10mL");
    step(&mut bench, "add v1 table_sugar 30g");
    let dissolved_g = phase_moles(&bench, Phase::Aqueous) * 342.2965;
    let solid_g = phase_moles(&bench, Phase::Solid) * 342.2965;
    assert!((dissolved_g - 20.0).abs() < 1e-9);
    assert!((solid_g - 10.0).abs() < 1e-9);
    assert!((dissolved_g + solid_g - 30.0).abs() < 1e-9);
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
