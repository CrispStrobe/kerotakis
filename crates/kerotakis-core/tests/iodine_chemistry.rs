//! Iodine family (2026-11-10): triiodide association, iodine optics,
//! ferric/iodide equilibrium and the iodometric thiosulfate row.
//!
//! `iodine_chemistry_coverage_is_current` pins the runtime constants to the
//! generated table `docs/iodine-chemistry-coverage-20261110.json`
//! (`python3 tools/iodine_chemistry_coverage.py` regenerates it). The rest
//! are positive, negative and exact-closure controls per mechanism.

use kerotakis_core::instrument::{InstrumentContract, Spectrophotometer};
use kerotakis_core::iodine_chemistry as iodine;
use kerotakis_core::solve::Equilibrator;
use kerotakis_core::*;

const TABLE: &str = include_str!("../../../docs/iodine-chemistry-coverage-20261110.json");

#[test]
fn iodine_chemistry_coverage_is_current() {
    let doc: serde_json::Value = serde_json::from_str(TABLE).unwrap();
    let c = &doc["constants"];
    assert_eq!(c["triiodide_k"].as_f64().unwrap(), iodine::TRIIODIDE_K);
    assert_eq!(
        c["e0_i2aq_iodide_v"].as_f64().unwrap(),
        iodine::E0_I2AQ_IODIDE_V
    );
    for (name, band) in [("i2_band", iodine::I2_BAND), ("i3_band", iodine::I3_BAND)] {
        let row: Vec<f64> = c[name]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        assert_eq!(row, vec![band.0, band.1, band.2], "{name}");
    }
    let keys: Vec<&str> = doc["optical_keys"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(keys, iodine::OPTICAL_KEYS);
}

fn water(ml: f64) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("water"), Moles(ml / 18.015), Phase::Liquid);
    v
}

fn aq(v: &mut Vessel, key: &str, n: f64) {
    v.deposit(SpeciesId::new(key), Moles(n), Phase::Aqueous);
}

fn m(v: &Vessel, key: &str, phase: Phase) -> f64 {
    v.contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}

// --- triiodide association -------------------------------------------------

#[test]
fn triiodide_split_meets_k_and_closes_both_pools() {
    let mut v = water(30.0);
    aq(&mut v, "I2", 3e-4);
    aq(&mut v, "I-", 3e-3);
    let s = iodine::speciate(&v).expect("aqueous");
    assert!((s.iodine + s.triiodide - 3e-4).abs() <= 1e-18);
    assert!((s.iodide + s.triiodide - 3e-3).abs() <= 1e-18);
    let litres = v.liquid_volume().0;
    let k = (s.triiodide / litres) / ((s.iodine / litres) * (s.iodide / litres));
    assert!((k / iodine::TRIIODIDE_K - 1.0).abs() < 1e-8, "{k}");
    // At 0.1 M iodide most dissolved iodine is triiodide.
    assert!(s.triiodide > 0.9 * 3e-4, "{s:?}");
}

#[test]
fn undissociated_iodide_salt_counts_as_iodide() {
    let mut a = water(30.0);
    aq(&mut a, "I2", 1e-4);
    aq(&mut a, "KI", 1e-3);
    let mut b = water(30.0);
    aq(&mut b, "I2", 1e-4);
    aq(&mut b, "I-", 1e-3);
    assert_eq!(iodine::speciate(&a), iodine::speciate(&b));
}

#[test]
fn chloride_is_not_an_iodine_ligand() {
    let mut v = water(30.0);
    aq(&mut v, "I2", 3e-5);
    aq(&mut v, "KCl", 3e-3);
    let s = iodine::speciate(&v).unwrap();
    assert_eq!(s.triiodide, 0.0);
    assert_eq!(s.iodine, 3e-5);
}

#[test]
fn iodide_raises_the_iodine_capacity_and_chloride_does_not() {
    let mut iodide = water(30.0);
    aq(&mut iodide, "I-", 3e-3);
    let mut chloride = water(30.0);
    aq(&mut chloride, "KCl", 3e-3);
    let (pure, litres) = iodine::pure_water_capacity(&chloride).expect("I2 solubility");
    assert_eq!(iodine::enhanced_capacity(&chloride, pure, litres), pure);
    let enhanced = iodine::enhanced_capacity(&iodide, pure, litres);
    // S0 (1 + K I / (1 + K S0)) at 0.1 M iodide: a twenty-fold rise or more.
    assert!(enhanced > 20.0 * pure, "{enhanced} vs {pure}");
    // Closed form agrees with the split: at capacity free I2 is S0.
    let (_, iodide_litres) = iodine::pure_water_capacity(&iodide).unwrap();
    let s = iodine::split(enhanced, 3e-3, iodide_litres);
    assert!((s.iodine - pure).abs() / pure < 1e-6, "{s:?}");
}

