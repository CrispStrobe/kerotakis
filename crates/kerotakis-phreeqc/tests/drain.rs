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

/// Extraction proper: ethanol in a water/hexane funnel follows its
/// computed partition coefficient — most goes with the water, a
/// measurable remainder stays dissolved in the hexane, and the split
/// comes from the γ∞ ratio, not a table.
#[test]
fn ethanol_partitions_between_the_layers() {
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    for _ in 0..2 {
        bench
            .step_with(Operator::NewVessel, &mut eq, &PermissiveScreen)
            .unwrap();
    }
    for (key, moles) in [("water", 2.0), ("hexane", 1.0), ("ethanol", 0.1)] {
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
    let f = events
        .iter()
        .find_map(|e| match e {
            Event::Partitioned {
                species,
                fraction_lower,
                ..
            } if species.0 == "ethanol" => Some(*fraction_lower),
            _ => None,
        })
        .expect("ethanol partitions");
    assert!(
        (0.70..0.97).contains(&f),
        "ethanol mostly follows the water but measurably not entirely, got {f:.3}"
    );

    let ethanol_in = |v: usize| -> f64 {
        bench.vessels[v]
            .contents
            .iter()
            .filter(|p| p.species.0 == "ethanol")
            .map(|p| p.moles.0)
            .sum()
    };
    let (kept, drained) = (ethanol_in(1), ethanol_in(2));
    assert!(
        (kept + drained - 0.1).abs() < 1e-9,
        "partitioning conserves the solute: {kept} + {drained}"
    );
    assert!(
        kept > 0.003,
        "a real remainder stays in the hexane, got {kept}"
    );
    assert!(drained > kept, "water takes the larger share");
}
