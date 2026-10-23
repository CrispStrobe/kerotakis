//! Regression coverage for the native numerical aqueous-particle census and
//! its use by the phase-coupled production solver.
//!
//! The original CaCl2/AgNO3 cooling forecast is retained as the first case.
//! Nearby and no-silver controls separate its numerical liquidus behaviour
//! from the AgCl precipitate without changing the 20 kJ cooling request.

#![cfg(feature = "engine")]

use kerotakis_core::ledger::ConservedLedger;
use kerotakis_core::script::parse_op;
use kerotakis_core::*;
use kerotakis_phreeqc::{Phreeqc, PhreeqcEquilibrator};
use std::collections::BTreeMap;

fn production_stack() -> SolverStack {
    kerotakis_stack::standard_stack(vec![Box::new(PhaseEquilibrator::wrapping(Box::new(
        PhreeqcEquilibrator::new().expect("native PHREEQC engine"),
    )))])
}

fn run(bench: &mut Bench, stack: &mut SolverStack, command: &str) -> Vec<Event> {
    bench
        .step_with(
            parse_op(command)
                .expect("parse command")
                .expect("one operator"),
            stack,
            &PermissiveScreen,
        )
        .expect("native production-stack step")
}

fn water_moles(vessel: &Vessel, phase: Option<Phase>) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|portion| portion.species.0 == "water")
        .filter(|portion| phase.is_none_or(|phase| portion.phase == phase))
        .map(|portion| portion.moles.0)
        .sum()
}

fn elements_without_hydrogen_or_oxygen(vessel: &Vessel) -> BTreeMap<String, f64> {
    let mut totals = BTreeMap::new();
    for portion in &vessel.contents {
        let data = species::lookup(&portion.species).unwrap_or_else(|| {
            panic!(
                "unregistered species in native output: {}",
                portion.species.0
            )
        });
        let formula = stoich::parse_formula(data.formula).unwrap_or_else(|error| {
            panic!(
                "invalid registry formula for {}: {error}",
                portion.species.0
            )
        });
        for (element, count) in formula.counts {
            if element != "H" && element != "O" {
                *totals.entry(element).or_insert(0.0) += count * portion.moles.0;
            }
        }
    }
    totals
}

fn assert_element_ledger(before: &Vessel, after: &Vessel, context: &str) {
    let before = elements_without_hydrogen_or_oxygen(before);
    let after = elements_without_hydrogen_or_oxygen(after);
    let elements: std::collections::BTreeSet<_> =
        before.keys().chain(after.keys()).cloned().collect();
    for element in elements {
        let left = before.get(&element).copied().unwrap_or(0.0);
        let right = after.get(&element).copied().unwrap_or(0.0);
        assert!(
            (left - right).abs() <= 1e-10,
            "{context}: {element} ledger changed {left:.15e} -> {right:.15e} mol"
        );
    }
}

fn strict_material_ledger(vessel: &Vessel) -> ConservedLedger {
    // ConservedLedger is the authority for the full represented inventory,
    // including H/O, the analytical base-equivalent basis coordinate, and
    // charge. Its implementation skips unregistered or unparsable contents,
    // so fail loudly if a new reservoir falls outside that scope.
    assert!(
        vessel.surfaces.is_empty(),
        "scope gap: unexpected surface inventory"
    );
    assert!(
        vessel.electrodes.is_empty(),
        "scope gap: unexpected electrode inventory"
    );
    assert!(
        vessel.material_objects.is_empty(),
        "scope gap: unexpected prepared-material inventory"
    );
    assert!(
        vessel.adsorbed.is_empty(),
        "scope gap: unexpected adsorbed inventory"
    );
    assert!(
        vessel.exchanges.is_empty(),
        "scope gap: unexpected exchanger inventory"
    );
    assert!(
        vessel.solid_solutions.is_empty(),
        "scope gap: unexpected solid-solution inventory"
    );
    let mut contents_elements = BTreeMap::<String, f64>::new();
    let mut contents_charge = 0.0;
    for portion in &vessel.contents {
        assert!(
            portion.moles.0.is_finite() && portion.moles.0 >= 0.0,
            "unledgerable amount for {}: {} mol",
            portion.species.0,
            portion.moles.0
        );
        let data = species::lookup(&portion.species).unwrap_or_else(|| {
            panic!(
                "ConservedLedger scope gap: unregistered {}",
                portion.species.0
            )
        });
        let formula = stoich::parse_formula(data.formula).unwrap_or_else(|error| {
            panic!(
                "ConservedLedger scope gap: {} has no parseable formula: {error}",
                portion.species.0
            )
        });
        for (element, count) in formula.counts {
            *contents_elements.entry(element).or_default() += count * portion.moles.0;
        }
        contents_charge += formula.charge * portion.moles.0;
    }
    let ledger = ConservedLedger::from_vessel(vessel);
    let elements: std::collections::BTreeSet<_> = contents_elements
        .keys()
        .chain(ledger.elements.keys())
        .cloned()
        .collect();
    for element in elements {
        let expected = contents_elements.get(&element).copied().unwrap_or(0.0);
        let actual = ledger.elements.get(&element).copied().unwrap_or(0.0);
        assert!(
            (expected - actual).abs() <= 1e-12,
            "ConservedLedger scope gap: contents {element}={expected:.15e}, ledger={actual:.15e}"
        );
    }
    assert!(
        (contents_charge - ledger.charge).abs() <= 1e-12,
        "ConservedLedger scope gap: contents charge={contents_charge:.15e}, ledger={:.15e}",
        ledger.charge
    );
    ledger
}

