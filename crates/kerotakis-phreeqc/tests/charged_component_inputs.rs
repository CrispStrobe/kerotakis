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
