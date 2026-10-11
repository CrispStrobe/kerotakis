//! Prospective candidate controls: valid scaled units retain the complete typed document.
use kerotakis_core::species_loader::parse_document;
use kerotakis_data::{build_pack, load_pack, RegistryDocument};
use serde_json::{json, Value};
fn scaled_pack_preserves_metadata(property: &str, symbol: &str, factor: f64) {
    let mut doc: Value = serde_json::from_str(include_str!(
        "../../../data/registry/registry-source-v1.json"
    ))
    .unwrap();
    let canonical = parse_document(&doc).unwrap();
    let rows: Vec<_> = doc["phase_thermodynamics"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .filter(|row| row["species_id"] == "MgSO4" && row["property"] == property)
        .collect();
    assert_eq!(rows.len(), 1);
    for row in rows {
        row["quantity"]["value"] = json!(row["quantity"]["value"].as_f64().unwrap() / factor);
        row["quantity"]["unit"]["symbol"] = json!(symbol);
    }
    let typed: RegistryDocument = serde_json::from_value(doc).unwrap();
    assert!(typed.validate().is_ok());
    let before = serde_json::to_value(&typed).unwrap();
    let decoded = load_pack(&build_pack(&typed)).unwrap();
    let after = serde_json::to_value(&decoded).unwrap();
    assert_eq!(
        before, after,
        "pack admission must preserve the complete valid metadata-bearing document"
    );
    let runtime = parse_document(&after).unwrap();
    let expected = canonical.iter().find(|row| row.key == "MgSO4").unwrap();
    let actual = runtime.iter().find(|row| row.key == "MgSO4").unwrap();
    for (a, b) in [
        (actual.molar_mass, expected.molar_mass),
        (actual.density, expected.density),
        (actual.heat_capacity, expected.heat_capacity),
    ] {
        assert!((a - b).abs() <= b.abs().max(1.0) * 1e-12);
    }
}
#[test]
fn scaled_mass_pack_preserves_document_and_normalizes_runtime() {
    scaled_pack_preserves_metadata("molar_mass", "kg/mol", 1000.0);
}
#[test]
fn scaled_density_pack_preserves_document_and_normalizes_runtime() {
    scaled_pack_preserves_metadata("mass_density", "kg/m3", 0.001);
}
#[test]
fn scaled_cp_pack_preserves_document_and_normalizes_runtime() {
    scaled_pack_preserves_metadata("molar_heat_capacity", "kJ/(mol.K)", 1000.0);
}
