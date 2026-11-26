//! The species that switch on dormant route rows (2026-11-25): bromine,
//! chromium(III), nitric acid, NO, NO2, copper(II) nitrate, KHSO4, K2SO4.
//!
//! Each family gets a positive control (the row now fires and books the
//! registered product) and a negative control (the same reagents without
//! the condition the source requires, or the reverse direction, do nothing).

use kerotakis_core::solve::Equilibrator;
use kerotakis_core::*;

fn water(moles: f64) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("water"), Moles(moles), Phase::Liquid);
    v
}

fn put(v: &mut Vessel, key: &str, n: f64, phase: Phase) {
    v.deposit(SpeciesId::new(key), Moles(n), phase);
}

/// Sulfuric acid as the ledger books it after a solve.
fn acid(v: &mut Vessel, protons: f64) {
    put(v, "H+", protons, Phase::Aqueous);
    put(v, "SO4-2", protons / 2.0, Phase::Aqueous);
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

fn all(v: &Vessel, key: &str) -> f64 {
    v.contents
        .iter()
        .filter(|p| p.species.0 == key)
        .map(|p| p.moles.0)
        .sum()
}

fn gas_out(events: &[Event], v: &Vessel, key: &str) -> f64 {
    let evolved: f64 = events
        .iter()
        .map(|e| match e {
            Event::GasEvolved { species, moles, .. } if species.0 == key => moles.0,
            _ => 0.0,
        })
        .sum();
    evolved
        + v.contents
            .iter()
            .filter(|p| p.species.0 == key && p.phase == Phase::Gas)
            .map(|p| p.moles.0)
            .sum::<f64>()
}

// ── registry rows ────────────────────────────────────────────────────

#[test]
fn the_new_species_are_registered_with_their_phase_and_colour() {
    for (key, phase, appearance) in [
        ("Br2", Phase::Liquid, "red-brown"),
        ("HNO3", Phase::Liquid, "colourless"),
        ("NO", Phase::Gas, "colourless"),
        ("NO2", Phase::Gas, "brown"),
        ("Cr+3", Phase::Aqueous, "green"),
        ("Cu(NO3)2", Phase::Solid, "blue-green"),
        ("KHSO4", Phase::Solid, "white"),
        ("K2SO4", Phase::Solid, "white"),
    ] {
        let data = species::lookup_key(key).unwrap_or_else(|| panic!("{key} registered"));
        assert_eq!(data.standard_phase, phase, "{key}");
        assert_eq!(data.appearance, Some(appearance), "{key}");
    }
    // Chromium(VI) ions (2026-11-26): a solved Cr(6) total books back on
    // them, never as Cr+3.
    for (key, appearance) in [
        ("CrO4-2", "yellow"),
        ("Cr2O7-2", "orange"),
        ("HCrO4-", "orange"),
    ] {
        let data = species::lookup_key(key).unwrap_or_else(|| panic!("{key} registered"));
        assert_eq!(data.standard_phase, Phase::Aqueous, "{key}");
        assert_eq!(data.appearance, Some(appearance), "{key}");
    }
}

// ── halogen displacement ─────────────────────────────────────────────

#[test]
fn chlorine_displaces_bromine_from_bromide() {
    let mut v = water(0.555);
    put(&mut v, "KBr", 0.002, Phase::Aqueous);
    put(&mut v, "Cl2", 0.0005, Phase::Aqueous);
    let events = run(&mut v);
    assert!(
        equations(&events).contains(&"Cl₂ + 2 Br⁻ → Br₂ + 2 Cl⁻".to_string()),
        "{:?}",
        equations(&events)
    );
    assert!((all(&v, "Br2") - 0.0005).abs() < 1e-9, "{}", all(&v, "Br2"));
}

#[test]
fn bromine_does_not_oxidise_chloride() {
    let mut v = water(0.555);
    put(&mut v, "NaCl", 0.002, Phase::Aqueous);
    put(&mut v, "Br2", 0.0005, Phase::Liquid);
    let events = run(&mut v);
    assert!(equations(&events).iter().all(|e| !e.contains("Cl₂")));
    assert!((all(&v, "Br2") - 0.0005).abs() < 1e-9);
}

#[test]
fn bromine_oxidises_iodide_and_iodine_leaves_bromide() {
    let mut v = water(0.555);
    put(&mut v, "KI", 0.002, Phase::Aqueous);
    put(&mut v, "Br2", 0.0005, Phase::Liquid);
    let events = run(&mut v);
    assert!(
        equations(&events).contains(&"Br₂ + 2 I⁻ → I₂ + 2 Br⁻".to_string()),
        "{:?}",
        equations(&events)
    );
    assert!(all(&v, "Br2") < 1e-9);
    // The reverse: iodine and bromide.
    let mut v = water(0.555);
    put(&mut v, "KBr", 0.002, Phase::Aqueous);
    put(&mut v, "I2", 0.0005, Phase::Solid);
    let events = run(&mut v);
    assert!(equations(&events).iter().all(|e| !e.contains("Br₂")));
    assert!(all(&v, "Br2") < 1e-12);
}

// ── oxidants in acid ─────────────────────────────────────────────────

#[test]
fn manganese_dioxide_oxidises_bromide_only_in_acid() {
    let mut v = water(0.555);
    put(&mut v, "MnO2", 0.001, Phase::Solid);
    put(&mut v, "KBr", 0.004, Phase::Aqueous);
    acid(&mut v, 0.02);
    let events = run(&mut v);
    assert!(
        equations(&events).contains(&"MnO₂ + 4 H⁺ + 2 Br⁻ → Mn²⁺ + Br₂ + 2 H₂O".to_string()),
        "{:?}",
        equations(&events)
    );
    assert!(all(&v, "Br2") > 1e-4, "{}", all(&v, "Br2"));
    let mut v = water(0.555);
    put(&mut v, "MnO2", 0.001, Phase::Solid);
    put(&mut v, "KBr", 0.004, Phase::Aqueous);
    let events = run(&mut v);
    assert!(equations(&events).iter().all(|e| !e.contains("Br₂")));
    assert!(all(&v, "Br2") < 1e-12);
}

#[test]
fn acidified_dichromate_oxidises_iodide_and_bromide() {
    let mut v = water(0.555);
    put(&mut v, "K2Cr2O7", 0.0002, Phase::Aqueous);
    put(&mut v, "KI", 0.003, Phase::Aqueous);
    acid(&mut v, 0.02);
    let events = run(&mut v);
    assert!(
        equations(&events).contains(&"Cr₂O₇²⁻ + 14 H⁺ + 6 I⁻ → 2 Cr³⁺ + 3 I₂ + 7 H₂O".to_string()),
        "{:?}",
        equations(&events)
    );
    assert!((all(&v, "Cr+3") - 0.0004).abs() < 1e-9);
    let mut v = water(0.555);
    put(&mut v, "K2Cr2O7", 0.0002, Phase::Aqueous);
    put(&mut v, "KBr", 0.003, Phase::Aqueous);
    acid(&mut v, 0.05);
    run(&mut v);
    assert!(all(&v, "Cr+3") > 0.0, "{}", all(&v, "Cr+3"));
    assert!(all(&v, "Br2") > 0.0);
}

// ── nitric acid ──────────────────────────────────────────────────────

#[test]
fn dilute_nitric_acid_oxidises_copper_to_nitrogen_monoxide() {
    let mut v = water(0.555);
    put(&mut v, "HNO3", 0.02, Phase::Aqueous);
    put(&mut v, "Cu", 0.003, Phase::Solid);
    let mut events = Vec::new();
    v.solute_charge = displacement::solute_charge(&v);
    assert!(concentrated_sulfuric::apply(&mut v, &mut events));
    assert!(all(&v, "Cu") < 1e-12, "{}", all(&v, "Cu"));
    assert!((all(&v, "Cu(NO3)2") - 0.003).abs() < 1e-9);
    assert!((gas_out(&events, &v, "NO") - 0.002).abs() < 1e-9);
    assert!(gas_out(&events, &v, "NO2") < 1e-12);
}

#[test]
fn ionised_nitric_acid_still_oxidises_copper() {
    // After its first solve, dilute nitric acid is booked as H+ and NO3-;
    // copper added afterwards meets the same oxidant.
    let mut v = water(0.555);
    put(&mut v, "H+", 0.02, Phase::Aqueous);
    put(&mut v, "NO3-", 0.02, Phase::Aqueous);
    put(&mut v, "Cu", 0.003, Phase::Solid);
    let mut events = Vec::new();
    v.solute_charge = displacement::solute_charge(&v);
    assert!(concentrated_sulfuric::apply(&mut v, &mut events));
    assert!(all(&v, "Cu") < 1e-12, "{}", all(&v, "Cu"));
    assert!((gas_out(&events, &v, "NO") - 0.002).abs() < 1e-9);
    // Protons, nitrate, copper and nitrogen close.
    let n =
        all(&v, "NO3-") + all(&v, "HNO3") + 2.0 * all(&v, "Cu(NO3)2") + gas_out(&events, &v, "NO");
    assert!((n - 0.02).abs() < 1e-9, "N {n}");
}

#[test]
fn neat_nitric_acid_oxidises_copper_to_nitrogen_dioxide() {
    // Fuming acid: about 97 % HNO3 by mass.
    let mut v = Vessel::new(VesselId(0), "beaker");
    put(&mut v, "HNO3", 0.05, Phase::Liquid);
    put(&mut v, "water", 0.005, Phase::Liquid);
    put(&mut v, "Cu", 0.005, Phase::Solid);
    let mut events = Vec::new();
    assert!(concentrated_sulfuric::apply(&mut v, &mut events));
    assert!(all(&v, "Cu") < 1e-12, "{}", all(&v, "Cu"));
    assert!((all(&v, "Cu(NO3)2") - 0.005).abs() < 1e-9);
    assert!((gas_out(&events, &v, "NO2") - 0.01).abs() < 1e-9);
    assert!(gas_out(&events, &v, "NO") < 1e-12);
}

#[test]
fn nitric_acid_leaves_a_metal_above_its_couple_alone_and_copper_needs_acid() {
    // Copper in plain water with a nitrate salt and no acid: nothing.
    let mut v = water(0.555);
    put(&mut v, "KNO3", 0.02, Phase::Aqueous);
    put(&mut v, "Cu", 0.003, Phase::Solid);
    let mut events = Vec::new();
    v.solute_charge = displacement::solute_charge(&v);
    concentrated_sulfuric::apply(&mut v, &mut events);
    assert!((all(&v, "Cu") - 0.003).abs() < 1e-12);
    assert!(gas_out(&events, &v, "NO") < 1e-12);
}

#[test]
fn concentrated_sulfuric_acid_frees_nitric_acid_from_nitrate() {
    for (salt, hydrogen_salt) in [("NaNO3", "NaHSO4"), ("KNO3", "KHSO4")] {
        let mut v = Vessel::new(VesselId(0), "beaker");
        put(&mut v, salt, 0.01, Phase::Solid);
        put(&mut v, "H2SO4", 0.05, Phase::Liquid);
        let mut events = Vec::new();
        assert!(concentrated_sulfuric::apply(&mut v, &mut events), "{salt}");
        assert!((all(&v, hydrogen_salt) - 0.01).abs() < 1e-12, "{salt}");
        assert!((all(&v, "HNO3") - 0.01).abs() < 1e-12, "{salt}");
    }
    // Dilute acid: the displacement needs the concentrated acid.
    let mut v = water(1.0);
    put(&mut v, "KNO3", 0.01, Phase::Solid);
    put(&mut v, "H2SO4", 0.02, Phase::Liquid);
    assert!(!concentrated_sulfuric::fires(&v));
}

#[test]
fn concentrated_sulfuric_acid_makes_potassium_hydrogen_sulfate_from_halides() {
    for salt in ["KCl", "KBr"] {
        let mut v = Vessel::new(VesselId(0), "beaker");
        put(&mut v, salt, 0.01, Phase::Solid);
        put(&mut v, "H2SO4", 0.05, Phase::Liquid);
        let mut events = Vec::new();
        assert!(concentrated_sulfuric::apply(&mut v, &mut events), "{salt}");
        assert!((all(&v, "KHSO4") - 0.01).abs() < 1e-12, "{salt}");
        assert!(all(&v, salt) < 1e-12, "{salt}");
    }
}

// ── sulfur(IV) with the new oxidants ─────────────────────────────────

#[test]
fn bromine_oxidises_hydrogen_sulfite() {
    let mut v = water(5.55);
    put(&mut v, "Br2", 1e-4, Phase::Aqueous);
    put(&mut v, "HSO3-", 1e-4, Phase::Aqueous);
    put(&mut v, "Na+", 1e-4, Phase::Aqueous);
    put(&mut v, "H+", 1e-2, Phase::Aqueous);
    put(&mut v, "Cl-", 1e-2, Phase::Aqueous);
    v.solute_charge = displacement::solute_charge(&v);
    assert!(sulfur_iv::other_plan(&v).is_some_and(|plan| plan.run.is_some()));
}

// ── neat nitric acid with no water at all (2026-11-26) ─────────────────

#[test]
fn neat_nitric_acid_with_no_water_still_oxidises_copper() {
    // 5 mL of nitric acid (1.51 g/mL) on copper turnings, nothing else.
    let hno3 = 5.0 * 1.51 / 63.012;
    let mut v = Vessel::new(VesselId(0), "beaker");
    put(&mut v, "HNO3", hno3, Phase::Liquid);
    put(&mut v, "Cu", 0.01, Phase::Solid);
    assert!(concentrated_sulfuric::fires(&v));
    let mut events = Vec::new();
    assert!(concentrated_sulfuric::apply(&mut v, &mut events));
    let gone = 0.01 - all(&v, "Cu");
    assert!(gone > 1e-3, "copper oxidised {gone}");
    // Cu + 4 HNO3 -> Cu(NO3)2 + 2 NO2 + 2 H2O: two NO2 per copper, no NO.
    assert!((gas_out(&events, &v, "NO2") - 2.0 * gone).abs() < 1e-9);
    assert!(gas_out(&events, &v, "NO") < 1e-12);
    // And through the curated pass the bench runs.
    let mut v = Vessel::new(VesselId(0), "beaker");
    put(&mut v, "HNO3", hno3, Phase::Liquid);
    put(&mut v, "Cu", 0.01, Phase::Solid);
    let events = CuratedEquilibrator.equilibrate(&mut v).unwrap();
    assert!(gas_out(&events, &v, "NO2") > 1e-3, "{events:?}");
}

#[test]
fn dry_solids_with_no_liquid_do_not_react() {
    let mut v = Vessel::new(VesselId(0), "beaker");
    put(&mut v, "Cu", 0.01, Phase::Solid);
    put(&mut v, "KNO3", 0.02, Phase::Solid);
    put(&mut v, "Cu(NO3)2", 0.01, Phase::Solid);
    assert!(!concentrated_sulfuric::fires(&v));
    let mut events = Vec::new();
    concentrated_sulfuric::apply(&mut v, &mut events);
    assert!((all(&v, "Cu") - 0.01).abs() < 1e-15);
    assert!(gas_out(&events, &v, "NO2") < 1e-15);
}

// ── dissolved bromine (2026-11-26) ─────────────────────────────────────

fn water_ml(v: &Vessel) -> f64 {
    let n: f64 = v
        .contents
        .iter()
        .filter(|p| p.species.0 == "water" && p.phase == Phase::Liquid)
        .map(|p| p.moles.0)
        .sum();
    species::lookup_key("water")
        .unwrap()
        .liters_from_moles(Moles(n))
        .0
        * 1000.0
}

fn liquid(v: &Vessel, key: &str) -> f64 {
    v.contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == Phase::Liquid)
        .map(|p| p.moles.0)
        .sum()
}

