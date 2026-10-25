//! Native control for the original v4 soda/indicator mixture
//! followed by a typed zero-time native re-equilibration.
//!
//! This tests conservation and the *installed* energy model. The standard
//! dissolution reference is not a finite-concentration calorimetry claim.
#![cfg(feature = "engine")]

use std::collections::{BTreeMap, BTreeSet};

use kerotakis_core::ledger::ConservedLedger;
use kerotakis_core::material::MaterialBasis;
use kerotakis_core::ops::MaterialComponentAdded;
use kerotakis_core::script::parse_op;
use kerotakis_core::*;
use kerotakis_phreeqc::enthalpy::heat_released_j;
use kerotakis_phreeqc::PhreeqcEquilibrator;

const TARGET: VesselId = VesselId(3);
const ATOMS_AND_CHARGE_BUDGET: f64 = 1e-10;
const PH_AND_T_BUDGET: f64 = 1e-6;
const IONIC_BUDGET: f64 = 1e-10;
const PHASE_HEAT_BUDGET_J: f64 = 0.001;

fn original_host_stack() -> SolverStack {
    // The CLI's real tail is phase -> displacement -> IPhreeqc.
    kerotakis_stack::standard_stack(vec![Box::new(PhaseEquilibrator::wrapping(Box::new(
        DisplacementEquilibrator::wrapping(Box::new(
            PhreeqcEquilibrator::new().expect("native PHREEQC"),
        )),
    )))])
}

fn run(bench: &mut Bench, stack: &mut SolverStack, command: &str) -> Vec<Event> {
    let operator = parse_op(command)
        .unwrap_or_else(|error| panic!("{command}: {error}"))
        .unwrap_or_else(|| panic!("{command}: expected an operator"));
    let events = bench
        .step_with(operator, stack, &PermissiveScreen)
        .unwrap_or_else(|error| panic!("{command}: {error:?}"));
    assert_no_failed_or_gas(&events, command);
    events
}

fn assert_no_failed_or_gas(events: &[Event], context: &str) {
    assert!(
        !events.iter().any(|event| matches!(
            event,
            Event::SolverFailed { .. }
                | Event::GasEvolved { .. }
                | Event::GasAbsorbed { .. }
                | Event::GasContained { .. }
        )),
        "{context}: unexpected failure or gas crossing: {events:?}"
    );
}

fn portion_amount(vessel: &Vessel, species: &str, phase: Phase) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|portion| portion.species.0 == species && portion.phase == phase)
        .map(|portion| portion.moles.0)
        .sum()
}

/// The public ledger counts represented portions including the analytical
/// base-equivalents OH-formula carrier. Measured free H+/OH- are diagnostics,
/// so adding them again here would double-count the conserved H/O/charge.
fn physical_ledger(vessel: &Vessel) -> ConservedLedger {
    assert!(vessel.surfaces.is_empty() && vessel.exchanges.is_empty());
    assert!(vessel.electrodes.is_empty() && vessel.material_objects.is_empty());
    assert!(vessel.adsorbed.is_empty() && vessel.solid_solutions.is_empty());
    assert!(
        vessel.free_proton.is_finite()
            && vessel.free_proton >= 0.0
            && vessel.free_hydroxide.is_finite()
            && vessel.free_hydroxide >= 0.0
            && vessel.solute_charge.is_finite()
    );
    for portion in &vessel.contents {
        assert!(portion.moles.0.is_finite() && portion.moles.0 >= 0.0);
        if portion.species.0 == species::BASE_EQUIVALENTS {
            assert_eq!(portion.phase, Phase::Aqueous);
            continue;
        }
        let data = species::lookup(&portion.species)
            .unwrap_or_else(|| panic!("unregistered physical portion: {}", portion.species.0));
        stoich::parse_formula(data.formula).expect("registered physical formula");
    }
    let represented_charge: f64 = vessel
        .contents
        .iter()
        .filter(|portion| {
            portion.phase == Phase::Aqueous && !species::is_acid_base_basis(&portion.species.0)
        })
        .map(|portion| {
            let data = species::lookup(&portion.species).expect("registered aqueous portion");
            stoich::parse_formula(data.formula)
                .expect("registered aqueous formula")
                .charge
                * portion.moles.0
        })
        .sum();
    assert!((represented_charge - vessel.solute_charge).abs() <= ATOMS_AND_CHARGE_BUDGET);
    ConservedLedger::from_vessel(vessel)
}

