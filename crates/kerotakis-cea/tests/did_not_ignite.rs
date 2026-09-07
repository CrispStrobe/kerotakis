//! What "nothing happened" is made of.
//!
//! `Event::DidNotIgnite` carried a vessel id and nothing else, which made
//! it the one row of the animation audit that could not be closed from
//! the client: with no quantity on the wire there is nothing for a visual
//! to be a function of, and anything drawn would have been a picture of
//! the word. Its sibling `FlameStarved` carries the fuel, what burned and
//! the oxygen fraction, and is drawn from those.
//!
//! This file is the INTEGRATION half: the one case where the reason is
//! not recomputed at all but taken from a solver that already said it.
//! `cea-thermal` answers a warm, unsparked fuel with
//! `Event::BelowAutoignition`, naming the fuel and the temperature it
//! would need, and `bench.rs` believes that event over anything it could
//! work out for itself — two sentences about one vessel must not
//! disagree about which substance they are describing.
//!
//! The four reasons themselves are pinned in
//! `kerotakis-core/tests/did_not_ignite.rs`, where the solver stack is a
//! fixture rather than a chemistry engine.

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
