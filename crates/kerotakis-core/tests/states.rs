//! Freezing, melting, and the plateau in between.
//!
//! These pin the two claims that make the states model worth having: the
//! temperature stops at the transition while the phase change is under way,
//! and the transition itself moves with how many particles are dissolved.

use kerotakis_core::*;
use std::cell::Cell;
use std::rc::Rc;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(StateEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn water_bench(moles: f64) -> Bench {
    let mut bench = Bench::new();
    let mut s = stack();
    bench
        .step_with(
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new("water"),
                moles: Moles(moles),
                at: None,
            },
            &mut s,
            &PermissiveScreen,
        )
        .expect("add");
    bench
}

fn cool(bench: &mut Bench, joules: f64) -> Vec<Event> {
    bench
        .step_with(
            Operator::Cool {
                vessel: VesselId(0),
                energy: Joules(joules),
            },
            &mut stack(),
            &PermissiveScreen,
        )
        .expect("cool")
}

#[test]
fn water_does_not_go_below_zero_while_it_is_still_freezing() {
    // The bug this whole module exists for: the bench used to report
    // liquid water at -71 C because nothing reconsidered the phase.
    let mut bench = water_bench(5.5343);
    let events = cool(&mut bench, 40_000.0);
    let v = bench.vessel(VesselId(0)).unwrap();
    assert!(
        (v.temperature.to_celsius() - 0.0).abs() < 0.05,
        "the plateau holds it at the freezing point, got {} C",
        v.temperature.to_celsius()
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::StateChanged { .. })),
        "{events:?}"
    );
    // Partly frozen: both phases present.
    let ice: f64 = v
        .contents
        .iter()
        .filter(|p| p.phase == Phase::Solid)
        .map(|p| p.moles.0)
        .sum();
    let liquid: f64 = v
        .contents
        .iter()
        .filter(|p| p.phase == Phase::Liquid)
        .map(|p| p.moles.0)
        .sum();
    assert!(ice > 0.0 && liquid > 0.0, "ice {ice}, liquid {liquid}");
    // Energy check: cooling to 0 C then freezing must account for the 40 kJ.
    let sensible = 5.5343 * 75.3 * 25.0;
    let expected_ice = (40_000.0 - sensible) / kerotakis_core::states::WATER_H_FUS;
    assert!(
        (ice - expected_ice).abs() < 0.05,
        "expected {expected_ice:.3} mol of ice, got {ice:.3}"
    );
}

#[test]
fn no_temperature_is_announced_that_the_vessel_never_reached() {
    let mut bench = water_bench(5.5343);
    let events = cool(&mut bench, 40_000.0);
    let actual = bench.vessel(VesselId(0)).unwrap().temperature;
    for e in &events {
        if let Event::TemperatureChanged { to, .. } = e {
            assert!(
                (to.0 - actual.0).abs() < 0.05,
                "announced {} K but the vessel is at {} K",
                to.0,
                actual.0
            );
        }
    }
}

#[test]
fn enough_cooling_freezes_it_solid_and_then_chills_the_ice() {
    let mut bench = water_bench(5.5343);
    cool(&mut bench, 200_000.0);
    let v = bench.vessel(VesselId(0)).unwrap();
    assert!(
        v.temperature.to_celsius() < -5.0,
        "past the plateau it gets colder again: {} C",
        v.temperature.to_celsius()
    );
    assert!(
        v.contents.iter().all(|p| p.phase != Phase::Liquid),
        "nothing liquid left: {:?}",
        v.contents
    );
}

#[test]
fn ice_melts_again_when_warmed() {
    let mut bench = water_bench(5.5343);
    cool(&mut bench, 200_000.0);
    let events = bench
        .step_with(
            Operator::Heat {
                vessel: VesselId(0),
                energy: Joules(200_000.0),
            },
            &mut stack(),
            &PermissiveScreen,
        )
        .expect("heat");
    let v = bench.vessel(VesselId(0)).unwrap();
    assert!(
        v.contents.iter().any(|p| p.phase == Phase::Liquid),
        "it melted back: {:?} / {events:?}",
        v.contents
    );
}

#[test]
fn a_frozen_vessel_has_no_ph() {
    // Ice is not a solution. Continuing to report a pH beside a block of
    // ice was the original complaint.
    let mut bench = water_bench(5.5343);
    cool(&mut bench, 200_000.0);
    assert!(bench.vessel(VesselId(0)).unwrap().solution.is_none());
}

struct ParticleBalanceSolver {
    particle_moles: f64,
    calls: Rc<Cell<usize>>,
}

