//! Why a thermoplastic softens and a thermoset does not (curiosity corpus
//! mat-025).
//!
//! The bench used to answer `add v1 thermoset_resin 2g; heat v1 5kJ` with
//! *"heating an empty vessel (container heat capacity not modelled)"* —
//! over a beaker with a two-gram block in it. Two things were missing and
//! they were different things: the block had no heat capacity, so nothing
//! could be warmed; and there was no reviewed answer to what heat does to
//! a plastic, so even a warm one had nothing to say.
//!
//! These tests pin both, and the contrast that is the actual question: the
//! same script over a thermoplastic and a thermoset must not give the same
//! answer.

use kerotakis_core::plastics;
use kerotakis_core::script::parse_op;
use kerotakis_core::solve::SolverStack;
use kerotakis_core::vessel::{Vessel, VesselId};
use kerotakis_core::{
    Bench, Event, HonestyEquilibrator, MixingEquilibrator, PermissiveScreen,
    PhaseRouteEquilibrator, PolymerState,
};

fn run(commands: &[&str]) -> (Bench, Vec<Event>) {
    let mut bench = Bench::new();
    let mut solver = SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(PhaseRouteEquilibrator),
        Box::new(HonestyEquilibrator),
    ]);
    let mut last = Vec::new();
    for command in commands {
        let op = parse_op(command)
            .unwrap_or_else(|error| panic!("parse {command}: {error}"))
            .expect("operator");
        last = bench
            .step_with(op, &mut solver, &PermissiveScreen)
            .unwrap_or_else(|error| panic!("{command}: {error}"));
    }
    (bench, last)
}

fn vessel(bench: &Bench) -> &Vessel {
    bench.vessel(VesselId(0)).expect("v1")
}

fn state(events: &[Event]) -> PolymerState {
    events
        .iter()
        .find_map(|event| match event {
            Event::PolymerHeated { state, .. } => Some(*state),
            _ => None,
        })
        .expect("a polymer verdict")
}

/// The block weighs something to the heater now. Before this it did not,
/// and `heat` answered a beaker holding two grams of resin by calling it
/// empty — which is the kind of confident wrong answer that reads as a
/// result rather than a gap.
#[test]
fn a_wholly_unresolved_block_can_be_heated_at_all() {
    let (bench, events) = run(&["add v1 thermoset_resin 2g"]);
    let cp = vessel(&bench).heat_capacity();
    assert!(
        (cp - 3.0).abs() < 1e-9,
        "two grams at 1.5 J/(g·K) is 3 J/K, got {cp}"
    );
    assert!(!events.iter().any(|event| matches!(
        event,
        Event::NotYetModeled { what, .. } if what.contains("empty vessel")
    )));
    // A vessel with no declared polymer in it is unchanged: this term is
    // narrow on purpose, so no existing material's energy accounting moves.
    let (plain, _) = run(&["add v1 chalk_stick 2g"]);
    let resolved: f64 = vessel(&plain)
        .contents
        .iter()
        .filter_map(|portion| {
            kerotakis_core::species::lookup(&portion.species)
                .map(|data| portion.moles.0 * data.heat_capacity)
        })
        .sum();
    assert!((vessel(&plain).heat_capacity() - resolved).abs() < 1e-12);
}

/// mat-025 itself. Five kilojoules into two grams is a great deal of heat,
/// and the network does the only thing a network can do.
#[test]
fn a_thermoset_never_softens_and_chars_instead() {
    let (bench, events) = run(&["add v1 thermoset_resin 2g", "heat v1 5kJ"]);
    assert_eq!(state(&events), PolymerState::Charred);
    let verdict = events
        .iter()
        .find_map(|event| match event {
            Event::PolymerHeated {
                state,
                cross_linked,
                reversible,
                threshold,
                ..
            } => Some((*state, *cross_linked, *reversible, threshold.0)),
            _ => None,
        })
        .expect("a verdict");
    assert!(verdict.1, "a cured thermoset is cross-linked");
    assert!(!verdict.2, "charring does not undo");
    assert!((verdict.3 - 573.15).abs() < 1e-9);
    assert!(vessel(&bench).temperature.0 > 573.15);
}

/// And cooling it back down does not bring it back. Every other observable
/// on this bench is recomputed from the state the vessel is in now, which
/// is right for softening and wrong for this.
#[test]
fn charring_does_not_undo_on_cooling() {
    let (bench, events) = run(&[
        "add v1 thermoset_resin 2g",
        "heat v1 5kJ",
        "cool v1 4kJ",
        "cool v1 1kJ",
    ]);
    assert_eq!(state(&events), PolymerState::Charred);
    assert!(
        vessel(&bench).temperature.0 < 573.15,
        "the beaker really did come back down: {}",
        vessel(&bench).temperature.0
    );
    assert_eq!(
        vessel(&bench).charred_materials,
        vec!["polymer/thermoset-resin".to_string()]
    );
}

