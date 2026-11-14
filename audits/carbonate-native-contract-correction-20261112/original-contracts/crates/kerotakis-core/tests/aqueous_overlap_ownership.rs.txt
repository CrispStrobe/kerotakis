//! Mechanism controls for typed route ownership. These synthetic answers do
//! not validate carbonate chemistry or a native aqueous calculation.

use kerotakis_core::aqueous_continuation::{CurrentAqueousAnswer, TrustedAqueousReadback};
use kerotakis_core::curated::CuratedEquilibrator;
use kerotakis_core::*;
use std::cell::Cell;
use std::rc::Rc;

fn stock() -> Vessel {
    let mut vessel = Vessel::new(VesselId(0), "aqueous overlap mechanism");
    vessel.deposit(SpeciesId::new("water"), Moles(5.0), Phase::Liquid);
    vessel.deposit(SpeciesId::new("HCO3-"), Moles(0.001), Phase::Aqueous);
    vessel.deposit(SpeciesId::new("CH3COOH"), Moles(0.001), Phase::Aqueous);
    vessel
}

fn characterized(vessel: &mut Vessel, scope: SolutionScope) {
    vessel.solution = Some(SolutionInfo {
        scope,
        solvent_kg: Some(0.09),
        particle_molality: Some(0.02),
        pe: None,
        redox: vec![],
        ph: 6.0,
        ionic_strength: 0.02,
        species: vec![],
        provenance: Some(Provenance {
            engine: "synthetic ownership control".into(),
            dataset: "none".into(),
            model: "no chemical claim".into(),
            dataset_sources: vec![],
            routing: "mechanism only".into(),
            routing_phrase: None,
            dataset_phrase: None,
            model_phrase: None,
        }),
        solvent_activity: None,
    });
}

#[derive(Clone, Copy)]
enum Answer {
    Complete,
    Partial,
    NestedFailure,
    NoOp,
    Fail,
}

struct Trusted {
    available: Rc<Cell<bool>>,
    answer: Answer,
}

impl Trusted {
    fn new(answer: Answer) -> Self {
        Self {
            available: Rc::new(Cell::new(true)),
            answer,
        }
    }
}

impl Equilibrator for Trusted {
    fn name(&self) -> &'static str {
        "synthetic-trusted-owner"
    }

    fn trusted_aqueous_readback(&mut self) -> Option<&mut dyn TrustedAqueousReadback> {
        Some(self)
    }

    fn applies(&self, _vessel: &Vessel) -> bool {
        self.available.get()
    }

    // Deliberately more permissive than `applies` to verify that selection
    // requires both the route's applicability and chemistry claim.
    fn chemistry_applies(&self, _vessel: &Vessel) -> bool {
        true
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        match self.answer {
            Answer::Complete => characterized(vessel, SolutionScope::Complete),
            Answer::Partial => characterized(vessel, SolutionScope::SolventOnly),
            Answer::NestedFailure => {
                characterized(vessel, SolutionScope::Complete);
                return Ok(vec![Event::SolverFailed {
                    reason: None,
                    vessel: vessel.id,
                    solver: "nested synthetic phase".into(),
                    detail: "controlled nested failure".into(),
                }]);
            }
            Answer::NoOp => {}
            Answer::Fail => {
                return Err(SolveError::NotConverged {
                    reason: None,
                    solver: self.name().into(),
                    detail: "controlled owner failure".into(),
                });
            }
        }
        Ok(vec![])
    }
}

impl TrustedAqueousReadback for Trusted {
    fn characterize_current(
        &mut self,
        vessel: &mut Vessel,
    ) -> Result<(Vec<Event>, CurrentAqueousAnswer), SolveError> {
        let input = vessel.clone();
        let events = self.equilibrate(vessel)?;
        let answer = CurrentAqueousAnswer::attest_computed(&input, vessel)?;
        Ok((events, answer))
    }
}

struct ZeroSizedOwner;

impl Equilibrator for ZeroSizedOwner {
    fn name(&self) -> &'static str {
        "zero-sized-owner"
    }

    fn trusted_aqueous_readback(&mut self) -> Option<&mut dyn TrustedAqueousReadback> {
        Some(self)
    }

    fn equilibrate(&mut self, _vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        panic!("an unidentifiable borrowed owner must not execute")
    }
}

impl TrustedAqueousReadback for ZeroSizedOwner {
    fn characterize_current(
        &mut self,
        _vessel: &mut Vessel,
    ) -> Result<(Vec<Event>, CurrentAqueousAnswer), SolveError> {
        panic!("an unidentifiable borrowed owner must not characterize")
    }
}

struct DropEligibility(Rc<Cell<bool>>);

impl Equilibrator for DropEligibility {
    fn name(&self) -> &'static str {
        "drop-later-eligibility"
    }

    fn chemistry_applies(&self, _vessel: &Vessel) -> bool {
        false
    }

    fn equilibrate(&mut self, _vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.0.set(false);
        Ok(vec![])
    }
}

