use kerotakis_core::foam;
use kerotakis_core::material::MaterialBasis;
use kerotakis_core::ops::Event;
use kerotakis_core::script::parse_op;
use kerotakis_core::vessel::{UnresolvedMaterialPortion, Vessel, VesselId};
use kerotakis_core::Bench;

fn household_foam_after(catalyst: &str, seconds: f64) -> (f64, f64, f64) {
    let mut bench = Bench::new();
    for command in [
        "add v1 Wasserstoffperoxid_3% 100mL".to_string(),
        "add v1 Spülmittel 10mL".to_string(),
        format!("add v1 {catalyst}"),
    ] {
        bench
            .step(parse_op(&command).unwrap().unwrap())
            .unwrap_or_else(|error| panic!("{command}: {error}"));
    }
    let command = format!("stir v1 500rpm {seconds}s");
    let events = bench
        .step(parse_op(&command).unwrap().unwrap())
        .unwrap_or_else(|error| panic!("{command}: {error}"));
    let oxygen = events
        .iter()
        .filter_map(|event| match event {
            Event::GasProduced { species, moles, .. } if species.0 == "O2" => Some(moles.0),
            _ => None,
        })
        .sum();
    let (foam, overflow) = events
        .iter()
        .rev()
        .find_map(|event| match event {
            Event::FoamChanged {
                volume_liters,
                overflow_liters,
                ..
            } => Some((*volume_liters, *overflow_liters)),
            _ => None,
        })
        .unwrap_or_default();
    (oxygen, foam, overflow)
}

#[test]
fn no_soap_control_cannot_make_persistent_foam() {
    let mut vessel = Vessel::new(VesselId(0), "beaker");
    assert!(foam::advance(&mut vessel, 1.0, 0.01).is_none());
    assert_eq!(vessel.foam.volume_liters, 0.0);
}

#[test]
fn declared_dish_soap_role_maps_gas_to_bounded_foam_and_decay() {
    let mut vessel = Vessel::new(VesselId(0), "beaker");
    vessel.unresolved_materials.push(UnresolvedMaterialPortion {
        material: "dish_soap".to_string(),
        recipe_id: "household/dish-soap-surrogate".to_string(),
        recipe_version: 1,
        basis: MaterialBasis::MassFraction,
        amount: 0.4,
    });
    let first = foam::advance(&mut vessel, 0.0, 0.01).expect("foam target");
    assert!(first.volume_liters > 0.2);
    let decayed = foam::advance(&mut vessel, 180.0, 0.0).expect("decayed foam");
    assert!((decayed.volume_liters / first.volume_liters - 0.5).abs() < 1e-12);
}

#[test]
fn peroxide_yeast_and_soap_react_during_stirring_and_overflow() {
    let mut bench = Bench::new();
    for command in [
        "add v1 Wasserstoffperoxid_3% 100mL",
        "add v1 Spülmittel 10mL",
        "add v1 Hefe 1g",
    ] {
        let op = parse_op(command)
            .unwrap_or_else(|error| panic!("{command}: {error}"))
            .expect("operator");
        bench.step(op).expect("material addition");
    }
    let before = bench.vessels[0].temperature;
    let events = bench
        .step(parse_op("stir v1 500rpm 1s").unwrap().unwrap())
        .expect("timed stirring");
    assert!(events.iter().any(|event| matches!(
        event,
        Event::GasProduced { species, moles, .. }
            if species.0 == "O2" && moles.0 > 0.0
    )));
    assert!(events.iter().any(
        |event| matches!(event, Event::ReactionHeatReleased { energy_j, .. } if *energy_j > 0.0)
    ));
    assert!(events.iter().any(|event| matches!(
        event,
        Event::FoamChanged {
            volume_liters,
            overflow_liters,
            ..
        } if *volume_liters > 0.0 && *overflow_liters > 0.0
    )));
    assert!(bench.vessels[0].temperature.0 > before.0);
}

