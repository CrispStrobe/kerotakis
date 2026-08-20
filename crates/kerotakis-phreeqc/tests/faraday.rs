//! Faraday's law: charge in, mass out, with the chemistry in the divisor.
//!
//! The cell operator asks what voltage a pair *produces*. This is the other
//! half of the same idea — what a current *moves* — and it is the half with
//! a number a learner can put on a balance.

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

/// The real stack: the electrode's potential is Nernst over the *activity*
/// the speciation reports, so this needs the engine rather than a stub.
fn bench_with(steps: &[(&str, f64)]) -> (Bench, VesselId, SolverStack) {
    let mut bench = Bench::new();
    let mut stack = SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(CuratedEquilibrator),
        Box::new(DisplacementEquilibrator::wrapping(Box::new(
            PhreeqcEquilibrator::new().expect("engine"),
        ))),
        Box::new(StateEquilibrator),
        Box::new(HonestyEquilibrator),
    ]);
    let v = VesselId(0);
    for (species, moles) in steps {
        bench
            .step_with(
                Operator::Add {
                    vessel: v,
                    species: SpeciesId::new(species),
                    moles: Moles(*moles),
                    at: None,
                },
                &mut stack,
                &PermissiveScreen,
            )
            .expect("add");
    }
    (bench, v, stack)
}

/// One faraday deposits one mole of a singly-charged metal and half a mole
/// of a doubly-charged one. That factor is the whole lesson.
///
/// 0.5 A for 1930 s is 965 C, which is 0.01 mol of electrons — and copper
/// is Cu²⁺, so 0.005 mol of copper, 0.318 g. The arithmetic is one
/// division; knowing it is 2 and not 1 is the chemistry, and the bench
/// reads that from the couple the vessel actually holds rather than being
/// told.
#[test]
fn charge_becomes_mass_through_the_couples_own_electron_count() {
    let (mut bench, v, mut stack) = bench_with(&[("water", 5.55), ("CuSO4", 0.05), ("Cu", 0.01)]);
    let events = bench
        .step_with(
            Operator::Electrolyse {
                vessel: v,
                amps: 0.5,
                seconds: 1930.0,
            },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("electrolyse");

    let run = events
        .iter()
        .find_map(|e| match e {
            Event::Electrolysed {
                coulombs,
                electrons,
                moles,
                grams,
                per_ion,
                ..
            } => Some((*coulombs, electrons.0, moles.0, *grams, *per_ion)),
            _ => None,
        })
        .expect("a current through a copper half-cell deposits copper");

    let (coulombs, electrons, moles, grams, per_ion) = run;
    assert!((coulombs - 965.0).abs() < 1e-9, "Q = I·t: {coulombs}");
    assert!((electrons - 0.01).abs() < 1e-4, "n(e⁻) = Q/F: {electrons}");
    assert_eq!(per_ion, 2.0, "copper(II) takes two electrons");
    assert!((moles - 0.005).abs() < 1e-4, "n(Cu) = n(e⁻)/2: {moles}");
    assert!((grams - 0.3178).abs() < 1e-3, "m = n·M: {grams}");

    // The other electrode has to be somewhere. Taking copper out of
    // solution leaves its charge behind and the solve balances that with
    // acid — pH 4.27 to 1.84 on 0.01 mol of electrons — which is the right
    // chemistry for an inert anode, 2 H₂O → O₂ + 4 H⁺ + 4 e⁻. The acid was
    // arriving without the oxygen that pays for it, so the oxygen is booked
    // and the water it came from is spent. Four electrons per O₂.
    let oxygen = events
        .iter()
        .find_map(|e| match e {
            Event::GasEvolved { species, moles, .. } if species.0 == "O2" => Some(moles.0),
            _ => None,
        })
        .expect("an inert anode oxidises the water, and the oxygen leaves");
    assert!(
        (oxygen - electrons / 4.0).abs() < 1e-6,
        "four electrons per O2: {oxygen} against {}",
        electrons / 4.0
    );

    // And the matter actually moved: the ion paid for the metal.
    let vessel = bench.vessel(v).expect("vessel");
    let solid: f64 = vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == "Cu" && p.phase == Phase::Solid)
        .map(|p| p.moles.0)
        .sum();
    assert!(
        (solid - 0.015).abs() < 1e-4,
        "0.010 mol of electrode plus 0.005 plated, got {solid}"
    );
}

/// A current cannot deposit an ion that is not there.
///
/// Past the supply a real cell starts electrolysing the water instead,
/// which this bench does not model — so it stops at what the solution held
/// and says so, rather than inventing metal to match the charge.
#[test]
fn charge_beyond_the_supply_is_refused_rather_than_invented() {
    let (mut bench, v, mut stack) = bench_with(&[("water", 5.55), ("CuSO4", 0.001), ("Cu", 0.001)]);
    let events = bench
        .step_with(
            Operator::Electrolyse {
                vessel: v,
                amps: 2.0,
                seconds: 3600.0, // 7200 C — far more than 0.001 mol can take
            },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("electrolyse");
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::NotYetModeled { what, .. } if what.contains("electrolysing the water")
        )),
        "running past the ion supply must be stated: {events:?}"
    );
    let deposited: f64 = events
        .iter()
        .find_map(|e| match e {
            Event::Electrolysed { moles, .. } => Some(moles.0),
            _ => None,
        })
        .unwrap_or(0.0);
    assert!(
        deposited <= 0.001 + 1e-9,
        "no more copper than the solution held: {deposited}"
    );
}

/// A beaker with no electrode says so instead of guessing one.
#[test]
fn a_vessel_with_no_electrode_is_refused() {
    let (mut bench, v, mut stack) = bench_with(&[("water", 5.55), ("NaCl", 0.1)]);
    let events = bench
        .step_with(
            Operator::Electrolyse {
                vessel: v,
                amps: 1.0,
                seconds: 60.0,
            },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("electrolyse");
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::NotYetModeled { what, .. } if what.contains("nothing here can be electrolysed")
        )),
        "brine has no metal electrode here, and chlorine is not modelled: {events:?}"
    );
}
