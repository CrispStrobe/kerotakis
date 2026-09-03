//! KID-12: the three fires a kitchen has, and the jar that puts one out.
//!
//! Every assertion here is about a claim the bench had no way to make
//! before: a candle, a sheet of paper and a spoonful of sugar all fell
//! outside NASA CEA's dataset, so `ignite` reached the model boundary and
//! said so. The interesting one is not that they burn — it is that a
//! covered flame stops with most of the oxygen still in the jar, which is
//! the opposite of what every child is told.

use kerotakis_core::combustion::{CombustionEquilibrator, LIMITING_OXYGEN_FRACTION};
use kerotakis_core::ops::Event;
use kerotakis_core::script::parse_op;
use kerotakis_core::solve::SolverStack;
use kerotakis_core::species::{Phase, SpeciesId};
use kerotakis_core::{Bench, HonestyEquilibrator, MixingEquilibrator, PermissiveScreen};

/// The stack a burning vessel needs: mixing to keep the bookkeeping
/// honest, combustion to answer, honesty to report anything left over.
fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(CombustionEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn run(commands: &[&str]) -> (Bench, Vec<Event>) {
    let mut bench = Bench::new();
    let mut solver = stack();
    let mut events = Vec::new();
    for command in commands {
        let op = parse_op(command)
            .unwrap_or_else(|error| panic!("parse {command}: {error}"))
            .expect("operator");
        events.extend(
            bench
                .step_with(op, &mut solver, &PermissiveScreen)
                .unwrap_or_else(|error| panic!("{command}: {error}")),
        );
    }
    (bench, events)
}

fn released_j(events: &[Event]) -> f64 {
    events
        .iter()
        .find_map(|event| match event {
            Event::ThermalEquilibrium {
                reaction_energy_j, ..
            } => *reaction_energy_j,
            _ => None,
        })
        .expect("a burn reports the heat it released")
}

fn moles(bench: &Bench, species: &str) -> f64 {
    bench.vessels[0].moles_of(&SpeciesId::new(species)).0
}

/// A candle in the open room burns away completely, and the heat is the
/// heat a candle actually carries: about 46 kJ per gram of wax.
#[test]
fn a_candle_in_open_air_burns_away_and_releases_the_heat_of_its_wax() {
    let (bench, events) = run(&["add v1 candle_wax 5g", "ignite v1"]);
    assert!(events.iter().any(|event| matches!(
        event,
        Event::ReactionOccurred { equation, .. } if equation.contains("C25H52")
    )));
    assert!(
        moles(&bench, "paraffin") < 1e-9,
        "nothing limits an open flame, so the wax goes"
    );
    // 5 g of wax is 4.6 g of paraffin after the recipe's unresolved
    // remainder, and 4.6 g x 46 kJ/g is a little over 200 kJ.
    let heat = released_j(&events);
    assert!(
        (heat - 211_000.0).abs() < 6_000.0,
        "expected about 211 kJ, got {heat}"
    );
    // The products left with the room air they burned in.
    assert!(events.iter().any(|event| matches!(
        event,
        Event::GasEvolved { species, .. } if species.0 == "CO2"
    )));
}

/// The demonstration, and the number that contradicts the story told
/// about it: the flame goes out with roughly three quarters of the jar's
/// oxygen still in the jar.
#[test]
fn a_covered_candle_goes_out_with_most_of_the_oxygen_still_there() {
    let (bench, events) = run(&["add v1 candle_wax 5g", "regulate v1 1atm 1L", "ignite v1"]);
    let oxygen_at_start = 0.21 * 0.0409; // 1 L of room air at 25 °C.
    let (burned, fraction) = events
        .iter()
        .find_map(|event| match event {
            Event::FlameStarved {
                fuel,
                burned,
                oxygen_fraction,
                ..
            } if fuel.0 == "paraffin" => Some((burned.0, *oxygen_fraction)),
            _ => None,
        })
        .expect("the jar, not the candle, ends this");
    assert!(burned > 0.0, "it did catch first");
    assert!(
        (fraction - LIMITING_OXYGEN_FRACTION).abs() < 0.005,
        "the flame quits at the limiting fraction, not at zero: {fraction}"
    );
    let oxygen_left = moles(&bench, "O2");
    assert!(
        oxygen_left / oxygen_at_start > 0.7,
        "most of the oxygen is still in the jar: {oxygen_left} of {oxygen_at_start}"
    );
    assert!(
        moles(&bench, "paraffin") > 0.012,
        "and nearly all the wax is still there too"
    );
    // What replaced the oxygen is in the jar, not gone from the ledger.
    assert!(moles(&bench, "CO2") > 0.0);
}

/// A fire extinguisher takes nothing away. The wax is all still there,
/// the oxygen is all still there, and the flame cannot start — because
/// carbon dioxide has taken up the room the oxygen needed.
#[test]
fn carbon_dioxide_smothers_a_flame_without_removing_anything() {
    let (bench, events) = run(&[
        "add v1 candle_wax 5g",
        "regulate v1 1atm 1L",
        "add v1 CO2 0.02mol",
        "ignite v1",
    ]);
    let (burned, fraction) = events
        .iter()
        .find_map(|event| match event {
            Event::FlameStarved {
                burned,
                oxygen_fraction,
                ..
            } => Some((burned.0, *oxygen_fraction)),
            _ => None,
        })
        .expect("the reason the flame did not take");
    assert_eq!(burned, 0.0, "it never caught at all");
    assert!(
        fraction < LIMITING_OXYGEN_FRACTION,
        "and the reason is the fraction: {fraction}"
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::ReactionOccurred { .. })),
        "nothing reacted"
    );
    assert!(
        (moles(&bench, "O2") - 0.21 * 0.0409).abs() < 0.002,
        "the oxygen was never touched"
    );
    assert!(moles(&bench, "paraffin") > 0.012, "nor was the wax");
    // The bench must not follow a stated reason with a contradiction.
    // "Not everything burns" is false about candle wax.
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::DidNotIgnite { .. })),
        "the smothering already said why; DidNotIgnite would claim the wax cannot burn"
    );
}

