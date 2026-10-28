//! Independent unit-admission controls frozen before first execution.
use kerotakis_core::species_loader::parse_document;
use kerotakis_data::{build_pack, load_pack, RegistryDocument};
use serde_json::{json, Value};
fn document() -> Value {
    serde_json::from_str(include_str!(
        "../../../data/registry/registry-source-v1.json"
    ))
    .unwrap()
}
fn quantity<'a>(doc: &'a mut Value, property: &str) -> &'a mut Value {
    let matches: Vec<_> = doc["phase_thermodynamics"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .filter(|row| row["species_id"] == "MgSO4" && row["property"] == property)
        .collect();
    assert_eq!(matches.len(), 1);
    &mut matches.into_iter().next().unwrap()["quantity"]
}
fn probe(property: &str, symbol: &str, factor: f64, wrong_dimension: bool) -> Value {
    let mut doc = document();
    let q = quantity(&mut doc, property);
    q["value"] = json!(q["value"].as_f64().unwrap() / factor);
    q["unit"]["symbol"] = json!(symbol);
    if wrong_dimension {
        q["unit"]["dimension"] = json!("molar_mass");
    }
    doc
}
fn typed(doc: Value) -> RegistryDocument {
    serde_json::from_value(doc).unwrap()
}
fn meaningful_refusal(error: impl std::fmt::Display) {
    let text = error.to_string().to_lowercase();
    assert!(
        text.contains("unit") || text.contains("dimension"),
        "unrelated refusal: {text}"
    );
}
fn direct_refuses(doc: Value) {
    match parse_document(&doc) {
        Ok(_) => panic!("invalid unit metadata admitted"),
        Err(e) => meaningful_refusal(e),
    }
}
fn pack_refuses(doc: Value) {
    match load_pack(&build_pack(&typed(doc))) {
        Ok(_) => panic!("well-hashed invalid units admitted"),
        Err(e) => meaningful_refusal(e),
    }
}
fn equivalent(doc: Value, packed: bool) {
    let expected = parse_document(&document()).unwrap();
    let expected = expected.iter().find(|row| row.key == "MgSO4").unwrap();
    let doc = if packed {
        match load_pack(&build_pack(&typed(doc))) {
            Ok(v) => serde_json::to_value(v).unwrap(),
            Err(e) => {
                meaningful_refusal(e);
                return;
            }
        }
    } else {
        doc
    };
    let actual = match parse_document(&doc) {
        Ok(v) => v,
        Err(e) => {
            meaningful_refusal(e);
            return;
        }
    };
    let actual = actual.iter().find(|row| row.key == "MgSO4").unwrap();
    for (a, b) in [
        (actual.molar_mass, expected.molar_mass),
        (actual.density, expected.density),
        (actual.heat_capacity, expected.heat_capacity),
    ] {
        assert!(
            (a - b).abs() <= b.abs().max(1.0) * 1e-12,
            "scaled unit silently misinterpreted: {a} vs {b}"
        );
    }
}
#[test]
fn canonical_registry_runtime_and_pack_supported() {
    let doc = document();
    assert_eq!(parse_document(&doc).unwrap().len(), 188);
    assert!(load_pack(&build_pack(&typed(doc))).is_ok());
}

#[test]
fn equivalent_cp_alias_runtime_supported() {
    assert!(parse_document(&probe("molar_heat_capacity", "J.mol-1.K-1", 1.0, false)).is_ok());
}

#[test]
fn equivalent_density_alias_runtime_supported() {
    assert!(parse_document(&probe("mass_density", "g/cm3", 1.0, false)).is_ok());
}

#[test]
fn scaled_mass_direct_normalizes_or_refuses() {
    equivalent(probe("molar_mass", "kg/mol", 1000.0, false), false);
}

#[test]
fn scaled_density_direct_normalizes_or_refuses() {
    equivalent(probe("mass_density", "kg/m3", 0.001, false), false);
}

#[test]
fn scaled_cp_direct_normalizes_or_refuses() {
    equivalent(
        probe("molar_heat_capacity", "kJ/(mol.K)", 1000.0, false),
        false,
    );
}

#[test]
fn wrong_mass_symbol_direct_refuses() {
    direct_refuses(probe("molar_mass", "J/mol", 1.0, false));
}

#[test]
fn wrong_density_dimension_direct_refuses() {
    direct_refuses(probe("mass_density", "g/mL", 1.0, true));
}

#[test]
fn unknown_cp_symbol_direct_refuses() {
    direct_refuses(probe("molar_heat_capacity", "unreviewed-unit", 1.0, false));
}

#[test]
fn typed_equivalent_cp_alias_supported() {
    assert!(
        typed(probe("molar_heat_capacity", "J.mol-1.K-1", 1.0, false))
            .validate()
            .is_ok()
    );
}

#[test]
fn typed_scaled_mass_supported() {
    assert!(typed(probe("molar_mass", "kg/mol", 1000.0, false))
        .validate()
        .is_ok());
}

#[test]
fn typed_mass_symbol_dimension_conflict_refuses() {
    assert!(typed(probe("molar_mass", "J/mol", 1.0, false))
        .validate()
        .is_err());
}

#[test]
fn typed_unknown_cp_unit_refuses() {
    assert!(
        typed(probe("molar_heat_capacity", "unreviewed-unit", 1.0, false))
            .validate()
            .is_err()
    );
}

#[test]
fn typed_wrong_density_dimension_refuses() {
    assert!(typed(probe("mass_density", "g/mL", 1.0, true))
        .validate()
        .is_err());
}

#[test]
fn well_hashed_wrong_mass_unit_refuses() {
    pack_refuses(probe("molar_mass", "J/mol", 1.0, false));
}

#[test]
fn well_hashed_wrong_density_dimension_refuses() {
    pack_refuses(probe("mass_density", "g/mL", 1.0, true));
}

#[test]
fn well_hashed_unknown_cp_unit_refuses() {
    pack_refuses(probe("molar_heat_capacity", "unreviewed-unit", 1.0, false));
}

#[test]
fn scaled_mass_pack_runtime_normalizes_or_refuses() {
    equivalent(probe("molar_mass", "kg/mol", 1000.0, false), true);
}

#[test]
fn scaled_density_pack_runtime_normalizes_or_refuses() {
    equivalent(probe("mass_density", "kg/m3", 0.001, false), true);
}

#[test]
fn scaled_cp_pack_runtime_normalizes_or_refuses() {
    equivalent(
        probe("molar_heat_capacity", "kJ/(mol.K)", 1000.0, false),
        true,
    );
}
