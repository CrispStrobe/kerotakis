//! CAP-8's acceptance: the titration-endpoint distribution under a
//! stated burette-side uncertainty reproduces the analytic expectation
//! of the linear case — endpoint volume = acid moles / concentration,
//! so an input σ of 1 % passes through unchanged — and two runs with
//! the same seed are byte-identical.

use std::process::Command;

fn lab_path() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join("kerotakis-study-mc");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("fine-titration.lab");
    std::fs::write(
        &path,
        "# Fine-step titration for the MC endpoint distribution\n\
         add v1 water 500mL\n\
         add v1 HCl 0.01mol\n\
         titrate v1 NaOH 1M 0.1mL until ph 7 max 400\n",
    )
    .unwrap();
    path
}

fn run_mc(extra: &[&str]) -> std::process::Output {
    let lab = lab_path();
    let mut args = vec![
        "study".to_string(),
        lab.to_string_lossy().into_owned(),
        "--vary".into(),
        "add:v1:HCl=normal(0.01,0.0001)".into(),
        "--collect".into(),
        "titrant_volume@v1".into(),
    ];
    args.extend(extra.iter().map(|s| s.to_string()));
    Command::new(env!("CARGO_BIN_EXE_kero"))
        .args(&args)
        .output()
        .expect("kero study runs")
}

#[test]
fn endpoint_distribution_matches_the_analytic_expectation() {
    let out = run_mc(&["--mc", "100", "--seed", "42"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    let last = stdout.lines().last().expect("a summary line");
    let summary: serde_json::Value = serde_json::from_str(last).unwrap();
    let s = &summary["summary"]["titrant_volume@v1"];
    let (mean, sd) = (s["mean"].as_f64().unwrap(), s["sd"].as_f64().unwrap());
    let (p5, p50, p95) = (
        s["p5"].as_f64().unwrap(),
        s["p50"].as_f64().unwrap(),
        s["p95"].as_f64().unwrap(),
    );
    // Linear case: endpoint volume = acid/1 M. Analytically the mean is
    // equivalence plus the mean crossing overshoot (half a 0.1 mL step),
    // the σ is the input σ (1e-4 L) with a little step-quantisation on
    // top, and p95−p5 is 2·1.645·σ snapped to the 0.1 mL grid.
    assert!(
        (mean - 0.01005).abs() < 0.0002,
        "mean endpoint {mean} vs analytic ≈0.01005 L"
    );
    assert!(
        (0.8e-4..=1.3e-4).contains(&sd),
        "σ {sd} should carry the 1e-4 input through the linear map"
    );
    assert!(
        (p50 - 0.0100).abs() <= 0.0002,
        "median endpoint {p50} near equivalence"
    );
    let spread = p95 - p5;
    assert!(
        (2.2e-4..=4.4e-4).contains(&spread),
        "p95−p5 {spread} vs analytic 3.29e-4 on a 1e-4 grid"
    );
    assert_eq!(s["n"], 100, "every run produced an endpoint");
}

#[test]
fn same_seed_same_bytes() {
    let a = run_mc(&["--mc", "40", "--seed", "7"]);
    let b = run_mc(&["--mc", "40", "--seed", "7"]);
    assert!(a.status.success() && b.status.success());
    assert_eq!(a.stdout, b.stdout, "determinism is the contract");
    let c = run_mc(&["--mc", "40", "--seed", "8"]);
    assert_ne!(
        a.stdout, c.stdout,
        "a different seed draws different samples"
    );
}

#[test]
fn the_flag_contract_is_enforced_out_loud() {
    // A distribution without --mc refuses.
    let out = run_mc(&[]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("--mc"));
    // --mc without a seed refuses: the seed is spoken, never invented.
    let out = run_mc(&["--mc", "10"]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("--seed"));
    // --mc over a linear range refuses and names the fix.
    let lab = lab_path();
    let out = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args([
            "study",
            &lab.to_string_lossy(),
            "--vary",
            "add:v1:HCl=0.005..0.02:4",
            "--collect",
            "ph@v1",
            "--mc",
            "10",
            "--seed",
            "1",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("normal"));
}
