//! Native integration controls for the typed atmospheric CO2 clock boundary.
//!
//! The production wait control uses the preserved water/NaOH/3600 s protocol.
//! Typed-pending controls test material ownership only; they make no new rate,
//! kinetic, calorimetric, or empirical transfer claim.
#![cfg(feature = "engine")]

use kerotakis_core::clock::{Clock, ClockContext, GasExchangeClock, CO2_TRANSFER_FLOOR_MOL};
use kerotakis_core::ledger::ConservedLedger;
use kerotakis_core::script::parse_op;
use kerotakis_core::vessel::StepStart;
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;
use std::collections::BTreeSet;

fn production_stack() -> SolverStack {
    kerotakis_stack::standard_stack(vec![Box::new(PhaseEquilibrator::wrapping(Box::new(
        PhreeqcEquilibrator::new().expect("native PHREEQC engine"),
    )))])
}

fn command(bench: &mut Bench, stack: &mut SolverStack, text: &str) -> Vec<Event> {
    let op = parse_op(text)
        .unwrap_or_else(|error| panic!("{text}: {error}"))
        .unwrap_or_else(|| panic!("{text}: expected one operator"));
    bench
        .step_with(op, stack, &PermissiveScreen)
        .unwrap_or_else(|error| panic!("{text}: {error:?}"))
}

fn apply(bench: &mut Bench, stack: &mut SolverStack, op: Operator, context: &str) -> Vec<Event> {
    bench
        .step_with(op, stack, &PermissiveScreen)
        .unwrap_or_else(|error| panic!("{context}: {error:?}"))
}

fn assert_no_solver_failure(events: &[Event], context: &str) {
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::SolverFailed { .. })),
        "{context}: {events:?}"
    );
}

fn assert_current_aqueous(vessel: &Vessel, context: &str, require_complete: bool) {
    let info = vessel
        .solution
        .as_ref()
        .unwrap_or_else(|| panic!("{context}: no current aqueous solution"));
    assert!(
        info.ph.is_finite() && info.ionic_strength.is_finite(),
        "{context}: invalid current solution: {info:?}"
    );
    if require_complete {
        assert_eq!(
            info.scope,
            vessel::SolutionScope::Complete,
            "{context}: incomplete aqueous answer"
        );
        assert!(
            info.particle_molality
                .is_some_and(|value| value.is_finite() && value >= 0.0),
            "{context}: current numerical particle census missing"
        );
    }
}

/// Fail closed if any physical stock falls outside the ledger's represented
/// species scope. Free H/OH and solute_charge are measured/basis diagnostics;
/// they are checked separately by the solver and are not double-counted here.
fn strict_ledger(vessel: &Vessel, context: &str) -> ConservedLedger {
    assert!(
        vessel.unresolved_materials.is_empty(),
        "{context}: unresolved physical inventory"
    );
    assert!(
        vessel.surfaces.is_empty(),
        "{context}: surface inventory gap"
    );
    assert!(
        vessel.electrodes.is_empty(),
        "{context}: electrode inventory gap"
    );
    assert!(
        vessel.material_objects.is_empty(),
        "{context}: material inventory gap"
    );
    assert!(
        vessel.adsorbed.is_empty(),
        "{context}: adsorbed inventory gap"
    );
    assert!(
        vessel.exchanges.is_empty(),
        "{context}: exchanger inventory gap"
    );
    assert!(
        vessel.solid_solutions.is_empty(),
        "{context}: solid-solution inventory gap"
    );
    assert!(
        vessel.free_proton.is_finite()
            && vessel.free_proton >= 0.0
            && vessel.free_hydroxide.is_finite()
            && vessel.free_hydroxide >= 0.0
            && vessel.solute_charge.is_finite(),
        "{context}: invalid current measured/basis coordinates"
    );
    let represented_charge: f64 = vessel
        .contents
        .iter()
        .filter(|portion| {
            portion.phase == Phase::Aqueous && !species::is_acid_base_basis(&portion.species.0)
        })
        .map(|portion| {
            let data = species::lookup(&portion.species)
                .unwrap_or_else(|| panic!("{context}: unknown ion {}", portion.species.0));
            stoich::parse_formula(data.formula)
                .unwrap_or_else(|error| panic!("{context}: {} formula: {error}", portion.species.0))
                .charge
                * portion.moles.0
        })
        .sum();
    assert!(
        (vessel.solute_charge - represented_charge).abs() <= 1e-10,
        "{context}: measured solute charge differs from represented ions"
    );
    for portion in &vessel.contents {
        assert!(
            portion.moles.0.is_finite() && portion.moles.0 >= 0.0,
            "{context}: invalid {} amount {} mol",
            portion.species.0,
            portion.moles.0
        );
        let data = species::lookup(&portion.species)
            .unwrap_or_else(|| panic!("{context}: ledger scope gap for {}", portion.species.0));
        stoich::parse_formula(data.formula)
            .unwrap_or_else(|error| panic!("{context}: {} formula: {error}", portion.species.0));
    }
    ConservedLedger::from_vessel(vessel)
}

