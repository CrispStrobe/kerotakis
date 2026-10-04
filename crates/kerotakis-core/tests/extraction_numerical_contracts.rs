//! Positive partition yields and minority remainders must survive stable calculation.
use kerotakis_core::apparatus::*;
fn near(actual: f64, expected: f64) {
    assert!(actual > 0.0 && actual.is_finite());
    assert!(
        (actual / expected - 1.0).abs() < 1e-8,
        "{actual} != {expected}"
    );
}
#[test]
fn tiny_positive_single_yield_is_not_cancelled() {
    let r = extract(1.0, 1.0, 1e-20, 1.0);
    near(r.organic_moles, 1e-20);
    near(r.efficiency, 1e-20);
    assert_eq!(r.aqueous_moles, 1.0);
}
#[test]
fn repeated_tiny_yield_accumulates() {
    let r = extract_repeated(1.0, 1.0, 1e-20, 1.0, 1_000_000);
    near(r.organic_moles, 1e-14);
}
#[test]
fn intermediate_ratio_product_overflow_preserves_finite_ratio() {
    let r = extract(1.0, 1e308, 1e308, 2.0);
    near(r.organic_moles, 2.0 / 3.0);
    near(r.aqueous_moles, 1.0 / 3.0);
}
#[test]
fn intermediate_ratio_product_underflow_preserves_finite_ratio() {
    let r = extract(1.0, 1e-308, 1e-308, 1e-20);
    near(r.organic_moles, 1e-20);
}
#[test]
fn repeated_normalized_remainder_underflow_is_rescaled() {
    let r = extract_repeated(1e308, 1.0, 1.0, 1e308, 2);
    near(r.aqueous_moles, 1e-308);
    near(r.organic_moles, 1e308);
}
#[test]
fn unrepresentable_ratio_can_still_have_representable_yield() {
    let r = extract(1e308, 1.0, 1e-308, 1e-308);
    near(r.organic_moles, 1e-308);
}
#[test]
fn unsaturated_solubility_path_preserves_tiny_yield() {
    let r = extract_repeated_with_aqueous_solubility(1.0, 1.0, 1e-20, 1.0, 1e20, 1).unwrap();
    near(r.organic_moles, 1e-20);
}
#[test]
fn ordinary_and_zero_stage_controls_remain_supported() {
    let r = extract(1.0, 1.0, 1.0, 1.0);
    assert_eq!(r.organic_moles, 0.5);
    assert_eq!(r.aqueous_moles, 0.5);
    let r = extract_repeated(1.0, 1.0, 1.0, 1.0, 3);
    near(r.aqueous_moles, 0.125);
    near(r.organic_moles, 0.875);
    let r = extract_repeated(1.0, 1.0, 1.0, 1.0, 0);
    assert_eq!(r.aqueous_moles, 1.0);
    assert_eq!(r.organic_moles, 0.0);
}
