//! Freezing, melting, and the plateau in between.
//!
//! These pin the two claims that make the states model worth having: the
//! temperature stops at the transition while the phase change is under way,
//! and the transition itself moves with how many particles are dissolved.

use kerotakis_core::*;
use std::cell::Cell;
use std::rc::Rc;

fn census_vessel(reported_each: f64) -> Vessel {
    let mut vessel = Vessel::new(VesselId(0), "particle-census control");
    vessel.deposit(SpeciesId::new("water"), Moles(5.5509), Phase::Liquid);
    vessel.solution = Some(
        serde_json::from_value(serde_json::json!({
            "ph": 7.0,
            "ionic_strength": reported_each,
            "species": [
                {"name": "Na+", "molality": reported_each, "activity": reported_each},
                {"name": "Cl-", "molality": reported_each, "activity": reported_each}
            ]
        }))
        .expect("historical solution shape"),
    );
    vessel
}

#[test]
fn native_numeric_particle_census_sets_liquidus_despite_rounded_species_rows() {
    let mut rounded = census_vessel(0.5);
    rounded.solution.as_mut().unwrap().particle_molality = Some(1.001);
    let precise_rows = census_vessel(0.5005);
    let old_rows = census_vessel(0.5);

    let numeric_liquidus = kerotakis_core::solve::vessel_transitions(&rounded)
        .0
        .freezing_k;
    let precise_liquidus = kerotakis_core::solve::vessel_transitions(&precise_rows)
        .0
        .freezing_k;
    let rounded_liquidus = kerotakis_core::solve::vessel_transitions(&old_rows)
        .0
        .freezing_k;
    assert!((numeric_liquidus - precise_liquidus).abs() < 1e-10);
    assert!((numeric_liquidus - rounded_liquidus).abs() > 1e-4);
}

#[test]
fn historical_solution_without_numeric_census_keeps_old_liquidus_and_wire_shape() {
    let historical = census_vessel(0.5);
    let info = historical.solution.as_ref().unwrap();
    assert_eq!(info.particle_molality, None);
    let wire = serde_json::to_value(info).unwrap();
    assert!(wire.get("particle_molality").is_none());
    let reread: SolutionInfo = serde_json::from_value(wire).unwrap();
    assert_eq!(reread.particle_molality, None);

    let mut restored = historical.clone();
    restored.solution = Some(reread);
    assert_eq!(
        kerotakis_core::solve::vessel_transitions(&historical)
            .0
            .freezing_k,
        kerotakis_core::solve::vessel_transitions(&restored)
            .0
            .freezing_k
    );
}

struct AlternatingThermalChemistry {
    calls: Rc<Cell<usize>>,
    solution: SolutionInfo,
}

impl Equilibrator for AlternatingThermalChemistry {
    fn name(&self) -> &'static str {
        "alternating-thermal-chemistry"
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        let call = self.calls.get() + 1;
        self.calls.set(call);
        // Deliberately never settle: each chemistry answer pushes a real
        // water-phase pass to the other side of this solution's liquidus.
        vessel.solution = Some(self.solution.clone());
        vessel.temperature = Kelvin(if call % 2 == 0 { 272.0 } else { 270.0 });
        Ok(Vec::new())
    }
}