/// Close every represented element, including hydrogen and oxygen, plus
/// formal charge against the actual gas crossings emitted by this operation.
fn assert_boundary_closed(before: &Vessel, after: &Vessel, events: &[Event], context: &str) {
    let mut expected = strict_ledger(before, context);
    for event in events {
        if let Event::VesselSealed {
            vessel,
            headspace_volume,
            trapped_air,
        } = event
        {
            assert_eq!(*vessel, before.id, "{context}: seal has wrong owner");
            let already_finite = matches!(
                before.headspace,
                vessel::Headspace::Sealed { .. } | vessel::Headspace::PressureControlled { .. }
            );
            let supplied = if already_finite {
                0.0
            } else {
                before.pressure.0 * headspace_volume.0 / (8_314.462_618 * before.temperature.0)
            };
            assert!(
                supplied.is_finite()
                    && supplied >= 0.0
                    && trapped_air.0.is_finite()
                    && (trapped_air.0 - supplied).abs() <= 1e-10,
                "{context}: seal did not report its actual ideal-gas input"
            );
            // Sealing imports a real boundary inventory. A swept vessel
            // traps nitrogen; an open one traps the stated dry room air.
            let (nitrogen, oxygen, carbon_dioxide) =
                if matches!(before.headspace, vessel::Headspace::Swept { .. }) {
                    (1.0, 0.0, 0.0)
                } else {
                    (0.7901, 0.2095, 0.0004)
                };
            *expected.elements.entry("N".to_owned()).or_default() += 2.0 * nitrogen * trapped_air.0;
            *expected.elements.entry("O".to_owned()).or_default() +=
                2.0 * (oxygen + carbon_dioxide) * trapped_air.0;
            *expected.elements.entry("C".to_owned()).or_default() += carbon_dioxide * trapped_air.0;
            continue;
        }
        let (species_id, amount, sign, owner) = match event {
            Event::GasAbsorbed {
                vessel,
                species,
                moles,
            } => (species, moles.0, 1.0, *vessel),
            Event::GasEvolved {
                vessel,
                species,
                moles,
            } => (species, moles.0, -1.0, *vessel),
            _ => continue,
        };
        assert_eq!(owner, before.id, "{context}: gas event has wrong owner");
        assert!(
            amount.is_finite() && amount >= 0.0,
            "{context}: invalid gas crossing {amount}"
        );
        let data = species::lookup(species_id)
            .unwrap_or_else(|| panic!("{context}: unknown gas {}", species_id.0));
        let formula = stoich::parse_formula(data.formula)
            .unwrap_or_else(|error| panic!("{context}: gas formula: {error}"));
        for (element, count) in formula.counts {
            *expected.elements.entry(element).or_default() += sign * count * amount;
        }
        expected.charge += sign * formula.charge * amount;
    }
    let actual = strict_ledger(after, context);
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
            "{context}: {element} changed {left:.15e} -> {right:.15e} mol"
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

fn co2_crossings(events: &[Event]) -> Vec<(bool, f64)> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::GasAbsorbed { species, moles, .. } if species.0 == "CO2" => {
                Some((true, moles.0))
            }
            Event::GasEvolved { species, moles, .. } if species.0 == "CO2" => {
                Some((false, moles.0))
            }
            _ => None,
        })
        .collect()
}

fn add_water_and_maybe_base(base: bool) -> (Bench, SolverStack) {
    let mut bench = Bench::new();
    let mut stack = production_stack();
    command(&mut bench, &mut stack, "add v1 water 100mL");
    if base {
        command(&mut bench, &mut stack, "add v1 NaOH 0.001mol");
    }
    (bench, stack)
}

