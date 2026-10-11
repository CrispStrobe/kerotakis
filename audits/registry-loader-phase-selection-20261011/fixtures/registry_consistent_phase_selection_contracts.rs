//! Independently authored runtime phase/ambiguity controls frozen before execution.
use kerotakis_core::species_loader::parse_document;
use serde_json::{json, Value};
fn document() -> Value {
    serde_json::from_str(include_str!(
        "../../../data/registry/registry-source-v1.json"
    ))
    .unwrap()
}
fn row_index(doc: &Value, property: &str) -> usize {
    let indices: Vec<_> = doc["phase_thermodynamics"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .filter(|(_, r)| r["species_id"] == "MgSO4" && r["property"] == property)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(indices.len(), 1);
    indices[0]
}
fn phase_or_ambiguity_refusal(doc: &Value) {
    let error = match parse_document(doc) {
        Ok(_) => panic!("phase-mismatched or ambiguous runtime property admitted"),
        Err(e) => e.to_lowercase(),
    };
    assert!(error.contains("mgso4"), "{error}");
    assert!(
        error.contains("phase") || error.contains("ambig") || error.contains("duplicate"),
        "unrelated refusal: {error}"
    );
}
fn same_values(doc: &Value, may_refuse: bool) {
    let expected = parse_document(&document()).unwrap();
    let actual = match parse_document(doc) {
        Ok(v) => v,
        Err(_) if may_refuse => {
            phase_or_ambiguity_refusal(doc);
            return;
        }
        Err(e) => panic!("supported input refused: {e}"),
    };
    assert_eq!(actual.len(), expected.len());
    for e in &expected {
        let a = actual.iter().find(|r| r.key == e.key).unwrap();
        assert_eq!(
            (a.molar_mass, a.density, a.heat_capacity, a.standard_phase),
            (e.molar_mass, e.density, e.heat_capacity, e.standard_phase)
        );
    }
}
fn foreign_record(property: &str, prepend: bool) {
    let mut doc = document();
    let i = row_index(&doc, property);
    let mut extra = doc["phase_thermodynamics"][i].clone();
    extra["id"] = json!(format!("independent_foreign_{property}"));
    extra["phase"] = json!("gas");
    extra["quantity"]["conditions"]["phase"] = json!("gas");
    extra["quantity"]["value"] = json!(extra["quantity"]["value"].as_f64().unwrap() * 2.0);
    let rows = doc["phase_thermodynamics"].as_array_mut().unwrap();
    if prepend {
        rows.insert(0, extra);
    } else {
        rows.push(extra);
    }
    same_values(&doc, true);
}
fn wrong_phase(property: &str) {
    let mut doc = document();
    let i = row_index(&doc, property);
    doc["phase_thermodynamics"][i]["phase"] = json!("gas");
    doc["phase_thermodynamics"][i]["quantity"]["conditions"]["phase"] = json!("gas");
    phase_or_ambiguity_refusal(&doc);
}

#[test]
fn consistent_foreign_cp_prepend() {
    foreign_record("molar_heat_capacity", true);
}

#[test]
fn consistent_foreign_cp_append() {
    foreign_record("molar_heat_capacity", false);
}

#[test]
fn consistent_foreign_density_prepend() {
    foreign_record("mass_density", true);
}

#[test]
fn consistent_foreign_density_append() {
    foreign_record("mass_density", false);
}

#[test]
fn consistent_wrong_phase_cp_refuses() {
    wrong_phase("molar_heat_capacity");
}

#[test]
fn consistent_wrong_phase_density_refuses() {
    wrong_phase("mass_density");
}
