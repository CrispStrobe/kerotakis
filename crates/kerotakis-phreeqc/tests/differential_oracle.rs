#![cfg(all(
    feature = "engine",
    any(feature = "legacy-basic-oracle", feature = "my-basic")
))]

use kerotakis_phreeqc::{databases, Phreeqc};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn load_expected(name: &str) -> HashMap<String, (f64, f64)> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/oracle/expected")
        .join(format!("{name}.json"));
    let content = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let parsed: serde_json::Value =
        serde_json::from_str(&content).unwrap_or_else(|e| panic!("parse {name}.json: {e}"));
    let observables = parsed["observables"].as_object().expect("observables map");
    observables
        .iter()
        .map(|(key, spec)| {
            let value = spec["value"].as_f64().expect("value field");
            let tol = spec["absolute_tolerance"].as_f64().unwrap_or(1e-6);
            (key.clone(), (value, tol))
        })
        .collect()
}

fn load_input(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/oracle/inputs")
        .join(format!("{name}.pqi"));
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn run_and_check(name: &str) {
    let input = load_input(name);
    let expected = load_expected(name);
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine.run(&input).unwrap();

    for (key, (expected_value, tolerance)) in &expected {
        let actual = if key.starts_with("k_") || key.starts_with("V_") {
            engine
                .last_value(key)
                .unwrap_or_else(|| panic!("missing output: {key}"))
        } else {
            let rows = engine.selected_output();
            let headings = &rows[0];
            let values = rows.last().expect("data row");
            let idx = headings
                .iter()
                .position(|h| h == key)
                .unwrap_or_else(|| panic!("missing heading: {key}"));
            values[idx]
                .parse::<f64>()
                .unwrap_or_else(|e| panic!("{key} not numeric: {e}"))
        };
        assert!(
            (actual - expected_value).abs() <= *tolerance,
            "{name}/{key}: actual={actual}, expected={expected_value}, tol={tolerance}"
        );
    }
}

#[test]
fn oracle_simple_kinetics() {
    run_and_check("simple_kinetics");
}

#[test]
fn oracle_calc_values_chain() {
    run_and_check("calc_values_chain");
}

#[test]
fn oracle_user_punch_multicolumn() {
    run_and_check("user_punch_multicolumn");
}

#[test]
fn oracle_data_read_rate() {
    run_and_check("data_read_rate");
}
