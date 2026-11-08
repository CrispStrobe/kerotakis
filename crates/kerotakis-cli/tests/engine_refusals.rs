//! Aqueous admission through the real binary: a vessel whose liquid is not
//! a water-majority solution must be refused with a typed reason, never
//! handed to the aqueous engine to fail (and fail again on every tick).

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static CASE: AtomicUsize = AtomicUsize::new(0);

fn run(script: &str) -> Vec<serde_json::Value> {
    let dir = std::env::temp_dir().join(format!(
        "kero-refusal-{}-{}",
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
        .filter_map(|s| s["events"].as_array())
        .flatten()
        .cloned()
        .collect()
}

/// The observed live bug: neat hydrochloric acid and neat hydrogen peroxide,
/// no water, on a ticking clock. Decomposition makes some water each tick;
/// that water must not admit ~3 mol of HCl as an aqueous solution.
pub const NEAT_HCL_PEROXIDE: &str = "add v1 HCl 3.2638mol\n\
     add v1 H2O2 3.2637mol\n\
     wait 1s\nwait 1s\nwait 1s\nwait 1s\nwait 1s\nwait 1s\n";

#[test]
fn neat_hcl_and_peroxide_refuse_without_a_solver_failure() {
    let steps = run(NEAT_HCL_PEROXIDE);
    let all = events(&steps);
    let failed: Vec<_> = all
        .iter()
        .filter(|e| e["event"] == "solver_failed")
        .collect();
    assert!(
        failed.is_empty(),
        "no aqueous solve may be attempted: {failed:?}"
    );
    let text = serde_json::to_string(&all).unwrap();
    assert!(
        text.contains("not-modeled.too-little-water")
            || text.contains("not-modeled.peroxide-rich-medium"),
        "a typed medium refusal must be spoken: {text}"
    );
    // Once decomposition water outnumbers the peroxide, the reason that
    // remains is the solute-sufficiency one.
    let later = serde_json::to_string(&events(&steps[2..])).unwrap();
    assert!(!later.contains("Cl has not converged"), "{later}");
}

#[test]
fn concentrated_37_percent_hydrochloric_acid_still_solves() {
    // 100 g of 37 % HCl: 37 g HCl (1.0148 mol) in 63 g water.
    let steps = run("add v1 water 63mL\nadd v1 HCl 1.0148mol\nwait 1s\n");
    let all = events(&steps);
    assert!(
        !all.iter().any(|e| e["event"] == "solver_failed"),
        "{all:?}"
    );
    let text = serde_json::to_string(&all).unwrap();
    assert!(!text.contains("not-modeled.too-little-water"), "{text}");
}
