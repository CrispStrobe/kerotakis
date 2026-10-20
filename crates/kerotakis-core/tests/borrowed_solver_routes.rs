//! Borrowed adapters must obey the installing stack's material policy.
use kerotakis_core::*;

struct UnbalancedProposal;
impl Equilibrator for UnbalancedProposal {
    fn name(&self) -> &'static str {
        "unbalanced-borrowed-test-proposal"
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        vessel.deposit(SpeciesId::new("NaCl"), Moles(0.25), Phase::Solid);
        Ok(vec![])
    }
}

#[test]
fn borrowed_adapter_cannot_bypass_required_material_conservation() {
    let mut vessel = Vessel::new(VesselId(0), "borrowed route control");
    vessel.deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
    let before = format!("{vessel:?}");
    let mut stack = SolverStack::with_required_conservation(vec![], 1e-10).unwrap();
    let events = stack.equilibrate_external(&mut UnbalancedProposal, &mut vessel);
    assert_eq!(format!("{vessel:?}"), before);
    assert!(
        matches!(&events[..], [Event::SolverFailed { solver, .. }] if solver == "unbalanced-borrowed-test-proposal")
    );
    assert_eq!(stack.last_routes.len(), 1);
    assert!(matches!(
        stack.last_routes[0].outcome,
        SolverRouteOutcome::Failed
    ));
}
