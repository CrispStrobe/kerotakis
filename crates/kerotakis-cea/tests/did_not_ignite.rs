//! What "nothing happened" is made of.
//!
//! `Event::DidNotIgnite` carried a vessel id and nothing else, which made
//! it the one row of the animation audit that could not be closed from
//! the client: with no quantity on the wire there is nothing for a visual
//! to be a function of, and anything drawn would have been a picture of
//! the word. Its sibling `FlameStarved` carries the fuel, what burned and
//! the oxygen fraction, and is drawn from those.
//!
//! These tests are about the two ends of the new `reason`. A sealed flask
//! of spent air was never going to burn and there is nothing to draw.
//! Propane in a heated bath, sparked and quenched, did not burn for a
//! reason with a number in it — and the number is on the event.
//!
//! Both vessels are held at a bath temperature, and that is not a trick
//! to make a test pass. `ignite` takes a vessel to 1200 K, above every
//! tabulated autoignition point, so a sparked vessel is examined at a
//! temperature no bench ever sits at; a thermostat pulls it straight back
//! and the solvers see the vessel the learner is left with, which is the
//! state these fields describe.

use kerotakis_cea::ThermalEquilibrator;
use kerotakis_core::ops::DidNotIgniteReason;
use kerotakis_core::vessel::{Headspace, ThermalMode};
use kerotakis_core::*;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(ThermalEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn ignite(bench: &mut Bench, stack: &mut SolverStack, v: VesselId) -> Vec<Event> {
    bench
        .step_with(Operator::Ignite { vessel: v }, stack, &PermissiveScreen)
        .expect("ignite")
}

/// The absence, and everything it says about itself.
fn absence(events: &[Event]) -> (DidNotIgniteReason, Option<String>, Option<f64>, Option<f64>) {
    events
        .iter()
        .find_map(|event| match event {
            Event::DidNotIgnite {
                reason,
                fuel,
                oxygen_fraction,
                gap_k,
                ..
            } => Some((
                *reason,
                fuel.as_ref().map(|f| f.0.clone()),
                *oxygen_fraction,
                *gap_k,
            )),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no DidNotIgnite among {events:?}"))
}

/// A sealed flask of the air a fire has already used: nothing in there
/// is a fuel, and that is the reason.
///
/// This is the case the old sentence — "not everything burns" — was
/// actually true of, and it is now the only one it is said about. The
/// client draws nothing for it, which is the point: there is no fuel to
/// scale a wisp by, so the event says so rather than offering a zero.
///
/// **It is deliberately not a beaker of water**, which is what this test
/// was first written as, and what the engine answers there is worth
/// recording. Water held in a flame goes past the aqueous model's 300 °C
/// ceiling, no chemistry solver claims the state, and the bench says
/// `NotYetModeled` rather than `DidNotIgnite` — it may not turn a gap in
/// the modelling into a claim that water does not burn. So the honest
/// `NoFuel` needs a vessel a solver actually examined, and a warm flask
/// of exhaust gas is one: CEA can name both species and finds nothing to
/// do with them.
#[test]
fn a_flask_of_spent_air_has_no_fuel_in_it() {
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    {
        let vessel = &mut bench.vessels[0];
        vessel.headspace = Headspace::Sealed {
            volume: Liters(1.0),
        };
        // Held at its bath, for the same reason the propane case below is:
        // the spark is quenched, so what the solver examines is the warm
        // vessel rather than a flash of 1200 K.
        vessel.thermal_mode = ThermalMode::Thermostatted(Kelvin(600.0));
        vessel.temperature = Kelvin(600.0);
        vessel.deposit(SpeciesId::new("N2"), Moles(0.04), Phase::Gas);
        vessel.deposit(SpeciesId::new("CO2"), Moles(0.01), Phase::Gas);
        vessel.refresh_pressure();
    }

    let events = ignite(&mut bench, &mut stack, v);
    let (reason, fuel, oxygen, gap) = absence(&events);

    assert_eq!(reason, DidNotIgniteReason::NoFuel, "{events:?}");
    assert_eq!(fuel, None, "there is no candidate to name");
    assert_eq!(gap, None, "and so no gap to any temperature");
    // The oxygen fraction is still a true reading, and a sealed vessel
    // owns its gas: there is none in there at all.
    let oxygen = oxygen.expect("a sealed vessel can report its own gas");
    assert!(oxygen < 1e-9, "spent air has no oxygen left: {oxygen}");
}

/// Propane in a bath at 600 K, sparked: the bath swallows the spark, the
/// thermal solver says the fuel is below its autoignition temperature,
/// and the absence carries the gap.
///
/// The mechanism is worth stating because it is what makes this
/// reachable at all. `ignite` takes a vessel to 1200 K, which is above
/// every tabulated autoignition temperature, so a sparked mixture is
/// normally never "not hot enough". A thermostatted vessel is: the bath
/// pulls it straight back to 600 K before any chemistry runs, and what
/// the solver then meets is a warm, unsparked fuel — 143 K short of
/// catching, which is the number a client draws the wisp from.
#[test]
fn propane_in_a_quenching_bath_is_short_of_its_autoignition_temperature() {
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    {
        let vessel = &mut bench.vessels[0];
        vessel.headspace = Headspace::Sealed {
            volume: Liters(1.0),
        };
        vessel.thermal_mode = ThermalMode::Thermostatted(Kelvin(600.0));
        vessel.temperature = Kelvin(600.0);
        vessel.deposit(SpeciesId::new("propane"), Moles(0.01), Phase::Gas);
        vessel.deposit(SpeciesId::new("O2"), Moles(0.05), Phase::Gas);
        vessel.refresh_pressure();
    }

    let events = ignite(&mut bench, &mut stack, v);
    let (reason, fuel, oxygen, gap) = absence(&events);

    assert_eq!(reason, DidNotIgniteReason::BelowAutoignition, "{events:?}");
    assert_eq!(fuel.as_deref(), Some("propane"));
    let gap = gap.expect("the gap is the answer");
    // Propane's tabulated autoignition temperature is 743.15 K.
    assert!((gap - 143.15).abs() < 1e-6, "{gap}");
    let oxygen = oxygen.expect("a sealed vessel owns its gas and can report it");
    assert!(oxygen > 0.5, "five parts oxygen to one of fuel: {oxygen}");

    // The fuel is untouched: an absence may not consume anything.
    assert!(
        (bench.vessels[0].moles_of(&SpeciesId::new("propane")).0 - 0.01).abs() < 1e-12,
        "nothing burned"
    );
    // And the spark left nothing behind — the vessel is back in its bath.
    assert!((bench.vessels[0].temperature.0 - 600.0).abs() < 1e-9);
}

/// The fuel that would have caught first is the one named.
///
/// Both tables are read — the vapours `GAS_AUTOIGNITION` gates and the
/// curated `FUELS` NASA CEA cannot name — and the lowest autoignition
/// temperature wins, because that is the one a rising flame reaches
/// first. Reading only the gas table would have called a candle
/// unburnable in exactly the vessels the curated table exists for.
#[test]
fn the_candidate_is_the_most_flammable_thing_present() {
    let mut vessel = Vessel::new(VesselId(0), "beaker");
    vessel.temperature = Kelvin(300.0);
    vessel.deposit(SpeciesId::new("methane"), Moles(0.01), Phase::Gas);
    let gas = kerotakis_core::combustion::ignition_candidate(&vessel).expect("methane is a fuel");
    assert_eq!(gas.fuel.0, "methane");

    // Paraffin lights at 473 K, well below methane's 810 K, so the wax is
    // what the flame is offered.
    vessel.deposit(SpeciesId::new("paraffin"), Moles(0.02), Phase::Solid);
    let wax = kerotakis_core::combustion::ignition_candidate(&vessel).expect("wax is a fuel");
    assert_eq!(wax.fuel.0, "paraffin");
    assert!((wax.autoignition_k - 473.0).abs() < 1e-9);
    assert!((wax.moles.0 - 0.02).abs() < 1e-12);

    // Nothing on either table is no candidate at all, rather than a zero.
    let mut water = Vessel::new(VesselId(1), "beaker");
    water.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    assert!(kerotakis_core::combustion::ignition_candidate(&water).is_none());
}