fn assert_full_material_and_charge_ledger(
    before: &Vessel,
    after: &Vessel,
    events: &[Event],
    context: &str,
) {
    let mut expected = strict_material_ledger(before);
    for event in events {
        let (species_id, moles, sign) = match event {
            Event::GasEvolved {
                vessel,
                species,
                moles,
            } => {
                assert_eq!(
                    *vessel, before.id,
                    "{context}: vent belongs to another vessel"
                );
                (species, moles.0, -1.0)
            }
            Event::GasAbsorbed {
                vessel,
                species,
                moles,
            } => {
                assert_eq!(
                    *vessel, before.id,
                    "{context}: absorption belongs to another vessel"
                );
                (species, moles.0, 1.0)
            }
            _ => continue,
        };
        assert!(
            moles.is_finite() && moles >= 0.0,
            "{context}: invalid gas event amount"
        );
        let data = species::lookup(species_id).unwrap_or_else(|| {
            panic!(
                "ConservedLedger scope gap: unregistered gas {}",
                species_id.0
            )
        });
        let formula = stoich::parse_formula(data.formula).unwrap_or_else(|error| {
            panic!("ConservedLedger scope gap: gas {}: {error}", species_id.0)
        });
        for (element, count) in formula.counts {
            *expected.elements.entry(element).or_default() += sign * count * moles;
        }
        expected.charge += sign * formula.charge * moles;
    }

    let actual = strict_material_ledger(after);
    let elements: std::collections::BTreeSet<_> = expected
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
            "{context}: full {element} ledger changed {left:.15e} -> {right:.15e} mol"
        );
    }
    assert!(
        expected.charge.is_finite()
            && actual.charge.is_finite()
            && (expected.charge - actual.charge).abs() <= 1e-10,
        "{context}: represented charge changed {:.15e} -> {:.15e} eq",
        expected.charge,
        actual.charge
    );
}

fn particle_census(vessel: &Vessel) -> f64 {
    let solution = vessel.solution.as_ref().expect("current aqueous solution");
    let value = solution
        .particle_molality
        .expect("new native solve carries its numerical particle census");
    assert!(
        value.is_finite() && value >= 0.0,
        "invalid particle census {value}"
    );
    value
}

fn last_water_transition(events: &[Event]) -> f64 {
    events
        .iter()
        .rev()
        .find_map(|event| match event {
            Event::StateChanged {
                species,
                from: Phase::Liquid,
                to: Phase::Solid,
                at,
                ..
            }
            | Event::StateChanged {
                species,
                from: Phase::Solid,
                to: Phase::Liquid,
                at,
                ..
            } if species.0 == "water" => Some(at.0),
            _ => None,
        })
        .expect("a water phase transition was recorded")
}

fn latest_water_transition(events: &[Event], previous: f64) -> f64 {
    events
        .iter()
        .rev()
        .find_map(|event| match event {
            Event::StateChanged {
                species,
                from: Phase::Liquid,
                to: Phase::Solid,
                at,
                ..
            }
            | Event::StateChanged {
                species,
                from: Phase::Solid,
                to: Phase::Liquid,
                at,
                ..
            } if species.0 == "water" => Some(at.0),
            _ => None,
        })
        .unwrap_or(previous)
}

