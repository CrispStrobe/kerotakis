//! Frozen direct runtime-loader controls; no new chemical data or promotion.
use kerotakis_core::species_loader::parse_document;
use serde_json::{json, Value};

fn document(key: &str) -> Value {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../data/registry/registry-source-v1.json"
    );
    let mut doc: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    doc["identities"]
        .as_array_mut()
        .unwrap()
        .retain(|row| row["id"] == key);
    assert_eq!(doc["identities"].as_array().unwrap().len(), 1);
    doc
}

fn set_property(doc: &mut Value, key: &str, property: &str, value: Value) {
    let rows = doc["phase_thermodynamics"].as_array_mut().unwrap();
    let found: Vec<_> = rows
        .iter_mut()
        .filter(|row| row["species_id"] == key && row["property"] == property)
        .collect();
    assert_eq!(found.len(), 1);
    for row in found {
        row["quantity"]["value"] = value.clone();
    }
}

fn rejects(property: &str, value: Value) {
    let mut doc = document("MgSO4");
    set_property(&mut doc, "MgSO4", property, value);
    assert!(
        parse_document(&doc).is_err(),
        "invalid {property} must refuse before constructing a species"
    );
}

#[test]
fn supported_solid_registry_properties_remain_supported() {
    let species = parse_document(&document("MgSO4")).unwrap();
    assert_eq!(species.len(), 1);
    assert_eq!(species[0].key, "MgSO4");
    assert!(
        species[0].molar_mass > 0.0 && species[0].density > 0.0 && species[0].heat_capacity > 0.0
    );
}

#[test]
fn aqueous_ion_zero_heat_capacity_remains_supported() {
    let species = parse_document(&document("Mg+2")).unwrap();
    assert_eq!(species.len(), 1);
    assert_eq!(species[0].heat_capacity, 0.0);
}

#[test]
fn missing_heat_capacity_refuses() {
    rejects("molar_heat_capacity", Value::Null);
}

#[test]
fn missing_density_refuses() {
    rejects("mass_density", Value::Null);
}

#[test]
fn negative_heat_capacity_refuses() {
    rejects("molar_heat_capacity", json!(-1.0));
}

#[test]
fn negative_density_refuses() {
    rejects("mass_density", json!(-1.0));
}

#[test]
fn zero_density_refuses() {
    rejects("mass_density", json!(0.0));
}

#[test]
fn negative_molar_mass_refuses() {
    rejects("molar_mass", json!(-1.0));
}

#[test]
fn zero_molar_mass_refuses() {
    rejects("molar_mass", json!(0.0));
}
