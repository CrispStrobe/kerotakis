//! Perturbation tests: does a number MOVE, in the right direction, when
//! its stated cause moves?
//!
//! This is the sibling of `metamorphic.rs` and the generalisation, across
//! time, of `tools/curiosity-answer-invariance.py`. That tool asks a
//! question across *vessels* — a prompt that distinguishes three metals
//! must produce three different answers — and caught `mat-012`, a row that
//! passed for as long as the corpus existed while weighing five grams of
//! each metal and reading the same number three times.
//!
//! The same question asked across *a change to an input* is the one this
//! file poses: change a cause the bench says it depends on, and the
//! consequence must move; change something the bench says it is
//! independent of, and the consequence must not. Neither half needs a
//! reference value, so nothing here has to be sourced, cited or licensed,
//! and nothing has to be re-blessed when the solver improves.
//!
//! Every expensive defect found here in the last fortnight has that shape
//! and every one was found by hand:
//!
//! | defect | the perturbation that finds it |
//! |---|---|
//! | float-or-sink read a species density, not the recipe's bulk density | move the recipe's bulk density; the verdict must move |
//! | alkalinity treated as a portion, not a charge balance | add acid; it must fall by the charge equivalent |
//! | an element arriving mid-solve was dropped on readback | transfer one in; the totals must change |
//! | a rate law keyed on the bottle rather than the ion | speciate the reagent; the reaction must still fire |
//!
//! **What this file is careful about.** A perturbation that moves an input
//! the engine simply multiplies by always passes and proves nothing. Each
//! test below says, in its own doc comment, what it establishes and what
//! it cannot — and the weak ones say that they are weak rather than being
//! quietly counted as coverage.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Distinguishes concurrent `run` calls — see the note in `metamorphic.rs`:
/// a content-derived directory name once collided between two threads.
static CASE: AtomicUsize = AtomicUsize::new(0);

/// Run a script through `kero run --json` and parse the step stream.
fn run(script: &str) -> Vec<serde_json::Value> {
    let dir = std::env::temp_dir().join(format!(
        "kero-perturb-{}-{}",
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

/// Named vessel state after the last step.
fn vessel(steps: &[serde_json::Value], index: usize) -> serde_json::Value {
    steps.last().expect("at least one step")["bench"]["vessels"][index].clone()
}
