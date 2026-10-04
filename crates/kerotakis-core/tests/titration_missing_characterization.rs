//! An old initial chart point is not a final measurement after an uncharacterized dose.
mod common {
    include!("common/titration_accounting.rs");
}
use common::*;
use kerotakis_core::*;
#[test]
fn missing_solver_never_repeats_initial_ph_as_final_dose_result() {
    let mut b = bench();
    stock(&mut b, "NaOH", 0.1);
    let e = b
        .step_with(
            op(10.0, 0.001, 30.0, 1),
            &mut SolverStack::new(vec![]),
            &PermissiveScreen,
        )
        .unwrap();
    assert!(b.vessels[0].solution.is_none());
    assert_eq!(b.vessels[0].moles_of(&SpeciesId::new("NaOH")).0, 0.01);
    assert!((remaining(&b, "NaOH") - 0.09).abs() < 1e-14);
    assert!(e.iter().any(|e| matches!(e, Event::NotYetModeled { .. })));
    assert!(
        !e.iter().any(|e| matches!(e, Event::Titrated { .. })),
        "cannot report the initial pH as final: {e:?}"
    );
    journal(&b, &e);
}
