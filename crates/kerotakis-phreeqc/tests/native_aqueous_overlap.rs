//! Native-stack controls for ownership of bicarbonate/acetic-acid overlap.
//!
//! These assertions exercise the production phase wrapper plus installed
//! PHREEQC route. They make no reaction-rate or reaction-heat claim.

#![cfg(feature = "engine")]

use kerotakis_core::ledger::ConservedLedger;
use kerotakis_core::script::parse_op;
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;
use std::collections::BTreeSet;

const IONIC_CURATED_ROW: &str = "HCO₃⁻ + CH₃COOH → CH₃COO⁻ + H₂O + CO₂↑";

fn production_stack() -> SolverStack {
    kerotakis_stack::standard_stack(vec![Box::new(PhaseEquilibrator::wrapping(Box::new(
        PhreeqcEquilibrator::new().expect("native PHREEQC engine"),
    )))])
}

fn run(bench: &mut Bench, stack: &mut SolverStack, command: &str) -> Vec<Event> {
    let op = parse_op(command)
        .unwrap_or_else(|error| panic!("{command}: {error}"))
        .unwrap_or_else(|| panic!("{command}: expected one operator"));
    bench
        .step_with(op, stack, &PermissiveScreen)
        .unwrap_or_else(|error| panic!("{command}: {error:?}"))
}

fn assert_native_current(vessel: &Vessel, context: &str) {
    let solution = vessel
        .solution
        .as_ref()
        .expect("current aqueous characterization");
    assert!(
        solution.ph.is_finite() && solution.ionic_strength.is_finite(),
        "{context}: invalid current aqueous characterization"
    );
    if solution.scope == kerotakis_core::vessel::SolutionScope::SolventOnly {
        // The existing pure-water analytic route intentionally has no native
        // particle census. This is setup admission, not a full mixture answer.
        assert!(
            vessel.contents.iter().all(|p| p.species.0 == "water"),
            "{context}: solvent-only answer cannot stand in for mixture speciation"
        );
    } else {
        assert_eq!(
            solution.scope,
            kerotakis_core::vessel::SolutionScope::Complete,
            "{context}: mixture lacks complete characterization"
        );
        assert!(
            solution
                .particle_molality
                .is_some_and(|n| n.is_finite() && n >= 0.0),
            "{context}: native numerical particle census absent or invalid"
        );
    }
}

fn gas_event_moles(events: &[Event], contained: bool) -> f64 {
    events
        .iter()
        .filter_map(|event| match (contained, event) {
            (false, Event::GasEvolved { species, moles, .. }) if species.0 == "CO2" => {
                Some(moles.0)
            }
            (true, Event::GasContained { species, moles, .. }) if species.0 == "CO2" => {
                Some(moles.0)
            }
            _ => None,
        })
        .sum()
}

fn assert_no_ionic_curated_replay(events: &[Event], context: &str) {
    assert!(
        !events.iter().any(|event| matches!(
            event,
            Event::ReactionOccurred { equation, .. } if equation == IONIC_CURATED_ROW
        )),
        "{context}: the tagged ionic curated row fired despite the selected aqueous owner"
    );
}

/// Check contents through the public conserved ledger and close gas crossing
/// events at the boundary. Unknown contents/reservoirs are a scope failure.
fn strict_ledger(vessel: &Vessel) -> ConservedLedger {
    assert!(
        vessel.unresolved_materials.is_empty(),
        "scope gap: unresolved inventory"
    );
    assert!(
        vessel.free_proton.is_finite()
            && vessel.free_proton >= 0.0
            && vessel.free_hydroxide.is_finite()
            && vessel.free_hydroxide >= 0.0
            && vessel.solute_charge.is_finite(),
        "invalid measured/basis coordinates"
    );
    // Free H+/OH- are measured diagnostics, not extra material to add on top
    // of the contents ledger. The analytical solute-charge coordinate must
    // agree with the represented non-basis aqueous ions.
    let represented_solute_charge: f64 = vessel
        .contents
        .iter()
        .filter(|p| p.phase == Phase::Aqueous && !species::is_acid_base_basis(&p.species.0))
        .map(|p| {
            stoich::parse_formula(species::lookup(&p.species).expect("registered ion").formula)
                .expect("ion formula")
                .charge
                * p.moles.0
        })
        .sum();
    assert!(
        (vessel.solute_charge - represented_solute_charge).abs() <= 1e-10,
        "analytical solute charge disagrees with represented ions"
    );
    assert!(vessel.surfaces.is_empty(), "scope gap: surface inventory");
    assert!(
        vessel.electrodes.is_empty(),
        "scope gap: electrode inventory"
    );
    assert!(
        vessel.material_objects.is_empty(),
        "scope gap: prepared material inventory"
    );
    assert!(vessel.adsorbed.is_empty(), "scope gap: adsorbed inventory");
    assert!(
        vessel.exchanges.is_empty(),
        "scope gap: exchanger inventory"
    );
    assert!(
        vessel.solid_solutions.is_empty(),
        "scope gap: solid-solution inventory"
    );
    for portion in &vessel.contents {
        assert!(
            portion.moles.0.is_finite() && portion.moles.0 >= 0.0,
            "invalid {} amount {} mol",
            portion.species.0,
            portion.moles.0
        );
        let data = species::lookup(&portion.species)
            .unwrap_or_else(|| panic!("ConservedLedger scope gap: {}", portion.species.0));
        stoich::parse_formula(data.formula).unwrap_or_else(|error| {
            panic!(
                "ConservedLedger scope gap: {} formula: {error}",
                portion.species.0
            )
        });
    }
    ConservedLedger::from_vessel(vessel)
}

