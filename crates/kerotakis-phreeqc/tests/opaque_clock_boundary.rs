//! Native controls for a represented aqueous clock boundary beside retained
//! recipe matter. The unresolved material has no invented elemental formula;
//! its identity and mass are checked separately from the known-component ledger.
#![cfg(feature = "engine")]

use std::collections::BTreeSet;

use kerotakis_core::clock::CO2_TRANSFER_FLOOR_MOL;
use kerotakis_core::ledger::ConservedLedger;
use kerotakis_core::script::parse_op;
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn stack() -> SolverStack {
    kerotakis_stack::standard_stack(vec![Box::new(PhreeqcEquilibrator::new().expect("engine"))])
}

fn step(bench: &mut Bench, stack: &mut SolverStack, text: &str) -> Vec<Event> {
    let operation = parse_op(text)
        .unwrap_or_else(|error| panic!("{text}: {error}"))
        .unwrap_or_else(|| panic!("{text}: no operation"));
    step_operator(bench, stack, operation, text)
}

fn step_operator(
    bench: &mut Bench,
    stack: &mut SolverStack,
    operation: Operator,
    label: &str,
) -> Vec<Event> {
    let events = bench
        .step_with(operation, stack, &PermissiveScreen)
        .unwrap_or_else(|error| panic!("{label}: {error:?}"));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::SolverFailed { .. })),
        "{label}: {events:?}"
    );
    events
}

fn zero_time_wait(bench: &mut Bench, stack: &mut SolverStack) -> Vec<Event> {
    step_operator(
        bench,
        stack,
        Operator::Wait { seconds: 0.0 },
        "typed wait 0s",
    )
}

fn known_ledger(vessel: &Vessel) -> ConservedLedger {
    assert!(!vessel.unresolved_materials.is_empty());
    assert!(vessel
        .unresolved_materials
        .iter()
        .all(|portion| portion.amount.is_finite() && portion.amount >= 0.0));
    for portion in &vessel.contents {
        assert!(portion.moles.0.is_finite() && portion.moles.0 >= 0.0);
        let data = species::lookup(&portion.species)
            .unwrap_or_else(|| panic!("unknown represented species {}", portion.species.0));
        stoich::parse_formula(data.formula).expect("represented formula");
    }
    ConservedLedger::from_vessel(vessel)
}

fn assert_known_boundary(before: &Vessel, after: &Vessel, events: &[Event]) {
    assert_eq!(before.unresolved_materials, after.unresolved_materials);
    let mut expected = known_ledger(before);
    let actual = known_ledger(after);
    for event in events {
        let (species, amount, sign, owner) = match event {
            Event::GasAbsorbed {
                vessel,
                species,
                moles,
            } => (species, moles.0, 1.0, vessel),
            Event::GasEvolved {
                vessel,
                species,
                moles,
            } => (species, moles.0, -1.0, vessel),
            _ => continue,
        };
        assert_eq!(*owner, before.id);
        assert!(amount.is_finite() && amount >= 0.0);
        let data = species::lookup(species).expect("registered gas crossing");
        let formula = stoich::parse_formula(data.formula).expect("gas formula");
        for (element, count) in formula.counts {
            *expected.elements.entry(element).or_default() += sign * amount * count;
        }
        expected.charge += sign * amount * formula.charge;
    }
    let keys: BTreeSet<_> = expected
        .elements
        .keys()
        .chain(actual.elements.keys())
        .cloned()
        .collect();
    for key in keys {
        let left = expected.elements.get(&key).copied().unwrap_or(0.0);
        let right = actual.elements.get(&key).copied().unwrap_or(0.0);
        assert!(
            left.is_finite() && right.is_finite() && (left - right).abs() <= 1e-10,
            "{key}: {left:.15e} -> {right:.15e}"
        );
    }
    assert!((expected.charge - actual.charge).abs() <= 1e-10);
}

#[test]
fn earned_clock_parcel_enters_known_milk_serum_once_without_changing_opaque_stock() {
    let mut bench = Bench::new();
    let mut solver = stack();
    step(&mut bench, &mut solver, "add v1 milk 100mL");
    let vessel = bench.vessel(VesselId(0)).expect("milk");
    assert!(!vessel.unresolved_materials.is_empty());
    assert_eq!(
        vessel.solution.as_ref().expect("fresh serum").scope,
        vessel::SolutionScope::Complete
    );

    // A declared already-earned parcel, comfortably above the existing
    // spend floor; this test claims no uptake rate from a zero-time wait.
    let earned = 2.0 * CO2_TRANSFER_FLOOR_MOL;
    assert!(earned > CO2_TRANSFER_FLOOR_MOL);
    bench.vessels[0].pending_co2_transfer_mol = earned;
    let before = bench.vessel(VesselId(0)).expect("before clock").clone();
    let events = zero_time_wait(&mut bench, &mut solver);
    let after = bench.vessel(VesselId(0)).expect("after clock");
    assert_known_boundary(&before, after, &events);
    assert_eq!(after.pending_co2_transfer_mol, 0.0);
    assert_eq!(
        after.solution.as_ref().expect("serum").scope,
        vessel::SolutionScope::Complete
    );
    let inlets: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            Event::GasAbsorbed { species, moles, .. } if species.0 == "CO2" => Some(moles.0),
            _ => None,
        })
        .collect();
    assert_eq!(inlets, vec![earned], "clock inlet should be announced once");

    let repeat = zero_time_wait(&mut bench, &mut solver);
    assert!(
        !repeat.iter().any(|event| matches!(
            event,
            Event::GasAbsorbed { species, .. } if species.0 == "CO2"
        )),
        "zero-time repeat cannot spend the same parcel again: {repeat:?}"
    );
}

#[test]
fn actual_milk_fermentation_keeps_lactate_speciation_and_missing_casein_warning() {
    let mut bench = Bench::new();
    let mut solver = stack();
    step(&mut bench, &mut solver, "add v1 milk 100mL");
    step(&mut bench, &mut solver, "add v1 yoghurt_culture 1g");
    let events = step(&mut bench, &mut solver, "wait 8h");
    let vessel = bench.vessel(VesselId(0)).expect("fermented milk");
    let info = vessel.solution.as_ref().expect("represented serum answer");
    assert_eq!(info.scope, vessel::SolutionScope::Complete);
    assert!((5.5..6.1).contains(&info.ph), "soured milk pH {}", info.ph);
    let acid = vessel.moles_of(&SpeciesId::new("lactic_acid")).0;
    let anion = vessel.moles_of(&SpeciesId::new("lactate")).0;
    assert!(acid > 0.0 && anion > 0.0, "lactate split {acid} + {anion}");
    assert!((acid + anion - 8.5336e-4).abs() < 2e-5);
    assert!(
        events.iter().any(|event| matches!(
            event,
            Event::NotYetModeled { reason: Some(reason), .. }
                if reason.key == "not-modeled.milk-casein-buffer-residual"
        )),
        "missing casein coverage must remain explicit: {events:?}"
    );
}
