//! BRD-002: pouring PART of a vessel takes the solutes with the solvent.
//!
//! Half a beaker of brine is half the water and half the salt, and what
//! stays behind is at the same concentration it was. The engine already
//! scales every liquid/aqueous portion by one shared `fraction`; these
//! tests pin that behaviour so a later change to `Decant` cannot quietly
//! make a pour concentrate or dilute what it moves.

use kerotakis_core::*;

fn brine() -> Bench {
    let mut bench = Bench::new();
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    bench
        .step(Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new("water"),
            moles: Moles(10.0),
            at: None,
        })
        .unwrap();
    bench
        .step(Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new("NaCl"),
            moles: Moles(0.4),
            at: None,
        })
        .unwrap();
    bench
}

/// Total moles of one species across both vessels — conservation first,
/// because a proportion that does not conserve is not a pour.
fn total(bench: &Bench, key: &str) -> f64 {
    let id = SpeciesId::new(key);
    bench.vessels.iter().map(|v| v.moles_of(&id).0).sum()
}

#[test]
fn decanting_part_of_a_vessel_moves_solute_in_proportion_with_solvent() {
    let mut bench = brine();
    let water_before = total(&bench, "water");
    let salt_before = total(&bench, "NaCl");
    assert!(salt_before > 0.0, "the salt has to be in the ledger to move");

    bench
        .step(Operator::Decant {
            from: VesselId(0),
            to: VesselId(1),
            fraction: 0.25,
        })
        .unwrap();

    let receiver = bench.vessel(VesselId(1)).unwrap();
    let source = bench.vessel(VesselId(0)).unwrap();
    let moved_water = receiver.moles_of(&SpeciesId::new("water")).0;
    let moved_salt = receiver.moles_of(&SpeciesId::new("NaCl")).0;

    assert!(
        (moved_water - 0.25 * water_before).abs() < 1e-9,
        "a quarter pour moves a quarter of the water: {moved_water} of {water_before}"
    );
    assert!(
        (moved_salt - 0.25 * salt_before).abs() < 1e-9,
        "and a quarter of the salt: {moved_salt} of {salt_before}"
    );

    // The concentration on both sides of the pour is the one it started
    // with — which is the whole claim, stated as a ratio.
    let poured_ratio = moved_salt / moved_water;
    let left_ratio = source.moles_of(&SpeciesId::new("NaCl")).0
        / source.moles_of(&SpeciesId::new("water")).0;
    let started_at = salt_before / water_before;
    assert!((poured_ratio - started_at).abs() < 1e-9);
    assert!((left_ratio - started_at).abs() < 1e-9);

    // Nothing was created or destroyed on the way across.
    assert!((total(&bench, "water") - water_before).abs() < 1e-9);
    assert!((total(&bench, "NaCl") - salt_before).abs() < 1e-9);
}

#[test]
fn a_full_pour_leaves_the_source_with_none_of_either() {
    let mut bench = brine();
    let salt_before = total(&bench, "NaCl");
    bench
        .step(Operator::Decant {
            from: VesselId(0),
            to: VesselId(1),
            fraction: 1.0,
        })
        .unwrap();
    let source = bench.vessel(VesselId(0)).unwrap();
    assert!(source.moles_of(&SpeciesId::new("water")).0 < 1e-12);
    assert!(
        source.moles_of(&SpeciesId::new("NaCl")).0 < 1e-12,
        "the solute does not cling to an emptied beaker"
    );
    assert!((total(&bench, "NaCl") - salt_before).abs() < 1e-9);
}