#[test]
fn solid_iodine_dissolves_in_iodide_without_a_supersaturation_flag() {
    let mut v = water(30.0);
    v.deposit(SpeciesId::new("I2"), Moles(3e-4), Phase::Solid);
    aq(&mut v, "I-", 3e-3);
    aq(&mut v, "K+", 3e-3);
    let moves = solve::saturation_moves(&v);
    assert!(
        !moves
            .iter()
            .any(|m| matches!(m, solve::SaturationMove::Supersaturated { .. })),
        "{moves:?}"
    );
    let dissolved: f64 = moves
        .iter()
        .filter_map(|m| match m {
            solve::SaturationMove::Dissolve(s, n) if s.0 == "I2" => Some(n.0),
            _ => None,
        })
        .sum();
    assert!((dissolved - 3e-4).abs() < 1e-15, "{moves:?}");
}

// --- optics ----------------------------------------------------------------

#[test]
fn iodine_absorbance_is_beer_lambert_over_the_split() {
    let mut v = water(100.0);
    aq(&mut v, "I2", 1e-5);
    aq(&mut v, "KI", 1e-3);
    let litres = v.liquid_volume().0;
    let s = iodine::split(1e-5, 1e-3, litres);
    let i2 = spectrum::bands(&[iodine::I2_BAND]);
    let i3 = spectrum::bands(&[iodine::I3_BAND]);
    let a = solution_optics::absorbance(&v, 1.0);
    for k in 0..spectrum::BANDS {
        let expected = (i2[k] * s.iodine + i3[k] * s.triiodide) / litres;
        assert!((a[k] - expected).abs() <= 1e-12 * expected.max(1.0));
    }
    let reading = Spectrophotometer::default().measure(&v).expect("reading");
    assert_eq!(reading.observable, "absorbance at 405 nm");
    assert!(reading.value > 0.0);
}

#[test]
fn iodide_deepens_the_colour_of_the_same_iodine() {
    let mut plain = water(100.0);
    aq(&mut plain, "I2", 1e-5);
    let mut lugol = plain.clone();
    aq(&mut lugol, "KI", 1e-3);
    let a = solution_optics::absorbance(&plain, 1.0)[0];
    let b = solution_optics::absorbance(&lugol, 1.0)[0];
    assert!(b > 5.0 * a, "{a} -> {b}");
}

#[test]
fn reduced_iodine_reads_zero_not_refused() {
    let mut v = water(100.0);
    aq(&mut v, "I-", 1e-3);
    aq(&mut v, "K+", 1e-3);
    assert_eq!(Spectrophotometer::default().measure(&v).unwrap().value, 0.0);
}

#[test]
fn starch_iodine_complex_is_refused_with_a_typed_translated_reason() {
    let mut v = water(100.0);
    aq(&mut v, "KI", 2.5e-4);
    aq(&mut v, "I2", 8e-5);
    v.deposit(SpeciesId::new("starch"), Moles(6e-3), Phase::Solid);
    assert!(solution_optics::coverage(&v).starch_iodine_complex);
    assert!(Spectrophotometer::default().measure(&v).is_none());
    let mut bench = Bench::new();
    bench.vessels[0] = v;
    let events = bench
        .step_with(
            Operator::Measure {
                vessel: VesselId(0),
                instrument: Instrument::Spectrophotometer,
            },
            &mut SolverStack::new(vec![]),
            &PermissiveScreen,
        )
        .unwrap();
    assert!(!events.iter().any(|e| matches!(e, Event::Measured { .. })));
    let reason = events
        .iter()
        .find_map(|event| match event {
            Event::NotYetModeled {
                reason: Some(reason),
                ..
            } if reason.key == "not-modeled.starch-iodine-absorptivity" => Some(reason),
            _ => None,
        })
        .expect("typed refusal");
    for locale in [Locale::parse("de"), Locale::parse("fr")] {
        assert_ne!(reason.render(locale), reason.render(Locale::EN));
    }
    // The eye still sees the complex.
    assert!(appearance::observe(&bench.vessels[0])
        .words
        .contains("blue-black"));
}

// --- Fe³⁺/I⁻ equilibrium ----------------------------------------------------

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

fn charge(v: &Vessel) -> f64 {
    3.0 * m(v, "Fe+3", Phase::Aqueous) + 2.0 * m(v, "Fe+2", Phase::Aqueous)
        - m(v, "I-", Phase::Aqueous)
}

