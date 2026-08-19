//! The pitzer routing arm (concentrated brines get the ion-interaction
//! model) and polyprotic phosphate chemistry — three pKa's from the
//! database, walked with a burette.

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn add(bench: &mut Bench, eq: &mut PhreeqcEquilibrator, v: VesselId, key: &str, moles: f64) {
    bench
        .step_with(
            Operator::Add {
                vessel: v,
                species: SpeciesId::new(key),
                moles: Moles(moles),
                at: None,
            },
            eq,
            &PermissiveScreen,
        )
        .expect("step");
}

fn ph(bench: &Bench, v: VesselId) -> f64 {
    bench
        .vessel(v)
        .unwrap()
        .solution
        .clone()
        .expect("characterised")
        .ph
}

#[test]
fn pitzer_gets_halite_solubility_right() {
    // The regime that broke minteq (3.7) and stretched wateq4f: the Pitzer
    // ion-interaction model is built for brines and lands on the real
    // solubility, ~6.15 mol/kgw.
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    add(&mut bench, &mut eq, v, "water", 55.51); // 1.000 kg
    add(&mut bench, &mut eq, v, "NaCl", 8.0);

    let vessel = bench.vessel(v).unwrap();
    let dissolved = vessel.moles_of(&SpeciesId::new("Na+")).0;
    assert!(
        (dissolved - 6.15).abs() < 0.3,
        "Pitzer halite solubility is ~6.15 mol/kgw, got {dissolved}"
    );
}

#[test]
fn dilute_solutions_stay_on_wateq4f() {
    // Routing sanity: a dilute AgNO3 problem contains Ag, which pitzer.dat
    // does not know — it must go through wateq4f and still work.
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    add(&mut bench, &mut eq, v, "water", 55.51);
    add(&mut bench, &mut eq, v, "AgNO3", 0.01);
    let info = bench
        .vessel(v)
        .unwrap()
        .solution
        .clone()
        .expect("characterised");
    assert!((info.ph - 7.0).abs() < 0.5);
}

#[test]
fn phosphoric_acid_is_a_strong_ish_first_proton() {
    // 0.1 m H3PO4: pKa1 = 2.15 makes the first proton come off readily —
    // pH ≈ 1.6, between a strong acid (1.0) and acetic (2.9).
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    add(&mut bench, &mut eq, v, "water", 55.51);
    add(&mut bench, &mut eq, v, "H3PO4", 0.1);
    let ph = ph(&bench, v);
    assert!(
        (ph - 1.6).abs() < 0.25,
        "0.1 m phosphoric acid should be pH ~1.6, got {ph}"
    );
}

#[test]
fn phosphate_titration_reads_the_second_pka() {
    // 1.5 equivalents of base: halfway between the first and second
    // equivalence points, the solution is an H2PO4-/HPO4-2 buffer sitting
    // at pKa2 ≈ 7.2 — the buffer that runs biology.
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    add(&mut bench, &mut eq, v, "water", 55.51);
    add(&mut bench, &mut eq, v, "H3PO4", 0.1);
    // Not the thermodynamic pKa2 = 7.20: at I ≈ 0.25 the divalent
    // HPO4-2's activity coefficient (~0.4) drags the *conditional* pKa
    // down to ~6.65 — the difference between the textbook constant and
    // what a pH meter actually reads in a real buffer.
    add(&mut bench, &mut eq, v, "NaOH", 0.15);
    let ph1 = ph(&bench, v);
    assert!(
        (ph1 - 6.65).abs() < 0.35,
        "1.5-equivalent point reads the conditional pKa2 (~6.65 at this I), got {ph1}"
    );

    // Push to the second equivalence: distinctly basic.
    add(&mut bench, &mut eq, v, "NaOH", 0.05);
    let ph2 = ph(&bench, v);
    assert!(
        ph2 > 9.0 && ph2 < 10.5,
        "second equivalence of phosphoric acid is ~9.7, got {ph2}"
    );
}
