use std::process::Command;

#[test]
fn curiosity_smoke_routes_without_crashing() {
    let output = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args(["coverage", "curiosity", "--smoke", "--check", "--json"])
        .output()
        .expect("run kero coverage curiosity");
    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("one JSON coverage report");
    assert_eq!(report["prompts"].as_array().map(Vec::len), Some(16));
    assert_eq!(report["expectation_mismatches"], 0);
    assert_eq!(report["failures"].as_array().map(Vec::len), Some(0));
    for disposition in ["computed", "curated", "qualitative", "boundary", "missing"] {
        assert!(
            report["by_observed"][disposition]
                .as_u64()
                .is_some_and(|count| count > 0),
            "smoke report omitted {disposition}"
        );
    }

    let repeated = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args(["coverage", "curiosity", "--smoke", "--check", "--json"])
        .output()
        .expect("repeat coverage report");
    assert!(repeated.status.success());
    assert_eq!(
        output.stdout, repeated.stdout,
        "JSON report must be byte-stable"
    );
}

#[test]
#[ignore = "scheduled native curiosity corpus"]
fn curiosity_full_reports_every_prompt_or_failure() {
    let output = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args(["coverage", "curiosity", "--json"])
        .output()
        .expect("run full curiosity corpus");
    assert!(output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON report");
    let classified = report["prompts"].as_array().map(Vec::len).unwrap_or(0);
    let failed = report["failures"].as_array().map(Vec::len).unwrap_or(0);
    assert_eq!(classified + failed, 500);
}
