//! Same-stock and matched-temperature controls for the unchanged lesson inputs.
//! These exercise model bookkeeping, not empirical calibration of concentrated
//! mixtures or a physical pressure controller's unrepresented work path.
#![cfg(feature = "engine")]

use kerotakis_core::{ledger::ConservedLedger, *};
use kerotakis_phreeqc::PhreeqcEquilibrator;
use std::collections::BTreeSet;

fn stack() -> SolverStack {
    kerotakis_stack::standard_stack(vec![Box::new(PhreeqcEquilibrator::new().unwrap())])
}

fn step(bench: &mut Bench, solver: &mut SolverStack, op: Operator, label: &str) -> Vec<Event> {
    let events = bench.step_with(op, solver, &PermissiveScreen).unwrap();
    println!(
        "{}",
        serde_json::json!({"label": label, "vessel": &bench.vessels[0], "events": &events})
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::SolverFailed { .. })),
        "{label}: native refusal: {events:?}"
    );
    events
}

fn command(bench: &mut Bench, solver: &mut SolverStack, line: &str) -> Vec<Event> {
    let op = script::parse_op(line)
        .unwrap()
        .expect("original lesson command");
    step(bench, solver, op, line)
}

fn closed(expected: &Vessel, actual: &Vessel, events: &[Event]) {
    let mut want = ConservedLedger::from_vessel(expected);
    for event in events {
        let (species, n, sign) = match event {
            Event::GasEvolved {
                vessel,
                species,
                moles,
            } => {
                assert_eq!(*vessel, expected.id);
                (species, moles.0, -1.0)
            }
            Event::GasAbsorbed {
                vessel,
                species,
                moles,
            } => {
                assert_eq!(*vessel, expected.id);
                (species, moles.0, 1.0)
            }
            _ => continue,
        };
        assert!(n.is_finite() && n >= 0.0);
        let gas = species::lookup(species).expect("known crossing formula");
        let formula = stoich::parse_formula(gas.formula).unwrap();
        for (el, count) in formula.counts {
            *want.elements.entry(el).or_default() += sign * n * count;
        }
        want.charge += sign * n * formula.charge;
    }
    let got = ConservedLedger::from_vessel(actual);
    let elements: BTreeSet<_> = want.elements.keys().chain(got.elements.keys()).collect();
    for el in elements {
        let delta = want.elements.get(el).copied().unwrap_or(0.0)
            - got.elements.get(el).copied().unwrap_or(0.0);
        assert!(delta.abs() <= 1e-10, "{el} ledger residual {delta}");
    }
    assert!((want.charge - got.charge).abs() <= 1e-10);
}

fn zero_repeat(bench: &mut Bench, solver: &mut SolverStack, label: &str) {
    let before = bench.vessels[0].clone();
    let events = step(bench, solver, Operator::Wait { seconds: 0.0 }, label);
    let after = &bench.vessels[0];
    closed(&before, after, &events);
    for event in &events {
        if let Event::GasEvolved { moles, .. } | Event::GasAbsorbed { moles, .. } = event {
            assert!(moles.0 <= 1e-10, "equilibrated stock transferred gas again");
        }
    }
    assert!((before.temperature.0 - after.temperature.0).abs() <= 1e-6);
    assert!(
        before
            .energy_between(before.temperature.0, after.temperature.0)
            .abs()
            <= 0.001
    );
    let a = before.solution.as_ref().expect("initial native reading");
    let b = after.solution.as_ref().expect("repeated native reading");
    assert_eq!(a.scope, vessel::SolutionScope::Complete);
    assert_eq!(b.scope, vessel::SolutionScope::Complete);
    assert!((a.ph - b.ph).abs() <= 1e-6);
    assert!((a.ionic_strength - b.ionic_strength).abs() <= 1e-10);
    assert_eq!(a.provenance.dataset, b.provenance.dataset);
    assert_eq!(a.provenance.model, b.provenance.model);
}

fn ideal_gas(vessel: &Vessel) {
    let volume = vessel.headspace_volume().expect("contained gas").0;
    let gas: f64 = vessel
        .contents
        .iter()
        .filter(|p| p.phase == Phase::Gas)
        .map(|p| p.moles.0)
        .sum();
    // Pa L/(mol K), the existing lesson gas-law contract's units and bound.
    let ratio = vessel.pressure.0 * volume / (gas * 8314.462618 * vessel.temperature.0);
    assert!((ratio - 1.0).abs() < 1e-5, "PV/nRT = {ratio}");
}

fn minteq_reference_energy_j(vessel: &Vessel) -> f64 {
    // Printed MINTEQ reference-reaction enthalpies, kJ/mol. These are
    // independently priced in the saved audit; no production heat helper
    // is used here. The two analytical H/O coordinates carry no heat.
    let stock: f64 = vessel
        .contents
        .iter()
        .map(|p| {
            let h = match (p.species.0.as_str(), p.phase) {
                ("H+" | "base_equivalents" | "Na+" | "CO3-2" | "CH3COO-", Phase::Aqueous)
                | ("water", Phase::Liquid)
                | ("N2" | "O2", Phase::Gas) => 0.0,
                ("CH3COOH", Phase::Aqueous) => 0.41,
                ("HCO3-", Phase::Aqueous) => -14.6,
                ("CO2(aq)", Phase::Aqueous) => -23.76,
                ("CO2", Phase::Gas) => -4.06,
                ("NaHCO3", Phase::Solid) => {
                    -14.6
                        - species::lookup(&p.species)
                            .unwrap()
                            .dissolution_enthalpy_kj
                            .unwrap()
                }
                other => panic!("unpriced changed lesson inventory: {other:?}"),
            };
            p.moles.0 * h
        })
        .sum();
    (stock + vessel.free_hydroxide * 55.81) * 1000.0
}