fn reacted(events: &[Event]) -> bool {
    events.iter().any(|event| {
        matches!(
            event,
            Event::ReactionOccurred { equation, .. } if equation.starts_with("HCO₃⁻ + CH₃COOH")
        )
    })
}

fn failed(events: &[Event]) -> bool {
    events
        .iter()
        .any(|event| matches!(event, Event::SolverFailed { .. }))
}

#[test]
fn direct_and_no_owner_stacks_keep_the_original_curated_fallback() {
    let mut direct = stock();
    let mut curated = CuratedEquilibrator;
    assert!(reacted(&curated.equilibrate(&mut direct).unwrap()));
    let mut without_owner = SolverStack::new(vec![Box::new(CuratedEquilibrator)]);
    let mut without_owner_stock = stock();
    let events = without_owner.equilibrate(&mut without_owner_stock).unwrap();
    assert!(reacted(&events));
    assert!(!failed(&events));

    let unavailable = Trusted::new(Answer::Complete);
    unavailable.available.set(false);
    let mut stack = SolverStack::new(vec![Box::new(CuratedEquilibrator), Box::new(unavailable)]);
    let mut unavailable_stock = stock();
    assert!(reacted(&stack.equilibrate(&mut unavailable_stock).unwrap()));
}

#[test]
fn an_untagged_bicarbonate_bottle_row_still_runs_before_an_aqueous_owner() {
    let mut vessel = Vessel::new(VesselId(0), "untagged curated mechanism");
    vessel.deposit(SpeciesId::new("water"), Moles(5.0), Phase::Liquid);
    vessel.deposit(SpeciesId::new("NaHCO3"), Moles(0.001), Phase::Solid);
    vessel.deposit(SpeciesId::new("CH3COOH"), Moles(0.001), Phase::Aqueous);
    let mut stack = SolverStack::new(vec![
        Box::new(CuratedEquilibrator),
        Box::new(Trusted::new(Answer::Complete)),
    ]);
    let events = stack.equilibrate(&mut vessel).unwrap();
    assert!(events.iter().any(|event| matches!(
        event,
        Event::ReactionOccurred { equation, .. } if equation.starts_with("NaHCO₃ + CH₃COOH")
    )));
    assert!(!failed(&events));
}

#[test]
fn selected_complete_owner_gets_only_tagged_row_and_preserves_stock() {
    let mut vessel = stock();
    let before = vessel.contents.clone();
    let mut stack = SolverStack::new(vec![
        Box::new(CuratedEquilibrator),
        Box::new(Trusted::new(Answer::Complete)),
    ]);
    let events = stack.equilibrate(&mut vessel).unwrap();
    assert!(!reacted(&events));
    assert!(!failed(&events));
    assert_eq!(vessel.contents, before);
    assert_eq!(
        vessel.solution.as_ref().unwrap().scope,
        SolutionScope::Complete
    );
    assert!(matches!(
        stack.last_routes[0].outcome,
        SolverRouteOutcome::NotApplicable
    ));
    assert!(stack.last_routes[0]
        .reason
        .as_ref()
        .unwrap()
        .contains("deferred"));
    assert!(matches!(
        stack.last_routes[1].outcome,
        SolverRouteOutcome::Succeeded { .. }
    ));
    assert!(!stack.aqueous_overlap_refused());
}

#[test]
fn selected_owner_failure_decline_or_partial_answer_does_not_consume_stock() {
    for answer in [Answer::Fail, Answer::Partial, Answer::NestedFailure] {
        let mut vessel = stock();
        let before = vessel.contents.clone();
        let mut stack = SolverStack::new(vec![
            Box::new(CuratedEquilibrator),
            Box::new(Trusted::new(answer)),
        ]);
        let events = stack.equilibrate(&mut vessel).unwrap();
        assert!(failed(&events));
        assert!(!reacted(&events));
        assert_eq!(vessel.contents, before);
        assert!(vessel.solution.is_none());
        assert!(matches!(
            stack.last_routes[1].outcome,
            SolverRouteOutcome::Failed
        ));
        assert!(stack.aqueous_overlap_refused());
    }

    let owner = Trusted::new(Answer::Complete);
    let available = owner.available.clone();
    let mut stack = SolverStack::new(vec![
        Box::new(CuratedEquilibrator),
        Box::new(DropEligibility(available)),
        Box::new(owner),
    ]);
    let mut vessel = stock();
    let before = vessel.contents.clone();
    let events = stack.equilibrate(&mut vessel).unwrap();
    assert!(failed(&events));
    assert!(!reacted(&events));
    assert_eq!(vessel.contents, before);
    assert!(vessel.solution.is_none());
}

#[test]
fn a_failed_selected_owner_stops_later_installed_routes() {
    let observed = Rc::new(Cell::new(true));
    let mut stack = SolverStack::new(vec![
        Box::new(CuratedEquilibrator),
        Box::new(Trusted::new(Answer::Fail)),
        Box::new(DropEligibility(observed.clone())),
    ]);
    let mut vessel = stock();
    assert!(failed(&stack.equilibrate(&mut vessel).unwrap()));
    assert!(stack.aqueous_overlap_refused());
    assert!(observed.get(), "the later route must not execute");
}

