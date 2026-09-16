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
///
/// THE NUMBER MOVED ON 2026-09-16 and the reason is not this file's. The
/// fermentation rate used to read the GRAMS of culture; it now reads their
/// concentration, against a declared one-litre reference volume, and 100 mL
/// of milk is 0.0896 L of liquid — so the same culture runs 11 times faster,
/// the same eight hours consume 53.2% of the milk's lactose instead of 6.6%,
/// and 0.0038 mol of acid became 0.0307. See `kerotakis_core::fermentation::REFERENCE_VOLUME_LITRES`. The
/// window below is wide because the magnitude behind it is editorial and
/// unestablished, not because the value is uncertain to that much.
#[test]
fn the_acid_the_fermentation_made_stays_in_the_ledger() {
    let (bench, v) = ferment_yoghurt();
    let acid = moles(&bench, v, "lactic_acid");
    let anion = moles(&bench, v, "lactate");
    assert!(
        (acid + anion - 0.0307).abs() < 2e-3,
        "the fermentation's 0.0307 mol is now {acid} acid + {anion} anion"
    );
    // And it is genuinely SPLIT, not booked wholly as one form.
    assert!(
        acid > 0.0 && anion > 0.0,
        "both forms should carry some of it: {acid} acid, {anion} anion"
    );
}

/// The point of the exercise: a fermented milk is acidic, and it is
/// acidic to a number rather than to an apology.
///
/// IT IS NOW TOO ACIDIC, AND THAT IS THE FINDING RATHER THAN A TOLERANCE
/// PROBLEM. Real yoghurt is pH 4.4-4.6. Before 2026-09-16 this read 3.89,
/// under that window for a reason the recipe states: milk's serum minerals
/// are modelled (citrate, phosphate, K/Na/Ca/Cl) but casein's buffering is
/// not, so the bench needs less acid than a real beaker to reach a given
/// pH. It now reads 2.83, and the extra unit is NOT the missing buffer: it
/// is the acid itself, eight times more of it, because the fermentation
/// rate started reading a concentration against a declared one-litre
/// reference volume and a corpus script pours 100 mL. The rate constants
/// were carried across that change unchanged rather than re-fitted, so what
/// this test now pins is that the classroom timescale is calibrated for a
/// litre and used at a tenth of one. Recalibrating it is a separate
/// decision; this window records where the bench actually is, and the
/// assertion still refuses a beaker that is not acidic at all.
#[test]
fn the_fermented_milk_is_acidic_and_now_overshoots_real_yoghurt() {
    let (bench, v) = ferment_yoghurt();
    let ph = bench
        .vessel(v)
        .unwrap()
        .solution
        .clone()
        .expect("a fermented milk must be characterised")
        .ph;
    assert!(
        (2.5..3.2).contains(&ph),
        "the over-fermented yoghurt should land near 2.83, got {ph}"
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
        fresh - soured > 3.0,
        "the culture should drop the pH by more than three units: {fresh} to {soured}"
    );
}
