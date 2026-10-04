//! Accepted progress is counted from committed doses, not initial pH availability.
mod common {
    include!("common/titration_accounting.rs");
}
use common::*;
use kerotakis_core::*;
#[test]
fn first_characterized_dose_is_counted_when_initial_ph_is_unavailable() {
    let mut b = bench();
    b.vessels[0].solution = None;
    stock(&mut b, "NaOH", 0.1);
    let mut m = Model::default();
    let e = b
        .step_with(op(10.0, 0.001, 30.0, 1), &mut m, &PermissiveScreen)
        .unwrap();
    assert_eq!(m.calls, 1);
    assert!(e.iter().any(|e|matches!(e,Event::Titrated{steps:1,total_volume,final_ph,curve,..} if total_volume.0==0.001 && *final_ph==11.0 && curve.len()==1)),"{e:?}");
    assert!((remaining(&b, "NaOH") - 0.09).abs() < 1e-14);
    journal(&b, &e);
}
