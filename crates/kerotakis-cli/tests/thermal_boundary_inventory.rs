//! Replay the original independent sealed gas-loss scripts through the CLI.
use std::collections::BTreeMap;
use std::process::Command;

fn inventory(vessel: &serde_json::Value, gases: bool) -> BTreeMap<String, f64> {
    let mut totals = BTreeMap::new();
    for portion in vessel["contents"].as_array().expect("contents") {
        if gases && portion["phase"] != "gas" {
            continue;
        }
        let amount = portion["moles"].as_f64().expect("finite moles");
        assert!(amount.is_finite() && amount >= 0.0);
        *totals
            .entry(portion["species"].as_str().unwrap().to_string())
            .or_insert(0.0) += amount;
    }
    totals
}

fn vessel(row: &serde_json::Value) -> &serde_json::Value {
    row["bench"]["vessels"]
        .as_array()
        .expect("bench")
        .iter()
        .find(|v| v["id"] == 0)
        .expect("vessel 0")
}

#[test]
fn original_sealed_combustion_scripts_retain_fuel_and_gases_through_observation() {
    for (name, script, fuel) in [
        (
            "magnesium",
            include_str!("fixtures/thermal-boundary/sealed-magnesium.lab"),
            "Mg",
        ),
        (
            "carbon",
            include_str!("fixtures/thermal-boundary/sealed-carbon.lab"),
            "C",
        ),
    ] {
        let dir = std::env::temp_dir().join(format!(
            "kero-thermal-boundary-{}-{name}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("case.lab");
        std::fs::write(&path, script).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kero"))
            .args(["run", path.to_str().unwrap(), "--json"])
            .env("KERO_LANG", "en")
            .output()
            .expect("CLI runs");
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(
            output.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty(), "{name}: unexpected stderr");
        let rows: Vec<serde_json::Value> = String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).expect("JSON row"))
            .collect();
        let ignition = rows
            .iter()
            .position(|r| r["operator"]["op"] == "ignite")
            .expect("ignition");
        assert!(ignition > 0);
        let before = vessel(&rows[ignition - 1]);
        let initial = inventory(before, false);
        let gases = inventory(before, true);
        assert!(initial.get(fuel).copied().unwrap_or(0.0) > 0.0 && !gases.is_empty());
        let events = rows[ignition]["events"].as_array().unwrap();
        assert!(
            events
                .iter()
                .any(|e| e["event"] == "not_yet_modeled" && e["cause"] == "boundary-mismatch"),
            "{name}: explicit finite-boundary refusal: {events:?}"
        );
        assert!(
            !events
                .iter()
                .any(|e| e["event"] == "thermal_equilibrium" || e["event"] == "gas_evolved"),
            "{name}: no open-air equilibrium or exhaust: {events:?}"
        );
        for row in &rows[ignition..] {
            let observed = vessel(row);
            assert_eq!(observed["headspace"]["boundary"], "sealed");
            assert!(observed["pressure"].as_f64().unwrap() > 0.0);
            let remaining = inventory(observed, false);
            assert!(
                (remaining.get(fuel).copied().unwrap_or(0.0) - initial[fuel]).abs() < 1e-12,
                "{name}: fuel identity retained; melting is allowed"
            );
            let actual = inventory(observed, true);
            assert_eq!(actual.len(), gases.len(), "{name}: gas identities retained");
            for (species, expected) in &gases {
                assert!(
                    (actual.get(species).copied().unwrap_or(0.0) - expected).abs() < 1e-12,
                    "{name}: {species} retained through subsequent observations"
                );
            }
        }
    }
}