/// The other half of the question, and the reason the thermoset row means
/// anything: the SAME script over a thermoplastic gives a different answer,
/// and a gentler one gives the answer the question is really about.
#[test]
fn a_thermoplastic_softens_and_sets_again() {
    // Cold: rigid, and the sentence names the temperature it would need.
    let (_, cold) = run(&["add v1 thermoplastic 2g"]);
    assert_eq!(state(&cold), PolymerState::Rigid);

    // Warm past the melt but nowhere near decomposition: softened, and
    // reversible, which is what recycling a bottle by melting it is.
    let (bench, warm) = run(&["add v1 thermoplastic 2g", "heat v1 600J"]);
    assert!(
        (403.15..673.15).contains(&vessel(&bench).temperature.0),
        "between the melt and decomposition: {}",
        vessel(&bench).temperature.0
    );
    assert_eq!(state(&warm), PolymerState::Softened);
    assert!(warm.iter().any(|event| matches!(
        event,
        Event::PolymerHeated {
            reversible: true,
            cross_linked: false,
            ..
        }
    )));

    // Cooled again, it is hard again. Nothing was broken and nothing made.
    let (_, cooled) = run(&["add v1 thermoplastic 2g", "heat v1 600J", "cool v1 600J"]);
    assert_eq!(state(&cooled), PolymerState::Rigid);

    // The same 5 kJ that charred the thermoset chars this too — a
    // thermoplastic is not immune to decomposition, it simply has a melt
    // on the way there and the network has none.
    let (_, burnt) = run(&["add v1 thermoplastic 2g", "heat v1 5kJ"]);
    assert_eq!(state(&burnt), PolymerState::Charred);
}

/// The contrast stated as one assertion, because it is the whole row: at a
/// temperature where the thermoplastic has gone soft, the thermoset has
/// not, and the reason given is that it has no softening point rather than
/// that it has a higher one.
#[test]
fn at_one_temperature_the_two_families_answer_differently() {
    // 150 °C: past polyethylene's 130 °C melt, far under the resin's
    // 300 °C decomposition, and above a cured epoxy's glass transition —
    // which the bench deliberately does not treat as softening.
    let heat_to = |material: &str, joules: f64| {
        let command = format!("add v1 {material} 2g");
        let heat = format!("heat v1 {joules}J");
        run(&[command.as_str(), heat.as_str()]).1
    };
    // 2 g × 1.82 J/(g·K) × (423 − 298) K ≈ 455 J.
    let plastic = heat_to("thermoplastic", 455.0);
    // 2 g × 1.5 J/(g·K) × (423 − 298) K ≈ 375 J.
    let resin = heat_to("thermoset_resin", 375.0);
    assert_eq!(state(&plastic), PolymerState::Softened);
    assert_eq!(state(&resin), PolymerState::Rigid);
    let network = resin.iter().any(|event| {
        matches!(
            event,
            Event::PolymerHeated {
                cross_linked: true,
                ..
            }
        )
    });
    assert!(network, "and the reason is the cross-linking, not a number");
}

/// A named object with no reviewed heat response says nothing at all. The
/// route speaks for the rows it has and stays quiet everywhere else.
#[test]
fn a_material_without_a_reviewed_row_gets_no_verdict() {
    let (bench, events) = run(&["add v1 expanded_PS 2g"]);
    assert!(plastics::observe(vessel(&bench)).is_empty());
    assert!(!events
        .iter()
        .any(|event| matches!(event, Event::PolymerHeated { .. })));
}

/// Both rows carry their citation and their caveat, and the tranche says
/// its provenance lane is not cleared.
#[test]
fn every_reviewed_row_carries_its_source_and_its_boundary() {
    use kerotakis_core::material::{self, MaterialRole};
    let mut rows = 0;
    for recipe in material::all() {
        for role in &recipe.roles {
            if let MaterialRole::PolymerHeatResponse {
                specific_heat_j_per_g_k,
                softens_above_k,
                chars_above_k,
                boundary,
                source,
            } = role
            {
                rows += 1;
                assert!(*specific_heat_j_per_g_k > 0.0, "{}", recipe.id);
                assert!(*chars_above_k > 0.0, "{}", recipe.id);
                if let Some(softens) = softens_above_k {
                    assert!(softens < chars_above_k, "{}", recipe.id);
                }
                assert!(source.contains("polymer heat-response tranche v1"));
                assert!(
                    source.contains("PENDING REVIEW"),
                    "the lane is not cleared and the row must say so"
                );
                assert!(!boundary.is_empty(), "{}", recipe.id);
            }
        }
    }
    assert_eq!(rows, 2, "one thermoplastic and one thermoset");
}
