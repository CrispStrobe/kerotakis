//! The separating funnel with real chemistry: dissolved salt is part of
//! its water and drains with it, and the organic layer comes out clean.

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

#[test]
fn dissolved_salt_travels_with_its_water() {
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    for _ in 0..2 {
        bench
            .step_with(Operator::NewVessel, &mut eq, &PermissiveScreen)
            .unwrap();
    }
    for (key, moles) in [("water", 2.0), ("NaCl", 0.2), ("hexane", 1.0)] {
        bench
            .step_with(
                Operator::Add {
                    vessel: VesselId(1),
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
            Operator::Drain {
                from: VesselId(1),
                to: VesselId(2),
            },
            &mut eq,
            &PermissiveScreen,
        )
        .unwrap();
    assert!(
        events.iter().any(|e| matches!(e, Event::Drained { .. })),
        "the brine drains"
    );
    // The funnel keeps only hexane; sodium and chloride went with the
    // water they were dissolved in.
    let funnel = &bench.vessels[1];
    assert!(
        funnel.contents.iter().all(|p| p.species.0 == "hexane"),
        "hexane alone remains, got {:?}",
        funnel
            .contents
            .iter()
            .map(|p| p.species.0.clone())
            .collect::<Vec<_>>()
    );
    let receiver = &bench.vessels[2];
    let salt_in_receiver: f64 = receiver
        .contents
        .iter()
        .filter(|p| p.species.0.contains("Na") || p.species.0.contains("Cl"))
        .map(|p| p.moles.0)
        .sum();
    assert!(
        salt_in_receiver > 0.19,
        "the dissolved salt travels with its water, got {salt_in_receiver} mol"
    );
}
