use kerotakis_core::species::SpeciesId;
use kerotakis_core::units::Moles;
use kerotakis_core::vessel::{MaterialObject, MaterialObjectState, ObjectComponent, Vessel};
use kerotakis_core::VesselId;

#[test]
fn old_vessel_snapshots_decode_without_object_state() {
    let vessel = Vessel::new(VesselId(0), "beaker");
    let mut value = serde_json::to_value(&vessel).unwrap();
    value.as_object_mut().unwrap().remove("material_objects");
    let restored: Vessel = serde_json::from_value(value).unwrap();
    assert!(restored.material_objects.is_empty());
}

#[test]
fn object_inventory_round_trips_and_counts_mass_once() {
    let mut vessel = Vessel::new(VesselId(0), "beaker");
    vessel.material_objects.push(MaterialObject {
        material: "prepared object".into(),
        recipe_id: "test/object".into(),
        recipe_version: 1,
        mass_g: 50.0,
        components: vec![ObjectComponent {
            species: SpeciesId::new("water"),
            moles: Moles(2.0),
        }],
        state: MaterialObjectState::default(),
    });
    assert!((vessel.mass().0 - 50.0).abs() < 1e-12);
    let restored: Vessel = serde_json::from_str(&serde_json::to_string(&vessel).unwrap()).unwrap();
    assert_eq!(restored.material_objects, vessel.material_objects);
    assert!((restored.mass().0 - 50.0).abs() < 1e-12);
}
