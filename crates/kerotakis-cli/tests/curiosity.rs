use std::collections::BTreeSet;
use std::process::Command;

#[test]
fn curiosity_smoke_routes_without_crashing() {
    let output = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args(["coverage", "curiosity", "--smoke", "--check", "--json"])
        .output()
        .expect("run kero coverage curiosity");
    // The report is on STDOUT and the failure message printed only
    // stderr, so a drifting row failed this test with an empty panic and
    // the one artefact that says WHICH row was thrown away. `--check`
    // exits non-zero on drift, so this assertion is the one that fires
    // first and it has to carry the evidence.
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("one JSON coverage report");
    assert_eq!(report["prompts"].as_array().map(Vec::len), Some(16));
    assert_eq!(report["expectation_mismatches"], 0);
    assert_eq!(report["failures"].as_array().map(Vec::len), Some(0));
    let drift = report["baseline_drift"].as_array().expect("drift array");
    assert!(
        drift.is_empty(),
        "baseline drift:\n{}",
        serde_json::to_string_pretty(drift).unwrap_or_default()
    );
    // The dispositions the smoke set exercises, as an EXACT set rather
    // than a floor.
    //
    // It used to be all five, and the fifth was `missing`, carried by
    // `bio-069` (yoghurt). That row stopped being `missing` when
    // `whole_milk` resolved its diffusible mineral buffer: milk became a
    // characterised solution, so the pH meter at the end of that script
    // succeeds instead of refusing. See the fourteenth refresh note in
    // tests/coverage/curiosity-v1/README.md — the row is NOT thereby
    // answered, and the note says so at length.
    //
    // No replacement exists. Two `missing` rows are left in the whole
    // corpus, `aq-053` and `aq-085`, and BOTH carry `expected =
    // "computed"` — so putting either into the smoke set would trip the
    // `expectation_mismatches == 0` assertion above, which is the
    // assertion that says the smoke set holds no open gaps. A `missing`
    // row that could sit here would have to be one the corpus does not
    // expect to compute, and there is none.
    //
    // Making the set EXACT is what keeps this a gate rather than a
    // weakened one: a smoke row that falls to `missing` now fails here,
    // where under "at least one of each" it would have passed as long as
    // some other row was still missing. `missing` itself stays covered by
    // the full `coverage curiosity --check`, which runs in CI beside this
    // and holds `aq-053` and `aq-085`.
    let exercised: BTreeSet<&str> = report["by_observed"]
        .as_object()
        .expect("the report counts its dispositions")
        .iter()
        .filter(|(_, count)| count.as_u64().is_some_and(|count| count > 0))
        .map(|(disposition, _)| disposition.as_str())
        .collect();
    assert_eq!(
        exercised,
        BTreeSet::from(["boundary", "computed", "curated", "qualitative"]),
        "the smoke set's dispositions moved"
    );

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
fn curiosity_check_rejects_a_synthetic_routing_regression() {
    let baseline = include_str!("../../../tests/coverage/curiosity-v1/baseline.toml");
    let original = "id = \"aq-003\"\nowning_task = \"EXP-17\"\noutcome = \"computed\"";
    let changed = "id = \"aq-003\"\nowning_task = \"EXP-17\"\noutcome = \"missing\"";
    assert!(baseline.contains(original));
    let path = std::env::temp_dir().join(format!(
        "kerotakis-curiosity-regression-{}.toml",
        std::process::id()
    ));
    std::fs::write(&path, baseline.replacen(original, changed, 1)).expect("write test baseline");

    let output = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args([
            "coverage",
            "curiosity",
            "--smoke",
            "--check",
            "--json",
            "--baseline",
            path.to_str().expect("UTF-8 temp path"),
        ])
        .output()
        .expect("run regressed coverage report");
    std::fs::remove_file(&path).expect("remove test baseline");

    assert!(!output.status.success(), "regression must fail --check");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON report");
    let drift = report["baseline_drift"].as_array().expect("drift array");
    assert_eq!(
        drift.len(),
        1,
        "one injected regression, one drift line — got:\n{}",
        serde_json::to_string_pretty(drift).unwrap_or_default()
    );
    assert_eq!(drift[0]["id"], "aq-003");
    assert_eq!(drift[0]["kind"], "changed");
}

#[test]
#[ignore = "scheduled native curiosity corpus"]
fn curiosity_full_reports_every_prompt_or_failure() {
    let output = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args(["coverage", "curiosity", "--check", "--json"])
        .output()
        .expect("run full curiosity corpus");
    assert!(output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON report");
    let classified = report["prompts"].as_array().map(Vec::len).unwrap_or(0);
    let failed = report["failures"].as_array().map(Vec::len).unwrap_or(0);
    assert_eq!(classified + failed, 500);
    assert_eq!(failed, 7, "known failures are pinned by the baseline");
    assert_eq!(report["baseline_drift"].as_array().map(Vec::len), Some(0));
}
