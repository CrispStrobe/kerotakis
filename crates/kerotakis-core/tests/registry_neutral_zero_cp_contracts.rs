//! Supplemental direct-loader compatibility: zero Cp is not exclusive to ions.
use kerotakis_core::species_loader::parse_document;
use serde_json::Value;

fn supported(key: &str) {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../data/registry/registry-source-v1.json"
    );
    let mut doc: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    doc["identities"]
        .as_array_mut()
        .unwrap()
        .retain(|row| row["id"] == key);
    let species = parse_document(&doc).unwrap();
    assert_eq!(species.len(), 1);
    assert_eq!(species[0].heat_capacity, 0.0);
}
#[test]
fn neutral_hypochlorous_acid_zero_cp_remains_supported() {
    supported("HClO");
}
#[test]
fn neutral_dissolved_carbon_dioxide_zero_cp_remains_supported() {
    supported("CO2(aq)");
}
