//! Thiosulfate, and the database it was said not to be in.
//!
//! `species.rs` said "Sodium thiosulfate is in no PHREEQC database we
//! ship". `codex/rates.toml` told a learner twice that "no shipped
//! database carries one for thiosulfate", and built a whole prediction
//! exercise on it. `vendor/iphreeqc/database/llnl.dat` is shipped in this
//! repository and carries, at line 229,
//! `S(+2)     S2O3-2    0         S`, the formation at line 739, and at
//! line 4585 `H+ + S2O3-2 = HS2O3-`, `log_k 1.0139`.
//!
//! `databases::minteq_v4()` borrows that one constant the way it already
//! borrows lactate and hypochlorite, as a pseudo-element with no redox
//! partner. These are the claims that buys — including the one the codex
//! got backwards.

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::derived::{self, DerivedRole};
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn stack() -> SolverStack {
    SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
        PhreeqcEquilibrator::new().expect("engine"),
    )]))
}

fn run(script: &[&str]) -> (Bench, VesselId) {
    let mut bench = Bench::new();
    let mut stack = stack();
    for op in script {
        bench
            .step_with(
                kerotakis_core::script::parse_op(op)
                    .expect("parse")
                    .expect("a known verb"),
                &mut stack,
                &PermissiveScreen,
            )
            .expect("step");
    }
    (bench, VesselId(0))
}

fn ph(bench: &Bench, v: VesselId) -> f64 {
    bench
        .vessel(v)
        .unwrap()
        .solution
        .clone()
        .expect("a thiosulfate solution must be characterised")
        .ph
}

fn moles(bench: &Bench, v: VesselId, key: &str) -> f64 {
    bench.vessel(v).unwrap().moles_of(&SpeciesId::new(key)).0
}

/// The vessel is characterised at all, which is the whole of what was
/// denied.
///
/// The codex's `a-salt-with-no-database` asked a learner what this bench
/// reports for 0.79 g of hypo in 100 mL of pure water and marked "Nothing
/// at all: no solver can speciate thiosulfate" as the right answer. It is
/// no longer true, and the number that replaces it is the interesting
/// part — see the next test.
#[test]
fn hypo_in_pure_water_now_has_a_solution_at_all() {
    let (bench, v) = run(&["add v1 water 100mL", "add v1 Na2S2O3 0.79g"]);
    assert!(
        bench.vessel(v).unwrap().solution.is_some(),
        "the salt speciates, so the vessel has a solution"
    );
    let ion = moles(&bench, v, "S2O3-2");
    assert!(
        (ion - 0.005).abs() < 2e-4,
        "0.79 g is 5 mmol and comes back as the ion: {ion}"
    );
    assert_eq!(
        moles(&bench, v, "Na2S2O3"),
        0.0,
        "the bottle name is gone from a solved vessel, which is why the rate law names the ion"
    );
}

/// And the answer is NOT the "slightly alkaline" one the codex called the
/// right chemistry.
///
/// The exercise's `misconception` said "S₂O₃²⁻ is the conjugate base of a
/// weak acid and a real solution is mildly alkaline", and offered pH 8.5
/// as the correct option. Thiosulfuric acid is not a weak acid: its second
/// pKa is 1.01 by the constant llnl.dat carries and 1.6–1.7 in the
/// handbooks, so K_b is around 10⁻¹³ and 0.05 mol/L hydrolyses to
/// [OH⁻] = √(K_b·C) ≈ 2 × 10⁻⁷ — pH 7.4, a couple of tenths above neutral
/// and nowhere near 8.5.
///
/// So the borrowed constant does not merely fill a gap; it settles a
/// question the codex had answered wrongly on both sides. The window here
/// is deliberately tight enough to fail if the couple is ever dropped
/// (without it the vessel has no solution at all) and if anyone ever
/// re-asserts the 8.5.
#[test]
fn a_thiosulfate_solution_is_barely_alkaline_not_mildly_alkaline() {
    let (bench, v) = run(&["add v1 water 100mL", "add v1 Na2S2O3 0.79g"]);
    let p = ph(&bench, v);
    assert!(
        (6.9..8.0).contains(&p),
        "pKa 1.01 leaves the anion barely hydrolysed: expected about 7.4, got {p}"
    );
    assert!(
        p < 8.2,
        "the codex's 'about pH 8.5, as the chemistry requires' is not what the chemistry requires: {p}"
    );
    eprintln!("HARVEST hypo-in-water pH = {p:.10}");
}

/// The pseudo-element carries no redox partner, and an open beaker is
/// where that would show.
///
/// llnl enters thiosulfate as `S(+2)`, one of nine sulfur states it
/// defines; the routed datasets define two, coupled through pe, and an
/// open vessel's pe is pinned near 19.6 from atmospheric oxygen. Entered
/// as a state of sulfur the bottle would oxidise itself to sulfate before
/// anything was added to it. It does not: every millimole is still
/// thiosulfate.
#[test]
fn an_open_beaker_does_not_oxidise_the_bottle_to_sulfate() {
    let (bench, v) = run(&["add v1 water 100mL", "add v1 Na2S2O3 0.79g", "wait 600s"]);
    let ion = moles(&bench, v, "S2O3-2");
    assert!(
        ion > 0.0049,
        "thiosulfate is kinetically persistent in a bottle: {ion} mol left"
    );
    assert_eq!(
        moles(&bench, v, "SO4-2"),
        0.0,
        "nothing turned it into sulfate"
    );
}

