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

/// Element totals (with H⁺ released to the ledger as charge counted back)
/// and net charge of everything in the vessel except water's own atoms.
fn ledger(v: &Vessel) -> (std::collections::BTreeMap<String, f64>, f64) {
    let mut el = std::collections::BTreeMap::new();
    let mut charge = 0.0;
    for p in &v.contents {
        if p.species.0 == "water" {
            continue;
        }
        let text = species::lookup_key(&p.species.0)
            .map(|d| d.formula)
            .unwrap_or(p.species.0.as_str());
        let f = stoich::parse_formula(text).unwrap();
        for (e, n) in f.counts {
            *el.entry(e).or_insert(0.0) += n * p.moles.0;
        }
        charge += f.charge * p.moles.0;
    }
    (el, charge)
}

fn water_h_o(v: &Vessel) -> f64 {
    m(v, "water", Phase::Liquid)
}

#[test]
fn bleach_and_sodium_sulfite_give_sulfate_and_chloride() {
    let mut v = water();
    aq(&mut v, "NaOCl", 0.001);
    aq(&mut v, "Na2SO3", 0.0004);
    let (el0, q0) = ledger(&v);
    let w0 = water_h_o(&v);
    let eq = react(&mut v);
    assert!(!eq.is_empty());
    assert!(m(&v, "Na2SO3", Phase::Aqueous) < 1e-15);
    assert!((m(&v, "NaOCl", Phase::Aqueous) - 0.0006).abs() < 1e-12);
    assert!((m(&v, "SO4-2", Phase::Aqueous) - 0.0004).abs() < 1e-12);
    assert!((m(&v, "Cl-", Phase::Aqueous) - 0.0004).abs() < 1e-12);
    // OCl⁻ + SO₃²⁻ → Cl⁻ + SO₄²⁻ moves no water and no protons.
    assert!((water_h_o(&v) - w0).abs() < 1e-12);
    assert_eq!(displacement::unspent_acidity(&v), 0.0);
    let (el1, q1) = ledger(&v);
    assert!((q1 - q0).abs() < 1e-12, "charge {q0} -> {q1}");
    for (e, n) in &el0 {
        assert!(
            (el1.get(e).copied().unwrap_or(0.0) - n).abs() < 1e-12,
            "{e}"
        );
    }
}

#[test]
fn hypochlorous_acid_takes_thiosulfate_to_sulfate() {
    let mut v = water();
    // HClO limiting, so no excess is left to meet the released acid and
    // chloride (the curated HClO + Cl⁻ + H⁺ row).
    aq(&mut v, "HClO", 0.0008);
    aq(&mut v, "Na2S2O3", 0.0004);
    let w0 = water_h_o(&v);
    let eq = react(&mut v);
    assert_eq!(
        eq,
        vec!["4 HClO + Na₂S₂O₃ + H₂O → 4 Cl⁻ + 2 SO₄²⁻ + 2 Na⁺ + 6 H⁺"]
    );
    assert!((m(&v, "Na2S2O3", Phase::Aqueous) - 0.0002).abs() < 1e-12);
    assert!(m(&v, "HClO", Phase::Aqueous) < 1e-15);
    assert!((m(&v, "SO4-2", Phase::Aqueous) - 0.0004).abs() < 1e-12);
    assert!((m(&v, "Cl-", Phase::Aqueous) - 0.0008).abs() < 1e-12);
    assert!((w0 - water_h_o(&v) - 0.0002).abs() < 1e-12);
    // The six released protons per extent are the ledger's strong acid.
    assert!((displacement::unspent_acidity(&v) - 0.0012).abs() < 1e-9);
    assert_eq!(m(&v, "S4O6-2", Phase::Aqueous), 0.0);
}