#[test]
fn exhausted_phase_coupling_fails_its_route_and_rolls_back_partial_phase_state() {
    let mut vessel = census_vessel(0.5);
    vessel.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Solid);
    vessel.deposit(SpeciesId::new("NaCl"), Moles(0.05), Phase::Aqueous);
    let before = vessel.clone();
    let calls = Rc::new(Cell::new(0));
    let chemistry = AlternatingThermalChemistry {
        calls: calls.clone(),
        solution: vessel.solution.as_ref().unwrap().clone(),
    };
    let mut solvers = SolverStack::new(vec![Box::new(PhaseEquilibrator::wrapping(Box::new(
        chemistry,
    )))]);

    let events = solvers.equilibrate(&mut vessel).unwrap();
    assert_eq!(
        calls.get(),
        32,
        "the bounded coupling path must be exercised"
    );
    assert!(matches!(
        &solvers.last_routes[0].outcome,
        SolverRouteOutcome::Failed
    ));
    assert!(events.iter().any(|event| matches!(
        event,
        Event::SolverFailed { solver, detail, .. }
            if solver == "phase-coupled" && detail.contains("did not settle within 32 bounded passes")
    )));
    assert_eq!(
        vessel.contents, before.contents,
        "partial ice transfer committed"
    );
    assert_eq!(
        vessel.temperature, before.temperature,
        "partial heat committed"
    );
    assert_eq!(
        vessel.solution, before.solution,
        "partial chemistry committed"
    );
}

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
                source: None,
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
    /// Whether this stand-in reports an ion-interaction solvent activity,
    /// the way `pitzer.dat` does for a real brine.
    ///
    /// It matters to what is being tested. The 252 K eutectic boundary is
    /// about brine, and brine is routed to the one dataset that computes a
    /// solvent activity — so a fixture that reported none would exercise
    /// the IDEAL route's shallower ceiling and never reach 252 K at all.
    /// The activity here is the one an osmotic coefficient of exactly 1
    /// gives, which keeps the fixture's arithmetic readable while still
    /// taking the route a brine takes.
    ion_interaction: bool,
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
            solvent_activity: None,
            scope: Default::default(),
            solvent_kg: None,
            particle_molality: None,
            pe: None,
            redox: Vec::new(),
            ph: 7.0,
            ionic_strength: particle_molality / 2.0,
            species: {
                let mut species = vec![SpeciesDetail {
                    name: "test particles".to_string(),
                    molality: particle_molality,
                    activity: particle_molality,
                }];
                if self.ion_interaction {
                    species.push(SpeciesDetail {
                        name: "H2O".to_string(),
                        molality: 1.0 / 0.018_015,
                        activity: (-0.018_015 * particle_molality).exp(),
                    });
                }
                species
            },
            provenance: self
                .ion_interaction
                .then(|| kerotakis_core::vessel::Provenance {
                    engine: "particle-balance-test".to_string(),
                    dataset: "test".to_string(),
                    model: format!(
                        "{} stand-in for a brine dataset",
                        kerotakis_core::states::ION_INTERACTION_MODEL_PREFIX
                    ),
                    dataset_sources: Vec::new(),
                    routing: "fixture".to_string(),
                    routing_phrase: None,
                    dataset_phrase: None,
                    model_phrase: None,
                }),
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
        ion_interaction: true,
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

    // The solvent is not a solute: the `H2O` row this fixture reports is
    // where the water ACTIVITY comes from, and summing it into the particle
    // count would make the residual brine 55 molal. `dissolved_particles`
    // filters it for exactly that reason, and this assertion has to filter
    // it the same way or it is measuring a different solution.
    let particle_molality: f64 = vessel
        .solution
        .as_ref()
        .expect("residual brine remains a solution")
        .species
        .iter()
        .filter(|species| species.name != "H2O")
        .map(|species| species.molality)
        .sum();
    assert!(
        particle_molality > 0.0 && particle_molality < 5.0,
        "the residual brine is a brine, not the solvent counted as solute:          {particle_molality} mol/kgw"
    );
    // The vessel's OWN liquidus, on the route its own speciation earned.
    // Re-deriving it from the molality through `states::transitions` would
    // put an ideal-solution answer beside an ion-interaction one and call
    // the difference non-convergence — the fixture reports a solvent
    // activity precisely so this path is the brine path.
    let liquidus = kerotakis_core::solve::vessel_transitions(&vessel)
        .0
        .freezing_k;
    assert!(
        (vessel.temperature.0 - liquidus).abs() < 1e-12,
        "phase state {} K and re-solved liquidus {liquidus} K disagree",
        vessel.temperature.0
    );
    let final_reported_liquidus = events.iter().rev().find_map(|event| match event {
        Event::StateChanged { at, .. } => Some(at.0),
        _ => None,
    });
    assert_eq!(final_reported_liquidus, Some(liquidus));

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
        ion_interaction: true,
    };
    let mut coupled = PhaseEquilibrator::wrapping(Box::new(chemistry));
    // Start far enough below the liquidus that the available sensible heat
    // would freeze past the declared concentration boundary. At 240 K this
    // fixture instead reaches an ordinary, warmer partial-freezing balance.
    let mut vessel = partially_frozen_test_vessel(200.0);
    let before = vessel.clone();
    let events = coupled.equilibrate(&mut vessel).unwrap();

    assert!(water_phase_moles(&vessel, Phase::Liquid) > 0.0);
    assert_eq!(
        water_phase_moles(&vessel, Phase::Solid),
        water_phase_moles(&before, Phase::Solid)
    );
    // This entering fixture has no ice. Refusal must not create any.
    // The former temperature-only cap assertion created unbooked heat.
    assert_eq!(vessel.temperature, before.temperature);
    assert_eq!(
        serde_json::to_value(&vessel.contents).unwrap(),
        serde_json::to_value(&before.contents).unwrap()
    );
    assert_eq!(vessel.excess_enthalpy_j, before.excess_enthalpy_j);
    assert!(!events
        .iter()
        .any(|event| matches!(event, Event::StateChanged { .. })));

    assert!(events.iter().any(|event| matches!(
        event,
        Event::NotYetModeled { what, .. } if what.contains("partial-freezing model boundary")
    )));
}