fn assert_gas_adjusted_ledger(
    before: &Vessel,
    after: &Vessel,
    events: &[Event],
    input: Option<(&SpeciesId, Moles)>,
    context: &str,
) {
    let mut expected = strict_ledger(before);
    // The external reagent dose is an inlet to this ledger, before any
    // solver. Parse supplied Add quantities once; never infer feed from output.
    if let Some((species_id, amount)) = input {
        let data = species::lookup(species_id).expect("registered input reagent");
        let formula = stoich::parse_formula(data.formula).expect("input formula");
        for (element, count) in formula.counts {
            *expected.elements.entry(element).or_default() += count * amount.0;
        }
        expected.charge += formula.charge * amount.0;
    }
    for event in events {
        let (species_id, moles, sign) = match event {
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
            moles.is_finite() && moles >= 0.0,
            "{context}: invalid gas event"
        );
        let data = species::lookup(species_id)
            .unwrap_or_else(|| panic!("ConservedLedger scope gap: gas {}", species_id.0));
        let formula = stoich::parse_formula(data.formula)
            .unwrap_or_else(|error| panic!("gas {} formula: {error}", species_id.0));
        for (element, count) in formula.counts {
            *expected.elements.entry(element).or_default() += sign * count * moles;
        }
        expected.charge += sign * formula.charge * moles;
    }
    let actual = strict_ledger(after);
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
            "{context}: {element} ledger changed {left:.15e} -> {right:.15e} mol"
        );
    }
    assert!(
        expected.charge.is_finite()
            && actual.charge.is_finite()
            && (expected.charge - actual.charge).abs() <= 1e-10,
        "{context}: charge changed {:.15e} -> {:.15e} eq",
        expected.charge,
        actual.charge
    );
}

fn apply_with_ledger(
    bench: &mut Bench,
    stack: &mut SolverStack,
    command: &str,
    context: &str,
) -> Vec<Event> {
    let before = bench
        .vessel(VesselId(0))
        .expect("vessel before step")
        .clone();
    let op = parse_op(command)
        .expect("input command")
        .expect("one input");
    let (input_species, input_moles) = match &op {
        Operator::Add { species, moles, .. } => (species.clone(), *moles),
        _ => panic!("dose control requires an Add operator: {command}"),
    };
    let events = bench
        .step_with(op, stack, &PermissiveScreen)
        .expect("native input step");
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::SolverFailed { .. })),
        "{context} {command}: {events:?}"
    );
    let after = bench.vessel(VesselId(0)).expect("vessel after step");
    assert_gas_adjusted_ledger(
        &before,
        after,
        &events,
        Some((&input_species, input_moles)),
        context,
    );
    assert_native_current(after, context);
    assert_no_ionic_curated_replay(&events, context);
    events
}

