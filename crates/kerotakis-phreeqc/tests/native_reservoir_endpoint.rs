//! Exhausted reservoir readback must close real gas and component stock jointly.
#![cfg(feature = "engine")]

use kerotakis_core::ledger::ConservedLedger;
use kerotakis_core::script::parse_op;
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;
use std::collections::BTreeSet;

fn step(bench: &mut Bench, solver: &mut SolverStack, op: Operator) {
    let before = bench.vessels.first().expect("vessel");
    let before_id = before.id;
    assert!(before.unresolved_materials.is_empty());
    for portion in &before.contents {
        assert!(portion.moles.0.is_finite() && portion.moles.0 >= 0.0);
        let data = species::lookup(&portion.species).expect("known input stock");
        stoich::parse_formula(data.formula).expect("input stock formula");
    }
    let mut expected = ConservedLedger::from_vessel(before);
    let finite_co2_dose = match &op {
        Operator::Add { species, moles, .. } if species.0 == "CO2" => Some(moles.0),
        _ => None,
    };
    if let Operator::Add { species, moles, .. } = &op {
        let formula = stoich::parse_formula(species::lookup(species).expect("input").formula)
            .expect("formula");
        for (element, count) in formula.counts {
            *expected.elements.entry(element).or_default() += count * moles.0;
        }
        expected.charge += formula.charge * moles.0;
    }
    let events = bench
        .step_with(op.clone(), solver, &PermissiveScreen)
        .expect("step");
    let mut absorbed_from_dose = 0.0;
    let mut vented_from_dose = 0.0;
    for event in &events {
        assert!(
            !matches!(event, Event::SolverFailed { .. }),
            "{op:?}: {events:?}"
        );
        let (vessel, species, moles, sign) = match event {
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
        assert_eq!(
            *vessel, before_id,
            "gas crossing belongs to the tested vessel"
        );
        assert!(moles.is_finite() && moles >= 0.0);
        let amount = sign * moles;
        let formula = stoich::parse_formula(species::lookup(species).expect("gas").formula)
            .expect("gas formula");
        if sign > 0.0 && species.0 == "CO2" && finite_co2_dose.is_some() {
            // The full Add was already booked above. This event tells how
            // much of that finite gas dose entered solution; it is not a
            // second room-air inlet. Unused dose still leaves as GasEvolved.
            absorbed_from_dose += moles;
            continue;
        }
        if sign < 0.0 && species.0 == "CO2" && finite_co2_dose.is_some() {
            vented_from_dose += moles;
        }
        for (element, count) in formula.counts {
            *expected.elements.entry(element).or_default() += count * amount;
        }
        expected.charge += formula.charge * amount;
    }
    if let Some(dose) = finite_co2_dose {
        assert!(dose.is_finite() && dose >= 0.0);
        assert!(
            absorbed_from_dose <= dose,
            "finite CO2 uptake {absorbed_from_dose} exceeds supplied dose {dose}"
        );
        assert!(
            (absorbed_from_dose + vented_from_dose - dose).abs() <= 1e-10,
            "finite CO2 dose must be accounted for by retained uptake and venting"
        );
    }
    let after = bench.vessels.first().expect("result vessel");
    assert!(after.unresolved_materials.is_empty());
    let actual = ConservedLedger::from_vessel(after);
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
            "{op:?}: {element}: {want:.16e} -> {got:.16e}"
        );
    }
    assert!(
        (actual.charge - expected.charge).abs() <= 1e-10,
        "{op:?}: charge"
    );
    assert!(after.temperature.0.is_finite());
    let solution = after.solution.as_ref().expect("current aqueous answer");
    assert!(solution.ph.is_finite() && solution.ionic_strength.is_finite());
    for portion in &after.contents {
        assert!(portion.moles.0.is_finite() && portion.moles.0 >= 0.0);
        let data = species::lookup(&portion.species).expect("known output stock");
        stoich::parse_formula(data.formula).expect("output stock formula");
    }
}

#[test]
fn aq064_original_and_nearby_co2_exhaustion_preserve_material_and_charge() {
    for dose in ["0.01mol", "0.001mol", "0.02mol"] {
        let mut bench = Bench::new();
        let mut solver = kerotakis_stack::standard_stack(vec![Box::new(
            PhreeqcEquilibrator::new().expect("engine"),
        )]);
        for command in [
            "add v1 water 100mL".to_string(),
            format!("add v1 CO2 {dose}"),
            "sweep v1 1bar".to_string(),
        ] {
            step(
                &mut bench,
                &mut solver,
                parse_op(&command).expect("syntax").expect("operator"),
            );
        }
        let before = bench.vessels[0].clone();
        step(&mut bench, &mut solver, Operator::Wait { seconds: 0.0 });
        let after = &bench.vessels[0];
        assert!((after.temperature.0 - before.temperature.0).abs() <= 1e-6);
        let keys: BTreeSet<_> = before
            .contents
            .iter()
            .chain(&after.contents)
            .map(|p| (p.species.clone(), p.phase))
            .collect();
        for (species, phase) in keys {
            let amount = |v: &Vessel| {
                v.contents
                    .iter()
                    .filter(|p| p.species == species && p.phase == phase)
                    .map(|p| p.moles.0)
                    .sum::<f64>()
            };
            assert!(
                (amount(after) - amount(&before)).abs() <= 1e-10,
                "zero-time {species:?}/{phase:?}"
            );
        }
    }
}
