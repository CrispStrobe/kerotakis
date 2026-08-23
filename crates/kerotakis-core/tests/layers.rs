//! Oil and water: two liquids in one vessel are not automatically one
//! solution. The computed activity decides — hexane on water layers,
//! ethanol in water does not — and both verdicts come from the same
//! UNIFAC machinery, which is what makes the difference a lesson
//! instead of a lookup.

use kerotakis_core::*;

fn bench_with(pairs: &[(&str, f64)]) -> Vec<Event> {
    let mut bench = Bench::new();
    let mut events = Vec::new();
    events.extend(bench.step(Operator::NewVessel).expect("new"));
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
