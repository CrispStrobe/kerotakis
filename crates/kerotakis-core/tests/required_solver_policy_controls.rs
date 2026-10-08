//! Source-informed policy controls added after the initial port, before execution.
use kerotakis_core::*;
struct Route(Option<f64>, bool);
impl Equilibrator for Route {
    fn name(&self) -> &'static str {
        "policy-control"
    }
    fn element_conservation_tolerance(&self) -> Option<f64> {
        self.0
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        v.temperature = Kelvin(310.0);
        if self.1 {
            v.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
        }
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
fn water() -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
    v
}
#[test]
fn invalid_route_policy_refuses_direct_and_mix_even_inside_required_stack() {
    for tolerance in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 1.0, 2.0] {
        for required in [false, true] {
            let mut stack = if required {
                SolverStack::with_required_conservation(
                    vec![Box::new(Route(Some(tolerance), false))],
                    1e-8,
                )
                .unwrap()
            } else {
                SolverStack::new(vec![Box::new(Route(Some(tolerance), false))])
            };
            let mut v = water();
            let before = format!("{v:?}");
            let events = stack.equilibrate(&mut v).unwrap();
            assert!(matches!(&events[..], [Event::SolverFailed { .. }]));
            assert_eq!(format!("{v:?}"), before);
            let source = water();
            assert!(stack
                .mix(&mut v, &source, 0.5, &source, 0.5)
                .unwrap()
                .is_err());
            assert_eq!(format!("{v:?}"), before);
        }
    }
}
#[test]
fn exact_route_policy_accepts_conserving_warmth_and_rejects_created_matter() {
    for create in [false, true] {
        let mut v = water();
        let before = format!("{v:?}");
        let mut stack =
            SolverStack::with_required_conservation(vec![Box::new(Route(Some(0.0), create))], 1e-8)
                .unwrap();
        let events = stack.equilibrate(&mut v).unwrap();
        if create {
            assert!(matches!(&events[..], [Event::SolverFailed { .. }]));
            assert_eq!(format!("{v:?}"), before);
        } else {
            assert!(events.is_empty());
            assert_eq!(v.temperature, Kelvin(310.0));
        }
    }
}
#[test]
fn legacy_constructor_keeps_explicit_opt_out_and_enforces_opt_in() {
    for policy in [None, Some(1e-8)] {
        let mut v = water();
        let before = format!("{v:?}");
        let mut stack = SolverStack::new(vec![Box::new(Route(policy, true))]);
        let events = stack.equilibrate(&mut v).unwrap();
        if policy.is_some() {
            assert!(matches!(&events[..], [Event::SolverFailed { .. }]));
            assert_eq!(format!("{v:?}"), before);
        } else {
            assert!(events.is_empty());
            assert_eq!(v.moles_of(&SpeciesId::new("water")), Moles(3.0));
        }
    }
}
