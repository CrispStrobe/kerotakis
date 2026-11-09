#![cfg(feature = "engine")]
//! Exact frozen temperature controls through the standard native solver stack.
use kerotakis_core::ledger::ConservedLedger;
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn same_material(actual: &Vessel, expected: &Vessel) {
    let a = ConservedLedger::from_vessel(actual);
    let b = ConservedLedger::from_vessel(expected);
    for element in a.elements.keys().chain(b.elements.keys()) {
        let difference = a.elements.get(element).copied().unwrap_or(0.0)
            - b.elements.get(element).copied().unwrap_or(0.0);
        assert!(difference.abs() <= 1e-8, "{element}: {difference}");
    }
    assert!((a.charge - b.charge).abs() <= 1e-8);
}

fn check(script: &str) {
    let mut bench = Bench::new();
    let mut solver = SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
        PhreeqcEquilibrator::new().expect("native engine"),
    )]));
    let mut checked = false;
    for line in script.lines() {
        let Some(op) = kerotakis_core::script::parse_op(line).unwrap() else {
            continue;
        };
        let before = matches!(op, Operator::Transport { .. }).then(|| {
            (
                bench.vessel(VesselId(0)).unwrap().clone(),
                bench.vessel(VesselId(1)).unwrap().clone(),
            )
        });
        let events = bench.step_with(op, &mut solver, &PermissiveScreen).unwrap();
        assert!(
            !events.iter().any(|event| matches!(
                event,
                Event::SolverFailed { .. } | Event::NotYetModeled { .. }
            )),
            "{line}: {events:?}"
        );
        if let Some((cell, inlet)) = before {
            let receiver = bench.vessel(VesselId(2)).unwrap();
            let updated_cell = bench.vessel(VesselId(0)).unwrap();
            assert!(
                (receiver.temperature.0 - cell.temperature.0).abs() <= 0.01,
                "effluent must arrive at the cell temperature"
            );
            assert!((updated_cell.temperature.0 - inlet.temperature.0).abs() <= 0.01);
            same_material(receiver, &cell);
            same_material(updated_cell, &inlet);
            same_material(bench.vessel(VesselId(1)).unwrap(), &inlet);
            checked = true;
        }
    }
    assert!(
        checked,
        "fixture must execute its frozen transport operation"
    );
}

#[test]
fn hot_cell_cold_inlet_native_temperature_and_inventory() {
    check(include_str!("fixtures/transport-hot-cell-cold-inlet.lab"));
}

#[test]
fn cold_cell_hot_inlet_native_temperature_and_inventory() {
    check(include_str!("fixtures/transport-cold-cell-hot-inlet.lab"));
}
