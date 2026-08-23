//! The finite-headspace state and events are part of the client-neutral JSON contract.

use std::io::Write;
use std::process::Command;

#[test]
fn every_gas_boundary_round_trips_through_the_cli_contract() {
    let dir = std::env::temp_dir().join(format!("kero-headspace-json-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let lab = dir.join("headspace.lab");
    let mut file = std::fs::File::create(&lab).unwrap();
    writeln!(
        file,
        "seal v1 500mL\nopen v1\nregulate v1 1.5bar 250mL\nsweep v1 90kPa\nopen v1\nadd v1 water 1L\nadd v1 Ca(OH)2 0.01mol\nadd v1 CO2 0.01mol"
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args(["run", lab.to_str().unwrap(), "--json"])
        .output()
        .expect("kero runs");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let steps: Vec<serde_json::Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(steps.len(), 8);
    assert_eq!(steps[0]["operator"]["op"], "seal");
    assert_eq!(
        steps[0]["bench"]["vessels"][0]["headspace"]["boundary"],
        "sealed"
    );
    assert!(
        steps[0]["bench"]["vessels"][0]["pressure"]
            .as_f64()
            .unwrap()
            > 100_000.0
    );
    assert_eq!(steps[1]["operator"]["op"], "open");
    assert_eq!(
        steps[1]["bench"]["vessels"][0]["headspace"]["boundary"],
        "open"
    );
    assert!(steps[1]["events"]
        .as_array()
        .unwrap()
        .iter()
        .any(|event| event["event"] == "gas_evolved"));
    assert_eq!(steps[2]["operator"]["op"], "regulate");
    assert_eq!(
        steps[2]["bench"]["vessels"][0]["headspace"]["boundary"],
        "pressure_controlled"
    );
    assert_eq!(steps[2]["bench"]["vessels"][0]["pressure"], 150_000.0);
    assert_eq!(steps[3]["operator"]["op"], "sweep");
    assert_eq!(
        steps[3]["bench"]["vessels"][0]["headspace"]["boundary"],
        "swept"
    );
    assert_eq!(steps[3]["bench"]["vessels"][0]["pressure"], 90_000.0);
    assert!(steps[7]["events"]
        .as_array()
        .unwrap()
        .iter()
        .any(|event| event["event"] == "gas_absorbed" && event["species"] == "CO2"));
    assert!(
        steps[7]["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event"] == "precipitated" && event["species"] == "CaCO3"),
        "limewater result had no calcite precipitation: events={}, vessel={}",
        steps[7]["events"],
        steps[7]["bench"]["vessels"][0]
    );

    std::fs::remove_dir_all(dir).ok();
}