fn add_inlet(ledger: &mut ConservedLedger, species: &SpeciesId, moles: f64) {
    assert!(moles.is_finite() && moles >= 0.0);
    let data = species::lookup(species).expect("registered inlet");
    let formula = stoich::parse_formula(data.formula).expect("registered inlet formula");
    for (element, count) in formula.counts {
        *ledger.elements.entry(element).or_default() += count * moles;
    }
    ledger.charge += formula.charge * moles;
}

fn assert_ledger_close(expected: &ConservedLedger, actual: &ConservedLedger, context: &str) {
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
            want.is_finite() && got.is_finite() && (got - want).abs() <= ATOMS_AND_CHARGE_BUDGET,
            "{context}: {element} {want:.15e} -> {got:.15e} mol"
        );
    }
    assert!(
        (expected.charge - actual.charge).abs() <= ATOMS_AND_CHARGE_BUDGET,
        "{context}: formal charge {:.15e} -> {:.15e} eq",
        expected.charge,
        actual.charge
    );
}

fn material_receipt<'a>(
    events: &'a [Event],
    recipe_id_expected: &str,
    amount_expected: f64,
    opaque_expected: f64,
) -> &'a [MaterialComponentAdded] {
    let mut receipts = events.iter().filter_map(|event| match event {
        Event::MaterialAdded {
            vessel,
            recipe_id,
            recipe_version,
            total_amount,
            basis,
            sample_seed,
            components,
            unresolved_amount,
            ..
        } => Some((
            vessel,
            recipe_id,
            recipe_version,
            total_amount,
            basis,
            sample_seed,
            components,
            unresolved_amount,
        )),
        _ => None,
    });
    let Some((vessel, recipe_id, version, total, basis, seed, components, opaque)) =
        receipts.next()
    else {
        panic!("missing material receipt: {events:?}");
    };
    assert!(
        receipts.next().is_none(),
        "duplicate material receipt: {events:?}"
    );
    assert_eq!(*vessel, TARGET);
    assert_eq!(recipe_id.as_str(), recipe_id_expected);
    assert_eq!(*version, 1);
    assert_eq!(*basis, MaterialBasis::MassFraction);
    assert_eq!(*seed, 0);
    assert!((*total - amount_expected).abs() <= 1e-12);
    assert!((*opaque - opaque_expected).abs() <= 1e-12);
    components.as_slice()
}

fn assert_components(components: &[MaterialComponentAdded], wanted: &[(&str, f64)]) {
    assert_eq!(components.len(), wanted.len());
    let amounts: BTreeMap<_, _> = components
        .iter()
        .map(|component| {
            let data = species::lookup(&component.species).expect("inlet component registered");
            let want_moles = data.moles_from_grams(Grams(component.basis_amount)).0;
            assert!((component.moles.0 - want_moles).abs() <= 1e-12);
            (component.species.0.as_str(), component.basis_amount)
        })
        .collect();
    assert_eq!(amounts.len(), wanted.len());
    for (species, grams) in wanted {
        let actual = amounts
            .get(*species)
            .copied()
            .expect("missing recipe component");
        assert!((actual - *grams).abs() <= 1e-12);
    }
}

fn assert_current_pitzer(vessel: &Vessel) {
    let solution = vessel.solution.as_ref().expect("native aqueous readback");
    assert_eq!(solution.scope, SolutionScope::Complete);
    assert!(solution.ph.is_finite() && solution.ionic_strength.is_finite());
    assert!(solution.ionic_strength >= 0.0);
    let provenance = solution.provenance.as_ref().expect("native provenance");
    assert!(provenance.engine.contains("IPhreeqc"));
    assert!(provenance.dataset.starts_with("pitzer.dat"));
    assert!(provenance.model.contains("Pitzer"));
}

