//! Sulfite, and the pH it moves.
//!
//! `vendor/iphreeqc/database/llnl.dat` line 231 is
//! `S(+4)     SO3-2     0         S` and line 4591 is
//! `SO3-2 + H+ = HSO3-`, `log_k 7.2054`. `databases::minteq_v4()` borrows
//! that one constant as the pseudo-element `Sulfite`, the way it already
//! borrows lactate, hypochlorite and thiosulfate.
//!
//! **This borrow is different from the three before it, and the harvest
//! below corrected the first explanation of why.** Their acid constants
//! sit outside the range a bench works in - thiosulfate's pKa is 1.01 - so
//! speciating them moved no vessel's pH. Sulfite's is 7.20, and the
//! tempting inference is that a sulfite beaker is therefore half anion and
//! half acid. IT IS NOT. A salt of a weak acid hydrolyses: 0.05 M sodium
//! sulfite settles at pH 9.70, two and a half units above the pKa, and is
//! 99.86 % `SO3-2`. That measurement was made before these assertions were
//! written, and it falsified the comment that had already been written
//! around it.
//!
//! What is true, and what the `PROTONATION_SPLITS` row is really for, is
//! that this bench stocks TWO sulfite bottles and they sit on OPPOSITE
//! sides of the constant - `Na2SO3` at pH 9.70, `NaHSO3` at pH 4.17 - so
//! no single booking ion serves both. And acidifying a sulfite solution
//! puts both forms in one vessel after all.
//!
//! What DID move, and it is the largest move of the four borrows: a vessel
//! holding sodium sulfite had no pH at all before this change, because the
//! salt was flagged `dissolves_without_speciation`. It now reads 9.70.

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
        .expect("a sulfite solution must be characterised")
        .ph
}

fn moles(bench: &Bench, v: VesselId, key: &str) -> f64 {
    bench.vessel(v).unwrap().moles_of(&SpeciesId::new(key)).0
}

/// Harvest, no assertions: the numbers the commit message and the PR body
/// quote come from here rather than from anybody's expectation. Run with
/// `--nocapture`.
#[test]
fn what_the_sulfite_beakers_read() {
    for (label, script) in [
        (
            "Na2SO3 0.005 mol in 100 mL",
            vec!["add v1 water 100mL", "add v1 Na2SO3 0.63g"],
        ),
        (
            "NaHSO3 0.005 mol in 100 mL",
            vec!["add v1 water 100mL", "add v1 NaHSO3 0.52g"],
        ),
        (
            "Na2SO3 with a little acid",
            vec![
                "add v1 water 100mL",
                "add v1 Na2SO3 0.63g",
                "add v1 HCl 0.002mol",
            ],
        ),
    ] {
        let (bench, v) = run(&script);
        let vessel = bench.vessel(v).unwrap();
        eprintln!(
            "HARVEST {label}: pH {:.6}  I {:?}  SO3-2 {:.8}  HSO3- {:.8}  Na+ {:.8}  Na2SO3 {:.8}  NaHSO3 {:.8}",
            ph(&bench, v),
            vessel.solution.as_ref().map(|s| s.ionic_strength),
            moles(&bench, v, "SO3-2"),
            moles(&bench, v, "HSO3-"),
            moles(&bench, v, "Na+"),
            moles(&bench, v, "Na2SO3"),
            moles(&bench, v, "NaHSO3"),
        );
    }
}

/// The vessel is characterised at all, and the bottle name is gone.
#[test]
fn a_sulfite_solution_speciates_instead_of_sitting_there() {
    let (bench, v) = run(&["add v1 water 100mL", "add v1 Na2SO3 0.63g"]);
    assert!(
        bench.vessel(v).unwrap().solution.is_some(),
        "sodium sulfite must now have a solution"
    );
    assert!(
        moles(&bench, v, "Na2SO3") == 0.0,
        "the bottle name does not survive a solve"
    );
    let total = moles(&bench, v, "SO3-2") + moles(&bench, v, "HSO3-");
    assert!(
        (total - 0.005).abs() < 3e-4,
        "the sulfur is conserved across the split, got {total}"
    );
}

