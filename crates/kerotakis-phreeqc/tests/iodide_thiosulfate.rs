//! Iodide ownership and the iodometric titration (2026-10-27).
//!
//! Two shared defects blocked about nineteen originals:
//!
//! 1. `KI` was "an ionic solute without an aqueous component mapping":
//!    iodine was not a halide residue in `derived`, although wateq4f.dat
//!    (line 39) and minteq.v4.dat (line 51) both define element `I` with
//!    master species `I-`.
//! 2. `I2 + 2 S2O3-2 -> 2 I- + S4O6-2` had no owner, so iodine and
//!    thiosulfate sat side by side indefinitely.
//!
//! Positive, negative (no thiosulfate; no iodine) and boundary (excess
//! iodine, excess thiosulfate, both addition orders) controls follow.

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::derived::{self, DerivedRole};
use kerotakis_phreeqc::PhreeqcEquilibrator;

const IODOMETRY: &str = "I₂ + 2 S₂O₃²⁻ → 2 I⁻ + S₄O₆²⁻";
const IODOMETRY_BOTTLE: &str = "I₂ + 2 Na₂S₂O₃ → 2 I⁻ + S₄O₆²⁻ + 4 Na⁺";

fn stack() -> SolverStack {
    SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
        PhreeqcEquilibrator::new().expect("engine"),
    )]))
}

fn run(script: &[&str]) -> (Bench, VesselId, Vec<Event>) {
    let mut bench = Bench::new();
    let mut stack = stack();
    let mut events = Vec::new();
    for op in script {
        events.extend(
            bench
                .step_with(
                    kerotakis_core::script::parse_op(op)
                        .expect("parse")
                        .expect("a known verb"),
                    &mut stack,
                    &PermissiveScreen,
                )
                .expect("step"),
        );
    }
    (bench, VesselId(0), events)
}

fn moles(bench: &Bench, v: VesselId, key: &str) -> f64 {
    bench.vessel(v).unwrap().moles_of(&SpeciesId::new(key)).0
}

fn titrated(events: &[Event]) -> bool {
    events.iter().any(|e| {
        matches!(e, Event::ReactionOccurred { equation, .. }
            if equation == IODOMETRY || equation == IODOMETRY_BOTTLE)
    })
}

fn unmapped(events: &[Event], key: &str) -> bool {
    events.iter().any(|e| {
        matches!(e, Event::NotYetModeled { what, .. }
            if what.contains("without an aqueous component mapping") && what.contains(key))
    })
}

/// Total elemental iodine across every iodine-bearing key.
fn iodine_atoms(bench: &Bench, v: VesselId) -> f64 {
    2.0 * moles(bench, v, "I2") + moles(bench, v, "I-") + moles(bench, v, "KI")
}

/// Total thiosulfate-family sulfur.
fn sulfur_atoms(bench: &Bench, v: VesselId) -> f64 {
    2.0 * moles(bench, v, "S2O3-2")
        + 2.0 * moles(bench, v, "Na2S2O3")
        + 4.0 * moles(bench, v, "S4O6-2")
}

#[test]
fn iodide_salts_and_tetrathionate_have_derived_aqueous_roles() {
    match derived::role("KI") {
        Some(DerivedRole::Dissolves(contrib)) => {
            assert!(
                contrib.iter().any(|(el, n)| el == "I" && *n == 1.0),
                "{contrib:?}"
            );
            assert!(
                contrib.iter().any(|(el, n)| el == "K" && *n == 1.0),
                "{contrib:?}"
            );
        }
        other => panic!("KI must dissolve into K and I, got {other:?}"),
    }
    assert_eq!(derived::booking_ion("I"), Some("I-"));
    // Molecular iodine must NOT become iodide in a solve: that would mint
    // two electrons per I2. It is a bounded-solubility solid with no role.
    assert!(derived::role("I2").is_none(), "{:?}", derived::role("I2"));
    match derived::role("S4O6-2") {
        Some(DerivedRole::Dissolves(contrib)) => {
            assert_eq!(contrib, &vec![("Tetrathionate".to_string(), 1.0)]);
        }
        other => panic!("S4O6-2 must book as Tetrathionate, got {other:?}"),
    }
    assert_eq!(derived::booking_ion("Tetrathionate"), Some("S4O6-2"));
    // Thiosulfate's exact rule must not claim tetrathionate, nor vice versa.
    match derived::role("S2O3-2") {
        Some(DerivedRole::Dissolves(contrib)) => {
            assert_eq!(contrib, &vec![("Thiosulfate".to_string(), 1.0)]);
        }
        other => panic!("S2O3-2 role changed: {other:?}"),
    }
}

