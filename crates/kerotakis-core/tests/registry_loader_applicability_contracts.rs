//! Source-informed context-free runtime fallback contracts, frozen before execution.
//! Synthetic conditions are engineering inputs, not chemical measurements.
use kerotakis_core::species_loader::parse_document;
use kerotakis_data::{build_pack, load_pack, select_runtime_scalar_records, RegistryDocument};
use serde_json::{json, Value};
const PROPERTIES: [&str; 3] = ["molar_mass", "molar_heat_capacity", "mass_density"];
const CONSTRAINTS: [&str; 5] = ["temperature", "pressure", "ph", "ionic_strength", "medium"];
fn document() -> Value {
    serde_json::from_str(include_str!(
        "../../../data/registry/registry-source-v1.json"
    ))
    .unwrap()
}
fn row_index(doc: &Value, property: &str) -> usize {
    let rows: Vec<_> = doc["phase_thermodynamics"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .filter(|(_, row)| row["species_id"] == "MgSO4" && row["property"] == property)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(rows.len(), 1);
    rows[0]
}
fn constraint(field: &str) -> Value {
    match field {
        "temperature" => json!({"lower": 290.0, "upper": 310.0,
            "unit": {"symbol": "K", "dimension": "temperature"}}),
        "pressure" => json!({"lower": 90000.0, "upper": 110000.0,
            "unit": {"symbol": "Pa", "dimension": "pressure"}}),
        "ph" => json!({"lower": 6.0, "upper": 8.0,
            "unit": {"symbol": "1", "dimension": "dimensionless"}}),
        "ionic_strength" => json!({"lower": 0.0, "upper": 0.1,
            "unit": {"symbol": "mol/L", "dimension": "concentration"}}),
        "medium" => json!("water"),
        _ => panic!("unknown synthetic constraint"),
    }
}
fn assert_error(error: String, property: &str, field: &str) {
    let error = error.to_lowercase();
    assert!(error.contains("mgso4"), "species-free refusal: {error}");
    assert!(error.contains(property), "wrong-property refusal: {error}");
    assert!(error.contains(field), "unrelated refusal: {error}");
}
fn assert_runtime_refuses(doc: &Value, property: &str, field: &str) {
    let rows = doc["phase_thermodynamics"].as_array().unwrap();
    assert_error(
        select_runtime_scalar_records(rows, "MgSO4")
            .err()
            .expect("context-free selector admitted conditional mandatory record"),
        property,
        field,
    );
    assert_error(
        parse_document(doc)
            .expect_err("direct runtime silently admitted conditional mandatory record"),
        property,
        field,
    );
    let typed: RegistryDocument = serde_json::from_value(doc.clone()).unwrap();
    let decoded =
        load_pack(&build_pack(&typed)).expect("pack DTO admission must retain conditional data");
    let unpacked = serde_json::to_value(decoded).unwrap();
    assert_error(
        parse_document(&unpacked)
            .expect_err("pack-to-runtime silently admitted conditional mandatory record"),
        property,
        field,
    );
}
fn assert_typed_pack_retains(doc: &Value) {
    let typed: RegistryDocument = serde_json::from_value(doc.clone()).unwrap();
    assert!(
        typed.validate().is_ok(),
        "well-formed conditional typed document must remain valid"
    );
    let before = serde_json::to_value(&typed).unwrap();
    let decoded = load_pack(&build_pack(&typed)).unwrap();
    assert_eq!(
        before,
        serde_json::to_value(decoded).unwrap(),
        "complete conditional DTO metadata changed"
    );
}
fn constrained(property: &str, field: &str) {
    let mut doc = document();
    let i = row_index(&doc, property);
    doc["phase_thermodynamics"][i]["quantity"]["conditions"][field] = constraint(field);
    assert_typed_pack_retains(&doc);
    assert_runtime_refuses(&doc, property, field);
}
fn phase_conflict(property: &str) {
    let mut doc = document();
    let i = row_index(&doc, property);
    assert_eq!(doc["phase_thermodynamics"][i]["phase"], "solid");
    doc["phase_thermodynamics"][i]["quantity"]["conditions"]["phase"] = json!("gas");
    // General typed validity is separate; pack admission remains bounded to scalar units/signs.
    assert_runtime_refuses(&doc, property, "phase");
}
fn same_runtime(doc: &Value) {
    let canonical = document();
    let expected = parse_document(&canonical).unwrap();
    assert_eq!(expected.len(), 188);
    assert_eq!(
        expected.iter().filter(|s| s.heat_capacity == 0.0).count(),
        49
    );
    let typed: RegistryDocument = serde_json::from_value(doc.clone()).unwrap();
    assert!(typed.validate().is_ok());
    let decoded = load_pack(&build_pack(&typed)).unwrap();
    let unpacked = serde_json::to_value(decoded).unwrap();
    for source in [doc, &unpacked] {
        let rows = source["phase_thermodynamics"].as_array().unwrap();
        for e in &expected {
            let selected = select_runtime_scalar_records(rows, e.key)
                .unwrap_or_else(|error| panic!("{error}"));
            assert_eq!(
                selected.molar_mass["phase"],
                selected.heat_capacity["phase"]
            );
            assert_eq!(selected.molar_mass["phase"], selected.density["phase"]);
        }
        let actual = parse_document(source).unwrap();
        assert_eq!(actual.len(), expected.len());
        for e in &expected {
            let a = actual.iter().find(|s| s.key == e.key).unwrap();
            assert_eq!(
                (a.molar_mass, a.heat_capacity, a.density, a.standard_phase),
                (e.molar_mass, e.heat_capacity, e.density, e.standard_phase)
            );
        }
    }
}

#[test]
fn mass_temperature_requires_runtime_context() {
    constrained("molar_mass", "temperature");
}

#[test]
fn mass_pressure_requires_runtime_context() {
    constrained("molar_mass", "pressure");
}

#[test]
fn mass_ph_requires_runtime_context() {
    constrained("molar_mass", "ph");
}

#[test]
fn mass_ionic_strength_requires_runtime_context() {
    constrained("molar_mass", "ionic_strength");
}

#[test]
fn mass_medium_requires_runtime_context() {
    constrained("molar_mass", "medium");
}

#[test]
fn cp_temperature_requires_runtime_context() {
    constrained("molar_heat_capacity", "temperature");
}

#[test]
fn cp_pressure_requires_runtime_context() {
    constrained("molar_heat_capacity", "pressure");
}

#[test]
fn cp_ph_requires_runtime_context() {
    constrained("molar_heat_capacity", "ph");
}

#[test]
fn cp_ionic_strength_requires_runtime_context() {
    constrained("molar_heat_capacity", "ionic_strength");
}

#[test]
fn cp_medium_requires_runtime_context() {
    constrained("molar_heat_capacity", "medium");
}

#[test]
fn density_temperature_requires_runtime_context() {
    constrained("mass_density", "temperature");
}

#[test]
fn density_pressure_requires_runtime_context() {
    constrained("mass_density", "pressure");
}

#[test]
fn density_ph_requires_runtime_context() {
    constrained("mass_density", "ph");
}

#[test]
fn density_ionic_strength_requires_runtime_context() {
    constrained("mass_density", "ionic_strength");
}

#[test]
fn density_medium_requires_runtime_context() {
    constrained("mass_density", "medium");
}

#[test]
fn mass_condition_phase_conflict_refuses() {
    phase_conflict("molar_mass");
}

#[test]
fn cp_condition_phase_conflict_refuses() {
    phase_conflict("molar_heat_capacity");
}

#[test]
fn density_condition_phase_conflict_refuses() {
    phase_conflict("mass_density");
}

#[test]
fn canonical_phase_consistent_unbounded_records_preserve_all_values() {
    same_runtime(&document());
}
#[test]
fn explicit_null_constraints_and_notes_preserve_all_values() {
    let mut doc = document();
    for property in PROPERTIES {
        let i = row_index(&doc, property);
        for field in CONSTRAINTS {
            doc["phase_thermodynamics"][i]["quantity"]["conditions"][field] = Value::Null;
        }
        doc["phase_thermodynamics"][i]["quantity"]["conditions"]["notes"] =
            json!("Synthetic descriptive note; no applicability restriction");
    }
    same_runtime(&doc);
}
#[test]
fn omitted_conditions_use_unbounded_default_and_preserve_all_values() {
    let mut doc = document();
    for property in PROPERTIES {
        let i = row_index(&doc, property);
        doc["phase_thermodynamics"][i]["quantity"]
            .as_object_mut()
            .unwrap()
            .remove("conditions");
    }
    same_runtime(&doc);
}
#[test]
fn null_condition_phase_preserves_all_values() {
    let mut doc = document();
    for property in PROPERTIES {
        let i = row_index(&doc, property);
        doc["phase_thermodynamics"][i]["quantity"]["conditions"]["phase"] = Value::Null;
    }
    same_runtime(&doc);
}
#[test]
fn excluded_conditional_foreign_phase_records_preserve_all_values() {
    let mut doc = document();
    for property in ["molar_heat_capacity", "mass_density"] {
        let i = row_index(&doc, property);
        let mut extra = doc["phase_thermodynamics"][i].clone();
        extra["id"] = json!(format!("independent_conditional_foreign_{property}"));
        extra["phase"] = json!("gas");
        extra["quantity"]["conditions"]["phase"] = json!("gas");
        extra["quantity"]["value"] = json!(extra["quantity"]["value"].as_f64().unwrap() * 2.0);
        for field in CONSTRAINTS {
            extra["quantity"]["conditions"][field] = constraint(field);
        }
        doc["phase_thermodynamics"]
            .as_array_mut()
            .unwrap()
            .insert(0, extra);
    }
    assert_typed_pack_retains(&doc);
    same_runtime(&doc);
}
#[test]
fn conditional_typed_and_pack_documents_preserve_complete_metadata() {
    let mut doc = document();
    for property in PROPERTIES {
        let i = row_index(&doc, property);
        for field in CONSTRAINTS {
            doc["phase_thermodynamics"][i]["quantity"]["conditions"][field] = constraint(field);
        }
        doc["phase_thermodynamics"][i]["quantity"]["conditions"]["notes"] =
            json!("Synthetic context-bound evidence; preserve this metadata");
    }
    assert_typed_pack_retains(&doc);
}
