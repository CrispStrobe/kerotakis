//! The soluble-salt class (A2, 2026-11-21): common ionic salts added to
//! water dissolve into their ions, hydrolyse where the routed database says
//! they do, and either price their heat from a sourced dissolution enthalpy
//! or say by name that it is unpriced. Before this lane KBr, CuCl2, Na3PO4
//! and Al2(SO4)3 were unknown species and aluminium had no booking ion.
//!
//! Expected numbers, by arithmetic stated here rather than by snapshot:
//! - 1 g KBr (8.40 mmol) in 100 mL: neutral, +19.88 kJ/mol (NBS/Wagman 1982)
//!   absorbs about 167 J, so the beaker cools by about 0.4 K.
//! - 1 g CuCl2 (7.44 mmol): Cu2+ hydrolysis makes it acidic (pH 3-5);
//!   -49.45 kJ/mol releases about 368 J, warming it by about 0.9 K.
//! - 1 g Na3PO4 (6.10 mmol): PO4-3 + H2O = HPO4-2 + OH-, pH 11.5-12.8.
//! - 1 g Al2(SO4)3 (2.92 mmol): Al3+ hydrolysis, pH 3-4.5;
//!   -350.16 kJ/mol releases about 1.0 kJ, warming it by about 2-3 K.
//! - 1 g BaSO4 stays (almost entirely) solid: the negative control.
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
                .expect("step"),
        );
    }
    (bench, steps)
}

fn salt(formula: &str) -> (Bench, Vec<Vec<Event>>) {
    let add = format!("add v1 {formula} 1g");
    run(&["add v1 water 100mL", &add, "wait 10s"])
}

fn moles(bench: &Bench, key: &str) -> f64 {
    bench
        .vessel(VesselId(0))
        .unwrap()
        .moles_of(&SpeciesId::new(key))
        .0
}

fn ph(bench: &Bench) -> f64 {
    bench
        .vessel(VesselId(0))
        .unwrap()
        .solution
        .clone()
        .expect("a salt solution is characterised")
        .ph
}

fn celsius(bench: &Bench) -> f64 {
    bench.vessel(VesselId(0)).unwrap().temperature.to_celsius()
}

fn noted(steps: &[Vec<Event>], key: &str) -> bool {
    steps.iter().flatten().any(|event| {
        matches!(event, Event::NotYetModeled { reason: Some(reason), .. } if reason.key == key)
    })
}

const UNPRICED: &str = "not-modeled.aqueous-heat-unpriced";

#[test]
fn potassium_bromide_dissolves_neutral_and_cools() {
    let (bench, steps) = salt("KBr");
    let n = 1.0 / 119.0023;
    assert!(moles(&bench, "KBr") < 1e-12, "{steps:#?}");
    assert!((moles(&bench, "K+") - n).abs() < 1e-9 * n.max(1.0));
    assert!((moles(&bench, "Br-") - n).abs() < 1e-9 * n.max(1.0));
    assert!((6.5..7.5).contains(&ph(&bench)), "pH {}", ph(&bench));
    let t = celsius(&bench);
    assert!((24.4..24.8).contains(&t), "endothermic dissolution: {t} C");
    assert!(!noted(&steps, UNPRICED), "the heat is priced: {steps:#?}");
}

#[test]
fn copper_chloride_dissolves_acidic_and_warms() {
    let (bench, steps) = salt("CuCl2");
    let n = 1.0 / 134.446;
    assert!(moles(&bench, "CuCl2") < 1e-12, "{steps:#?}");
    let cl = moles(&bench, "Cl-");
    assert!((cl - 2.0 * n).abs() < 1e-3 * n, "chloride kept: {cl}");
    // A trace of basic copper chloride may form at the database's own
    // saturation; copper is conserved between the ion and that solid.
    let cu = moles(&bench, "Cu+2") + 2.0 * moles(&bench, "atacamite");
    assert!((cu - n).abs() < 1e-6 * n.max(1.0), "copper kept: {cu}");
    assert!(moles(&bench, "Cu+2") > 0.99 * n);
    assert!((3.0..5.0).contains(&ph(&bench)), "pH {}", ph(&bench));
    let t = celsius(&bench);
    assert!((25.5..26.3).contains(&t), "exothermic dissolution: {t} C");
}

#[test]
fn trisodium_phosphate_dissolves_strongly_basic() {
    let (bench, steps) = salt("Na3PO4");
    let n = 1.0 / 163.9391;
    assert!(moles(&bench, "Na3PO4") < 1e-12, "{steps:#?}");
    assert!((moles(&bench, "Na+") - 3.0 * n).abs() < 1e-9);
    let p: f64 = ["H3PO4", "H2PO4-", "HPO4-2", "PO4-3"]
        .iter()
        .map(|k| moles(&bench, k))
        .sum();
    assert!((p - n).abs() < 1e-9, "phosphorus conserved: {p}");
    assert!(moles(&bench, "HPO4-2") > 0.1 * n, "phosphate hydrolyses");
    assert!((11.5..12.8).contains(&ph(&bench)), "pH {}", ph(&bench));
    // Either the heat is priced or the step says so by name; never silent.
    let t = celsius(&bench);
    assert!(
        noted(&steps, UNPRICED) || (t - 25.0).abs() > 0.1,
        "a phosphate dissolution neither priced nor declared: {t} C"
    );
}

#[test]
fn aluminium_sulfate_dissolves_acidic_and_warms() {
    let (bench, steps) = salt("Al2(SO4)3");
    let n = 1.0 / 342.1311;
    assert!(moles(&bench, "Al2(SO4)3") < 1e-12, "{steps:#?}");
    assert!((moles(&bench, "Al+3") - 2.0 * n).abs() < 1e-9);
    assert!((moles(&bench, "SO4-2") - 3.0 * n).abs() < 1e-9);
    assert!((3.0..4.5).contains(&ph(&bench)), "pH {}", ph(&bench));
    let t = celsius(&bench);
    assert!((26.5..28.5).contains(&t), "exothermic dissolution: {t} C");
}

#[test]
fn barium_sulfate_stays_solid() {
    let (bench, _) = salt("BaSO4");
    let n = 1.0 / 233.39;
    assert!(
        moles(&bench, "BaSO4") > 0.99 * n,
        "insoluble salt stays solid"
    );
    // Barite dissolves to about 1e-5 M: 1 umol of 4.3 mmol here.
    assert!(moles(&bench, "Ba+2") < 1e-3 * n);
}

#[test]
fn corundum_is_not_dissolved_by_the_aluminium_booking() {
    // Aluminium joined the residue elements but not the oxide-as-water
    // cations, so the insoluble oxide keeps its old behaviour.
    let (bench, _) = salt("Al2O3");
    assert!(moles(&bench, "Al2O3") > 0.99 / 101.961);
    assert!(moles(&bench, "Al+3") < 1e-12);
}
