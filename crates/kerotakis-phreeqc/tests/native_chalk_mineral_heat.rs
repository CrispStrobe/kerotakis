//! Native controls for the original aq-090 and mat-115 chalk decants.
//! Reference enthalpy is pinned separately in enthalpy's source-data tests;
//! these checks demand a complete material and adiabatic step ledger.
#![cfg(feature = "engine")]

use std::collections::BTreeSet;

use kerotakis_core::ledger::ConservedLedger;
use kerotakis_core::script::parse_op;
use kerotakis_core::*;
use kerotakis_phreeqc::enthalpy::heat_released_j;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn stack() -> SolverStack {
    kerotakis_stack::standard_stack(vec![Box::new(PhreeqcEquilibrator::new().expect("engine"))])
}

fn known_ledger(bench: &Bench) -> ConservedLedger {
    let mut combined = ConservedLedger {
        elements: Default::default(),
        mass: 0.0,
        charge: 0.0,
        energy: 0.0,
    };
    for vessel in &bench.vessels {
        assert!(
            vessel.unresolved_materials.is_empty(),
            "unresolved chalk stock"
        );
        assert!(vessel.surfaces.is_empty() && vessel.exchanges.is_empty());
        assert!(vessel.electrodes.is_empty() && vessel.material_objects.is_empty());
        assert!(vessel.adsorbed.is_empty() && vessel.solid_solutions.is_empty());
        for portion in &vessel.contents {
            assert!(portion.moles.0.is_finite() && portion.moles.0 >= 0.0);
            let data =
                species::lookup(&portion.species).expect("all phase stock has a registry formula");
            stoich::parse_formula(data.formula).expect("all phase formulas parse");
        }
        let one = ConservedLedger::from_vessel(vessel);
        for (element, amount) in one.elements {
            *combined.elements.entry(element).or_default() += amount;
        }
        combined.charge += one.charge;
    }
    combined
}

fn assert_step_ledger(before: &Bench, after: &Bench, op: &Operator, events: &[Event]) {
    let mut expected = known_ledger(before);
    if let Operator::Add { species, moles, .. } = op {
        let data = species::lookup(species).expect("input formula");
        let formula = stoich::parse_formula(data.formula).expect("input formula parses");
        for (element, count) in formula.counts {
            *expected.elements.entry(element).or_default() += count * moles.0;
        }
        expected.charge += formula.charge * moles.0;
    }
    for event in events {
        let (owner, species, moles, sign) = match event {
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
        assert!(before.vessel(*owner).is_ok() || after.vessel(*owner).is_ok());
        assert!(moles.is_finite() && moles >= 0.0);
        let data = species::lookup(species).expect("gas formula");
        let formula = stoich::parse_formula(data.formula).expect("gas formula parses");
        for (element, count) in formula.counts {
            *expected.elements.entry(element).or_default() += sign * moles * count;
        }
        expected.charge += sign * moles * formula.charge;
    }
    let actual = known_ledger(after);
    let elements: BTreeSet<_> = expected
        .elements
        .keys()
        .chain(actual.elements.keys())
        .cloned()
        .collect();
    for element in elements {
        let want = expected.elements.get(&element).copied().unwrap_or(0.0);
        let got = actual.elements.get(&element).copied().unwrap_or(0.0);
        assert!(
            want.is_finite() && got.is_finite() && (got - want).abs() <= 1e-10,
            "{op:?}: {element} changed {want:.15e} -> {got:.15e}"
        );
    }
    assert!(
        (actual.charge - expected.charge).abs() <= 1e-10,
        "{op:?}: charge changed {:.15e} -> {:.15e}",
        expected.charge,
        actual.charge
    );
}

fn step(bench: &mut Bench, solver: &mut SolverStack, op: Operator) -> Vec<Event> {
    let before = bench.clone();
    let events = bench
        .step_with(op.clone(), solver, &PermissiveScreen)
        .expect("native step");
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::SolverFailed { .. })),
        "{op:?}: {events:?}"
    );
    assert_step_ledger(&before, bench, &op, &events);
    for vessel in &bench.vessels {
        assert!(vessel.temperature.0.is_finite() && vessel.temperature.0 >= 252.0);
        if vessel
            .contents
            .iter()
            .any(|portion| portion.species.0 == "CaCO3")
        {
            let solution = vessel.solution.as_ref().expect("current chalk readback");
            assert_eq!(solution.scope, SolutionScope::Complete);
            assert!(solution.ph.is_finite() && solution.ionic_strength.is_finite());
        }
    }
    events
}

