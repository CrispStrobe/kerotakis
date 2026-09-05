use kerotakis_core::ops::{Event, Operator};
use kerotakis_core::script::parse_op;
use kerotakis_core::Bench;
use kerotakis_core::{Moles, Phase, SpeciesId};

fn run(bench: &mut Bench, command: &str) -> Vec<Event> {
    bench.step(parse_op(command).unwrap().unwrap()).unwrap()
}

#[test]
fn naked_egg_owns_its_inventory_and_osmosis_conserves_mass() {
    let mut bench = Bench::new();
    run(&mut bench, "add v1 water 100g");
    run(&mut bench, "add v1 naked_egg 50g");
    assert_eq!(bench.vessels[0].material_objects.len(), 1);
    let before = bench.vessels[0].mass().0;
    let events = bench.step(Operator::Wait { seconds: 3600.0 }).unwrap();
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::OsmosisChanged { mass_change_g, .. } if *mass_change_g > 0.0)));
    assert!((bench.vessels[0].mass().0 - before).abs() < 1e-8);
}

#[test]
fn cut_apple_browns_only_as_a_prepared_surface() {
    let mut bench = Bench::new();
    run(&mut bench, "add v1 cut_apple 20g");
    let events = bench.step(Operator::Wait { seconds: 900.0 }).unwrap();
    assert!(events.iter().any(
        |e| matches!(e, Event::BrowningChanged { browned_fraction, .. } if *browned_fraction > 0.0)
    ));
    assert!(bench.vessels[0].material_objects[0].state.browned_fraction > 0.0);
}

#[test]
fn liquid_pour_does_not_move_a_coherent_object() {
    let mut bench = Bench::new();
    run(&mut bench, "add v1 water 100g");
    run(&mut bench, "add v1 cut_apple 20g");
    run(&mut bench, "decant v1 v2 0.5");
    assert_eq!(bench.vessels[0].material_objects.len(), 1);
    assert!(bench
        .vessels
        .get(1)
        .is_none_or(|v| v.material_objects.is_empty()));
}

#[test]
fn old_snapshot_without_new_aggregate_state_still_decodes() {
    let bench = Bench::new();
    let mut value = serde_json::to_value(&bench).unwrap();
    for vessel in value["vessels"].as_array_mut().unwrap() {
        vessel.as_object_mut().unwrap().remove("soap_scum");
        vessel.as_object_mut().unwrap().remove("material_objects");
    }
    let restored: Bench = serde_json::from_value(value).unwrap();
    assert!(restored.vessels[0].soap_scum.is_none());
}

#[test]
fn declared_fatty_soap_forms_a_stoichiometric_conserved_aggregate() {
    let mut bench = Bench::new();
    bench.vessels[0].deposit(SpeciesId::new("Ca+2"), Moles(0.001), Phase::Aqueous);
    let before = bench.vessels[0].mass().0;
    let events = run(&mut bench, "add v1 fatty_soap 1g");
    assert!(events.iter().any(|e| matches!(e, Event::SoapScumFormed { divalent_ion_moles, .. } if (*divalent_ion_moles - 0.001).abs() < 1e-12)));
    let scum = bench.vessels[0].soap_scum.as_ref().unwrap();
    assert!((scum.soap_equivalent_moles - 0.002).abs() < 1e-12);
    // Calcium-stearate-equivalent and released sodium use rounded registry
    // molar masses, so conservation is pinned at the declared surrogate scale.
    let after = bench.vessels[0].mass().0;
    assert!(
        (after - (before + 1.0)).abs() < 2e-3,
        "before={before} after={after}"
    );
}
