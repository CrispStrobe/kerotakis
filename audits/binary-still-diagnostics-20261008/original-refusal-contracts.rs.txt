//! Independent operator expectations: refusal categories and unchanged ownership.
use kerotakis_core::solve::{Equilibrator, PermissiveScreen, SolveError};
use kerotakis_core::{
    Bench, Event, Joules, Kelvin, Moles, Operator, Pascal, Phase, SpeciesId, Vessel, VesselId,
};

#[derive(Default)]
struct MutatingSolver {
    calls: usize,
}

impl Equilibrator for MutatingSolver {
    fn name(&self) -> &'static str {
        "still-refusal-observer"
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.calls += 1;
        vessel.temperature = Kelvin(400.0);
        vessel.deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
        Ok(Vec::new())
    }
}

fn bench(water: f64, ethanol: f64, pressure_pa: f64) -> Bench {
    let mut bench = Bench::new();
    bench.vessels.push(Vessel::new(VesselId(1), "beaker"));
    let donor = bench.vessel_mut(VesselId(0)).unwrap();
    donor.deposit(SpeciesId::new("water"), Moles(water), Phase::Liquid);
    donor.deposit(SpeciesId::new("ethanol"), Moles(ethanol), Phase::Liquid);
    donor.pressure = Pascal(pressure_pa);
    bench
}

fn state_without_log(bench: &Bench) -> serde_json::Value {
    let mut value = serde_json::to_value(bench).unwrap();
    value.as_object_mut().unwrap().remove("log");
    value
}

#[test]
fn binary_refusals_keep_independent_categories_and_skip_all_solver_mutation() {
    // Categories are declared here, not obtained by querying the kernel.
    for (water, ethanol, fraction, energy, stages, pressure, key) in [
        (
            1e-300,
            1e100,
            Some(0.1),
            None,
            4,
            101325.0,
            "composition-precision",
        ),
        (
            1e-300,
            1e-300,
            Some(1e-30),
            None,
            4,
            101325.0,
            "request-precision",
        ),
        (1e308, 0.0, Some(0.5), None, 4, 101325.0, "energy-precision"),
        (1.0, 0.0, Some(0.1), None, 4, 1e12, "phase-evaluation"),
        (1.0, 0.0, Some(0.1), None, 129, 101325.0, "invalid-input"),
        (
            1.0,
            0.0,
            None,
            Some(Joules(f64::NAN)),
            4,
            101325.0,
            "invalid-input",
        ),
    ] {
        let mut bench = bench(water, ethanol, pressure);
        let before = state_without_log(&bench);
        let mut solver = MutatingSolver::default();
        let events = bench
            .step_with(
                Operator::Distil {
                    from: VesselId(0),
                    to: VesselId(1),
                    fraction,
                    energy,
                    stages,
                },
                &mut solver,
                &PermissiveScreen,
            )
            .unwrap();
        assert_eq!(
            solver.calls, 0,
            "{key}: refused operations must skip all solvers"
        );
        assert_eq!(
            state_without_log(&bench),
            before,
            "{key}: owned state and caches must not change"
        );
        assert_eq!(bench.log.len(), 1, "attempts remain logged");
        let reasons: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                Event::NotYetModeled {
                    cause,
                    reason: Some(reason),
                    ..
                } => Some((cause, reason)),
                _ => None,
            })
            .collect();
        assert_eq!(reasons.len(), 1);
        assert_eq!(
            *reasons[0].0,
            kerotakis_core::ops::NotModelledCause::ModelBoundary
        );
        assert_eq!(reasons[0].1.key, format!("not-modeled.still-{key}"));
        assert!(!events
            .iter()
            .any(|event| matches!(event, Event::Distilled { .. })));
    }
}

#[test]
fn ordinary_binary_cut_still_transfers_and_runs_downstream_solvers() {
    let mut bench = bench(1.0, 0.0, 101325.0);
    let mut solver = MutatingSolver::default();
    let events = bench
        .step_with(
            Operator::Distil {
                from: VesselId(0),
                to: VesselId(1),
                fraction: Some(0.1),
                energy: None,
                stages: 1,
            },
            &mut solver,
            &PermissiveScreen,
        )
        .unwrap();
    assert_eq!(
        solver.calls, 2,
        "both accepted-operation vessels are settled"
    );
    assert!(events.iter().any(
        |event| matches!(event, Event::Distilled { water, ethanol, .. }
        if (water.0 - 0.1).abs() <= 1e-14 && ethanol.0 == 0.0)
    ));
    assert_eq!(
        bench.vessel(VesselId(0)).unwrap().temperature,
        Kelvin(400.0)
    );
    assert_eq!(
        bench.vessel(VesselId(1)).unwrap().temperature,
        Kelvin(400.0)
    );
}
