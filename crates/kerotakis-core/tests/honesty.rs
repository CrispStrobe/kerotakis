//! The bench may not turn a gap in our modelling into a claim about the
//! world.
//!
//! "Nothing ignited" is an observation: it says this substance, held in a
//! flame, does not burn. The engine is only entitled to that sentence when
//! a chemistry solver actually examined the vessel and found no reaction.
//! When no solver claims the state — ethanol has no condensed form in the
//! NASA thermochemical data, so the thermal engine never engages — the
//! honest report is that the lab cannot say.

use kerotakis_core::*;

/// A bench with the physics and honesty passes but no chemistry engine:
/// the situation every unmodelled substance is in.
fn bench_without_chemistry() -> Bench {
    Bench::new()
}

fn ignite_events(species: &str, moles: f64, phase: Phase) -> Vec<Event> {
    let mut bench = bench_without_chemistry();
    let mut solver = SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(HonestyEquilibrator),
    ]);
    let screen = PermissiveScreen;
    let v = VesselId(0);
    bench
        .step_with(
            Operator::Add {
                vessel: v,
                species: SpeciesId::new(species),
                moles: Moles(moles),
                at: None,
            },
            &mut solver,
            &screen,
        )
        .expect("add");
    let _ = phase;
    bench
        .step_with(Operator::Ignite { vessel: v }, &mut solver, &screen)
        .expect("ignite")
}

#[test]
fn an_unexamined_substance_is_not_reported_as_incombustible() {
    let events = ignite_events("ethanol", 0.17, Phase::Liquid);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::DidNotIgnite { .. })),
        "with no combustion solver wired, the bench must not claim the \
         substance failed to ignite: {events:?}"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::NotYetModeled { .. })),
        "it must say so instead: {events:?}"
    );
}

#[test]
fn an_empty_vessel_really_does_not_ignite() {
    // The one case where "nothing ignited" needs no solver: there is
    // nothing there. This must not regress into a modelling apology.
    let mut bench = bench_without_chemistry();
    let mut solver = SolverStack::new(vec![Box::new(MixingEquilibrator)]);
    let events = bench
        .step_with(
            Operator::Ignite {
                vessel: VesselId(0),
            },
            &mut solver,
            &PermissiveScreen,
        )
        .expect("ignite");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::DidNotIgnite { .. })),
        "{events:?}"
    );
}

/// A vessel cannot give up more heat than it holds, and must say when asked.
///
/// `cool` clamped at absolute zero and said nothing, so a request for more
/// energy than the contents contained was silently granted: two grams of
/// magnesia at 2769 °C, asked for 10 kJ it did not have, came back at
/// exactly −273.15 °C as though that were an answer. No coolant is
/// modelled — nothing here sets how cold the surroundings are — so the
/// vessel's own heat content is the only bound available, and running past
/// it has to be said rather than absorbed.
#[test]
fn cooling_past_what_the_vessel_holds_is_stated() {
    let mut bench = bench_without_chemistry();
    let mut solver = SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(HonestyEquilibrator),
    ]);
    let screen = PermissiveScreen;
    let v = VesselId(0);
    bench
        .step_with(
            Operator::Add {
                vessel: v,
                species: SpeciesId::new("water"),
                moles: Moles(1.0),
                at: None,
            },
            &mut solver,
            &screen,
        )
        .expect("add");
    let events = bench
        .step_with(
            Operator::Cool {
                vessel: v,
                energy: Joules(1.0e6),
            },
            &mut solver,
            &screen,
        )
        .expect("cool");
    assert!(
        bench.vessel(v).expect("vessel").temperature.0 >= 0.0,
        "nothing may go below absolute zero"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::NotYetModeled { what, .. } if what.contains("before absolute zero")
        )),
        "asking for more heat than the vessel holds must be stated, not granted by \
         a silent clamp: {events:?}"
    );
}

/// Evaporating the last of the solvent leaves dissolved matter with nothing
/// to be dissolved in, and the bench has to say so.
///
/// Unlike evaporating to 99%, where all three databases refuse the
/// resulting 100 mol/kgw brine out loud, this one arrives in silence: with
/// no water left, no aqueous solver applies, so nothing is there to object.
/// The bench cannot repair it by crystallising either — which solids form
/// is not decidable from the ions alone — so it names what it is holding
/// and what it cannot decide.
#[test]
fn evaporating_to_dryness_says_the_ions_are_stranded() {
    let mut bench = bench_without_chemistry();
    let mut solver = SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(HonestyEquilibrator),
    ]);
    let screen = PermissiveScreen;
    let v = VesselId(0);
    for (species, moles) in [("water", 5.55), ("Na+", 0.1), ("Cl-", 0.1)] {
        bench
            .step_with(
                Operator::Add {
                    vessel: v,
                    species: SpeciesId::new(species),
                    moles: Moles(moles),
                    at: None,
                },
                &mut solver,
                &screen,
            )
            .expect("add");
    }
    let events = bench
        .step_with(
            Operator::Evaporate {
                vessel: v,
                fraction: 1.0,
            },
            &mut solver,
            &screen,
        )
        .expect("evaporate");
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::NotYetModeled { what, .. } if what.contains("not a state a beaker can be in")
        )),
        "dissolved ions with no solvent must be named as impossible: {events:?}"
    );
}