#[test]
fn fresh_bicarbonate_and_acetic_acid_fizz_in_both_addition_orders_then_accept_more_acid() {
    for (label, first, second) in [
        ("base first", "NaHCO3 0.02mol", "CH3COOH 0.005mol"),
        ("acid first", "CH3COOH 0.005mol", "NaHCO3 0.02mol"),
    ] {
        let mut bench = Bench::new();
        let mut stack = production_stack();
        apply_with_ledger(&mut bench, &mut stack, "add v1 water 100mL", label);
        apply_with_ledger(&mut bench, &mut stack, &format!("add v1 {first}"), label);
        let fizz = apply_with_ledger(&mut bench, &mut stack, &format!("add v1 {second}"), label);
        assert!(
            gas_event_moles(&fizz, false) > 1e-4,
            "{label}: expected actual outward CO2 gas above 1e-4 mol, got {}",
            gas_event_moles(&fizz, false)
        );

        let before_later_dose = bench.vessel(VesselId(0)).expect("vessel").clone();
        assert!(
            before_later_dose.moles_of(&SpeciesId::new("HCO3-")).0 > 1e-3,
            "{label}: no represented bicarbonate remains for the fresh acid dose"
        );
        let later_fizz =
            apply_with_ledger(&mut bench, &mut stack, "add v1 CH3COOH 0.005mol", label);
        assert!(
            gas_event_moles(&later_fizz, false) > 1e-4,
            "{label}: later fresh acid did not evolve actual CO2"
        );
    }
}

#[test]
fn open_and_sealed_boundaries_book_native_route_co2_to_their_actual_owner() {
    for sealed in [false, true] {
        let label = if sealed { "sealed" } else { "open" };
        let mut bench = Bench::new();
        let mut stack = production_stack();
        apply_with_ledger(&mut bench, &mut stack, "add v1 water 100mL", label);
        if sealed {
            let events = run(&mut bench, &mut stack, "seal v1 500mL");
            assert!(
                !events
                    .iter()
                    .any(|event| matches!(event, Event::SolverFailed { .. })),
                "{label}: sealing failed: {events:?}"
            );
        }
        apply_with_ledger(&mut bench, &mut stack, "add v1 NaHCO3 0.02mol", label);
        let fizz = apply_with_ledger(&mut bench, &mut stack, "add v1 CH3COOH 0.005mol", label);
        let owned = if sealed {
            let contained = gas_event_moles(&fizz, true);
            assert!(
                contained > 1e-4,
                "sealed route contained only {contained} mol CO2"
            );
            let vessel = bench.vessel(VesselId(0)).expect("sealed vessel");
            assert!(
                vessel.contents.iter().any(|portion| {
                    portion.species.0 == "CO2"
                        && portion.phase == Phase::Gas
                        && portion.moles.0 > 1e-4
                }),
                "sealed CO2 event is not represented in the owned gas headspace"
            );
            assert_eq!(
                gas_event_moles(&fizz, false),
                0.0,
                "sealed route vented CO2"
            );
            contained
        } else {
            let vented = gas_event_moles(&fizz, false);
            assert!(vented > 1e-4, "open route vented only {vented} mol CO2");
            assert_eq!(gas_event_moles(&fizz, true), 0.0, "open route retained CO2");
            vented
        };
        assert!(owned.is_finite());
    }
}

#[test]
fn unrelated_salt_and_typed_zero_time_repeats_do_not_replay_the_ionic_row() {
    let mut bench = Bench::new();
    let mut stack = production_stack();
    apply_with_ledger(&mut bench, &mut stack, "add v1 water 100mL", "setup");
    apply_with_ledger(&mut bench, &mut stack, "add v1 NaHCO3 0.02mol", "setup");
    let fizz = apply_with_ledger(
        &mut bench,
        &mut stack,
        "add v1 CH3COOH 0.005mol",
        "initial fizz",
    );
    assert!(gas_event_moles(&fizz, false) > 1e-4);
    apply_with_ledger(
        &mut bench,
        &mut stack,
        "add v1 NaCl 0.001mol",
        "unrelated addition",
    );

    for repeat in 0..3 {
        let before = bench.vessel(VesselId(0)).expect("before repeat").clone();
        let events = bench
            .step_with(
                Operator::Wait { seconds: 0.0 },
                &mut stack,
                &PermissiveScreen,
            )
            .expect("typed zero-time repeat");
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, Event::SolverFailed { .. })),
            "zero-time repeat {repeat}: {events:?}"
        );
        let after = bench.vessel(VesselId(0)).expect("after repeat");
        assert_gas_adjusted_ledger(
            &before,
            after,
            &events,
            None,
            &format!("zero-time {repeat}"),
        );
        assert_native_current(after, &format!("zero-time {repeat}"));
        let scalar_delta = (before.free_proton - after.free_proton).abs()
            + (before.free_hydroxide - after.free_hydroxide).abs()
            + (before.solute_charge - after.solute_charge).abs();
        assert!(
            scalar_delta <= 1e-10,
            "zero-time {repeat}: measured/basis coordinate drift {scalar_delta} mol/eq"
        );
        assert_no_ionic_curated_replay(&events, &format!("zero-time {repeat}"));
    }
}
