//! Additional-solvent selection must not hide malformed individual owners.
use kerotakis_core::solve::{Equilibrator, PermissiveScreen, SolveError};
use kerotakis_core::{
    Bench, Event, Kelvin, Moles, Operator, Phase, Portion, SpeciesId, Vessel, VesselId,
};

#[derive(Default)]
struct MutatingSolver {
    calls: usize,
}
impl Equilibrator for MutatingSolver {
    fn name(&self) -> &'static str {
        "ideal-owner-observer"
    }
    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.calls += 1;
        vessel.temperature = Kelvin(400.0);
        vessel.deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
        Ok(Vec::new())
    }
}

fn bench(portions: &[(&str, f64, Phase)]) -> Bench {
    let mut bench = Bench::new();
    bench.vessels.push(Vessel::new(VesselId(1), "beaker"));
    // Public persisted-owner shape: push individual entries without coalescing.
    for (species, amount, phase) in portions {
        bench
            .vessel_mut(VesselId(0))
            .unwrap()
            .contents
            .push(Portion {
                species: SpeciesId::new(*species),
                moles: Moles(*amount),
                phase: *phase,
            });
    }
    bench
}
fn state(bench: &Bench) -> serde_json::Value {
    let mut value = serde_json::to_value(bench).unwrap();
    value.as_object_mut().unwrap().remove("log");
    value
}
fn owned_bits(bench: &Bench) -> Vec<u64> {
    // JSON maps nonfinite values to null; retain their exact ownership bits too.
    bench
        .vessels
        .iter()
        .flat_map(|vessel| {
            vessel
                .contents
                .iter()
                .map(|portion| portion.moles.0.to_bits())
        })
        .collect()
}
fn distil() -> Operator {
    Operator::Distil {
        from: VesselId(0),
        to: VesselId(1),
        fraction: Some(0.1),
        energy: None,
        stages: 1,
    }
}

#[test]
fn malformed_individual_portions_refuse_before_route_loss_or_coalescing() {
    for invalid in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for phase in [Phase::Liquid, Phase::Aqueous] {
            for portions in [
                vec![("water", 1.0, Phase::Liquid), ("methanol", invalid, phase)],
                vec![("water", invalid, phase), ("methanol", 1.0, Phase::Liquid)],
                // At -1 the two methanol owners sum to +1; cancellation is forbidden.
                vec![
                    ("methanol", 2.0, Phase::Liquid),
                    ("methanol", invalid, phase),
                ],
            ] {
                let mut bench = bench(&portions);
                let before = state(&bench);
                let bits = owned_bits(&bench);
                let mut solver = MutatingSolver::default();
                let events = bench
                    .step_with(distil(), &mut solver, &PermissiveScreen)
                    .unwrap();
                assert_eq!(solver.calls, 0, "{portions:?}");
                assert_eq!(state(&bench), before);
                assert_eq!(owned_bits(&bench), bits);
                assert_eq!(bench.log.len(), 1);
                assert_eq!(events.len(), 1);
                assert!(matches!(&events[0],Event::NotYetModeled {
                    cause:kerotakis_core::ops::NotModelledCause::ModelBoundary,
                    reason:Some(reason),..
                } if reason.key=="not-modeled.ideal-still-invalid-input"));
            }
        }
    }
}

#[test]
fn zero_inactive_components_do_not_block_an_ordinary_additional_solvent_cut() {
    let mut bench = bench(&[
        ("methanol", 1.0, Phase::Liquid),
        ("water", 0.0, Phase::Liquid),
        ("isopropanol", 0.0, Phase::Aqueous),
    ]);
    let mut solver = MutatingSolver::default();
    let events = bench
        .step_with(distil(), &mut solver, &PermissiveScreen)
        .unwrap();
    assert_eq!(solver.calls, 2);
    assert!(events.iter().any(|event| matches!(event,Event::Distilled { components,.. }
        if components.iter().any(|(species,amount)| species.0=="methanol" && (amount.0-0.1).abs()<=1e-14))));
}

#[test]
fn binary_only_selection_and_zero_additional_coordinates_keep_existing_route() {
    use kerotakis_core::volatility::additional_solvent_cut;
    use kerotakis_thermo::vle::StillTake;
    for invalid in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let bench = bench(&[
            ("water", invalid, Phase::Liquid),
            ("ethanol", 1.0, Phase::Liquid),
            ("methanol", 0.0, Phase::Liquid),
        ]);
        let result = additional_solvent_cut(
            bench.vessel(VesselId(0)).unwrap(),
            StillTake::Fraction(0.1),
            1,
        );
        assert!(
            matches!(result, Ok(None)),
            "water/ethanol-only input must retain binary ownership: {result:?}"
        );
    }
}