fn assert_settled_partial_brine(vessel: &Vessel, events: &[Event], context: &str) {
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::SolverFailed { .. })),
        "{context}: phase/aqueous loop failed: {events:?}"
    );
    assert!(
        water_moles(vessel, Some(Phase::Solid)) > 0.0,
        "{context}: no ice"
    );
    assert!(
        water_moles(vessel, Some(Phase::Liquid)) > 0.0,
        "{context}: no residual liquid"
    );
    let _ = particle_census(vessel);
    let liquidus = solve::vessel_transitions(vessel).0.freezing_k;
    let event_temperature = last_water_transition(events);
    assert!(
        (vessel.temperature.0 - liquidus).abs() <= 1e-6,
        "{context}: vessel T={} K vs own liquidus {liquidus} K",
        vessel.temperature.0
    );
    assert!(
        (vessel.temperature.0 - event_temperature).abs() <= 1e-6,
        "{context}: vessel T={} K vs last water event {event_temperature} K",
        vessel.temperature.0
    );
}

fn assert_microtransfer_closure(vessel: &Vessel, last_actual_water_event_k: f64, context: &str) {
    assert!(
        water_moles(vessel, Some(Phase::Solid)) > 0.0,
        "{context}: no ice"
    );
    assert!(
        water_moles(vessel, Some(Phase::Liquid)) > 0.0,
        "{context}: no liquid"
    );
    let solution = vessel.solution.as_ref().expect("fresh native solution");
    let census = particle_census(vessel);
    assert!(solution.ph.is_finite(), "{context}: nonfinite native pH");
    assert!(
        solution.ionic_strength.is_finite() && solution.ionic_strength >= 0.0,
        "{context}: invalid native ionic strength"
    );
    let liquidus = solve::vessel_transitions(vessel).0.freezing_k;
    assert!(
        liquidus.is_finite() && (252.0..=273.15).contains(&liquidus),
        "{context}: current liquidus outside admitted domain: {liquidus} K"
    );
    assert!(
        (vessel.temperature.0 - liquidus).abs() <= 1e-6,
        "{context}: vessel T={} K vs own liquidus {liquidus} K",
        vessel.temperature.0
    );
    assert!(
        (vessel.temperature.0 - last_actual_water_event_k).abs() <= 1e-6,
        "{context}: vessel T={} K vs last actual water event {last_actual_water_event_k} K",
        vessel.temperature.0
    );
    let zero_transfer_sensible_residual_j = vessel.energy_between(vessel.temperature.0, liquidus);
    assert!(
        zero_transfer_sensible_residual_j.is_finite()
            && zero_transfer_sensible_residual_j.abs() <= 0.001,
        "{context}: zero-transfer sensible phase residual is {zero_transfer_sensible_residual_j} J (particle census {census})"
    );
}

fn transferred_energy(events: &[Event], heating: bool) -> f64 {
    events
        .iter()
        .find_map(|event| match event {
            Event::EnergyTransferred {
                heating: actual_heating,
                delivered_j,
                ..
            } if *actual_heating == heating => Some(*delivered_j),
            _ => None,
        })
        .expect("the requested energy transfer was recorded")
}

