//! Halogen / oxyhalogen redox family (2026-10-27).
//!
//! `halogen_redox_coverage_is_current` pins the runtime classification to
//! the generated table `docs/halogen-redox-coverage-20261027.json`
//! (`python3 tools/halogen_redox_coverage.py` regenerates it from llnl.dat,
//! the registry and the curated/kinetic tables). The remaining tests are
//! positive, negative and boundary controls for the admitted writers.

use kerotakis_core::halogen_redox::{self, PairStatus};
use kerotakis_core::solve::Equilibrator;
use kerotakis_core::*;

const TABLE: &str = include_str!("../../../docs/halogen-redox-coverage-20261027.json");

#[test]
fn halogen_redox_coverage_is_current() {
    let doc: serde_json::Value = serde_json::from_str(TABLE).unwrap();
    assert_eq!(
        doc["complete_margin_v"].as_f64().unwrap(),
        halogen_redox::COMPLETE_MARGIN_V
    );
    let halves = doc["halves"].as_array().unwrap();
    assert_eq!(halves.len(), halogen_redox::HALVES.len());
    for (json, half) in halves.iter().zip(halogen_redox::HALVES) {
        assert_eq!(json["id"], half.id);
        assert_eq!(json["oxidised"], half.oxidised);
        assert_eq!(
            json["electrons"].as_f64().unwrap(),
            half.electrons,
            "{}",
            half.id
        );
        assert_eq!(
            json["protons"].as_f64().unwrap(),
            half.protons,
            "{}",
            half.id
        );
        assert_eq!(json["water"].as_f64().unwrap(), half.water, "{}", half.id);
        assert!(
            (json["e0_v"].as_f64().unwrap() - half.e0_v).abs() < 1e-9,
            "{} E° drifted from the generated table",
            half.id
        );
        assert_eq!(
            json["cross_check"].as_bool().unwrap(),
            half.cross_check,
            "{}",
            half.id
        );
    }
    let pairs = doc["pairs"].as_array().unwrap();
    let runtime = halogen_redox::pairs();
    assert_eq!(pairs.len(), runtime.len());
    for (json, pair) in pairs.iter().zip(runtime) {
        let label = format!("{} x {}", pair.oxidant, pair.reductant);
        assert_eq!(json["oxidant"], pair.oxidant, "{label}");
        assert_eq!(json["reductant"], pair.reductant, "{label}");
        assert_eq!(json["status"], pair.status.key(), "{label}");
        let forms = |v: &serde_json::Value| -> Vec<String> {
            v.as_array()
                .unwrap()
                .iter()
                .map(|s| s.as_str().unwrap().to_string())
                .collect()
        };
        assert_eq!(forms(&json["oxidant_forms"]), pair.oxidant_forms, "{label}");
        assert_eq!(
            forms(&json["reductant_forms"]),
            pair.reductant_forms,
            "{label}"
        );
        match (json["delta_e_v"].as_f64(), pair.delta_e_v) {
            (Some(a), Some(b)) => assert!((a - b).abs() < 1e-9, "{label}"),
            (None, None) => {}
            other => panic!("{label}: {other:?}"),
        }
    }
    assert_eq!(halogen_redox::rejected_rows(), 0);
}

fn water() -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
    v
}

fn aq(v: &mut Vessel, key: &str, n: f64) {
    v.deposit(SpeciesId::new(key), Moles(n), Phase::Aqueous);
}

fn react(v: &mut Vessel) -> Vec<String> {
    v.solute_charge = displacement::solute_charge(v);
    CuratedEquilibrator
        .equilibrate(v)
        .unwrap()
        .into_iter()
        .filter_map(|e| match e {
            Event::ReactionOccurred { equation, .. } => Some(equation),
            _ => None,
        })
        .collect()
}

fn m(v: &Vessel, key: &str, phase: Phase) -> f64 {
    v.contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}

#[test]
fn iodine_oxidises_sulfite_to_completion() {
    let mut v = water();
    aq(&mut v, "I2", 0.001);
    aq(&mut v, "SO3-2", 0.0004);
    aq(&mut v, "Na+", 0.0008);
    assert_eq!(displacement::unspent_acidity(&v), 0.0);
    let eq = react(&mut v);
    assert_eq!(eq, vec!["I₂ + SO₃²⁻ + H₂O → 2 I⁻ + SO₄²⁻ + 2 H⁺"]);
    assert!((m(&v, "I2", Phase::Aqueous) - 0.0006).abs() < 1e-12);
    assert!(m(&v, "SO3-2", Phase::Aqueous) < 1e-15);
    assert!((m(&v, "I-", Phase::Aqueous) - 0.0008).abs() < 1e-12);
    assert!((m(&v, "SO4-2", Phase::Aqueous) - 0.0004).abs() < 1e-12);
    // The released protons are the charge imbalance: acid.
    assert!((displacement::unspent_acidity(&v) - 0.0008).abs() < 1e-9);
}

#[test]
fn sodium_sulfite_bottle_releases_its_sodium() {
    let mut v = water();
    aq(&mut v, "I2", 0.0002);
    aq(&mut v, "Na2SO3", 0.001);
    let eq = react(&mut v);
    assert_eq!(eq, vec!["I₂ + Na₂SO₃ + H₂O → 2 I⁻ + SO₄²⁻ + 2 Na⁺ + 2 H⁺"]);
    assert!(m(&v, "I2", Phase::Aqueous) < 1e-15);
    assert!((m(&v, "Na2SO3", Phase::Aqueous) - 0.0008).abs() < 1e-12);
    assert!((m(&v, "Na+", Phase::Aqueous) - 0.0004).abs() < 1e-12);
}

