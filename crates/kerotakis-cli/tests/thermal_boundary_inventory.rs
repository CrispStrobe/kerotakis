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
fn sealed_combustion_scripts_retain_inventory_or_account_for_a_pressure_burst() {
    let magnesium = include_str!("fixtures/thermal-boundary/sealed-magnesium.lab");
    let carbon = include_str!("fixtures/thermal-boundary/sealed-carbon.lab");
    let low_magnesium = magnesium.replacen("seal v1 ", "sweep v1 0.1bar\nseal v1 ", 1);
    let low_carbon = carbon.replacen("seal v1 ", "sweep v1 0.1bar\nseal v1 ", 1);
    for (name, script, fuel, should_burst) in [
        ("magnesium", magnesium, "Mg", true),
        ("carbon", carbon, "C", true),
        (
            "low-pressure-magnesium",
            low_magnesium.as_str(),
            "Mg",
            false,
        ),
        ("low-pressure-carbon", low_carbon.as_str(), "C", false),
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
            !events.iter().any(|e| e["event"] == "thermal_equilibrium"),
            "{name}: no invented closed thermal chemistry: {events:?}"
        );
        let bursts: Vec<_> = events.iter().filter(|e| e["event"] == "burst").collect();
        let mut vented = BTreeMap::new();
        for event in events.iter().filter(|e| e["event"] == "gas_evolved") {
            *vented
                .entry(event["species"].as_str().unwrap().to_string())
                .or_insert(0.0) += event["moles"].as_f64().unwrap();
        }
        if should_burst {
            assert_eq!(
                bursts.len(),
                1,
                "{name}: the original air-filled beaker exceeds its rating"
            );
            assert!(
                bursts[0]["at_pa"].as_f64().unwrap() > bursts[0]["rating_pa"].as_f64().unwrap()
            );
            assert!(events.iter().any(|e| e["event"] == "hazard_warning"
                && e["rule"] == "sealed-vessel-burst"
                && e["severity"] == "danger"));
        } else {
            assert!(
                bursts.is_empty() && vented.is_empty(),
                "{name}: low pressure must not vent"
            );
        }
        for row in &rows[ignition..] {
            let observed = vessel(row);
            assert_eq!(
                observed["headspace"]["boundary"],
                if should_burst { "open" } else { "sealed" }
            );
            assert!(observed["pressure"].as_f64().unwrap() > 0.0);
            let remaining = inventory(observed, false);
            assert!(
                (remaining.get(fuel).copied().unwrap_or(0.0) - initial[fuel]).abs() < 1e-12,
                "{name}: fuel identity retained; melting is allowed"
            );
            let actual = inventory(observed, true);
            let mut accounted = actual.clone();
            for (species, amount) in &vented {
                *accounted.entry(species.clone()).or_insert(0.0) += amount;
            }
            assert_eq!(
                accounted.len(),
                gases.len(),
                "{name}: retained plus vented gas identities"
            );
            for (species, expected) in &gases {
                assert!(
                    (accounted.get(species).copied().unwrap_or(0.0) - expected).abs() < 1e-12,
                    "{name}: {species} retained or explicitly vented through subsequent observations"
                );
            }
        }
    }
}
