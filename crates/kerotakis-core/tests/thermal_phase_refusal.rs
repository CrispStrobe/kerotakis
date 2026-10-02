//! Failed chemistry cannot turn an unaccepted sensible proposal into hot ice.
use kerotakis_core::*;

struct FailingPhase;
impl Equilibrator for FailingPhase {
    fn name(&self) -> &'static str {
        "unavailable-phase-test"
    }
    fn equilibrate(&mut self, _: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        Err(SolveError::NotConverged {
            solver: self.name().into(),
            detail: "no phase answer".into(),
        })
    }
}

fn bench(moles: f64, phase: Phase) -> Bench {
    let mut bench = Bench::new();
    let vessel = bench.vessel_mut(VesselId(0)).unwrap();
    vessel.deposit(SpeciesId::new("water"), Moles(moles), phase);
    vessel.deposit(SpeciesId::new("N2"), Moles(0.001), Phase::Gas);
    vessel.temperature = Kelvin(263.15);
    vessel.headspace = kerotakis_core::vessel::Headspace::Sealed {
        volume: Liters(0.1),
    };
    vessel.refresh_pressure();
    bench
}

#[test]
fn failed_heat_with_superheated_ice_rolls_back_complete_bench_across_scales() {
    for amount in [0.05, 0.5, 5.0] {
        let mut bench = bench(amount, Phase::Solid);
        let before = serde_json::to_value(&bench).unwrap();
        let error = bench
            .step_with(
                Operator::Heat {
                    vessel: VesselId(0),
                    energy: Joules(amount * 10_000.0),
                    source: None,
                },
                &mut FailingPhase,
                &PermissiveScreen,
            )
            .unwrap_err();
        assert!(matches!(error, BenchError::InvalidState(_)));
        assert_eq!(serde_json::to_value(&bench).unwrap(), before);
    }
}

#[test]
fn unrelated_chemistry_failure_still_allows_sensible_liquid_heat() {
    let mut bench = bench(0.5, Phase::Liquid);
    let events = bench
        .step_with(
            Operator::Heat {
                vessel: VesselId(0),
                energy: Joules(50.0),
                source: None,
            },
            &mut FailingPhase,
            &PermissiveScreen,
        )
        .unwrap();
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::SolverFailed { .. })));
    assert!(bench.vessel(VesselId(0)).unwrap().temperature.0 > 263.15);
}

#[test]
fn working_phase_solver_melts_ice_and_commits_heat() {
    let mut bench = bench(0.5, Phase::Solid);
    let events = bench
        .step_with(
            Operator::Heat {
                vessel: VesselId(0),
                energy: Joules(5_000.0),
                source: None,
            },
            &mut StateEquilibrator,
            &PermissiveScreen,
        )
        .unwrap();
    assert!(!events
        .iter()
        .any(|event| matches!(event, Event::SolverFailed { .. })));
    let vessel = bench.vessel(VesselId(0)).unwrap();
    assert!(vessel
        .contents
        .iter()
        .all(|portion| portion.species.0 != "water" || portion.phase != Phase::Solid));
    assert!(vessel.temperature.0 > states::WATER_FREEZING_K);
}
