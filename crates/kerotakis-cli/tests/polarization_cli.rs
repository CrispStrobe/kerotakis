use std::process::Command;

#[test]
fn predicts_from_explicit_columns_area_and_reference() {
    let root = env!("CARGO_MANIFEST_DIR");
    let output = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args([
            "polarization",
            "predict",
            "--model",
            &format!("{root}/tests/fixtures/polarization-model.json"),
            "--data",
            &format!("{root}/tests/fixtures/polarization-curve.csv"),
            "--potential-column",
            "instrument potential",
            "--current-column",
            "current",
            "--area-m2",
            "0.0002",
            "--current-sign",
            "anodic-positive",
            "--reference-offset-v",
            "0.210",
            "--reference-label",
            "declared test reference",
        ])
        .output()
        .expect("run kero polarization predict");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let predictions = json["predictions"].as_array().unwrap();
    assert_eq!(predictions.len(), 3);
    assert!((predictions[0]["potential_she_v"].as_f64().unwrap() + 0.5).abs() < 1e-12);
    assert!(
        (predictions[0]["total_current_density_a_per_m2"]
            .as_f64()
            .unwrap()
            - 0.2 / 10f64.powf(0.1 / 0.12))
        .abs()
            < 1e-12
    );
}

#[test]
fn reads_whitespace_export_and_derives_scan_rate() {
    let root = env!("CARGO_MANIFEST_DIR");
    let output = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args([
            "polarization",
            "predict",
            "--model",
            &format!("{root}/tests/fixtures/polarization-model.json"),
            "--data",
            &format!("{root}/tests/fixtures/polarization-curve.txt"),
            "--format",
            "whitespace",
            "--skip-lines",
            "2",
            "--header-line",
            "1",
            "--potential-header-token",
            "potential",
            "--current-header-token",
            "current",
            "--potential-index",
            "2",
            "--current-index",
            "3",
            "--time-index",
            "1",
            "--derive-sweep-rate",
            "--allow-time-segments",
            "--min-abs-sweep-rate",
            "0.0005",
            "--max-abs-sweep-rate",
            "0.002",
            "--area-m2",
            "0.0002",
            "--current-sign",
            "anodic-positive",
            "--reference-offset-v",
            "0.210",
            "--reference-label",
            "declared test reference",
        ])
        .output()
        .expect("run kero polarization predict");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["observations"].as_array().unwrap().len(), 4);
    assert!(
        (json["observations"][1]["sweep_rate_v_per_s"]
            .as_f64()
            .unwrap()
            - 0.001)
            .abs()
            < 1e-12
    );
}

#[test]
fn refuses_a_mismatched_whitespace_schema() {
    let root = env!("CARGO_MANIFEST_DIR");
    let output = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args([
            "polarization",
            "predict",
            "--model",
            &format!("{root}/tests/fixtures/polarization-model.json"),
            "--data",
            &format!("{root}/tests/fixtures/polarization-curve.txt"),
            "--format",
            "whitespace",
            "--skip-lines",
            "2",
            "--header-line",
            "1",
            "--potential-header-token",
            "frequency",
            "--current-header-token",
            "current",
            "--potential-index",
            "2",
            "--current-index",
            "3",
            "--area-m2",
            "0.0002",
            "--current-sign",
            "anodic-positive",
            "--reference",
            "she",
        ])
        .output()
        .expect("run kero polarization predict");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("declared potential index 2 does not select header token 'frequency'"));
}
