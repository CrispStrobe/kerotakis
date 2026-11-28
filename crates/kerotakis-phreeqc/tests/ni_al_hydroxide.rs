//! Nickel(II) and aluminium hydroxides on the hydroxide precipitation /
//! redissolution route (2026-11-27), alongside `hydroxide_redissolution.rs`.
//!
//! Every bound below is read from the routed databases' own constants, not
//! from a scenario yield. Ignoring activity corrections (they enter
//! numerator and denominator alike for the singly charged pairs used):
//!
//! * Aluminium (wateq4f.dat `Al(OH)3(a)` log K 10.8, `Al(OH)4-` log K
//!   -22.7; minteq.v4.dat `Al(OH)3(am)` 10.8, `Al(OH)4-` -22.688). With
//!   Kw = 1e-14, `Al(OH)3(a) + OH- = Al(OH)4-` has log K 10.8 - 22.7 + 14
//!   = 2.1, so [Al(OH)4-]/[OH-] ~ 126 at saturation. In ~0.7 M lye that
//!   ceiling is far above the 0.03 M of aluminium present: the whole
//!   precipitate dissolves. In an NH3/NH4+ buffer at 1:1 (pH ~ pKa 9.25,
//!   [OH-] ~ 1.8e-5 M) the ceiling is ~2.2e-3 M, i.e. ~22 umol of the
//!   200 umol in 10 mL: most of the precipitate stays.
//! * Nickel (wateq4f.dat `Ni(OH)2` log K 10.8, `Ni(OH)3-` log K -30;
//!   minteq.v4.dat 12.794 and -29.991). `Ni(OH)2 + OH- = Ni(OH)3-` has
//!   log K -5.2 (wateq4f) or -3.2 (minteq.v4), so in ~0.7 M lye at most
//!   ~4.5e-4 M nickel dissolves (3 umol in 7 mL, minteq.v4; ~0.03 umol on
//!   wateq4f): nickel hydroxide is not amphoteric at bench scale.
#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn run(script: &[&str]) -> (Bench, Vec<Vec<Event>>) {
    let mut bench = Bench::new();
    let mut stack = SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
        PhreeqcEquilibrator::new().expect("engine"),
    )]));
    let mut steps = Vec::new();
    for op in script {
        steps.push(
            bench
                .step_with(
                    kerotakis_core::script::parse_op(op)
                        .expect("parse")
                        .expect("a known verb"),
                    &mut stack,
                    &PermissiveScreen,
                )
                .unwrap_or_else(|e| panic!("{op}: {e}")),
        );
    }
    (bench, steps)
}

fn vessel(bench: &Bench) -> &Vessel {
    bench.vessel(VesselId(0)).unwrap()
}

fn solid(bench: &Bench, key: &str) -> f64 {
    vessel(bench)
        .contents
        .iter()
        .filter(|p| p.species == SpeciesId::new(key) && p.phase == Phase::Solid)
        .map(|p| p.moles.0)
        .sum()
}

fn molality(bench: &Bench, name: &str) -> f64 {
    vessel(bench)
        .solution
        .as_ref()
        .map(|s| {
            s.species
                .iter()
                .filter(|x| x.name == name)
                .map(|x| x.molality)
                .sum()
        })
        .unwrap_or(0.0)
}

fn element(bench: &Bench, el: &str) -> f64 {
    kerotakis_core::ledger::ConservedLedger::from_vessel(vessel(bench))
        .elements
        .get(el)
        .copied()
        .unwrap_or(0.0)
}

#[test]
fn aluminium_hydroxide_precipitates_white_from_alum_and_lye() {
    // 0.0002 mol Al+3 and exactly 3 OH- each.
    let (bench, steps) = run(&[
        "add v1 water 7mL",
        "add v1 Al2(SO4)3 0.0001mol",
        "add v1 NaOH 0.0006mol",
    ]);
    let down = solid(&bench, "Al(OH)3");
    assert!(down > 1.5e-4, "Al(OH)3 {down}: {steps:#?}");
    assert!((element(&bench, "Al") - 0.0002).abs() < 1e-9);
    let look = kerotakis_core::appearance::observe(vessel(&bench));
    assert!(look.words.contains("white"), "{}", look.words);
}

