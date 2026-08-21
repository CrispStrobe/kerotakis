use std::io::Write;
use std::process::Command;

const MECHANISM: &str = r#"
description: CLI elementary mechanism
units: {length: cm, quantity: mol, activation-energy: kJ/mol}
phases:
- name: gas
  thermo: ideal-gas
  species: [H2, O2, H2O]
species:
- name: H2
  composition: {H: 2}
- name: O2
  composition: {O: 2}
- name: H2O
  composition: {H: 2, O: 1}
reactions:
- equation: 2 H2 + O2 => 2 H2O
  rate-constant: {A: 1.0e12, b: 0.5, Ea: 41.84}
"#;

const SIMULATION_MECHANISM: &str = r#"
description: CLI gas simulation
phases:
- name: gas
  thermo: ideal-gas
  species: [H2, H]
species:
- name: H2
  composition: {H: 2}
- name: H
  composition: {H: 1}
reactions:
- equation: H2 => 2 H
  rate-constant: {A: 1.0, b: 0, Ea: 0}
"#;

const REVERSIBLE_MECHANISM: &str = r#"
description: CLI reversible simulation
phases:
- name: gas
  thermo: ideal-gas
  species: [A, B]
species:
- name: A
  composition: {X: 1}
  thermo:
    model: NASA7
    temperature-ranges: [200.0, 3000.0]
    data: [[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]]
- name: B
  composition: {X: 1}
  thermo:
    model: NASA7
    temperature-ranges: [200.0, 3000.0]
    data: [[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.3862943611198906]]
reactions:
- equation: A <=> B
  rate-constant: {A: 1.0, b: 0, Ea: 0}
"#;

const MULTISTEP_MECHANISM: &str = r#"
description: CLI multistep rates
phases:
- name: gas
  thermo: ideal-gas
  species: [A, B, C]
species:
- name: A
  composition: {X: 1}
- name: B
  composition: {X: 1}
- name: C
  composition: {X: 1}
reactions:
- equation: A => B
  rate-constant: {A: 1.0, b: 0, Ea: 0}
- equation: B => C
  rate-constant: {A: 0.25, b: 0, Ea: 0}
"#;

fn mechanism_file(name: &str, contents: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("kero-mechanism-{}-{}", std::process::id(), name));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("mechanism.yaml");
    let mut file = std::fs::File::create(&path).unwrap();
    file.write_all(contents.as_bytes()).unwrap();
    path
}

#[test]
fn mechanism_inspect_json_is_machine_readable_and_normalized() {
    let path = mechanism_file("valid", MECHANISM);
    let output = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args(["mechanism", "inspect", path.to_str().unwrap(), "--json"])
        .output()
        .expect("kero runs");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["name"], "CLI elementary mechanism");
    assert_eq!(value["species"], 3);
    assert_eq!(value["reactions"], 1);
    assert_eq!(value["reaction_details"][0]["total_order"], 3.0);
    assert_eq!(value["reaction_details"][0]["rate_model"], "elementary");
    assert_eq!(value["reaction_details"][0]["reversible"], false);
    assert!(value["reaction_details"][0]["low_pressure_pre_exponential"].is_null());
    assert_eq!(
        value["reaction_details"][0]["activation_energy_j_per_mol"],
        41_840.0
    );
    // mol/cm³ -> mol/L and a third-order rate gives a C^-2 conversion.
    assert_eq!(value["reaction_details"][0]["pre_exponential"], 1.0e6);
    std::fs::remove_dir_all(path.parent().unwrap()).ok();
}

