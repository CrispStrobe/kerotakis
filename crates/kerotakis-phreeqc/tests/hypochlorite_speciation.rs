//! Bleach, and the alkalinity it was refused.
//!
//! `aq-053` asks whether diluted bleach stays alkaline. The bench used to
//! stand aside and tell the learner that "no thermodynamic database defines
//! a hypochlorite species … and the `ClO-` matches are all perchlorate".
//! Both halves were false. `vendor/iphreeqc/database/llnl.dat` is vendored
//! in this repository and carries, at line 107,
//! `Cl(1)     ClO-      0         Cl` — perchlorate is `Cl(7)`, three lines
//! below — and at line 4493 `H+ + ClO- = HClO`, `log_k 7.5692`.
//!
//! That constant is the whole answer. `databases::minteq_v4()` borrows it
//! the way it already borrows lactate, and these are the claims that buys.

#![cfg(feature = "engine")]

use kerotakis_core::*;
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

fn moles(bench: &Bench, v: VesselId, key: &str) -> f64 {
    bench.vessel(v).unwrap().moles_of(&SpeciesId::new(key)).0
}

fn ph(bench: &Bench, v: VesselId) -> f64 {
    bench
        .vessel(v)
        .unwrap()
        .solution
        .clone()
        .expect("a bleach solution must be characterised")
        .ph
}

/// `aq-053`, as the corpus writes it: 0.005 mol of bleach in 500 mL.
///
/// 0.01 mol/L of the conjugate base of an acid with pKa 7.57 hydrolyses to
/// [OH⁻] = √(K_b·C) = √(10^−6.43 × 0.01) = 6.1 × 10⁻⁵, which is pH 9.79 —
/// the textbook arithmetic, and what a bottle of household bleach diluted
/// for surface cleaning actually reads. The window here is wide enough to
/// survive activity corrections and narrow enough to fail if the couple is
/// ever dropped: without it the solution is sodium ion and charge balance,
/// which is NaOH at 0.01 M and reads pH 12.
#[test]
fn diluted_bleach_stays_alkaline() {
    let (bench, v) = run(&["add v1 water 500mL", "add v1 NaOCl 0.005mol"]);
    let p = ph(&bench, v);
    assert!(
        (9.0..10.5).contains(&p),
        "0.01 mol/L hypochlorite hydrolyses to about pH 9.8, got {p}"
    );
}

/// It is alkaline BECAUSE it is buffered, which is the part a pH alone
/// does not show.
///
/// Ten times more bleach is only about half a unit more alkaline — the
/// square-root dependence of a weak base's hydrolysis. A strong base at ten
/// times the concentration moves a full unit. This is the test that would
/// notice hypochlorite being mistaken for hydroxide somewhere upstream: a
/// bench that booked the bleach's alkalinity as free OH⁻ would answer pH 12
/// and pH 13 to these two beakers.
#[test]
fn the_couple_buffers_rather_than_titrates() {
    let (weak, v) = run(&["add v1 water 500mL", "add v1 NaOCl 0.005mol"]);
    let (strong, w) = run(&["add v1 water 500mL", "add v1 NaOCl 0.05mol"]);
    let (dilute, x) = run(&["add v1 water 500mL", "add v1 NaOH 0.005mol"]);
    let (concentrated, y) = run(&["add v1 water 500mL", "add v1 NaOH 0.05mol"]);

    let bleach_step = ph(&strong, w) - ph(&weak, v);
    let alkali_step = ph(&concentrated, y) - ph(&dilute, x);
    assert!(
        (0.3..0.75).contains(&bleach_step),
        "a tenfold dose of a weak base moves half a unit, got {bleach_step}"
    );
    // Not quite the full unit the arithmetic promises — 0.90 rather than
    // 1.00 — because at 0.1 mol/L the activity coefficient has started to
    // matter. That is the solver being right about a real solution rather
    // than reciting pOH = −log C, and the bound is set below it on
    // purpose: what this test is for is the CONTRAST with the 0.5 above,
    // not the third decimal of either number.
    assert!(
        alkali_step > 0.85,
        "the same dose of a strong base moves close to a full unit, got {alkali_step}"
    );
}

