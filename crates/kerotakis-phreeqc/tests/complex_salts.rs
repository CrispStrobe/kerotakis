//! Complex salts and their colours (2026-11-25): one table-driven route.
//!
//! * Hexacyanoferrate(II)/(III) dissolve to conserved anions (minteq.v4
//!   slice `shipped-complex-borrows`); their insoluble salts are database
//!   phases with cited solubility products, so the iron(III) test gives
//!   Prussian blue and copper(II) gives the brown hexacyanoferrate.
//! * Native complexes take their colour from `complex_optics` rows, so the
//!   tetraammine copper solution is blue and no longer a spectral gap.
//!
//! * Iron(II) with hexacyanoferrate(III) (Turnbull's blue) gives the same
//!   Prussian blue by electron transfer.
//!
//! Negatives: hexacyanoferrate(III) with iron(III) and hexacyanoferrate(II)
//! alone throw nothing down; a dilute acid does not open the anion.
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
fn iron_three_with_hexacyanoferrate_two_gives_prussian_blue() {
    let (bench, steps) = run(&[
        "add v1 water 10mL",
        "add v1 FeCl3 0.0004mol",
        "add v1 K4Fe(CN)6 0.0003mol",
        "wait 10s",
    ]);
    let pb = solid(&bench, "Prussian_blue");
    assert!(pb > 0.9e-4, "Prussian blue {pb}: {steps:#?}");
    // Iron, carbon and nitrogen are conserved through the precipitate.
    assert!((element(&bench, "Fe") - 0.0007).abs() < 1e-9);
    assert!((element(&bench, "N") - 0.0018).abs() < 1e-9);
    let look = kerotakis_core::appearance::observe(vessel(&bench));
    assert!(look.words.contains("blue"), "{}", look.words);
}

/// C1R-6: Turnbull's blue is Prussian blue. Iron(II) with
/// hexacyanoferrate(III) transfers an electron (core potential-redox route,
/// driven by the precipitate) and the solve throws down the same phase.
#[test]
fn iron_two_with_hexacyanoferrate_three_gives_turnbulls_blue_which_is_prussian_blue() {
    let (bench, steps) = run(&[
        "add v1 water 10mL",
        "add v1 FeSO4 0.0004mol",
        "add v1 K3Fe(CN)6 0.0003mol",
        "wait 10s",
    ]);
    let pb = solid(&bench, "Prussian_blue");
    assert!(pb > 0.5e-4, "Prussian blue {pb}: {steps:#?}");
    assert!((element(&bench, "Fe") - 0.0007).abs() < 1e-9);
    assert!((element(&bench, "N") - 0.0018).abs() < 1e-9);
    let look = kerotakis_core::appearance::observe(vessel(&bench));
    assert!(look.words.contains("blue"), "{}", look.words);
}

#[test]
fn copper_two_with_hexacyanoferrate_two_gives_the_brown_salt() {
    let (bench, _) = run(&[
        "add v1 water 10mL",
        "add v1 CuSO4 0.0004mol",
        "add v1 K4Fe(CN)6 0.0002mol",
        "wait 10s",
    ]);
    assert!(solid(&bench, "Cu2Fe(CN)6") > 1.8e-4);
}

#[test]
fn hexacyanoferrate_alone_or_with_its_own_iron_three_throws_nothing_down() {
    for script in [
        &[
            "add v1 water 10mL",
            "add v1 K4Fe(CN)6 0.0003mol",
            "wait 10s",
        ][..],
        &[
            "add v1 water 10mL",
            "add v1 FeCl3 0.0004mol",
            "add v1 K3Fe(CN)6 0.0003mol",
            "wait 10s",
        ],
    ] {
        let (bench, _) = run(script);
        assert!(solid(&bench, "Prussian_blue") < 1e-12, "{script:?}");
        let provenance = vessel(&bench)
            .solution
            .as_ref()
            .and_then(|s| s.provenance.as_ref())
            .map(|p| p.dataset.clone())
            .unwrap_or_default();
        assert!(provenance.contains("minteq"), "{provenance}");
    }
}

#[test]
fn a_dilute_acid_does_not_open_the_hexacyanoferrate_anion() {
    let (bench, _) = run(&[
        "add v1 water 10mL",
        "add v1 K4Fe(CN)6 0.0002mol",
        "add v1 HCl 0.001mol",
        "wait 10s",
    ]);
    let anion = molality(&bench, "Fe(CN)6-4")
        + molality(&bench, "HFe(CN)6-3")
        + molality(&bench, "H2Fe(CN)6-2");
    assert!(anion > 0.015, "intact anion {anion}");
    assert!(molality(&bench, "Fe+2") < 1e-9, "no free iron released");
    assert!(
        !vessel(&bench)
            .contents
            .iter()
            .any(|p| p.species.0.contains("HCN")),
        "no hydrogen cyanide"
    );
}

#[test]
fn tetraammine_copper_is_blue_and_not_a_spectral_gap() {
    let (bench, _) = run(&[
        "add v1 water 5mL",
        "add v1 CuSO4 0.0006mol",
        "add v1 NH3 0.01mol",
        "wait 10s",
    ]);
    assert!(molality(&bench, "Cu(NH3)4+2") > 0.05);
    let look = kerotakis_core::appearance::observe(vessel(&bench));
    assert!(
        !look.spectral_gaps.iter().any(|g| g == "Cu(NH3)4+2"),
        "{:?}",
        look.spectral_gaps
    );
    assert!(look.words.contains("blue"), "{}", look.words);
    assert!(
        !look.words.starts_with("The liquid is colourless"),
        "{}",
        look.words
    );
}

#[test]
fn iron_thiocyanate_is_red() {
    let (bench, _) = run(&[
        "add v1 water 10mL",
        "add v1 FeCl3 0.0001mol",
        "add v1 KSCN 0.0005mol",
        "wait 10s",
    ]);
    let look = kerotakis_core::appearance::observe(vessel(&bench));
    assert!(
        look.words.contains("red") || look.words.contains("orange"),
        "{}",
        look.words
    );
    assert!(
        !look.spectral_gaps.iter().any(|g| g.contains("SCN")),
        "{:?}",
        look.spectral_gaps
    );
}

#[test]
fn zinc_ammine_solution_is_colourless_and_dissolves_the_hydroxide() {
    let (bench, _) = run(&[
        "add v1 water 10mL",
        "add v1 ZnSO4 0.0005mol",
        "add v1 NH3 0.01mol",
        "wait 10s",
    ]);
    assert!(molality(&bench, "Zn(NH3)4+2") > 0.01, "tetraammine zinc");
    let look = kerotakis_core::appearance::observe(vessel(&bench));
    assert!(look.words.contains("colourless"), "{}", look.words);
}