fn sensible(bench: &Bench) -> f64 {
    bench
        .vessels
        .iter()
        .map(|vessel| {
            kerotakis_core::solve::portions_enthalpy(
                vessel
                    .contents
                    .iter()
                    .map(|portion| (&portion.species, portion.moles.0, portion.phase)),
                298.15,
                vessel.temperature.0,
            )
        })
        .sum()
}

fn assert_adiabatic_decant(before: &Bench, after: &Bench, events: &[Event]) {
    let before_contents: Vec<_> = before
        .vessels
        .iter()
        .flat_map(|v| v.contents.iter().cloned())
        .collect();
    let after_contents: Vec<_> = after
        .vessels
        .iter()
        .flat_map(|v| v.contents.iter().cloned())
        .collect();
    let oh_before: f64 = before.vessels.iter().map(|v| v.free_hydroxide).sum();
    let oh_after: f64 = after.vessels.iter().map(|v| v.free_hydroxide).sum();
    let mut gas = Vec::new();
    let mut gas_sensible = 0.0;
    for event in events {
        let (owner, species, signed) = match event {
            Event::GasEvolved {
                vessel,
                species,
                moles,
            } => (vessel, species, moles.0),
            Event::GasAbsorbed {
                vessel,
                species,
                moles,
            } => (vessel, species, -moles.0),
            _ => continue,
        };
        gas.push((species.0.clone(), signed));
        let data = species::lookup(species).expect("known gas");
        let temperature = after.vessel(*owner).expect("gas owner").temperature.0;
        gas_sensible += signed * states::enthalpy_between(data, Phase::Gas, 298.15, temperature);
    }
    let quote = heat_released_j(
        &before_contents,
        oh_before,
        &after_contents,
        oh_after,
        &gas,
        "wateq4f",
    )
    .expect("the entire native chalk decant has a priced reference");
    let delta_sensible = sensible(after) - sensible(before) + gas_sensible;
    assert!(
        (delta_sensible - quote).abs() <= 0.001,
        "adiabatic chalk decant: sensible {delta_sensible:.9} J vs chemical {quote:.9} J"
    );
}

fn assert_receiver_fraction(before: &Bench, after: &Bench, events: &[Event], fraction: f64) {
    let donor = before.vessel(VesselId(0)).expect("source before decant");
    let receiver_before = before.vessel(VesselId(1)).expect("empty receiver");
    let receiver_after = after.vessel(VesselId(1)).expect("receiver after decant");
    let mut expected = ConservedLedger::from_vessel(receiver_before);
    for portion in &donor.contents {
        if !matches!(portion.phase, Phase::Liquid | Phase::Aqueous) {
            continue;
        }
        let data = species::lookup(&portion.species).expect("known transferred species");
        let formula = stoich::parse_formula(data.formula).expect("transferred formula");
        for (element, count) in formula.counts {
            *expected.elements.entry(element).or_default() += fraction * portion.moles.0 * count;
        }
        expected.charge += fraction * portion.moles.0 * formula.charge;
    }
    for event in events {
        let (species, signed) = match event {
            Event::GasEvolved {
                vessel: VesselId(1),
                species,
                moles,
            } => (species, -moles.0),
            Event::GasAbsorbed {
                vessel: VesselId(1),
                species,
                moles,
            } => (species, moles.0),
            _ => continue,
        };
        let formula =
            stoich::parse_formula(species::lookup(species).expect("receiver gas").formula)
                .expect("receiver gas formula");
        for (element, count) in formula.counts {
            *expected.elements.entry(element).or_default() += signed * count;
        }
        expected.charge += signed * formula.charge;
    }
    let actual = ConservedLedger::from_vessel(receiver_after);
    let elements: BTreeSet<_> = expected
        .elements
        .keys()
        .chain(actual.elements.keys())
        .cloned()
        .collect();
    for element in elements {
        let want = expected.elements.get(&element).copied().unwrap_or(0.0);
        let got = actual.elements.get(&element).copied().unwrap_or(0.0);
        assert!(
            (got - want).abs() <= 1e-10,
            "receiver {element} from the {fraction} decant changed {want:.15e} -> {got:.15e}"
        );
    }
    assert!(
        (actual.charge - expected.charge).abs() <= 1e-10,
        "receiver charge from the {fraction} decant changed {:.15e} -> {:.15e}",
        expected.charge,
        actual.charge
    );
}

