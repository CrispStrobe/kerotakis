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

/// The unrepresented ionic salt here is SULFITE, and it used to be
/// thiosulfate.
///
/// That swap is the test doing its job rather than being worked around.
/// `Na2S2O3` was this file's example of "an ion this bench cannot place",
/// and `databases::minteq_v4()` now borrows llnl.dat's thiosulfate couple,
/// so the example stopped being an example: the salt speciates, the vessel
/// gets a solution, and the warning this test looks for is correctly not
/// raised. The property under test is untouched and still needs a subject,
/// so it moves to the sibling salt one oxidation state up.
///
/// `Na2SO3` is the honest choice and not merely the convenient one. llnl
/// defines sulfur(IV) too - `S(+4)  SO3-2` at line 231 and
/// `SO3-2 + H+ = HSO3-`, `log_k 7.2054`, at line 4592 - so it is absent
/// from the three datasets this lab ROUTES rather than absent outright,
/// which is the distinction the thiosulfate work exists to insist on. It is
/// deliberately not borrowed in the same change: pKa 7.20 sits squarely in
/// the range a bench works in, so unlike thiosulfate's 1.01 it would move
/// the pH of every vessel that has ever held it, and that wants its own
/// review rather than a ride on this one. Until it gets one, sulfite is a
/// real unrepresented ion and this test has a real subject.
#[test]
fn unknown_ionic_feed_is_distinct_from_a_neutral_molecular_solute() {
    let mut eq = PhreeqcEquilibrator::new().unwrap();
    for scale in [0.01, 0.1, 1.0, 10.0] {
        for acid in [0.0, 1e-4] {
            let mut v = Vessel::new(VesselId(0), "beaker");
            v.deposit(SpeciesId::new("water"), Moles(5.55 * scale), Phase::Liquid);
            v.deposit(
                SpeciesId::new("Na2SO3"),
                Moles(0.001 * scale),
                Phase::Aqueous,
            );
            if acid > 0.0 {
                v.deposit(SpeciesId::new("HCl"), Moles(acid * scale), Phase::Aqueous);
            }
            assert!(eq.applies(&v));
            let events = eq.equilibrate(&mut v).unwrap();
            assert!(events.iter().any(|e| matches!(e,
                Event::NotYetModeled { what, .. } if what.contains("Na2SO3"))));
            assert_eq!(
                v.solution.is_some(),
                acid > 0.0,
                "no pure-water pH for the unrepresented ionic mixture"
            );
            assert!(v
                .contents
                .iter()
                .any(|p| p.species.0 == "Na2SO3" && (p.moles.0 - 0.001 * scale).abs() < 1e-12));
        }
        // The contrast this test is named for: a neutral molecular solute is
        // not accused of being an unmapped ion. It is a separate question
        // whether a vessel whose only solute has no derived role should be
        // characterised at all — this adapter declines that as it always has,
        // because nothing is dissolved that it can speciate.
        let mut sugar = Vessel::new(VesselId(0), "beaker");
        sugar.deposit(SpeciesId::new("water"), Moles(5.55 * scale), Phase::Liquid);
        sugar.deposit(
            SpeciesId::new("sucrose"),
            Moles(0.001 * scale),
            Phase::Aqueous,
        );
        let events = eq.equilibrate(&mut sugar).unwrap();
        assert!(
            !events.iter().any(|e| matches!(e,
                Event::NotYetModeled { what, .. }
                if what.contains("ionic solute without an aqueous component mapping"))),
            "a neutral solute is not an unmapped ion: {events:?}"
        );
        assert!(
            sugar
                .contents
                .iter()
                .any(|p| p.species.0 == "sucrose" && (p.moles.0 - 0.001 * scale).abs() < 1e-12),
            "and it is still all there"
        );
    }
}