#[test]
fn mild_base_original_hour_wait_closes_h_o_charge_and_books_actual_co2_once() {
    // This is the preserved original failure protocol: water100mL + NaOH0.001mol + wait3600s.
    let (mut bench, mut stack) = add_water_and_maybe_base(true);
    let before = bench
        .vessel(VesselId(0))
        .expect("before original wait")
        .clone();
    assert!(
        before.co2_partial_pressure_atm.is_some_and(f64::is_finite),
        "native setup must measure CO2 pressure before the clock interval"
    );
    assert!(
        !before.contents.iter().any(|portion| {
            portion.species.0 == "CO2" && portion.phase == Phase::Gas && portion.moles.0 > 0.0
        }),
        "first atmospheric CO2 inlet must not depend on a preexisting CO2(g) phase"
    );

    let events = command(&mut bench, &mut stack, "wait 3600s");
    assert_no_solver_failure(&events, "original one-hour CO2 wait");
    let after = bench.vessel(VesselId(0)).expect("after original wait");
    assert_current_aqueous(after, "original one-hour CO2 wait", true);
    assert_boundary_closed(&before, after, &events, "original one-hour CO2 wait");

    let crossings = co2_crossings(&events);
    assert_eq!(
        crossings.len(),
        1,
        "one clock parcel must be published once: {events:?}"
    );
    assert!(
        crossings[0].0,
        "ambient clock should absorb CO2: {crossings:?}"
    );
    assert!(
        crossings[0].1 > CO2_TRANSFER_FLOOR_MOL,
        "one-hour native/base case must spend an above-floor parcel, got {crossings:?}"
    );
    assert_eq!(
        after.pending_co2_transfer_mol, 0.0,
        "spent parcel remains pending"
    );
}

#[test]
fn bare_water_spends_an_explicitly_earned_carrier_without_claiming_a_new_rate() {
    let (mut bench, mut stack) = add_water_and_maybe_base(false);
    let before = bench.vessel(VesselId(0)).expect("bare-water state").clone();
    assert!(
        !before.contents.iter().any(|portion| {
            portion.species.0 == "CO2" && portion.phase == Phase::Gas && portion.moles.0 > 0.0
        }),
        "bare-water carrier test starts without CO2(g) inventory"
    );

    // This is an explicitly supplied already-earned clock parcel. It tests
    // the first-inlet native material boundary, not whether the room clock
    // can infer a rate from a bare-water state with no measured pCO2.
    let earned = CO2_TRANSFER_FLOOR_MOL * 2.0;
    bench.vessels[0].pending_co2_transfer_mol = earned;
    let before_spend = bench
        .vessel(VesselId(0))
        .expect("before bare-water solve")
        .clone();
    let events = apply(
        &mut bench,
        &mut stack,
        Operator::Wait { seconds: 0.0 },
        "bare-water earned carrier",
    );
    assert_no_solver_failure(&events, "bare-water earned carrier");
    let after = bench.vessel(VesselId(0)).expect("after bare-water solve");
    assert_current_aqueous(after, "bare-water earned carrier", true);
    assert_boundary_closed(&before_spend, after, &events, "bare-water earned carrier");
    let crossings = co2_crossings(&events);
    let inlets: Vec<_> = crossings
        .iter()
        .copied()
        .filter(|(inlet, _)| *inlet)
        .collect();
    assert_eq!(
        inlets,
        vec![(true, earned)],
        "earned clock inlet must occur once"
    );
    let outlets: Vec<_> = crossings
        .iter()
        .copied()
        .filter(|(inlet, _)| !*inlet)
        .collect();
    assert!(
        outlets.iter().map(|(_, amount)| amount).sum::<f64>() <= earned,
        "bare water cannot release more carbon than its actual inlet"
    );
    // Native equilibrium may release part of this inlet. Every outward
    // crossing remains in the independent full-element/charge ledger above.
    assert_eq!(after.pending_co2_transfer_mol, 0.0);
    assert!(before
        .contents
        .iter()
        .all(|portion| portion.species.0 == "water"));
}

