//! Additional-solvent numerical boundaries must not become false domain claims.
use kerotakis_core::solve::{Equilibrator, PermissiveScreen, SolveError};
use kerotakis_core::{
    Bench, Event, Kelvin, Moles, Operator, Pascal, Phase, SpeciesId, Vessel, VesselId,
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
    let donor = bench
        .vessels
        .iter_mut()
        .find(|vessel| vessel.id == VesselId(0))
        .unwrap();
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
fn additional_solvent_refusals_keep_categories_and_unchanged_state() {
    for (water, methanol, fraction, stages, pressure, key) in [
        (1e100, 1e-300, 0.1, 1, 101325.0, "composition-precision"),
        (0.0, 1e-300, 1e-30, 1, 101325.0, "request-precision"),
        (0.0, 1e308, 0.5, 1, 101325.0, "energy-precision"),
        (0.0, 1.0, 0.1, 1, 1e12, "phase-evaluation"),
        (0.0, 1.0, 0.1, 0, 101325.0, "invalid-input"),
    ] {
        let mut bench = bench(water, 0.0, pressure);
        bench
            .vessels
            .iter_mut()
            .find(|vessel| vessel.id == VesselId(0))
            .unwrap()
            .deposit(SpeciesId::new("methanol"), Moles(methanol), Phase::Liquid);
        let before = state_without_log(&bench);
        let mut solver = MutatingSolver::default();
        let events = bench
            .step_with(
                Operator::Distil {
                    from: VesselId(0),
                    to: VesselId(1),
                    fraction: Some(fraction),
                    energy: None,
                    stages,
                },
                &mut solver,
                &PermissiveScreen,
            )
            .unwrap();
        assert_eq!(solver.calls, 0);
        assert_eq!(state_without_log(&bench), before);
        assert_eq!(bench.log.len(), 1);
        assert_eq!(events.len(), 1);
        assert!(
            matches!(&events[0],Event::NotYetModeled { cause:kerotakis_core::ops::NotModelledCause::ModelBoundary,reason:Some(reason),.. } if reason.key==format!("not-modeled.ideal-still-{key}")),
            "{key}: {events:?}"
        );
    }
}

#[test]
fn supported_pure_methanol_cut_retains_transfer_and_downstream_processing() {
    let mut bench = bench(0.0, 0.0, 101325.0);
    bench
        .vessels
        .iter_mut()
        .find(|vessel| vessel.id == VesselId(0))
        .unwrap()
        .deposit(SpeciesId::new("methanol"), Moles(1.0), Phase::Liquid);
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
    assert_eq!(solver.calls, 2);
    assert!(events.iter().any(|event| matches!(event,Event::Distilled { components,.. } if components.iter().any(|(species,amount)| species.0=="methanol" && (amount.0-0.1).abs()<=1e-14))));
}