/// What the derivation books, and what the exact rule excludes.
///
/// The matcher is EXACTLY two sulfurs and three oxygens — the sulfate-like
/// centre with one oxygen replaced by the second sulfur, which is the
/// whole of why acid takes it apart. A greedy `S2O3` group would take
/// those counts out of any formula that could spare them, and the shipped
/// databases are full of ones that can.
#[test]
fn the_thiosulfate_matcher_is_exact() {
    match derived::role("S2O3-2") {
        Some(DerivedRole::Dissolves(els)) => {
            assert_eq!(els.as_slice(), &[("Thiosulfate".to_string(), 1.0)]);
        }
        other => panic!("the ion must book one thiosulfate, got {other:?}"),
    }
    match derived::role("Na2S2O3") {
        Some(DerivedRole::Dissolves(els)) => {
            assert!(els.contains(&("Thiosulfate".to_string(), 1.0)));
            assert!(els.contains(&("Na".to_string(), 2.0)));
        }
        other => panic!("the salt must book thiosulfate plus its sodium, got {other:?}"),
    }
    // Sulfite is one sulfur, so the count is wrong before the oxygen is
    // looked at; no routed dataset defines sulfur(IV) and none is
    // borrowed here, so it stays unmappable.
    assert!(derived::role("SO3-2").is_none());
    assert_eq!(derived::booking_ion("Thiosulfate"), Some("S2O3-2"));
}

/// The clock reaction still runs, on the name the vessel actually holds.
///
/// This is the assertion that would have gone silent. `thiosulfate-acid`
/// used to name `Na2S2O3` in both its stoichiometry and its rate orders;
/// a solved beaker holds `S2O3-2`, so the reaction would have matched in a
/// dry vessel, failed in every wet one, and reported nothing either way —
/// there is no `curated_reactants_survive_a_solve` for rate laws.
#[test]
fn the_disappearing_cross_still_disappears() {
    let (bench, v) = run(&[
        "add v1 water 100mL",
        "add v1 Na2S2O3 0.79g",
        "add v1 HCl 0.002mol",
        "wait 60s",
    ]);
    let sulfur = moles(&bench, v, "S");
    assert!(
        sulfur > 1e-5,
        "acid on thiosulfate deposits sulfur, got {sulfur} mol"
    );
    let left = moles(&bench, v, "S2O3-2");
    assert!(
        left < 0.005,
        "and it is the thiosulfate that was spent: {left} mol left of 0.005"
    );
}

/// Harvesting run for the codex numbers this change invalidates.
///
/// `a-salt-with-no-database` and `the-acid-is-never-used-up` both quote
/// the acid beaker's pH to twelve figures and both assert that the
/// thiosulfate contributes nothing to it. With pKa 1.01 in the database
/// that is no longer true at the practical's pH — about a seventh of the
/// anion is protonated at pH 1.75, and protonating it consumes acid. This
/// prints what the three beakers actually read so the codex can be
/// rewritten from measurements rather than from the old sentence.
#[test]
fn what_the_three_beakers_read() {
    let (with, v) = run(&[
        "add v1 water 100mL",
        "add v1 Na2S2O3 0.79g",
        "add v1 HCl 0.002mol",
    ]);
    let (without, w) = run(&["add v1 water 100mL", "add v1 HCl 0.002mol"]);
    let (alone, a) = run(&["add v1 water 100mL", "add v1 Na2S2O3 0.79g"]);
    eprintln!("HARVEST acid+thiosulfate pH = {:.10}", ph(&with, v));
    eprintln!("HARVEST acid alone        pH = {:.10}", ph(&without, w));
    eprintln!("HARVEST thiosulfate alone pH = {:.10}", ph(&alone, a));
    eprintln!(
        "HARVEST ionic strength with = {:?}",
        with.vessel(v)
            .unwrap()
            .solution
            .as_ref()
            .map(|s| s.ionic_strength)
    );
    eprintln!(
        "HARVEST ionic strength without = {:?}",
        without
            .vessel(w)
            .unwrap()
            .solution
            .as_ref()
            .map(|s| s.ionic_strength)
    );
}

/// The ten-minute run `the-acid-is-never-used-up` is built on, harvested.
///
/// Its whole observation was that the pH "never moves off 1.74858175066".
/// It moves now, and for a reason that sharpens the entry rather than
/// contradicting it: the rate law still does not consume protons, but the
/// thiosulfate that was HOLDING some at pKa 1.01 is being turned into
/// sulfur, and it hands them back as it goes.
#[test]
fn the_ten_minute_run() {
    let (bench, v) = run(&[
        "add v1 water 100mL",
        "add v1 Na2S2O3 0.79g",
        "add v1 HCl 0.002mol",
    ]);
    eprintln!("HARVEST t=0    pH = {:.10}", ph(&bench, v));
    eprintln!("HARVEST t=0    S2O3-2 = {:.10}", moles(&bench, v, "S2O3-2"));
    let (after, w) = run(&[
        "add v1 water 100mL",
        "add v1 Na2S2O3 0.79g",
        "add v1 HCl 0.002mol",
        "wait 10min",
    ]);
    eprintln!("HARVEST t=600  pH = {:.10}", ph(&after, w));
    eprintln!("HARVEST t=600  S2O3-2 = {:.10}", moles(&after, w, "S2O3-2"));
    eprintln!("HARVEST t=600  S      = {:.10}", moles(&after, w, "S"));

    // The forty-second observable the pre-exponential is calibrated to.
    let (clock, c) = run(&[
        "add v1 water 100mL",
        "add v1 Na2S2O3 0.79g",
        "add v1 HCl 0.002mol",
        "wait 40s",
    ]);
    eprintln!("HARVEST t=40   S      = {:.10}", moles(&clock, c, "S"));
    eprintln!("HARVEST t=40   S2O3-2 = {:.10}", moles(&clock, c, "S2O3-2"));
}