#[test]
fn ferric_iron_oxidises_iodide_to_its_equilibrium_with_exact_closure() {
    let mut v = water(25.0);
    aq(&mut v, "Fe+3", 4e-4);
    aq(&mut v, "I-", 1e-3);
    let charge_before = charge(&v);
    assert_eq!(react(&mut v), vec!["2 Fe³⁺ + 2 I⁻ → 2 Fe²⁺ + I₂"]);
    let fe3 = m(&v, "Fe+3", Phase::Aqueous);
    let fe2 = m(&v, "Fe+2", Phase::Aqueous);
    let i2 = m(&v, "I2", Phase::Aqueous);
    let iodide = m(&v, "I-", Phase::Aqueous);
    assert!((fe3 + fe2 - 4e-4).abs() < 1e-16, "Fe closure");
    assert!((iodide + 2.0 * i2 - 1e-3).abs() < 1e-16, "I closure");
    assert!((fe2 - 2.0 * i2).abs() < 1e-16, "electron closure");
    assert!((charge(&v) - charge_before).abs() < 1e-16, "charge closure");
    // Mostly converted, and at K with free species from the triiodide split.
    assert!(fe2 > 0.9 * 4e-4, "{fe2}");
    let litres = v.liquid_volume().0;
    let s = iodine::split(i2, iodide, litres);
    let q = (fe2 / litres).powi(2) * (s.iodine / litres)
        / ((fe3 / litres).powi(2) * (s.iodide / litres).powi(2));
    assert!(
        (q.log10() - iodine::ferric_iodide_log_k()).abs() < 1e-6,
        "{q}"
    );
    // Idempotent: a vessel at equilibrium does not react again.
    assert!(react(&mut v).is_empty());
}

#[test]
fn ferrous_iron_and_iodide_do_nothing() {
    let mut v = water(25.0);
    aq(&mut v, "Fe+2", 4e-4);
    aq(&mut v, "I-", 1e-3);
    assert!(react(&mut v).is_empty());
    assert_eq!(m(&v, "I2", Phase::Aqueous), 0.0);
}

#[test]
fn ferric_iodide_needs_liquid_water_and_the_bench_range() {
    let mut dry = Vessel::new(VesselId(0), "beaker");
    aq(&mut dry, "Fe+3", 4e-4);
    aq(&mut dry, "I-", 1e-3);
    assert_eq!(iodine::ferric_iodide_extent(&dry), 0.0);
    let mut hot = water(25.0);
    aq(&mut hot, "Fe+3", 4e-4);
    aq(&mut hot, "I-", 1e-3);
    hot.temperature = Kelvin(353.15);
    assert_eq!(iodine::ferric_iodide_extent(&hot), 0.0);
}

#[test]
fn iodine_and_ferrous_iron_run_back_only_to_the_same_equilibrium() {
    // Approach from the other side: the end state satisfies the same K.
    let mut v = water(25.0);
    aq(&mut v, "Fe+2", 4e-4);
    aq(&mut v, "I2", 2e-4);
    let eq = react(&mut v);
    assert_eq!(eq, vec!["2 Fe²⁺ + I₂ → 2 Fe³⁺ + 2 I⁻"]);
    let fe3 = m(&v, "Fe+3", Phase::Aqueous);
    let fe2 = m(&v, "Fe+2", Phase::Aqueous);
    let i2 = m(&v, "I2", Phase::Aqueous);
    let iodide = m(&v, "I-", Phase::Aqueous);
    assert!((fe3 + fe2 - 4e-4).abs() < 1e-16);
    assert!((iodide + 2.0 * i2 - 4e-4).abs() < 1e-16);
    let litres = v.liquid_volume().0;
    let s = iodine::split(i2, iodide, litres);
    let q = (fe2 / litres).powi(2) * (s.iodine / litres)
        / ((fe3 / litres).powi(2) * (s.iodide / litres).powi(2));
    assert!((q.log10() - iodine::ferric_iodide_log_k()).abs() < 1e-6);
}

// --- iodometry (existing curated row, pinned here as a family member) -----

#[test]
fn thiosulfate_reduces_iodine_two_to_one_exactly() {
    let mut v = water(25.0);
    aq(&mut v, "I2", 1e-4);
    aq(&mut v, "S2O3-2", 1.5e-4);
    aq(&mut v, "Na+", 3e-4);
    assert_eq!(react(&mut v), vec!["I₂ + 2 S₂O₃²⁻ → 2 I⁻ + S₄O₆²⁻"]);
    assert!((m(&v, "I2", Phase::Aqueous) - 2.5e-5).abs() < 1e-16);
    assert!(m(&v, "S2O3-2", Phase::Aqueous).abs() < 1e-16);
    assert!((m(&v, "S4O6-2", Phase::Aqueous) - 7.5e-5).abs() < 1e-16);
    assert!((m(&v, "I-", Phase::Aqueous) - 1.5e-4).abs() < 1e-16);
}
