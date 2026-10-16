#![cfg(feature = "engine")]
//! Admission/reporting controls, not measurements or reaction calibration.
use kerotakis_core::{
    ledger::ConservedLedger, Equilibrator, Event, Grams, Headspace, Kelvin, Liters, Moles, Phase,
    SpeciesId, SurfaceModel, SurfaceSites, ThermalMode, Vessel, VesselId,
};
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn water() -> Vessel {
    let mut vessel = Vessel::new(VesselId(0), "charged component control");
    vessel.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
    vessel.thermal_mode = ThermalMode::Thermostatted(Kelvin(298.15));
    vessel
}

fn deposit(vessel: &mut Vessel, key: &str, moles: f64) {
    let data = kerotakis_core::species::lookup_key(key).expect("registered component");
    vessel.deposit(SpeciesId::new(key), Moles(moles), data.standard_phase);
}

fn named(events: &[Event], key: &str) -> bool {
    // Assert the public structured diagnostic identity, not translated prose.
    events
        .iter()
        .any(|event| serde_json::to_value(event).unwrap()["reason"]["key"].as_str() == Some(key))
}

fn close_inventory(before: &Vessel, after: &Vessel) {
    let before = ConservedLedger::from_vessel(before);
    let after = ConservedLedger::from_vessel(after);
    assert!((before.charge - after.charge).abs() < 1e-9);
    for (element, amount) in before.elements {
        // These fixtures contain no gas boundary/exchange operation.
        let observed = after.elements.get(&element).copied().unwrap_or(0.0);
        assert!(
            (amount - observed).abs() < 1e-8 + amount.abs() * 1e-6,
            "{element}: {amount} -> {observed}"
        );
    }
}

#[test]
fn bare_hydrogen_hydroxide_and_equal_pair_do_not_inherit_pure_water_readings() {
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    for inputs in [
        vec![("H+", 0.001)],
        vec![("OH-", 0.001)],
        vec![("H+", 0.001), ("OH-", 0.001)],
    ] {
        let mut vessel = water();
        solver.equilibrate(&mut vessel).unwrap();
        assert!(vessel.solution.is_some(), "positive pure-water control");
        for (key, moles) in inputs {
            deposit(&mut vessel, key, moles);
        }
        let before = vessel.clone();
        assert!(solver.chemistry_applies(&vessel));
        for _ in 0..2 {
            let events = solver.equilibrate(&mut vessel).unwrap();
            assert!(named(&events, "not-modeled.bare-acid-base-basis"));
            assert!(vessel.solution.is_none());
            assert_eq!(vessel.free_proton, 0.0);
            assert_eq!(vessel.free_hydroxide, 0.0);
            assert_eq!(
                serde_json::to_value(&vessel.contents).unwrap(),
                serde_json::to_value(&before.contents).unwrap()
            );
            assert_eq!(vessel.temperature, before.temperature);
            close_inventory(&before, &vessel);
        }
    }
}

#[test]
fn positive_trace_basis_is_not_misreported_as_plain_water() {
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    for dose in [1e-14, 1e-20] {
        for ion in ["H+", "OH-"] {
            let mut vessel = water();
            solver.equilibrate(&mut vessel).unwrap();
            deposit(&mut vessel, ion, dose);
            let before = serde_json::to_value(&vessel.contents).unwrap();
            let events = solver.equilibrate(&mut vessel).unwrap();
            assert!(named(&events, "not-modeled.bare-acid-base-basis"));
            assert!(vessel.solution.is_none());
            assert_eq!(serde_json::to_value(&vessel.contents).unwrap(), before);
        }
    }
}

#[test]
fn a_typed_hfo_interface_retains_its_own_charge_reporting() {
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    let mut vessel = water();
    // Same reviewed geometry/capacity as tests/surface.rs. An unbalanced
    // analytical zinc component deliberately exercises the generic warning
    // before adding the interface that owns its separate charge model.
    deposit(&mut vessel, "NaNO3", 0.01);
    deposit(&mut vessel, "Zn+2", 1e-6);
    let mut bulk = vessel.clone();
    let bulk_events = solver.equilibrate(&mut bulk).unwrap();
    assert!(named(
        &bulk_events,
        "not-modeled.analytical-component-charge"
    ));
    vessel.surfaces.push(SurfaceSites {
        label: "HFO charge control".to_string(),
        model: SurfaceModel::HydrousFerricOxide,
        mass: Grams(0.09),
        specific_area_m2_per_g: 600.0,
        strong_capacity: Moles(5e-6),
        weak_capacity: Moles(2e-4),
        occupancy: Vec::new(),
        water_release: Moles(0.0),
    });
    for _ in 0..2 {
        let events = solver.equilibrate(&mut vessel).unwrap();
        assert!(!named(&events, "not-modeled.analytical-component-charge"));
        assert!(!named(&events, "not-modeled.bare-acid-base-basis"));
        assert!(vessel.solution.is_some());
        assert!(vessel.surfaces[0].has_valid_capacity());
    }
}

