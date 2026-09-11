//! Faraday's law: charge in, mass out, with the chemistry in the divisor.
//!
//! The cell operator asks what voltage a pair *produces*. This is the other
//! half of the same idea — what a current *moves* — and it is the half with
//! a number a learner can put on a balance.

#![cfg(feature = "engine")]

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

/// Every mole of a gas that left or was caught, by key.
///
/// Both events matter: a stoppered flask reports `GasContained` and an open
/// beaker `GasEvolved`, and an electron ledger that counted only one of
/// them would balance or not depending on the glassware.
fn gas_moles(events: &[Event], key: &str) -> f64 {
    events
        .iter()
        .filter_map(|e| match e {
            Event::GasEvolved { species, moles, .. }
            | Event::GasContained { species, moles, .. }
                if species.0 == key =>
            {
                Some(moles.0)
            }
            _ => None,
        })
        .sum()
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
                current_efficiency,
                ..
            } => Some((
                *coulombs,
                electrons.0,
                moles.0,
                *grams,
                *per_ion,
                *current_efficiency,
            )),
            _ => None,
        })
        .expect("a current through a copper half-cell deposits copper");

    let (coulombs, electrons, moles, grams, per_ion, efficiency) = run;
    // The ion is in ample supply, so this model resolves no loss and says
    // so with a 1 rather than by staying silent.
    assert!((efficiency - 1.0).abs() < 1e-12, "{efficiency}");
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

/// A current cannot deposit an ion that is not there — and does not stop.
///
/// This test used to assert a refusal, and the refusal said the rest of the
/// charge "went nowhere". Nowhere is not somewhere a coulomb can go. A
/// galvanostat holds the current whatever the beaker thinks; the supply of
/// copper limits the DEPOSIT, not the cell. So the metal stops at what the
/// solution held — which was always the point, and still is — and the rest
/// of the charge reduces water, which is the same half-reaction the
/// no-metal brine cell below already uses.
///
/// The observable consequences, in order of how much they matter:
///
/// * the mass on the electrode stops rising, and does not resume;
/// * hydrogen starts coming off the cathode;
/// * the run reports a current efficiency below 1, computed rather than
///   assumed, because the fraction is exactly the fraction.
#[test]
fn charge_beyond_the_supply_becomes_hydrogen_rather_than_metal() {
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

    let (deposited, electrons, efficiency) = events
        .iter()
        .find_map(|e| match e {
            Event::Electrolysed {
                moles,
                electrons,
                current_efficiency,
                ..
            } => Some((moles.0, electrons.0, *current_efficiency)),
            _ => None,
        })
        .expect("a run happened");

    // The metal stopped at the supply. This is the assertion the old
    // refusal test was really making, and it survives the change.
    assert!(
        deposited <= 0.001 + 1e-9,
        "no more copper than the solution held: {deposited}"
    );
    // And the charge did not stop with it.
    assert!(
        (electrons - 7200.0 / 96_485.332_12).abs() < 1e-6,
        "the full charge still passed: {electrons}"
    );
    // 0.001 mol of Cu at two electrons each is 0.002 mol of the 0.0746 mol
    // the charge delivered: about 2.7%.
    let expected = 0.002 / electrons;
    assert!(
        (efficiency - expected).abs() < 2e-3,
        "current efficiency is computed, not assumed: {efficiency} against {expected}"
    );
    assert!(
        efficiency < 0.05,
        "almost all of this charge missed the copper: {efficiency}"
    );
    // The rest of it went somewhere, and where is hydrogen.
    let hydrogen = gas_moles(&events, "H2");
    assert!(
        hydrogen > 0.0,
        "the charge past the copper reduces water: {events:?}"
    );
    assert!(
        (hydrogen - (electrons - 0.002) / 2.0).abs() < 1e-4,
        "two electrons per H2: {hydrogen}"
    );
    // Nothing anywhere may still claim the charge vanished.
    assert!(
        !events.iter().any(|e| matches!(
            e,
            Event::NotYetModeled { what, .. } if what.contains("went nowhere")
        )),
        "charge does not go nowhere: {events:?}"
    );
}

