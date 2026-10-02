//! Actual host stack must preserve finite boundaries and price combustion once.
use serde_json::Value;
use std::process::Command;
fn run(name: &str, script: &str) -> Vec<Value> {
    let path = std::env::temp_dir().join(format!("kero-thermal-{}-{name}.lab", std::process::id()));
    std::fs::write(&path, script).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args(["run", path.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    std::fs::remove_file(path).unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let steps: Vec<Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    for step in &steps {
        assert!(
            !step["events"]
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e["event"] == "solver_failed"),
            "{step}"
        );
    }
    steps
}
#[test]
fn open_combustion_does_not_price_its_gas_products_again_on_water_addition() {
    let steps = run(
        "once",
        "add v1 ethanol 1g\nignite v1\nadd v1 water 100mL\ninspect v1\n",
    );
    let ignition = &steps[1];
    let temperature = ignition["bench"]["vessels"][0]["temperature"]
        .as_f64()
        .unwrap();
    assert!((1800.0..2400.0).contains(&temperature), "{ignition}");
    assert!(ignition["events"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["event"] == "thermal_equilibrium"));
    let final_temperature = steps.last().unwrap()["bench"]["vessels"][0]["temperature"]
        .as_f64()
        .unwrap();
    assert!(
        (final_temperature - 298.15).abs() < 0.1,
        "{final_temperature}"
    );
}
#[test]
fn finite_flame_keeps_products_and_closes_pressure_on_the_actual_cli() {
    let steps = run(
        "sealed",
        "seal v1 1L\nadd v1 H2 0.003mol\nadd v1 O2 0.002mol\nignite v1\ninspect v1\n",
    );
    let before = &steps[2]["bench"]["vessels"][0];
    let ignition = &steps[3];
    let after = &ignition["bench"]["vessels"][0];
    let events = ignition["events"].as_array().unwrap();
    assert!(
        events.iter().any(|e| e["event"] == "thermal_equilibrium"),
        "{ignition}"
    );
    assert!(
        !events
            .iter()
            .any(|e| e["event"] == "gas_absorbed" || e["event"] == "gas_evolved"),
        "{ignition}"
    );
    let contents = after["contents"].as_array().unwrap();
    let water: f64 = contents
        .iter()
        .filter(|p| p["species"] == "water")
        .map(|p| p["moles"].as_f64().unwrap())
        .sum();
    assert!(water > 0.0029, "{after}");
    let n: f64 = contents.iter().map(|p| p["moles"].as_f64().unwrap()).sum();
    let temperature = after["temperature"].as_f64().unwrap();
    assert!(temperature > 1000.0);
    let volume = after["headspace"]["volume"].as_f64().unwrap();
    assert_eq!(volume, before["headspace"]["volume"].as_f64().unwrap());
    let expected = n * 8314.462618 * temperature / volume;
    assert!((after["pressure"].as_f64().unwrap() / expected - 1.0).abs() < 1e-8);
}

#[test]
fn a_finite_mole_input_that_overflows_heat_capacity_is_an_atomic_cli_refusal() {
    let path = std::env::temp_dir().join(format!("kero-overflow-{}.lab", std::process::id()));
    std::fs::write(&path, "add v1 water 8e306mol\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args(["run", path.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    std::fs::remove_file(path).unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains("numeric domain"), "{error}");
    assert!(!error.contains("InvalidState {"), "{error}");
}

#[test]
fn a_hot_sealed_flame_bursts_and_accounts_for_the_released_products() {
    let steps = run(
        "burst",
        "seal v1 1L\nadd v1 H2 0.02mol\nadd v1 O2 0.015mol\nignite v1\ninspect v1\n",
    );
    let ignition = &steps[3];
    let events = ignition["events"].as_array().unwrap();
    let burst = events.iter().find(|e| e["event"] == "burst").unwrap();
    assert!(burst["at_pa"].as_f64().unwrap() > burst["rating_pa"].as_f64().unwrap());
    let vessel = &ignition["bench"]["vessels"][0];
    assert_eq!(vessel["headspace"]["boundary"], "open");
    assert!(vessel["contents"].as_array().unwrap().is_empty());
    let amount = |kind: &str| {
        events
            .iter()
            .filter(|e| e["event"] == kind && e["species"] == "water")
            .map(|e| e["moles"].as_f64().unwrap())
            .sum::<f64>()
    };
    assert!(amount("gas_contained") > 0.019);
    assert!((amount("gas_contained") - amount("gas_evolved")).abs() < 1e-12);
}

#[test]
fn a_pressure_controlled_flame_retains_products_and_expands() {
    let steps = run(
        "piston",
        "regulate v1 1atm 1L\nadd v1 H2 0.02mol\nadd v1 O2 0.015mol\nignite v1\ninspect v1\n",
    );
    let before = &steps[2]["bench"]["vessels"][0];
    let ignition = &steps[3];
    let after = &ignition["bench"]["vessels"][0];
    assert_eq!(after["headspace"]["boundary"], "pressure_controlled");
    assert!((after["pressure"].as_f64().unwrap() - 101325.0).abs() < 1e-6);
    assert!(
        after["headspace"]["volume"].as_f64().unwrap()
            > before["headspace"]["volume"].as_f64().unwrap()
    );
    let events = ignition["events"].as_array().unwrap();
    assert!(!events.iter().any(|e| matches!(
        e["event"].as_str(),
        Some("burst" | "gas_absorbed" | "gas_evolved")
    )));
    let contents = after["contents"].as_array().unwrap();
    let water: f64 = contents
        .iter()
        .filter(|p| p["species"] == "water")
        .map(|p| p["moles"].as_f64().unwrap())
        .sum();
    assert!(water > 0.019);
    let n: f64 = contents.iter().map(|p| p["moles"].as_f64().unwrap()).sum();
    let expected = n * 8314.462618 * after["temperature"].as_f64().unwrap()
        / after["headspace"]["volume"].as_f64().unwrap();
    assert!((after["pressure"].as_f64().unwrap() / expected - 1.0).abs() < 1e-8);
}
