//! Required conservation policy for stacks containing untrusted custom routes.
use kerotakis_core::*;
struct CreateWater;
impl Equilibrator for CreateWater {
    fn name(&self) -> &'static str {
        "create-water"
    }
    fn element_conservation_tolerance(&self) -> Option<f64> {
        None
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        v.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
        Ok(vec![])
    }
    fn mix(
        &mut self,
        v: &mut Vessel,
        _: &Vessel,
        _: f64,
        _: &Vessel,
        _: f64,
    ) -> Option<Result<Vec<Event>, SolveError>> {
        Some(self.equilibrate(v))
    }
}
struct Warm;
impl Equilibrator for Warm {
    fn name(&self) -> &'static str {
        "warm"
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        v.temperature = Kelvin(310.0);
        Ok(vec![])
    }
}
fn water() -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
    v
}
#[test]
fn opting_out_route_cannot_bypass_required_stack_policy() {
    let mut v = water();
    let before = serde_json::to_value(&v).unwrap();
    let mut s = SolverStack::with_required_conservation(vec![Box::new(CreateWater)], 1e-8).unwrap();
    let e = s.equilibrate(&mut v).unwrap();
    assert_eq!(serde_json::to_value(v).unwrap(), before);
    assert!(e
        .iter()
        .any(|e| matches!(e,Event::SolverFailed {solver,..} if solver=="create-water")));
}
#[test]
fn failed_custom_stage_does_not_block_conserving_later_stage() {
    let mut v = water();
    let mut s =
        SolverStack::with_required_conservation(vec![Box::new(CreateWater), Box::new(Warm)], 1e-8)
            .unwrap();
    let e = s.equilibrate(&mut v).unwrap();
    assert_eq!(v.temperature, Kelvin(310.0));
    assert_eq!(v.contents[0].moles, Moles(2.0));
    assert!(e.iter().any(|e| matches!(e, Event::SolverFailed { .. })));
}
#[test]
fn custom_mix_cannot_bypass_required_conservation() {
    let mut v = water();
    let before = serde_json::to_value(&v).unwrap();
    let a = water();
    let b = water();
    let mut s = SolverStack::with_required_conservation(vec![Box::new(CreateWater)], 1e-8).unwrap();
    assert!(s.mix(&mut v, &a, 0.5, &b, 0.5).unwrap().is_err());
    assert_eq!(serde_json::to_value(v).unwrap(), before);
}
#[test]
fn invalid_required_tolerances_refuse_construction() {
    for t in [-1.0, 0.0, f64::NAN, f64::INFINITY, 1.0] {
        assert!(SolverStack::with_required_conservation(vec![], t).is_err());
    }
}
#[test]
fn required_policy_keeps_the_more_stringent_route_tolerance() {
    struct Small;
    impl Equilibrator for Small {
        fn name(&self) -> &'static str {
            "small"
        }
        fn element_conservation_tolerance(&self) -> Option<f64> {
            Some(1e-12)
        }
        fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
            v.deposit(SpeciesId::new("water"), Moles(1e-9), Phase::Liquid);
            Ok(vec![])
        }
    }
    let mut v = water();
    let mut s = SolverStack::with_required_conservation(vec![Box::new(Small)], 1e-6).unwrap();
    let e = s.equilibrate(&mut v).unwrap();
    assert_eq!(v.contents[0].moles, Moles(2.0));
    assert!(e.iter().any(|e| matches!(e, Event::SolverFailed { .. })));
}