/// The electron ledger closes: every electron the anode released, the
/// cathode took.
///
/// This is the invariant the arithmetic actually rests on, and until now it
/// was only ever asserted in prose. It is worth a test of its own because
/// it is the one that breaks quietly: the anode books against the whole
/// current and the cathode books against what the solution could supply, so
/// a beaker that runs out of ion used to evolve oxygen that nothing paid
/// for. Charge in = electrons = Σ(product × z) at each end, separately.
///
/// Three runs, chosen so the cathode does something different in each:
/// copper all the way, copper then hydrogen, and hydrogen from the start.
#[test]
fn electrons_released_at_the_anode_equal_electrons_taken_at_the_cathode() {
    // Both electrolysis models, and inside each the cathode is made to do
    // something different: the half-cell path with a copper electrode
    // standing in it, and the solvent path with carbon rods and no metal at
    // all. The solvent path is where the bookkeeping was rewritten, so an
    // exhausted run through it is the case this test most needs.
    for (label, steps, amps, seconds) in [
        (
            "half-cell, comfortably supplied",
            &[("water", 5.55), ("CuSO4", 0.05), ("Cu", 0.01)][..],
            0.5,
            1930.0,
        ),
        (
            "half-cell, exhausted part-way",
            &[("water", 5.55), ("CuSO4", 0.001), ("Cu", 0.01)][..],
            2.0,
            3600.0,
        ),
        (
            "no metal electrode, copper exhausted part-way",
            &[("water", 5.55), ("CuSO4", 0.001)][..],
            2.0,
            3600.0,
        ),
        (
            "no metal electrode, water all the way",
            &[("water", 5.55), ("Na2SO4", 0.01)][..],
            0.5,
            600.0,
        ),
    ] {
        let (mut bench, v, mut stack) = bench_with(steps);
        let events = bench
            .step_with(
                Operator::Electrolyse {
                    vessel: v,
                    amps,
                    seconds,
                },
                &mut stack,
                &PermissiveScreen,
            )
            .expect("electrolyse");

        let (electrons, metal, per_ion) = events
            .iter()
            .find_map(|e| match e {
                Event::Electrolysed {
                    electrons,
                    moles,
                    per_ion,
                    ..
                } => Some((electrons.0, moles.0, *per_ion)),
                _ => None,
            })
            .unwrap_or_else(|| panic!("{label}: a run happened"));

        // CATHODE: whatever was named, at its own z, plus any hydrogen.
        // When the named product IS hydrogen the first term already counts
        // it and `gas_moles` would count it twice, so the split is on which
        // product the run reported rather than on the model that ran.
        let named_is_hydrogen = events.iter().any(|e| {
            matches!(e, Event::Electrolysed { species, .. } if species.0 == "H2")
        });
        let cathode = if named_is_hydrogen {
            gas_moles(&events, "H2") * 2.0
        } else {
            metal * per_ion + gas_moles(&events, "H2") * 2.0
        };
        // ANODE: oxygen at four electrons per molecule, chlorine at two.
        let anode = gas_moles(&events, "O2") * 4.0 + gas_moles(&events, "Cl2") * 2.0;

        assert!(
            (cathode - electrons).abs() < 1e-6 * electrons.max(1.0) + 1e-9,
            "{label}: cathode took {cathode} of {electrons} electrons"
        );
        assert!(
            (anode - electrons).abs() < 1e-6 * electrons.max(1.0) + 1e-9,
            "{label}: anode released {anode} of {electrons} electrons"
        );
    }
}

/// The worked example, with the prediction written out by hand.
///
/// This is the whole value of the item: a learner does the arithmetic on
/// paper, runs the bench, and the two agree. If they ever stop agreeing the
/// number that moved is named here rather than blessed from an output.
///
///     I = 1.000 A, t = 965 s
///     Q = I·t          = 965 C
///     n(e⁻) = Q/F      = 965 / 96485.33212 = 1.00015e-2 mol
///     n(Cu) = n(e⁻)/2  = 5.00077e-3 mol        ← the 2 is the chemistry
///     m = n·M          = 5.00077e-3 × 63.546 = 0.31778 g
///
/// Copper(II) is the couple the beaker holds, so z is 2 and not 1; that one
/// factor is the only step a learner cannot get from the ammeter and the
/// clock.
#[test]
fn the_worked_copper_example_agrees_with_arithmetic_done_by_hand() {
    const PREDICTED_G: f64 = 0.317_78;
    let (mut bench, v, mut stack) = bench_with(&[("water", 5.55), ("CuSO4", 0.05), ("Cu", 0.01)]);
    let events = bench
        .step_with(
            Operator::Electrolyse {
                vessel: v,
                amps: 1.0,
                seconds: 965.0,
            },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("electrolyse");
    let (grams, efficiency) = events
        .iter()
        .find_map(|e| match e {
            Event::Electrolysed {
                grams,
                current_efficiency,
                ..
            } => Some((*grams, *current_efficiency)),
            _ => None,
        })
        .expect("copper plates");
    assert!(
        (grams - PREDICTED_G).abs() < 5e-4,
        "hand arithmetic says {PREDICTED_G} g, bench says {grams} g"
    );
    // And the bench is not quietly claiming more than it can support: with
    // the ion in ample supply this model resolves no loss, so it reports a
    // ceiling and the register says it is one.
    assert!(
        (efficiency - 1.0).abs() < 1e-12,
        "no loss is resolved here: {efficiency}"
    );
    let line = kerotakis_core::render::render_event(
        events
            .iter()
            .find(|e| matches!(e, Event::Electrolysed { .. }))
            .unwrap(),
        Register::LV3,
    );
    assert!(
        line.contains("or less, never more"),
        "the ceiling must be stated where the number lives: {line}"
    );
}

/// Brine electrolyses with no metal electrode at all.
///
/// This test used to assert the opposite, and its own comment said why:
/// "brine has no metal electrode here, **and chlorine is not modelled**".
/// It was pinning a limitation, correctly, and the limitation has been
/// lifted — two carbon rods, hydrogen at one and chlorine at the other, is
/// the school cell and it needs no metal.
///
/// The refusal it used to check still exists and still matters, for the
/// case that genuinely has nothing to work with: pure water does not
/// conduct, and `electrolyse_solution.rs` holds that test now.
#[test]
fn brine_electrolyses_with_no_metal_electrode() {
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
    let gas = |key: &str| -> f64 {
        events
            .iter()
            .filter_map(|e| match e {
                Event::GasEvolved { species, moles, .. }
                | Event::GasContained { species, moles, .. }
                    if species.0 == key =>
                {
                    Some(moles.0)
                }
                _ => None,
            })
            .sum()
    };
    // 1.0 A for 60 s is 60 C, so 6.22e-4 mol of electrons: half that as
    // hydrogen, and the same again as chlorine.
    assert!(
        (gas("H2") - 3.11e-4).abs() < 2e-5,
        "hydrogen at the cathode: {events:?}"
    );
    assert!(
        (gas("Cl2") - gas("H2")).abs() < 1e-9,
        "and one chlorine per hydrogen: {events:?}"
    );
    assert!(
        !events.iter().any(|e| matches!(
            e,
            Event::NotYetModeled { what, .. } if what.contains("nothing here can be electrolysed")
        )),
        "there is plenty here to electrolyse: {events:?}"
    );
}
