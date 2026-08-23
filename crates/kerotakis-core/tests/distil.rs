//! Distillation: one computed equilibrium stage, and its honest limits.
//!
//! The vapour composition is not curated — it comes from the bubble point
//! with full UNIFAC γ(T), which is why distilling brine yields pure water
//! and distilling wine-strength ethanol enriches the receiver, while the
//! azeotrope refuses to enrich at all. Each of those is a computed
//! consequence, and each is checked here.

use kerotakis_core::*;

fn bench_with(ops: &[Operator]) -> (Bench, Vec<Event>) {
    let mut bench = Bench::new();
    let mut events = Vec::new();
    for op in ops {
        events.extend(bench.step(op.clone()).expect("operator applies"));
    }
    (bench, events)
}

fn moles_in(bench: &Bench, vessel: usize, key: &str) -> f64 {
    bench.vessels[vessel]
        .contents
        .iter()
        .filter(|p| p.species.0 == key)
        .map(|p| p.moles.0)
        .sum()
}

#[test]
fn distilling_brine_makes_distilled_water() {
    let (bench, events) = bench_with(&[
        Operator::NewVessel,
        Operator::NewVessel,
        Operator::Add {
            vessel: VesselId(1),
            species: SpeciesId::new("water"),
            moles: Moles(5.0),
            at: Some(Kelvin::from_celsius(20.0)),
        },
        Operator::Add {
            vessel: VesselId(1),
            species: SpeciesId::new("NaCl"),
            moles: Moles(0.5),
            at: Some(Kelvin::from_celsius(20.0)),
        },
        Operator::Distil {
            from: VesselId(1),
            to: VesselId(2),
            fraction: 0.5,
        },
    ]);

    let over = events
        .iter()
        .find_map(|e| match e {
            Event::Distilled { water, ethanol, .. } => Some((water.0, ethanol.0)),
            _ => None,
        })
        .expect("brine distils");
    assert!(over.0 > 0.0, "water came over");
    assert!(over.1 == 0.0, "no ethanol existed to come over");

    // The receiver holds only water; every trace of salt stayed behind.
    let salty_in_receiver: f64 = bench.vessels[2]
        .contents
        .iter()
        .filter(|p| p.species.0 != "water")
        .map(|p| p.moles.0)
        .sum();
    assert!(
        salty_in_receiver == 0.0,
        "distilled water must carry no solutes, found {salty_in_receiver} mol"
    );
}

#[test]
fn distilling_dilute_ethanol_enriches_the_receiver() {
    // Wine-strength: x_ethanol ≈ 0.06. One stage should lift the receiver
    // well above the pot — that is what a still is for.
    let (bench, events) = bench_with(&[
        Operator::NewVessel,
        Operator::NewVessel,
        Operator::Add {
            vessel: VesselId(1),
            species: SpeciesId::new("water"),
            moles: Moles(9.4),
            at: Some(Kelvin::from_celsius(20.0)),
        },
        Operator::Add {
            vessel: VesselId(1),
            species: SpeciesId::new("ethanol"),
            moles: Moles(0.6),
            at: Some(Kelvin::from_celsius(20.0)),
        },
        Operator::Distil {
            from: VesselId(1),
            to: VesselId(2),
            fraction: 0.1,
        },
    ]);

    let azeotropic = events
        .iter()
        .find_map(|e| match e {
            Event::Distilled { azeotropic, .. } => Some(*azeotropic),
            _ => None,
        })
        .expect("the mixture distils");
    assert!(!azeotropic, "x = 0.06 is nowhere near the azeotrope");

    let (w2, e2) = (moles_in(&bench, 2, "water"), moles_in(&bench, 2, "ethanol"));
    let x_receiver = e2 / (w2 + e2);
    assert!(
        x_receiver > 0.3,
        "one stage from x = 0.06 should reach x > 0.3 in the receiver \
         (UNIFAC puts y near 0.4), got {x_receiver:.3}"
    );
}

#[test]
fn the_azeotrope_refuses_to_enrich() {
    // At the azeotrope the vapour is the liquid; the receiver's composition
    // matches the pot and the event says so.
    let (bench, events) = bench_with(&[
        Operator::NewVessel,
        Operator::NewVessel,
        Operator::Add {
            vessel: VesselId(1),
            species: SpeciesId::new("ethanol"),
            moles: Moles(8.94),
            at: Some(Kelvin::from_celsius(20.0)),
        },
        Operator::Add {
            vessel: VesselId(1),
            species: SpeciesId::new("water"),
            moles: Moles(1.06),
            at: Some(Kelvin::from_celsius(20.0)),
        },
        Operator::Distil {
            from: VesselId(1),
            to: VesselId(2),
            fraction: 0.3,
        },
    ]);

    let azeotropic = events
        .iter()
        .find_map(|e| match e {
            Event::Distilled { azeotropic, .. } => Some(*azeotropic),
            _ => None,
        })
        .expect("the azeotrope still boils");
    assert!(
        azeotropic,
        "x = 0.894 is the azeotrope and must report as one"
    );

    let (w2, e2) = (moles_in(&bench, 2, "water"), moles_in(&bench, 2, "ethanol"));
    let x_receiver = e2 / (w2 + e2);
    assert!(
        (x_receiver - 0.894).abs() < 0.01,
        "azeotropic vapour matches the liquid, got x = {x_receiver:.3}"
    );
}

#[test]
fn distillation_conserves_matter() {
    let (bench, _) = bench_with(&[
        Operator::NewVessel,
        Operator::NewVessel,
        Operator::Add {
            vessel: VesselId(1),
            species: SpeciesId::new("water"),
            moles: Moles(4.0),
            at: Some(Kelvin::from_celsius(20.0)),
        },
        Operator::Add {
            vessel: VesselId(1),
            species: SpeciesId::new("ethanol"),
            moles: Moles(1.0),
            at: Some(Kelvin::from_celsius(20.0)),
        },
        Operator::Distil {
            from: VesselId(1),
            to: VesselId(2),
            fraction: 0.4,
        },
    ]);
    let w = moles_in(&bench, 1, "water") + moles_in(&bench, 2, "water");
    let e = moles_in(&bench, 1, "ethanol") + moles_in(&bench, 2, "ethanol");
    assert!((w - 4.0).abs() < 1e-9, "water conserved, got {w}");
    assert!((e - 1.0).abs() < 1e-9, "ethanol conserved, got {e}");
}
