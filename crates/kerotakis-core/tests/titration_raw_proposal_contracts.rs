//! Separate raw-state preparation boundaries, frozen before titration repair.
mod common { include!("common/titration_accounting.rs"); }
use common::*;
use kerotakis_core::*;
use std::cell::RefCell;
#[derive(Default)] struct Raw { seen:RefCell<Vec<Vessel>> }
impl SafetyScreen for Raw {
    fn assess(&self,_:&Vessel)->SafetyVerdict { panic!("use contextual pour") }
    fn assess_pour(&self,_:&Vessel,after:&Vessel)->SafetyVerdict {
        self.seen.borrow_mut().push(after.clone());SafetyVerdict::Allow
    }
}
#[test]
fn raw_titration_screen_uses_owned_pressure_at_actual_mixed_temperature() {
    let mut b=bench();b.vessels[0].temperature=Kelvin(340.0);
    b.vessels[0].headspace=Headspace::Sealed { volume:Liters(0.5) };
    b.vessels[0].deposit(SpeciesId::new("N2"),Moles(0.01),Phase::Gas);b.vessels[0].refresh_pressure();
    let screen=Raw::default();b.step_with(op(10.0,0.001,30.0,1),&mut Model::default(),&screen).unwrap();
    let seen=screen.seen.borrow();assert_eq!(seen.len(),1);let mut expected=seen[0].clone();expected.refresh_pressure();
    assert_eq!(seen[0].pressure,expected.pressure);
}
#[test]
fn raw_titration_screen_does_not_reuse_initial_solution_characterization() {
    let mut b=bench();let screen=Raw::default();
    b.step_with(op(10.0,0.001,30.0,1),&mut Model::default(),&screen).unwrap();
    assert!(screen.seen.borrow()[0].solution.is_none());
}
#[test]
fn raw_titration_screen_invalidates_resolved_cache_with_old_solution() {
    let mut b=bench();b.vessels[0].resolved.valid=true;b.vessels[0].resolved.solution=b.vessels[0].solution.clone();let screen=Raw::default();
    b.step_with(op(10.0,0.001,30.0,1),&mut Model::default(),&screen).unwrap();
    let seen=screen.seen.borrow();assert!(!seen[0].resolved.valid);assert!(seen[0].resolved.solution.is_none());
}
