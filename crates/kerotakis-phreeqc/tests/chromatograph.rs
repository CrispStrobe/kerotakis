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