#[test]
fn positive_subfloor_clock_transfer_is_parked_then_accumulates_and_spends_once() {
    let (mut bench, mut stack) = add_water_and_maybe_base(true);
    let clock = GasExchangeClock;
    {
        let vessel = &mut bench.vessels[0];
        assert!(vessel.co2_partial_pressure_atm.is_some());
        clock
            .advance(vessel, 1.0, &ClockContext::default(), &mut Vec::new())
            .expect("first positive subfloor clock transfer");
    }
    let first_pending = bench.vessel(VesselId(0)).unwrap().pending_co2_transfer_mol;
    assert!(
        first_pending > 0.0 && first_pending < CO2_TRANSFER_FLOOR_MOL,
        "one-second transfer must be retained below the spending floor, got {first_pending}"
    );

    let before_parked_solve = bench.vessel(VesselId(0)).unwrap().clone();
    let parked_events = apply(
        &mut bench,
        &mut stack,
        Operator::Wait { seconds: 0.0 },
        "subfloor native characterization",
    );
    assert_no_solver_failure(&parked_events, "subfloor native characterization");
    let parked = bench.vessel(VesselId(0)).unwrap();
    assert_boundary_closed(
        &before_parked_solve,
        parked,
        &parked_events,
        "subfloor native characterization",
    );
    assert!(co2_crossings(&parked_events).is_empty());
    assert_eq!(parked.pending_co2_transfer_mol, first_pending);

    // Accumulate using the existing clock owner while the measured open
    // boundary remains fixed; stop at the existing spending threshold.
    let mut seconds = 0usize;
    while bench.vessel(VesselId(0)).unwrap().pending_co2_transfer_mol <= CO2_TRANSFER_FLOOR_MOL
        && seconds < 20_000
    {
        clock
            .advance(
                &mut bench.vessels[0],
                1.0,
                &ClockContext::default(),
                &mut Vec::new(),
            )
            .expect("subfloor accumulation clock tick");
        seconds += 1;
    }
    let earned = bench.vessel(VesselId(0)).unwrap().pending_co2_transfer_mol;
    assert!(
        earned > CO2_TRANSFER_FLOOR_MOL && seconds < 20_000,
        "positive subfloor parcel did not accumulate across the unchanged floor: {earned}"
    );
    let before_spend = bench.vessel(VesselId(0)).unwrap().clone();
    let spent_events = apply(
        &mut bench,
        &mut stack,
        Operator::Wait { seconds: 0.0 },
        "accumulated clock carrier",
    );
    assert_no_solver_failure(&spent_events, "accumulated clock carrier");
    let after_spend = bench.vessel(VesselId(0)).unwrap();
    assert_current_aqueous(after_spend, "accumulated clock carrier", true);
    assert_boundary_closed(
        &before_spend,
        after_spend,
        &spent_events,
        "accumulated clock carrier",
    );
    assert_eq!(co2_crossings(&spent_events), vec![(true, earned)]);
    assert_eq!(after_spend.pending_co2_transfer_mol, 0.0);
}

#[test]
fn already_earned_subfloor_parcel_survives_sealing_without_further_uptake() {
    let (mut bench, mut stack) = add_water_and_maybe_base(true);
    let clock = GasExchangeClock;
    clock
        .advance(
            &mut bench.vessels[0],
            1.0,
            &ClockContext::default(),
            &mut Vec::new(),
        )
        .expect("earn positive subfloor parcel while open");
    let earned = bench.vessel(VesselId(0)).unwrap().pending_co2_transfer_mol;
    assert!(earned > 0.0 && earned < CO2_TRANSFER_FLOOR_MOL);

    let before_seal = bench.vessel(VesselId(0)).unwrap().clone();
    let seal_events = command(&mut bench, &mut stack, "seal v1 500mL");
    assert_no_solver_failure(&seal_events, "seal with parked CO2 parcel");
    let after_seal = bench.vessel(VesselId(0)).unwrap();
    assert_boundary_closed(
        &before_seal,
        after_seal,
        &seal_events,
        "seal with parked CO2 parcel",
    );
    assert!(co2_crossings(&seal_events).is_empty());
    assert_eq!(after_seal.pending_co2_transfer_mol, earned);

    clock
        .advance(
            &mut bench.vessels[0],
            3600.0,
            &ClockContext::default(),
            &mut Vec::new(),
        )
        .expect("sealed clock interval");
    assert_eq!(
        bench.vessel(VesselId(0)).unwrap().pending_co2_transfer_mol,
        earned,
        "sealed boundary earned additional atmospheric CO2"
    );
}

fn native_carbon_seed() -> Vessel {
    let mut vessel = Vessel::new(VesselId(0), "clock CO2 carrier test");
    vessel.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
    vessel.deposit(SpeciesId::new("Na+"), Moles(0.001), Phase::Aqueous);
    vessel.deposit(SpeciesId::new("HCO3-"), Moles(0.001), Phase::Aqueous);
    vessel.step_start = Some(StepStart::capture(&vessel));
    vessel
}