fn dissolved(v: &Vessel, key: &str) -> f64 {
    v.contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == Phase::Aqueous)
        .map(|p| p.moles.0)
        .sum()
}

#[test]
fn bromine_dissolves_up_to_its_solubility_and_the_rest_stays_a_layer() {
    // 3.36 g/100 mL at 25 C (HSDB); 0.1 mol of bromine is 16 g, far more.
    let mut v = water(5.55);
    put(&mut v, "Br2", 0.1, Phase::Liquid);
    let cap = 3.36 * water_ml(&v) / 100.0 / species::lookup_key("Br2").unwrap().molar_mass;
    MixingEquilibrator.equilibrate(&mut v).unwrap();
    assert!(
        (dissolved(&v, "Br2") - cap).abs() < 1e-9,
        "{}",
        dissolved(&v, "Br2")
    );
    assert!((liquid(&v, "Br2") - (0.1 - cap)).abs() < 1e-9);
    // A second pass dissolves nothing more.
    MixingEquilibrator.equilibrate(&mut v).unwrap();
    assert!((dissolved(&v, "Br2") - cap).abs() < 1e-9);
}

#[test]
fn a_little_bromine_dissolves_entirely() {
    let mut v = water(5.55);
    put(&mut v, "Br2", 0.001, Phase::Liquid);
    MixingEquilibrator.equilibrate(&mut v).unwrap();
    assert!((dissolved(&v, "Br2") - 0.001).abs() < 1e-12);
    assert!(liquid(&v, "Br2") < 1e-15);
}

#[test]
fn bromine_with_no_water_stays_liquid_and_excess_separates_again() {
    let mut v = Vessel::new(VesselId(0), "beaker");
    put(&mut v, "Br2", 0.01, Phase::Liquid);
    MixingEquilibrator.equilibrate(&mut v).unwrap();
    assert!((liquid(&v, "Br2") - 0.01).abs() < 1e-15);
    assert!(dissolved(&v, "Br2") < 1e-15);
    // More dissolved than the water holds: the excess is a layer again,
    // with no supersaturation or precipitation claimed.
    let mut v = water(0.555);
    put(&mut v, "Br2", 0.01, Phase::Aqueous);
    let cap = 3.36 * water_ml(&v) / 100.0 / species::lookup_key("Br2").unwrap().molar_mass;
    let events = MixingEquilibrator.equilibrate(&mut v).unwrap();
    assert!((dissolved(&v, "Br2") - cap).abs() < 1e-9);
    assert!((liquid(&v, "Br2") - (0.01 - cap)).abs() < 1e-9);
    assert!(events
        .iter()
        .all(|e| !matches!(e, Event::Supersaturated { .. } | Event::Precipitated { .. })));
}
