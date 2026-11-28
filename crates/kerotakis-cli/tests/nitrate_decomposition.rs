//! The 2026-11-28 sweep's nitrate scripts through the real binary and the
//! full solver stack (S2/S3 in docs/reaction-sweep-gaps-20261120.md).

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static CASE: AtomicUsize = AtomicUsize::new(0);

fn run(script: &str) -> Vec<serde_json::Value> {
    let dir = std::env::temp_dir().join(format!(
        "kero-nitrate-{}-{}",
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

fn evolved(steps: &[serde_json::Value], species: &str) -> f64 {
    events(steps)
        .iter()
        .filter(|e| e["event"] == "gas_evolved" && e["species"] == species)
        .map(|e| e["moles"].as_f64().unwrap())
        .sum()
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

const ALKALI: &[&str] = &["Li", "Na", "K", "Rb", "Cs"];

#[test]
fn heated_sodium_nitrate_books_oxygen_and_never_sodium_metal() {
    // The sweep's t22 script, unchanged.
    let steps = run("new crucible\nadd v2 NaNO3 1g\nheat v2 15kJ\ninspect v2\n");
    let n = 1.0 / 84.9947;
    assert!(moles(&steps, "NaNO3") < 1e-9, "{:#?}", contents(&steps));
    assert!(
        (moles(&steps, "NaNO2") - n).abs() < 1e-6,
        "{:#?}",
        contents(&steps)
    );
    let o2 = evolved(&steps, "O2");
    assert!((o2 - 0.0059).abs() < 0.0001, "{o2}");
    for metal in ALKALI {
        assert_eq!(
            moles(&steps, metal),
            0.0,
            "{metal}: {:#?}",
            contents(&steps)
        );
    }
}

#[test]
fn heated_potassium_nitrate_stops_at_the_nitrite() {
    let steps = run("new crucible\nadd v2 KNO3 1g\nheat v2 15kJ\ninspect v2\n");
    let n = 1.0 / 101.1032;
    assert!(moles(&steps, "KNO3") < 1e-9, "{:#?}", contents(&steps));
    assert!(
        (moles(&steps, "KNO2") - n).abs() < 1e-6,
        "{:#?}",
        contents(&steps)
    );
    assert!((evolved(&steps, "O2") - n / 2.0).abs() < 1e-6);
    for metal in ALKALI {
        assert_eq!(moles(&steps, metal), 0.0, "{metal}");
    }
}

#[test]
fn heated_copper_nitrate_decomposes_to_oxide_and_brown_gas() {
    // The sweep's t23 script, unchanged: it used to stay intact at 1500 °C.
    let steps = run("new crucible\nadd v2 Cu(NO3)2 1g\nheat v2 10kJ\nlook v2\ninspect v2\n");
    let n = 1.0 / 187.554;
    assert!(moles(&steps, "Cu(NO3)2") < 1e-9, "{:#?}", contents(&steps));
    assert!((evolved(&steps, "NO2") - 2.0 * n).abs() < 1e-6);
    // Copper stays as an oxide (CuO, or Cu2O if the Gibbs route takes the
    // 1500 °C crucible on to cuprite), never the nitrate and never the metal.
    let copper = moles(&steps, "CuO") + 2.0 * moles(&steps, "Cu2O");
    assert!((copper - n).abs() < 1e-6, "{:#?}", contents(&steps));
    assert_eq!(moles(&steps, "Cu"), 0.0);
}

#[test]
fn copper_nitrate_on_the_shelf_is_blue_green() {
    let steps = run("new crucible\nadd v2 Cu(NO3)2 1g\nlook v2\n");
    let text = serde_json::to_string(&events(&steps)).unwrap();
    assert!(text.contains("blue-green"), "{text}");
}

#[test]
fn a_gentle_heat_leaves_the_nitrates_whole() {
    // 300 J takes 1 g of each salt well past room temperature but nowhere
    // near its onset (600 °C and 650 °C).
    for (key, nitrite, molar_mass) in [("NaNO3", "NaNO2", 84.9947), ("KNO3", "KNO2", 101.1032)] {
        let steps = run(&format!(
            "new crucible\nadd v2 {key} 1g\nheat v2 0.3kJ\ninspect v2\n"
        ));
        assert!(
            (moles(&steps, key) - 1.0 / molar_mass).abs() < 1e-6,
            "{key}: {:#?}",
            contents(&steps)
        );
        assert_eq!(moles(&steps, nitrite), 0.0, "{key}");
        assert_eq!(evolved(&steps, "O2"), 0.0, "{key}");
        assert_eq!(evolved(&steps, "NO2"), 0.0, "{key}");
    }
}
