//! Source-informed failure injection for the production direct solver stack.
use kerotakis_core::{
    solve::{Equilibrator, SolveError, SolverRouteOutcome, SolverStack},
    vessel::StepStart,
    *,
};

fn water() -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    v.step_start = Some(StepStart::capture(&v));
    v
}
fn key(v: &Vessel) -> String {
    format!("{v:?}")
}
fn mark(v: &mut Vessel) {
    v.free_proton += 0.001;
    v.elapsed_seconds += 1.0;
    v.honesty_said.push("accepted trial marker".into());
    v.aqueous_routing_said = Some("trial route".into());
}
#[derive(Clone, Copy)]
enum Trial {
    Fail,
    Decline,
    Accept,
    Invalid,
    Identity,
    BadEvent,
}
impl Equilibrator for Trial {
    fn name(&self) -> &'static str {
        "atomic-trial"
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        mark(v);
        match self {
            Self::Fail | Self::Decline => {
                v.contents.clear();
                v.pressure = Pascal(f64::NAN);
                Err(SolveError::NotConverged {
                    solver: self.name().into(),
                    detail: "after mutation".into(),
                })
            }
            Self::Invalid => {
                v.temperature = Kelvin(f64::NAN);
                Ok(vec![gas(v.id)])
            }
            Self::Identity => {
                v.id = VesselId(99);
                Ok(vec![])
            }
            Self::BadEvent => Ok(vec![gas(VesselId(99))]),
            Self::Accept => Ok(vec![]),
        }
    }
    fn mix(
        &mut self,
        v: &mut Vessel,
        _: &Vessel,
        _: f64,
        _: &Vessel,
        _: f64,
    ) -> Option<Result<Vec<Event>, SolveError>> {
        if matches!(self, Self::Decline) {
            mark(v);
            v.contents.clear();
            v.pressure = Pascal(f64::NAN);
            None
        } else {
            Some(self.equilibrate(v))
        }
    }
}
fn gas(vessel: VesselId) -> Event {
    Event::GasEvolved {
        vessel,
        species: SpeciesId::new("water"),
        moles: Moles(0.001),
    }
}
struct Probe {
    expected: String,
}
impl Equilibrator for Probe {
    fn name(&self) -> &'static str {
        "accepted-checkpoint-probe"
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        assert_eq!(
            key(v),
            self.expected,
            "next route must see only accepted state"
        );
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

#[test]
fn failed_direct_route_restores_complete_vessel_and_reports_failure() {
    let mut v = water();
    let before = key(&v);
    let mut stack = SolverStack::new(vec![Box::new(Trial::Fail)]);
    let events = stack.equilibrate(&mut v).unwrap();
    assert_eq!(key(&v), before);
    assert!(matches!(&events[..], [Event::SolverFailed { .. }]));
    assert!(matches!(
        stack.last_routes[0].outcome,
        SolverRouteOutcome::Failed
    ));
}
#[test]
fn next_direct_route_observes_the_accepted_checkpoint() {
    let mut v = water();
    let before = key(&v);
    let mut stack = SolverStack::new(vec![
        Box::new(Trial::Fail),
        Box::new(Probe {
            expected: before.clone(),
        }),
    ]);
    stack.equilibrate(&mut v).unwrap();
    assert_eq!(key(&v), before);
}
#[test]
fn earlier_success_survives_later_direct_failure() {
    let mut v = water();
    let mut expected = v.clone();
    mark(&mut expected);
    let mut stack = SolverStack::new(vec![Box::new(Trial::Accept), Box::new(Trial::Fail)]);
    stack.equilibrate(&mut v).unwrap();
    assert_eq!(key(&v), key(&expected));
}
#[test]
fn failed_mix_restores_complete_vessel() {
    let mut v = water();
    let before = key(&v);
    let source = water();
    let mut stack = SolverStack::new(vec![Box::new(Trial::Fail)]);
    assert!(stack
        .mix(&mut v, &source, 0.5, &source, 0.5)
        .unwrap()
        .is_err());
    assert_eq!(key(&v), before);
}
#[test]
fn declined_mix_restores_state_before_the_next_route() {
    let mut v = water();
    let before = key(&v);
    let source = water();
    let mut stack = SolverStack::new(vec![
        Box::new(Trial::Decline),
        Box::new(Probe {
            expected: before.clone(),
        }),
    ]);
    assert!(stack
        .mix(&mut v, &source, 0.5, &source, 0.5)
        .unwrap()
        .is_ok());
    assert_eq!(key(&v), before);
}
#[test]
fn all_declined_mix_routes_leave_no_trial_state() {
    let mut v = water();
    let before = key(&v);
    let source = water();
    let mut stack = SolverStack::new(vec![Box::new(Trial::Decline), Box::new(Trial::Decline)]);
    assert!(stack.mix(&mut v, &source, 0.5, &source, 0.5).is_none());
    assert_eq!(key(&v), before);
}
#[test]
fn supported_direct_route_preserves_all_accepted_metadata() {
    let mut v = water();
    let mut expected = v.clone();
    mark(&mut expected);
    let mut stack = SolverStack::new(vec![Box::new(Trial::Accept)]);
    assert!(stack.equilibrate(&mut v).unwrap().is_empty());
    assert_eq!(key(&v), key(&expected));
}
#[test]
fn supported_mix_route_preserves_all_accepted_metadata() {
    let mut v = water();
    let mut expected = v.clone();
    mark(&mut expected);
    let source = water();
    let mut stack = SolverStack::new(vec![Box::new(Trial::Accept)]);
    assert!(stack
        .mix(&mut v, &source, 0.5, &source, 0.5)
        .unwrap()
        .unwrap()
        .is_empty());
    assert_eq!(key(&v), key(&expected));
}
#[test]
fn invalid_successful_direct_result_discards_state_and_trial_events() {
    let mut v = water();
    let before = key(&v);
    let mut stack = SolverStack::new(vec![Box::new(Trial::Invalid)]);
    let events = stack.equilibrate(&mut v).unwrap();
    assert!(matches!(&events[..], [Event::SolverFailed { .. }]));
    assert_eq!(key(&v), before);
}
#[test]
fn invalid_successful_mix_result_discards_state_and_trial_events() {
    let mut v = water();
    let before = key(&v);
    let source = water();
    let mut stack = SolverStack::new(vec![Box::new(Trial::Invalid)]);
    assert!(stack
        .mix(&mut v, &source, 0.5, &source, 0.5)
        .unwrap()
        .is_err());
    assert_eq!(key(&v), before);
}
#[test]
fn solver_cannot_replace_the_target_vessel_identity() {
    let mut v = water();
    let before = key(&v);
    let mut stack = SolverStack::new(vec![Box::new(Trial::Identity)]);
    assert!(matches!(
        &stack.equilibrate(&mut v).unwrap()[..],
        [Event::SolverFailed { .. }]
    ));
    assert_eq!(key(&v), before);
}
#[test]
fn rejected_gas_event_cannot_enter_the_step_boundary_ledger() {
    let mut v = water();
    let before = key(&v);
    let mut stack = SolverStack::new(vec![Box::new(Trial::BadEvent)]);
    assert!(matches!(
        &stack.equilibrate(&mut v).unwrap()[..],
        [Event::SolverFailed { .. }]
    ));
    assert_eq!(key(&v), before);
}

struct OverflowEvent;
impl Equilibrator for OverflowEvent {
    fn name(&self) -> &'static str {
        "boundary-overflow"
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        Ok(vec![Event::GasEvolved {
            vessel: v.id,
            species: SpeciesId::new("water"),
            moles: Moles(f64::MAX),
        }])
    }
}
#[test]
fn accumulated_step_gas_overflow_refuses_the_entire_attempt() {
    let mut v = water();
    v.step_start
        .as_mut()
        .unwrap()
        .note_gas_out(&SpeciesId::new("water"), f64::MAX);
    let before = key(&v);
    let mut stack = SolverStack::new(vec![Box::new(OverflowEvent)]);
    assert!(matches!(
        &stack.equilibrate(&mut v).unwrap()[..],
        [Event::SolverFailed { .. }]
    ));
    assert_eq!(key(&v), before);
}
