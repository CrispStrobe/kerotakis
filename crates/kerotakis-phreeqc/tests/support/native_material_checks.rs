//! Strict native-route evidence shared by targeted integration regressions.

use kerotakis_core::ledger::ConservedLedger;
use kerotakis_core::*;
use kerotakis_safety::ReactiveGroupScreen;
use std::collections::BTreeSet;

pub fn add_with_native_material_closure(
    bench: &mut Bench,
    stack: &mut SolverStack,
    vessel: VesselId,
    species: &str,
    moles: f64,
    context: &str,
) -> Vec<Event> {
    let before = bench
        .vessel(vessel)
        .unwrap_or_else(|error| panic!("{context}: missing vessel before input: {error:?}"))
        .clone();
    let species_id = SpeciesId::new(species);
    let amount = Moles(moles);
    let events = bench
        .step_with(
            Operator::Add {
                vessel,
                species: species_id.clone(),
                moles: amount,
                at: None,
            },
            stack,
            &ReactiveGroupScreen,
        )
        .unwrap_or_else(|error| panic!("{context}: add {species} failed: {error:?}"));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::SolverFailed { .. })),
        "{context}: solver failure while adding {species}: {events:?}"
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::SolutionCharacterized { .. })),
        "{context}: input did not produce fresh characterization: {events:?}"
    );
    let after = bench
        .vessel(vessel)
        .unwrap_or_else(|error| panic!("{context}: missing vessel after input: {error:?}"));
    assert_complete_native_current(after, context);
    let solution = after.solution.as_ref().expect("checked current solution");
    // Routing announcements are intentionally deduplicated when the native
    // dataset stays the same. Bind this dose's numerical characterization
    // to its current native answer rather than require another UI message.
    assert!(
        events.iter().any(|event| matches!(event,
            Event::SolutionCharacterized { vessel: id, ph, ionic_strength }
                if *id == vessel && *ph == solution.ph && *ionic_strength == solution.ionic_strength
        )),
        "{context}: fresh characterization does not match current answer"
    );
    assert_gas_adjusted_inventory(&before, after, &events, &species_id, amount, context);
    events
}

pub fn assert_complete_native_current(vessel: &Vessel, context: &str) {
    assert!(
        vessel.unresolved_materials.is_empty(),
        "{context}: unresolved material inventory"
    );
    assert!(
        vessel.surfaces.is_empty()
            && vessel.adsorbed.is_empty()
            && vessel.exchanges.is_empty()
            && vessel.solid_solutions.is_empty()
            && vessel.material_objects.is_empty()
            && vessel.electrodes.is_empty(),
        "{context}: this ledger check does not own every material reservoir"
    );
    let solution = vessel
        .solution
        .as_ref()
        .unwrap_or_else(|| panic!("{context}: no current solution characterization"));
    assert_eq!(
        solution.scope,
        kerotakis_core::vessel::SolutionScope::Complete,
        "{context}: mixture does not have complete solution scope"
    );
    assert_eq!(
        solution
            .provenance
            .as_ref()
            .map(|provenance| provenance.engine.as_str()),
        Some("PHREEQC (IPhreeqc, USGS)"),
        "{context}: current solution is not from the selected native route"
    );
    assert!(
        solution.ph.is_finite()
            && solution.ionic_strength.is_finite()
            && solution
                .particle_molality
                .is_some_and(|value| value.is_finite() && value >= 0.0),
        "{context}: incomplete or nonfinite current native diagnostics"
    );
}

pub fn positive_co2_gas_out(events: &[Event]) -> f64 {
    events
        .iter()
        .filter_map(|event| match event {
            Event::GasEvolved { species, moles, .. } if species.0 == "CO2" => Some(moles.0),
            _ => None,
        })
        .sum()
}

fn assert_gas_adjusted_inventory(
    before: &Vessel,
    after: &Vessel,
    events: &[Event],
    input_species: &SpeciesId,
    input_moles: Moles,
    context: &str,
) {
    let mut expected = strict_inventory(before, context);
    let input = species::lookup(input_species)
        .unwrap_or_else(|| panic!("{context}: unknown input species {}", input_species.0));
    let formula = stoich::parse_formula(input.formula)
        .unwrap_or_else(|error| panic!("{context}: input formula: {error}"));
    for (element, count) in formula.counts {
        *expected.elements.entry(element).or_default() += count * input_moles.0;
    }
    expected.charge += formula.charge * input_moles.0;

    // GasEvolved and GasAbsorbed cross the vessel boundary; contained gas is
    // an internal transfer and remains represented by the vessel ledger.
    for event in events {
        let (species, amount, sign) = match event {
            Event::GasEvolved {
                vessel,
                species,
                moles,
            } => {
                assert_eq!(*vessel, before.id, "{context}: gas-out vessel mismatch");
                (species, moles.0, -1.0)
            }
            Event::GasAbsorbed {
                vessel,
                species,
                moles,
            } => {
                assert_eq!(*vessel, before.id, "{context}: gas-in vessel mismatch");
                (species, moles.0, 1.0)
            }
            _ => continue,
        };
        assert!(
            amount.is_finite() && amount >= 0.0,
            "{context}: invalid gas event"
        );
        let gas = species::lookup(species)
            .unwrap_or_else(|| panic!("{context}: unknown gas species {}", species.0));
        let formula = stoich::parse_formula(gas.formula)
            .unwrap_or_else(|error| panic!("{context}: gas formula: {error}"));
        for (element, count) in formula.counts {
            *expected.elements.entry(element).or_default() += sign * count * amount;
        }
        expected.charge += sign * formula.charge * amount;
    }

    let actual = strict_inventory(after, context);
    let elements: BTreeSet<_> = expected
        .elements
        .keys()
        .chain(actual.elements.keys())
        .cloned()
        .collect();
    for element in elements {
        let left = expected.elements.get(&element).copied().unwrap_or(0.0);
        let right = actual.elements.get(&element).copied().unwrap_or(0.0);
        assert!(
            left.is_finite() && right.is_finite() && (left - right).abs() <= 1e-10,
            "{context}: {element} inventory changed {left:.15e} -> {right:.15e} mol"
        );
    }
    assert!(
        expected.charge.is_finite()
            && actual.charge.is_finite()
            && (expected.charge - actual.charge).abs() <= 1e-10,
        "{context}: formal charge changed {:.15e} -> {:.15e} eq",
        expected.charge,
        actual.charge
    );
}

fn strict_inventory(vessel: &Vessel, context: &str) -> ConservedLedger {
    assert!(
        vessel.unresolved_materials.is_empty(),
        "{context}: unresolved material inventory"
    );
    for portion in &vessel.contents {
        assert!(
            portion.moles.0.is_finite() && portion.moles.0 >= 0.0,
            "{context}: invalid {} amount {} mol",
            portion.species.0,
            portion.moles.0
        );
        let data = species::lookup(&portion.species).unwrap_or_else(|| {
            panic!(
                "{context}: unsupported inventory species {}",
                portion.species.0
            )
        });
        stoich::parse_formula(data.formula)
            .unwrap_or_else(|error| panic!("{context}: {} formula: {error}", portion.species.0));
    }
    ConservedLedger::from_vessel(vessel)
}