/// Beyond the ideal activity cap, refuse with that route's own reason.
/// Preserve the entering energy and phase inventory rather than snapping to
/// the warmer cap liquidus. The 0.3 mol particle input starts within the
/// six-molal range, so this checks a root beyond the cap, not initial admission.
#[test]
fn the_ideal_route_stops_at_its_own_range_and_names_that_instead() {
    let chemistry = ParticleBalanceSolver {
        particle_moles: 0.3,
        calls: Rc::new(Cell::new(0)),
        ion_interaction: false,
    };
    let mut coupled = PhaseEquilibrator::wrapping(Box::new(chemistry));
    let mut vessel = partially_frozen_test_vessel(200.0);
    let before = vessel.clone();
    let events = coupled.equilibrate(&mut vessel).unwrap();

    assert!(water_phase_moles(&vessel, Phase::Liquid) > 0.0);
    assert_eq!(
        water_phase_moles(&vessel, Phase::Solid),
        water_phase_moles(&before, Phase::Solid)
    );
    // Preserve delivered cooling instead of forcing an unpriced liquidus.
    assert_eq!(vessel.temperature, before.temperature);
    assert_eq!(
        serde_json::to_value(&vessel.contents).unwrap(),
        serde_json::to_value(&before.contents).unwrap()
    );
    assert_eq!(vessel.excess_enthalpy_j, before.excess_enthalpy_j);
    assert!(!events
        .iter()
        .any(|event| matches!(event, Event::StateChanged { .. })));
    assert!(
        events.iter().any(|event| matches!(
            event,
            Event::NotYetModeled { what, .. }
                if what.contains("Raoult") && !what.contains("eutectic")
        )),
        "{events:?}"
    );
}

/// A solution that is ALREADY past the route's stated range gets no
/// transition at all, warm or cold.
///
/// This is the other half of the boundary and it was the untested half.
/// The test above watches a solution walk up to the ceiling and stop; this
/// one starts beyond it, where there is nothing to walk. The relation would
/// still return a number — it is a logarithm, it always returns a number —
/// and the whole point is that the number is not offered. 8 mol/kgw of
/// particles on Raoult's law is about a third out (see
/// `states::IDEAL_MAX_PARTICLE_MOLALITY`), and a third out printed to one
/// decimal place is the shape of an answer without the substance of one.
///
/// The refusal names the route and the range, and it fires because the
/// vessel would otherwise have CHANGED STATE — a syrup standing at room
/// temperature is not lectured about a ceiling it is nowhere near, which is
/// the second assertion.
#[test]
fn a_solution_past_the_stated_range_is_refused_a_transition_rather_than_given_one() {
    let mut coupled = PhaseEquilibrator::wrapping(Box::new(ParticleBalanceSolver {
        // 0.8 mol of particles in this fixture's 0.1 kg of water: 8 mol/kgw,
        // past the ideal route's stated 6.
        particle_moles: 0.8,
        calls: Rc::new(Cell::new(0)),
        ion_interaction: false,
    }));
    let mut vessel = partially_frozen_test_vessel(200.0);
    let events = coupled.equilibrate(&mut vessel).unwrap();

    assert_eq!(water_phase_moles(&vessel, Phase::Solid), 0.0);
    assert!(water_phase_moles(&vessel, Phase::Liquid) > 0.0);
    assert!(
        events.iter().any(|event| matches!(
            event,
            Event::NotYetModeled { what, .. }
                if what.contains("Raoult") && what.contains("stated range")
        )),
        "{events:?}"
    );

    // …and the same solution, standing where nothing was going to happen
    // anyway, is told nothing. A boundary that announces itself to every
    // vessel that merely contains a syrup is noise, and noise is how a real
    // refusal stops being read.
    let mut warm = PhaseEquilibrator::wrapping(Box::new(ParticleBalanceSolver {
        particle_moles: 0.8,
        calls: Rc::new(Cell::new(0)),
        ion_interaction: false,
    }));
    let mut standing = partially_frozen_test_vessel(298.15);
    let quiet = warm.equilibrate(&mut standing).unwrap();
    assert!(
        !quiet.iter().any(
            |event| matches!(event, Event::NotYetModeled { what, .. } if what.contains("Raoult"))
        ),
        "{quiet:?}"
    );
}

