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
    assert_eq!(
        value["reaction_details"][0]["activation_energy_j_per_mol"],
        41_840.0
    );
    // mol/cm³ -> mol/L and a third-order rate gives a C^-2 conversion.
    assert_eq!(value["reaction_details"][0]["pre_exponential"], 1.0e6);
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
