use kerotakis_data::{PhaseProperty, RegistryDocument};
#[test]
fn reexport_preserves_the_reviewed_potassium_nitrate_heat_record() {
    let source: RegistryDocument = serde_json::from_str(include_str!(
        "../../../data/registry/registry-source-v1.json"
    ))
    .unwrap();
    let exported = kerotakis_registry_export::export_current_registry().unwrap();
    let rows: Vec<_> = exported
        .phase_thermodynamics
        .iter()
        .filter(|r| r.species_id == "KNO3" && r.property == PhaseProperty::EnthalpyOfDissolution)
        .collect();
    assert_eq!(rows.len(), 1);
    let expected = source
        .phase_thermodynamics
        .iter()
        .find(|r| r.species_id == "KNO3" && r.property == PhaseProperty::EnthalpyOfDissolution)
        .unwrap();
    assert_eq!(*rows[0], *expected);
    assert!(exported
        .sources
        .iter()
        .any(|s| s.id == expected.quantity.source_id));
}