#[test]
fn original_cabbage_v4_closes_modeled_heat_and_zero_time_native_stock() {
    let mut bench = Bench::new();
    let mut stack = original_host_stack();
    // Bench starts with v1; create the remaining original vessel identities.
    assert_eq!(bench.vessels.len(), 1);
    for _ in 0..3 {
        run(&mut bench, &mut stack, "new");
    }
    assert_eq!(bench.vessels.len(), 4);
    let initial = bench.vessel(TARGET).expect("v4 created");
    assert_eq!(initial.temperature.0, 298.15);
    assert_eq!(initial.thermal_mode, ThermalMode::Adiabatic);

    let water_events = run(&mut bench, &mut stack, "add v4 water 100mL");
    let baseline = bench.vessel(TARGET).expect("water-only baseline").clone();
    let expected_water = species::lookup(&SpeciesId::new("water"))
        .expect("water registry")
        .moles_from_liters(Liters(0.1))
        .0;
    let water_receipts: Vec<_> = water_events
        .iter()
        .filter_map(|event| match event {
            Event::Added {
                vessel,
                species,
                moles,
                ..
            } => Some((vessel, species, moles)),
            _ => None,
        })
        .collect();
    let [(owner, water, moles)] = water_receipts.as_slice() else {
        panic!("original 100 mL water dose missing: {water_events:?}");
    };
    assert_eq!(**owner, TARGET);
    assert_eq!(water.0, "water");
    assert!((moles.0 - expected_water).abs() <= 1e-12);
    assert!((portion_amount(&baseline, "water", Phase::Liquid) - expected_water).abs() <= 1e-12);
    assert_eq!(baseline.temperature.0, 298.15);
    assert!(baseline.unresolved_materials.is_empty());

    let soda_events = run(&mut bench, &mut stack, "add v4 washing_soda 4g");
    let soda = material_receipt(&soda_events, "household/washing-soda", 4.0, 0.0);
    assert_components(soda, &[("Na2CO3", 4.0)]);
    let after_soda = bench.vessel(TARGET).expect("post-soda");
    assert_current_pitzer(after_soda);
    assert!(after_soda.unresolved_materials.is_empty());
    assert!(after_soda.temperature.0.is_finite() && after_soda.temperature.0 >= 252.0);

    let juice_events = run(&mut bench, &mut stack, "add v4 red_cabbage_juice 10mL");
    let juice = material_receipt(
        &juice_events,
        "household/red-cabbage-juice-surrogate",
        10.0,
        0.197,
    );
    assert_components(juice, &[("water", 9.8), ("anthocyanin", 0.003)]);
    let final_state = bench.vessel(TARGET).expect("post-juice").clone();
    assert_current_pitzer(&final_state);
    assert!(final_state.temperature.0.is_finite() && final_state.temperature.0 >= 252.0);
    assert_eq!(final_state.thermal_mode, ThermalMode::Adiabatic);
    assert_eq!(final_state.unresolved_materials.len(), 1);
    let opaque = &final_state.unresolved_materials[0];
    assert_eq!(opaque.recipe_id, "household/red-cabbage-juice-surrogate");
    assert_eq!(opaque.recipe_version, 1);
    assert!((opaque.amount - 0.197).abs() <= 1e-12);
    let pigment = juice
        .iter()
        .find(|component| component.species.0 == "anthocyanin")
        .expect("pigment receipt");
    assert!(
        (portion_amount(&final_state, "anthocyanin", Phase::Aqueous) - pigment.moles.0).abs()
            <= ATOMS_AND_CHARGE_BUDGET
    );

    // The input ledger is the *actual* water-only physical state plus the
    // exact two material receipts. Opaque 0.197 g is held separately; no
    // fictitious formula is assigned to it.
    let mut expected_stock = physical_ledger(&baseline);
    let mut input = baseline.contents.clone();
    for component in soda.iter().chain(juice) {
        add_inlet(&mut expected_stock, &component.species, component.moles.0);
        let data = species::lookup(&component.species).expect("inlet component");
        input.push(Portion {
            species: component.species.clone(),
            moles: component.moles,
            phase: data.standard_phase,
        });
    }
    assert_ledger_close(
        &expected_stock,
        &physical_ledger(&final_state),
        "original v4 input",
    );

    let quote = heat_released_j(
        &input,
        baseline.free_hydroxide,
        &final_state.contents,
        final_state.free_hydroxide,
        &[],
        "pitzer",
    )
    .expect("complete modeled reference for moved native components");
    let sensible = kerotakis_core::solve::portions_enthalpy(
        final_state
            .contents
            .iter()
            .map(|portion| (&portion.species, portion.moles.0, portion.phase)),
        298.15,
        final_state.temperature.0,
    );
    assert!(
        (sensible - quote).abs() <= PHASE_HEAT_BUDGET_J,
        "original v4 modeled sensible {sensible:.9} J vs native references {quote:.9} J"
    );

    let before_wait = final_state;
    let wait_events = bench
        .step_with(
            Operator::Wait { seconds: 0.0 },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("typed zero-time native re-equilibration");
    assert_no_failed_or_gas(&wait_events, "typed zero-time wait");
    assert!(
        !wait_events.iter().any(|event| matches!(
            event,
            Event::Added { .. }
                | Event::MaterialAdded { .. }
                | Event::Dissolved { .. }
                | Event::DissolvedInSolvent { .. }
                | Event::ReactionOccurred { .. }
                | Event::HeatOfMixing { .. }
                | Event::EnergyTransferred { .. }
        )),
        "zero-time wait repeated a material or heat event: {wait_events:?}"
    );
    let repeated = bench.vessel(TARGET).expect("repeated v4");
    assert_current_pitzer(repeated);
    assert_eq!(
        repeated.unresolved_materials,
        before_wait.unresolved_materials
    );
    assert_ledger_close(
        &physical_ledger(&before_wait),
        &physical_ledger(repeated),
        "typed zero-time wait",
    );
    let keys: BTreeSet<_> = before_wait
        .contents
        .iter()
        .chain(&repeated.contents)
        .map(|portion| (portion.species.0.clone(), portion.phase))
        .collect();
    for (species, phase) in keys {
        let delta = portion_amount(repeated, &species, phase)
            - portion_amount(&before_wait, &species, phase);
        assert!(
            delta.abs() <= ATOMS_AND_CHARGE_BUDGET,
            "zero-time changed {species} {phase:?} by {delta:.15e} mol"
        );
    }
    assert!((repeated.temperature.0 - before_wait.temperature.0).abs() <= PH_AND_T_BUDGET);
    let first = before_wait.solution.as_ref().expect("first native state");
    let second = repeated.solution.as_ref().expect("repeated native state");
    assert!((second.ph - first.ph).abs() <= PH_AND_T_BUDGET);
    assert!((second.ionic_strength - first.ionic_strength).abs() <= IONIC_BUDGET);
    let thermal_delta = kerotakis_core::solve::portions_enthalpy(
        repeated
            .contents
            .iter()
            .map(|portion| (&portion.species, portion.moles.0, portion.phase)),
        before_wait.temperature.0,
        repeated.temperature.0,
    );
    assert!(thermal_delta.abs() <= PHASE_HEAT_BUDGET_J);
}

#[test]
fn plain_carbonate_water_closes_modeled_heat_and_zero_time_native_stock() {
    // This companion isolates the represented carbonate solution from the
    // optical reporter and opaque juice. It uses the same source reaction,
    // physical budgets and native stack as the untouched cabbage control.
    let mut bench = Bench::new();
    let mut stack = original_host_stack();
    for _ in 0..3 {
        run(&mut bench, &mut stack, "new");
    }
    let initial = bench.vessel(TARGET).expect("v4 created");
    assert_eq!(initial.temperature.0, 298.15);
    assert_eq!(initial.thermal_mode, ThermalMode::Adiabatic);

    let water_events = run(&mut bench, &mut stack, "add v4 water 100mL");
    let baseline = bench.vessel(TARGET).expect("water-only baseline").clone();
    let expected_water = species::lookup(&SpeciesId::new("water"))
        .expect("water registry")
        .moles_from_liters(Liters(0.1))
        .0;
    let water_receipts: Vec<_> = water_events
        .iter()
        .filter_map(|event| match event {
            Event::Added {
                vessel,
                species,
                moles,
                ..
            } => Some((vessel, species, moles)),
            _ => None,
        })
        .collect();
    let [(owner, water, moles)] = water_receipts.as_slice() else {
        panic!("100 mL water dose missing: {water_events:?}");
    };
    assert_eq!(**owner, TARGET);
    assert_eq!(water.0, "water");
    assert!((moles.0 - expected_water).abs() <= 1e-12);
    assert!((portion_amount(&baseline, "water", Phase::Liquid) - expected_water).abs() <= 1e-12);
    assert_eq!(baseline.temperature.0, 298.15);
    assert!(baseline.unresolved_materials.is_empty());

    let soda_events = run(&mut bench, &mut stack, "add v4 washing_soda 4g");
    let soda = material_receipt(&soda_events, "household/washing-soda", 4.0, 0.0);
    assert_components(soda, &[("Na2CO3", 4.0)]);
    let carbonate = bench.vessel(TARGET).expect("post-soda").clone();
    assert_current_pitzer(&carbonate);
    assert_eq!(carbonate.thermal_mode, ThermalMode::Adiabatic);
    assert!(carbonate.unresolved_materials.is_empty());
    assert!(carbonate.temperature.0.is_finite() && carbonate.temperature.0 >= 252.0);

    let mut expected_stock = physical_ledger(&baseline);
    let mut input = baseline.contents.clone();
    for component in soda {
        add_inlet(&mut expected_stock, &component.species, component.moles.0);
        let data = species::lookup(&component.species).expect("registered soda component");
        input.push(Portion {
            species: component.species.clone(),
            moles: component.moles,
            phase: data.standard_phase,
        });
    }
    assert_ledger_close(
        &expected_stock,
        &physical_ledger(&carbonate),
        "carbonate input",
    );
    let quote = heat_released_j(
        &input,
        baseline.free_hydroxide,
        &carbonate.contents,
        carbonate.free_hydroxide,
        &[],
        "pitzer",
    )
    .expect("complete modeled carbonate reference");
    let sensible = kerotakis_core::solve::portions_enthalpy(
        carbonate
            .contents
            .iter()
            .map(|portion| (&portion.species, portion.moles.0, portion.phase)),
        298.15,
        carbonate.temperature.0,
    );
    assert!(
        (sensible - quote).abs() <= PHASE_HEAT_BUDGET_J,
        "carbonate modeled sensible {sensible:.9} J vs native references {quote:.9} J"
    );

    let wait_events = bench
        .step_with(
            Operator::Wait { seconds: 0.0 },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("typed zero-time native re-equilibration");
    assert_no_failed_or_gas(&wait_events, "plain carbonate zero-time wait");
    assert!(
        !wait_events.iter().any(|event| matches!(
            event,
            Event::Added { .. }
                | Event::MaterialAdded { .. }
                | Event::Dissolved { .. }
                | Event::DissolvedInSolvent { .. }
                | Event::ReactionOccurred { .. }
                | Event::HeatOfMixing { .. }
                | Event::EnergyTransferred { .. }
        )),
        "zero-time wait repeated material or heat: {wait_events:?}"
    );
    let repeated = bench.vessel(TARGET).expect("repeated carbonate");
    assert_current_pitzer(repeated);
    assert!(repeated.unresolved_materials.is_empty());
    assert_ledger_close(
        &physical_ledger(&carbonate),
        &physical_ledger(repeated),
        "plain carbonate zero-time wait",
    );
    let keys: BTreeSet<_> = carbonate
        .contents
        .iter()
        .chain(&repeated.contents)
        .map(|portion| (portion.species.0.clone(), portion.phase))
        .collect();
    for (species, phase) in keys {
        let delta =
            portion_amount(repeated, &species, phase) - portion_amount(&carbonate, &species, phase);
        assert!(
            delta.abs() <= ATOMS_AND_CHARGE_BUDGET,
            "plain carbonate zero-time changed {species} {phase:?} by {delta:.15e} mol"
        );
    }
    assert!((repeated.temperature.0 - carbonate.temperature.0).abs() <= PH_AND_T_BUDGET);
    let first = carbonate
        .solution
        .as_ref()
        .expect("first native carbonate state");
    let second = repeated
        .solution
        .as_ref()
        .expect("repeated native carbonate state");
    assert!((second.ph - first.ph).abs() <= PH_AND_T_BUDGET);
    assert!((second.ionic_strength - first.ionic_strength).abs() <= IONIC_BUDGET);
    let thermal_delta = kerotakis_core::solve::portions_enthalpy(
        repeated
            .contents
            .iter()
            .map(|portion| (&portion.species, portion.moles.0, portion.phase)),
        carbonate.temperature.0,
        repeated.temperature.0,
    );
    assert!(thermal_delta.abs() <= PHASE_HEAT_BUDGET_J);
}
