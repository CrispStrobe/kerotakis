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

/// Adjudicated 2026-11-24 (C1 closed-vessel combustion). This test was
/// written when the only engine for a sealed burn was the open-air CEA
/// route, which correctly refused a finite boundary, so "no invented closed
/// thermal chemistry" meant "no thermal chemistry at all". Magnesium in a
/// sealed flask of oxygen is now burned by the spark route over sourced data
/// (2 Mg + O2 -> 2 MgO, Wagman 1982 enthalpy of formation, limited by the
/// oxygen the flask owns). The INTENT is kept and made stricter for that
/// case: the chemistry must be the spark route's and nobody else's, the CEA
/// refusal must not contradict it, and every atom must be accounted for —
/// metal plus oxide equals the metal fed, and oxygen retained plus vented
/// plus consumed equals the oxygen present. Carbon has no spark-route row,
/// so its cases still assert the boundary refusal and untouched inventory.
#[test]
fn sealed_combustion_scripts_retain_inventory_or_account_for_a_pressure_burst() {
    let magnesium = include_str!("fixtures/thermal-boundary/sealed-magnesium.lab");
    let carbon = include_str!("fixtures/thermal-boundary/sealed-carbon.lab");
    let low_magnesium = magnesium.replacen("seal v1 ", "sweep v1 0.1bar\nseal v1 ", 1);
    let low_carbon = carbon.replacen("seal v1 ", "sweep v1 0.1bar\nseal v1 ", 1);
    // (name, script, fuel, burns on the spark route, expected burst)
    for (name, script, fuel, burns, should_burst) in [
        ("magnesium", magnesium, "Mg", true, Some(true)),
        ("carbon", carbon, "C", false, Some(true)),
        (
            "low-pressure-magnesium",
            low_magnesium.as_str(),
            "Mg",
            true,
            // Whether the adiabatic upper bound bursts a flask holding a
            // tenth of an atmosphere is the arithmetic's to say; either way
            // the gas must be accounted for below.
            None,
        ),
        (
            "low-pressure-carbon",
            low_carbon.as_str(),
            "C",
            false,
            Some(false),
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
        let refused = events
            .iter()
            .any(|e| e["event"] == "not_yet_modeled" && e["cause"] == "boundary-mismatch");
        let equilibria: Vec<_> = events
            .iter()
            .filter(|e| e["event"] == "thermal_equilibrium")
            .collect();
        let mut consumed = BTreeMap::new();
        if burns {
            assert!(
                !refused,
                "{name}: a committed burn may not be followed by a refusal that says nothing was committed: {events:?}"
            );
            assert!(
                !equilibria.is_empty()
                    && equilibria.iter().all(
                        |e| e["provenance"]["dataset"] == "kerotakis:combustion:spark-flame-v1"
                    ),
                "{name}: the only closed thermal chemistry is the spark route's: {events:?}"
            );
            for e in events.iter().filter(|e| e["event"] == "consumed") {
                *consumed
                    .entry(e["species"].as_str().unwrap().to_string())
                    .or_insert(0.0) += e["moles"].as_f64().unwrap();
            }
            assert!(consumed.get("O2").copied().unwrap_or(0.0) > 0.0);
        } else {
            assert!(
                refused,
                "{name}: explicit finite-boundary refusal: {events:?}"
            );
            assert!(
                equilibria.is_empty(),
                "{name}: no invented closed thermal chemistry: {events:?}"
            );
        }
        let bursts: Vec<_> = events.iter().filter(|e| e["event"] == "burst").collect();
        let mut vented = BTreeMap::new();
        for event in events.iter().filter(|e| e["event"] == "gas_evolved") {
            *vented
                .entry(event["species"].as_str().unwrap().to_string())
                .or_insert(0.0) += event["moles"].as_f64().unwrap();
        }
        let burst = !bursts.is_empty();
        if let Some(expected) = should_burst {
            assert_eq!(burst, expected, "{name}: burst expectation");
        }
        if burst {
            assert_eq!(bursts.len(), 1, "{name}: one burst");
            assert!(
                bursts[0]["at_pa"].as_f64().unwrap() > bursts[0]["rating_pa"].as_f64().unwrap()
            );
            assert!(events.iter().any(|e| e["event"] == "hazard_warning"
                && e["rule"] == "sealed-vessel-burst"
                && e["severity"] == "danger"));
        } else {
            assert!(vented.is_empty(), "{name}: an intact vessel must not vent");
        }
        for row in &rows[ignition..] {
            let observed = vessel(row);
            assert_eq!(
                observed["headspace"]["boundary"],
                if burst { "open" } else { "sealed" }
            );
            assert!(observed["pressure"].as_f64().unwrap() > 0.0);
            let remaining = inventory(observed, false);
            if burns {
                // Metal plus oxide is the metal fed: nothing invented, nothing lost.
                let metal = remaining.get(fuel).copied().unwrap_or(0.0)
                    + remaining.get("MgO").copied().unwrap_or(0.0);
                assert!(
                    (metal - initial[fuel]).abs() < 1e-12,
                    "{name}: Mg + MgO equals the Mg fed: {remaining:?}"
                );
                assert!(remaining.get("MgO").copied().unwrap_or(0.0) > 0.0);
            } else {
                assert!(
                    (remaining.get(fuel).copied().unwrap_or(0.0) - initial[fuel]).abs() < 1e-12,
                    "{name}: fuel identity retained; melting is allowed"
                );
            }
            let actual = inventory(observed, true);
            let mut accounted = actual.clone();
            for (species, amount) in vented.iter().chain(consumed.iter()) {
                if gases.contains_key(species) {
                    *accounted.entry(species.clone()).or_insert(0.0) += amount;
                }
            }
            assert_eq!(
                accounted.len(),
                gases.len(),
                "{name}: retained plus vented plus consumed gas identities: {accounted:?}"
            );
            for (species, expected) in &gases {
                assert!(
                    (accounted.get(species).copied().unwrap_or(0.0) - expected).abs() < 1e-12,
                    "{name}: {species} retained, explicitly vented or consumed through subsequent observations"
                );
            }
        }
    }
}
