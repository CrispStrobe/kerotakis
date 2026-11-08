//! E1: reagents added as solutions with a concentration carry their water.

use kerotakis_core::script::parse_op;
use kerotakis_core::solution_stock::{self, SolutionDensityBasis};
use kerotakis_core::*;

fn solution(line: &str) -> Operator {
    parse_op(line)
        .unwrap_or_else(|e| panic!("{line}: {e}"))
        .unwrap_or_else(|| panic!("{line}: no operator"))
}

fn pinned(op: &Operator) -> (f64, f64) {
    match op {
        Operator::AddSolution {
            solute_moles,
            water_moles,
            ..
        } => (solute_moles.0, water_moles.0),
        other => panic!("expected AddSolution, got {other:?}"),
    }
}

#[test]
fn every_grammar_form_parses_to_a_solution() {
    for line in [
        "add v1 HCl 50mL 37%",
        "add v1 HCl 50mL 37%w/w",
        "add v1 HCl 50mL 37% w/w",
        "add v1 HCl 50mL 37% (w/w)",
        "add v1 HCl 0.1mol as 1M",
        "add v1 NaOH 100mL 1M",
        "add v1 H2O2 20g 30%",
        "add v1 HCl 50mL 37% @ 10C",
        "add v1 hydrochloric_acid_37_percent 50mL",
        "add v1 sodium_hydroxide_1_molar 100mL @ 30C",
    ] {
        let (solute, water) = pinned(&solution(line));
        assert!(solute > 0.0 && water > 0.0, "{line}");
    }
    // Glacial acetic acid is the pure liquid: no water arrives.
    let (_, water) = pinned(&solution("add v1 glacial_acetic_acid 10mL"));
    assert_eq!(water, 0.0);
    // A bare amount keeps its old meaning: pure substance.
    assert!(matches!(
        solution("add v1 HCl 0.1mol"),
        Operator::Add { .. }
    ));
}

#[test]
fn malformed_concentrations_are_refused() {
    for line in [
        "add v1 HCl 10mL 120%",
        "add v1 HCl 10mL 0%",
        "add v1 HCl 10mL -5%",
        "add v1 HCl 10mL -1M",
        "add v1 HCl -10mL 37%",
        "add v1 HCl 10mL as",
        "add v1 HCl 1cup 1M",
        "add v1 HCl 10mL 45%",
        "add v1 NaCl 100mL 1M",
        "add v1 hydrochloric_acid_37_percent 1spoon",
    ] {
        assert!(parse_op(line).is_err(), "{line} should be refused");
    }
}

#[test]
fn ten_ml_of_37_percent_hcl_matches_density_volume_fraction() {
    let op = solution("add v1 HCl 10mL 37%");
    let (rho, source) = solution_stock::density_at("HCl", 0.37).unwrap();
    assert_eq!(source, "crc-concentrative-properties");
    let hcl = species::lookup_key("HCl").unwrap();
    let water = species::lookup_key("water").unwrap();
    let expect_hcl = 10.0 * rho * 0.37 / hcl.molar_mass;
    let expect_water = 10.0 * rho * 0.63 / water.molar_mass;
    let (n_hcl, n_water) = pinned(&op);
    assert!((n_hcl - expect_hcl).abs() < 1e-9);
    assert!((n_water - expect_water).abs() < 1e-9);
    assert!(matches!(
        &op,
        Operator::AddSolution {
            density_basis: SolutionDensityBasis::Reviewed { .. },
            ..
        }
    ));

    let mut bench = Bench::new();
    let events = bench.step(op).expect("the bench admits a solution dose");
    let added: Vec<(String, f64)> = events
        .iter()
        .filter_map(|e| match e {
            Event::Added { species, moles, .. } => Some((species.0.clone(), moles.0)),
            _ => None,
        })
        .collect();
    assert_eq!(added.len(), 2);
    assert!(added
        .iter()
        .any(|(s, n)| s == "HCl" && (n - expect_hcl).abs() < 1e-9));
    assert!(added
        .iter()
        .any(|(s, n)| s == "water" && (n - expect_water).abs() < 1e-9));
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::SolutionAdded { .. })));
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::NotYetModeled { .. } | Event::SafetyVeto { .. })),
        "aqueous HCl must be admitted: {events:?}"
    );
}

#[test]
fn a_solution_round_trips_through_the_saved_log() {
    let op = solution("add v1 HCl 0.1mol as 1M");
    let json = serde_json::to_string(&op).unwrap();
    assert!(json.contains("\"op\":\"add_solution\""));
    let back: Operator = serde_json::from_str(&json).unwrap();
    assert_eq!(back, op);
}
