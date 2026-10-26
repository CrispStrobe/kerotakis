#![cfg(feature = "engine")]
//! The same contents give the same answer, whatever order they arrived in.
//!
//! `solution.solvent_kg` used to carry a residue of the order the beaker
//! was filled in. `aq-023` — 100 mL of water, 0.01 mol of calcium chloride
//! and 5 g of a carbonate laundry powder — read one solvent mass with the
//! salt poured first and another with the powder first, one part in 1e4
//! apart, about 10 mg of water in 100 g. Every molality is per kg of that
//! mass, so `ionic_strength` carried the same residue into every activity
//! coefficient, every saturation index and the pH.
//!
//! Both numbers came from the same line of the same function. The vessel's
//! own water INVENTORY never departed — `complete_basis` rebuilds it from
//! conserved hydrogen and oxygen, which cannot care about order — but
//! PHREEQC's `mass_H2O` tracks the water the INPUT declared, and the input
//! to the last operation is the intermediate vessel plus one reagent. The
//! two orderings have different intermediate vessels, holding different
//! shares of their hydrogen and oxygen inside species rather than inside
//! water, so the last solve was handed one final state in two
//! representations and answered each faithfully.
//!
//! `PhreeqcEquilibrator::recharacterise_canonically` poses the settled
//! contents again before reporting them. This file is the measurement that
//! says it worked, and the budget that says what it cost.

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;
use std::collections::BTreeSet;

fn stack() -> SolverStack {
    SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
        PhreeqcEquilibrator::new().expect("engine"),
    )]))
}

/// What a reader is shown about this beaker, plus the water the inventory
/// says is in it.
struct Read {
    solvent_kg: f64,
    ionic_strength: f64,
    ph: f64,
    inventory_water_mol: f64,
    known_input: ConservedLedger,
    final_ledger: ConservedLedger,
    unresolved_material_amount_g: f64,
}

