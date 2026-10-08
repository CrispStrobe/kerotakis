//! Supplemental compatibility forecasts, frozen before their production guards.
use kerotakis_core::*;

#[derive(Default)]
struct CountingSolver(usize);
impl Equilibrator for CountingSolver {
    fn name(&self) -> &'static str {
        "missing-vessel-contract"
    }
    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.0 += 1;
        vessel.temperature.0 += 1.0;
        Ok(vec![])
    }
}
fn missing(op: Operator) {
    let mut bench = Bench::new();
    let before = serde_json::to_value(&bench).unwrap();
    let mut solver = CountingSolver::default();
    let result = bench.step_with(op, &mut solver, &PermissiveScreen);
    assert!(matches!(result, Err(BenchError::NoSuchVessel(VesselId(9)))));
    assert_eq!(solver.0, 0);
    assert_eq!(serde_json::to_value(&bench).unwrap(), before);
}
#[test]
fn unsupported_nuclide_on_missing_vessel_retains_exception() {
    missing(Operator::SpikeNuclide {
        vessel: VesselId(9),
        nuclide: "Xe-135".into(),
        moles: Moles(0.001),
    });
}
#[test]
fn unknown_reaction_on_missing_vessel_retains_exception() {
    missing(Operator::React {
        vessel: VesselId(9),
        reaction: "__unsupported_refusal_contract__".into(),
    });
}