#[test]
fn valid_negative_carrier_removes_all_native_carbon_and_overdraw_refuses_atomically() {
    let initial = native_carbon_seed();
    let mut characterization = PhreeqcEquilibrator::new().expect("native PHREEQC engine");
    characterization
        .equilibrate(&mut initial.clone())
        .expect("native carbon seed characterization");
    let total_carbon = strict_ledger(&initial, "native carbon seed")
        .elements
        .get("C")
        .copied()
        .expect("inorganic carbon in seed");
    assert!(total_carbon > CO2_TRANSFER_FLOOR_MOL);

    let mut valid = initial.clone();
    valid.step_start = Some(StepStart::capture(&valid));
    valid.pending_co2_transfer_mol = -total_carbon;
    let before_valid = valid.clone();
    let mut solver = PhreeqcEquilibrator::new().expect("native PHREEQC engine");
    let events = solver
        .equilibrate(&mut valid)
        .expect("exact removal of all native inorganic carbon is a valid boundary");
    assert_current_aqueous(&valid, "exact negative carbon boundary", true);
    assert_boundary_closed(
        &before_valid,
        &valid,
        &events,
        "exact negative carbon boundary",
    );
    assert_eq!(co2_crossings(&events), vec![(false, total_carbon)]);
    assert_eq!(valid.pending_co2_transfer_mol, 0.0);

    let mut overdraw = initial;
    overdraw.step_start = Some(StepStart::capture(&overdraw));
    overdraw.pending_co2_transfer_mol = -(total_carbon + CO2_TRANSFER_FLOOR_MOL);
    let before_overdraw = serde_json::to_value(&overdraw).expect("serialize refusal snapshot");
    let mut refusing_solver = PhreeqcEquilibrator::new().expect("native PHREEQC engine");
    assert!(
        refusing_solver.equilibrate(&mut overdraw).is_err(),
        "negative clock carrier beyond all available native C must refuse"
    );
    assert_eq!(
        serde_json::to_value(&overdraw).expect("serialize after refusal"),
        before_overdraw,
        "overdraw refusal changed physical state or discarded the pending obligation"
    );
}

fn mix_solution(id: usize, solutes: &[(&str, f64)]) -> Vessel {
    let mut vessel = Vessel::new(VesselId(id), "clock boundary MIX guard");
    vessel.thermal_mode = ThermalMode::Thermostatted(Kelvin::STANDARD);
    vessel.deposit(SpeciesId::new("water"), Moles(2.775), Phase::Liquid);
    for (species_id, amount) in solutes {
        vessel.deposit(SpeciesId::new(species_id), Moles(*amount), Phase::Aqueous);
    }
    vessel
}

fn mix_target(a: &Vessel, fraction_a: f64, b: &Vessel, fraction_b: f64) -> Vessel {
    let mut target = Vessel::new(VesselId(2), "clock boundary MIX target");
    target.thermal_mode = ThermalMode::Thermostatted(Kelvin::STANDARD);
    for (source, fraction) in [(a, fraction_a), (b, fraction_b)] {
        for portion in &source.contents {
            if let Some(existing) = target
                .contents
                .iter_mut()
                .find(|row| row.species == portion.species && row.phase == portion.phase)
            {
                existing.moles.0 += portion.moles.0 * fraction;
            } else {
                let mut row = portion.clone();
                row.moles.0 *= fraction;
                target.contents.push(row);
            }
        }
    }
    target
}

