//! The generic potential-driven redox route (`kerotakis_core::potential_redox`):
//! positive and negative controls for every pair it newly enables, and the
//! typed notes of its dormant rows.

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

fn all(v: &Vessel, key: &str) -> f64 {
    v.contents
        .iter()
        .filter(|p| p.species.0 == key)
        .map(|p| p.moles.0)
        .sum()
}

fn iodine(v: &Vessel) -> f64 {
    all(v, "I2") + all(v, "I3-")
}

// ── iodine from an oxidant in acid ───────────────────────────────────

#[test]
fn manganese_dioxide_and_acidified_iodide_make_iodine() {
    let mut v = water(0.555);
    put(&mut v, "MnO2", 0.002, Phase::Solid);
    put(&mut v, "KI", 0.002, Phase::Aqueous);
    acid(&mut v, 0.01);
    let events = run(&mut v);
    let eq = equations(&events);
    assert!(
        eq.contains(&"MnO₂ + 4 H⁺ + 2 I⁻ → Mn²⁺ + I₂ + 2 H₂O".to_string()),
        "{eq:?}"
    );
    // Iodide is limiting: one I₂ (free or as triiodide) per two iodide.
    assert!(
        (all(&v, "Mn+2") - 0.001).abs() < 1e-9,
        "{}",
        all(&v, "Mn+2")
    );
    let i_total = 2.0 * all(&v, "I2") + 3.0 * all(&v, "I3-") + all(&v, "I-") + all(&v, "KI");
    assert!((i_total - 0.002).abs() < 1e-12, "{i_total}");
    assert!(iodine(&v) > 0.0009 - 1e-6, "{}", iodine(&v));
    assert!(note_keys(&events).contains(&"not-modeled.potential-redox-ideal".to_string()));
}

#[test]
fn manganese_dioxide_and_iodide_without_acid_do_nothing() {
    let mut v = water(0.555);
    put(&mut v, "MnO2", 0.002, Phase::Solid);
    put(&mut v, "KI", 0.002, Phase::Aqueous);
    let events = run(&mut v);
    assert!(equations(&events).is_empty(), "{:?}", equations(&events));
    assert_eq!(all(&v, "MnO2"), 0.002);
    assert_eq!(iodine(&v), 0.0);
}

#[test]
fn acidified_permanganate_oxidises_iodide_and_releases_potassium() {
    let mut v = water(0.555);
    put(&mut v, "KMnO4", 0.0002, Phase::Aqueous);
    put(&mut v, "KI", 0.002, Phase::Aqueous);
    acid(&mut v, 0.01);
    let events = run(&mut v);
    assert!(equations(&events)
        .iter()
        .any(|e| e.starts_with("2 MnO₄⁻ + 16 H⁺ + 10 I⁻")));
    // Permanganate is limiting: all of it reduced, five electrons each.
    assert!(all(&v, "KMnO4") + all(&v, "MnO4-") < 1e-12);
    assert!((all(&v, "Mn+2") - 0.0002).abs() < 1e-12);
    assert!((iodine(&v) - 0.0005).abs() < 1e-9, "{}", iodine(&v));
    // Potassium is conserved: released by the permanganate and by the
    // iodide that reacted, still bound in the iodide that did not.
    let k = all(&v, "K+") + all(&v, "KI") + all(&v, "KMnO4");
    assert!((k - 0.0022).abs() < 1e-12, "{k}");
    assert!(all(&v, "K+") >= 0.0012 - 1e-12, "{}", all(&v, "K+"));
}

#[test]
fn neutral_permanganate_and_iodide_are_not_this_route() {
    // Without acid the acid-medium couple (MnO₄⁻/Mn²⁺) does not apply.
    let mut v = water(0.555);
    put(&mut v, "KMnO4", 0.0002, Phase::Aqueous);
    put(&mut v, "KI", 0.002, Phase::Aqueous);
    let eq = equations(&run(&mut v));
    assert!(!eq.iter().any(|e| e.contains("MnO₄⁻ + 16 H⁺")), "{eq:?}");
}

#[test]
fn iodine_does_not_oxidise_manganese_two() {
    // Negative: no route runs uphill (I₂/I⁻ 0.54 V is far below MnO₂ and
    // MnO₄⁻). Iron(II) is left out: the solved Fe³⁺/I⁻ equilibrium owns it.
    let mut v = water(0.555);
    put(&mut v, "I2", 0.0005, Phase::Aqueous);
    put(&mut v, "Mn+2", 0.001, Phase::Aqueous);
    put(&mut v, "SO4-2", 0.001, Phase::Aqueous);
    acid(&mut v, 0.01);
    run(&mut v);
    assert!(all(&v, "MnO2") < 1e-15);
    assert!(all(&v, "MnO4-") < 1e-15);
    assert!((all(&v, "I2") + all(&v, "I3-") - 0.0005).abs() < 1e-12);
}