#[test]
fn actual_counterions_restore_complete_acid_and_base_characterization() {
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    for (ion, counterion, acid) in [("H+", "Cl-", true), ("OH-", "Na+", false)] {
        let mut vessel = water();
        deposit(&mut vessel, ion, 0.001);
        solver.equilibrate(&mut vessel).unwrap();
        assert!(vessel.solution.is_none());
        deposit(&mut vessel, counterion, 0.001);
        let before = vessel.clone();
        let events = solver.equilibrate(&mut vessel).unwrap();
        assert!(!named(&events, "not-modeled.analytical-component-charge"));
        let ph = vessel.solution.as_ref().unwrap().ph;
        assert!(
            if acid { ph < 3.0 } else { ph > 11.0 },
            "{ion}/{counterion}: pH {ph}"
        );
        close_inventory(&before, &vessel);
    }
}

#[test]
fn legacy_unbalanced_components_remain_explicit_analytical_questions() {
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    for ion in ["NH4+", "CO3-2", "Na+", "Cl-"] {
        let mut vessel = water();
        // Retain carbon so this checks inventory/reporting, not open-boundary
        // carbon dioxide loss.
        vessel.headspace = Headspace::Sealed {
            volume: Liters(1.0),
        };
        deposit(&mut vessel, ion, 0.001);
        let before = vessel.clone();
        for _ in 0..2 {
            let events = solver.equilibrate(&mut vessel).unwrap();
            assert!(
                named(&events, "not-modeled.analytical-component-charge"),
                "{ion}"
            );
            assert!(
                vessel.solution.is_some(),
                "legacy analytical route retained"
            );
            close_inventory(&before, &vessel);
        }
    }
}

#[test]
fn complete_stocks_and_explicit_charge_balanced_components_agree() {
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    for (stock, ions) in [
        ("HCl", vec![("H+", 0.001), ("Cl-", 0.001)]),
        ("NaOH", vec![("Na+", 0.001), ("OH-", 0.001)]),
        ("NH4Cl", vec![("NH4+", 0.001), ("Cl-", 0.001)]),
        ("Na2CO3", vec![("Na+", 0.002), ("CO3-2", 0.001)]),
    ] {
        let mut physical = water();
        deposit(&mut physical, stock, 0.001);
        let stock_events = solver.equilibrate(&mut physical).unwrap();
        assert!(
            !named(&stock_events, "not-modeled.analytical-component-charge"),
            "{stock}"
        );
        let mut analytical = water();
        for (ion, moles) in ions {
            deposit(&mut analytical, ion, moles);
        }
        let before = analytical.clone();
        let ion_events = solver.equilibrate(&mut analytical).unwrap();
        assert!(
            !named(&ion_events, "not-modeled.analytical-component-charge"),
            "{stock}"
        );
        let ph = analytical.solution.as_ref().unwrap().ph;
        let reference = physical.solution.as_ref().unwrap().ph;
        assert!(
            (ph - reference).abs() < 1e-4,
            "{stock}: stock {reference}, ions {ph}"
        );
        close_inventory(&before, &analytical);
        // Analytical H/O closure left by a legitimate stock must not become
        // a new naked-ion warning during the next equilibration.
        let repeated = solver.equilibrate(&mut physical).unwrap();
        assert!(
            !named(&repeated, "not-modeled.analytical-component-charge"),
            "{stock}"
        );
        assert!(
            !named(&repeated, "not-modeled.bare-acid-base-basis"),
            "{stock}"
        );
    }
}

#[test]
fn completing_weak_ion_countercharge_clears_the_analytical_input_note() {
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    for (ion, counterion, counterdose, acid) in
        [("NH4+", "Cl-", 0.001, true), ("CO3-2", "Na+", 0.002, false)]
    {
        let mut vessel = water();
        vessel.headspace = Headspace::Sealed {
            volume: Liters(1.0),
        };
        deposit(&mut vessel, ion, 0.001);
        let first = solver.equilibrate(&mut vessel).unwrap();
        assert!(named(&first, "not-modeled.analytical-component-charge"));
        deposit(&mut vessel, counterion, counterdose);
        let before = vessel.clone();
        for _ in 0..2 {
            let events = solver.equilibrate(&mut vessel).unwrap();
            assert!(!named(&events, "not-modeled.analytical-component-charge"));
            let ph = vessel.solution.as_ref().unwrap().ph;
            assert!(if acid { ph < 7.0 } else { ph > 9.0 }, "{ion}: pH {ph}");
            close_inventory(&before, &vessel);
        }
    }
}

