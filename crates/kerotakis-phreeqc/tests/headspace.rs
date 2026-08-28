//! Live finite-volume gas/liquid equilibrium through IPhreeqc.

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn step(bench: &mut Bench, solver: &mut PhreeqcEquilibrator, operator: Operator) -> Vec<Event> {
    bench
        .step_with(operator, solver, &PermissiveScreen)
        .expect("bench step")
}

fn add(
    bench: &mut Bench,
    solver: &mut PhreeqcEquilibrator,
    vessel: VesselId,
    species: &str,
    moles: f64,
) -> Vec<Event> {
    step(
        bench,
        solver,
        Operator::Add {
            vessel,
            species: SpeciesId::new(species),
            moles: Moles(moles),
            at: None,
        },
    )
}

fn carbon_moles(vessel: &Vessel) -> f64 {
    vessel
        .contents
        .iter()
        .map(|portion| {
            let formula = species::lookup(&portion.species)
                .and_then(|data| stoich::parse_formula(data.formula).ok());
            portion.moles.0
                * formula
                    .and_then(|formula| formula.counts.get("C").copied())
                    .unwrap_or(0.0)
        })
        .sum()
}

fn gas_moles_of(vessel: &Vessel, species: &str) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|portion| portion.phase == Phase::Gas && portion.species.0 == species)
        .map(|portion| portion.moles.0)
        .sum()
}

fn solid_moles_of(vessel: &Vessel, species: &str) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|portion| portion.phase == Phase::Solid && portion.species.0 == species)
        .map(|portion| portion.moles.0)
        .sum()
}

fn event_gas_moles(events: &[Event], absorbed: bool) -> f64 {
    events
        .iter()
        .filter_map(|event| match event {
            Event::GasAbsorbed { species, moles, .. } if absorbed && species.0 == "CO2" => {
                Some(moles.0)
            }
            Event::GasEvolved { species, moles, .. } if !absorbed && species.0 == "CO2" => {
                Some(moles.0)
            }
            _ => None,
        })
        .sum()
}

#[test]
fn sparkling_water_recipe_fizzes_into_an_open_vessel() {
    let mut bench = Bench::new();
    let mut solver = PhreeqcEquilibrator::new().expect("engine");
    let op = kerotakis_core::script::parse_op("add v1 Sprudel 100mL")
        .expect("valid localized sparkling-water command")
        .expect("operator");
    let events = step(&mut bench, &mut solver, op);

    assert!(
        events.iter().any(|event| matches!(event,
            Event::GasEvolved { species, moles, .. }
                if species.0 == "CO2" && moles.0 > 0.001
        )),
        "open sparkling water must visibly lose computed CO2: {events:?}"
    );
    let solution = bench.vessels[0].solution.as_ref().expect("aqueous state");
    assert!(
        solution.ph < 6.0,
        "dissolved CO2 makes the water acidic: {}",
        solution.ph
    );
}

#[test]
fn closed_bicarbonate_keeps_carbon_and_differs_from_an_open_beaker() {
    let vessel = VesselId(0);

    let mut open_solver = PhreeqcEquilibrator::new().expect("engine");
    let mut open = Bench::new();
    add(&mut open, &mut open_solver, vessel, "water", 55.51);
    let open_events = add(&mut open, &mut open_solver, vessel, "NaHCO3", 0.05);
    let open_ph = open.vessel(vessel).unwrap().solution.as_ref().unwrap().ph;
    assert!(open_events.iter().any(|event| matches!(
        event,
        Event::GasEvolved { species, .. } if species.0 == "CO2"
    )));

    let mut sealed_solver = PhreeqcEquilibrator::new().expect("engine");
    let mut sealed = Bench::new();
    step(
        &mut sealed,
        &mut sealed_solver,
        Operator::Seal {
            vessel,
            headspace_volume: Liters(1.0),
        },
    );
    let initially_sealed = sealed.vessel(vessel).unwrap();
    let trapped_carbon = carbon_moles(initially_sealed);
    let trapped_n2 = gas_moles_of(initially_sealed, "N2");
    let trapped_o2 = gas_moles_of(initially_sealed, "O2");
    add(&mut sealed, &mut sealed_solver, vessel, "water", 55.51);
    let sealed_events = add(&mut sealed, &mut sealed_solver, vessel, "NaHCO3", 0.05);
    let state = sealed.vessel(vessel).unwrap();
    let sealed_ph = state.solution.as_ref().unwrap().ph;

    assert!(sealed_events
        .iter()
        .any(|event| matches!(event, Event::HeadspaceEquilibrated { .. })));
    let reported_pressure = sealed_events.iter().find_map(|event| match event {
        Event::HeadspaceEquilibrated { pressure, .. } => Some(pressure.0),
        _ => None,
    });
    assert_eq!(reported_pressure, Some(state.pressure.0));
    assert!(
        !sealed_events.iter().any(|event| matches!(
            event,
            Event::GasEvolved { species, .. } if species.0 == "CO2"
        )),
        "a sealed vessel must not emit an escaped-gas event"
    );
    assert!(
        (carbon_moles(state) - (0.05 + trapped_carbon)).abs() < 2e-7,
        "trapped and added carbon remain in the sealed vessel: {:?}",
        state.contents
    );
    assert!(state.moles_of(&SpeciesId::new("CO2")).0 > 0.0);
    assert!((gas_moles_of(state, "N2") - trapped_n2).abs() < 1e-12);
    assert!((gas_moles_of(state, "O2") - trapped_o2).abs() < 1e-12);
    assert!(
        sealed_ph < open_ph - 0.2,
        "retaining CO2 should keep bicarbonate less alkaline: sealed={sealed_ph}, open={open_ph}"
    );
}