#[test]
fn hypochlorous_acid_oxidises_ascorbic_acid() {
    let mut v = water();
    aq(&mut v, "HClO", 0.0003);
    aq(&mut v, "ascorbic_acid", 0.001);
    let eq = react(&mut v);
    assert_eq!(
        eq,
        vec!["HClO + C₆H₈O₆ → Cl⁻ + C₆H₆O₆ + H₂O + H⁺"
            .replace("C₆H₈O₆", &halogen_redox::pretty("ascorbic_acid"))
            .replace("C₆H₆O₆", &halogen_redox::pretty("dehydroascorbic_acid"))]
    );
    assert!(m(&v, "HClO", Phase::Aqueous) < 1e-15);
    assert!((m(&v, "ascorbic_acid", Phase::Aqueous) - 0.0007).abs() < 1e-12);
    assert!((m(&v, "dehydroascorbic_acid", Phase::Aqueous) - 0.0003).abs() < 1e-12);
    assert!((m(&v, "Cl-", Phase::Aqueous) - 0.0003).abs() < 1e-12);
}

fn note_keys(v: &Vessel) -> Vec<String> {
    halogen_redox::unpriced_notes(v)
        .into_iter()
        .filter_map(|e| match e {
            Event::NotYetModeled {
                reason: Some(r), ..
            } => Some(r.key),
            _ => None,
        })
        .collect()
}

#[test]
fn bleach_and_sulfite_in_ethanol_are_named_not_reacted() {
    let mut v = Vessel::new(VesselId(0), "ethanol");
    v.deposit(SpeciesId::new("ethanol"), Moles(1.7), Phase::Liquid);
    v.deposit(SpeciesId::new("NaOCl"), Moles(0.001), Phase::Solid);
    v.deposit(SpeciesId::new("Na2SO3"), Moles(0.001), Phase::Solid);
    aq(&mut v, "Na2SO3", 0.0001);
    assert!(react(&mut v).is_empty());
    assert_eq!(m(&v, "SO4-2", Phase::Aqueous), 0.0);
    assert_eq!(note_keys(&v), vec!["not-modeled.redox-pair-outside-medium"]);
    // The honesty tail carries it.
    let events = HonestyEquilibrator.equilibrate(&mut v).unwrap();
    assert!(events.iter().any(|e| matches!(e,
        Event::NotYetModeled { reason: Some(r), .. } if r.key == "not-modeled.redox-pair-outside-medium")));
}

#[test]
fn permanganate_and_ascorbic_acid_are_named_not_silent() {
    let mut v = water();
    aq(&mut v, "MnO4-", 0.0002);
    aq(&mut v, "K+", 0.0002);
    aq(&mut v, "ascorbic_acid", 0.001);
    assert!(react(&mut v).is_empty());
    let notes = halogen_redox::unpriced_notes(&v);
    assert_eq!(notes.len(), 1, "one event per step");
    match &notes[0] {
        Event::NotYetModeled {
            reason: Some(r), ..
        } => {
            assert_eq!(r.key, "not-modeled.redox-pair-unpriced");
            let pairs = r.text_slot("pairs").unwrap();
            assert!(pairs.contains("MnO₄⁻"), "{pairs}");
        }
        other => panic!("{other:?}"),
    }
    let pair = halogen_redox::pairs()
        .iter()
        .find(|p| p.oxidant == "MnO4-" && p.reductant == "DHA/ascorbic_acid")
        .unwrap();
    assert_eq!(pair.status, PairStatus::NoReviewedCouple);
}

#[test]
fn covered_pairs_raise_no_note() {
    // Admitted (acid-limited here), owned by a curated row, or nothing at all.
    let mut v = water();
    aq(&mut v, "ClO-", 0.0005);
    aq(&mut v, "Na+", 0.0005);
    aq(&mut v, "I-", 0.002);
    aq(&mut v, "K+", 0.002);
    assert!(note_keys(&v).is_empty());
    let mut v = water();
    aq(&mut v, "I2", 0.0005);
    aq(&mut v, "ascorbic_acid", 0.001);
    assert!(note_keys(&v).is_empty(), "curated I2 + C6H8O6 owns it");
    assert!(note_keys(&water()).is_empty());
}

#[test]
fn equilibrium_limited_iron_three_and_iodide_raise_no_note() {
    // Owned by the aqueous equilibrium solver, not unpriced.
    let mut v = water();
    aq(&mut v, "Fe+3", 0.001);
    aq(&mut v, "I-", 0.003);
    assert!(note_keys(&v).is_empty());
}
