//! BRD-002: pouring PART of a vessel takes the dissolved matter with the
//! solvent.
//!
//! Half a beaker of brine is half the water and half the salt, and what
//! stays behind is at the same concentration it was. The engine already
//! scales every liquid/aqueous portion by one shared `fraction`; these
//! tests pin that behaviour so a later change to `Decant` cannot quietly
//! make a pour concentrate or dilute what it moves.
//!
//! The solutes are deposited as `Phase::Aqueous` directly rather than by
//! dissolving a salt, because dissolution is the aqueous solver's job and
//! this file is about the *transfer*. Adding `NaCl` through the core-only
//! stack leaves it a solid — which a decant then correctly leaves behind,
//! and which would test the opposite of what is claimed here.

use kerotakis_core::species::Phase;
use kerotakis_core::*;

/// Water with two dissolved ions and one undissolved solid at the bottom.
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
    let source = bench
        .vessels
        .iter_mut()
        .find(|vessel| vessel.id == VesselId(0))
        .expect("v1 is on the bench");
    source.deposit(SpeciesId::new("Na+"), Moles(0.4), Phase::Aqueous);
    source.deposit(SpeciesId::new("Cl-"), Moles(0.4), Phase::Aqueous);
    // Undissolved solid, which a pour must NOT take with it.
    source.deposit(SpeciesId::new("NaCl"), Moles(0.05), Phase::Solid);
    bench
}

/// Total moles of one species across the whole bench — conservation first,
/// because a proportion that does not conserve is not a pour.
fn total(bench: &Bench, key: &str) -> f64 {
    let id = SpeciesId::new(key);
    bench.vessels.iter().map(|v| v.moles_of(&id).0).sum()
}

fn held(bench: &Bench, vessel: VesselId, key: &str) -> f64 {
    bench
        .vessel(vessel)
        .unwrap()
        .moles_of(&SpeciesId::new(key))
        .0
}

#[test]
fn decanting_part_of_a_vessel_moves_solute_in_proportion_with_solvent() {
    let mut bench = brine();
    let water_before = total(&bench, "water");
    let sodium_before = total(&bench, "Na+");
    assert!(
        sodium_before > 0.0,
        "the solute has to be dissolved to move"
    );

    bench
        .step(Operator::Decant {
            from: VesselId(0),
            to: VesselId(1),
            fraction: 0.25,
        })
        .unwrap();

    let moved_water = held(&bench, VesselId(1), "water");
    let moved_sodium = held(&bench, VesselId(1), "Na+");
    let moved_chloride = held(&bench, VesselId(1), "Cl-");

    assert!(
        (moved_water - 0.25 * water_before).abs() < 1e-9,
        "a quarter pour moves a quarter of the water: {moved_water} of {water_before}"
    );
    assert!(
        (moved_sodium - 0.25 * sodium_before).abs() < 1e-9,
        "and a quarter of the sodium: {moved_sodium} of {sodium_before}"
    );
    assert!((moved_chloride - 0.25 * sodium_before).abs() < 1e-9);

    // The concentration on both sides of the pour is the one it started
    // with — which is the whole claim, stated as a ratio.
    let started_at = sodium_before / water_before;
    let poured_ratio = moved_sodium / moved_water;
    let left_ratio = held(&bench, VesselId(0), "Na+") / held(&bench, VesselId(0), "water");
    assert!(
        (poured_ratio - started_at).abs() < 1e-9,
        "what was poured is at the original concentration"
    );
    assert!(
        (left_ratio - started_at).abs() < 1e-9,
        "and so is what stayed behind"
    );

    // Nothing was created or destroyed on the way across.
    assert!((total(&bench, "water") - water_before).abs() < 1e-9);
    assert!((total(&bench, "Na+") - sodium_before).abs() < 1e-9);
}

#[test]
fn a_decant_leaves_undissolved_solid_behind() {
    // The other half of the same rule: proportional applies to what is IN
    // the liquid. A solid sitting on the bottom is what decanting is for.
    let mut bench = brine();
    let solid_before = total(&bench, "NaCl");
    bench
        .step(Operator::Decant {
            from: VesselId(0),
            to: VesselId(1),
            fraction: 0.5,
        })
        .unwrap();
    assert!(held(&bench, VesselId(1), "NaCl") < 1e-12, "no solid poured");
    assert!((held(&bench, VesselId(0), "NaCl") - solid_before).abs() < 1e-12);
}

#[test]
fn a_full_pour_takes_all_the_solution_and_none_of_the_solid() {
    let mut bench = brine();
    let sodium_before = total(&bench, "Na+");
    let solid_before = total(&bench, "NaCl");
    bench
        .step(Operator::Decant {
            from: VesselId(0),
            to: VesselId(1),
            fraction: 1.0,
        })
        .unwrap();
    assert!(held(&bench, VesselId(0), "water") < 1e-12);
    assert!(
        held(&bench, VesselId(0), "Na+") < 1e-12,
        "dissolved matter does not cling to an emptied beaker"
    );
    assert!((held(&bench, VesselId(1), "Na+") - sodium_before).abs() < 1e-9);
    assert!((held(&bench, VesselId(0), "NaCl") - solid_before).abs() < 1e-12);
    assert!((total(&bench, "Na+") - sodium_before).abs() < 1e-9);
}