/// `aq-023`'s own script, with the two reagents in the stated order.
fn aq_023(second: &str, third: &str) -> Read {
    let mut bench = Bench::new();
    let mut solvers = stack();
    let mut all_events = Vec::new();
    for line in ["add v1 water 100mL", second, third] {
        let op = kerotakis_core::script::parse_op(line)
            .expect("the corpus script parses")
            .expect("a known verb");
        all_events.extend(
            bench
                .step_with(op, &mut solvers, &PermissiveScreen)
                .expect("the step runs"),
        );
    }
    let vessel = bench.vessel(VesselId(0)).expect("the beaker");
    let info = vessel
        .solution
        .clone()
        .expect("a calcium chloride solution is characterised");
    let mut known_input = Vessel::new(VesselId(0), "aq-023 actual named inputs");
    let mut added_species = Vec::new();
    let mut material_additions = 0usize;
    for event in &all_events {
        match event {
            Event::Added {
                vessel: owner,
                species,
                moles,
                ..
            } if *owner == vessel.id => {
                let phase = if species.0 == "water" {
                    Phase::Liquid
                } else {
                    Phase::Solid
                };
                known_input.deposit(species.clone(), *moles, phase);
                added_species.push(species.0.clone());
            }
            Event::MaterialAdded {
                vessel: owner,
                material,
                recipe_id,
                recipe_version,
                total_amount,
                basis,
                components,
                unresolved_amount,
                ..
            } if *owner == vessel.id => {
                material_additions += 1;
                assert_eq!(material, "laundry_detergent");
                assert_eq!(recipe_id, "household/laundry-detergent-powder");
                assert_eq!(*recipe_version, 1);
                assert_eq!(*total_amount, 5.0);
                assert_eq!(
                    *basis,
                    kerotakis_core::material::MaterialBasis::MassFraction
                );
                assert_eq!(*unresolved_amount, 3.5);
                assert_eq!(components.len(), 1);
                assert_eq!(components[0].species.0, "Na2CO3");
                assert_eq!(components[0].basis_amount, 1.5);
                for component in components {
                    known_input.deposit(component.species.clone(), component.moles, Phase::Solid);
                }
            }
            _ => {}
        }
    }
    added_species.sort();
    assert_eq!(added_species, ["CaCl2", "water"]);
    assert_eq!(material_additions, 1);
    assert_eq!(vessel.unresolved_materials.len(), 1);
    let opaque = &vessel.unresolved_materials[0];
    assert_eq!(opaque.material, "laundry_detergent");
    assert_eq!(opaque.recipe_id, "household/laundry-detergent-powder");
    assert_eq!(opaque.recipe_version, 1);
    assert_eq!(
        opaque.basis,
        kerotakis_core::material::MaterialBasis::MassFraction
    );
    assert_eq!(
        opaque.amount, 3.5,
        "opaque recipe balance changed in the vessel"
    );

    // The known input ledger is sourced from the actual Added and
    // MaterialAdded event payloads. Only named gas-boundary crossings alter
    // it; the unresolved 3.5 g is checked separately and never given an
    // invented formula.
    let mut known_input = ConservedLedger::from_vessel(&known_input);
    for event in &all_events {
        let (owner, species, moles, sign) = match event {
            Event::GasAbsorbed {
                vessel,
                species,
                moles,
            } => (*vessel, species, moles.0, 1.0),
            Event::GasEvolved {
                vessel,
                species,
                moles,
            } => (*vessel, species, moles.0, -1.0),
            _ => continue,
        };
        if owner != vessel.id {
            continue;
        }
        assert!(moles.is_finite() && moles >= 0.0);
        let data = species::lookup(species).expect("gas boundary has a registered species");
        let formula = stoich::parse_formula(data.formula).expect("gas formula parses");
        for (element, count) in formula.counts {
            *known_input.elements.entry(element).or_default() += sign * count * moles;
        }
        known_input.charge += sign * formula.charge * moles;
    }
    let final_ledger = ConservedLedger::from_vessel(vessel);
    let reading = Read {
        solvent_kg: info
            .solvent_kg
            .expect("a characterised solution has a solvent mass"),
        ionic_strength: info.ionic_strength,
        ph: info.ph,
        inventory_water_mol: vessel
            .contents
            .iter()
            .filter(|p| p.species.0 == "water" && p.phase == Phase::Liquid)
            .map(|p| p.moles.0)
            .sum(),
        known_input,
        final_ledger,
        unresolved_material_amount_g: opaque.amount,
    };
    let before_wait = vessel.clone();
    assert_eq!(info.scope, SolutionScope::Complete);
    let previous_secondary = info
        .solvent_activity
        .as_ref()
        .expect("the supported final mixture has native solvent diagnostics");
    assert!(previous_secondary.dataset.starts_with("pitzer.dat"));
    assert!(previous_secondary.model.contains("Pitzer"));
    // This is a new native fixed-point control, not a calibrated value.
    // Inspect the actually accepted diagnostics again after typed zero time;
    // no authored CLI command, reagent or original comparison is changed.
    let wait_events = bench
        .step_with(
            Operator::Wait { seconds: 0.0 },
            &mut solvers,
            &PermissiveScreen,
        )
        .expect("typed zero-time native re-equilibration");
    assert!(!wait_events
        .iter()
        .any(|event| matches!(event, Event::SolverFailed { .. })));
    let after_wait = bench.vessel(VesselId(0)).expect("the repeated beaker");
    let repeated_info = after_wait.solution.as_ref().expect("the repeated solution");
    assert_eq!(repeated_info.scope, SolutionScope::Complete);
    let repeated_secondary = repeated_info
        .solvent_activity
        .as_ref()
        .expect("zero time retains the actual native solvent answer");
    assert_eq!(previous_secondary.dataset, repeated_secondary.dataset);
    assert_eq!(previous_secondary.model, repeated_secondary.model);
    for (coordinate, before, after) in [
        (
            "water activity",
            previous_secondary.water_activity,
            repeated_secondary.water_activity,
        ),
        (
            "particle molality",
            previous_secondary.particle_molality,
            repeated_secondary.particle_molality,
        ),
        (
            "ionic strength",
            previous_secondary.ionic_strength,
            repeated_secondary.ionic_strength,
        ),
    ] {
        assert!(
            before.is_finite() && after.is_finite() && (after - before).abs() <= 1e-10,
            "native auxiliary {coordinate} changed at zero time: {before:.15e} -> {after:.15e}"
        );
    }
    assert_eq!(
        before_wait.unresolved_materials,
        after_wait.unresolved_materials
    );
    assert!((before_wait.temperature.0 - after_wait.temperature.0).abs() <= 1e-6);
    let before_ledger = ConservedLedger::from_vessel(&before_wait);
    let after_ledger = ConservedLedger::from_vessel(after_wait);
    let elements: BTreeSet<_> = before_ledger
        .elements
        .keys()
        .chain(after_ledger.elements.keys())
        .collect();
    for element in elements {
        let before = before_ledger.elements.get(element).copied().unwrap_or(0.0);
        let after = after_ledger.elements.get(element).copied().unwrap_or(0.0);
        assert!(
            before.is_finite() && after.is_finite() && (after - before).abs() <= 1e-10,
            "{element} ledger changed during zero-time auxiliary control"
        );
    }
    assert!(
        before_ledger.charge.is_finite()
            && after_ledger.charge.is_finite()
            && (after_ledger.charge - before_ledger.charge).abs() <= 1e-10
    );
    reading
}

const SALT_FIRST: (&str, &str) = ("add v1 CaCl2 0.01mol", "add v1 laundry_detergent 5g");
const POWDER_FIRST: (&str, &str) = ("add v1 laundry_detergent 5g", "add v1 CaCl2 0.01mol");

