//! Kinetics trajectory parity: verify that the MY-BASIC preview adapter
//! produces identical time-stepped remaining-moles trajectories to the
//! legacy oracle through PHREEQC's internal Runge-Kutta integrator.
//!
//! Each fixture runs a multi-step KINETICS block and compares the remaining
//! moles at every integration step — not just the final value. This catches
//! cumulative drift between interpreter backends.

#![cfg(all(
    feature = "engine",
    feature = "my-basic",
))]

use kerotakis_phreeqc::{databases, Phreeqc};
use std::fs;
use std::path::Path;

/// Expected trajectory point captured from the legacy oracle.
#[derive(Debug)]
struct TrajectoryExpected {
    column: String,
    /// One expected value per integration step (row).
    values: Vec<f64>,
    tolerance: f64,
}

/// Load a trajectory-format expected file.
///
/// Format:
/// ```json
/// {
///   "description": "...",
///   "oracle": "MY-BASIC adapter",
///   "trajectories": {
///     "k_Decay": {
///       "values": [0.882, 0.778, 0.606, 0.367],
///       "absolute_tolerance": 5e-4
///     }
///   }
/// }
/// ```
fn load_trajectory_expected(name: &str) -> Vec<TrajectoryExpected> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/oracle/expected")
        .join(format!("{name}.json"));
    let content = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let parsed: serde_json::Value =
        serde_json::from_str(&content).unwrap_or_else(|e| panic!("parse {name}.json: {e}"));
    let trajectories = parsed["trajectories"]
        .as_object()
        .expect("trajectories map");
    trajectories
        .iter()
        .map(|(key, spec)| {
            let values: Vec<f64> = spec["values"]
                .as_array()
                .expect("values array")
                .iter()
                .map(|v| v.as_f64().expect("numeric value"))
                .collect();
            let tolerance = spec["absolute_tolerance"].as_f64().unwrap_or(1e-6);
            TrajectoryExpected {
                column: key.clone(),
                values,
                tolerance,
            }
        })
        .collect()
}

fn load_input(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/oracle/inputs")
        .join(format!("{name}.pqi"));
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn run_and_check_trajectory(name: &str) {
    let input = load_input(name);
    let expected_list = load_trajectory_expected(name);
    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    engine
        .run(&input)
        .unwrap_or_else(|e| panic!("{name} failed: {e}"));

    let rows = engine.selected_output();
    assert!(
        rows.len() >= 2,
        "{name}: expected at least 2 rows (heading + data), got {}",
        rows.len()
    );
    let headings = &rows[0];
    let data_rows = &rows[1..];

    for expected in &expected_list {
        let col_idx = headings
            .iter()
            .position(|h| h == &expected.column)
            .unwrap_or_else(|| {
                panic!(
                    "{name}: missing column '{}' in headings {:?}",
                    expected.column, headings
                )
            });
        assert_eq!(
            data_rows.len(),
            expected.values.len(),
            "{name}/{}: expected {} rows, got {}",
            expected.column,
            expected.values.len(),
            data_rows.len()
        );
        for (step, (row, &oracle_value)) in
            data_rows.iter().zip(&expected.values).enumerate()
        {
            let actual: f64 = row[col_idx]
                .parse()
                .unwrap_or_else(|e| panic!("{name}/{} step {step}: not numeric: {e}", expected.column));
            assert!(
                (actual - oracle_value).abs() <= expected.tolerance,
                "{name}/{} step {step}: actual={actual:.15e}, oracle={oracle_value:.15e}, \
                 diff={:.3e}, tol={:.3e}",
                expected.column,
                (actual - oracle_value).abs(),
                expected.tolerance,
            );
        }
    }
}

#[test]
fn trajectory_multistep_firstorder_decay() {
    run_and_check_trajectory("kinetics_multistep");
}

#[test]
fn trajectory_temperature_dependent_rate() {
    run_and_check_trajectory("kinetics_temperature");
}

#[test]
fn trajectory_data_driven_rate_constant() {
    run_and_check_trajectory("kinetics_data_rate");
}

#[test]
fn trajectory_multicomponent_kinetics() {
    run_and_check_trajectory("kinetics_multicomponent");
}

#[test]
fn trajectory_parm_array_fractional_order() {
    run_and_check_trajectory("kinetics_parm_array");
}
