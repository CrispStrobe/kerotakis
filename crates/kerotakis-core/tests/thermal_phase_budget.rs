//! Independent calorimetric budgets must distinguish fusion from sensible heat.
use kerotakis_core::*;

fn water(amount: f64, phase: Phase, temperature: f64) -> Bench {
    let mut bench = Bench::new();
    bench.vessels[0].deposit(SpeciesId::new("water"), Moles(amount), phase);
    bench.vessels[0].temperature = Kelvin(temperature);
    bench
}

fn transfer(events: &[Event]) -> (f64, f64, Option<bool>) {
    events
        .iter()
        .find_map(|event| match event {
            Event::EnergyTransferred {
                delivered_j,
                sensible_j,
                energy_partition_complete,
                ..
            } => Some((*delivered_j, *sensible_j, *energy_partition_complete)),
            _ => None,
        })
        .expect("thermal transfer account")
}

fn near(actual: f64, expected: f64, scale: f64) {
    assert!(
        (actual - expected).abs() <= 1e-7 * scale,
        "actual {actual}, expected {expected}, scale {scale}"
    );
}

#[test]
fn half_freeze_and_half_thaw_price_committed_fusion_across_scales() {
    for amount in [0.1, 0.5, 2.0] {
        for heating in [false, true] {
            let initial_phase = if heating { Phase::Solid } else { Phase::Liquid };
            let initial_t = if heating { 263.15 } else { 298.15 };
            let mut bench = water(amount, initial_phase, initial_t);
            let sensible = bench.vessels[0].energy_between(
                initial_t.min(states::WATER_FREEZING_K),
                initial_t.max(states::WATER_FREEZING_K),
            );
            let fusion = 0.5 * amount * states::WATER_H_FUS;
            let dose = sensible + fusion;
            let op = if heating {
                Operator::Heat {
                    vessel: VesselId(0),
                    energy: Joules(dose),
                    source: None,
                }
            } else {
                Operator::Cool {
                    vessel: VesselId(0),
                    energy: Joules(dose),
                }
            };
            let events = bench
                .step_with(op, &mut StateEquilibrator, &PermissiveScreen)
                .unwrap();
            assert!(!events
                .iter()
                .any(|event| matches!(event, Event::SolverFailed { .. })));
            let vessel = &bench.vessels[0];
            near(vessel.temperature.0, states::WATER_FREEZING_K, 300.0);
            for phase in [Phase::Solid, Phase::Liquid] {
                let n: f64 = vessel
                    .contents
                    .iter()
                    .filter(|p| p.phase == phase)
                    .map(|p| p.moles.0)
                    .sum();
                near(n, amount * 0.5, amount);
            }
            let (delivered, reported_sensible, complete) = transfer(&events);
            assert_eq!(complete, Some(true), "heating={heating}, amount={amount}");
            near(delivered, dose, dose);
            near(reported_sensible, sensible, dose);
            near(delivered - reported_sensible, fusion, dose);
        }
    }
}

#[test]
fn supported_water_sensible_cycle_has_no_fabricated_latent_remainder() {
    let mut bench = water(0.5, Phase::Liquid, 298.15);
    let dose = bench.vessels[0].energy_between(298.15, 303.15);
    for heating in [true, false] {
        let op = if heating {
            Operator::Heat {
                vessel: VesselId(0),
                energy: Joules(dose),
                source: None,
            }
        } else {
            Operator::Cool {
                vessel: VesselId(0),
                energy: Joules(dose),
            }
        };
        let events = bench
            .step_with(op, &mut StateEquilibrator, &PermissiveScreen)
            .unwrap();
        let (delivered, sensible, complete) = transfer(&events);
        assert_eq!(complete, Some(true));
        near(delivered, dose, dose);
        near(sensible, dose, dose);
    }
    near(bench.vessels[0].temperature.0, 298.15, 300.0);
}

#[test]
fn supported_partial_freeze_then_thaw_restores_pure_water_inventory_and_energy() {
    let mut bench = water(0.5, Phase::Liquid, 298.15);
    let sensible = bench.vessels[0].energy_between(states::WATER_FREEZING_K, 298.15);
    let fusion = 0.25 * states::WATER_H_FUS;
    let dose = sensible + fusion;
    for heating in [false, true] {
        let op = if heating {
            Operator::Heat {
                vessel: VesselId(0),
                energy: Joules(dose),
                source: None,
            }
        } else {
            Operator::Cool {
                vessel: VesselId(0),
                energy: Joules(dose),
            }
        };
        let events = bench
            .step_with(op, &mut StateEquilibrator, &PermissiveScreen)
            .unwrap();
        let (delivered, reported_sensible, complete) = transfer(&events);
        assert_eq!(complete, Some(true));
        near(delivered, dose, dose);
        near(reported_sensible, sensible, dose);
        near(delivered - reported_sensible, fusion, dose);
    }
    near(bench.vessels[0].temperature.0, 298.15, 300.0);
    assert!(bench.vessels[0]
        .contents
        .iter()
        .all(|p| p.phase == Phase::Liquid));
    near(
        bench.vessels[0].moles_of(&SpeciesId::new("water")).0,
        0.5,
        0.5,
    );
}

