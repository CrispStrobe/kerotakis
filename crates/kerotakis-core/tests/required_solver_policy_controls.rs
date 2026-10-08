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

// A boundary pair distinguishes actual tolerance precedence from unconditional
// refusal of newly created matter. The hydrogen residual is about 5e-5 of the
// changed throughput: inside the stack's 1e-3 policy, outside the route's 1e-6.
#[test]
fn tighter_route_tolerance_changes_acceptance_of_the_same_reaction_in_direct_and_mix() {
    struct Reaction(Option<f64>);
    impl Equilibrator for Reaction {
        fn name(&self) -> &'static str {
            "tolerance-boundary-reaction"
        }
        fn element_conservation_tolerance(&self) -> Option<f64> {
            self.0
        }
        fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
            v.contents.clear();
            v.deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
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
    for native_mix in [false, true] {
        for route in [None, Some(1e-6), Some(1e-2)] {
            let mut v = Vessel::new(VesselId(0), "reaction");
            v.deposit(SpeciesId::new("H2"), Moles(2.0001), Phase::Gas);
            v.deposit(SpeciesId::new("O2"), Moles(1.0), Phase::Gas);
            let source = v.clone();
            let before = format!("{v:?}");
            let mut stack =
                SolverStack::with_required_conservation(vec![Box::new(Reaction(route))], 1e-3)
                    .unwrap();
            let accepted = if native_mix {
                stack
                    .mix(&mut v, &source, 0.5, &source, 0.5)
                    .unwrap()
                    .is_ok()
            } else {
                !stack
                    .equilibrate(&mut v)
                    .unwrap()
                    .iter()
                    .any(|event| matches!(event, Event::SolverFailed { .. }))
            };
            assert_eq!(
                accepted,
                route != Some(1e-6),
                "route={route:?}, mix={native_mix}"
            );
            if accepted {
                assert_eq!(v.contents.len(), 1);
                assert_eq!(v.moles_of(&SpeciesId::new("water")), Moles(2.0));
            } else {
                assert_eq!(format!("{v:?}"), before);
            }
        }
    }
}
