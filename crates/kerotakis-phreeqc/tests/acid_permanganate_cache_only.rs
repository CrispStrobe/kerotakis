#![cfg(not(feature = "engine"))]
//! A missing actual initial question cannot authorize an endpoint or alter stock.
use kerotakis_core::vessel::StepStart;
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;
use std::{cell::Cell, rc::Rc};

#[test]
fn cache_miss_and_failed_hook_leave_complete_endpoint_stock_unchanged() {
    let mut v = Vessel::new(VesselId(0), "uncached bounded endpoint stock");
    for (key, n, phase) in [
        ("water", 5.55, Phase::Liquid),
        ("H2SO4", 0.006, Phase::Liquid),
        ("KMnO4", 0.002, Phase::Solid),
        ("H2O2", 0.005, Phase::Liquid),
    ] {
        v.deposit(SpeciesId::new(key), Moles(n), phase);
    }
    v.step_start = Some(StepStart::capture(&v));
    let before = format!("{v:?}");
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    assert!(displacement::over(&mut solver, &mut v).is_err());
    assert_eq!(format!("{v:?}"), before);
    let calls = Rc::new(Cell::new(0));
    let observed = Rc::clone(&calls);
    solver.set_hook(Box::new(move |_, _| {
        observed.set(observed.get() + 1);
        Err("controlled endpoint initial-question failure".into())
    }));
    assert!(displacement::over(&mut solver, &mut v).is_err());
    assert!(calls.get() > 0);
    assert_eq!(format!("{v:?}"), before);
}
