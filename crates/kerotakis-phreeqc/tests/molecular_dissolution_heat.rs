//! Molecular (non-ionising) solutes and their heat of dissolution.
//!
//! Glucose, sucrose and urea go into water as the molecule they left the
//! crystal as. No vendored database speciates them, so the aqueous engine
//! routes such a beaker to the solvent relation — and that route never ran
//! the heat balance. The dissolution left the thermometer where it was and
//! said nothing about it, which is the one combination the heat ledger
//! forbids everywhere else.
//!
//! Now, per class:
//! * a molecular solute WITH a registry enthalpy of dissolution (urea,
//!   +15.09 kJ/mol, NBS/Wagman 1982) is priced: the beaker cools by
//!   `n ΔH / C`;
//! * one WITHOUT (glucose, sucrose) carries the typed
//!   `not-modeled.aqueous-heat-unpriced` note and keeps its temperature;
//! * a molecule that does not move (a second `wait`) costs and says nothing.
#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::enthalpy::{self, molecular_dissolution_route};
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn run(script: &[&str]) -> (Bench, Vec<Vec<Event>>) {
    let mut bench = Bench::new();
    let mut stack = SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
        PhreeqcEquilibrator::new().expect("engine"),
    )]));
    let mut steps = Vec::new();
    for op in script {
        steps.push(
            bench
                .step_with(
                    kerotakis_core::script::parse_op(op)
                        .expect("parse")
                        .expect("a known verb"),
                    &mut stack,
                    &PermissiveScreen,
                )
                .expect("step"),
        );
    }
    (bench, steps)
}

const UNPRICED: &str = "not-modeled.aqueous-heat-unpriced";

fn unpriced_for(steps: &[Event], species: &str) -> bool {
    steps.iter().any(|event| {
        matches!(event, Event::NotYetModeled { reason: Some(reason), .. }
            if reason.key == UNPRICED
                && reason.slots.iter().any(|(_, slot)| format!("{slot:?}").contains(species)))
    })
}

fn any_unpriced(steps: &[Event]) -> bool {
    steps.iter().any(|event| {
        matches!(event, Event::NotYetModeled { reason: Some(reason), .. } if reason.key == UNPRICED)
    })
}

fn celsius(bench: &Bench) -> f64 {
    bench.vessel(VesselId(0)).unwrap().temperature.to_celsius()
}

fn aqueous(bench: &Bench, key: &str) -> f64 {
    bench
        .vessel(VesselId(0))
        .unwrap()
        .contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == Phase::Aqueous)
        .map(|p| p.moles.0)
        .sum()
}

/// Negative class: a sugar with no reviewed enthalpy of dissolution says so
/// on the step it dissolves, keeps the supplied temperature, and is silent
/// once it has dissolved.
#[test]
fn an_unpriced_sugar_declares_its_heat_and_keeps_the_temperature() {
    for (key, grams) in [("glucose", 5.0), ("sucrose", 5.0)] {
        assert_eq!(
            molecular_dissolution_route(key).map(|r| r.is_err()),
            Some(true),
            "{key} is a molecular solute without a reviewed enthalpy"
        );
        let add = format!("add v1 {key} {grams}g");
        let (bench, steps) = run(&["add v1 water 100mL", &add, "wait 10s"]);
        let data = species::lookup_key(key).unwrap();
        let n = grams / data.molar_mass;
        assert!(
            (aqueous(&bench, key) - n).abs() < 1e-9 * n.max(1.0),
            "{key} dissolves whole: {steps:#?}"
        );
        assert!(
            unpriced_for(&steps[1], key),
            "{key} dissolved with neither a heat nor a note: {:#?}",
            steps[1]
        );
        assert!(
            (celsius(&bench) - 25.0).abs() < 1e-9,
            "{key}: temperature kept"
        );
        assert!(
            !any_unpriced(&steps[2]),
            "nothing moved on the wait, so nothing is refused: {:#?}",
            steps[2]
        );
    }
}

/// Positive class: every registry molecular solute that carries an
/// enthalpy of dissolution is priced, with the sign and size the number
/// says. (Urea, NBS/Wagman 1982 +15.09 kJ/mol, is the row this was written
/// for.)
#[test]
fn a_priced_molecular_solute_moves_the_thermometer_by_its_enthalpy() {
    for data in species::REGISTRY {
        let Some(Ok(dh)) = molecular_dissolution_route(data.key) else {
            continue;
        };
        let grams = 1.0;
        let n = grams / data.molar_mass;
        let add = format!("add v1 {} {grams}g", data.key);
        let (bench, steps) = run(&["add v1 water 100mL", &add, "wait 10s"]);
        if aqueous(&bench, data.key) < 0.99 * n {
            // Bounded below the dose: covered by the ledger test below.
            continue;
        }
        assert!(!any_unpriced(&steps[1]), "{}: {:#?}", data.key, steps[1]);
        let c = bench.vessel(VesselId(0)).unwrap().heat_capacity();
        let expected = 25.0 - n * dh * 1000.0 / c;
        let t = celsius(&bench);
        assert!(
            (t - expected).abs() < 0.05 + 0.05 * (expected - 25.0).abs(),
            "{}: {t} C vs about {expected} C",
            data.key
        );
        assert!((t - 25.0).signum() == -dh.signum(), "{}: sign", data.key);
    }
}

/// The ledger term itself: a pure phase move is priced (or refused by
/// name), a molecule a reaction consumed is not a dissolution, and an ion
/// the engine speciates is never a molecular solute.
#[test]
fn the_ledger_term_prices_only_a_pure_phase_move() {
    let p = |key: &str, moles: f64, phase: Phase| Portion {
        species: SpeciesId::new(key),
        moles: Moles(moles),
        phase,
    };
    let solid = [p("glucose", 0.01, Phase::Solid)];
    let dissolved = [p("glucose", 0.01, Phase::Aqueous)];
    let moved = enthalpy::molecular_dissolution(&solid, &dissolved);
    assert!(moved.priced.is_empty());
    assert_eq!(
        moved.unpriced.as_ref().map(|u| u.species.as_str()),
        Some("glucose")
    );
    // Half of it consumed by something else: not a dissolution.
    let consumed = [p("glucose", 0.005, Phase::Aqueous)];
    let reacted = enthalpy::molecular_dissolution(&solid, &consumed);
    assert!(reacted.unpriced.is_none() && reacted.priced.is_empty());
    // A salt the engine speciates is not a molecular solute.
    assert!(!enthalpy::is_molecular_solute("NaCl"));
    assert!(!enthalpy::is_molecular_solute("water"));
    assert!(!enthalpy::is_molecular_solute("Cu"));
    // Nothing moved: nothing to say.
    let still = enthalpy::molecular_dissolution(&dissolved, &dissolved);
    assert!(still.unpriced.is_none() && still.priced.is_empty() && still.released_j == 0.0);
}
