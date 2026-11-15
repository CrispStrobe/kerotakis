use kerotakis_core::{
    amount::Amount,
    component_split::{ComponentSplit, SplitError},
};
#[test]
fn independent_dyadic_residue_and_condensate_remain_positive_and_close() {
    let residue = 2.0_f64.powi(-70);
    let split = ComponentSplit::from_residue(1.0, residue).unwrap();
    assert_eq!(split.retained().parts(), (residue, 0.0));
    assert_eq!(split.transferred().parts(), (1.0, -residue));
    assert_eq!(
        split.transferred().checked_add(split.retained()).unwrap(),
        Amount::new(1.0).unwrap()
    );
    assert_eq!(
        split.scalar_write_plan(1.0, 0.0),
        Err(SplitError::ScalarBoundary)
    );
}
#[test]
fn actual_sp003_step629_correction_cannot_authorize_a_scalar_writer() {
    // Actual source-bound trace, not a synthetic qualification. Next
    // positive provisional residue is 3/4 of the recorded residue.
    let initial = 0.10853285289457118;
    let recorded_residue = 2.7755575615628914e-17;
    let next_residue = recorded_residue - 6.938893903907228e-18;
    let split = ComponentSplit::from_residue(initial, next_residue).unwrap();
    assert_eq!(split.retained().to_f64(), recorded_residue * 0.75);
    assert_ne!(split.transferred().parts().1, 0.0);
    assert_eq!(
        split.transferred().checked_add(split.retained()).unwrap(),
        Amount::new(initial).unwrap()
    );
    assert_eq!(
        split.scalar_write_plan(initial, 0.0),
        Err(SplitError::ScalarBoundary)
    );
}
#[test]
fn representable_split_plans_both_writes_and_stale_donor_is_rejected() {
    let split = ComponentSplit::from_residue(1.0, 0.25).unwrap();
    let plan = split.scalar_write_plan(1.0, 0.5).unwrap();
    assert_eq!(
        (plan.donor_after, plan.receiver_after, plan.transferred),
        (0.25, 1.25, 0.75)
    );
    assert_eq!(
        split.scalar_write_plan(2.0, 0.5),
        Err(SplitError::StaleDonor)
    );
    assert_eq!(
        split.scalar_write_plan(1.0, 2.0_f64.powi(54)),
        Err(SplitError::ScalarBoundary)
    );
}
#[test]
fn invalid_split_owners_never_receive_write_authority() {
    for (initial, residue) in [
        (1.0, 2.0),
        (f64::NAN, 0.0),
        (1.0, f64::INFINITY),
        (-1.0, 0.0),
    ] {
        assert!(ComponentSplit::from_residue(initial, residue).is_err());
    }
    let split = ComponentSplit::from_residue(1.0, 0.25).unwrap();
    for receiver in [-1.0, f64::NAN, f64::INFINITY] {
        assert!(split.scalar_write_plan(1.0, receiver).is_err());
    }
}