/// Compare the solvent mass against the water amount each actual ordering
/// produced. The old raw-mass equality depended on the obsolete nine-decimal
/// PHREEQC input-text format: full-precision input preserves each distinct
/// water target, so equal raw kg is not the invariant. The per-side physical
/// conversion remains strict, and the two normalized readings retain the
/// original relative bound.
#[test]
fn the_same_beaker_reads_the_same_solvent_mass_in_either_order() {
    let salt = aq_023(SALT_FIRST.0, SALT_FIRST.1);
    let powder = aq_023(POWDER_FIRST.0, POWDER_FIRST.1);

    let report = format!(
        "\n              solvent_kg     ionic strength   pH             contents[water]\n\
         salt first    {:.10}   {:.10}     {:.10}   {:.10}\n\
         powder first  {:.10}   {:.10}     {:.10}   {:.10}",
        salt.solvent_kg,
        salt.ionic_strength,
        salt.ph,
        salt.inventory_water_mol,
        powder.solvent_kg,
        powder.ionic_strength,
        powder.ph,
        powder.inventory_water_mol,
    );

    let relative = |a: f64, b: f64| (a - b).abs() / a.abs().max(b.abs());
    let water_molar_mass = kerotakis_core::constants::WATER_MOLAR_MASS_KG_PER_MOL;
    for (order, reading) in [("salt first", &salt), ("powder first", &powder)] {
        let expected_solvent_kg = reading.inventory_water_mol * water_molar_mass;
        assert!(
            expected_solvent_kg.is_finite()
                && relative(reading.solvent_kg, expected_solvent_kg) < 1e-9,
            "{order}: solvent mass does not match the actual water inventory:{report}"
        );
    }
    assert!(
        relative(
            salt.solvent_kg / salt.inventory_water_mol,
            powder.solvent_kg / powder.inventory_water_mol,
        ) < 1e-9,
        "solvent mass per actual water mole depends on order:{report}"
    );
    assert!(
        relative(salt.ionic_strength, powder.ionic_strength) < 1e-7,
        "the ionic strength still depends on the order the beaker was filled in:{report}"
    );
    assert!(
        (salt.ph - powder.ph).abs() < 1e-7,
        "the pH still depends on the order the beaker was filled in:{report}"
    );
    // The inventory never departed, and saying so here is what keeps the
    // two surfaces from being confused for one another: the fix did not
    // move `contents[water]` towards `solvent_kg`, it moved the question
    // the solver is asked.
    assert!(
        relative(salt.inventory_water_mol, powder.inventory_water_mol) < 1e-7,
        "the conserved water inventory departed, which is a different \
         defect from the one this file is about:{report}"
    );
    for (order, reading) in [("salt first", &salt), ("powder first", &powder)] {
        let elements: BTreeSet<_> = reading
            .known_input
            .elements
            .keys()
            .chain(reading.final_ledger.elements.keys())
            .cloned()
            .collect();
        for element in elements {
            let input = reading
                .known_input
                .elements
                .get(&element)
                .copied()
                .unwrap_or(0.0);
            let final_amount = reading
                .final_ledger
                .elements
                .get(&element)
                .copied()
                .unwrap_or(0.0);
            assert!(
                input.is_finite()
                    && final_amount.is_finite()
                    && (input - final_amount).abs() <= 1e-10,
                "{order}: {element} ledger changed {input:.15e} -> {final_amount:.15e}{report}"
            );
        }
        assert!(
            reading.known_input.charge.is_finite()
                && reading.final_ledger.charge.is_finite()
                && (reading.known_input.charge - reading.final_ledger.charge).abs() <= 1e-10,
            "{order}: charge ledger changed {:.15e} -> {:.15e}{report}",
            reading.known_input.charge,
            reading.final_ledger.charge
        );
        assert_eq!(reading.unresolved_material_amount_g, 3.5);
    }
}

/// What the canonical pose costs, held as a budget.
///
/// One re-pose per equilibration, and never more. The opening water is
/// solvent-only — there is no engine call behind that answer to repeat —
/// so three commands buy at most two. Measured when this landed: the
/// salt-first ordering went from 8 engine calls to 9 and the powder-first
/// from 6 to 8, one of the salt-first re-poses having been answered from
/// the content-addressed cache instead of the engine.
#[test]
fn the_canonical_pose_costs_at_most_one_solve_per_equilibration() {
    let mut phreeqc = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    for line in ["add v1 water 100mL", SALT_FIRST.0, SALT_FIRST.1] {
        let op = kerotakis_core::script::parse_op(line)
            .expect("the corpus script parses")
            .expect("a known verb");
        bench
            .step_with(op, &mut phreeqc, &PermissiveScreen)
            .expect("the step runs");
    }
    let calls = phreeqc.engine_calls();
    assert!(
        calls <= 12,
        "aq-023 cost {calls} engine calls; it cost 8 before the canonical \
         pose and 9 after, and the budget is there so a second re-pose \
         cannot be added without someone noticing"
    );
}
