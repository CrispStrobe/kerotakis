//! Source-informed final-acceptance contracts for the special titration path.
//! Earlier accepted doses survive a later veto; provisional trials do not.
mod common { include!("common/titration_accounting.rs"); }
use common::*;
use kerotakis_core::*;
use std::cell::RefCell;

#[derive(Default)]
struct Warm { inputs: Vec<Vessel> }
impl Equilibrator for Warm {
    fn name(&self) -> &'static str { "titration-final-model" }
    fn equilibrate(&mut self,v:&mut Vessel)->Result<Vec<Event>,SolveError> {
        self.inputs.push(v.clone()); v.temperature=Kelvin(330.0);
        let ph=3.0+800.0*v.moles_of(&SpeciesId::new("NaOH")).0;
        v.solution=Some(serde_json::from_value(serde_json::json!({"ph":ph,"ionic_strength":0.01})).unwrap());
        Ok(vec![])
    }
}
struct Screen { mode:u8, seen:RefCell<Vec<(Vessel,Vessel)>> }
impl Screen { fn new(mode:u8)->Self { Self { mode,seen:RefCell::new(vec![]) } } }
fn warning(rule:String)->SafetyVerdict { SafetyVerdict::Warn { severity:Severity::Danger,rule,hazard:"final proposal warning".into(),real_world:"frozen control".into() } }
impl SafetyScreen for Screen {
    fn assess(&self,_:&Vessel)->SafetyVerdict { SafetyVerdict::Allow }
    fn assess_pour(&self,_:&Vessel,_:&Vessel)->SafetyVerdict {
        if self.mode==5 { warning("shared-warning".into()) } else { SafetyVerdict::Allow }
    }
    fn assess_equilibrated(&self,before:&Vessel,after:&Vessel)->SafetyVerdict {
        self.seen.borrow_mut().push((before.clone(),after.clone()));
        let n=after.moles_of(&SpeciesId::new("NaOH")).0;
        if self.mode==1 || self.mode==2 && n>0.015 || self.mode==3 && (0.004..0.006).contains(&n) {
            SafetyVerdict::Veto { reason:"final titration veto".into() }
        } else if self.mode==4 { warning(format!("final-{n:.6}")) }
        else if self.mode==5 { warning("shared-warning".into()) }
        else { SafetyVerdict::Allow }
    }
}
fn stocked()->Bench { let mut b=bench();stock(&mut b,"NaOH",0.1);stock(&mut b,"water",1.0);b }

#[test]
fn first_final_veto_preserves_flask_and_stock_with_only_veto_journal() {
    let mut b=stocked();let before=physical(&b);let mut model=Warm::default();let s=Screen::new(1);
    let e=b.step_with(op(10.0,0.001,30.0,1),&mut model,&s).unwrap();
    assert_eq!(model.inputs.len(),1);assert_eq!(s.seen.borrow().len(),1);
    assert_eq!(physical(&b),before);assert!(matches!(e.as_slice(),[Event::SafetyVeto { .. }]));journal(&b,&e);
}
#[test]
fn later_final_veto_retains_only_prior_accepted_increment_and_stock_debit() {
    let mut expected=stocked();expected.step_with(op(10.0,0.001,30.0,1),&mut Warm::default(),&Screen::new(0)).unwrap();
    let mut b=stocked();let mut model=Warm::default();let s=Screen::new(2);
    let e=b.step_with(op(10.0,0.001,30.0,3),&mut model,&s).unwrap();
    assert_eq!(physical(&b),physical(&expected));assert_eq!(model.inputs.len(),2);assert_eq!(s.seen.borrow().len(),2);
    assert!(e.iter().any(|e|matches!(e,Event::SafetyVeto { .. })));
    assert!(e.iter().any(|e|matches!(e,Event::Titrated { steps:1,.. })));journal(&b,&e);
}
#[test]
fn final_veto_in_refinement_discards_provisional_full_and_fractional_doses() {
    let mut b=stocked();let before=physical(&b);let mut model=Warm::default();let s=Screen::new(3);
    let e=b.step_with(op(10.0,0.001,7.0,1),&mut model,&s).unwrap();
    assert_eq!(model.inputs.len(),2);assert_eq!(s.seen.borrow().len(),2);
    assert_eq!(physical(&b),before);assert!(matches!(e.as_slice(),[Event::SafetyVeto { .. }]));journal(&b,&e);
}
#[test]
fn hook_observes_each_exact_pre_solver_and_settled_virtual_state() {
    let mut b=stocked();let mut model=Warm::default();let s=Screen::new(0);
    b.step_with(op(10.0,0.001,7.0,1),&mut model,&s).unwrap();
    let seen=s.seen.borrow();assert_eq!(seen.len(),2);assert_eq!(model.inputs.len(),2);
    for ((before,after),input) in seen.iter().zip(model.inputs.iter()) {
        assert_eq!(serde_json::to_value(before).unwrap(),serde_json::to_value(input).unwrap());
        assert!(before.solution.is_none());assert!(after.solution.is_some());assert_eq!(after.temperature,Kelvin(330.0));
    }
}
#[test]
fn discarded_virtual_final_warnings_are_not_committed() {
    let mut b=stocked();let s=Screen::new(4);
    let e=b.step_with(op(10.0,0.001,7.0,1),&mut Warm::default(),&s).unwrap();
    let rules:Vec<_>=e.iter().filter_map(|e|if let Event::HazardWarning { rule,.. }=e {Some(rule.as_str())}else{None}).collect();
    assert_eq!(rules,vec!["final-0.005000"]);assert_eq!(s.seen.borrow().len(),2);journal(&b,&e);
}
#[test]
fn stock_refusal_discards_staged_final_warning_and_physical_proposal() {
    let mut b=stocked();stock(&mut b,"NaOH",0.005);let before=physical(&b);let s=Screen::new(4);
    let e=b.step_with(op(10.0,0.001,30.0,1),&mut Warm::default(),&s).unwrap();
    assert_eq!(s.seen.borrow().len(),1);assert_eq!(physical(&b),before);
    assert!(!e.iter().any(|e|matches!(e,Event::HazardWarning { .. })));journal(&b,&e);
}
#[test]
fn same_rule_raw_and_final_warning_is_committed_once() {
    let mut b=stocked();let s=Screen::new(5);
    let e=b.step_with(op(10.0,0.001,30.0,1),&mut Warm::default(),&s).unwrap();
    assert_eq!(s.seen.borrow().len(),1);
    assert_eq!(e.iter().filter(|e|matches!(e,Event::HazardWarning { rule,.. } if rule=="shared-warning")).count(),1);
}
#[test]
fn zero_budget_and_already_reached_endpoint_have_no_final_trial_hook() {
    for (target,max) in [(30.0,0),(3.0,10)] {
        let mut b=stocked();let before=physical(&b);let s=Screen::new(1);let mut model=Warm::default();
        b.step_with(op(10.0,0.001,target,max),&mut model,&s).unwrap();
        assert_eq!(physical(&b),before);assert!(s.seen.borrow().is_empty());assert!(model.inputs.is_empty());
    }
}