#[test]
fn native_mix_declines_a_pending_carrier_in_either_source_or_target() {
    let a = mix_solution(0, &[("Na+", 0.001), ("Cl-", 0.001)]);
    let b = mix_solution(1, &[("K+", 0.002), ("Cl-", 0.002)]);
    let clean_target = mix_target(&a, 1.0, &b, 0.5);

    // Prove this is a normally eligible native MIX shape before exercising
    // the pending-owner guard; a blanket unsupported-MIX path is not enough.
    let mut baseline_solver = PhreeqcEquilibrator::new().expect("native PHREEQC engine");
    let mut baseline_target = clean_target.clone();
    baseline_solver
        .mix(&mut baseline_target, &a, 1.0, &b, 0.5)
        .expect("unperturbed aqueous MIX should remain supported")
        .expect("unperturbed native MIX should succeed");
    assert_current_aqueous(&baseline_target, "unperturbed native MIX", true);

    for position in 0..3 {
        let mut source_a = a.clone();
        let mut source_b = b.clone();
        let mut target = clean_target.clone();
        match position {
            0 => source_a.pending_co2_transfer_mol = CO2_TRANSFER_FLOOR_MOL * 0.5,
            1 => source_b.pending_co2_transfer_mol = CO2_TRANSFER_FLOOR_MOL * 0.5,
            _ => target.pending_co2_transfer_mol = CO2_TRANSFER_FLOOR_MOL * 0.5,
        }
        let before_target = serde_json::to_value(&target).expect("target snapshot");
        let mut solver = PhreeqcEquilibrator::new().expect("native PHREEQC engine");
        assert!(
            solver
                .mix(&mut target, &source_a, 1.0, &source_b, 0.5)
                .is_none(),
            "MIX bypassed the clock carrier in vessel role {position}"
        );
        assert_eq!(solver.engine_calls(), 0, "declined MIX called PHREEQC");
        assert_eq!(
            serde_json::to_value(&target).expect("target after declined MIX"),
            before_target,
            "declined MIX mutated target for vessel role {position}"
        );
        let retained = match position {
            0 => source_a.pending_co2_transfer_mol,
            1 => source_b.pending_co2_transfer_mol,
            _ => target.pending_co2_transfer_mol,
        };
        assert_eq!(retained, CO2_TRANSFER_FLOOR_MOL * 0.5);
    }
}

#[test]
fn cached_and_fresh_native_adapters_spend_the_same_clock_carrier_once() {
    let mut bench = Bench::new();
    let mut stack = production_stack();
    command(&mut bench, &mut stack, "add v1 water 100mL");
    command(&mut bench, &mut stack, "add v1 NaOH 0.001mol");
    let mut initial = bench
        .vessel(VesselId(0))
        .expect("native solution seed")
        .clone();
    // This direct adapter call is a new operation after setup; do not reuse
    // the Bench snapshot captured for the preceding NaOH addition.
    initial.step_start = Some(StepStart::capture(&initial));
    let clock = GasExchangeClock;
    clock
        .advance(
            &mut initial,
            3600.0,
            &ClockContext::default(),
            &mut Vec::new(),
        )
        .expect("known open boundary earns CO2");
    let pending = initial.pending_co2_transfer_mol;
    assert!(pending > CO2_TRANSFER_FLOOR_MOL);
    assert!(
        !initial.contents.iter().any(|portion| {
            portion.species.0 == "CO2" && portion.phase == Phase::Gas && portion.moles.0 > 0.0
        }),
        "cache parity first-inlet case must not depend on CO2(g) phase inventory"
    );

    let mut cached_solver = PhreeqcEquilibrator::new().expect("cached native adapter");
    let mut first = initial.clone();
    let first_events = cached_solver
        .equilibrate(&mut first)
        .expect("first carrier spend");
    let calls_after_first = cached_solver.engine_calls();
    assert!(
        calls_after_first > 0,
        "first carrier answer did not call PHREEQC"
    );
    assert_current_aqueous(&first, "first carrier spend", true);
    assert_boundary_closed(&initial, &first, &first_events, "first carrier spend");
    assert_eq!(co2_crossings(&first_events), vec![(true, pending)]);

    let hits_before_repeat = cached_solver.cache_hits();
    let mut repeated = initial.clone();
    let repeated_events = cached_solver
        .equilibrate(&mut repeated)
        .expect("cached carrier spend");
    assert_eq!(cached_solver.engine_calls(), calls_after_first);
    assert!(cached_solver.cache_hits() > hits_before_repeat);
    assert_current_aqueous(&repeated, "cached carrier spend", true);
    assert_boundary_closed(
        &initial,
        &repeated,
        &repeated_events,
        "cached carrier spend",
    );

    let mut fresh_solver = PhreeqcEquilibrator::new().expect("fresh native adapter");
    let mut fresh = initial.clone();
    let fresh_events = fresh_solver
        .equilibrate(&mut fresh)
        .expect("fresh carrier spend");
    assert!(fresh_solver.engine_calls() > 0);
    assert_current_aqueous(&fresh, "fresh carrier spend", true);
    assert_boundary_closed(&initial, &fresh, &fresh_events, "fresh carrier spend");

    assert_eq!(
        repeated_events, fresh_events,
        "cached/fresh carrier event differs"
    );
    assert_eq!(format!("{repeated:?}"), format!("{fresh:?}"));
    assert_eq!(repeated.pending_co2_transfer_mol, 0.0);
    assert_eq!(fresh.pending_co2_transfer_mol, 0.0);
}