#[test]
fn potassium_iodide_is_a_retained_catalyst_that_makes_visible_foam() {
    let mut bench = Bench::new();
    for command in [
        "add v1 Wasserstoffperoxid_3% 100mL",
        "add v1 Spülmittel 10mL",
        "add v1 KI 1g",
    ] {
        let events = bench
            .step(parse_op(command).unwrap().unwrap())
            .unwrap_or_else(|error| panic!("{command}: {error}"));
        assert!(!events.iter().any(|event| matches!(
            event,
            Event::NotYetModeled { what, .. } if what.contains("potassium iodide")
        )));
    }
    let iodide_before = bench.vessels[0]
        .moles_of(&kerotakis_core::SpeciesId::new("KI"))
        .0;
    assert!(bench.vessels[0].contents.iter().any(|portion| {
        portion.species.0 == "KI" && portion.phase == kerotakis_core::Phase::Aqueous
    }));
    assert!(!bench.vessels[0].contents.iter().any(|portion| {
        portion.species.0 == "KI" && portion.phase == kerotakis_core::Phase::Solid
    }));
    let events = bench
        .step(parse_op("stir v1 500rpm 10s").unwrap().unwrap())
        .expect("iodide-catalysed stirring interval");

    assert!(events.iter().any(|event| matches!(
        event,
        Event::Reacted {
            reaction,
            catalyst: Some(catalyst),
            ..
        } if reaction == "peroxide-decomposition" && catalyst == "potassium iodide"
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        Event::FoamChanged {
            height_cm,
            overflow_liters,
            ..
        } if *height_cm > 0.0 && *overflow_liters > 0.0
    )));
    let iodide_after = bench.vessels[0]
        .moles_of(&kerotakis_core::SpeciesId::new("KI"))
        .0;
    assert!((iodide_after - iodide_before).abs() < 1e-12);
}

#[test]
fn potassium_iodide_dissolves_even_when_it_is_added_before_the_liquid() {
    let mut bench = Bench::new();
    bench
        .step(parse_op("add v1 KI 1g").unwrap().unwrap())
        .unwrap();
    let before = bench.vessels[0]
        .moles_of(&kerotakis_core::SpeciesId::new("KI"))
        .0;
    let events = bench
        .step(parse_op("add v1 water 100mL").unwrap().unwrap())
        .unwrap();
    assert!(events.iter().any(|event| matches!(
        event,
        Event::Dissolved { species, moles, .. }
            if species.0 == "KI" && (moles.0 - before).abs() < 1e-12
    )));
    assert!(bench.vessels[0].contents.iter().any(
        |portion| portion.species.0 == "KI" && portion.phase == kerotakis_core::Phase::Aqueous
    ));
    assert!(!events.iter().any(|event| matches!(
        event,
        Event::NotYetModeled { what, .. } if what.contains("potassium iodide")
    )));
    assert!(
        (bench.vessels[0]
            .moles_of(&kerotakis_core::SpeciesId::new("KI"))
            .0
            - before)
            .abs()
            < 1e-12
    );
}

#[test]
fn more_potassium_iodide_makes_more_oxygen_and_foam_on_the_same_clock() {
    let low = household_foam_after("KI 0.25g", 10.0);
    let high = household_foam_after("KI 1g", 10.0);
    assert!(
        high.0 > low.0 * 2.0,
        "oxygen must respond to KI dose: low={low:?}, high={high:?}"
    );
    assert!(
        high.1 > low.1 * 2.0,
        "foam must inherit the chemistry ordering: low={low:?}, high={high:?}"
    );
    assert!(
        high.2 > low.2,
        "overflow must be dose-responsive: low={low:?}, high={high:?}"
    );
}

#[test]
fn more_yeast_makes_more_oxygen_and_foam_on_the_same_clock() {
    // The catalase path is deliberately spectacular at household doses. A
    // millisecond initial interval preserves the causal dose comparison before
    // both vessels run into the same finite-peroxide ceiling, while keeping
    // both GasProduced events above the observable-event floor.
    let low = household_foam_after("Hefe 0.25g", 0.001);
    let high = household_foam_after("Hefe 1g", 0.001);
    assert!(
        high.0 > low.0 * 2.0,
        "oxygen must respond to yeast loading: low={low:?}, high={high:?}"
    );
    assert!(
        high.1 > low.1 * 2.0,
        "foam must inherit the chemistry ordering: low={low:?}, high={high:?}"
    );
    assert!(
        high.2 > low.2,
        "overflow must be dose-responsive: low={low:?}, high={high:?}"
    );
}

#[test]
fn fresh_yeast_is_immediately_available_while_dry_yeast_hydrates() {
    // 3.333 g fresh yeast carries the same modeled dry solids and catalase as
    // 1 g dry yeast. Only the fresh form arrives already hydrated.
    let dry = household_foam_after("Trockenhefe 1g", 0.0001);
    let fresh = household_foam_after("Frischhefe 3.333333g", 0.0001);
    assert!(
        fresh.0 > dry.0 * 10.0,
        "initial oxygen must distinguish hydrated fresh yeast: dry={dry:?}, fresh={fresh:?}"
    );
    assert!(fresh.1 > dry.1 * 10.0);
}
