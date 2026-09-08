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

/// The unrepresented ionic salt here is IODATE. It has been thiosulfate,
/// then sulfite, and now potassium iodate, and each move was this test
/// doing its job rather than being worked around: a subject stops being a
/// subject when the bench learns to place it.
///
/// `Na2S2O3` went first, when `databases::minteq_v4()` borrowed llnl.dat's
/// thiosulfate couple. `Na2SO3` replaced it, and the comment that stood
/// here said in so many words that sulfite was the honest choice only
/// UNTIL somebody reviewed the borrow - pKa 7.20 sits inside the range a
/// bench works in, so speciating it would move the pH of every vessel that
/// had ever held the salt, and that wanted its own change. That change is
/// the one that rewrote this comment, so sulfite is placed now too.
///
/// **`KIO3` is the honest successor, and the standard it meets is worth
/// stating exactly, because it is the WEAKER of the two standards this
/// file has used.** Iodate is absent from the three datasets this lab
/// ROUTES - `IO3-` appears zero times in minteq.v4, wateq4f and pitzer -
/// and `oxyanion_groups()` has no row for it, so the bench genuinely
/// cannot place it. But it is NOT absent outright: llnl.dat, sit.dat and
/// the Thermoddem database, all vendored in this repository, define it.
/// So iodate sits exactly where sulfite sat, and could be borrowed by the
/// same three-line move if somebody reviews the constants.
///
/// **The stronger standard was searched for and is not available**, which
/// is the part worth recording so the next person does not repeat the
/// search. Of the registry's ionic salts, every anion - iodate,
/// permanganate, nitrate, tetraborate, thiosulfate, sulfite, sulfate,
/// bisulfate, hypochlorite - is defined in at least one vendored database.
/// The single registry species that IS absent from every vendored file is
/// `methyl_orange`, an azo dye no geochemical database carries, and it
/// cannot be this test's subject for an unrelated reason: it is flagged
/// `dissolves_without_speciation`, so it is deliberately treated as a
/// dissolved neutral and never raises the unmapped-ion warning at all. It
/// would test the sugar branch below, not this one.
///
/// So: the property is unchanged and still needs a subject, the subject is
/// the best available rather than the ideal one, and the file says which.
#[test]
fn unknown_ionic_feed_is_distinct_from_a_neutral_molecular_solute() {
    let mut eq = PhreeqcEquilibrator::new().unwrap();
    for scale in [0.01, 0.1, 1.0, 10.0] {
        for acid in [0.0, 1e-4] {
            let mut v = Vessel::new(VesselId(0), "beaker");
            v.deposit(SpeciesId::new("water"), Moles(5.55 * scale), Phase::Liquid);
            v.deposit(SpeciesId::new("KIO3"), Moles(0.001 * scale), Phase::Aqueous);
            if acid > 0.0 {
                v.deposit(SpeciesId::new("HCl"), Moles(acid * scale), Phase::Aqueous);
            }
            assert!(eq.applies(&v));
            let events = eq.equilibrate(&mut v).unwrap();
            assert!(events.iter().any(|e| matches!(e,
                Event::NotYetModeled { what, .. } if what.contains("KIO3"))));
            assert_eq!(
                v.solution.is_some(),
                acid > 0.0,
                "no pure-water pH for the unrepresented ionic mixture"
            );
            assert!(v
                .contents
                .iter()
                .any(|p| p.species.0 == "KIO3" && (p.moles.0 - 0.001 * scale).abs() < 1e-12));
        }
        // The contrast this test is named for: a neutral molecular solute is
        // not accused of being an unmapped ion and does not prevent the
        // solvent's own autoionisation from being characterized.
        let mut sugar = Vessel::new(VesselId(0), "beaker");
        sugar.deposit(SpeciesId::new("water"), Moles(5.55 * scale), Phase::Liquid);
        sugar.deposit(
            SpeciesId::new("sucrose"),
            Moles(0.001 * scale),
            Phase::Aqueous,
        );
        let calls_before = eq.engine_calls();
        let events = eq.equilibrate(&mut sugar).unwrap();
        let solution = sugar.solution.as_ref().expect("water is characterized");
        assert!(solution.ph.is_finite());
        assert!(solution.ionic_strength.is_finite());
        assert_eq!(
            eq.engine_calls(),
            calls_before,
            "solvent-only characterization must remain a constant-time analytic evaluation"
        );
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

#[test]
fn solvent_autoionisation_tracks_temperature_without_native_calls() {
    let mut eq = PhreeqcEquilibrator::new().unwrap();
    let mut readings = Vec::new();
    for temperature in [273.15, 298.15, 323.15] {
        let mut vessel = Vessel::new(VesselId(0), "beaker");
        vessel.temperature = Kelvin(temperature);
        vessel.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
        let events = eq.equilibrate(&mut vessel).unwrap();
        assert!(events.is_empty(), "plain water needs no narrated reaction");
        let solution = vessel.solution.expect("liquid water is characterized");
        assert_eq!(solution.species.len(), 2);
        assert!((solution.species[0].molality - solution.species[1].molality).abs() < 1e-15);
        readings.push(solution.ph);
    }
    assert!((readings[0] - 7.47).abs() < 0.03, "0 C: {readings:?}");
    assert!((readings[1] - 7.00).abs() < 0.01, "25 C: {readings:?}");
    assert!((readings[2] - 6.63).abs() < 0.03, "50 C: {readings:?}");
    assert_eq!(eq.engine_calls(), 0);
}

#[test]
fn solvent_support_does_not_claim_mostly_organic_mixtures_as_chemistry() {
    let eq = PhreeqcEquilibrator::new().unwrap();
    let mut vessel = Vessel::new(VesselId(0), "beaker");
    // Representative post-esterification inventory: water exists, but the
    // liquid medium remains mostly organic and is outside the aqueous model.
    vessel.deposit(SpeciesId::new("water"), Moles(0.067), Phase::Liquid);
    vessel.deposit(SpeciesId::new("ethanol"), Moles(0.033), Phase::Liquid);
    vessel.deposit(SpeciesId::new("ethyl_acetate"), Moles(0.067), Phase::Liquid);
    vessel.deposit(SpeciesId::new("CH3COOH"), Moles(0.033), Phase::Aqueous);

    assert!(!eq.applies(&vessel));
    assert!(!eq.chemistry_applies(&vessel));
}

#[test]
fn solvent_characterization_preserves_honesty_for_unmodelled_spectators() {
    let mut eq = PhreeqcEquilibrator::new().unwrap();
    let mut vessel = Vessel::new(VesselId(0), "beaker");
    vessel.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
    vessel.deposit(SpeciesId::new("PET"), Moles(0.001), Phase::Solid);
    let before = vessel.contents.clone();

    let support_events = eq.equilibrate(&mut vessel).unwrap();
    assert!(
        support_events.is_empty(),
        "solvent support state is not a narrated reaction"
    );
    let events = HonestyEquilibrator.equilibrate(&mut vessel).unwrap();

    assert!(
        vessel.solution.is_some(),
        "the water still has a solution state"
    );
    assert_eq!(
        vessel.contents, before,
        "a partial solve cannot relabel matter"
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::Inert { species, .. }
            if species.0 == "PET")),
        "the solvent answer must not silence the spectator's typed verdict: {events:?}"
    );
}
