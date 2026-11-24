//! Acid–base indicator colour endpoints read where the indicator turns.
//!
//! A titration "until the colour persists" with an acid–base indicator in
//! the flask must stop where that indicator's sourced transition puts the
//! change (`Indicator::endpoint_ph`), not where a dilution happens to tip
//! the colour word of an overdosed dye. Before this the bromothymol-blue
//! titration of 0.01 mol HCl stopped at pH 0.4, a hundredth of the way to
//! equivalence, and the phenolphthalein back-titration of ammonia stopped
//! at pH 9.4 while the flask was still deep pink.

#![cfg(feature = "engine")]

use kerotakis_core::script::parse_op;
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn stack() -> SolverStack {
    let aqueous = PhreeqcEquilibrator::new().expect("engine");
    SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(aqueous)]))
}

fn step(bench: &mut Bench, s: &mut SolverStack, line: &str) -> Vec<Event> {
    let events = bench
        .step_with(
            parse_op(line).expect("parse").expect("operator"),
            s,
            &PermissiveScreen,
        )
        .unwrap_or_else(|e| panic!("{line}: {e}"));
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::SolverFailed { .. })),
        "{line}: {events:?}"
    );
    events
}

fn titration(events: &[Event]) -> (f64, bool) {
    events
        .iter()
        .find_map(|e| match e {
            Event::Titrated {
                total_volume,
                endpoint_reached,
                ..
            } => Some((total_volume.0 * 1000.0, endpoint_reached.unwrap_or(false))),
            _ => None,
        })
        .expect("a titration event")
}

fn ph(bench: &Bench) -> f64 {
    bench
        .vessel(VesselId(0))
        .expect("vessel")
        .solution
        .as_ref()
        .expect("solved")
        .ph
}

/// Positive: strong acid with strong base, bromothymol blue. Equivalence
/// is 10.0 mL of 1 M NaOH; the endpoint is read at pKIn 7.1 and so lands
/// within one 0.1 mL increment of it.
#[test]
fn bromothymol_blue_stops_a_strong_acid_titration_at_equivalence() {
    let mut bench = Bench::new();
    let mut s = stack();
    for line in [
        "add v1 water 10mL",
        "add v1 HCl 0.01mol",
        "add v1 bromothymol_blue 0.00001mol",
    ] {
        step(&mut bench, &mut s, line);
    }
    let events = step(
        &mut bench,
        &mut s,
        "titrate v1 NaOH 1M 0.1mL until colour persists max 200",
    );
    let (ml, reached) = titration(&events);
    assert!(reached, "{events:?}");
    assert!((ml - 10.0).abs() <= 0.1 + 1e-9, "endpoint at {ml} mL");
    assert!(ph(&bench) >= 7.1, "stopped at pH {}", ph(&bench));
}

/// Positive: ammonia back-titrated with HCl, phenolphthalein. The pink
/// is gone only below the 8.3 edge of its transition range, so the run
/// does not stop while the flask is still pink.
#[test]
fn phenolphthalein_endpoint_is_where_the_pink_has_gone() {
    let mut bench = Bench::new();
    let mut s = stack();
    for line in [
        "add v1 water 20mL",
        "add v1 NH3 0.013mol",
        "add v1 phenolphthalein 0.00001mol",
    ] {
        step(&mut bench, &mut s, line);
    }
    let events = step(
        &mut bench,
        &mut s,
        "titrate v1 HCl 1M 0.1mL until colour persists max 300",
    );
    let (_, reached) = titration(&events);
    assert!(reached, "{events:?}");
    let end = ph(&bench);
    assert!(end <= 8.3 && end > 7.0, "stopped at pH {end}");
}

/// Negative: a neutral phenolphthalein solution is colourless, even at
/// the millimolar concentration a bench overdose reaches — the pink
/// dianion is two protons away.
#[test]
fn neutral_phenolphthalein_is_colourless() {
    let mut bench = Bench::new();
    let mut s = stack();
    for line in [
        "add v1 ethanol 1.5mL",
        "add v1 phenolphthalein 0.01g",
        "add v1 water 2mL",
        "add v1 NaCl 0.7g",
    ] {
        step(&mut bench, &mut s, line);
    }
    let seen = appearance::observe(bench.vessel(VesselId(0)).expect("vessel"));
    assert!(
        seen.words.contains("colourless"),
        "pH {}: {}",
        ph(&bench),
        seen.words
    );
}

/// Negative: with no indicator the colour endpoint is still read off the
/// colour word — a burette of acid into plain water never "turns".
#[test]
fn no_indicator_means_no_colour_endpoint() {
    let mut bench = Bench::new();
    let mut s = stack();
    step(&mut bench, &mut s, "add v1 water 10mL");
    step(&mut bench, &mut s, "add v1 NaOH 0.001mol");
    let events = step(
        &mut bench,
        &mut s,
        "titrate v1 HCl 1M 0.1mL until colour persists max 20",
    );
    let (_, reached) = titration(&events);
    assert!(!reached, "{events:?}");
}
