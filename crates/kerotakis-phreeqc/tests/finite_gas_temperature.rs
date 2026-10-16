//! Source-bound native ownership checks, not calibration of Henry constants.
#![cfg(feature = "engine")]
use kerotakis_core::{ledger::ConservedLedger, *};
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn step(bench: &mut Bench, solver: &mut PhreeqcEquilibrator, op: Operator) -> Vec<Event> {
    bench.step_with(op, solver, &PermissiveScreen).unwrap()
}

fn add(bench: &mut Bench, solver: &mut PhreeqcEquilibrator, key: &str, moles: f64) {
    step(
        bench,
        solver,
        Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new(key),
            moles: Moles(moles),
            at: None,
        },
    );
}

fn gas(vessel: &Vessel, key: &str) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|portion| portion.phase == Phase::Gas && portion.species.0 == key)
        .map(|portion| portion.moles.0)
        .sum()
}

fn carbon(vessel: &Vessel) -> f64 {
    ConservedLedger::from_vessel(vessel)
        .elements
        .get("C")
        .copied()
        .unwrap_or(0.0)
}

fn assert_closed(expected: &Vessel, actual: &Vessel) {
    let a = ConservedLedger::from_vessel(expected);
    let b = ConservedLedger::from_vessel(actual);
    assert!(
        (a.charge - b.charge).abs() < 1e-10,
        "charge {} -> {}",
        a.charge,
        b.charge
    );
    assert!(
        (carbon(expected) - carbon(actual)).abs() < 1e-12,
        "owned carbon {} -> {}; expected T={} headspace={:?} contents={:?}; actual elapsed={} T={} headspace={:?} contents={:?}; native solvent_kg={:?}; ledger before={:?} after={:?}",
        carbon(expected), carbon(actual), expected.temperature.0, expected.headspace, expected.contents,
        actual.elapsed_seconds, actual.temperature.0, actual.headspace, actual.contents,
        actual.solution.as_ref().and_then(|solution| solution.solvent_kg), a, b
    );
    let elements: std::collections::BTreeSet<_> =
        a.elements.keys().chain(b.elements.keys()).collect();
    for element in elements {
        let before = a.elements.get(element).copied().unwrap_or(0.0);
        let after = b.elements.get(element).copied().unwrap_or(0.0);
        assert!(
            (before - after).abs() < 1e-9 + 1e-9 * before.abs(),
            "{element}: {before} -> {after}"
        );
    }
}

#[test]
fn sealed_carbonated_water_keeps_owned_carbon_and_charge_through_repeated_solves_and_opening() {
    let mut bench = Bench::new();
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    add(&mut bench, &mut solver, "water", 5.534276991396059);
    step(
        &mut bench,
        &mut solver,
        Operator::Seal {
            vessel: VesselId(0),
            headspace_volume: Liters(0.1),
        },
    );
    let initial = bench.vessel(VesselId(0)).unwrap().clone();
    let mut expected = initial.clone();
    expected.deposit(SpeciesId::new("CO2"), Moles(0.02), Phase::Gas);
    add(&mut bench, &mut solver, "CO2", 0.02);
    for _ in 0..3 {
        let state = bench.vessel(VesselId(0)).unwrap();
        assert_closed(&expected, state);
        assert!((gas(state, "N2") - gas(&initial, "N2")).abs() < 1e-14);
        assert!((gas(state, "O2") - gas(&initial, "O2")).abs() < 1e-14);
        assert!(state.solution.is_some());
        step(&mut bench, &mut solver, Operator::Wait { seconds: 1.0 });
    }
    let events = step(
        &mut bench,
        &mut solver,
        Operator::Open {
            vessel: VesselId(0),
        },
    );
    let state = bench.vessel(VesselId(0)).unwrap();
    assert_eq!(state.headspace, Headspace::Open);
    assert!(
        ConservedLedger::from_vessel(state).charge.abs() < 1e-10,
        "venting neutral CO2 must not leave the old 3.9e-9 eq inherited residual"
    );
    let mut accounted = state.clone();
    for event in events {
        if let Event::GasEvolved { species, moles, .. } = event {
            accounted.deposit(species, moles, Phase::Gas);
        }
    }
    assert!((carbon(&expected) - carbon(&accounted)).abs() < 1e-10);
    for _ in 0..3 {
        step(&mut bench, &mut solver, Operator::Wait { seconds: 1.0 });
        assert!(
            ConservedLedger::from_vessel(bench.vessel(VesselId(0)).unwrap())
                .charge
                .abs()
                < 1e-10
        );
    }
}

#[test]
fn pressure_controlled_carbon_uses_finite_matter_while_volume_changes() {
    let mut bench = Bench::new();
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    bench.vessels[0].thermal_mode = ThermalMode::Thermostatted(Kelvin(298.15));
    add(&mut bench, &mut solver, "water", 5.534276991396059);
    step(
        &mut bench,
        &mut solver,
        Operator::Regulate {
            vessel: VesselId(0),
            pressure: Pascal(200_000.0),
            initial_volume: Liters(0.5),
        },
    );
    let mut expected = bench.vessel(VesselId(0)).unwrap().clone();
    expected.deposit(SpeciesId::new("CO2"), Moles(0.01), Phase::Gas);
    add(&mut bench, &mut solver, "CO2", 0.01);
    for _ in 0..3 {
        let state = bench.vessel(VesselId(0)).unwrap();
        assert_closed(&expected, state);
        assert!((state.pressure.0 - 200_000.0).abs() < 1e-6);
        step(&mut bench, &mut solver, Operator::Wait { seconds: 1.0 });
    }
}
