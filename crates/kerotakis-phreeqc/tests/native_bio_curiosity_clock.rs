//! Native regression for the three frozen bio curiosity clock refusals.
//! These controls establish a represented aqueous answer beside immutable
//! recipe/catalyst stock. They do not claim cabbage fermentation or pepsin's
//! molecular composition; those remain separate scientific coverage limits.
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
    let op = parse_op(text)
        .unwrap_or_else(|error| panic!("{text}: {error}"))
        .unwrap_or_else(|| panic!("{text}: no operation"));
    let events = bench
        .step_with(op, stack, &PermissiveScreen)
        .unwrap_or_else(|error| panic!("{text}: {error:?}"));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::SolverFailed { .. })),
        "{text}: {events:?}"
    );
    events
}

fn known_ledger(vessel: &Vessel) -> ConservedLedger {
    let mut ledger = ConservedLedger::from_vessel(vessel);
    for portion in &vessel.contents {
        assert!(portion.moles.0.is_finite() && portion.moles.0 >= 0.0);
        if portion.species.0 == "pepsin" {
            // The catalyst catalogue uses `C` solely as a placeholder. The
            // unchanged dose is checked below; it is not known carbon stock.
            assert_eq!(portion.phase, Phase::Aqueous);
            *ledger.elements.entry("C".into()).or_default() -= portion.moles.0;
        } else {
            let data = species::lookup(&portion.species).expect("known represented formula");
            stoich::parse_formula(data.formula).expect("parse represented formula");
        }
    }
    ledger
}

fn immutable_spectators(vessel: &Vessel) -> Vec<(String, Phase, u64)> {
    let mut rows = vessel
        .contents
        .iter()
        .filter(|portion| {
            matches!(
                portion.species.0.as_str(),
                "pepsin" | "glucose" | "fructose" | "cellulose"
            )
        })
        .map(|portion| {
            (
                portion.species.0.clone(),
                portion.phase,
                portion.moles.0.to_bits(),
            )
        })
        .collect::<Vec<_>>();
    rows.sort();
    rows
}

fn assert_clock_boundary(before: &Vessel, after: &Vessel, events: &[Event]) {
    assert_eq!(before.unresolved_materials, after.unresolved_materials);
    assert_eq!(immutable_spectators(before), immutable_spectators(after));
    let mut expected = known_ledger(before);
    let actual = known_ledger(after);
    let mut absorbed_co2 = 0.0;
    for event in events {
        let (species, moles, sign, owner) = match event {
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
        assert!(moles.is_finite() && moles >= 0.0);
        if sign > 0.0 && species.0 == "CO2" {
            absorbed_co2 += moles;
        }
        let formula = stoich::parse_formula(species::lookup(species).expect("gas formula").formula)
            .expect("parsed gas formula");
        for (element, count) in formula.counts {
            *expected.elements.entry(element).or_default() += sign * moles * count;
        }
        expected.charge += sign * moles * formula.charge;
    }
    assert!(
        absorbed_co2 > 0.0,
        "the actual open-room CO2 parcel must be owned once"
    );
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
            left.is_finite() && right.is_finite() && (right - left).abs() <= 1e-10,
            "{key}: {left:.15e} -> {right:.15e}"
        );
    }
    assert!((expected.charge - actual.charge).abs() <= 1e-10);
    assert_eq!(after.pending_co2_transfer_mol, 0.0);
    let info = after.solution.as_ref().expect("current native solution");
    assert_eq!(info.scope, vessel::SolutionScope::Complete);
    assert!(info.ph.is_finite() && info.ionic_strength.is_finite());
    assert!(info
        .particle_molality
        .is_some_and(|value| value.is_finite() && value >= 0.0));
}

