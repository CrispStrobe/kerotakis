//! Separately frozen direct and pack-runtime applicability witnesses.
//! These do not call the shared selector first, so baseline route evidence is independent.
use kerotakis_core::species_loader::parse_document;
use kerotakis_data::{build_pack, load_pack, RegistryDocument};
use serde_json::{json, Value};
fn conditional_document() -> Value {
    let mut doc: Value = serde_json::from_str(include_str!(
        "../../../data/registry/registry-source-v1.json"
    ))
    .unwrap();
    let rows: Vec<_> = doc["phase_thermodynamics"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .filter(|row| row["species_id"] == "MgSO4" && row["property"] == "molar_heat_capacity")
        .collect();
    assert_eq!(rows.len(), 1);
    for row in rows {
        row["quantity"]["conditions"]["temperature"] = json!({
            "lower": 290.0, "upper": 310.0,
            "unit": {"symbol": "K", "dimension": "temperature"}
        });
    }
    doc
}
fn retained_pack_document(doc: &Value) -> Value {
    let typed: RegistryDocument = serde_json::from_value(doc.clone()).unwrap();
    assert!(
        typed.validate().is_ok(),
        "conditional typed document must remain valid"
    );
    let before = serde_json::to_value(&typed).unwrap();
    let decoded = load_pack(&build_pack(&typed)).unwrap();
    let after = serde_json::to_value(decoded).unwrap();
    assert_eq!(
        before, after,
        "complete conditional pack DTO metadata must survive"
    );
    after
}
fn assert_runtime_refusal(doc: &Value) {
    let error = parse_document(doc)
        .err()
        .expect("runtime silently admitted a temperature-constrained fallback Cp")
        .to_lowercase();
    assert!(error.contains("mgso4"), "species-free refusal: {error}");
    assert!(
        error.contains("molar_heat_capacity"),
        "wrong-property refusal: {error}"
    );
    assert!(error.contains("temperature"), "unrelated refusal: {error}");
}
#[test]
fn direct_runtime_temperature_constraint_refuses_without_selector_precheck() {
    let doc = conditional_document();
    // Establish DTO retention before observing the independently invoked direct runtime route.
    retained_pack_document(&doc);
    assert_runtime_refusal(&doc);
}
#[test]
fn decoded_pack_runtime_temperature_constraint_refuses_without_selector_precheck() {
    let doc = conditional_document();
    let decoded = retained_pack_document(&doc);
    assert_runtime_refusal(&decoded);
}
