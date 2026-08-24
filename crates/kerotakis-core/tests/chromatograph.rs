//! The column and the funnel must tell one story: K here is the same
//! UNIFAC γ∞(water)/γ∞(alkane) ratio the separating funnel partitions
//! on, so the solute the funnel calls hydrophobic is the one the column
//! retains. Three solutes with three different group inventories elute
//! in the order their groups dictate — nothing in the code ranks them
//! by name.

use kerotakis_core::*;

fn chromatogram_of(pairs: &[(&str, f64)]) -> Vec<Event> {
    // Bench::new() already holds vessel 0, which the grammar calls v1.
    let mut bench = Bench::new();
    for (key, moles) in pairs {
        bench
            .step(Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new(key),
                moles: Moles(*moles),
                at: None,
            })
            .expect("add");
    }
    bench
        .step(
            script::parse_op("chromatograph v1")
                .expect("grammar")
                .expect("an operator"),
        )
        .expect("measure")
}

#[test]
fn solutes_elute_in_the_order_their_groups_dictate() {
    let events = chromatogram_of(&[
        ("water", 2.0),
        ("propanone", 0.05),
        ("methanol", 0.05),
        ("ethanol", 0.10),
    ]);
    let (peaks, void_time) = events
        .iter()
        .find_map(|e| match e {
            Event::Chromatographed {
                peaks, void_time_s, ..
            } => Some((peaks.clone(), *void_time_s)),
            _ => None,
        })
        .expect("a chromatogram");
    let order: Vec<&str> = peaks.iter().map(|p| p.species.0.as_str()).collect();
    assert_eq!(
        order,
        vec!["methanol", "ethanol", "propanone"],
        "one OH and one carbon elutes first; the ketone, happiest in the \
         alkane phase, is retained longest"
    );
    // K strictly increasing with hydrophobicity, and every peak after t0.
    assert!(peaks[0].partition_k < peaks[1].partition_k);
    assert!(peaks[1].partition_k < peaks[2].partition_k);
    assert!(peaks[0].retention_time_s > void_time);
    // The ketone actually partitions into the stationary phase (K > 1);
    // the alcohols mostly ride the water (K < 1).
    assert!(
        peaks[2].partition_k > 1.0,
        "K(propanone) = {}",
        peaks[2].partition_k
    );
    assert!(
        peaks[1].partition_k < 1.0,
        "K(ethanol) = {}",
        peaks[1].partition_k
    );
    // Adjacent peaks resolve: Rs = 2Δt/(w₁+w₂) > 1 on the school column.
    for pair in peaks.windows(2) {
        let rs = 2.0 * (pair[1].retention_time_s - pair[0].retention_time_s)
            / (pair[0].width_s + pair[1].width_s);
        assert!(
            rs > 1.0,
            "{} vs {} resolve at Rs = {rs:.2}",
            pair[0].species.0,
            pair[1].species.0
        );
    }
    // The detector counts moles: ethanol was injected at twice the other
    // two, so it owns the largest peak and the area ratio is 1:2.
    let area = |key: &str| {
        peaks
            .iter()
            .find(|p| p.species.0 == key)
            .map(|p| p.relative_area)
            .unwrap()
    };
    assert!((area("ethanol") - 1.0).abs() < 1e-12);
    assert!((area("methanol") - 0.5).abs() < 1e-9);
    assert!((area("propanone") - 0.5).abs() < 1e-9);
}

#[test]
fn a_settled_solid_is_not_part_of_the_injection() {
    // Core-level NaCl is a solid (the aqueous engine is what dissolves
    // it — the engine-backed test in kerotakis-phreeqc is where the
    // ions get named outside the method). A solid sitting on the bottom
    // is not in the sample loop: no peak, and no outside-method entry
    // either, because it was never injected.
    let events = chromatogram_of(&[("water", 2.0), ("NaCl", 0.1), ("ethanol", 0.05)]);
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
        .expect("ethanol still gives a chromatogram");
    assert!(peaks.iter().any(|p| p.species.0 == "ethanol"));
    assert!(
        !peaks
            .iter()
            .any(|p| p.species.0.contains("Na") || p.species.0.contains("Cl")),
        "no peak may claim the salt"
    );
    assert!(
        outside.is_empty(),
        "a settled solid was never injected, so it is not 'outside the method'"
    );
}

#[test]
fn a_solutefree_sample_refuses_out_loud() {
    let events = chromatogram_of(&[("water", 2.0), ("NaCl", 0.1)]);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::Chromatographed { .. })),
        "nothing separable, so no chromatogram"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::NotYetModeled { .. })),
        "the refusal is spoken, not silent"
    );
}

#[test]
fn a_dry_column_has_no_mobile_phase() {
    let events = chromatogram_of(&[("hexane", 1.0)]);
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::NotYetModeled { .. })));
}

#[test]
fn the_injection_consumes_nothing() {
    let mut bench = Bench::new();
    for (key, moles) in [("water", 2.0), ("ethanol", 0.1), ("propanone", 0.05)] {
        bench
            .step(Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new(key),
                moles: Moles(moles),
                at: None,
            })
            .unwrap();
    }
    let before = format!("{:?}", bench.vessel(VesselId(0)).unwrap().contents);
    bench
        .step(Operator::Measure {
            vessel: VesselId(0),
            instrument: Instrument::Chromatograph,
        })
        .unwrap();
    let after = format!("{:?}", bench.vessel(VesselId(0)).unwrap().contents);
    assert_eq!(before, after, "an analytical injection moves no ledger");
}
