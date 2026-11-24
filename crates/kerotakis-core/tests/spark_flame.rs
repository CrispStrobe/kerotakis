//! C1 audit, closed-vessel combustion: a spark lights what is flammable
//! where it is held, without raising the whole vessel to flame
//! temperature. Positive and negative controls for each branch of
//! `combustion::flame_at_spark`.

use kerotakis_core::combustion::{CombustionEquilibrator, LIMITING_OXYGEN_FRACTION};
use kerotakis_core::ops::{DidNotIgniteReason, Event};
use kerotakis_core::script::parse_op;
use kerotakis_core::solve::SolverStack;
use kerotakis_core::species::{Phase, SpeciesId};
use kerotakis_core::{Bench, HonestyEquilibrator, MixingEquilibrator, PermissiveScreen};

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

fn moles(bench: &Bench, species: &str) -> f64 {
    bench.vessels[0].moles_of(&SpeciesId::new(species)).0
}

fn phase_moles(bench: &Bench, species: &str, phase: Phase) -> f64 {
    bench.vessels[0]
        .contents
        .iter()
        .filter(|p| p.species.0 == species && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}

fn ignited(events: &[Event]) -> bool {
    events.iter().any(|e| matches!(e, Event::Ignited { .. }))
}

fn has_note(events: &[Event], key: &str) -> bool {
    events.iter().any(|e| match e {
        Event::NotYetModeled {
            reason: Some(reason),
            ..
        } => reason.key == key,
        _ => false,
    })
}

/// Oxyhydrogen above water: the headspace gas burns, the liquid does not
/// boil, and the heat is the sourced 241.8 kJ per mole of water.
#[test]
fn oxyhydrogen_over_water_pops_and_leaves_the_water_liquid() {
    let (bench, events) = run(&[
        "add v1 water 50mL",
        "seal v1 100mL",
        "add v1 H2 0.002mol",
        "add v1 O2 0.001mol",
        "ignite v1",
    ]);
    assert!(ignited(&events), "a 2:1 hydrogen/oxygen mixture must light");
    assert!(moles(&bench, "H2") < 1e-9, "all the hydrogen burned");
    assert!(
        phase_moles(&bench, "water", Phase::Liquid) > 2.7,
        "the liquid water was not boiled away by the spark"
    );
    let heat = events
        .iter()
        .find_map(|e| match e {
            Event::Ignited { energy_j, .. } => *energy_j,
            _ => None,
        })
        .expect("energy on the ignition event");
    assert!((heat - 0.002 * 241_818.0).abs() < 1.0, "heat {heat}");
    assert!(
        bench.vessels[0].temperature.0 < 320.0,
        "0.48 kJ into 50 g of water warms it a few kelvin, not to 1200 K: {}",
        bench.vessels[0].temperature.0
    );
    assert!(has_note(&events, "not-modeled.closed-flame-wall-loss"));
}

/// Negative: 3% hydrogen in air is below the 4% lower limit.
#[test]
fn hydrogen_below_its_lower_flammability_limit_does_not_burn() {
    // 500 mL of air is about 0.0204 mol; 0.0006 mol H2 is ~2.9%.
    let (bench, events) = run(&["seal v1 500mL", "add v1 H2 0.0006mol", "ignite v1"]);
    assert!(!ignited(&events));
    assert!((moles(&bench, "H2") - 0.0006).abs() < 1e-12);
    assert!(has_note(&events, "not-modeled.gas-mixture-too-lean"));
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, Event::DidNotIgnite { .. }))
            .count(),
        1
    );
}

/// Positive just inside the limit: ~10% hydrogen in air lights.
#[test]
fn hydrogen_inside_its_limits_in_air_burns() {
    let (bench, events) = run(&["seal v1 500mL", "add v1 H2 0.0023mol", "ignite v1"]);
    assert!(ignited(&events));
    assert!(moles(&bench, "H2") < 1e-9);
}