fn integrated_sensible_j(vessel: &Vessel, from: f64, to: f64) -> f64 {
    // Numerical quadrature of tabulated Cp(T), independently of the
    // production analytic polynomial integral and its inverse.
    const PANELS: usize = 256;
    let width = (to - from) / PANELS as f64;
    let mut sum = 0.0;
    for i in 0..=PANELS {
        let temperature = from + i as f64 * width;
        let cp: f64 = vessel
            .contents
            .iter()
            .map(|p| {
                if p.species.0 == "base_equivalents" {
                    return 0.0;
                }
                let data = species::lookup(&p.species).expect("known heat-capacity data");
                let mut cp = states::heat_capacity_at(data, p.phase, temperature);
                if p.phase == Phase::Gas && vessel.is_sealed() {
                    cp -= constants::GAS_CONSTANT;
                }
                p.moles.0 * cp
            })
            .sum();
        let weight = if i == 0 || i == PANELS {
            1.0
        } else if i.is_multiple_of(2) {
            2.0
        } else {
            4.0
        };
        sum += weight * cp;
    }
    sum * width / 3.0
}

fn balloon_heat(mut before: Vessel, after: &Vessel, new_soda: bool) {
    assert!(after
        .solution
        .as_ref()
        .unwrap()
        .provenance
        .dataset
        .starts_with("minteq.v4"));
    // Both sides use the boundary in force during this equilibrium step.
    // This checks the declared model, not an unrepresented controller work path.
    before.headspace = after.headspace;
    let initial_sensible = integrated_sensible_j(&before, 298.15, before.temperature.0);
    if new_soda {
        let soda = species::lookup(&SpeciesId::new("NaHCO3")).unwrap();
        // The incoming 5 g is at the 298.15 K reference: no sensible heat.
        before.deposit(
            SpeciesId::new("NaHCO3"),
            Moles(5.0 / soda.molar_mass),
            Phase::Solid,
        );
    }
    let chemical_heat = minteq_reference_energy_j(&before) - minteq_reference_energy_j(after);
    let sensible = integrated_sensible_j(after, 298.15, after.temperature.0) - initial_sensible;
    let residual = chemical_heat - sensible;
    println!(
        "{}",
        serde_json::json!({"chemical_heat_j": chemical_heat,
        "sensible_change_j": sensible, "residual_j": residual})
    );
    assert!(
        residual.abs() <= 0.001,
        "balloon model heat residual {residual} J"
    );
}

#[test]
fn original_balloon_boundaries_preserve_stock_on_zero_time_repeat() {
    let mut bench = Bench::new();
    let mut solver = stack();
    let mut expected = None;
    let mut subsequent_events = Vec::new();
    for line in include_str!("../../../lessons/balloon-pressure.lab").lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line == "register lv2" {
            continue;
        }
        let before = bench.vessels[0].clone();
        let events = command(&mut bench, &mut solver, line);
        if line == "seal v1 500mL" {
            let mut input = bench.vessels[0].clone();
            let soda = species::lookup(&SpeciesId::new("NaHCO3")).unwrap();
            input.deposit(
                SpeciesId::new("NaHCO3"),
                Moles(5.0 / soda.molar_mass),
                Phase::Solid,
            );
            expected = Some(input);
        } else if let Some(input) = &expected {
            subsequent_events.extend(events);
            if line == "add v1 baking_soda 5g" || line == "regulate v1 1bar 500mL" {
                closed(input, &bench.vessels[0], &subsequent_events);
                ideal_gas(&bench.vessels[0]);
                balloon_heat(before, &bench.vessels[0], line == "add v1 baking_soda 5g");
                // Keep the original command sequence untouched. Ask a fresh
                // solver about an independent copy of each settled boundary.
                let mut repeated = Bench::new();
                repeated.vessels[0] = bench.vessels[0].clone();
                zero_repeat(
                    &mut repeated,
                    &mut stack(),
                    "balloon typed zero-time repeat",
                );
            }
        }
    }
    assert!(
        expected.is_some(),
        "the original sealed input must be reached"
    );
    assert!(
        bench.vessels[0].temperature.0 < 298.15,
        "the reaction cools"
    );
}

#[test]
fn kitchen_bicarbonate_compares_adiabatic_and_matched_bath_without_new_ph_golden() {
    for held in [false, true] {
        let mut bench = Bench::new();
        let mut solver = stack();
        command(&mut bench, &mut solver, "add v1 water 100mL");
        if held {
            bench.vessels[0].thermal_mode = ThermalMode::Thermostatted(Kelvin(298.15));
        }
        let mut expected = bench.vessels[0].clone();
        let soda = species::lookup(&SpeciesId::new("NaHCO3")).unwrap();
        expected.deposit(
            SpeciesId::new("NaHCO3"),
            Moles(5.0 / soda.molar_mass),
            Phase::Solid,
        );
        let events = command(&mut bench, &mut solver, "add v1 NaHCO3 5g");
        closed(&expected, &bench.vessels[0], &events);
        let vessel = &bench.vessels[0];
        let info = vessel.solution.as_ref().expect("computed bicarbonate");
        assert!(info.ph.is_finite() && info.ionic_strength.is_finite());
        if held {
            assert!((vessel.temperature.0 - 298.15).abs() <= 1e-6);
        } else {
            assert!(
                vessel.temperature.0 < 298.15,
                "bicarbonate dissolution cools"
            );
        }
        println!(
            "{}",
            serde_json::json!({"held_at_298_15_k": held, "reading": vessel})
        );
        command(&mut bench, &mut solver, "measure v1 ph");
        zero_repeat(
            &mut bench,
            &mut solver,
            "bicarbonate typed zero-time repeat",
        );
    }
}
