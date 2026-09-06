//! Lactic acid, and the yoghurt it makes.
//!
//! None of the three databases this lab loads defines a lactate species,
//! so the commonest acid a kitchen makes had no acidity: the carboxylic
//! proton was absent from every pH, and a fermented milk was refused a
//! characterisation rather than reported without its only acid.
//! `databases::minteq_v4()` adds one reviewed definition, and these are
//! the claims that buys.

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn stack() -> SolverStack {
    SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
        PhreeqcEquilibrator::new().expect("engine"),
    )]))
}

fn ferment_yoghurt() -> (Bench, VesselId) {
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    for op in ["add v1 milk 100mL", "add v1 yoghurt_culture 1g", "wait 8h"] {
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
    (bench, v)
}

fn moles(bench: &Bench, v: VesselId, key: &str) -> f64 {
    bench.vessel(v).unwrap().moles_of(&SpeciesId::new(key)).0
}

/// The acid the bacteria made must still be in the beaker.
///
/// This is the test that would have stopped the first version shipping.
/// Giving `lactic_acid` a derived role turns it from a portion the tail
/// ignores into one the tail partitions into an element total — and if
/// that total does not come back, the acid's mass leaves the ledger
/// silently. It did: the element went in at 0.0038 mol and returned 0.0,
/// and the vessel weighed 0.3 g less than the same vessel on main, which
/// is 0.0038 mol x 90 g/mol to the tenth of a gram.
///
/// The cause was that `minteq.v4.dat` ends with `END`, and PHREEQC stops
/// reading a database there. The extension was appended AFTER it — not a
/// block the engine rejects loudly, one it never sees.
#[test]
fn the_acid_the_fermentation_made_stays_in_the_ledger() {
    let (bench, v) = ferment_yoghurt();
    let acid = moles(&bench, v, "lactic_acid");
    let anion = moles(&bench, v, "lactate");
    assert!(
        (acid + anion - 0.0038).abs() < 2e-4,
        "the fermentation's 0.0038 mol is now {acid} acid + {anion} anion"
    );
    // And it is genuinely SPLIT, not booked wholly as one form: at pH 3.8
    // against pKa 3.86 the two are within a factor of two of each other.
    assert!(
        acid > 0.0 && anion > 0.0,
        "both forms should carry some of it: {acid} acid, {anion} anion"
    );
}

/// The point of the exercise: a fermented milk is acidic, and it is
/// acidic to a number rather than to an apology.
///
/// Real yoghurt is pH 4.4–4.6 and this reads BELOW that, on purpose and
/// with the reason known: milk's serum minerals are in the recipe
/// (citrate, phosphate, K/Na/Ca/Cl) but casein's buffering is not — it
/// stays in the unresolved fraction — so the bench under-reads a real
/// beaker. The recipe calls any yoghurt pH a lower bound and this test
/// pins it as one, against BOTH ends: an unbuffered lactic acid solution
/// at this concentration would be pH 2.6, so the serum buffer is doing
/// real work, and the remaining gap to 4.4 is the protein.
#[test]
fn yoghurt_is_acidic_and_the_number_is_a_lower_bound() {
    let (bench, v) = ferment_yoghurt();
    let ph = bench
        .vessel(v)
        .unwrap()
        .solution
        .clone()
        .expect("a fermented milk must be characterised")
        .ph;
    assert!(
        ph > 3.0 && ph < 4.4,
        "yoghurt should land between the unbuffered 2.6 and the real 4.4, got {ph}"
    );
}

/// Fresh milk, for contrast and as a control on the recipe: near neutral,
/// and the fermentation is what moves it.
#[test]
fn the_fermentation_is_what_acidifies_the_milk() {
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    bench
        .step_with(
            kerotakis_core::script::parse_op("add v1 milk 100mL")
                .expect("parse")
                .expect("a known verb"),
            &mut stack,
            &PermissiveScreen,
        )
        .expect("step");
    let fresh = bench.vessel(v).unwrap().solution.clone().expect("milk").ph;
    assert!(
        (6.4..=7.0).contains(&fresh),
        "fresh milk is near neutral, got {fresh}"
    );

    let (fermented_bench, fv) = ferment_yoghurt();
    let soured = fermented_bench
        .vessel(fv)
        .unwrap()
        .solution
        .clone()
        .expect("yoghurt")
        .ph;
    assert!(
        fresh - soured > 2.0,
        "the culture should drop the pH by more than two units: {fresh} to {soured}"
    );
}
