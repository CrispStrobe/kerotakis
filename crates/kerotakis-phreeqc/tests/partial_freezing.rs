//! AQ-010: pure ice and a re-equilibrated residual brine must coexist.

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(PhaseEquilibrator::wrapping(Box::new(
            PhreeqcEquilibrator::new().expect("engine"),
        ))),
        Box::new(HonestyEquilibrator),
    ])
}

fn step(bench: &mut Bench, stack: &mut SolverStack, operator: Operator) -> Vec<Event> {
    bench
        .step_with(operator, stack, &PermissiveScreen)
        .expect("AQ-010 bench step")
}

fn water_moles(vessel: &Vessel, phase: Phase) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|portion| portion.species.0 == "water" && portion.phase == phase)
        .map(|portion| portion.moles.0)
        .sum()
}

fn particle_molality(vessel: &Vessel) -> f64 {
    vessel
        .solution
        .as_ref()
        .expect("residual brine was re-equilibrated")
        .species
        .iter()
        .filter(|species| species.name != "H2O")
        .map(|species| species.molality)
        .sum()
}

#[test]
fn cooling_salt_water_removes_pure_ice_and_resolves_the_residual_brine() {
    let mut bench = Bench::new();
    let mut solvers = stack();
    let vessel_id = VesselId(0);
    step(
        &mut bench,
        &mut solvers,
        Operator::Add {
            vessel: vessel_id,
            species: SpeciesId::new("water"),
            moles: Moles(5.5509),
            at: None,
        },
    );
    step(
        &mut bench,
        &mut solvers,
        Operator::Add {
            vessel: vessel_id,
            species: SpeciesId::new("NaCl"),
            moles: Moles(0.05),
            at: None,
        },
    );

    let before = bench.vessel(vessel_id).unwrap();
    let initial_particles = particle_molality(before);
    let initial_water = before.moles_of(&SpeciesId::new("water")).0;
    let initial_sodium =
        before.moles_of(&SpeciesId::new("Na+")).0 + before.moles_of(&SpeciesId::new("NaCl")).0;
    let events = step(
        &mut bench,
        &mut solvers,
        Operator::Cool {
            vessel: vessel_id,
            energy: Joules(20_000.0),
        },
    );
    let frozen = bench.vessel(vessel_id).unwrap();

    assert!(
        water_moles(frozen, Phase::Solid) > 0.0,
        "no ice: {frozen:?}"
    );
    assert!(
        water_moles(frozen, Phase::Liquid) > 0.0,
        "no residual brine: {frozen:?}"
    );
    assert!(
        frozen
            .contents
            .iter()
            .filter(|portion| portion.phase == Phase::Solid)
            .all(|portion| portion.species.0 == "water"),
        "the ice compartment must contain pure water: {:?}",
        frozen.contents
    );
    assert!(particle_molality(frozen) > initial_particles);
    let liquidus = kerotakis_core::states::transitions(particle_molality(frozen)).freezing_k;
    assert!(
        (frozen.temperature.0 - liquidus).abs() < 0.02,
        "temperature {} K and re-solved liquidus {liquidus} K disagree",
        frozen.temperature.0
    );
    assert!(
        (frozen.moles_of(&SpeciesId::new("water")).0 - initial_water).abs() < 2e-6,
        "water ledger changed: {} -> {}",
        initial_water,
        frozen.moles_of(&SpeciesId::new("water")).0
    );
    let final_sodium =
        frozen.moles_of(&SpeciesId::new("Na+")).0 + frozen.moles_of(&SpeciesId::new("NaCl")).0;
    assert!(
        (final_sodium - initial_sodium).abs() < 1e-8,
        "sodium ledger changed: {initial_sodium} -> {final_sodium}"
    );
    assert!(events.iter().any(|event| matches!(
        event,
        Event::StateChanged {
            from: Phase::Liquid,
            to: Phase::Solid,
            ..
        }
    )));
    assert!(!events.iter().any(|event| matches!(
        event,
        Event::NotYetModeled { what, .. } if what.contains("what the dissolved substances do")
    )));

    let liquid_before = water_moles(frozen, Phase::Liquid);
    let temperature_before = frozen.temperature.0;
    let vessel = bench
        .vessels
        .iter_mut()
        .find(|vessel| vessel.id == vessel_id)
        .expect("AQ-010 vessel");
    let repeat = solvers.equilibrate(vessel).expect("repeat equilibrium");
    let settled = bench.vessel(vessel_id).unwrap();
    assert!(!repeat
        .iter()
        .any(|event| matches!(event, Event::StateChanged { .. })));
    assert!((water_moles(settled, Phase::Liquid) - liquid_before).abs() < 2e-8);
    assert!((settled.temperature.0 - temperature_before).abs() < 1e-8);
}