impl Equilibrator for ParticleBalanceSolver {
    fn name(&self) -> &'static str {
        "particle-balance-test"
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.calls.set(self.calls.get() + 1);
        let liquid_water_moles: f64 = vessel
            .contents
            .iter()
            .filter(|portion| portion.species.0 == "water" && portion.phase == Phase::Liquid)
            .map(|portion| portion.moles.0)
            .sum();
        let liquid_kg = liquid_water_moles * 0.018_015;
        if liquid_kg <= 0.0 {
            vessel.solution = None;
            return Ok(Vec::new());
        }
        let particle_molality = self.particle_moles / liquid_kg;
        vessel.solution = Some(SolutionInfo {
            pe: None,
            redox: Vec::new(),
            ph: 7.0,
            ionic_strength: particle_molality / 2.0,
            species: vec![SpeciesDetail {
                name: "test particles".to_string(),
                molality: particle_molality,
                activity: particle_molality,
            }],
            provenance: None,
        });
        Ok(Vec::new())
    }
}

fn partially_frozen_test_vessel(temperature_k: f64) -> Vessel {
    let mut vessel = Vessel::new(VesselId(0), "brine");
    vessel.deposit(SpeciesId::new("water"), Moles(5.5509), Phase::Liquid);
    vessel.deposit(SpeciesId::new("NaCl"), Moles(0.05), Phase::Aqueous);
    vessel.temperature = Kelvin(temperature_k);
    vessel
}

fn water_phase_moles(vessel: &Vessel, phase: Phase) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|portion| portion.species.0 == "water" && portion.phase == phase)
        .map(|portion| portion.moles.0)
        .sum()
}

#[test]
fn phase_coupling_respeciates_the_residual_brine_until_both_states_agree() {
    let calls = Rc::new(Cell::new(0));
    let chemistry = ParticleBalanceSolver {
        particle_moles: 0.1,
        calls: calls.clone(),
    };
    let mut coupled = PhaseEquilibrator::wrapping(Box::new(chemistry));
    let mut vessel = partially_frozen_test_vessel(260.0);
    let initial_water = vessel.moles_of(&SpeciesId::new("water")).0;

    let events = coupled.equilibrate(&mut vessel).unwrap();
    assert!(calls.get() > 1, "residual brine was not re-solved");
    assert!(
        calls.get() < 32,
        "phase coupling only stopped at its pass cap"
    );
    assert!(events.iter().any(|event| matches!(
        event,
        Event::StateChanged {
            from: Phase::Liquid,
            to: Phase::Solid,
            ..
        }
    )));
    assert!(!events
        .iter()
        .any(|event| matches!(event, Event::SolverFailed { .. })));
    assert!(water_phase_moles(&vessel, Phase::Liquid) > 0.0);
    assert!(water_phase_moles(&vessel, Phase::Solid) > 0.0);
    assert!((vessel.moles_of(&SpeciesId::new("water")).0 - initial_water).abs() < 1e-12);

    let particle_molality: f64 = vessel
        .solution
        .as_ref()
        .expect("residual brine remains a solution")
        .species
        .iter()
        .map(|species| species.molality)
        .sum();
    let liquidus = kerotakis_core::states::transitions(particle_molality).freezing_k;
    assert!(
        (vessel.temperature.0 - liquidus).abs() <= PHASE_COUPLED_TEMPERATURE_TOLERANCE_K,
        "phase state {} K and re-solved liquidus {liquidus} K disagree",
        vessel.temperature.0
    );

    let settled = vessel.clone();
    let second = coupled.equilibrate(&mut vessel).unwrap();
    assert!(!second
        .iter()
        .any(|event| matches!(event, Event::StateChanged { .. })));
    assert!((vessel.temperature.0 - settled.temperature.0).abs() < 1e-12);
    assert!(
        (water_phase_moles(&vessel, Phase::Liquid) - water_phase_moles(&settled, Phase::Liquid))
            .abs()
            < 1e-12
    );
}

#[test]
fn partial_freezing_stops_with_liquid_at_the_declared_model_boundary() {
    let chemistry = ParticleBalanceSolver {
        particle_moles: 0.6,
        calls: Rc::new(Cell::new(0)),
    };
    let mut coupled = PhaseEquilibrator::wrapping(Box::new(chemistry));
    // Start far enough below the liquidus that the available sensible heat
    // would freeze past the declared concentration boundary. At 240 K this
    // fixture instead reaches an ordinary, warmer partial-freezing balance.
    let mut vessel = partially_frozen_test_vessel(200.0);
    let events = coupled.equilibrate(&mut vessel).unwrap();

    assert!(water_phase_moles(&vessel, Phase::Liquid) > 0.0);
    assert!(water_phase_moles(&vessel, Phase::Solid) > 0.0);
    assert!((vessel.temperature.0 - kerotakis_core::states::BRINE_MODEL_MIN_K).abs() < 1e-9);
    assert!(events.iter().any(|event| matches!(
        event,
        Event::NotYetModeled { what, .. } if what.contains("partial-freezing model boundary")
    )));
}
