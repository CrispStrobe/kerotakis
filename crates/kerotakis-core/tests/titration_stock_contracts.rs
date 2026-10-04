//! Only accepted physical doses debit finite shelf inventory.
mod common {
    include!("common/titration_accounting.rs");
}
use common::*;
use kerotakis_core::*;
fn run(b: &mut Bench, target: f64, max: u32) -> Vec<Event> {
    b.step_with(
        op(10.0, 0.001, target, max),
        &mut Model::default(),
        &PermissiveScreen,
    )
    .unwrap()
}
#[test]
fn full_dose_debits_titrant_and_carrier() {
    let mut b = bench();
    stock(&mut b, "NaOH", 0.1);
    let w = water_dose();
    stock(&mut b, "water", w * 4.0);
    let e = run(&mut b, 30.0, 1);
    assert!((remaining(&b, "NaOH") - 0.09).abs() < 1e-14);
    assert!((remaining(&b, "water") - w * 3.0).abs() < 1e-14);
    journal(&b, &e);
}
#[test]
fn refined_dose_charges_only_selected_fraction_even_when_full_trial_exceeds_stock() {
    let mut b = bench();
    stock(&mut b, "NaOH", 0.0075);
    let w = water_dose();
    stock(&mut b, "water", w * 0.75);
    let e = run(&mut b, 7.0, 1);
    assert!((remaining(&b, "NaOH") - 0.0025).abs() < 1e-14);
    assert!((remaining(&b, "water") - w * 0.25).abs() < 1e-14);
    assert!(e.iter().any(|e| matches!(
        e,
        Event::Titrated {
            endpoint_reached: Some(true),
            ..
        }
    )));
    journal(&b, &e);
}
#[test]
fn exhausted_titrant_preserves_complete_physical_bench() {
    let mut b = bench();
    stock(&mut b, "NaOH", 0.005);
    let before = physical(&b);
    let e = run(&mut b, 30.0, 1);
    assert_eq!(physical(&b), before);
    assert!(e
        .iter()
        .any(|e| matches!(e,Event::StockExhausted{key,..} if key=="NaOH")));
    journal(&b, &e);
}
#[test]
fn exhausted_carrier_rolls_back_affordable_titrant_debit() {
    let mut b = bench();
    stock(&mut b, "NaOH", 0.1);
    stock(&mut b, "water", water_dose() / 2.0);
    let before = physical(&b);
    let e = run(&mut b, 30.0, 1);
    assert_eq!(physical(&b), before);
    assert!(e
        .iter()
        .any(|e| matches!(e,Event::StockExhausted{key,..} if key=="water")));
    journal(&b, &e);
}
#[test]
fn later_exhaustion_keeps_only_previous_dose_and_debits() {
    let mut b = bench();
    stock(&mut b, "NaOH", 0.015);
    let w = water_dose();
    stock(&mut b, "water", w * 3.0);
    let e = run(&mut b, 30.0, 3);
    assert_eq!(b.vessels[0].moles_of(&SpeciesId::new("NaOH")).0, 0.01);
    assert!((remaining(&b, "NaOH") - 0.005).abs() < 1e-14);
    assert!((remaining(&b, "water") - w * 2.0).abs() < 1e-14);
    assert!(e
        .iter()
        .any(|e| matches!(e, Event::Titrated { steps: 1, .. })));
    journal(&b, &e);
}
#[test]
fn water_titrant_combines_carrier_and_titrant_into_one_stock_request() {
    let mut b = bench();
    let w = water_dose();
    stock(&mut b, "water", w + 0.005);
    let before = physical(&b);
    let mut o = op(10.0, 0.001, 30.0, 1);
    if let Operator::Titrate { titrant, .. } = &mut o {
        *titrant = SpeciesId::new("water");
    }
    let e = b
        .step_with(o, &mut Model::default(), &PermissiveScreen)
        .unwrap();
    assert_eq!(physical(&b), before);
    assert!(e.iter().any(|e|matches!(e,Event::StockExhausted{key,requested,..} if key=="water" && (*requested-(w+0.01)).abs()<1e-14)));
    journal(&b, &e);
}
#[test]
fn invalid_solver_and_safety_veto_do_not_consume_stock() {
    for fail in [false, true] {
        let mut b = bench();
        stock(&mut b, "NaOH", 0.1);
        stock(&mut b, "water", 1.0);
        let before = b.stock.clone();
        struct Veto;
        impl SafetyScreen for Veto {
            fn assess(&self, _: &Vessel) -> SafetyVerdict {
                SafetyVerdict::Veto {
                    reason: "contract".into(),
                }
            }
        }
        let mut m = Model {
            fail,
            ..Model::default()
        };
        let s: &dyn SafetyScreen = if fail { &PermissiveScreen } else { &Veto };
        b.step_with(op(10.0, 0.001, 30.0, 1), &mut m, s).unwrap();
        assert_eq!(b.stock, before);
    }
}
#[test]
fn untracked_supplies_stay_unlimited_independently() {
    for key in ["NaOH", "water"] {
        let mut b = bench();
        stock(&mut b, key, 1.0);
        run(&mut b, 30.0, 1);
        assert!(remaining(&b, key) < 1.0);
        let other = if key == "NaOH" { "water" } else { "NaOH" };
        assert!(b.stock.remaining(other).is_none());
    }
}
#[test]
fn zero_steps_and_already_reached_endpoint_do_not_draw_stock() {
    for (target, max) in [(30.0, 0), (3.0, 3)] {
        let mut b = bench();
        stock(&mut b, "NaOH", 0.1);
        stock(&mut b, "water", 1.0);
        let before = physical(&b);
        run(&mut b, target, max);
        assert_eq!(physical(&b), before);
    }
}
#[test]
fn zero_debit_against_huge_titrant_stock_refuses_current_dose() {
    let mut b = bench();
    stock(&mut b, "NaOH", 1e20);
    let before = physical(&b);
    let e = run(&mut b, 30.0, 1);
    assert_eq!(physical(&b), before);
    assert!(e.iter().any(|e| matches!(e, Event::NotYetModeled { .. })));
    journal(&b, &e);
}