#[test]
fn mechanism_inspect_reports_troe_falloff_parameters() {
    let mechanism = MECHANISM.replace(
        "- equation: 2 H2 + O2 => 2 H2O\n  rate-constant: {A: 1.0e12, b: 0.5, Ea: 41.84}",
        "- equation: 2 H2 + O2 (+M) => 2 H2O (+M)\n  type: falloff\n  high-P-rate-constant: {A: 1.0e12, b: 0.5, Ea: 41.84}\n  low-P-rate-constant: {A: 1.0e15, b: 0.5, Ea: 41.84}\n  efficiencies: {H2O: 4.0}\n  Troe: {A: 0.5, T3: 1000.0, T1: 10000.0}",
    );
    let path = mechanism_file("troe", &mechanism);
    let output = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args(["mechanism", "inspect", path.to_str().unwrap(), "--json"])
        .output()
        .expect("kero runs");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["reaction_details"][0]["rate_model"], "troe");
    assert_eq!(
        value["reaction_details"][0]["low_pressure_pre_exponential"],
        1.0e6
    );
    std::fs::remove_dir_all(path.parent().unwrap()).ok();
}

#[test]
fn mechanism_inspect_rejects_unbalanced_input_with_reaction_number() {
    let path = mechanism_file("unbalanced", &MECHANISM.replace("2 H2O", "H2O"));
    let output = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args(["mechanism", "inspect", path.to_str().unwrap()])
        .output()
        .expect("kero runs");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("reaction 1"), "{stderr}");
    assert!(stderr.contains("element H"), "{stderr}");
    std::fs::remove_dir_all(path.parent().unwrap()).ok();
}

#[test]
fn mechanism_simulate_advances_a_finite_gas_reactor_as_json() {
    let path = mechanism_file("simulate", SIMULATION_MECHANISM);
    let output = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args([
            "mechanism",
            "simulate",
            path.to_str().unwrap(),
            "--seconds",
            "0.1",
            "--volume-l",
            "1.0",
            "--temperature-k",
            "300.0",
            "--feed",
            "H2=1.0",
            "--samples",
            "4",
            "--json",
        ])
        .output()
        .expect("kero runs");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["mechanism"], "CLI gas simulation");
    assert_eq!(value["duration_seconds"], 0.1);
    assert_eq!(value["volume_litres"], 1.0);
    assert_eq!(value["temperature_k"], 300.0);
    assert_eq!(value["sample_intervals"], 4);
    assert_eq!(value["samples"].as_array().unwrap().len(), 5);
    for (index, sample) in value["samples"].as_array().unwrap().iter().enumerate() {
        let elapsed = index as f64 * 0.025;
        assert!((sample["elapsed_seconds"].as_f64().unwrap() - elapsed).abs() < 1e-14);
        let h2 = sample["moles"][0]["moles"].as_f64().unwrap();
        assert!((h2 - (-elapsed).exp()).abs() < 1e-6, "{sample}");
        if index > 0 {
            let previous = value["samples"][index - 1]["moles"][0]["moles"]
                .as_f64()
                .unwrap();
            assert!(h2 < previous);
        }
    }
    let remaining = value["final_moles"][0]["moles"].as_f64().unwrap();
    let product = value["final_moles"][1]["moles"].as_f64().unwrap();
    let expected_remaining = (-0.1f64).exp();
    assert!((remaining - expected_remaining).abs() < 1e-6, "{value}");
    assert!((product - 2.0 * (1.0 - expected_remaining)).abs() < 2e-6);
    assert!(
        value["final_pressure_pa"].as_f64().unwrap()
            > value["initial_pressure_pa"].as_f64().unwrap()
    );
    assert_eq!(value["extents"][0]["reaction"], "reaction-1");
    assert!(value["statistics"]["accepted_steps"].as_u64().unwrap() > 0);
    std::fs::remove_dir_all(path.parent().unwrap()).ok();
}

#[test]
fn mechanism_simulate_reversible_gas_approaches_thermodynamic_equilibrium() {
    let path = mechanism_file("reversible", REVERSIBLE_MECHANISM);
    let output = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args([
            "mechanism",
            "simulate",
            path.to_str().unwrap(),
            "--seconds",
            "20",
            "--volume-l",
            "1",
            "--temperature-k",
            "500",
            "--feed",
            "A=1",
            "--samples",
            "4",
            "--json",
        ])
        .output()
        .expect("kero runs");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let a = value["final_moles"][0]["moles"].as_f64().unwrap();
    let b = value["final_moles"][1]["moles"].as_f64().unwrap();
    assert!((a - 0.2).abs() < 1e-6, "{value}");
    assert!((b - 0.8).abs() < 1e-6, "{value}");
    assert!((b / a - 4.0).abs() < 1e-5, "{value}");
    std::fs::remove_dir_all(path.parent().unwrap()).ok();
}

