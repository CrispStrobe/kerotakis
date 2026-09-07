//! Partial coverage must not become a pure-water answer for an unknown salt.
#![cfg(feature = "engine")]
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

#[test]
fn analytical_proton_and_hydroxide_are_not_unsupported_ionic_feeds() {
    let mut eq = PhreeqcEquilibrator::new().unwrap();
    for reagent in ["HCl", "NaOH"] {
        for amount in [1e-6, 1e-4, 1e-2] {
            let mut v = Vessel::new(VesselId(0), "beaker");
            v.deposit(SpeciesId::new("water"), Moles(55.5), Phase::Liquid);
            v.deposit(SpeciesId::new(reagent), Moles(amount), Phase::Aqueous);
            for _ in 0..2 {
                let events = eq.equilibrate(&mut v).unwrap();
                assert!(v.solution.is_some());
                assert!(!events.iter().any(|e| matches!(e,
                    Event::NotYetModeled { what, .. }
                    if what.contains("ionic solute without an aqueous component mapping"))),
                    "represented {reagent} must not acquire a false missing-ion warning: {events:?}");
            }
        }
    }
}

#[test]
fn unknown_ionic_feed_is_distinct_from_a_neutral_molecular_solute() {
    let mut eq = PhreeqcEquilibrator::new().unwrap();
    for scale in [0.01, 0.1, 1.0, 10.0] {
        for acid in [0.0, 1e-4] {
            let mut v = Vessel::new(VesselId(0), "beaker");
            v.deposit(SpeciesId::new("water"), Moles(5.55 * scale), Phase::Liquid);
            v.deposit(
                SpeciesId::new("Na2S2O3"),
                Moles(0.001 * scale),
                Phase::Aqueous,
            );
            if acid > 0.0 {
                v.deposit(SpeciesId::new("HCl"), Moles(acid * scale), Phase::Aqueous);
            }
            assert!(eq.applies(&v));
            let events = eq.equilibrate(&mut v).unwrap();
            assert!(events.iter().any(|e| matches!(e,
                Event::NotYetModeled { what, .. } if what.contains("Na2S2O3"))));
            assert_eq!(
                v.solution.is_some(),
                acid > 0.0,
                "no pure-water pH for the unrepresented ionic mixture"
            );
            assert!(v
                .contents
                .iter()
                .any(|p| p.species.0 == "Na2S2O3" && (p.moles.0 - 0.001 * scale).abs() < 1e-12));
        }
        let mut sugar = Vessel::new(VesselId(0), "beaker");
        sugar.deposit(SpeciesId::new("water"), Moles(5.55 * scale), Phase::Liquid);
        sugar.deposit(
            SpeciesId::new("sucrose"),
            Moles(0.001 * scale),
            Phase::Aqueous,
        );
        eq.equilibrate(&mut sugar).unwrap();
        assert!(
            sugar.solution.is_some(),
            "neutral solutes retain water characterization"
        );
    }
}
