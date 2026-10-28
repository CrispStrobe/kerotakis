//! The phreeqc stack hands the core crystallisation ledger the SAME
//! dissolution enthalpy its own heat balance charges (#867 `solid_route`),
//! so crystallisation heat and dissolution heat cannot disagree.

use kerotakis_core::crystallisation;
use kerotakis_phreeqc::enthalpy;

#[test]
fn constructing_the_stack_registers_the_routed_dissolution_pricer() {
    let _stack = kerotakis_phreeqc::PhreeqcEquilibrator::new();
    // KNO3 has no curated registry enthalpy; its price is #867's reviewed
    // llnl.dat Niter reference (-delta_H 35.4794 kJ/mol).
    let kno3 = crystallisation::dissolution_enthalpy_kj("KNO3").expect("KNO3 priced");
    assert!((kno3 - 35.4794).abs() < 0.05, "{kno3}");
    assert_eq!(Some(kno3), enthalpy::dissolution_enthalpy_kj("KNO3"));
    // A curated salt comes back as its registry enthalpy of dissolution.
    let nacl = enthalpy::dissolution_enthalpy_kj("NaCl").expect("NaCl priced");
    assert!((nacl - 3.88).abs() < 1e-6, "{nacl}");
    // Nothing is invented for an unpriced solid.
    assert!(enthalpy::dissolution_enthalpy_kj("sucrose").is_none());
}

/// With the routed price in place, the coupled solve against the Berkeley
/// validation curve releases crystallisation heat at that price; at
/// runtime KNO3 is still refused for want of a validated routed phase.
#[test]
fn the_curve_mechanism_charges_the_routed_price() {
    use kerotakis_core::*;
    let _stack = kerotakis_phreeqc::PhreeqcEquilibrator::new();
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(
        SpeciesId::new("water"),
        Moles(2.775_464_890_369_136_6),
        Phase::Liquid,
    );
    v.deposit(
        SpeciesId::new("K+"),
        Moles(0.296_726_513_107_399_17),
        Phase::Aqueous,
    );
    v.deposit(
        SpeciesId::new("NO3-"),
        Moles(0.296_726_513_107_399_17),
        Phase::Aqueous,
    );
    v.temperature = Kelvin(321.76);
    let curve = crystallisation::curve_for("KNO3").unwrap();
    let from = v.temperature.0;
    let sensible = |t: f64, n: f64| {
        let mut end = v.clone();
        crystallisation::apply(&mut end, curve, n);
        end.energy_between(t, from)
    };
    let owned = crystallisation::cool_against_curve(&v, curve, 12_000.0, &sensible).unwrap();
    let price = enthalpy::dissolution_enthalpy_kj("KNO3").unwrap();
    assert!((owned.heat_j - owned.moles * price * 1000.0).abs() < 1e-6);
    assert!((289.0..294.0).contains(&owned.final_k), "{owned:?}");
    assert!(matches!(
        crystallisation::cool_with_crystallisation(&v, 12_000.0, &sensible),
        Some(Err(crystallisation::Refusal::NoRoutedPhase {
            salt: "KNO3"
        }))
    ));
}
