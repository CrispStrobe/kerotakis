//! S(IV) reductant coupling (KI-011, 2026-11-11).
//!
//! SO₂ / HSO₃⁻ / SO₃²⁻ beside iodine or hydrogen peroxide is oxidised to
//! sulfate (Yiin & Margerum 1990, doi:10.1021/ic00333a023; McArdle &
//! Hoffmann 1983, doi:10.1021/j150644a024). `sulfur_iv_coverage_is_current`
//! pins the module's constants and forms to
//! `docs/sulfur-iv-coupling-coverage-20261111.json`
//! (`python3 tools/sulfur_iv_coupling_coverage.py` regenerates it). The rest
//! are positive and negative controls, plus the sub-observable kinetic
//! event that KI-011 found missing.

use kerotakis_core::solve::Equilibrator;
use kerotakis_core::*;
use kerotakis_core::{halogen_redox, sulfur_iv};

const TABLE: &str = include_str!("../../../docs/sulfur-iv-coupling-coverage-20261111.json");

fn water() -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    // 100 mL.
    v.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
    v
}

fn aq(v: &mut Vessel, key: &str, n: f64) {
    v.deposit(SpeciesId::new(key), Moles(n), Phase::Aqueous);
}

/// Sulfuric acid the way the ledger books it after a solve.
fn acidify(v: &mut Vessel, h2so4: f64) {
    aq(v, "SO4-2", h2so4);
    aq(v, "H+", 2.0 * h2so4);
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

fn m(v: &Vessel, key: &str) -> f64 {
    v.contents
        .iter()
        .filter(|p| p.species.0 == key)
        .map(|p| p.moles.0)
        .sum()
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
fn sulfur_iv_coverage_is_current() {
    let doc: serde_json::Value = serde_json::from_str(TABLE).unwrap();
    let k = &doc["constants"];
    assert_eq!(k["K_I2_HSO3"].as_f64().unwrap(), sulfur_iv::K_I2_HSO3);
    assert_eq!(k["K_I3_HSO3"].as_f64().unwrap(), sulfur_iv::K_I3_HSO3);
    assert_eq!(k["K_I2_SO3"].as_f64().unwrap(), sulfur_iv::K_I2_SO3);
    assert_eq!(k["K_I3_SO3"].as_f64().unwrap(), sulfur_iv::K_I3_SO3);
    assert_eq!(k["MH_K1_M1_S1"].as_f64().unwrap(), sulfur_iv::MH_K1_M1_S1);
    assert_eq!(
        k["MH_K2_OVER_KM1_M1"].as_f64().unwrap(),
        sulfur_iv::MH_K2_OVER_KM1_M1
    );
    assert_eq!(
        k["MH_DH_ACT_J_MOL"].as_f64().unwrap(),
        sulfur_iv::MH_DH_ACT_J_MOL
    );
    assert_eq!(
        k["MAX_INSTANT_S"].as_f64().unwrap(),
        sulfur_iv::MAX_INSTANT_S
    );
    let forms: Vec<(String, String)> = doc["forms"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| {
            (
                f["form"].as_str().unwrap().to_string(),
                f["ion"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    let runtime: Vec<(String, String)> = sulfur_iv::forms()
        .into_iter()
        .map(|(f, ion)| (f, ion.key.to_string()))
        .collect();
    assert_eq!(forms, runtime);
    let oxidants: Vec<&str> = doc["oxidants"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o.as_str().unwrap())
        .collect();
    assert_eq!(oxidants, sulfur_iv::OXIDANTS);
    assert_eq!(k["HC_K_M2_S1"].as_f64().unwrap(), sulfur_iv::HC_K_M2_S1);
    assert_eq!(k["HC_KEQ_M1"].as_f64().unwrap(), sulfur_iv::HC_KEQ_M1);
    // Every table oxidant: key, pricing and rate constants.
    let table: Vec<(String, String, Vec<Option<f64>>)> = doc["oxidant_table"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| {
            (
                o["oxidant"].as_str().unwrap().to_string(),
                o["pricing"].as_str().unwrap().to_string(),
                o["k_so2_hso3_so3"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_f64())
                    .collect(),
            )
        })
        .collect();
    let runtime: Vec<(String, String, Vec<Option<f64>>)> = sulfur_iv::OXIDANT_TABLE
        .iter()
        .map(|o| (o.key.to_string(), o.pricing.key().to_string(), o.k.to_vec()))
        .collect();
    assert_eq!(table, runtime);
}

#[test]
fn sulfur_dioxide_from_thiosulfate_reduces_iodine() {
    // The thiosulfate-acid law books its SO2 as gas held by the vessel.
    let mut v = water();
    aq(&mut v, "I2", 1e-4);
    v.deposit(SpeciesId::new("SO2"), Moles(4e-5), Phase::Gas);
    assert!((sulfur_iv::reducing_capacity_i2_equivalents(&v) - 4e-5).abs() < 1e-18);
    let eq = react(&mut v);
    assert_eq!(eq, vec!["I₂ + SO₂ + 2 H₂O → 2 I⁻ + SO₄²⁻ + 4 H⁺"]);
    assert!((m(&v, "I2") - 6e-5).abs() < 1e-15);
    assert!(m(&v, "SO2") < 1e-15);
    assert!((m(&v, "I-") - 8e-5).abs() < 1e-15);
    assert!((m(&v, "SO4-2") - 4e-5).abs() < 1e-15);
    assert_eq!(sulfur_iv::reducing_capacity_i2_equivalents(&v), 0.0);
}

#[test]
fn acid_peroxide_oxidises_sulfur_dioxide_to_sulfate() {
    let mut v = water();
    acidify(&mut v, 0.001);
    v.deposit(SpeciesId::new("H2O2"), Moles(0.002), Phase::Liquid);
    aq(&mut v, "SO2", 1e-5);
    let eq = react(&mut v);
    assert_eq!(eq, vec!["H₂O₂ + SO₂ → SO₄²⁻ + 2 H⁺"]);
    assert!(m(&v, "SO2") < 1e-15);
    assert!((m(&v, "H2O2") - (0.002 - 1e-5)).abs() < 1e-15);
    assert!((m(&v, "SO4-2") - (0.001 + 1e-5)).abs() < 1e-15);
}

#[test]
fn iodine_and_peroxide_share_sulfur_iv_by_their_rate_laws() {
    // KI-011 v0 at 60 s: oxidised iodine, a peroxide excess, and the SO2
    // the thiosulfate-acid route left behind.
    let mut v = water();
    acidify(&mut v, 0.001);
    aq(&mut v, "I-", 0.002);
    aq(&mut v, "K+", 0.002);
    aq(&mut v, "I2", 1.5e-5);
    v.deposit(SpeciesId::new("H2O2"), Moles(1.945e-3), Phase::Liquid);
    v.deposit(SpeciesId::new("SO2"), Moles(2.5e-5), Phase::Gas);
    let plan = sulfur_iv::plan(&v).expect("both oxidants and S(IV)");
    assert!(plan.peroxide_competes);
    assert!(plan.to_iodine > 0.0 && plan.to_iodine < 1.5e-5);
    assert!(plan.to_peroxide > 0.0);
    assert!((plan.to_iodine + plan.to_peroxide - 2.5e-5).abs() < 1e-15);
    let (i2_0, h_0, s_0) = (m(&v, "I2"), m(&v, "H2O2"), m(&v, "SO4-2"));
    let eq = react(&mut v);
    assert!(eq.contains(&"I₂ + SO₂ + 2 H₂O → 2 I⁻ + SO₄²⁻ + 4 H⁺".to_string()));
    assert!(eq.contains(&"H₂O₂ + SO₂ → SO₄²⁻ + 2 H⁺".to_string()));
    // No S(IV) survives beside either oxidant.
    assert!(m(&v, "SO2") < 1e-15);
    let used = (i2_0 - m(&v, "I2")) + (h_0 - m(&v, "H2O2"));
    assert!((used - 2.5e-5).abs() < 1e-15);
    assert!((m(&v, "SO4-2") - s_0 - 2.5e-5).abs() < 1e-15);
    // The iodine left closes the integrated competition.
    let i = m(&v, "I2");
    let beta = (2.5e-5 - (1.5e-5 - i)) / (1.5e-5f64 / i).ln();
    assert!(beta > 0.0);
    assert!((sulfur_iv::iodine_left(2.5e-5, 1.5e-5, beta) - i).abs() < 1e-12);
}

#[test]
fn hydrogen_sulfite_salt_with_acid_peroxide_releases_its_sodium() {
    let mut v = water();
    acidify(&mut v, 0.001);
    v.deposit(SpeciesId::new("H2O2"), Moles(0.001), Phase::Liquid);
    aq(&mut v, "NaHSO3", 2e-5);
    let eq = react(&mut v);
    assert_eq!(eq, vec!["H₂O₂ + HSO₃⁻ → SO₄²⁻ + H⁺ + H₂O"]);
    assert!(m(&v, "NaHSO3") < 1e-15);
    assert!((m(&v, "Na+") - 2e-5).abs() < 1e-15);
}

#[test]
fn neutral_peroxide_and_sulfite_is_refused_with_a_typed_note() {
    // Outside the measured pH 0-4.5 range: no writer, a named gap.
    let mut v = water();
    aq(&mut v, "SO3-2", 1e-4);
    aq(&mut v, "Na+", 2e-4);
    v.deposit(SpeciesId::new("H2O2"), Moles(1e-3), Phase::Liquid);
    v.solute_charge = displacement::solute_charge(&v);
    assert!(!sulfur_iv::fires(&v));
    assert!(react(&mut v).is_empty());
    assert!((m(&v, "SO3-2") - 1e-4).abs() < 1e-18);
    let keys = note_keys(&v);
    assert!(
        keys.contains(&"not-modeled.sulfite-peroxide-outside-measured-acid".to_string()),
        "{keys:?}"
    );
    // The generic unpriced-pair note no longer stands in for it.
    assert!(
        !keys.contains(&"not-modeled.redox-pair-unpriced".to_string()),
        "{keys:?}"
    );
}

#[test]
fn trace_peroxide_is_too_slow_to_be_instant_and_says_so() {
    let mut v = water();
    acidify(&mut v, 0.001);
    v.deposit(SpeciesId::new("H2O2"), Moles(1e-8), Phase::Liquid);
    aq(&mut v, "SO2", 1e-5);
    assert!(!sulfur_iv::fires(&v));
    assert!(react(&mut v).is_empty());
    assert!((m(&v, "SO2") - 1e-5).abs() < 1e-18);
    assert!(note_keys(&v).contains(&"not-modeled.sulfite-peroxide-slow".to_string()));
}

#[test]
fn sulfur_iv_without_an_oxidant_or_water_is_left_alone() {
    // Iodide is not an oxidant.
    let mut v = water();
    aq(&mut v, "I-", 1e-3);
    aq(&mut v, "K+", 1e-3);
    aq(&mut v, "SO2", 1e-5);
    assert!(sulfur_iv::plan(&v).is_none());
    assert!(react(&mut v).is_empty());
    assert!((m(&v, "SO2") - 1e-5).abs() < 1e-18);
    // Dry: gas and crystals do not titrate.
    let mut dry = Vessel::new(VesselId(1), "dry");
    dry.deposit(SpeciesId::new("SO2"), Moles(1e-4), Phase::Gas);
    dry.deposit(SpeciesId::new("I2"), Moles(1e-4), Phase::Solid);
    assert!(!sulfur_iv::fires(&dry));
    assert_eq!(sulfur_iv::reducing_capacity_i2_equivalents(&dry), 0.0);
}

#[test]
fn sulfite_and_iodine_without_peroxide_stay_with_the_halogen_writer() {
    let mut v = water();
    aq(&mut v, "I2", 0.001);
    aq(&mut v, "SO3-2", 0.0004);
    aq(&mut v, "Na+", 0.0008);
    assert!(!sulfur_iv::fires(&v));
    assert_eq!(
        react(&mut v),
        vec!["I₂ + SO₃²⁻ + H₂O → 2 I⁻ + SO₄²⁻ + 2 H⁺"]
    );
}

#[test]
fn sub_observable_kinetic_extent_is_still_a_reacted_event() {
    // KI-011: 8e-8 mol O2 from peroxide decomposition changed the ledger
    // with no event.
    let decomposition = kinetics::REGISTRY
        .iter()
        .find(|r| r.id == "peroxide-decomposition")
        .unwrap();
    let mut v = water();
    v.deposit(SpeciesId::new("H2O2"), Moles(0.002 - 1.6e-7), Phase::Liquid);
    v.deposit(SpeciesId::new("O2"), Moles(8e-8), Phase::Gas);
    const { assert!(8e-8 < OBSERVABLE_MOLES) };
    assert!(clock::ledger_visible_extent(&v, decomposition, 8e-8));
}

#[test]
fn ledger_invisible_extent_stays_silent() {
    let decomposition = kinetics::REGISTRY
        .iter()
        .find(|r| r.id == "peroxide-decomposition")
        .unwrap();
    let mut v = water();
    v.deposit(SpeciesId::new("H2O2"), Moles(0.002), Phase::Liquid);
    v.deposit(SpeciesId::new("O2"), Moles(1.0), Phase::Gas);
    assert!(!clock::ledger_visible_extent(&v, decomposition, 1e-12));
    assert!(!clock::ledger_visible_extent(&v, decomposition, 0.0));
}

fn equilibrate_events(v: &mut Vessel) -> Vec<Event> {
    v.solute_charge = displacement::solute_charge(v);
    CuratedEquilibrator.equilibrate(v).unwrap()
}

fn event_keys(events: &[Event]) -> Vec<String> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::NotYetModeled {
                reason: Some(r), ..
            } => Some(r.key.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn acid_permanganate_titrates_hydrogen_sulfite_stoichiometrically() {
    let mut v = water();
    acidify(&mut v, 0.001);
    aq(&mut v, "MnO4-", 2e-5);
    aq(&mut v, "K+", 2e-5);
    aq(&mut v, "HSO3-", 5e-5);
    aq(&mut v, "Na+", 5e-5);
    v.solute_charge = displacement::solute_charge(&v);
    let plan = sulfur_iv::other_plan(&v).expect("permanganate and S(IV)");
    // All of it: one proton per five hydrogen sulfite is well inside the
    // represented acid, and 2e-5 mol MnO4- takes exactly 5e-5 mol S(IV).
    assert_eq!(plan.run.map(|r| (r.oxidant, r.s_iv)), Some(("MnO4-", 5e-5)));
    assert!(plan.gaps.is_empty(), "{:?}", plan.gaps);
    assert!(sulfur_iv::fires(&v));
    let events = equilibrate_events(&mut v);
    let equations: Vec<String> = events
        .iter()
        .filter_map(|e| match e {
            Event::ReactionOccurred { equation, .. } => Some(equation.clone()),
            _ => None,
        })
        .collect();
    assert!(
        equations.contains(&"2 MnO₄⁻ + 5 HSO₃⁻ + H⁺ → 2 Mn²⁺ + 5 SO₄²⁻ + 3 H₂O".to_string()),
        "{equations:?}"
    );
    assert!(
        event_keys(&events).contains(&"not-modeled.sulfite-oxidant-no-rate-claimed".to_string())
    );
    assert!(m(&v, "MnO4-") < 1e-15);
    assert!(m(&v, "HSO3-") < 1e-15);
    assert!((m(&v, "Mn+2") - 2e-5).abs() < 1e-15);
    assert!((m(&v, "SO4-2") - (0.001 + 5e-5)).abs() < 1e-15);
}

#[test]
fn neutral_permanganate_and_sulfite_is_acid_limited_and_typed() {
    let mut v = water();
    aq(&mut v, "MnO4-", 1e-5);
    aq(&mut v, "K+", 1e-5);
    aq(&mut v, "SO3-2", 1e-5);
    aq(&mut v, "Na+", 2e-5);
    v.solute_charge = displacement::solute_charge(&v);
    let plan = sulfur_iv::other_plan(&v).expect("permanganate and sulfite");
    assert!(plan.run.is_none());
    assert!(!sulfur_iv::fires(&v));
    let keys = note_keys(&v);
    assert!(
        keys.contains(&"not-modeled.sulfite-oxidant-acid-limited".to_string()),
        "{keys:?}"
    );
    assert!(
        !keys.contains(&"not-modeled.redox-pair-unpriced".to_string()),
        "{keys:?}"
    );
    assert!(react(&mut v).is_empty());
    assert!((m(&v, "MnO4-") - 1e-5).abs() < 1e-18);
}

#[test]
fn permanganate_beside_iodine_is_an_unpriced_competition() {
    let mut v = water();
    acidify(&mut v, 0.001);
    aq(&mut v, "I2", 1e-5);
    aq(&mut v, "MnO4-", 1e-5);
    aq(&mut v, "K+", 1e-5);
    aq(&mut v, "SO2", 1e-5);
    let plan = sulfur_iv::other_plan(&v).unwrap();
    assert!(plan.run.is_none());
    assert!(plan
        .gaps
        .contains(&sulfur_iv::OtherGap::Competition { oxidant: "MnO4-" }));
    assert!(note_keys(&v).contains(&"not-modeled.sulfite-oxidant-competition-unpriced".to_string()));
}

#[test]
fn nitrite_and_sulfite_are_refused_for_the_unregistered_product() {
    let mut v = water();
    acidify(&mut v, 0.001);
    aq(&mut v, "NO2-", 1e-4);
    aq(&mut v, "Na+", 1e-4);
    aq(&mut v, "SO2", 1e-5);
    assert!(!sulfur_iv::fires(&v));
    let plan = sulfur_iv::other_plan(&v).unwrap();
    assert_eq!(
        plan.gaps,
        vec![sulfur_iv::OtherGap::ProductNotInRegistry {
            oxidant: "NO2-",
            product: sulfur_iv::NITRITE_PRODUCT,
        }]
    );
    assert!(
        note_keys(&v).contains(&"not-modeled.sulfite-oxidant-product-not-in-registry".to_string())
    );
}

#[test]
fn dissolved_oxygen_autoxidation_is_refused_typed() {
    let mut v = water();
    aq(&mut v, "O2", 2.5e-5);
    aq(&mut v, "HSO3-", 1e-5);
    aq(&mut v, "Na+", 1e-5);
    assert!(!sulfur_iv::fires(&v));
    assert!(note_keys(&v).contains(&"not-modeled.sulfite-autoxidation".to_string()));
    // Oxygen only in the gas phase is not dissolved oxygen.
    let mut gas = water();
    gas.deposit(SpeciesId::new("O2"), Moles(1e-3), Phase::Gas);
    aq(&mut gas, "HSO3-", 1e-5);
    aq(&mut gas, "Na+", 1e-5);
    assert!(sulfur_iv::other_plan(&gas).is_none());
}

#[test]
fn rated_oxidants_without_a_registry_form_never_fire() {
    // Ozone, HOBr and OBr- carry cited constants but no registry form;
    // the table keeps them so that a registry addition is priced at once.
    for key in ["O3", "HBrO", "BrO-", "Br2", "Cr2O7-2", "HCrO4-"] {
        let o = sulfur_iv::oxidant(key).unwrap();
        assert!(halogen_redox::registry_forms(o.key).is_empty(), "{key}");
    }
    let o3 = sulfur_iv::oxidant("O3").unwrap();
    assert_eq!(o3.k, [Some(2.4e4), Some(3.7e5), Some(1.5e9)]);
    assert_eq!(o3.pricing, sulfur_iv::Pricing::Rated);
}