#[test]
fn hypochlorous_acid_oxidises_iron_two_in_acid_but_not_without() {
    let mut v = water(0.555);
    put(&mut v, "HClO", 0.0005, Phase::Aqueous);
    put(&mut v, "FeSO4", 0.002, Phase::Aqueous);
    acid(&mut v, 0.01);
    let eq = equations(&run(&mut v));
    assert!(
        eq.contains(&"HClO + H⁺ + 2 Fe²⁺ → 2 Fe³⁺ + Cl⁻ + H₂O".to_string()),
        "{eq:?}"
    );
    assert!(
        (all(&v, "Fe+3") - 0.001).abs() < 1e-9,
        "{}",
        all(&v, "Fe+3")
    );
    assert!(all(&v, "HClO") < 1e-12);
    // No represented acid: the route does not run.
    let mut v = water(0.555);
    put(&mut v, "HClO", 0.0005, Phase::Aqueous);
    put(&mut v, "FeSO4", 0.002, Phase::Aqueous);
    let eq = equations(&run(&mut v));
    assert!(!eq.iter().any(|e| e.starts_with("HClO + H⁺")), "{eq:?}");
}

// ── rows that were dormant until bromine and chromium(III) joined ────

#[test]
fn acidified_permanganate_oxidises_bromide_to_bromine() {
    let mut v = water(0.555);
    put(&mut v, "KMnO4", 0.0002, Phase::Aqueous);
    put(&mut v, "KBr", 0.002, Phase::Aqueous);
    acid(&mut v, 0.01);
    let events = run(&mut v);
    let eq = equations(&events);
    assert!(
        eq.contains(&"2 MnO₄⁻ + 16 H⁺ + 10 Br⁻ → 2 Mn²⁺ + 5 Br₂ + 8 H₂O".to_string()),
        "{eq:?}"
    );
    // Permanganate is limiting: 5/2 Br₂ per MnO₄⁻.
    assert!((all(&v, "Br2") - 0.0005).abs() < 1e-9, "{}", all(&v, "Br2"));
    assert!((all(&v, "Mn+2") - 0.0002).abs() < 1e-9);
    let keys = note_keys(&halogen_redox::unpriced_notes(&v));
    assert!(
        !keys.contains(&"not-modeled.potential-redox-product-unregistered".to_string()),
        "{keys:?}"
    );
}

#[test]
fn acidified_dichromate_oxidises_iron_two_to_chromium_three() {
    let mut v = water(0.555);
    put(&mut v, "K2Cr2O7", 0.0002, Phase::Aqueous);
    put(&mut v, "FeSO4", 0.002, Phase::Aqueous);
    acid(&mut v, 0.01);
    let events = run(&mut v);
    let eq = equations(&events);
    assert!(
        eq.contains(&"Cr₂O₇²⁻ + 14 H⁺ + 6 Fe²⁺ → 2 Cr³⁺ + 6 Fe³⁺ + 7 H₂O".to_string()),
        "{eq:?}"
    );
    assert!(
        (all(&v, "Cr+3") - 0.0004).abs() < 1e-9,
        "{}",
        all(&v, "Cr+3")
    );
    assert!((all(&v, "Fe+3") - 0.0012).abs() < 1e-9);
    assert!(all(&v, "K2Cr2O7") < 1e-12);
    // Without acid the dichromate couple is not admitted.
    let mut v = water(0.555);
    put(&mut v, "K2Cr2O7", 0.0002, Phase::Aqueous);
    put(&mut v, "FeSO4", 0.002, Phase::Aqueous);
    let events = run(&mut v);
    assert!(equations(&events).iter().all(|e| !e.contains("Cr")));
    assert!(all(&v, "Cr+3") < 1e-12);
}

/// C1R-1: hydrogen peroxide reduces acidified dichromate to green
/// chromium(III) and oxygen (the blue CrO(O2)2 intermediate is not booked).
/// The shelf books peroxide as its liquid, so the reductant pool reads it
/// there. Cr, O, H and charge are conserved by the data row.
#[test]
fn hydrogen_peroxide_reduces_acidified_dichromate_to_chromium_three() {
    let mut v = water(0.166);
    put(&mut v, "K2Cr2O7", 0.000068, Phase::Aqueous);
    put(&mut v, "H2O2", 0.00294, Phase::Liquid);
    acid(&mut v, 0.006);
    let events = run(&mut v);
    let eq = equations(&events);
    assert!(
        eq.contains(&"Cr₂O₇²⁻ + 8 H⁺ + 3 H₂O₂ → 2 Cr³⁺ + 3 O₂↑ + 7 H₂O".to_string()),
        "{eq:?}"
    );
    assert!(
        (all(&v, "Cr+3") - 0.000136).abs() < 1e-9,
        "{}",
        all(&v, "Cr+3")
    );
    assert!(all(&v, "K2Cr2O7") < 1e-12);
    // 3 mol peroxide per mol dichromate are spent, the rest remains.
    assert!((all(&v, "H2O2") - (0.00294 - 3.0 * 0.000068)).abs() < 1e-9);
    // Without acid the dichromate couple is not admitted.
    let mut v = water(0.166);
    put(&mut v, "K2Cr2O7", 0.000068, Phase::Aqueous);
    put(&mut v, "H2O2", 0.00294, Phase::Liquid);
    let events = run(&mut v);
    assert!(equations(&events).iter().all(|e| !e.contains("Cr")));
    assert!(all(&v, "Cr+3") < 1e-12);
}