#[test]
fn aluminium_hydroxide_is_amphoteric_and_redissolves_in_excess_lye() {
    let (bench, steps) = run(&[
        "add v1 water 7mL",
        "add v1 Al2(SO4)3 0.0001mol",
        "add v1 NaOH 0.0006mol",
        "add v1 NaOH 0.005mol",
    ]);
    // The aluminate ceiling (~126 x [OH-], module docs) is two orders above
    // the ~0.03 mol/kgw of aluminium, so no solid survives.
    let left = solid(&bench, "Al(OH)3");
    assert!(left < 1e-6, "Al(OH)3 left {left}: {steps:#?}");
    assert!(molality(&bench, "Al(OH)4-") > 0.02);
    assert!((element(&bench, "Al") - 0.0002).abs() < 1e-9);
}

#[test]
fn aluminium_hydroxide_mostly_stays_in_buffered_excess_ammonia() {
    // 0.0006 mol NH3 throws it down and leaves 0.0006 mol NH4+; the second
    // 0.0006 mol makes a 1:1 buffer near pH 9.25. Neither routed database
    // defines an aluminium ammine, so only hydroxide acts: ~22 of 200 umol
    // may dissolve as aluminate (module docs); most stays.
    let (bench, steps) = run(&[
        "add v1 water 10mL",
        "add v1 Al2(SO4)3 0.0001mol",
        "add v1 NH3 0.0006mol",
        "add v1 NH3 0.0006mol",
    ]);
    let down = solid(&bench, "Al(OH)3");
    assert!(down > 1.5e-4, "Al(OH)3 {down}: {steps:#?}");
    assert!((element(&bench, "Al") - 0.0002).abs() < 1e-9);
}

#[test]
fn nickel_hydroxide_is_green_and_stays_in_excess_lye() {
    let (bench, steps) = run(&[
        "add v1 water 7mL",
        "add v1 Ni(OH)2 0.0001mol",
        "add v1 NaOH 0.005mol",
    ]);
    // At most ~3 umol as Ni(OH)3- on minteq.v4, far less on wateq4f.
    let down = solid(&bench, "Ni(OH)2");
    assert!(down > 0.95e-4, "Ni(OH)2 {down}: {steps:#?}");
    assert!((element(&bench, "Ni") - 0.0001).abs() < 1e-9);
    let look = kerotakis_core::appearance::observe(vessel(&bench));
    assert!(look.words.contains("green"), "{}", look.words);
}

/// On the bench nickel(II) hydroxide dissolves in excess ammonia as the
/// blue-violet hexaammine, Ni(NH3)6+2. No routed database carries it:
/// minteq.v4.dat stops at Ni(NH3)2+2 (NIST46.4, log K -13.598 on the NH4+
/// basis), which with `Ni(OH)2` 12.794 gives `Ni(OH)2 + 2 NH4+ =
/// Ni(NH3)2+2 + 2 H2O` log K -0.80 — sub-micromolar nickel at the few mM
/// of NH4+ in 3 M ammonia. llnl.dat defines Ni(NH3)6+2 but is not routed.
/// Ignored rather than inverted: the expectation is the real chemistry,
/// and it waits on routed hexaammine data.
#[test]
#[ignore = "routed databases lack Ni(NH3)6+2 (minteq.v4 stops at Ni(NH3)2+2)"]
fn nickel_hydroxide_redissolves_in_excess_ammonia_to_the_hexaammine() {
    let (bench, steps) = run(&[
        "add v1 water 10mL",
        "add v1 Ni(OH)2 0.0001mol",
        "add v1 NH3 0.03mol",
    ]);
    let left = solid(&bench, "Ni(OH)2");
    assert!(left < 1e-5, "Ni(OH)2 left {left}: {steps:#?}");
    assert!((element(&bench, "Ni") - 0.0001).abs() < 1e-9);
}
