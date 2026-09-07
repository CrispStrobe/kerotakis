//! Oil and water: two liquids in one vessel are not automatically one
//! solution. The computed activity decides — hexane on water layers,
//! ethanol in water does not — and both verdicts come from the same
//! UNIFAC machinery, which is what makes the difference a lesson
//! instead of a lookup.

use kerotakis_core::*;

fn bench_with(pairs: &[(&str, f64)]) -> Vec<Event> {
    let mut bench = Bench::new();
    let mut events = Vec::new();
    events.extend(bench.step(Operator::NewVessel { kind: None }).expect("new"));
    for (key, moles) in pairs {
        events.extend(
            bench
                .step(Operator::Add {
                    vessel: VesselId(1),
                    species: SpeciesId::new(key),
                    moles: Moles(*moles),
                    at: None,
                })
                .expect("add"),
        );
    }
    events
}

#[test]
fn hexane_layers_on_water() {
    let events = bench_with(&[("water", 2.0), ("hexane", 1.0)]);
    let layered = events.iter().find_map(|e| match e {
        Event::LayersFormed { upper, lower, .. } => Some((upper.0.clone(), lower.0.clone())),
        _ => None,
    });
    let (upper, lower) = layered.expect("water and hexane must layer");
    assert_eq!(upper, "hexane", "hexane floats: density 0.66 vs 1.0");
    assert_eq!(lower, "water");
}

#[test]
fn ethanol_does_not_layer_on_water() {
    let events = bench_with(&[("water", 2.0), ("ethanol", 1.0)]);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::LayersFormed { .. })),
        "ethanol and water are miscible; no layers"
    );
}

#[test]
fn scene_carries_the_partition_while_both_layers_stand() {
    let mut bench = Bench::new();
    for (key, moles) in [("water", 2.0), ("hexane", 1.0), ("ethanol", 0.2)] {
        bench
            .step(Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new(key),
                moles: Moles(moles),
                at: None,
            })
            .unwrap();
    }
    let before = scene(&bench).vessels[0].partition[0].clone();
    assert_eq!(before.species, "ethanol");
    assert_eq!(before.lower_solvent, "water");
    assert_eq!(before.upper_solvent, "hexane");
    assert!((before.lower_moles + before.upper_moles - before.total_moles).abs() < 1e-12);
    assert!(before.fraction_lower > 0.0 && before.fraction_lower < 1.0);
    assert!(before.boundary.contains("no mass-transfer rate"));
    assert!(before.provenance.contains("Fredenslund"));

    // Serialization is the save/reload boundary. There is no event history
    // in the restored vessel from which this answer could be reconstructed.
    let mut restored: Bench =
        serde_json::from_str(&serde_json::to_string(&bench).unwrap()).unwrap();
    assert_eq!(scene(&restored).vessels[0].partition, vec![before.clone()]);

    let expected_fraction = before.fraction_lower;
    let events = restored
        .step(Operator::Drain {
            from: VesselId(0),
            to: VesselId(1),
        })
        .unwrap();
    let event_fraction = events
        .iter()
        .find_map(|event| match event {
            Event::Partitioned {
                species,
                fraction_lower,
                ..
            } if species.0 == "ethanol" => Some(*fraction_lower),
            _ => None,
        })
        .expect("the drain reports the standing split it applied");
    assert!((event_fraction - expected_fraction).abs() < 1e-12);
    assert!(
        scene(&restored)
            .vessels
            .iter()
            .all(|vessel| vessel.partition.is_empty()),
        "after draining, neither separated vessel pretends both layers remain"
    );
}

#[test]
fn draining_takes_the_brine_and_leaves_the_hexane() {
    let mut bench = Bench::new();
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    for (key, moles) in [("water", 2.0), ("NaCl", 0.2), ("hexane", 1.0)] {
        bench
            .step(Operator::Add {
                vessel: VesselId(1),
                species: SpeciesId::new(key),
                moles: Moles(moles),
                at: None,
            })
            .unwrap();
    }
    let events = bench
        .step(Operator::Drain {
            from: VesselId(1),
            to: VesselId(2),
        })
        .unwrap();
    let drained = events.iter().find_map(|e| match e {
        Event::Drained { solvent, moles, .. } => Some((solvent.0.clone(), moles.0)),
        _ => None,
    });
    let (solvent, moles) = drained.expect("the lower layer drains");
    assert_eq!(solvent, "water", "water is the lower layer under hexane");
    assert!(
        (moles - 2.0).abs() < 1e-9,
        "all the water drains, got {moles}"
    );

    // Without the aqueous engine attached, core-level NaCl is a solid —
    // and a stopcock passes liquid, so the solid stays in the funnel
    // with the hexane. (The engine-backed test in kerotakis-phreeqc is
    // where dissolved salt travels with its water.)
    let funnel = &bench.vessels[1];
    let kept: Vec<&str> = funnel
        .contents
        .iter()
        .map(|p| p.species.0.as_str())
        .collect();
    assert!(
        kept.contains(&"hexane") && kept.contains(&"NaCl"),
        "funnel keeps the upper layer and the settled solid, got {kept:?}"
    );
    assert!(
        !kept.contains(&"water"),
        "the water is gone through the stopcock"
    );
}

#[test]
fn draining_one_phase_is_refused_out_loud() {
    let mut bench = Bench::new();
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    for (key, moles) in [("water", 2.0), ("ethanol", 1.0)] {
        bench
            .step(Operator::Add {
                vessel: VesselId(1),
                species: SpeciesId::new(key),
                moles: Moles(moles),
                at: None,
            })
            .unwrap();
    }
    let events = bench
        .step(Operator::Drain {
            from: VesselId(1),
            to: VesselId(2),
        })
        .unwrap();
    assert!(
        !events.iter().any(|e| matches!(e, Event::Drained { .. })),
        "one phase must not drain"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::NotYetModeled { .. })),
        "the refusal is said out loud"
    );
    assert!(bench.vessels[2].is_empty(), "nothing moved on a refusal");
}