/// Paper is cellulose and burns as cellulose — the same arithmetic with
/// a different fuel, and roughly a third of the energy per gram.
#[test]
fn paper_burns_as_cellulose_and_carries_a_third_of_the_energy_of_wax() {
    let (_, paper) = run(&["add v1 paper 5g", "ignite v1"]);
    assert!(paper.iter().any(|event| matches!(
        event,
        Event::Consumed { species, .. } if species.0 == "cellulose"
    )));
    let (_, wax) = run(&["add v1 candle_wax 5g", "ignite v1"]);
    let ratio = released_j(&paper) / released_j(&wax);
    assert!(
        (0.28..0.42).contains(&ratio),
        "paper should carry roughly a third of wax's energy, got {ratio}"
    );
}

/// A nitrogen purge is not a low-oxygen atmosphere, it is a no-oxygen
/// one, and nothing burns in it at all.
#[test]
fn nothing_burns_under_a_nitrogen_sweep() {
    let (bench, events) = run(&["add v1 candle_wax 5g", "sweep v1 1bar", "ignite v1"]);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::ReactionOccurred { .. })),
        "there is no oxygen to burn in"
    );
    assert!(moles(&bench, "paraffin") > 0.012);
}

/// Below its autoignition temperature a fuel is just a solid. Warming
/// wax by a hundred degrees must not burn it, or the model would claim a
/// fire that needs no ignition.
#[test]
fn a_warm_fuel_below_its_autoignition_temperature_does_not_burn() {
    let (bench, events) = run(&["add v1 candle_wax 5g", "heat v1 1kJ"]);
    assert!(!events
        .iter()
        .any(|event| matches!(event, Event::ReactionOccurred { .. })));
    assert!(moles(&bench, "paraffin") > 0.012);
    assert_eq!(bench.vessels[0].contents[0].phase, Phase::Solid);
}