/// The mass stays in the ledger, split between the two members by the
/// solve's own arithmetic rather than by a curated guess.
///
/// This is the check that would have caught the lactate mistake described
/// in `lib.rs`: an extension appended after a database's trailing `END` is
/// never read, the element goes in and comes back as exactly zero, and the
/// substance's mass leaves the vessel in silence.
#[test]
fn the_bleach_is_still_in_the_beaker_afterwards() {
    let (bench, v) = run(&["add v1 water 500mL", "add v1 NaOCl 0.005mol"]);
    let anion = moles(&bench, v, "ClO-");
    let acid = moles(&bench, v, "HClO");
    assert!(
        (anion + acid - 0.005).abs() < 1e-5,
        "0.005 mol of hypochlorite is now {anion} anion + {acid} acid"
    );
    // Overwhelmingly the anion: pH 9.8 is more than two units above pKa
    // 7.57, so the acid form is under a percent of it. It is NOT zero, and
    // that matters — the protonation split exists so the ledger can say
    // which member the beaker holds, and a beaker that held only ever one
    // of them would not need one.
    assert!(
        anion > 0.99 * (anion + acid) && acid > 0.0,
        "alkaline bleach is almost all anion, with some acid: {anion} / {acid}"
    );
    // And the sodium came with it, as sodium ion, which is what makes the
    // charge balance produce a hydrolysis rather than an acid.
    assert!(
        (moles(&bench, v, "Na+") - 0.005).abs() < 1e-5,
        "the sodium books as Na+: {}",
        moles(&bench, v, "Na+")
    );
}

/// Acidify it and the couple swings the other way, which is the mechanism
/// behind the one thing everybody is told never to do.
///
/// Below pKa 7.57 the undissociated acid dominates, and hypochlorous acid
/// is the form that goes on to make chlorine. The bench does not derive
/// that step — the chlorine is curated and stays curated — but the ledger
/// now says which species the curated step would be acting on, instead of
/// calling both of them "bleach".
///
/// Sulfuric acid rather than hydrochloric, deliberately: this test is
/// about the couple, and adding a chloride would hand the vessel the
/// curated `ClO⁻ + Cl⁻ + 2 H⁺` row and evolve the hypochlorite away as
/// chlorine before the speciation could be read.
#[test]
fn acid_turns_the_anion_into_the_undissociated_acid() {
    let (bench, v) = run(&[
        "add v1 water 500mL",
        "add v1 NaOCl 0.005mol",
        "add v1 H2SO4 0.0025mol",
    ]);
    let anion = moles(&bench, v, "ClO-");
    let acid = moles(&bench, v, "HClO");
    assert!(
        acid > anion,
        "under acid the couple is mostly HClO: {anion} anion / {acid} acid"
    );
    assert!(
        (anion + acid - 0.005).abs() < 1e-5,
        "and none of it has gone missing: {anion} + {acid}"
    );
}

/// What the borrowed constant does NOT buy, stated as a test so that the
/// limit is checked rather than promised in a comment.
///
/// llnl.dat also writes `Cl- + 0.5 O2 = ClO-` at log K −15.1, which would
/// make hypochlorite a redox state of chlorine and let an open beaker's
/// atmospheric pe decide how much of the bleach has already reduced itself
/// to chloride. `HYPOCHLORITE_EXTENSION` deliberately does not borrow that
/// half: `Hypochlorite` is its own element with no redox partner, exactly
/// as `Lactate` is. So bleach standing in water stays bleach, which is what
/// a bottle of it does for months, and nothing here claims to know how
/// strongly it oxidises.
#[test]
fn standing_in_water_does_not_reduce_the_bleach_to_chloride() {
    let (bench, v) = run(&[
        "add v1 water 500mL",
        "add v1 NaOCl 0.005mol",
        "wait 8h",
        "stir v1",
    ]);
    assert!(
        (moles(&bench, v, "ClO-") + moles(&bench, v, "HClO") - 0.005).abs() < 1e-5,
        "the hypochlorite must not decay into chloride on its own"
    );
    assert!(
        moles(&bench, v, "Cl-") < 1e-6,
        "no chloride should appear: {}",
        moles(&bench, v, "Cl-")
    );
}

/// The bleaching demonstrations survive the change, in BOTH orders.
///
/// `curated` runs before the aqueous tail, so a curated row spelled on the
/// bottle key can only fire on the step the bottle is added. Before this
/// change that cost nothing, because nothing renamed `NaOCl`; now the tail
/// speciates it, and `th-077` — bleach into the beaker first, pigment
/// afterwards — would have gone silent. The sibling rows written on `ClO⁻`
/// are what keep it working, and this test pins the order that used to be
/// safe by accident.
#[test]
fn bleach_decolourises_a_pigment_whichever_goes_in_first() {
    for script in [
        // th-077's order: the bleach is already in solution.
        [
            "add v1 water 100mL",
            "add v1 NaOCl 0.01mol",
            "add v1 betanin 0.001mol",
        ],
        // bio-094's order: the bleach arrives last.
        [
            "add v1 water 100mL",
            "add v1 betanin 0.001mol",
            "add v1 NaOCl 0.005mol",
        ],
    ] {
        let (bench, v) = run(&script);
        assert!(
            moles(&bench, v, "betanin") < 1e-9,
            "the pigment must be destroyed in this order too: {script:?}"
        );
        assert!(
            moles(&bench, v, "betanin_ox") > 9e-4,
            "and its oxidised form must appear: {script:?}"
        );
    }
}