#[test]
fn iodine_vapour_is_never_withdrawn_crystals_dissolve_as_consumed() {
    let mut v = water();
    v.deposit(SpeciesId::new("I2"), Moles(0.001), Phase::Gas);
    aq(&mut v, "SO3-2", 0.0004);
    aq(&mut v, "Na+", 0.0008);
    assert!(
        react(&mut v).is_empty(),
        "headspace iodine is not in solution"
    );
    assert_eq!(m(&v, "I2", Phase::Gas), 0.001);
    // Crystals in contact with the solution: dissolved share first.
    v.deposit(SpeciesId::new("I2"), Moles(0.0001), Phase::Aqueous);
    v.deposit(SpeciesId::new("I2"), Moles(0.001), Phase::Solid);
    assert_eq!(react(&mut v).len(), 1);
    assert!(m(&v, "I2", Phase::Aqueous) < 1e-15);
    assert!((m(&v, "I2", Phase::Solid) - 0.0007).abs() < 1e-12);
    assert_eq!(m(&v, "I2", Phase::Gas), 0.001);
}

#[test]
fn dry_powders_do_not_react() {
    let mut v = Vessel::new(VesselId(0), "dry");
    v.deposit(SpeciesId::new("I2"), Moles(0.001), Phase::Solid);
    v.deposit(SpeciesId::new("Na2SO3"), Moles(0.001), Phase::Solid);
    assert!(react(&mut v).is_empty());
}

#[test]
fn acidified_hypochlorite_liberates_iodine() {
    let mut v = water();
    aq(&mut v, "ClO-", 0.0005);
    aq(&mut v, "Na+", 0.0005);
    aq(&mut v, "I-", 0.002);
    aq(&mut v, "K+", 0.002);
    // Sulfuric-acid acidity booked the ledger way: anion, no cation.
    aq(&mut v, "SO4-2", 0.002);
    let eq = react(&mut v);
    assert_eq!(eq, vec!["ClO⁻ + 2 I⁻ + 2 H⁺ → Cl⁻ + I₂ + H₂O"]);
    assert!(m(&v, "ClO-", Phase::Aqueous) < 1e-15);
    assert!((m(&v, "I2", Phase::Aqueous) - 0.0005).abs() < 1e-12);
    assert!((m(&v, "I-", Phase::Aqueous) - 0.001).abs() < 1e-12);
}

#[test]
fn acid_limits_hypochlorite_iodide() {
    let mut v = water();
    aq(&mut v, "ClO-", 0.0005);
    aq(&mut v, "Na+", 0.0005);
    aq(&mut v, "I-", 0.002);
    aq(&mut v, "K+", 0.002);
    aq(&mut v, "SO4-2", 0.0001); // 0.0002 mol protons: extent 0.0001
    react(&mut v);
    assert!((m(&v, "I2", Phase::Aqueous) - 0.0001).abs() < 1e-12);
    assert!((m(&v, "ClO-", Phase::Aqueous) - 0.0004).abs() < 1e-12);
}

#[test]
fn neutral_bleach_and_iodide_do_nothing_without_acid() {
    let mut v = water();
    aq(&mut v, "ClO-", 0.0005);
    aq(&mut v, "Na+", 0.0005);
    aq(&mut v, "I-", 0.002);
    aq(&mut v, "K+", 0.002);
    assert!(react(&mut v).is_empty());
}

#[test]
fn outside_the_temperature_domain_nothing_is_written() {
    let mut v = water();
    aq(&mut v, "I2", 0.001);
    aq(&mut v, "SO3-2", 0.0004);
    aq(&mut v, "Na+", 0.0008);
    v.temperature = Kelvin(halogen_redox::DOMAIN_K.1 + 1.0);
    assert!(react(&mut v).is_empty());
    v.temperature = Kelvin(halogen_redox::DOMAIN_K.1);
    assert_eq!(react(&mut v).len(), 1);
}

#[test]
fn iron_three_and_iodide_are_left_to_an_equilibrium_solver() {
    let mut v = water();
    aq(&mut v, "Fe+3", 0.001);
    aq(&mut v, "I-", 0.003);
    assert!(react(&mut v).is_empty());
    let pair = halogen_redox::pairs()
        .iter()
        .find(|p| p.oxidant == "Fe+3/Fe+2" && p.reductant == "I2/I-")
        .unwrap();
    assert_eq!(pair.status, PairStatus::EquilibriumLimited);
}

#[test]
fn iodometry_rows_never_take_iodine_vapour() {
    let mut v = water();
    v.deposit(SpeciesId::new("I2"), Moles(0.001), Phase::Gas);
    aq(&mut v, "S2O3-2", 0.002);
    aq(&mut v, "Na+", 0.004);
    assert!(react(&mut v).is_empty(), "vapour is not titrated");
    aq(&mut v, "I2", 0.0005);
    let eq = react(&mut v);
    assert_eq!(eq, vec!["I₂ + 2 S₂O₃²⁻ → 2 I⁻ + S₄O₆²⁻"]);
    assert_eq!(m(&v, "I2", Phase::Gas), 0.001);
    assert!(m(&v, "I2", Phase::Aqueous) < 1e-15);
}