// Bench correctly invalidates cached speciation before a thermal step. This
// fixture recomputes the analytical particle census on each solver call, then
// exercises the actual activity-domain and phase-boundary implementation.
// The ideal solvent route is intentional; no empirical ionic activity is fitted.
struct CountedSaltPhase;
impl Equilibrator for CountedSaltPhase {
    fn name(&self) -> &'static str {
        "counted-salt-phase-test"
    }
    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        let water_kg: f64 = vessel
            .contents
            .iter()
            .filter(|p| p.species.0 == "water" && p.phase == Phase::Liquid)
            .map(|p| p.moles.0 * constants::WATER_MOLAR_MASS_KG_PER_MOL)
            .sum();
        if water_kg > 0.0 {
            let species: Vec<_> = ["Na+", "Cl-"]
                .into_iter()
                .map(|name| {
                    let molality = vessel.moles_of(&SpeciesId::new(name)).0 / water_kg;
                    SpeciesDetail {
                        name: name.into(),
                        molality,
                        activity: molality,
                    }
                })
                .collect();
            vessel.solution = Some(SolutionInfo {
                scope: Default::default(),
                solvent_kg: Some(water_kg),
                pe: None,
                redox: Vec::new(),
                ph: 7.0,
                ionic_strength: species.iter().map(|s| 0.5 * s.molality).sum(),
                species,
                provenance: None,
                solvent_activity: None,
            });
        }
        StateEquilibrator.equilibrate(vessel)
    }
}

#[test]
fn unsupported_brine_cooling_and_boiling_restore_complete_checkpoint() {
    for heating in [false, true] {
        let mut bench = water(2.0, Phase::Liquid, 298.15);
        // Both cases cross an actual solvent-model boundary. Deep cooling
        // concentrates a dilute brine past its activity/eutectic cap; boiling
        // starts with a salt stock already beyond the fitted activity range.
        let ions = if heating { 1.0 } else { 0.02 };
        bench.vessels[0].deposit(SpeciesId::new("Na+"), Moles(ions), Phase::Aqueous);
        bench.vessels[0].deposit(SpeciesId::new("Cl-"), Moles(ions), Phase::Aqueous);
        let before = serde_json::to_value(&bench).unwrap();
        let op = if heating {
            Operator::Heat {
                vessel: VesselId(0),
                energy: Joules(100_000.0),
                source: None,
            }
        } else {
            Operator::Cool {
                vessel: VesselId(0),
                energy: Joules(30_000.0),
            }
        };
        let error = bench
            .step_with(op, &mut CountedSaltPhase, &PermissiveScreen)
            .unwrap_err();
        assert!(matches!(error, BenchError::InvalidState(_)));
        assert_eq!(serde_json::to_value(&bench).unwrap(), before);
    }
}

#[test]
fn counted_brine_inside_phase_domain_keeps_small_cooling_and_inventory() {
    let mut bench = water(2.0, Phase::Liquid, 298.15);
    for species in ["Na+", "Cl-"] {
        bench.vessels[0].deposit(SpeciesId::new(species), Moles(0.02), Phase::Aqueous);
    }
    let contents = bench.vessels[0].contents.clone();
    let events = bench
        .step_with(
            Operator::Cool {
                vessel: VesselId(0),
                energy: Joules(10.0),
            },
            &mut CountedSaltPhase,
            &PermissiveScreen,
        )
        .unwrap();
    assert_eq!(bench.vessels[0].contents, contents);
    assert!(bench.vessels[0].temperature.0 < 298.15);
    near(transfer(&events).0, 10.0, 10.0);
    assert_eq!(transfer(&events).2, Some(false));
}

struct MissingChemistry;
impl Equilibrator for MissingChemistry {
    fn name(&self) -> &'static str {
        "missing-budget-test"
    }
    fn equilibrate(&mut self, _: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        Err(SolveError::NotConverged {
            solver: self.name().into(),
            detail: "unrelated chemistry unavailable".into(),
        })
    }
}

#[test]
fn failed_or_unpriced_chemistry_is_not_a_complete_partition_certificate() {
    for unpriced in [false, true] {
        let mut bench = water(0.5, Phase::Liquid, 298.15);
        if unpriced {
            bench.vessels[0].unpriced_heat.push(SpeciesId::new("water"));
        }
        let op = Operator::Cool {
            vessel: VesselId(0),
            energy: Joules(10.0),
        };
        let events = if unpriced {
            bench.step_with(op, &mut StateEquilibrator, &PermissiveScreen)
        } else {
            bench.step_with(op, &mut MissingChemistry, &PermissiveScreen)
        }
        .unwrap();
        assert_eq!(transfer(&events).2, Some(false));
        assert!(bench.vessels[0].temperature.0 < 298.15);
    }
}

#[test]
fn disclosed_absolute_zero_cooling_cap_still_commits() {
    let mut bench = Bench::new();
    bench.vessels[0].deposit(SpeciesId::new("Fe"), Moles(0.5), Phase::Solid);
    let events = bench
        .step_with(
            Operator::Cool {
                vessel: VesselId(0),
                energy: Joules(100_000.0),
            },
            &mut StateEquilibrator,
            &PermissiveScreen,
        )
        .unwrap();
    assert_eq!(bench.vessels[0].temperature.0, 0.0);
    assert!(events.iter().any(|event| matches!(event,
        Event::NotYetModeled { reason: Some(reason), .. }
        if reason.key == "not-modeled.cooling-below-absolute-zero")));
    assert!(transfer(&events).0 < 100_000.0);
}