#[test]
fn mixing_a_withheld_ion_input_falls_back_to_the_complete_target_problem() {
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    let mut acid_component = water();
    deposit(&mut acid_component, "H+", 0.001);
    solver.equilibrate(&mut acid_component).unwrap();
    assert!(acid_component.solution.is_none());
    let mut counterion_component = water();
    deposit(&mut counterion_component, "Cl-", 0.001);
    let mut mixed = water();
    mixed.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
    deposit(&mut mixed, "H+", 0.001);
    deposit(&mut mixed, "Cl-", 0.001);
    assert!(solver
        .mix(&mut mixed, &acid_component, 1.0, &counterion_component, 1.0)
        .is_none());
    let before = mixed.clone();
    let events = solver.equilibrate(&mut mixed).unwrap();
    assert!(!named(&events, "not-modeled.analytical-component-charge"));
    assert!(mixed.solution.as_ref().unwrap().ph < 3.0);
    close_inventory(&before, &mixed);
}

#[test]
fn bulk_charge_reporting_has_a_declared_readback_resolution() {
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    for (dose, warned) in [(5e-9, false), (2e-8, true), (1e-6, true), (1e-3, true)] {
        for ion in ["Na+", "Cl-", "NH4+"] {
            let mut vessel = water();
            deposit(&mut vessel, ion, dose);
            let before = vessel.clone();
            let events = solver.equilibrate(&mut vessel).unwrap();
            assert_eq!(
                named(&events, "not-modeled.analytical-component-charge"),
                warned,
                "{ion}, {dose} mol"
            );
            close_inventory(&before, &vessel);
        }
    }
}

fn production_stack() -> kerotakis_core::SolverStack {
    use kerotakis_core::{DisplacementEquilibrator, PhaseEquilibrator, SolverStack};
    SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
        PhaseEquilibrator::wrapping(Box::new(DisplacementEquilibrator::wrapping(Box::new(
            PhreeqcEquilibrator::new().unwrap(),
        )))),
    )]))
}

fn execute_complete_input_script(script: &[&str]) -> (kerotakis_core::Bench, Vec<Event>) {
    let mut bench = kerotakis_core::Bench::new();
    let mut solver = production_stack();
    let mut events = Vec::new();
    for line in script {
        let op = kerotakis_core::script::parse_op(line)
            .expect("syntax")
            .expect("operator");
        let more = bench
            .step_with(op, &mut solver, &kerotakis_core::PermissiveScreen)
            .unwrap();
        assert!(
            !named(&more, "not-modeled.analytical-component-charge"),
            "{line}"
        );
        assert!(!named(&more, "not-modeled.bare-acid-base-basis"), "{line}");
        events.extend(more);
    }
    (bench, events)
}

#[test]
fn complete_carbonated_water_paths_do_not_become_charged_input_gaps() {
    for script in [
        vec![
            "add v1 water 100mL",
            "seal v1 100mL",
            "add v1 CO2 0.02mol",
            "open v1",
            "wait 1s",
        ],
        vec![
            "add v1 water 100mL",
            "regulate v1 2bar 500mL",
            "add v1 CO2 0.01mol",
            "wait 1s",
        ],
    ] {
        let (bench, events) = execute_complete_input_script(&script);
        assert!(bench.vessel(VesselId(0)).unwrap().solution.is_some());
        assert!(events
            .iter()
            .any(|event| matches!(event, Event::HeadspaceEquilibrated { .. })));
    }
}

#[test]
fn chloralkali_retains_countercharge_and_water_through_the_aqueous_tail() {
    let (bench, events) = execute_complete_input_script(&[
        "add v1 water 100mL",
        "add v1 NaCl 0.01mol",
        "electrolyse v1 0.5A 30min",
    ]);
    let vessel = bench.vessel(VesselId(0)).unwrap();
    let charge = ConservedLedger::from_vessel(vessel).charge;
    assert!(
        charge.abs() < 1e-8,
        "charge retained after electrolysis: {charge}"
    );
    let electrons = 0.5 * 1800.0 / kerotakis_core::displacement::FARADAY;
    assert!(
        (vessel.moles_of(&SpeciesId::new("water")).0 - (5.534276991396059 - electrons)).abs()
            < 1e-7
    );
    assert!(vessel.solution.as_ref().unwrap().ph > 11.0);
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::Electrolysed { .. })));
}
