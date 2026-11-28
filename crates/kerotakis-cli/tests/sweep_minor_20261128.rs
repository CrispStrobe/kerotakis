//! The 2026-11-28 sweep's minor observations (S7 in
//! docs/reaction-sweep-gaps-20261120.md) through the real binary.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static CASE: AtomicUsize = AtomicUsize::new(0);

fn run(script: &str) -> Vec<serde_json::Value> {
    let dir = std::env::temp_dir().join(format!(
        "kero-sweep-minor-{}-{}",
        std::process::id(),
        CASE.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let lab = dir.join("case.lab");
    std::fs::write(&lab, script).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args(["run", lab.to_str().unwrap(), "--json"])
        .output()
        .expect("kero runs");
    assert!(
        out.status.success(),
        "script failed:\n{script}\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let steps = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).expect("every line is JSON"))
        .collect();
    std::fs::remove_dir_all(&dir).ok();
    steps
}

fn events(steps: &[serde_json::Value]) -> Vec<serde_json::Value> {
    steps
        .iter()
        .flat_map(|s| s["events"].as_array().cloned().unwrap_or_default())
        .collect()
}

/// The last vessel's contents (the scripts work in the vessel they create).
fn contents(steps: &[serde_json::Value]) -> Vec<serde_json::Value> {
    let vessels = steps.last().unwrap()["bench"]["vessels"]
        .as_array()
        .cloned()
        .unwrap();
    vessels.last().unwrap()["contents"]
        .as_array()
        .cloned()
        .unwrap()
}

fn moles(steps: &[serde_json::Value], species: &str) -> f64 {
    contents(steps)
        .iter()
        .filter(|p| p["species"] == species)
        .map(|p| p["moles"].as_f64().unwrap())
        .sum()
}

/// The sealed-water pair of S7 (2026-11-28 sweep): acidifying plain water
/// raises no fizz hazard, acid on chalk still does.
fn hazard_rules(steps: &[serde_json::Value]) -> Vec<String> {
    events(steps)
        .iter()
        .filter_map(|e| e["rule"].as_str().map(str::to_string))
        .collect()
}

#[test]
fn acid_into_sealed_plain_water_raises_no_carbonate_fizz() {
    let steps = run("seal v1 500mL\nadd v1 water 50mL\nadd v1 HCl 0.02mol\n");
    assert!(
        !hazard_rules(&steps)
            .iter()
            .any(|r| r == "acid-carbonate-co2"),
        "{:#?}",
        events(&steps)
    );
}

#[test]
fn acid_on_chalk_still_raises_carbonate_fizz() {
    let steps = run("add v1 water 50mL\nadd v1 HCl 0.02mol\nadd v1 CaCO3 0.5g\n");
    assert!(
        hazard_rules(&steps)
            .iter()
            .any(|r| r == "acid-carbonate-co2"),
        "{:#?}",
        events(&steps)
    );
}

#[test]
fn heated_baking_soda_books_positive_heat_into_chemistry() {
    // S7: the decomposition used to pay nothing, so the step reported
    // -1.44 kJ "into chemistry". It is endothermic.
    let steps = run("new crucible\nadd v2 NaHCO3 2g\nheat v2 5kJ\ninspect v2\n");
    let (delivered, sensible) = events(&steps)
        .iter()
        .find(|e| e["event"] == "energy_transferred")
        .map(|e| {
            (
                e["delivered_j"].as_f64().unwrap(),
                e["sensible_j"].as_f64().unwrap(),
            )
        })
        .expect("an energy account");
    let reaction = 2.0 / 84.007 / 2.0 * 91_600.0;
    assert!(
        delivered - sensible > 0.9 * reaction,
        "{delivered} {sensible}"
    );
    assert!(moles(&steps, "NaHCO3") < 1e-9, "{:#?}", contents(&steps));
}
