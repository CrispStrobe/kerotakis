//! The column with real chemistry attached: the aqueous engine dissolves
//! the salt, and the chromatogram then has to say what it did with the
//! ions — they ride the mobile phase unseparated, named outside the
//! method, never silently dropped.

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

#[test]
fn dissolved_ions_are_named_outside_the_method() {
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    for (key, moles) in [("water", 2.0), ("NaCl", 0.1), ("ethanol", 0.05)] {
        bench
            .step_with(
                Operator::Add {
                    vessel: VesselId(0),
                    species: SpeciesId::new(key),
                    moles: Moles(moles),
                    at: None,
                },
                &mut eq,
                &PermissiveScreen,
            )
            .unwrap();
    }
    let events = bench
        .step_with(
            Operator::Measure {
                vessel: VesselId(0),
                instrument: Instrument::Chromatograph,
            },
            &mut eq,
            &PermissiveScreen,
        )
        .unwrap();
    let (peaks, outside) = events
        .iter()
        .find_map(|e| match e {
            Event::Chromatographed {
                peaks,
                outside_method,
                ..
            } => Some((peaks.clone(), outside_method.clone())),
            _ => None,
        })
        .expect("ethanol gives a chromatogram");
    assert!(peaks.iter().any(|p| p.species.0 == "ethanol"));
    assert!(
        !peaks
            .iter()
            .any(|p| p.species.0.contains("Na") || p.species.0.contains("Cl")),
        "no peak may claim the salt"
    );
    assert!(
        outside
            .iter()
            .any(|s| s.0.contains("Na") || s.0.contains("Cl")),
        "the dissolved ions must be named outside the method, got {:?}",
        outside
    );
}

/// aq-087: "Will dissolved salt appear in this neutral-solute chromatography
/// method?" The answer is no — and *no* is an answer, not a gap.
///
/// A sample of nothing but salt water used to be refused: "nothing dissolved
/// here has a curated UNIFAC decomposition, so the column's method is
/// silent". That reports the ENGINE's silence rather than the COLUMN's
/// result, and the information needed to answer properly was already in
/// hand — `outside` holds exactly the species this column cannot separate,
/// and it was being discarded.
///
/// An empty chromatogram is a chromatogram: the run happened, the detector
/// saw nothing, and naming what went past unseparated is the whole of the
/// method's scope.
#[test]
fn a_sample_the_method_cannot_see_is_an_empty_chromatogram_not_a_refusal() {
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    for (key, moles) in [("water", 1.0), ("NaCl", 0.01)] {
        bench
            .step_with(
                Operator::Add {
                    vessel: VesselId(0),
                    species: SpeciesId::new(key),
                    moles: Moles(moles),
                    at: None,
                },
                &mut eq,
                &PermissiveScreen,
            )
            .unwrap();
    }
    let events = bench
        .step_with(
            Operator::Measure {
                vessel: VesselId(0),
                instrument: Instrument::Chromatograph,
            },
            &mut eq,
            &PermissiveScreen,
        )
        .unwrap();

    let (peaks, outside) = events
        .iter()
        .find_map(|e| match e {
            Event::Chromatographed {
                peaks,
                outside_method,
                ..
            } => Some((peaks.clone(), outside_method.clone())),
            _ => None,
        })
        .expect("the column ran, so it reports a chromatogram");
    assert!(
        peaks.is_empty(),
        "nothing this method can separate: {peaks:?}"
    );
    assert!(
        outside
            .iter()
            .any(|s| s.0.contains("Na") || s.0.contains("Cl")),
        "and it names what rode past unseparated: {outside:?}"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::NotYetModeled { .. })),
        "the column answered, so nothing stands aside: {events:?}"
    );
}
