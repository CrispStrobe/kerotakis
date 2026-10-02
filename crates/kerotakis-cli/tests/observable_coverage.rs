//! Capability reports must agree with the state and support exposed by `run`.
use serde_json::Value;
use std::process::{Command, Output};

fn run_case(name: &str, script: &str, args: &[&str]) -> Output {
    let path =
        std::env::temp_dir().join(format!("kero-observable-{}-{name}.lab", std::process::id()));
    std::fs::write(&path, script).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args(args)
        .arg(&path)
        .arg("--json")
        .output()
        .unwrap();
    std::fs::remove_file(path).unwrap();
    output
}

#[test]
fn coverage_matches_final_scene_after_real_chemistry() {
    let script = "add v1 water 100mL\nadd v1 NaCl 0.001mol\nheat v1 100J\n";
    let coverage = run_case("coverage", script, &["coverage", "observables"]);
    assert!(
        coverage.status.success(),
        "{}",
        String::from_utf8_lossy(&coverage.stderr)
    );
    let report: Value = serde_json::from_slice(&coverage.stdout).unwrap();
    let run = run_case("run", script, &["run"]);
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    let last: Value = serde_json::from_str(
        String::from_utf8(run.stdout)
            .unwrap()
            .lines()
            .last()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(report["schema_version"], 1);
    assert_eq!(
        report["vessels"][0]["observables"],
        last["scene"]["vessels"][0]["observables"]
    );
}

#[test]
fn a_missing_rate_is_a_supported_report_with_an_unsupported_observable() {
    let output = run_case(
        "rate",
        "add v1 water 100mL\nadd v1 HCl 0.001mol\nadd v1 Zn 0.001mol\n",
        &["coverage", "observables"],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rate = report["vessels"][0]["observables"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["observable"] == "reaction_rate")
        .unwrap();
    assert_eq!(rate["status"], "unsupported");
    assert!(!rate["reasons"].as_array().unwrap().is_empty());
}

#[test]
fn invalid_input_never_produces_a_successful_capability_report() {
    let output = run_case(
        "invalid",
        "add v1 water 100mL\nthis is not kero syntax\n",
        &["coverage", "observables"],
    );
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("line 2"));
}