/// Sweep all original570 initial temperatures. An unsupported beyond-cap
/// root preserves the entering physical state; an interior solution reaches
/// its own liquidus. A genuinely closed exact-cap root may commit the boundary
/// with the existing typed retained-brine notice. Keep that success distinct
/// from the direct refusal, even if no gridpoint happens to land exactly on it.
/// The original205..262K range is retained, including both populated regimes.
#[test]
fn the_boundary_is_announced_only_when_the_brine_actually_reached_it() {
    let mut announced = 0;
    let mut coexisting = 0;
    for tenths in 2050..2620 {
        let start = f64::from(tenths) / 10.0;
        let mut coupled = PhaseEquilibrator::wrapping(Box::new(ParticleBalanceSolver {
            particle_moles: 0.6,
            calls: Rc::new(Cell::new(0)),
            ion_interaction: true,
        }));
        let mut vessel = partially_frozen_test_vessel(start);
        let before = vessel.clone();
        let events = coupled.equilibrate(&mut vessel).unwrap();
        let liquid = water_phase_moles(&vessel, Phase::Liquid);
        let ice = water_phase_moles(&vessel, Phase::Solid);
        assert!(liquid > 0.0, "{start} K: {liquid} / {ice}");

        let refused = events.iter().any(|event| matches!(event,
            Event::NotYetModeled { cause: ops::NotModelledCause::ModelBoundary, reason: Some(reason), .. }
                if reason.key == "not-modeled.partial-freezing-cap"));
        let committed_cap = events.iter().any(|event| matches!(event,
            Event::NotYetModeled { cause: ops::NotModelledCause::ModelBoundary, reason: Some(reason), .. }
                if reason.key == "not-modeled.brine-retained-at-boundary"));
        assert!(!(refused && committed_cap), "{start} K: {events:?}");
        if refused {
            // A refusal describes an unsupported root, not a fabricated
            // equilibrium. Keep the entering physical state and its energy.
            assert_eq!(liquid, water_phase_moles(&before, Phase::Liquid));
            assert_eq!(ice, water_phase_moles(&before, Phase::Solid));
            assert_eq!(vessel.temperature, before.temperature, "{start} K");
            assert_eq!(
                serde_json::to_value(&vessel.contents).unwrap(),
                serde_json::to_value(&before.contents).unwrap(),
                "{start} K"
            );
            assert_eq!(vessel.excess_enthalpy_j, before.excess_enthalpy_j);
            assert!(
                !events
                    .iter()
                    .any(|event| matches!(event, Event::StateChanged { .. })),
                "{start} K: {events:?}"
            );
            announced += 1;
        } else {
            assert!(ice > 0.0, "{start} K: supported freezing must create ice");
            let liquidus = kerotakis_core::solve::vessel_transitions(&vessel)
                .0
                .freezing_k;
            assert!(
                (vessel.temperature.0 - liquidus).abs() < 1e-6,
                "{start} K: settled at {} K, liquidus {liquidus} K",
                vessel.temperature.0
            );
            if committed_cap {
                assert!(
                    (vessel.temperature.0 - kerotakis_core::states::BRINE_MODEL_MIN_K).abs() < 1e-6,
                    "{start} K"
                );
                assert!(
                    events
                        .iter()
                        .any(|event| matches!(event, Event::StateChanged { .. })),
                    "{start} K"
                );
            } else {
                assert!(
                    vessel.temperature.0 > kerotakis_core::states::BRINE_MODEL_MIN_K,
                    "{start} K: silent at {} K",
                    vessel.temperature.0
                );
            }
            coexisting += 1;
        }
    }
    // Both halves of the sweep have to be populated or the invariant was
    // never tested: the cold end must refuse beyond the cap and the warm end must
    // stop short of it.
    assert!(
        announced > 0 && coexisting > 0,
        "{announced} / {coexisting}"
    );
}