fn assert_native_state_stable(before: &Vessel, after: &Vessel, context: &str) {
    let mut left = BTreeMap::<(String, Phase), f64>::new();
    let mut right = BTreeMap::<(String, Phase), f64>::new();
    for portion in &before.contents {
        *left
            .entry((portion.species.0.clone(), portion.phase))
            .or_default() += portion.moles.0;
    }
    for portion in &after.contents {
        *right
            .entry((portion.species.0.clone(), portion.phase))
            .or_default() += portion.moles.0;
    }
    let identities: std::collections::BTreeSet<_> =
        left.keys().chain(right.keys()).cloned().collect();
    let represented_delta: f64 = identities
        .iter()
        .map(|identity| {
            (left.get(identity).copied().unwrap_or(0.0)
                - right.get(identity).copied().unwrap_or(0.0))
            .abs()
        })
        .sum();
    let scalar_delta = (before.free_proton - after.free_proton).abs()
        + (before.free_hydroxide - after.free_hydroxide).abs()
        + (before.solute_charge - after.solute_charge).abs();
    assert!(
        represented_delta.is_finite()
            && scalar_delta.is_finite()
            && represented_delta + scalar_delta <= 1e-10,
        "{context}: fresh represented inventory/free-ion/charge drift is {} mol/eq",
        represented_delta + scalar_delta
    );
    assert!(
        (before.temperature.0 - after.temperature.0).abs() <= 1e-6,
        "{context}: zero-time temperature changed {} -> {} K",
        before.temperature.0,
        after.temperature.0
    );
    let before_liquidus = solve::vessel_transitions(before).0.freezing_k;
    let after_liquidus = solve::vessel_transitions(after).0.freezing_k;
    assert!(
        before_liquidus.is_finite()
            && after_liquidus.is_finite()
            && (before_liquidus - after_liquidus).abs() <= 1e-6
            && (before.temperature.0 - before_liquidus).abs() <= 1e-6
            && (after.temperature.0 - after_liquidus).abs() <= 1e-6,
        "{context}: successive own liquidus states do not agree: T={} K / {} K, liquidus={} K / {} K",
        before.temperature.0,
        after.temperature.0,
        before_liquidus,
        after_liquidus
    );
    let before_solution = before.solution.as_ref().expect("previous fresh solution");
    let after_solution = after.solution.as_ref().expect("current fresh solution");
    assert!(
        (before_solution.ph - after_solution.ph).abs() <= 1e-6
            && (before_solution.ionic_strength - after_solution.ionic_strength).abs() <= 1e-10
            && (particle_census(before) - particle_census(after)).abs() <= 1e-10,
        "{context}: fresh native pH, ionic strength, or particle census drifted"
    );
}

#[test]
fn original_nearby_and_no_silver_cooling_and_partial_rewarming_settle() {
    // The first row is the exact frozen CLI protocol. The second perturbs both
    // salt amounts slightly; the third removes the precipitating Ag reagent.
    let cases = [
        ("original Ca/Ag", 0.05, Some(0.005)),
        ("nearby Ca/Ag", 0.050123, Some(0.0050123)),
        ("Ca only", 0.05, None),
    ];

    for (label, calcium_chloride, silver_nitrate) in cases {
        let mut bench = Bench::new();
        let mut stack = production_stack();
        let mut events = run(&mut bench, &mut stack, "add v1 water 100mL");
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, Event::SolverFailed { .. })),
            "{label}: {events:?}"
        );
        events = run(
            &mut bench,
            &mut stack,
            &format!("add v1 CaCl2 {calcium_chloride:.10}mol"),
        );
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, Event::SolverFailed { .. })),
            "{label}: {events:?}"
        );
        if let Some(silver) = silver_nitrate {
            events = run(
                &mut bench,
                &mut stack,
                &format!("add v1 AgNO3 {silver:.10}mol"),
            );
            assert!(
                !events
                    .iter()
                    .any(|event| matches!(event, Event::SolverFailed { .. })),
                "{label}: {events:?}"
            );
        }

        let before_cooling = bench.vessel(VesselId(0)).expect("beaker").clone();
        let water_before = water_moles(&before_cooling, None);
        let cooling_events = run(&mut bench, &mut stack, "cool v1 20kJ");
        let cooled = bench
            .vessel(VesselId(0))
            .expect("beaker after cooling")
            .clone();
        assert_settled_partial_brine(&cooled, &cooling_events, label);
        assert_element_ledger(&before_cooling, &cooled, label);
        assert!(
            (water_moles(&cooled, None) - water_before).abs() <= 1e-10,
            "{label}: total water changed {} -> {} mol",
            water_before,
            water_moles(&cooled, None)
        );
        assert!(
            (transferred_energy(&cooling_events, false) - 20_000.0).abs() <= 0.001,
            "{label}: delivered cooling was not 20,000 J"
        );

        if silver_nitrate.is_some() {
            let held_agcl = cooled.moles_of(&SpeciesId::new("AgCl")).0;
            assert!(
                held_agcl > 1e-3,
                "original AgCl precipitate was not retained: {held_agcl}"
            );
            let solution = cooled.solution.as_ref().expect("residual brine solution");
            let report_sum: f64 = solution
                .species
                .iter()
                .filter(|species| species.name != "H2O")
                .map(|species| species.molality)
                .sum();
            let held_solid_molality = held_agcl / solution.solvent_kg.expect("native water basis");
            let census = particle_census(&cooled);
            assert!(
                (census - report_sum).abs() < 0.5 * held_solid_molality,
                "{label}: held AgCl appears to have entered aqueous particle census; census={census}, report sum={report_sum}, solid={held_solid_molality} mol/kgw"
            );
            assert!(
                (census - (report_sum + held_solid_molality)).abs() > 0.5 * held_solid_molality,
                "{label}: census matches the rounded aqueous sum plus the held AgCl solid"
            );
        }

        let water_before_warming = water_moles(&cooled, None);
        let warm_events = run(&mut bench, &mut stack, "heat v1 100J");
        let warmed = bench
            .vessel(VesselId(0))
            .expect("beaker after warming")
            .clone();
        assert_settled_partial_brine(&warmed, &warm_events, &format!("{label} after 100 J"));
        assert_element_ledger(&cooled, &warmed, &format!("{label} after 100 J"));
        assert!(
            (water_moles(&warmed, None) - water_before_warming).abs() <= 1e-10,
            "{label}: total water changed during rewarming"
        );
        assert!(
            (transferred_energy(&warm_events, true) - 100.0).abs() <= 0.001,
            "{label}: delivered rewarming was not 100 J"
        );
    }
}