#[test]
fn every_route_is_a_balanced_data_row() {
    for route in potential_redox::ROUTES {
        let eq = potential_redox::equation(route);
        assert!(eq.contains('→'), "{eq}");
        assert!(potential_redox::couple(route.oxidant).is_some());
        assert!(potential_redox::couple(route.reductant).is_some());
    }
}

/// C1R-6 Turnbull's blue: iron(II) with hexacyanoferrate(III). On standard
/// potentials alone (0.358 V < 0.771 V) nothing happens; the Prussian blue
/// solubility product drives the electron transfer, so the route books
/// iron(III) and hexacyanoferrate(II) for the solver to precipitate.
#[test]
fn iron_two_and_hexacyanoferrate_three_transfer_an_electron_for_prussian_blue() {
    const { assert!(potential_redox::E0_FE_CN6 < potential_redox::E0_FE3) };
    let mut v = water(0.555);
    put(&mut v, "FeSO4", 0.0004, Phase::Solid);
    put(&mut v, "K3Fe(CN)6", 0.0003, Phase::Solid);
    let events = run(&mut v);
    assert!(
        equations(&events)
            .iter()
            .any(|e| e.contains("Fe(CN)") && e.contains("Fe")),
        "{events:#?}"
    );
    assert!(
        note_keys(&events).contains(&"not-modeled.potential-redox-precipitate-driven".to_string())
    );
    let fe3 = all(&v, "Fe+3");
    let fcy4 = all(&v, "Fe(CN)6-4");
    // Electron balance: every iron(III) made reduced one hexacyanoferrate(III).
    assert!(fe3 > 1e-4, "iron(III) {fe3}");
    assert!((fe3 - fcy4).abs() < 1e-12, "{fe3} vs {fcy4}");
    // Potassium stays a spectator: three per hexacyanoferrate taken.
    assert!((all(&v, "K+") + 3.0 * all(&v, "K3Fe(CN)6") - 0.0009).abs() < 1e-12);
}

#[test]
fn iron_three_with_hexacyanoferrate_three_or_iron_two_with_two_is_not_this_route() {
    let mut a = water(0.555);
    put(&mut a, "FeCl3", 0.0004, Phase::Solid);
    put(&mut a, "K3Fe(CN)6", 0.0003, Phase::Solid);
    assert!(equations(&run(&mut a))
        .iter()
        .all(|e| !e.contains("Fe(CN)")));
    let mut b = water(0.555);
    put(&mut b, "FeSO4", 0.0004, Phase::Solid);
    put(&mut b, "K4Fe(CN)6", 0.0003, Phase::Solid);
    assert!(equations(&run(&mut b))
        .iter()
        .all(|e| !e.contains("Fe(CN)")));
}

/// C1R-5: acidified dichromate and ethanol. The registry has no
/// acetaldehyde, so no route is posed; the honesty tail names the gap
/// instead of leaving a silent, unreacted beaker.
#[test]
fn acidified_dichromate_and_ethanol_name_the_missing_acetaldehyde() {
    let mut v = water(0.555);
    put(&mut v, "K2Cr2O7", 0.0003, Phase::Solid);
    acid(&mut v, 0.01);
    put(&mut v, "ethanol", 0.005, Phase::Liquid);
    // The honesty tail reads the vessel after a solve, when the ledger
    // carries the acid as solute charge (as `run` books it).
    v.solute_charge = displacement::solute_charge(&v);
    let notes = note_keys(&potential_redox::dormant_notes(&v));
    assert!(
        notes.contains(&"not-modeled.oxidation-product-unregistered".to_string()),
        "{notes:?}"
    );
    // Without acid, or without ethanol, the note is not shown.
    let mut neutral = water(0.555);
    put(&mut neutral, "K2Cr2O7", 0.0003, Phase::Solid);
    put(&mut neutral, "ethanol", 0.005, Phase::Liquid);
    neutral.solute_charge = displacement::solute_charge(&neutral);
    assert!(!note_keys(&potential_redox::dormant_notes(&neutral))
        .contains(&"not-modeled.oxidation-product-unregistered".to_string()));
}