/// THE TEST THE THREE EARLIER BORROWS COULD NOT HAVE WRITTEN, and not the
/// one I first wrote.
///
/// The first version of this asserted that a sulfite beaker holds both
/// forms in comparable amounts, on the reasoning that pKa 7.20 is inside
/// the bench's range. It failed, correctly: the beaker is 99.86 % dianion,
/// because the salt hydrolyses to pH 9.70 and a solution two and a half
/// units above a pKa is not a mixture.
///
/// The property that IS true, and that actually justifies the
/// `PROTONATION_SPLITS` row, is that the two bottles straddle the
/// constant. Whichever single ion were chosen as the booking ion, the
/// other bottle would be booked as a species it is barely any of.
#[test]
fn the_two_bottles_land_on_opposite_sides_of_the_constant() {
    let (basic, v1) = run(&["add v1 water 100mL", "add v1 Na2SO3 0.63g"]);
    let (acidic, v2) = run(&["add v1 water 100mL", "add v1 NaHSO3 0.52g"]);

    let basic_ph = ph(&basic, v1);
    let acidic_ph = ph(&acidic, v2);
    assert!(
        basic_ph > 9.0,
        "sodium sulfite hydrolyses and is alkaline, got {basic_ph}"
    );
    assert!(
        acidic_ph < 5.0,
        "sodium bisulfite is acidic, got {acidic_ph}"
    );
    assert!(
        basic_ph > acidic_ph + 4.0,
        "the two bottles must straddle pKa 7.20, got {basic_ph} and {acidic_ph}"
    );

    // Each bottle is dominated by a DIFFERENT ion, which is the whole
    // argument for the split.
    let dianion_share =
        moles(&basic, v1, "SO3-2") / (moles(&basic, v1, "SO3-2") + moles(&basic, v1, "HSO3-"));
    let acid_share =
        moles(&acidic, v2, "HSO3-") / (moles(&acidic, v2, "SO3-2") + moles(&acidic, v2, "HSO3-"));
    assert!(
        dianion_share > 0.95,
        "the sulfite bottle is essentially all dianion, got {dianion_share}"
    );
    assert!(
        acid_share > 0.95,
        "the bisulfite bottle is essentially all monoanion, got {acid_share}"
    );
}

/// And acidifying a sulfite solution DOES put both forms in one vessel,
/// which is the case a single booking ion would misreport even if the
/// bench stocked only one bottle.
#[test]
fn an_acidified_sulfite_solution_really_holds_both_forms() {
    let (bench, v) = run(&[
        "add v1 water 100mL",
        "add v1 Na2SO3 0.63g",
        "add v1 HCl 0.002mol",
    ]);
    let anion = moles(&bench, v, "SO3-2");
    let acid = moles(&bench, v, "HSO3-");
    let minor = anion.min(acid) / (anion + acid);
    assert!(
        minor > 0.2,
        "half-neutralised sulfite is a real mixture, got SO3-2 {anion} and HSO3- {acid}"
    );
    let buffered = ph(&bench, v);
    assert!(
        (buffered - 7.04).abs() < 0.4,
        "and it buffers near the pKa rather than following the acid, got {buffered}"
    );
}

/// The pseudo-element claim, made falsifiable exactly as thiosulfate's is.
///
/// Entered as sulfur's `S(+4)` the engine would have air-oxidised the
/// bottle to sulfate, and for sulfite that is not a modelling artefact but
/// the substance's headline chemistry.
#[test]
fn an_open_beaker_does_not_oxidise_the_bottle_to_sulfate() {
    let (bench, v) = run(&["add v1 water 100mL", "add v1 Na2SO3 0.63g", "wait 600s"]);
    let total = moles(&bench, v, "SO3-2") + moles(&bench, v, "HSO3-");
    assert!(
        total > 0.0049,
        "sulfur(IV) must still be sulfur(IV) after ten minutes, got {total}"
    );
    assert_eq!(
        moles(&bench, v, "SO4-2"),
        0.0,
        "nothing may quietly oxidise it to sulfate"
    );
}

/// The matcher is exact, and the near-miss is a real registry formula.
#[test]
fn the_sulfite_matcher_is_exact() {
    match derived::role("SO3-2") {
        Some(DerivedRole::Dissolves(els)) => {
            assert!(els.iter().any(|(e, n)| e == "Sulfite" && *n == 1.0));
        }
        other => panic!("sulfite books its own element, got {other:?}"),
    }
    match derived::role("NaHSO3") {
        Some(DerivedRole::Dissolves(els)) => {
            assert!(els.iter().any(|(e, n)| e == "Sulfite" && *n == 1.0));
            assert!(els.iter().any(|(e, n)| e == "Na" && *n == 1.0));
        }
        other => panic!("the bisulfite salt books sulfite plus its sodium, got {other:?}"),
    }
    // `methyl_orange` is `C14 H14 N3 Na1 O3 S1` - exactly one sulfur and
    // three oxygens, the only registry formula besides the two sulfites
    // that passes the count. A greedy `("SO3", "Sulfite")` group row would
    // have booked an azo dye's sulfonate as a bottle of sulfite; the exact
    // rule refuses it on its carbon and nitrogen.
    match derived::role("methyl_orange") {
        Some(DerivedRole::Dissolves(els)) => assert!(
            !els.iter().any(|(e, _)| e == "Sulfite"),
            "the dye is not a sulfite: {els:?}"
        ),
        _ => {}
    }
    assert_eq!(derived::booking_ion("Sulfite"), Some("SO3-2"));
}