/// Negative: hydrogen in a closed tube of nitrogen has no oxygen to burn
/// in (sweeping first leaves nitrogen, not air, behind the closure).
#[test]
fn hydrogen_without_oxygen_in_a_closed_tube_is_too_rich_to_burn() {
    let (bench, events) = run(&[
        "sweep v1 1bar",
        "regulate v1 1atm 20mL",
        "add v1 H2 0.0008mol",
        "ignite v1",
    ]);
    assert!(moles(&bench, "O2") < 1e-12);
    assert!(!ignited(&events));
    assert!((moles(&bench, "H2") - 0.0008).abs() < 1e-12);
    assert!(has_note(&events, "not-modeled.gas-mixture-too-rich"));
    assert!(events.iter().any(|e| matches!(
        e,
        Event::DidNotIgnite {
            reason: DidNotIgniteReason::NoOxygen,
            ..
        }
    )));
}

/// Hydrogen in an open vessel meets the room at the mouth and burns.
#[test]
fn hydrogen_in_an_open_vessel_burns_at_the_mouth() {
    let (bench, events) = run(&["add v1 water 20mL", "add v1 H2 0.001mol", "ignite v1"]);
    assert!(ignited(&events));
    assert!(moles(&bench, "H2") < 1e-9);
    assert!(!has_note(&events, "not-modeled.closed-flame-wall-loss"));
}

/// Magnesium burns in a closed jar of oxygen, limited by the oxygen.
#[test]
fn magnesium_burns_in_a_closed_jar_of_oxygen_until_the_oxygen_is_gone() {
    // 0.2 g Mg is 8.2 mmol and needs 4.1 mmol O2; the jar holds less.
    let (bench, events) = run(&[
        "regulate v1 1atm 100mL",
        "add v1 O2 0.001mol",
        "add v1 Mg 0.2g",
        "ignite v1",
    ]);
    assert!(ignited(&events));
    assert!(
        moles(&bench, "O2") < 1e-9,
        "magnesium burns the oxygen it is given"
    );
    assert!(moles(&bench, "MgO") > 0.003);
    assert!(
        moles(&bench, "Mg") > 1e-4,
        "the oxygen, not the metal, ran out"
    );
}

/// Negative: magnesium under a nitrogen-only closed boundary.
#[test]
fn magnesium_without_oxygen_does_not_burn() {
    let (bench, events) = run(&[
        "sweep v1 1bar",
        "regulate v1 1atm 200mL",
        "add v1 Mg 0.2g",
        "ignite v1",
    ]);
    assert!(moles(&bench, "O2") < 1e-12);
    assert!(!ignited(&events));
    assert!(moles(&bench, "MgO") < 1e-12);
}

/// A candle under a jar over water burns until 16% oxygen, and the water
/// stays water.
#[test]
fn a_candle_over_water_under_a_jar_goes_out_at_the_limiting_fraction() {
    let (bench, events) = run(&[
        "add v1 water 200mL",
        "add v1 candle_wax 1g",
        "regulate v1 1atm 400mL",
        "ignite v1",
    ]);
    assert!(ignited(&events));
    let fraction = events
        .iter()
        .find_map(|e| match e {
            Event::FlameStarved {
                oxygen_fraction, ..
            } => Some(*oxygen_fraction),
            _ => None,
        })
        .expect("the jar ends the flame");
    assert!((fraction - LIMITING_OXYGEN_FRACTION).abs() < 0.005);
    assert!(phase_moles(&bench, "water", Phase::Liquid) > 10.0);
}

/// A 1 g candle sealed in a 500 mL jar must not start from 1200 K and
/// burst it: the burn starts at room temperature.
#[test]
fn a_sealed_candle_burns_from_room_temperature() {
    let (bench, events) = run(&["add v1 candle_wax 1g", "seal v1 500mL", "ignite v1"]);
    assert!(ignited(&events));
    assert!(!events.iter().any(|e| matches!(e, Event::Burst { .. })));
    assert!(bench.vessels[0].temperature.0 < 800.0);
    assert!(has_note(&events, "not-modeled.closed-flame-wall-loss"));
}
