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
    assert_eq!(report["baseline_drift"].as_array().map(Vec::len), Some(0));
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
    assert_eq!(drift.len(), 1);
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

/// A row that answers its question AND names a limitation must be counted
/// as answered.
///
/// This is a positive control in the awkward shape, and it exists because
/// the classifier failed it. `mat-096` asks what iron needs in order to
/// rust, and the bench answers with an extent and a rate:
///
///     0.0010 mol reacted in 3600 s  —  4 Fe + 3 O₂ → 2 Fe₂O₃↓
///
/// It also says, honestly, that no wired solver models iron dissolving in
/// liquid. The classifier treated any such note as proof that nothing had
/// been answered, so the row was filed under "the engine stood aside" —
/// on the strength of the caveat, not the absence of an answer.
///
/// That is the worst shape this defect family takes. Every other instance
/// merely fails to notice something; this one **punishes the behaviour the
/// bench exists to have.** A row that answers and then qualifies scores as
/// a row that answered nothing, so the incentive it creates is to delete
/// the caveat — and the rows that survive the filter are selected for
/// silence. (Named as such by crispasr-ba, whose CI watch had the dual
/// bug: it keyed on `status == "completed"` and read four CANCELLED runs
/// as settled. Mine under-accepts an answer, theirs over-accepts a
/// finish; both test a proxy for the property instead of the property.)
///
/// The assertion is deliberately about the shape rather than the row: what
/// must hold is that answering and qualifying beats qualifying alone. If
/// `mat-096` ever stops emitting both kinds of event this test should be
/// re-pointed at another row that does, not deleted.
#[test]
fn a_row_that_answers_and_qualifies_is_not_a_gap() {
    let output = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args(["coverage", "curiosity", "--check", "--json"])
        .output()
        .expect("run kero coverage curiosity");
    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("one JSON coverage report");

    let row = report["prompts"]
        .as_array()
        .expect("prompts")
        .iter()
        .find(|p| p["id"] == "mat-096")
        .expect("mat-096 is in the corpus");

    assert_ne!(
        row["observed"], "missing",
        "mat-096 answers with an extent and a rate and then names a \
         limitation; a caveat is not the absence of an answer. Row: {row}"
    );
}
