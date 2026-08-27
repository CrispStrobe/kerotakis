use kerotakis_core::foam;
use kerotakis_core::material::MaterialBasis;
use kerotakis_core::ops::Event;
use kerotakis_core::script::parse_op;
use kerotakis_core::vessel::{UnresolvedMaterialPortion, Vessel, VesselId};
use kerotakis_core::Bench;

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
fn peroxide_yeast_and_soap_emit_chemistry_heat_and_visual_events() {
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
        .step(parse_op("wait 1s").unwrap().unwrap())
        .expect("kinetic interval");
    assert!(events.iter().any(|event| matches!(
        event,
        Event::GasProduced { species, moles, .. }
            if species.0 == "O2" && moles.0 > 0.0
    )));
    assert!(events.iter().any(
        |event| matches!(event, Event::ReactionHeatReleased { energy_j, .. } if *energy_j > 0.0)
    ));
    assert!(events.iter().any(
        |event| matches!(event, Event::FoamChanged { volume_liters, .. } if *volume_liters > 0.0)
    ));
    assert!(bench.vessels[0].temperature.0 > before.0);
}