#[test]
fn carbonate_weak_acid_microtransfers_settle_and_repeat_without_drift() {
    // These are prospective regression expectations, not newly observed
    // passes. The two exact inputs preserve the frozen failures; one-percent
    // weak-acid perturbations check nearby states without tuning the trace.
    let cases = [
        (
            "carbonate + acetic acid original",
            "Na2CO3",
            0.02,
            "CH3COOH",
            0.05,
        ),
        (
            "carbonate + acetic acid nearby",
            "Na2CO3",
            0.02,
            "CH3COOH",
            0.0505,
        ),
        (
            "bicarbonate + phosphoric acid original",
            "NaHCO3",
            0.05,
            "H3PO4",
            0.02,
        ),
        (
            "bicarbonate + phosphoric acid nearby",
            "NaHCO3",
            0.05,
            "H3PO4",
            0.0202,
        ),
    ];

    for (label, base, base_mol, acid, acid_mol) in cases {
        let mut bench = Bench::new();
        let mut stack = production_stack();
        for command in [
            "add v1 water 100mL".to_string(),
            format!("add v1 {base} {base_mol:.10}mol"),
            format!("add v1 {acid} {acid_mol:.10}mol"),
        ] {
            let events = run(&mut bench, &mut stack, &command);
            assert!(
                !events
                    .iter()
                    .any(|event| matches!(event, Event::SolverFailed { .. })),
                "{label} setup {command}: {events:?}"
            );
        }
        let initial = bench
            .vessel(VesselId(0))
            .expect("current brine before cooling");
        let _ = particle_census(initial);
        assert!(
            initial
                .solution
                .as_ref()
                .expect("current brine")
                .ph
                .is_finite(),
            "{label}: setup did not leave a fresh finite native solution"
        );

        let before_cooling = bench
            .vessel(VesselId(0))
            .expect("beaker before cooling")
            .clone();
        let cooling_events = run(&mut bench, &mut stack, "cool v1 20kJ");
        assert!(
            !cooling_events
                .iter()
                .any(|event| matches!(event, Event::SolverFailed { .. })),
            "{label}: phase/aqueous route failed: {cooling_events:?}"
        );
        let cooled = bench
            .vessel(VesselId(0))
            .expect("beaker after cooling")
            .clone();
        let mut last_event_k = last_water_transition(&cooling_events);
        assert_microtransfer_closure(&cooled, last_event_k, label);
        assert_full_material_and_charge_ledger(&before_cooling, &cooled, &cooling_events, label);
        assert!(
            (transferred_energy(&cooling_events, false) - 20_000.0).abs() <= 0.001,
            "{label}: delivered cooling did not remain 20,000 J"
        );

        let mut previous = cooled;
        for repeat in 0..3 {
            // The public script grammar requires positive durations; exercise
            // internal zero-time re-entry through its typed operator API.
            let zero_time_events = bench
                .step_with(
                    Operator::Wait { seconds: 0.0 },
                    &mut stack,
                    &PermissiveScreen,
                )
                .expect("typed zero-time phase re-entry");
            assert!(
                !zero_time_events
                    .iter()
                    .any(|event| matches!(event, Event::SolverFailed { .. })),
                "{label} zero-time repeat {repeat}: {zero_time_events:?}"
            );
            last_event_k = latest_water_transition(&zero_time_events, last_event_k);
            let current = bench
                .vessel(VesselId(0))
                .expect("beaker after zero-time repeat")
                .clone();
            assert_microtransfer_closure(
                &current,
                last_event_k,
                &format!("{label} zero-time repeat {repeat}"),
            );
            assert_full_material_and_charge_ledger(
                &previous,
                &current,
                &zero_time_events,
                &format!("{label} zero-time repeat {repeat}"),
            );
            assert_native_state_stable(
                &previous,
                &current,
                &format!("{label} zero-time repeat {repeat}"),
            );
            previous = current;
        }
    }
}