#[test]
fn a_selected_no_op_cannot_inherit_an_old_complete_answer() {
    let mut vessel = stock();
    characterized(&mut vessel, SolutionScope::Complete);
    let before = vessel.contents.clone();
    let mut stack = SolverStack::new(vec![
        Box::new(CuratedEquilibrator),
        Box::new(Trusted::new(Answer::NoOp)),
    ]);
    let events = stack.equilibrate(&mut vessel).unwrap();
    assert!(failed(&events));
    assert!(!reacted(&events));
    assert_eq!(vessel.contents, before);
    assert!(vessel.solution.is_none());
    assert!(matches!(
        stack.last_routes[1].outcome,
        SolverRouteOutcome::Failed
    ));
    assert!(stack.aqueous_overlap_refused());
}

#[test]
fn ambiguous_owners_fail_without_choosing_one() {
    let mut vessel = stock();
    let before = vessel.contents.clone();
    let mut stack = SolverStack::new(vec![
        Box::new(CuratedEquilibrator),
        Box::new(Trusted::new(Answer::Complete)),
        Box::new(Trusted::new(Answer::Complete)),
    ]);
    let events = stack.equilibrate(&mut vessel).unwrap();
    assert!(failed(&events));
    assert!(!reacted(&events));
    assert_eq!(vessel.contents, before);
    assert!(stack.last_routes.iter().any(|route| {
        route.solver == "aqueous-overlap-ownership"
            && matches!(route.outcome, SolverRouteOutcome::Failed)
    }));
    assert!(stack.aqueous_overlap_refused());
}

#[test]
fn borrowed_owner_must_be_the_selected_instance_and_cannot_leak_to_next_range() {
    let mut stack = SolverStack::new(vec![
        Box::new(CuratedEquilibrator),
        Box::new(DropEligibility(Rc::new(Cell::new(true)))),
    ]);
    let mut owner = Trusted::new(Answer::Complete);
    let mut vessel = stock();
    let before = vessel.contents.clone();
    stack.begin_equilibration();
    let before_events = stack
        .equilibrate_range_with_external_aqueous(0..1, &mut vessel, &mut owner)
        .unwrap();
    assert!(!reacted(&before_events));
    assert!(matches!(
        stack.last_routes[0].outcome,
        SolverRouteOutcome::NotApplicable
    ));
    let owned_events = stack.equilibrate_external(&mut owner, &mut vessel);
    assert!(!failed(&owned_events));
    assert_eq!(vessel.contents, before);
    assert_eq!(
        vessel.solution.as_ref().unwrap().scope,
        SolutionScope::Complete
    );
    assert!(!failed(
        &stack.equilibrate_range(1..2, &mut vessel).unwrap()
    ));

    let mut wrong = Trusted::new(Answer::Complete);
    let mut second = stock();
    stack.begin_equilibration();
    stack
        .equilibrate_range_with_external_aqueous(0..1, &mut second, &mut owner)
        .unwrap();
    assert!(failed(&stack.equilibrate_external(&mut wrong, &mut second)));
    assert!(stack.aqueous_overlap_refused());
    assert!(second.solution.is_none());
    assert!(!failed(
        &stack.equilibrate_range(1..2, &mut second).unwrap()
    ));

    let mut third = stock();
    stack.begin_equilibration();
    assert!(!stack.aqueous_overlap_refused());
    stack
        .equilibrate_range_with_external_aqueous(0..1, &mut third, &mut owner)
        .unwrap();
    assert!(failed(&stack.equilibrate_range(1..2, &mut third).unwrap()));
    assert!(!failed(&stack.equilibrate_range(1..2, &mut third).unwrap()));

    let mut old_answer = stock();
    characterized(&mut old_answer, SolutionScope::Complete);
    let stock_before = old_answer.contents.clone();
    let mut no_op = Trusted::new(Answer::NoOp);
    stack.begin_equilibration();
    stack
        .equilibrate_range_with_external_aqueous(0..1, &mut old_answer, &mut no_op)
        .unwrap();
    assert!(failed(
        &stack.equilibrate_external(&mut no_op, &mut old_answer)
    ));
    assert_eq!(old_answer.contents, stock_before);
    assert!(old_answer.solution.is_none());
}

#[test]
fn a_zero_sized_borrowed_owner_cannot_claim_instance_identity() {
    let mut stack = SolverStack::new(vec![Box::new(CuratedEquilibrator)]);
    let mut owner = ZeroSizedOwner;
    let mut vessel = stock();
    let before = vessel.contents.clone();
    stack.begin_equilibration();
    let events = stack
        .equilibrate_range_with_external_aqueous(0..1, &mut vessel, &mut owner)
        .unwrap();
    assert!(failed(&events));
    assert!(stack.aqueous_overlap_refused());
    assert!(!reacted(&events));
    assert_eq!(vessel.contents, before);
}
