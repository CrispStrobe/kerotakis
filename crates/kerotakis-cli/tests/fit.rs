//! CAP-9's public contract: fit one curated kinetic constant by replaying a
//! lesson against learner measurements.  These are deliberately binary-level
//! tests: a fit disconnected from the bench's kinetics cannot satisfy the
//! synthetic-data case and the reporting assertions together.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static CASE: AtomicUsize = AtomicUsize::new(0);

struct Case {
    dir: PathBuf,
}

impl Case {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "kero-fit-{}-{}",
            std::process::id(),
            CASE.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        Self { dir }
    }

    fn write(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.dir.join(name);
        std::fs::write(&path, contents).unwrap();
        path
    }

    fn fit(&self, lab: &Path, csv: &Path, extra: &[&str]) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_kero"));
        command
            .arg("fit")
            .arg(lab)
            .args(["--param", "rate:peroxide-decomposition:pre_exponential"])
            .args(["--observe", "amount:H2O2@v1"])
            .arg("--data")
            .arg(csv)
            .args(["--bounds", "2e7..2e8"])
            .args(extra)
            .output()
            .expect("kero fit runs")
    }
}

impl Drop for Case {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.dir).ok();
    }
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn refuses_finite_observations_whose_squared_residuals_overflow() {
    let case = Case::new();
    let lab = case.write("peroxide.lab", "add v1 water 100mL\nadd v1 H2O2 0.01mol\n");
    let csv = case.write("measurements.csv", "t,observation\n0,1e308\n1,1e308\n");
    let output = case.fit(&lab, &csv, &["--loss", "sse"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("squared residuals overflow"));
}

#[test]
fn replay_recovers_a_known_rate_and_reports_residuals_and_curation() {
    let case = Case::new();
    // Uncatalysed first-order peroxide is intentionally used here: its
    // independent oracle is n(t)=n0 exp(-2 A exp(-Ea/RT)t), while `fit` must
    // still obtain every prediction by replaying this lesson through WAIT.
    // A_true=8.4e7 s^-1 (1.5 times the curated value), T=298.15 K after the
    // aqueous solver establishes its reference-temperature solution, and
    // Ea=75 kJ/mol.  The alternating sub-micromole offsets are deterministic
    // measurement noise rather than values copied from the optimiser.
    let lab = case.write("peroxide.lab", "add v1 water 100mL\nadd v1 H2O2 0.01mol\n");
    let csv = case.write(
        "measurements.csv",
        "t,observation\n\
         0,0.01000000\n\
         10000,0.00885308\n\
         20000,0.00783669\n\
         30000,0.00693804\n\
         40000,0.00614175\n\
         50000,0.00543719\n",
    );
    let output = case.fit(&lab, &csv, &["--loss", "sse"]);
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        stderr(&output)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "fit output is one JSON report: {e}: {}",
            String::from_utf8_lossy(&output.stdout)
        )
    });

    let fitted = report["parameter"]["fitted_value"]
        .as_f64()
        .expect("numeric fitted value");
    let known = 8.4e7;
    assert!(
        ((fitted - known) / known).abs() < 0.03,
        "expected the synthetic A={known:.6e} within 3%, got {fitted:.6e}: {report}"
    );
    assert_eq!(
        report["parameter"]["selector"],
        "rate:peroxide-decomposition:pre_exponential"
    );
    assert_eq!(report["parameter"]["curated_value"], 5.6e7);
    assert_eq!(report["parameter"]["unit"], "rate-law dependent");
    assert!(
        report["parameter"]["provenance"]
            .as_str()
            .is_some_and(|p| p.contains("Editorial judgement") && p.contains("not measured")),
        "the curated comparison needs its honest provenance: {report}"
    );
    assert!(
        report["parameter"]["source_ids"]
            .as_array()
            .is_some_and(|ids| ids
                .iter()
                .any(|id| id == "kerotakis:kinetics:peroxide-decomposition")),
        "curated source id missing: {report}"
    );
    assert_eq!(report["loss"]["name"], "sse");
    assert_eq!(report["loss"]["n"], 6);

    let chart = &report["chart"];
    assert!(
        chart["title"]
            .as_str()
            .is_some_and(|s| s.to_ascii_lowercase().contains("residual")),
        "residual chart title: {chart}"
    );
    assert_eq!(chart["x"]["unit"], "s");
    assert_eq!(chart["y"]["unit"], "mol");
    assert!(
        chart["y"]["label"]
            .as_str()
            .is_some_and(|label| label.contains("observed − predicted")),
        "the residual sign must be explicit: {chart}"
    );
    let points = chart["series"][0]["points"]
        .as_array()
        .expect("residual scatter points");
    assert_eq!(points.len(), 6);
    assert_eq!(points[0][0], 0.0);
    assert!(
        chart["provenance"]
            .as_str()
            .is_some_and(|p| p.contains("fresh replay") && p.contains("measurements")),
        "chart must state the model and measurement origin: {chart}"
    );
}

#[test]
fn malformed_measurements_and_unsupported_options_are_refused() {
    let case = Case::new();
    let lab = case.write("experiment.lab", "add v1 H2O2 0.01mol\n");

    for (name, csv, expected) in [
        ("header.csv", "seconds,value\n0,0.01\n", "t,observation"),
        ("negative.csv", "t,observation\n-1,0.01\n", "nonnegative"),
        ("nan.csv", "t,observation\n1,NaN\n", "finite"),
    ] {
        let csv = case.write(name, csv);
        let output = case.fit(&lab, &csv, &["--loss", "sse"]);
        assert!(!output.status.success(), "{name} unexpectedly passed");
        assert!(
            stderr(&output).contains(expected),
            "{name}: expected {expected:?} in diagnostic, got {}",
            stderr(&output)
        );
    }

    let valid = case.write("valid.csv", "t,observation\n0,0.01\n1,0.00999\n");
    let timed_lab = case.write("already-timed.lab", "add v1 H2O2 0.01mol\nwait 1s\n");
    let output = case.fit(&timed_lab, &valid, &["--loss", "sse"]);
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("must not contain wait"),
        "{}",
        stderr(&output)
    );

    let output = case.fit(&lab, &valid, &["--loss", "absolute"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("sse"), "{}", stderr(&output));

    let output = Command::new(env!("CARGO_BIN_EXE_kero"))
        .arg("fit")
        .arg(&lab)
        .args(["--param", "rate:no-such-reaction:pre_exponential"])
        .args(["--observe", "amount:H2O2@v1"])
        .args(["--bounds", "2e7..2e8"])
        .arg("--data")
        .arg(&valid)
        .args(["--loss", "sse"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("no-such-reaction"),
        "{}",
        stderr(&output)
    );

    let output = Command::new(env!("CARGO_BIN_EXE_kero"))
        .arg("fit")
        .arg(&lab)
        .args(["--param", "amount:v1:H2O2"])
        .args(["--observe", "amount:H2O2@v1"])
        .args(["--bounds", "2e7..2e8"])
        .arg("--data")
        .arg(&valid)
        .args(["--loss", "sse"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(stderr(&output).contains("selector"), "{}", stderr(&output));
}