#[test]
fn pressure_controlled_headspace_conserves_carbon_at_its_target_pressure() {
    let vessel = VesselId(0);
    let mut solver = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    step(
        &mut bench,
        &mut solver,
        Operator::Regulate {
            vessel,
            pressure: Pascal(150_000.0),
            initial_volume: Liters(1.0),
        },
    );
    let initial_carbon = carbon_moles(bench.vessel(vessel).unwrap());
    add(&mut bench, &mut solver, vessel, "water", 55.51);
    let events = add(&mut bench, &mut solver, vessel, "NaHCO3", 0.05);
    let state = bench.vessel(vessel).unwrap();

    assert!(matches!(
        state.headspace,
        Headspace::PressureControlled { pressure, volume }
            if pressure == Pascal(150_000.0) && volume.0 > 1.0
    ));
    assert_eq!(state.pressure, Pascal(150_000.0));
    assert!((carbon_moles(state) - (initial_carbon + 0.05)).abs() < 2e-7);
    assert!(events.iter().any(|event| matches!(
        event,
        Event::HeadspaceEquilibrated { pressure, .. }
            if *pressure == Pascal(150_000.0)
    )));
}

#[test]
fn nitrogen_sweep_removes_virtually_all_bicarbonate_carbon() {
    let vessel = VesselId(0);
    let mut solver = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    step(
        &mut bench,
        &mut solver,
        Operator::Sweep {
            vessel,
            pressure: Pascal::ATMOSPHERIC,
        },
    );
    add(&mut bench, &mut solver, vessel, "water", 55.51);
    let events = add(&mut bench, &mut solver, vessel, "NaHCO3", 0.05);
    let state = bench.vessel(vessel).unwrap();

    assert!(carbon_moles(state) < 1e-4, "state: {:?}", state.contents);
    assert!(events.iter().any(|event| matches!(
        event,
        Event::GasEvolved { species, moles, .. }
            if species.0 == "CO2" && moles.0 > 0.049
    )));
    assert_eq!(state.gas_moles(), Moles(0.0));
    assert_eq!(state.pressure, Pascal::ATMOSPHERIC);
}

#[test]
fn reopening_vents_the_headspace_and_returns_to_open_equilibrium() {
    let vessel = VesselId(0);
    let mut solver = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    step(
        &mut bench,
        &mut solver,
        Operator::Seal {
            vessel,
            headspace_volume: Liters(0.5),
        },
    );
    add(&mut bench, &mut solver, vessel, "water", 55.51);
    add(&mut bench, &mut solver, vessel, "NaHCO3", 0.05);
    let before = carbon_moles(bench.vessel(vessel).unwrap());

    let events = step(&mut bench, &mut solver, Operator::Open { vessel });
    let vented_co2: f64 = events
        .iter()
        .filter_map(|event| match event {
            Event::GasEvolved { species, moles, .. } if species.0 == "CO2" => Some(moles.0),
            _ => None,
        })
        .sum();
    let state = bench.vessel(vessel).unwrap();
    assert_eq!(state.headspace, Headspace::Open);
    assert!(vented_co2 > 0.0);
    assert!(carbon_moles(state) < before);
    assert!((state.pressure.0 - Pascal::ATMOSPHERIC.0).abs() < 1e-9);
}

#[test]
fn co2_dose_makes_limewater_milky_then_excess_redissolves_it() {
    let vessel = VesselId(0);
    let mut solver = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    add(&mut bench, &mut solver, vessel, "water", 55.51);
    add(&mut bench, &mut solver, vessel, "Ca(OH)2", 0.01);

    let first = add(&mut bench, &mut solver, vessel, "CO2", 0.01);
    let milky = solid_moles_of(bench.vessel(vessel).unwrap(), "CaCO3");
    assert!(milky > 0.005, "first CO2 dose should form calcite: {milky}");
    assert!(event_gas_moles(&first, true) > 0.009);
    assert!(first.iter().any(|event| matches!(
        event,
        Event::Precipitated { species, .. } if species.0 == "CaCO3"
    )));
    assert!(
        (carbon_moles(bench.vessel(vessel).unwrap()) + event_gas_moles(&first, false) - 0.01).abs()
            < 2e-6
    );

    let second = add(&mut bench, &mut solver, vessel, "CO2", 0.05);
    let state = bench.vessel(vessel).unwrap();
    let remaining = solid_moles_of(state, "CaCO3");
    assert!(
        remaining < milky * 0.1,
        "excess CO2 should redissolve calcite: before={milky}, after={remaining}, state={:?}",
        state.contents
    );
    assert!(event_gas_moles(&second, true) > 0.009);
    assert!(event_gas_moles(&second, false) > 0.005);
    assert!(second.iter().any(|event| matches!(
        event,
        Event::Dissolved { species, .. } if species.0 == "CaCO3"
    )));

    let vented = event_gas_moles(&first, false) + event_gas_moles(&second, false);
    assert!(
        (carbon_moles(state) + vented - 0.06).abs() < 2e-6,
        "all dosed carbon is condensed or vented: condensed={}, vented={vented}",
        carbon_moles(state)
    );
    assert_eq!(state.gas_moles(), Moles(0.0));
}