#[test]
fn mechanism_rates_reports_progress_species_production_and_limiting_step() {
    let path = mechanism_file("multistep-rates", MULTISTEP_MECHANISM);
    let output = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args([
            "mechanism",
            "rates",
            path.to_str().unwrap(),
            "--volume-l",
            "1",
            "--temperature-k",
            "500",
            "--feed",
            "A=1",
            "--feed",
            "B=1",
            "--json",
        ])
        .output()
        .expect("kero runs");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["mechanism"], "CLI multistep rates");
    assert_eq!(value["reaction_rates"][0]["reaction"], "reaction-1");
    assert_eq!(
        value["reaction_rates"][0]["forward_moles_per_litre_second"],
        1.0
    );
    assert_eq!(
        value["reaction_rates"][0]["reverse_moles_per_litre_second"],
        0.0
    );
    assert_eq!(
        value["reaction_rates"][1]["net_moles_per_litre_second"],
        0.25
    );
    assert_eq!(
        value["species_rates"][0]["net_production_moles_per_litre_second"],
        -1.0
    );
    assert_eq!(
        value["species_rates"][1]["net_production_moles_per_litre_second"],
        0.75
    );
    assert_eq!(
        value["species_rates"][2]["net_production_moles_per_litre_second"],
        0.25
    );
    assert_eq!(value["rate_determining_step"]["reaction"], "reaction-2");
    assert!(value["rate_determining_criterion"]
        .as_str()
        .unwrap()
        .contains("smallest non-zero absolute net progress"));
    std::fs::remove_dir_all(path.parent().unwrap()).ok();
}

#[test]
fn mechanism_rates_exposes_balanced_forward_and_reverse_flux_at_equilibrium() {
    let path = mechanism_file("equilibrium-rates", REVERSIBLE_MECHANISM);
    let output = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args([
            "mechanism",
            "rates",
            path.to_str().unwrap(),
            "--volume-l",
            "1",
            "--temperature-k",
            "500",
            "--feed",
            "A=0.2",
            "--feed",
            "B=0.8",
            "--json",
        ])
        .output()
        .expect("kero runs");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let rates = &value["reaction_rates"][0];
    assert_eq!(rates["forward_moles_per_litre_second"], 0.2);
    assert!((rates["reverse_moles_per_litre_second"].as_f64().unwrap() - 0.2).abs() < 1e-14);
    assert!(rates["net_moles_per_litre_second"].as_f64().unwrap().abs() < 1e-14);
    assert!(value["rate_determining_step"].is_null());
    std::fs::remove_dir_all(path.parent().unwrap()).ok();
}

#[test]
fn mechanism_simulate_rejects_zero_sample_intervals() {
    let path = mechanism_file("zero-samples", SIMULATION_MECHANISM);
    let output = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args([
            "mechanism",
            "simulate",
            path.to_str().unwrap(),
            "--seconds",
            "1",
            "--volume-l",
            "1",
            "--temperature-k",
            "300",
            "--feed",
            "H2=1",
            "--samples",
            "0",
        ])
        .output()
        .expect("kero runs");
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("--samples must be positive"), "{stderr}");
    std::fs::remove_dir_all(path.parent().unwrap()).ok();
}

#[test]
fn mechanism_simulate_rejects_an_unknown_feed_species() {
    let path = mechanism_file("unknown-feed", SIMULATION_MECHANISM);
    let output = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args([
            "mechanism",
            "simulate",
            path.to_str().unwrap(),
            "--seconds",
            "1",
            "--volume-l",
            "1",
            "--temperature-k",
            "300",
            "--feed",
            "NOPE=1",
        ])
        .output()
        .expect("kero runs");
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("feed species 'NOPE' is not declared"),
        "{stderr}"
    );
    std::fs::remove_dir_all(path.parent().unwrap()).ok();
}