#[test]
fn bio_050_basic_pepsin_keeps_opaque_catalyst_and_clock_carbon_separate() {
    let mut bench = Bench::new();
    let mut solver = stack();
    for command in [
        "add v1 protein 1g",
        "add v1 water 100mL",
        "add v1 NaOH 0.01mol",
        "add v1 pepsin 0.001mol",
    ] {
        step(&mut bench, &mut solver, command);
    }
    let before = bench.vessel(VesselId(0)).expect("protein beaker").clone();
    assert!(!before.unresolved_materials.is_empty());
    assert!(immutable_spectators(&before)
        .iter()
        .any(|(key, _, _)| key == "pepsin"));
    let events = step(&mut bench, &mut solver, "wait 1h");
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::EnzymeHydrolysed { .. })),
        "pepsin in base cannot be presented as active protein hydrolysis"
    );
    let after = bench.vessel(VesselId(0)).expect("after wait");
    assert_clock_boundary(&before, after, &events);
    assert!(after.solution.as_ref().expect("native pH").ph > 7.0);
    step(&mut bench, &mut solver, "look v1");
}

fn cabbage_case(salt: &str) {
    let mut bench = Bench::new();
    let mut solver = stack();
    step(&mut bench, &mut solver, "add v1 cabbage 100g");
    step(&mut bench, &mut solver, salt);
    let before = bench.vessel(VesselId(0)).expect("cabbage beaker").clone();
    assert!(!before.unresolved_materials.is_empty());
    for name in ["glucose", "fructose", "cellulose"] {
        assert!(
            immutable_spectators(&before)
                .iter()
                .any(|(key, _, _)| key == name),
            "the reviewed cabbage recipe must retain {name}"
        );
    }
    let events = step(&mut bench, &mut solver, "wait 24h");
    // This source has no cabbage FermentationCulture role. The numerical
    // clock repair must not fabricate a fermentation result or salt effect.
    assert!(!events
        .iter()
        .any(|event| matches!(event, Event::Fermented { .. })));
    let after = bench.vessel(VesselId(0)).expect("after wait");
    assert_clock_boundary(&before, after, &events);
    step(&mut bench, &mut solver, "measure v1 ph");
}

#[test]
fn bio_072_cabbage_two_grams_salt_retains_carbohydrates() {
    cabbage_case("add v1 NaCl 2g");
}

#[test]
fn bio_074_cabbage_ten_grams_salt_retains_carbohydrates() {
    cabbage_case("add v1 NaCl 10g");
}

#[test]
fn clock_owner_refuses_unmapped_ion_acid_unknown_stock_and_finite_gas_dose_atomically() {
    let mut bench = Bench::new();
    let mut solver = stack();
    for command in [
        "add v1 protein 1g",
        "add v1 water 100mL",
        "add v1 NaOH 0.01mol",
        "add v1 pepsin 0.001mol",
    ] {
        step(&mut bench, &mut solver, command);
    }
    let seed = bench
        .vessel(VesselId(0))
        .expect("valid protein beaker")
        .clone();
    for (species, phase) in [
        ("KIO3", Phase::Aqueous),
        ("malic_acid", Phase::Aqueous),
        ("unknown_clock_stock", Phase::Solid),
        ("CO2", Phase::Gas), // a finite gas Dose is not the room's clock
    ] {
        let mut vessel = seed.clone();
        vessel.deposit(SpeciesId::new(species), Moles(0.001), phase);
        vessel.pending_co2_transfer_mol = 2.0 * CO2_TRANSFER_FLOOR_MOL;
        let before = serde_json::to_value(&vessel).expect("full input snapshot");
        let mut native = PhreeqcEquilibrator::new().expect("native adapter");
        let error = native
            .equilibrate(&mut vessel)
            .expect_err("unsupported clock stock must not receive a certified solution");
        assert!(
            error
                .to_string()
                .contains("clock CO2 parcel lacks a certified native component projection"),
            "{species}: {error}"
        );
        assert_eq!(
            serde_json::to_value(&vessel).unwrap(),
            before,
            "{species}: failed native route must retain complete stock and pending parcel"
        );
    }
}