fn original_chalk_decant(script: &[&str], fraction: f64) {
    let mut bench = Bench::new();
    let mut solver = stack();
    for command in script {
        let op = parse_op(command)
            .expect("original syntax")
            .expect("original operator");
        let decant = matches!(op, Operator::Decant { .. });
        if *command == "add v1 water 100mL" {
            let expected = species::lookup(&SpeciesId::new("water"))
                .expect("registry water")
                .moles_from_liters(Liters(0.1))
                .0;
            assert!(
                matches!(&op, Operator::Add { moles, .. } if (moles.0 - expected).abs() <= 1e-12)
            );
        }
        if *command == "add v1 CaCO3 5g" {
            let expected = species::lookup(&SpeciesId::new("CaCO3"))
                .expect("registry chalk")
                .moles_from_grams(Grams(5.0))
                .0;
            assert!(
                matches!(&op, Operator::Add { moles, .. } if (moles.0 - expected).abs() <= 1e-12)
            );
        }
        let before_decant = decant.then(|| bench.clone());
        let events = step(&mut bench, &mut solver, op);
        if decant {
            assert!(
                events.iter().any(|event| matches!(event,
                Event::Transferred { from: VesselId(0), to: VesselId(1), fraction: actual }
                    if (*actual - fraction).abs() < f64::EPSILON)),
                "original decant fraction was not executed: {events:?}"
            );
            assert_adiabatic_decant(before_decant.as_ref().expect("pre-decant"), &bench, &events);
            assert_receiver_fraction(
                before_decant.as_ref().expect("pre-decant"),
                &bench,
                &events,
                fraction,
            );
        }
    }
    let previous = bench.clone();
    step(&mut bench, &mut solver, Operator::Wait { seconds: 0.0 });
    for (before, after) in previous.vessels.iter().zip(&bench.vessels) {
        assert!((after.temperature.0 - before.temperature.0).abs() <= 1e-6);
        let old = before.solution.as_ref().expect("old native readback");
        let new = after.solution.as_ref().expect("new native readback");
        assert!((new.ph - old.ph).abs() <= 1e-6);
        assert!((new.ionic_strength - old.ionic_strength).abs() <= 1e-10);
        let keys: BTreeSet<_> = before
            .contents
            .iter()
            .chain(&after.contents)
            .map(|portion| (portion.species.0.clone(), portion.phase))
            .collect();
        for (key, phase) in keys {
            let amount = |v: &Vessel| {
                v.contents
                    .iter()
                    .filter(|p| p.species.0 == key && p.phase == phase)
                    .map(|p| p.moles.0)
                    .sum::<f64>()
            };
            assert!(
                (amount(after) - amount(before)).abs() <= 1e-10,
                "zero-time repeat changed {key} {phase:?}"
            );
        }
    }
}

#[test]
fn aq090_half_chalk_decant_closes_native_mineral_heat_and_all_stock() {
    original_chalk_decant(
        &[
            "new",
            "add v1 water 100mL",
            "add v1 CaCO3 5g",
            "decant v1 v2 0.5",
        ],
        0.5,
    );
}

#[test]
fn mat115_four_fifths_chalk_decant_closes_native_mineral_heat_and_all_stock() {
    original_chalk_decant(
        &[
            "add v1 water 100mL",
            "add v1 CaCO3 5g",
            "new",
            "decant v1 v2 0.8",
        ],
        0.8,
    );
}
