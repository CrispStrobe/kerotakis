//! Supplemental typed-validation and correctly hashed pack controls.
use kerotakis_data::{build_pack, load_pack, PhaseProperty, RegistryDocument};

fn document() -> RegistryDocument {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../data/registry/registry-source-v1.json"
    );
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn invalid(property: PhaseProperty, value: f64) -> RegistryDocument {
    let mut doc = document();
    let found: Vec<_> = doc
        .phase_thermodynamics
        .iter_mut()
        .filter(|row| row.species_id == "MgSO4" && row.property == property)
        .collect();
    assert_eq!(found.len(), 1);
    for row in found {
        row.quantity.value = value;
    }
    doc
}

#[test]
fn supported_complete_registry_and_pack_remain_supported() {
    let doc = document();
    doc.validate().unwrap();
    assert_eq!(load_pack(&build_pack(&doc)).unwrap(), doc);
}
#[test]
fn typed_negative_molar_mass_refuses() {
    assert!(invalid(PhaseProperty::MolarMass, -1.0).validate().is_err());
}
#[test]
fn typed_zero_molar_mass_refuses() {
    assert!(invalid(PhaseProperty::MolarMass, 0.0).validate().is_err());
}
#[test]
fn typed_negative_density_refuses() {
    assert!(invalid(PhaseProperty::MassDensity, -1.0)
        .validate()
        .is_err());
}
#[test]
fn typed_zero_density_refuses() {
    assert!(invalid(PhaseProperty::MassDensity, 0.0).validate().is_err());
}
#[test]
fn typed_negative_heat_capacity_refuses() {
    assert!(invalid(PhaseProperty::MolarHeatCapacity, -1.0)
        .validate()
        .is_err());
}
#[test]
fn well_hashed_negative_density_pack_refuses() {
    assert!(load_pack(&build_pack(&invalid(PhaseProperty::MassDensity, -1.0))).is_err());
}
#[test]
fn well_hashed_zero_molar_mass_pack_refuses() {
    assert!(load_pack(&build_pack(&invalid(PhaseProperty::MolarMass, 0.0))).is_err());
}
#[test]
fn well_hashed_negative_heat_capacity_pack_refuses() {
    assert!(load_pack(&build_pack(&invalid(
        PhaseProperty::MolarHeatCapacity,
        -1.0
    )))
    .is_err());
}
#[test]
fn signed_dissolution_enthalpy_remains_supported() {
    let doc = document();
    assert!(doc.phase_thermodynamics.iter().any(|row| row.property
        == PhaseProperty::EnthalpyOfDissolution
        && row.quantity.value < 0.0));
    doc.validate().unwrap();
    assert!(load_pack(&build_pack(&doc)).is_ok());
}
