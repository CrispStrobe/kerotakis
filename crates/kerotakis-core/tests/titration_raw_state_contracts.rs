//! Invalid aggregate quantities must not be offered to a chemistry engine.
mod common {
    include!("common/titration_accounting.rs");
}
use common::*;
#[test]
fn finite_receiver_amount_with_overflowing_mass_refuses_before_solver() {
    quantity_refusal(bench(), op(1e308, 1.0, 30.0, 1), Model::default(), 0);
}
