//! Missing-species lane (2026-11-24): MgCl2, its hexahydrate bischofite,
//! CoCl2 (with the Co+2 booking ion), magnetite and urea were unknown
//! species, and `CaOH2` / `calcium hydroxide` did not reach slaked lime.
//!
//! Expected numbers, by arithmetic stated here rather than by snapshot
//! (100 mL water takes about 418 J per kelvin):
//! - 1 g MgCl2 (10.50 mmol), -159.83 kJ/mol (NBS/Wagman 1982): about
//!   1.68 kJ out, so the beaker warms by about 4 K. Mg2+ barely hydrolyses.
//! - 1 g CoCl2 (7.70 mmol), -80.0 kJ/mol: about 0.62 kJ out, about +1.5 K;
//!   Co2+ hydrolysis leaves it weakly acidic.
//! - 1 g urea (16.65 mmol), +15.09 kJ/mol: about 0.25 kJ in, about -0.6 K
//!   (priced by the molecular route of #919), and it stays a neutral molecule.
//! - 1 g bischofite dissolves to Mg2+ and 2 Cl- with nothing lost; hydrate
//!   heat is not priced, so the step must say so rather than stay silent.
//! - 1 g magnetite stays solid: the negative control.
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
fn magnesium_chloride_dissolves_and_warms() {
    let (bench, steps) = salt("MgCl2");
    let n = 1.0 / 95.205;
    assert!(moles(&bench, "MgCl2") < 1e-12, "{steps:#?}");
    assert!((moles(&bench, "Mg+2") - n).abs() < 1e-6 * n.max(1.0));
    assert!((moles(&bench, "Cl-") - 2.0 * n).abs() < 1e-6 * n.max(1.0));
    assert!((5.0..8.0).contains(&ph(&bench)), "pH {}", ph(&bench));
    let t = celsius(&bench);
    assert!((28.0..30.0).contains(&t), "exothermic dissolution: {t} C");
    assert!(!noted(&steps, UNPRICED), "the heat is priced: {steps:#?}");
}

#[test]
fn bischofite_dissolves_conserved_and_declares_its_heat() {
    let (bench, steps) = salt("bischofite");
    let n = 1.0 / 203.295;
    assert!(moles(&bench, "bischofite") < 1e-12, "{steps:#?}");
    let mg = moles(&bench, "Mg+2");
    assert!((mg - n).abs() < 1e-6 * n.max(1.0), "Mg {mg} vs {n}");
    let cl = moles(&bench, "Cl-");
    assert!((cl - 2.0 * n).abs() < 1e-6 * n.max(1.0), "Cl {cl}");
    let t = celsius(&bench);
    assert!(
        noted(&steps, UNPRICED) || (t - 25.0).abs() > 0.1,
        "a hydrate dissolution neither priced nor declared: {t} C"
    );
}

#[test]
fn cobalt_chloride_dissolves_weakly_acidic_and_warms() {
    let (bench, steps) = salt("CoCl2");
    let n = 1.0 / 129.833;
    assert!(moles(&bench, "CoCl2") < 1e-12, "{steps:#?}");
    assert!(
        (moles(&bench, "Co+2") - n).abs() < 1e-6 * n.max(1.0),
        "{steps:#?}"
    );
    assert!((moles(&bench, "Cl-") - 2.0 * n).abs() < 1e-6 * n.max(1.0));
    assert!((4.5..7.5).contains(&ph(&bench)), "pH {}", ph(&bench));
    let t = celsius(&bench);
    assert!((25.8..27.2).contains(&t), "exothermic dissolution: {t} C");
}

#[test]
fn urea_dissolves_as_a_neutral_molecule_and_cools() {
    let (bench, steps) = salt("urea");
    let n = 1.0 / 60.056;
    assert!(moles_in(&bench, "urea", Phase::Solid) < 1e-12, "{steps:#?}");
    assert!((moles(&bench, "urea") - n).abs() < 1e-9, "urea kept whole");
    assert!(moles_in(&bench, "urea", Phase::Aqueous) > 0.99 * n);
    let t = celsius(&bench);
    assert!((24.0..24.8).contains(&t), "endothermic dissolution: {t} C");
}

#[test]
fn magnetite_stays_solid() {
    let (bench, _) = salt("Fe3O4");
    assert!(moles_in(&bench, "Fe3O4", Phase::Solid) > 0.99 / 231.531);
    assert!(moles(&bench, "Fe+2") + moles(&bench, "Fe+3") < 1e-6);
}

fn added_key(line: &str) -> Option<String> {
    match kerotakis_core::script::parse_op(line) {
        Ok(Some(Operator::Add { species, .. })) => Some(species.0),
        _ => None,
    }
}

#[test]
fn slaked_lime_and_urea_spellings_resolve() {
    for spelling in ["Ca(OH)2", "CaOH2", "calcium hydroxide", "slaked lime"] {
        assert_eq!(
            added_key(&format!("add v1 {spelling} 1g")).as_deref(),
            Some("Ca(OH)2"),
            "{spelling}"
        );
    }
    assert_eq!(added_key("add v1 CO(NH2)2 1g").as_deref(), Some("urea"));
    assert_eq!(
        added_key("add v1 MgCl2·6H2O 1g").as_deref(),
        Some("bischofite")
    );
    // Negative: a near miss is not silently mapped onto slaked lime.
    assert_eq!(added_key("add v1 CaOH 1g"), None);
}
