//! The spectrophotometer refuses unparameterised chromophores (2026-10-27).
//!
//! Iodine read "absorbance 0.0 at 405 nm" in QA-007, KI-011 and RX-007:
//! I2 is curated as coloured but has no reviewed ε(λ), so the Beer–Lambert
//! sum silently skipped it. Every coloured species without ε(λ) is now a
//! missing parameter of the shared instrument coverage disclosure, and the
//! reading is a typed not-modelled refusal instead of a zero.
//!
//! 2026-11-10: iodine itself is now parameterised from cited ε values over
//! the computed triiodide split (`iodine_chemistry`), so the refusal cases
//! below use chlorine, which is still coloured with no reviewed ε(λ). The
//! starch–iodine complex refuses with its own typed reason.

use kerotakis_core::instrument::{InstrumentContract, Spectrophotometer};
use kerotakis_core::solution_optics::{self, ChromophoreClass};
use kerotakis_core::*;

const TABLE: &str = include_str!("../../../docs/chromophore-coverage-20261027.json");

#[test]
fn chromophore_coverage_is_current() {
    let doc: serde_json::Value = serde_json::from_str(TABLE).unwrap();
    let rows = doc["species"].as_array().unwrap();
    let mut seen = std::collections::BTreeSet::new();
    for row in rows {
        let key = row["species"].as_str().unwrap();
        seen.insert(key.to_string());
        assert_eq!(
            solution_optics::chromophore_class(key).key(),
            row["class"].as_str().unwrap(),
            "{key}: runtime classification differs from the generated table"
        );
    }
    for data in species::REGISTRY {
        assert!(
            seen.contains(data.key),
            "{} is registered but missing from docs/chromophore-coverage-20261027.json",
            data.key
        );
    }
}

fn water() -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
    v
}

#[test]
fn dissolved_unparameterised_chromophore_is_refused_not_zero() {
    let mut v = water();
    v.deposit(SpeciesId::new("KCl"), Moles(0.002), Phase::Aqueous);
    v.deposit(SpeciesId::new("Cl2"), Moles(0.0001), Phase::Aqueous);
    let coverage = solution_optics::coverage(&v);
    assert_eq!(coverage.missing_absorptivity, vec!["Cl2"]);
    assert!(Spectrophotometer::default().measure(&v).is_none());
    let disclosure = solution_optics::disclosure(&v);
    assert_eq!(disclosure.missing_parameters, vec!["Cl2"]);
    assert!(disclosure.outside_method.contains(&"KCl".to_string()));
    assert!(!disclosure.is_complete());
}

#[test]
fn dissolved_iodine_in_iodide_is_read_not_refused() {
    let mut v = water();
    v.deposit(SpeciesId::new("KI"), Moles(0.002), Phase::Aqueous);
    v.deposit(SpeciesId::new("I2"), Moles(0.0001), Phase::Aqueous);
    let coverage = solution_optics::coverage(&v);
    assert!(coverage.missing_absorptivity.is_empty());
    assert!(coverage.is_complete());
    let reading = Spectrophotometer::default().measure(&v).expect("reading");
    assert!(reading.value > 0.0);
    let disclosure = solution_optics::disclosure(&v);
    assert_eq!(disclosure.supported, vec!["I2"]);
    assert!(disclosure.is_complete());
}

#[test]
fn iodine_crystals_alone_are_not_in_the_cuvette() {
    let mut v = water();
    v.deposit(SpeciesId::new("I2"), Moles(0.0001), Phase::Solid);
    assert!(solution_optics::coverage(&v)
        .missing_absorptivity
        .is_empty());
    assert_eq!(Spectrophotometer::default().measure(&v).unwrap().value, 0.0);
}

#[test]
fn parameterised_chromophores_keep_their_values() {
    let mut v = water();
    v.deposit(SpeciesId::new("MnO4-"), Moles(1e-6), Phase::Aqueous);
    let reading = Spectrophotometer::default().measure(&v).unwrap();
    assert!(reading.value > 0.0);
    let disclosure = solution_optics::disclosure(&v);
    assert_eq!(disclosure.supported, vec!["MnO4-"]);
    assert!(disclosure.is_complete());
}

#[test]
fn a_parameterised_and_an_unparameterised_chromophore_together_refuse() {
    let mut v = water();
    v.deposit(SpeciesId::new("MnO4-"), Moles(1e-6), Phase::Aqueous);
    v.deposit(SpeciesId::new("Cl2"), Moles(1e-6), Phase::Aqueous);
    // The partial spectrum is still computed for diagnostics...
    assert!(solution_optics::absorbance(&v, 1.0)
        .iter()
        .any(|a| *a > 0.0));
    // ...but it is not reported as the reading.
    assert!(Spectrophotometer::default().measure(&v).is_none());
}

#[test]
fn colour_vocabulary_boundary() {
    assert!(solution_optics::appearance_is_coloured(
        "dark violet crystals"
    ));
    assert!(solution_optics::appearance_is_coloured("pale yellow"));
    assert!(!solution_optics::appearance_is_coloured(
        "white to yellowish powder"
    ));
    assert!(!solution_optics::appearance_is_coloured("colourless"));
    assert!(!solution_optics::appearance_is_coloured(
        "sharp-smelling, toxic"
    ));
    assert_eq!(
        solution_optics::chromophore_class("I2"),
        ChromophoreClass::Parameterised
    );
    assert_eq!(
        solution_optics::chromophore_class("Cl2"),
        ChromophoreClass::MissingAbsorptivity
    );
    assert_eq!(
        solution_optics::chromophore_class("methyl_orange"),
        ChromophoreClass::ParameterisedPh
    );
}

#[test]
fn bench_refusal_is_typed_and_translated() {
    let mut v = water();
    v.deposit(SpeciesId::new("Cl2"), Moles(0.0001), Phase::Aqueous);
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
    // The 16-band peak reading stays refused (no registry band spectrum);
    // since 2026-11-12 Cl2(aq) has cited single-wavelength rows (Fergusson
    // 1936), which are the only readings allowed.
    assert!(!events.iter().any(|e| matches!(
        e,
        Event::Measured { unit, .. } if !unit.ends_with("(cited molar absorptivity)")
    )));
    assert!(events.iter().any(|e| matches!(
        e,
        Event::Measured { unit, .. } if unit == "absorbance at 334 nm (cited molar absorptivity)"
    )));
    let reason = events
        .iter()
        .find_map(|event| match event {
            Event::NotYetModeled {
                reason: Some(reason),
                ..
            } if reason.key == "not-modeled.unreviewed-molar-absorptivity" => Some(reason),
            _ => None,
        })
        .expect("typed refusal");
    for locale in [Locale::parse("de"), Locale::parse("fr")] {
        assert_ne!(reason.render(locale), reason.render(Locale::EN));
    }
}

#[test]
fn every_instrument_declares_its_disclosure() {
    use kerotakis_core::instrument_coverage::{disclosure_of, Disclosure};
    assert_eq!(
        disclosure_of(Instrument::Spectrophotometer).0,
        Disclosure::Shared
    );
    assert_eq!(
        disclosure_of(Instrument::Chromatograph).0,
        Disclosure::OwnEquivalent
    );
}
