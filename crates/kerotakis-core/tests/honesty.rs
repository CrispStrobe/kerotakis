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
