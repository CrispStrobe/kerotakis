//! Chlorine as a reagent (C1 audit class `C1A-chlorine`): positive and
//! negative controls for every rule in `kerotakis_core::chlorine`.

use kerotakis_core::chlorine;
use kerotakis_core::solve::Equilibrator;
use kerotakis_core::vessel::Headspace;
use kerotakis_core::*;

fn water(moles: f64) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("water"), Moles(moles), Phase::Liquid);
    v
}

fn put(v: &mut Vessel, key: &str, n: f64, phase: Phase) {
    v.deposit(SpeciesId::new(key), Moles(n), phase);
}

fn run(v: &mut Vessel) -> Vec<Event> {
    v.solute_charge = displacement::solute_charge(v);
    CuratedEquilibrator.equilibrate(v).unwrap()
}

fn equations(events: &[Event]) -> Vec<String> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::ReactionOccurred { equation, .. } => Some(equation.clone()),
            _ => None,
        })
        .collect()
}

fn note_keys(events: &[Event]) -> Vec<String> {
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

fn evolved(events: &[Event], key: &str) -> f64 {
    events
        .iter()
        .map(|e| match e {
            Event::GasEvolved { species, moles, .. } if species.0 == key => moles.0,
            _ => 0.0,
        })
        .sum()
}

fn m(v: &Vessel, key: &str, phase: Phase) -> f64 {
    v.contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}

fn all(v: &Vessel, key: &str) -> f64 {
    v.contents
        .iter()
        .filter(|p| p.species.0 == key)
        .map(|p| p.moles.0)
        .sum()
}

/// Hydrochloric acid the way the ledger books it after a solve.
fn hcl(v: &mut Vessel, n: f64) {
    put(v, "H+", n, Phase::Aqueous);
    put(v, "Cl-", n, Phase::Aqueous);
}

fn sealed(v: &mut Vessel, litres: f64) {
    v.headspace = Headspace::Sealed {
        volume: Liters(litres),
    };
}

// ── preparation ──────────────────────────────────────────────────────

#[test]
fn manganese_dioxide_and_concentrated_hydrochloric_acid_make_chlorine() {
    let mut v = water(0.111); // 2 mL
    sealed(&mut v, 0.05);
    put(&mut v, "MnO2", 0.023, Phase::Solid);
    hcl(&mut v, 0.06);
    let events = run(&mut v);
    let eq = equations(&events);
    assert!(
        eq.iter().any(|e| e.starts_with("MnO₂ + 4 H⁺ + 2 Cl⁻")),
        "{eq:?}"
    );
    let mn = m(&v, "Mn+2", Phase::Aqueous);
    assert!(mn > 1e-3, "{mn}");
    // Chlorine made equals manganese reduced (one Cl₂ per Mn).
    let cl_made = all(&v, "Cl2") + all(&v, "HClO") + all(&v, "ClO-") + evolved(&events, "Cl2");
    assert!((cl_made - mn).abs() < 1e-9, "{cl_made} vs {mn}");
    assert!(note_keys(&events).contains(&"not-modeled.chlorine-preparation-ideal".to_string()));
}

#[test]
fn manganese_dioxide_and_dilute_acid_stay_inert() {
    let mut v = water(5.55); // 100 mL, 0.1 mol/kg HCl
    put(&mut v, "MnO2", 0.01, Phase::Solid);
    hcl(&mut v, 0.01);
    let events = run(&mut v);
    assert!(equations(&events).is_empty(), "{:?}", equations(&events));
    assert_eq!(m(&v, "MnO2", Phase::Solid), 0.01);
}

#[test]
fn manganese_dioxide_and_salt_water_without_acid_do_nothing() {
    let mut v = water(0.111);
    put(&mut v, "MnO2", 0.02, Phase::Solid);
    put(&mut v, "NaCl", 0.03, Phase::Solid);
    assert!(equations(&run(&mut v)).is_empty());
}

#[test]
fn permanganate_and_hydrochloric_acid_make_chlorine_and_release_potassium() {
    let mut v = water(0.222); // 4 mL
    sealed(&mut v, 0.1);
    put(&mut v, "KMnO4", 0.0095, Phase::Solid);
    hcl(&mut v, 0.12);
    let events = run(&mut v);
    assert!(equations(&events)
        .iter()
        .any(|e| e.starts_with("2 MnO₄⁻ + 16 H⁺ + 10 Cl⁻")));
    let mn = m(&v, "Mn+2", Phase::Aqueous);
    assert!((mn - 0.0095).abs() < 1e-9, "{mn}");
    assert!((m(&v, "K+", Phase::Aqueous) - 0.0095).abs() < 1e-9);
    let cl_made = all(&v, "Cl2") + all(&v, "HClO") + evolved(&events, "Cl2");
    assert!((cl_made - 2.5 * 0.0095).abs() < 1e-9, "{cl_made}");
}

// ── chlorine water ───────────────────────────────────────────────────

#[test]
fn chlorine_over_water_dissolves_and_hydrolyses_to_its_constant() {
    let mut v = water(0.1665); // 3 mL
    sealed(&mut v, 0.017);
    put(&mut v, "Cl2", 0.0008, Phase::Gas);
    let events = run(&mut v);
    assert!(equations(&events).contains(&"Cl₂ + H₂O ⇌ HClO + H⁺ + Cl⁻".to_string()));
    let hclo = m(&v, "HClO", Phase::Aqueous);
    assert!(hclo > 1e-5, "{hclo}");
    // Chlorine conserved: pool + hypochlorous acid.
    assert!((all(&v, "Cl2") + hclo - 0.0008).abs() < 1e-12);
    assert!((m(&v, "Cl-", Phase::Aqueous) - hclo).abs() < 1e-15);
    // The acid it makes is the charge imbalance.
    assert!((displacement::unspent_acidity(&v) - hclo).abs() < 1e-9);
    // Re-running the step changes nothing (it is at its constant).
    assert!(equations(&run(&mut v)).is_empty());
    assert!(note_keys(&events).contains(&"not-modeled.chlorine-water-pool".to_string()));
}

#[test]
fn hydrolysis_is_suppressed_by_acid_and_chloride() {
    let mut plain = water(0.1665);
    sealed(&mut plain, 0.017);
    put(&mut plain, "Cl2", 0.0008, Phase::Gas);
    run(&mut plain);
    let mut acid = water(0.1665);
    sealed(&mut acid, 0.017);
    put(&mut acid, "Cl2", 0.0008, Phase::Gas);
    hcl(&mut acid, 0.003); // 1 mol/kg HCl
    run(&mut acid);
    assert!(m(&acid, "HClO", Phase::Aqueous) < 0.05 * m(&plain, "HClO", Phase::Aqueous));
}

#[test]
fn dry_chlorine_at_room_temperature_does_nothing() {
    let mut v = Vessel::new(VesselId(0), "jar");
    sealed(&mut v, 0.02);
    put(&mut v, "Cl2", 0.0008, Phase::Gas);
    put(&mut v, "Fe", 0.0018, Phase::Solid);
    assert!(equations(&run(&mut v)).is_empty());
    assert_eq!(m(&v, "Cl2", Phase::Gas), 0.0008);
}

// ── alkali ───────────────────────────────────────────────────────────

#[test]
fn chlorine_in_sodium_hydroxide_makes_hypochlorite() {
    let mut v = water(0.5);
    put(&mut v, "Na+", 0.0025, Phase::Aqueous);
    put(&mut v, "base_equivalents", 0.0025, Phase::Aqueous);
    put(&mut v, "Cl2", 0.001, Phase::Gas);
    let events = run(&mut v);
    assert!(equations(&events).contains(&"Cl₂ + 2 OH⁻ → ClO⁻ + Cl⁻ + H₂O".to_string()));
    assert!((m(&v, "ClO-", Phase::Aqueous) - 0.001).abs() < 1e-12);
    assert!(all(&v, "Cl2") < 1e-12);
    assert!((m(&v, "base_equivalents", Phase::Aqueous) - 0.0005).abs() < 1e-12);
}

#[test]
fn chlorine_on_slaked_lime_makes_bleaching_liquor() {
    let mut v = water(0.06);
    put(&mut v, "Ca(OH)2", 0.07, Phase::Solid);
    put(&mut v, "Cl2", 0.01, Phase::Gas);
    run(&mut v);
    assert!((m(&v, "ClO-", Phase::Aqueous) - 0.01).abs() < 1e-12);
    assert!((m(&v, "Ca+2", Phase::Aqueous) - 0.01).abs() < 1e-12);
    assert!((m(&v, "Ca(OH)2", Phase::Solid) - 0.06).abs() < 1e-12);
}

// ── reductants ───────────────────────────────────────────────────────

#[test]
fn chlorine_oxidises_iron_two() {
    let mut v = water(0.555);
    put(&mut v, "Fe+2", 0.0165, Phase::Aqueous);
    put(&mut v, "SO4-2", 0.0165, Phase::Aqueous);
    put(&mut v, "Cl2", 0.01, Phase::Gas);
    let eq = equations(&run(&mut v));
    assert!(
        eq.contains(&"Cl₂ + 2 Fe²⁺ → 2 Fe³⁺ + 2 Cl⁻".to_string()),
        "{eq:?}"
    );
    assert!(m(&v, "Fe+2", Phase::Aqueous) < 1e-12);
    assert!((m(&v, "Fe+3", Phase::Aqueous) - 0.0165).abs() < 1e-12);
}

#[test]
fn chlorine_displaces_iodine_from_potassium_iodide() {
    let mut v = water(0.555);
    put(&mut v, "KI", 0.002, Phase::Aqueous);
    put(&mut v, "Cl2", 0.0005, Phase::Gas);
    run(&mut v);
    // One Cl₂ takes two electrons from iodide: two chlorides, one iodine
    // (free or held as triiodide by the iodide left over).
    assert!(all(&v, "Cl2") < 1e-15);
    assert!((m(&v, "Cl-", Phase::Aqueous) - 0.001).abs() < 1e-12);
    assert!((m(&v, "K+", Phase::Aqueous) - 0.001).abs() < 1e-12);
    let iodine = 2.0 * all(&v, "I2") + 3.0 * all(&v, "I3-") + all(&v, "I-") + all(&v, "KI");
    assert!((iodine - 0.002).abs() < 1e-12, "{iodine}");
    assert!(all(&v, "I2") + all(&v, "I3-") > 0.0004, "{}", all(&v, "I2"));
}

#[test]
fn iodine_does_not_oxidise_chloride() {
    // Negative: the reverse displacement never runs.
    let mut v = water(0.555);
    put(&mut v, "I2", 0.0005, Phase::Aqueous);
    put(&mut v, "NaCl", 0.002, Phase::Aqueous);
    run(&mut v);
    assert!(all(&v, "Cl2") < 1e-15);
}

// ── bleaching ────────────────────────────────────────────────────────

#[test]
fn chlorine_water_bleaches_anthocyanin_with_a_typed_uncalibrated_note() {
    let mut v = water(0.0277);
    sealed(&mut v, 0.02);
    put(&mut v, "anthocyanin", 1.1e-5, Phase::Aqueous);
    put(&mut v, "Cl2", 0.0008, Phase::Gas);
    let events = run(&mut v);
    assert!(all(&v, "anthocyanin") < 1e-15);
    assert!((m(&v, "anthocyanin_ox", Phase::Aqueous) - 1.1e-5).abs() < 1e-15);
    assert!(note_keys(&events).contains(&"not-modeled.chlorine-bleach-uncalibrated".to_string()));
}

#[test]
fn pigment_without_an_oxidant_is_untouched() {
    let mut v = water(0.0277);
    put(&mut v, "anthocyanin", 1.1e-5, Phase::Aqueous);
    hcl(&mut v, 0.001);
    run(&mut v);
    assert_eq!(m(&v, "anthocyanin", Phase::Aqueous), 1.1e-5);
}

// ── dry chlorination ─────────────────────────────────────────────────

#[test]
fn hot_iron_burns_in_chlorine_heat_unpriced() {
    let mut v = Vessel::new(VesselId(0), "jar");
    sealed(&mut v, 0.235);
    put(&mut v, "Fe", 0.00179, Phase::Solid);
    put(&mut v, "Cl2", 0.0008, Phase::Gas);
    v.temperature = Kelvin(900.0);
    let events = run(&mut v);
    assert!(equations(&events).contains(&"2 Fe + 3 Cl₂ → 2 FeCl₃".to_string()));
    let fecl3 = m(&v, "FeCl3", Phase::Solid);
    assert!((fecl3 - 0.0008 * 2.0 / 3.0).abs() < 1e-12, "{fecl3}");
    assert!(m(&v, "Cl2", Phase::Gas) < 1e-12);
    assert!(
        note_keys(&events).contains(&"not-modeled.chlorine-combustion-heat-unpriced".to_string())
    );
}

#[test]
fn ignited_hydrogen_and_chlorine_make_hydrogen_chloride_that_water_takes_up() {
    let mut v = Vessel::new(VesselId(0), "jar");
    sealed(&mut v, 0.25);
    put(&mut v, "Cl2", 0.01, Phase::Gas);
    put(&mut v, "H2", 0.005, Phase::Gas);
    v.ignition = true;
    run(&mut v);
    assert!((m(&v, "HCl", Phase::Gas) - 0.01).abs() < 1e-12);
    assert!((m(&v, "Cl2", Phase::Gas) - 0.005).abs() < 1e-12);
    v.ignition = false;
    put(&mut v, "water", 0.555, Phase::Liquid);
    run(&mut v);
    assert!(m(&v, "HCl", Phase::Gas) < 1e-12);
    assert!(displacement::unspent_acidity(&v) > 0.0099);
}

#[test]
fn hydrolysis_constant_from_nbs() {
    let k = chlorine::hydrolysis_k(298.15);
    assert!((k - 4.58e-4).abs() < 1e-6, "{k}");
}
