//! A match held over a fizzing beaker does not set the beaker on fire.
//!
//! The bench used to read "something was consumed or a gas came off during
//! the ignite step" as ignition. A beaker of vinegar and baking soda fizzes
//! on every step whether a match is held over it or not — the aqueous tail
//! leaves bicarbonate and acetic acid coexisting, and the curated route
//! reacts what it finds — so that CO₂ was taken as fire: the spark's 1200 K
//! stayed, the water boiled, and a lesson logged "388 °C" over a beaker in
//! which nothing can burn. Only a combustion engine's `ThermalEquilibrium`
//! carrying released energy says something burned.

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn stack() -> SolverStack {
    SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
        PhreeqcEquilibrator::new().expect("engine"),
    )]))
}

fn add(bench: &mut Bench, stack: &mut SolverStack, key: &str, moles: f64) -> Vec<Event> {
    bench
        .step_with(
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new(key),
                moles: Moles(moles),
                at: None,
            },
            stack,
            &PermissiveScreen,
        )
        .expect("add")
}

#[test]
fn a_match_over_fizzing_vinegar_does_not_cook_the_beaker() {
    let mut bench = Bench::new();
    let mut stack = stack();
    // Excess acid over the bicarbonate, as in the lesson: the tail keeps
    // some HCO₃⁻ beside the acetic acid, and each later step fizzes a
    // little more.
    add(&mut bench, &mut stack, "water", 19.4);
    add(&mut bench, &mut stack, "NaHCO3", 0.06);
    add(&mut bench, &mut stack, "CH3COOH", 0.08);
    let before = bench.vessel(VesselId(0)).unwrap().temperature.0;

    let events = bench
        .step_with(
            Operator::Ignite {
                vessel: VesselId(0),
            },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("ignite");

    let after = bench.vessel(VesselId(0)).unwrap().temperature.0;
    assert!(
        (after - before).abs() < 2.0,
        "nothing in this beaker burns, so the spark is put back out: \
         {before:.1} K → {after:.1} K; {events:?}"
    );
    assert!(
        !events.iter().any(|e| matches!(e, Event::Ignited { .. })),
        "a fizz is not a fire: {events:?}"
    );
    assert!(
        !events.iter().any(|e| matches!(
            e,
            Event::TemperatureChanged { to, .. } if to.0 > 373.15
        )),
        "no announcement of a boil the flame did not cause: {events:?}"
    );
    // The water is still water, in the beaker, as liquid.
    let water = bench
        .vessel(VesselId(0))
        .unwrap()
        .contents
        .iter()
        .filter(|p| p.species.0 == "water" && p.phase == kerotakis_core::species::Phase::Liquid)
        .map(|p| p.moles.0)
        .sum::<f64>();
    assert!(water > 19.0, "the water did not boil away: {water} mol");
}
