//! Hydroxide precipitates redissolving in excess ligand (2026-11-27).
//!
//! A metal hydroxide thrown down by a little base goes back into solution
//! when the excess base is itself a complexing ligand: copper(II) and zinc
//! hydroxides in excess ammonia (tetraammine complexes), zinc hydroxide in
//! excess lye (zincate, Zn(OH)4 2-). Nothing here is a scenario yield: the
//! routed databases' own mass action decides, and the fresh precipitate is
//! the databases' amorphous hydroxide (`derived::fresh_precipitate_rank`),
//! not an aged crystal.
//!
//! Negative: iron(III) hydroxide forms no ammine and stays down in excess
//! ammonia.
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
fn copper_hydroxide_redissolves_in_excess_ammonia_to_the_tetraammine() {
    let (before, _) = run(&[
        "add v1 water 10mL",
        "add v1 CuSO4 0.0001mol",
        "add v1 NaOH 0.0002mol",
    ]);
    assert!(solid(&before, "Cu(OH)2") > 0.9e-4, "precipitate first");
    let (bench, steps) = run(&[
        "add v1 water 10mL",
        "add v1 CuSO4 0.0001mol",
        "add v1 NaOH 0.0002mol",
        "add v1 NH3 0.03mol",
    ]);
    let left = solid(&bench, "Cu(OH)2");
    assert!(left < 1e-6, "Cu(OH)2 left {left}: {steps:#?}");
    // ~0.01 mol/kgw of copper, most of it the tetraammine.
    let ammine = molality(&bench, "Cu(NH3)4+2");
    assert!(ammine > 0.005, "Cu(NH3)4+2 {ammine}");
    assert!((element(&bench, "Cu") - 0.0001).abs() < 1e-9);
    let look = kerotakis_core::appearance::observe(vessel(&bench));
    assert!(look.words.contains("blue"), "{}", look.words);
}

#[test]
fn zinc_hydroxide_redissolves_in_excess_ammonia() {
    let (bench, steps) = run(&[
        "add v1 water 10mL",
        "add v1 ZnSO4 0.0001mol",
        "add v1 NaOH 0.0002mol",
        "add v1 NH3 0.02mol",
    ]);
    let left = solid(&bench, "Zn(OH)2");
    assert!(left < 1e-6, "Zn(OH)2 left {left}: {steps:#?}");
    assert!(molality(&bench, "Zn(NH3)4+2") > 0.003);
    assert!((element(&bench, "Zn") - 0.0001).abs() < 1e-9);
}

#[test]
fn zinc_hydroxide_is_amphoteric_and_redissolves_in_excess_lye() {
    let (before, _) = run(&[
        "add v1 water 7mL",
        "add v1 ZnSO4 0.0002mol",
        "add v1 NaOH 0.0004mol",
    ]);
    assert!(solid(&before, "Zn(OH)2") > 1.5e-4, "precipitate first");
    let (bench, steps) = run(&[
        "add v1 water 7mL",
        "add v1 ZnSO4 0.0002mol",
        "add v1 NaOH 0.0004mol",
        "add v1 NaOH 0.005mol",
    ]);
    let left = solid(&bench, "Zn(OH)2");
    assert!(left < 2e-5, "Zn(OH)2 left {left}: {steps:#?}");
    assert!(molality(&bench, "Zn(OH)4-2") > 0.01);
    assert!((element(&bench, "Zn") - 0.0002).abs() < 1e-9);
}

#[test]
fn iron_three_hydroxide_does_not_redissolve_in_excess_ammonia() {
    let (bench, steps) = run(&[
        "add v1 water 10mL",
        "add v1 FeCl3 0.0001mol",
        "add v1 NH3 0.0003mol",
        "add v1 NH3 0.03mol",
    ]);
    let down = solid(&bench, "Fe(OH)3");
    assert!(down > 0.99e-4, "Fe(OH)3 {down}: {steps:#?}");
    assert!((element(&bench, "Fe") - 0.0001).abs() < 1e-9);
}