fn independent_molality_sum(vessel: &Vessel) -> f64 {
    let solution = vessel.solution.as_ref().expect("solution");
    let water_kg = solution.solvent_kg.expect("native water basis");
    let totals = elements_without_hydrogen_or_oxygen(vessel);
    let names: Vec<_> = solution
        .species
        .iter()
        .filter(|species| species.name != "H2O")
        .map(|species| species.name.as_str())
        .collect();
    assert!(
        !names.is_empty(),
        "the native report named no aqueous species"
    );
    let headings = (0..names.len())
        .map(|index| format!("mol_{index}"))
        .collect::<Vec<_>>()
        .join(" ");
    let punches = names
        .iter()
        .map(|name| format!("MOL(\"{name}\")"))
        .collect::<Vec<_>>()
        .join(", ");
    let celsius = vessel.temperature.to_celsius();
    let calcium = totals.get("Ca").copied().unwrap_or(0.0) / water_kg;
    let chlorine = totals.get("Cl").copied().unwrap_or(0.0) / water_kg;
    let input = format!(
        "SELECTED_OUTPUT 1\n    -reset false\n    -high_precision true\n    -user_punch true\nUSER_PUNCH 1\n    -headings {headings}\n    -start\n10 PUNCH {punches}\n    -end\nSOLUTION 1\n    units mol/kgw\n    temp {celsius:.4}\n    pH 7 charge\n    water {water_kg:.9}\n    Ca {calcium:.12e}\n    Cl {chlorine:.12e}\nEND\n"
    );
    let database = kerotakis_phreeqc::aqueous_database("pitzer").expect("routed Pitzer database");
    let mut raw =
        Phreeqc::with_database(database.as_bytes()).expect("independent raw native engine");
    raw.run(&input)
        .expect("independent selected-output solution");
    let rows = raw.selected_output();
    let header = rows.first().expect("selected-output headings");
    let values = rows.last().expect("selected-output row");
    let columns: Vec<_> = (0..names.len())
        .map(|index| {
            header
                .iter()
                .position(|heading| heading == &format!("mol_{index}"))
                .expect("raw MOL selected-output column")
        })
        .collect();
    columns
        .into_iter()
        .map(|index| values[index].parse::<f64>().expect("native MOL value"))
        .sum()
}

#[test]
fn native_particle_census_retains_precision_and_matches_independent_mol_columns() {
    let mut bench = Bench::new();
    let mut stack = production_stack();
    run(&mut bench, &mut stack, "add v1 water 100mL");
    run(&mut bench, &mut stack, "add v1 CaCl2 0.0501234567mol");
    let vessel = bench.vessel(VesselId(0)).expect("brine");
    let solution = vessel.solution.as_ref().expect("native solution");
    let census = particle_census(vessel);
    let rounded_report_sum: f64 = solution
        .species
        .iter()
        .filter(|species| species.name != "H2O")
        .map(|species| species.molality)
        .sum();
    assert!(
        (census - rounded_report_sum).abs() > 1e-8,
        "non-round salt amount did not distinguish numerical census {census} from rounded report {rounded_report_sum}"
    );
    let raw_sum = independent_molality_sum(vessel);
    assert!(
        (census - raw_sum).abs() <= 1e-8,
        "SYS aqueous census {census} disagrees with independent raw MOL-column sum {raw_sum}"
    );
}
