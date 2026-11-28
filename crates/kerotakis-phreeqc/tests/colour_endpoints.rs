//! Gap S1 (2026-11-28 sweep): the colour endpoints are judged by what the
//! titrant does to the flask, not by "the colour word changed".
//!
//! `until colour persists` ends when the TITRANT's own colour survives in
//! the flask; a colour the titrant produces in the analyte (iron(III)
//! from iron(II)) is not that. `until colour disappears` ends when the
//! colour the titrant consumes (iodine, with or without starch) no longer
//! shows. Both volumes are checked against the stoichiometry, and both
//! endpoints are shown not to fire before equivalence.

#![cfg(feature = "engine")]

use kerotakis_core::script::parse_op;
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn stack() -> SolverStack {
    let aqueous = PhreeqcEquilibrator::new().expect("engine");
    SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(aqueous)]))
}

fn line(bench: &mut Bench, stack: &mut SolverStack, text: &str) -> Vec<Event> {
    let events = bench
        .step_with(
            parse_op(text).expect("parse").expect("operator"),
            stack,
            &PermissiveScreen,
        )
        .unwrap_or_else(|e| panic!("{text}: {e}"));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::SolverFailed { .. })),
        "{text}: solver failure: {events:?}"
    );
    events
}

/// Run a setup script and then one titration; return (mL, reached, bench).
fn titrate(setup: &[&str], titration: &str) -> (f64, bool, Bench) {
    let mut bench = Bench::new();
    let mut s = stack();
    for text in setup {
        line(&mut bench, &mut s, text);
    }
    let events = line(&mut bench, &mut s, titration);
    let (ml, reached) = events
        .iter()
        .find_map(|e| match e {
            Event::Titrated {
                total_volume,
                endpoint_reached,
                ..
            } => Some((total_volume.0 * 1000.0, endpoint_reached.unwrap_or(false))),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no titration event: {events:?}"));
    (ml, reached, bench)
}

// ── Iron(II) with permanganate: the titrant's own colour persists ──

const FE_MOL: f64 = 0.0025;
const MNO4_M: f64 = 0.02;
/// MnO4- + 5 Fe2+ + 8 H+ -> Mn2+ + 5 Fe3+ + 4 H2O: one permanganate per
/// five iron(II), so 2.5 mmol Fe(II) takes 0.5 mmol, 25.0 mL at 0.02 M.
const FE_STOICHIOMETRIC_ML: f64 = FE_MOL / 5.0 / MNO4_M * 1000.0;
const FE_SETUP: &[&str] = &[
    "add v1 water 50mL",
    "add v1 H2SO4 0.01mol",
    "add v1 FeSO4 0.0025mol",
];

/// Past equivalence the excess permanganate must stay permanganate. The
/// coupled redox solve used to refuse that state (its pe lies above the
/// stability field of water) and fall back to an uncoupled solve pinned to
/// atmospheric oxygen, which reduced every manganese to Mn(II) by
/// oxidising water — chemistry that needs a catalyst and does not happen
/// in a titration flask (`WATER_INERT_OXIDANTS`).
#[test]
fn iron_ii_with_permanganate_stops_at_the_stoichiometric_volume() {
    assert!((FE_STOICHIOMETRIC_ML - 25.0).abs() < 1e-9);
    assert_iron_endpoint("titrate v1 KMnO4 0.02M 0.5mL until colour persists max 80");
}

/// The same titration in 1 mL increments. It used to fail outright at
/// 17 mL ("coupled aqueous temperature did not converge within 64
/// passes"): the electron balance was located only to 1e-4 of the
/// inventory, so its reaction heat wandered by ~1e-4 K between thermal
/// passes that must agree to 1e-6 K.
#[test]
fn iron_ii_with_permanganate_in_one_millilitre_steps() {
    assert_iron_endpoint("titrate v1 KMnO4 0.02M 1mL until colour persists max 40");
}

fn assert_iron_endpoint(titration: &str) {
    let (ml, reached, bench) = titrate(FE_SETUP, titration);
    assert!(reached, "endpoint must be reached, stopped at {ml:.2} mL");
    assert!(
        (ml - FE_STOICHIOMETRIC_ML).abs() <= 1.0,
        "endpoint at {ml:.2} mL, stoichiometric {FE_STOICHIOMETRIC_ML:.2} mL"
    );
    // Not before equivalence: the burette has delivered at least the
    // permanganate the iron needs.
    assert!(
        ml + 1e-9 >= FE_STOICHIOMETRIC_ML,
        "fired early at {ml:.2} mL"
    );
    let iron_ii: f64 = bench
        .vessel(VesselId(0))
        .expect("vessel")
        .contents
        .iter()
        .filter(|p| p.species.0 == "Fe+2" || p.species.0 == "iron(II) ion")
        .map(|p| p.moles.0)
        .sum();
    assert!(
        iron_ii < 0.01 * FE_MOL,
        "iron(II) must be consumed at the endpoint, {iron_ii:.3e} mol left"
    );
    // The excess the endpoint saw is permanganate still in the flask.
    let permanganate: f64 = amount(&bench, "MnO4-");
    assert!(
        permanganate > 0.0,
        "past equivalence permanganate must remain, found none"
    );
}

fn amount(bench: &Bench, species: &str) -> f64 {
    bench
        .vessel(VesselId(0))
        .expect("vessel")
        .contents
        .iter()
        .filter(|p| p.species.0 == species)
        .map(|p| p.moles.0)
        .sum()
}

/// Negative control: holding permanganate inert toward WATER must not
/// hold it inert toward iron(II). Short of equivalence the fast couple
/// still equilibrates completely — every permanganate is reduced to
/// Mn(II), five iron(III) for each, and none is left purple.
#[test]
fn below_equivalence_permanganate_still_reacts_completely_with_iron_ii() {
    let mut bench = Bench::new();
    let mut s = stack();
    for text in FE_SETUP {
        line(&mut bench, &mut s, text);
    }
    line(&mut bench, &mut s, "add v1 KMnO4 0.0002mol");
    let manganese_ii = amount(&bench, "Mn+2");
    let permanganate = amount(&bench, "MnO4-");
    let iron_iii = amount(&bench, "Fe+3");
    assert!(
        (manganese_ii - 2e-4).abs() < 1e-6,
        "Mn(II) {manganese_ii:.4e} mol, expected 2.0e-4"
    );
    assert!(permanganate < 1e-9, "{permanganate:.3e} mol MnO4- left");
    assert!(
        (iron_iii - 1e-3).abs() < 1e-5,
        "Fe(III) {iron_iii:.4e} mol, expected 1.0e-3"
    );
}

#[test]
fn the_iron_iii_colour_does_not_count_as_the_permanganate_colour() {
    // 80 increments of 0.25 mL is 20 mL, short of 25 mL: the flask turns
    // from iron(II) to iron(III) the whole way, and that change of colour
    // is not the titrant's.
    let (ml, reached, _) = titrate(
        FE_SETUP,
        "titrate v1 KMnO4 0.02M 0.25mL until colour persists max 80",
    );
    assert!(
        !reached,
        "the endpoint must not fire before equivalence (fired at {ml:.2} mL)"
    );
    assert!((ml - 20.0).abs() < 1e-6, "{ml}");
}

// ── Iodine with thiosulfate: the consumed colour disappears ──

const I2_MOL: f64 = 0.001;
const S2O3_M: f64 = 0.1;
/// I2 + 2 S2O3^2- -> 2 I- + S4O6^2-: 1 mmol iodine takes 2 mmol
/// thiosulfate, 20.0 mL at 0.1 M.
const I2_STOICHIOMETRIC_ML: f64 = 2.0 * I2_MOL / S2O3_M * 1000.0;
const I2_SETUP: &[&str] = &[
    "add v1 water 50mL",
    "add v1 I2 0.001mol",
    "add v1 KI 0.005mol",
];

fn with_starch() -> Vec<&'static str> {
    let mut setup = I2_SETUP.to_vec();
    setup.push("add v1 starch 0.0005mol");
    setup
}

fn iodine_left(bench: &Bench) -> f64 {
    bench
        .vessel(VesselId(0))
        .expect("vessel")
        .contents
        .iter()
        .filter(|p| p.species.0 == "I2")
        .map(|p| p.moles.0)
        .sum()
}

#[test]
fn iodine_with_thiosulfate_decolourises_at_the_stoichiometric_volume() {
    assert!((I2_STOICHIOMETRIC_ML - 20.0).abs() < 1e-9);
    let mut volumes = Vec::new();
    for (label, setup) in [("no starch", I2_SETUP.to_vec()), ("starch", with_starch())] {
        let (ml, reached, bench) = titrate(
            &setup,
            "titrate v1 Na2S2O3 0.1M 0.5mL until colour disappears max 60",
        );
        assert!(reached, "{label}: endpoint must be reached ({ml:.2} mL)");
        assert!(
            (ml - I2_STOICHIOMETRIC_ML).abs() <= 1.0,
            "{label}: endpoint at {ml:.2} mL, stoichiometric {I2_STOICHIOMETRIC_ML:.2} mL"
        );
        assert!(
            ml + 1e-9 >= I2_STOICHIOMETRIC_ML - 0.5,
            "{label}: fired early at {ml:.2} mL"
        );
        let left = iodine_left(&bench);
        assert!(
            left < 0.05 * I2_MOL,
            "{label}: {left:.3e} mol iodine left at the endpoint"
        );
        volumes.push(ml);
    }
    // Starch is an indicator, not a reagent: it must not move the endpoint.
    assert!(
        (volumes[0] - volumes[1]).abs() <= 0.5 + 1e-9,
        "starch moved the endpoint: {volumes:?}"
    );
}

#[test]
fn the_starch_iodine_colour_does_not_disappear_before_equivalence() {
    // 30 increments of 0.5 mL is 15 mL, short of 20 mL: the blue-black
    // complex fades but iodine remains, and the endpoint must not fire.
    let (ml, reached, bench) = titrate(
        &with_starch(),
        "titrate v1 Na2S2O3 0.1M 0.5mL until colour disappears max 30",
    );
    assert!(
        !reached,
        "the endpoint must not fire before equivalence (fired at {ml:.2} mL)"
    );
    assert!(iodine_left(&bench) > 0.1 * I2_MOL);
}

#[test]
fn a_colourless_titrant_never_persists() {
    // Thiosulfate brings no colour of its own, so "until colour persists"
    // cannot end on the starch-iodine colour fading — the S1 symptom was
    // exactly that, at 11 mL with 0.45 mmol iodine left.
    let (ml, reached, bench) = titrate(
        &with_starch(),
        "titrate v1 Na2S2O3 0.1M 1mL until colour persists max 15",
    );
    assert!(!reached, "fired at {ml:.2} mL");
    assert!(iodine_left(&bench) > 0.2 * I2_MOL);
}