#[test]
fn potassium_iodide_speciates_instead_of_being_unmapped() {
    let (bench, v, events) = run(&["add v1 water 100mL", "add v1 KI 0.002mol"]);
    assert!(!unmapped(&events, "potassium iodide"), "{events:?}");
    assert!(bench.vessel(v).unwrap().solution.is_some());
    assert!((moles(&bench, v, "I-") - 0.002).abs() < 1e-9);
    assert!((moles(&bench, v, "K+") - 0.002).abs() < 1e-9);
    assert!(moles(&bench, v, "KI") < 1e-12);
}

#[test]
fn iodine_is_reduced_by_thiosulfate_with_limiting_stoichiometry() {
    // Thiosulfate in excess: all iodine is reduced.
    let (bench, v, events) = run(&[
        "add v1 water 100mL",
        "add v1 Na2S2O3 0.002mol",
        "add v1 I2 0.0005mol",
    ]);
    assert!(titrated(&events), "{events:?}");
    assert!(moles(&bench, v, "I2") < 1e-12);
    assert!((moles(&bench, v, "I-") - 0.001).abs() < 1e-9);
    assert!((moles(&bench, v, "S4O6-2") - 0.0005).abs() < 1e-9);
    assert!((moles(&bench, v, "S2O3-2") - 0.001).abs() < 1e-9);
    assert!((iodine_atoms(&bench, v) - 0.001).abs() < 1e-9);
    assert!((sulfur_atoms(&bench, v) - 0.004).abs() < 1e-9);
    assert!((moles(&bench, v, "Na+") - 0.004).abs() < 1e-9);
}

#[test]
fn excess_iodine_remains_after_thiosulfate_is_spent() {
    // Iodine first, then thiosulfate: the bottle sibling's order.
    let (bench, v, events) = run(&[
        "add v1 water 100mL",
        "add v1 KI 0.002mol",
        "add v1 I2 0.0005mol",
        "add v1 Na2S2O3 0.0004mol",
    ]);
    assert!(titrated(&events), "{events:?}");
    assert!((moles(&bench, v, "I2") - 0.0003).abs() < 1e-9);
    assert!((moles(&bench, v, "S4O6-2") - 0.0002).abs() < 1e-9);
    assert!(moles(&bench, v, "S2O3-2") < 1e-12);
    assert!(moles(&bench, v, "Na2S2O3") < 1e-12);
    // 0.002 from KI plus 0.0004 from the reduced iodine.
    assert!((moles(&bench, v, "I-") - 0.0024).abs() < 1e-9);
    assert!((iodine_atoms(&bench, v) - 0.003).abs() < 1e-9);
}

#[test]
fn iodine_without_thiosulfate_does_not_react() {
    let (bench, v, events) = run(&[
        "add v1 water 100mL",
        "add v1 KI 0.002mol",
        "add v1 I2 0.0002mol",
    ]);
    assert!(!titrated(&events));
    assert_eq!(moles(&bench, v, "S4O6-2"), 0.0);
    assert!((moles(&bench, v, "I2") - 0.0002).abs() < 1e-12);
}

#[test]
fn thiosulfate_without_iodine_does_not_react() {
    let (bench, v, events) = run(&[
        "add v1 water 100mL",
        "add v1 KI 0.002mol",
        "add v1 Na2S2O3 0.001mol",
    ]);
    assert!(!titrated(&events));
    assert_eq!(moles(&bench, v, "S4O6-2"), 0.0);
    assert!((moles(&bench, v, "S2O3-2") - 0.001).abs() < 1e-9);
}

#[test]
fn a_titrated_vessel_stays_electroneutral_and_characterised() {
    let (bench, v, _) = run(&[
        "add v1 water 100mL",
        "add v1 Na2S2O3 0.001mol",
        "add v1 I2 0.0005mol",
    ]);
    let vessel = bench.vessel(v).unwrap();
    let solution = vessel
        .solution
        .clone()
        .expect("tetrathionate is routed, not unmapped");
    // Na2S4O6 + NaI is a neutral salt solution: no invented acid or base
    // from an unrepresented dianion.
    assert!(solution.ph > 5.0 && solution.ph < 9.0, "pH {}", solution.ph);
    let charge = kerotakis_core::displacement::solute_charge(vessel);
    assert!(
        (vessel.solute_charge - charge).abs() < 1e-12,
        "solute_charge refreshed: {} vs {charge}",
        vessel.solute_charge
    );
}
