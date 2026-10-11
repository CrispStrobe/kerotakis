//! Additional alias numeric-equivalence controls, frozen separately.
use kerotakis_core::species_loader::parse_document;
use serde_json::Value;
fn equivalent_alias(property: &str, symbol: &str) {
    let mut doc: Value = serde_json::from_str(include_str!(
        "../../../data/registry/registry-source-v1.json"
    ))
    .unwrap();
    let before = parse_document(&doc).unwrap();
    let rows: Vec<_> = doc["phase_thermodynamics"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .filter(|r| r["species_id"] == "MgSO4" && r["property"] == property)
        .collect();
    assert_eq!(rows.len(), 1);
    for row in rows {
        row["quantity"]["unit"]["symbol"] = Value::String(symbol.to_owned());
    }
    let after = parse_document(&doc).unwrap();
    let a = before.iter().find(|r| r.key == "MgSO4").unwrap();
    let b = after.iter().find(|r| r.key == "MgSO4").unwrap();
    assert_eq!(
        (a.molar_mass, a.density, a.heat_capacity),
        (b.molar_mass, b.density, b.heat_capacity)
    );
}
#[test]
fn cp_equivalent_alias_preserves_runtime_values() {
    equivalent_alias("molar_heat_capacity", "J.mol-1.K-1");
}
#[test]
fn density_equivalent_alias_preserves_runtime_values() {
    equivalent_alias("mass_density", "g/cm3");
}
