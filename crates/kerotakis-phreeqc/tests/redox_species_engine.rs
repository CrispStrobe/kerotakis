//! Redox species (2026-11-25) through the full solver stack: the new salts
//! dissolve and conserve, and the routes they switch on survive the aqueous
//! solve that follows (chromium(III) books back as Cr+3).
//!
//! Expected numbers by arithmetic: 1 g Cu(NO3)2 is 5.332 mmol; 1 g K2SO4 is
//! 5.739 mmol; 1 g KHSO4 is 7.344 mmol, whose hydrogen sulfate leaves 100 mL
//! clearly acidic (pKa2 of sulfuric acid about 2); 0.01 mol HNO3 in 100 mL
//! is 0.1 mol/L strong acid, pH about 1.

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

fn salt(key: &str) -> (Bench, Vec<Vec<Event>>) {
    let add = format!("add v1 {key} 1g");
    run(&["add v1 water 100mL", &add, "wait 10s"])
}

fn moles(bench: &Bench, key: &str) -> f64 {
    bench
        .vessel(VesselId(0))
        .unwrap()
        .moles_of(&SpeciesId::new(key))
        .0
}

fn moles_in(bench: &Bench, key: &str, phase: Phase) -> f64 {
    bench
        .vessel(VesselId(0))
        .unwrap()
        .contents
        .iter()
        .filter(|p| p.species == SpeciesId::new(key) && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
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

#[test]
fn copper_nitrate_dissolves_to_copper_and_nitrate() {
    let (bench, steps) = salt("Cu(NO3)2");
    let n = 1.0 / 187.554;
    assert!(moles(&bench, "Cu(NO3)2") < 1e-12, "{steps:#?}");
    let cu = moles(&bench, "Cu+2");
    assert!(cu > 0.5 * n && cu <= n * (1.0 + 1e-9), "Cu+2 {cu} of {n}");
    assert!(ph(&bench) < 7.0, "pH {}", ph(&bench));
}

#[test]
fn potassium_sulfates_dissolve_and_the_hydrogen_salt_is_acidic() {
    let (bench, _) = salt("K2SO4");
    assert!(moles(&bench, "K2SO4") < 1e-12);
    assert!((moles(&bench, "K+") - 2.0 / 174.252).abs() < 1e-6);
    assert!((5.5..8.5).contains(&ph(&bench)), "pH {}", ph(&bench));
    let (bench, _) = salt("KHSO4");
    assert!(moles(&bench, "KHSO4") < 1e-12);
    assert!((moles(&bench, "K+") - 1.0 / 136.162).abs() < 1e-6);
    assert!(ph(&bench) < 2.6, "pH {}", ph(&bench));
}

#[test]
fn nitric_acid_is_a_strong_acid_in_water() {
    let (bench, _) = run(&["add v1 water 100mL", "add v1 HNO3 0.01mol", "wait 10s"]);
    assert!((0.8..1.3).contains(&ph(&bench)), "pH {}", ph(&bench));
}

#[test]
fn chlorine_water_turns_bromide_into_bromine_on_the_bench() {
    let (bench, steps) = run(&[
        "add v1 water 100mL",
        "add v1 KBr 0.002mol",
        "add v1 Cl2 0.0005mol",
        "stir v1",
    ]);
    let br2 = moles(&bench, "Br2");
    assert!((br2 - 0.0005).abs() < 1e-6, "Br2 {br2}: {steps:#?}");
}

#[test]
fn dichromate_and_iron_two_leave_chromium_three_after_the_solve() {
    let (bench, steps) = run(&[
        "add v1 water 100mL",
        "add v1 H2SO4 0.01mol",
        "add v1 FeSO4 0.003mol",
        "add v1 K2Cr2O7 0.0005mol",
        "stir v1",
    ]);
    let cr = moles(&bench, "Cr+3");
    assert!((cr - 0.001).abs() < 1e-6, "Cr+3 {cr}: {steps:#?}");
    assert!(moles(&bench, "K2Cr2O7") < 1e-9);
}

#[test]
fn copper_and_dilute_nitric_acid_give_nitrogen_monoxide() {
    let (bench, steps) = run(&[
        "add v1 water 20mL",
        "add v1 Cu 0.003mol",
        "add v1 HNO3 0.02mol",
        "stir v1",
    ]);
    let gone = 0.003 - moles_in(&bench, "Cu", Phase::Solid);
    let no: f64 = steps
        .iter()
        .flatten()
        .map(|e| match e {
            Event::GasEvolved { species, moles, .. } if species.0 == "NO" => moles.0,
            _ => 0.0,
        })
        .sum::<f64>()
        + moles_in(&bench, "NO", Phase::Gas);
    assert!(gone > 1e-4, "copper dissolved {gone}: {steps:#?}");
    assert!(
        (no - 2.0 / 3.0 * gone).abs() < 1e-6,
        "NO {no} for {gone} Cu"
    );
}
