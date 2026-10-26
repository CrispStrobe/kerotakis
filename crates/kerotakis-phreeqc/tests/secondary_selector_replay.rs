//! Native controls for an auxiliary Pitzer selector crossing its unchanged
//! input floor during a WATEQ carbonate solve.
#![cfg(feature = "engine")]

use kerotakis_core::{ledger::ConservedLedger, *};
use kerotakis_phreeqc::PhreeqcEquilibrator;
use kerotakis_safety::ReactiveGroupScreen;
use std::collections::BTreeSet;

fn step(bench: &mut Bench, solver: &mut PhreeqcEquilibrator, op: Operator) -> Vec<Event> {
    let events = bench
        .step_with(op, solver, &ReactiveGroupScreen)
        .expect("native carbonate step");
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::SolverFailed { .. })),
        "native refusal: {events:?}"
    );
    events
}

fn add(bench: &mut Bench, solver: &mut PhreeqcEquilibrator, key: &str, n: f64) -> Vec<Event> {
    step(
        bench,
        solver,
        Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new(key),
            moles: Moles(n),
            at: None,
        },
    )
}

fn gas_out(events: &[Event]) -> f64 {
    events
        .iter()
        .filter_map(|event| match event {
            Event::GasEvolved { species, moles, .. } if species.0 == "CO2" => Some(moles.0),
            _ => None,
        })
        .sum()
}

fn assert_known_ledger_closed(expected: &Vessel, actual: &Vessel, events: &[Event]) {
    let mut after_input = ConservedLedger::from_vessel(expected);
    // Signed crossings are the only external matter in this zero-time or
    // acid-add step. Contained gas stays in the actual vessel ledger.
    for event in events {
        let (vessel, species, n, sign) = match event {
            Event::GasEvolved {
                vessel,
                species,
                moles,
            } => (vessel, species, moles.0, -1.0),
            Event::GasAbsorbed {
                vessel,
                species,
                moles,
            } => (vessel, species, moles.0, 1.0),
            _ => continue,
        };
        assert_eq!(
            *vessel, expected.id,
            "gas crossing belongs to a different vessel"
        );
        assert!(n.is_finite() && n >= 0.0, "invalid signed gas amount");
        let gas = species::lookup(species).expect("crossing gas has a registered formula");
        let formula = stoich::parse_formula(gas.formula).expect("crossing gas formula");
        for (element, count) in formula.counts {
            *after_input.elements.entry(element).or_default() += sign * count * n;
        }
        after_input.charge += sign * formula.charge * n;
    }
    let observed = ConservedLedger::from_vessel(actual);
    let elements: BTreeSet<_> = after_input
        .elements
        .keys()
        .chain(observed.elements.keys())
        .collect();
    for element in elements {
        let expected_n = after_input.elements.get(element).copied().unwrap_or(0.0);
        let actual_n = observed.elements.get(element).copied().unwrap_or(0.0);
        assert!(
            (expected_n - actual_n).abs() <= 1e-10,
            "{element} ledger: {expected_n} vs {actual_n}"
        );
    }
    assert!(
        (after_input.charge - observed.charge).abs() <= 1e-10,
        "formal charge: {} vs {}",
        after_input.charge,
        observed.charge
    );
}

fn carbonate_arm(solver: &mut PhreeqcEquilibrator, bicarbonate: f64) -> (Bench, f64) {
    let mut bench = Bench::new();
    add(&mut bench, solver, "water", 55.51);
    add(&mut bench, solver, "NaHCO3", bicarbonate);
    let before = bench.vessels[0].clone();
    let events = add(&mut bench, solver, "HCl", 0.03);
    let co2 = gas_out(&events);
    assert!(co2 > 0.0, "acidified bicarbonate must vent actual CO2");
    assert!(
        co2 <= bicarbonate + 1e-10,
        "cannot vent more carbon than supplied"
    );
    let mut expected = before;
    expected.deposit(SpeciesId::new("HCl"), Moles(0.03), Phase::Liquid);
    assert_known_ledger_closed(&expected, &bench.vessels[0], &events);
    let solution = bench.vessels[0]
        .solution
        .as_ref()
        .expect("complete carbonate answer");
    assert_eq!(solution.scope, vessel::SolutionScope::Complete);
    assert!(solution.ph.is_finite() && solution.ionic_strength.is_finite());
    (bench, co2)
}

#[test]
fn strong_acid_carbonate_and_near_floor_stocks_repeat_from_generated_cache() {
    // The first arm is the unchanged failing source case. The neighbors
    // challenge the floor without asserting a result from an unrun probe.
    for bicarbonate in [0.0199, 0.02, 0.0201] {
        let mut native = PhreeqcEquilibrator::new().expect("native engine");
        let (mut fresh, first_co2) = carbonate_arm(&mut native, bicarbonate);
        if bicarbonate == 0.02 {
            assert!(first_co2 > 0.015, "preserve the original fizz assertion");
        }
        let before_repeat = fresh.vessels[0].clone();
        let zero = step(&mut fresh, &mut native, Operator::Wait { seconds: 0.0 });
        assert!(
            gas_out(&zero).abs() <= 1e-10,
            "equilibrated zero-time stock cannot vent twice"
        );
        let after_repeat = &fresh.vessels[0];
        assert_known_ledger_closed(&before_repeat, after_repeat, &zero);
        let before_info = before_repeat.solution.as_ref().unwrap();
        let after_info = after_repeat.solution.as_ref().unwrap();
        assert!((before_info.ph - after_info.ph).abs() <= 1e-6);
        assert!((before_info.ionic_strength - after_info.ionic_strength).abs() <= 1e-10);
        assert!((before_repeat.temperature.0 - after_repeat.temperature.0).abs() <= 1e-6);
        for (a, b) in [
            (before_info.solvent_kg, after_info.solvent_kg),
            (before_info.particle_molality, after_info.particle_molality),
        ] {
            assert!(
                a.zip(b).is_some_and(|(a, b)| (a - b).abs() <= 1e-10),
                "zero-time aqueous numerical coordinate drifted: {a:?} vs {b:?}"
            );
        }

        let warmed = native.export_cache();
        let mut imported = PhreeqcEquilibrator::new().expect("cache replay engine");
        imported.import_cache(warmed);
        let calls_before = imported.engine_calls();
        let (replayed, cached_co2) = carbonate_arm(&mut imported, bicarbonate);
        assert_eq!(
            imported.engine_calls(),
            calls_before,
            "same-stock generated cache must answer every native question"
        );
        assert!((cached_co2 - first_co2).abs() <= 1e-10);
        let a = before_repeat.solution.as_ref().unwrap();
        let b = replayed.vessels[0].solution.as_ref().unwrap();
        assert!((a.ph - b.ph).abs() <= 1e-10);
        assert!((a.ionic_strength - b.ionic_strength).abs() <= 1e-10);
        assert_eq!(
            a.solvent_activity.as_ref().map(|x| (&x.dataset, &x.model)),
            b.solvent_activity.as_ref().map(|x| (&x.dataset, &x.model))
        );
    }
}
