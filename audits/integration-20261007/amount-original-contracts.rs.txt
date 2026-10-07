//! Independently frozen requirements for a standalone, finite nonnegative
//! two-component Amount. No production inventory is migrated by these tests.
#[path = "../src/amount.rs"]
mod amount;
use amount::Amount;

fn scalar(value: f64) -> Amount {
    Amount::new(value).unwrap()
}

#[test]
fn zero_and_scalar_inputs_are_canonical() {
    for value in [0.0, -0.0, f64::from_bits(1), 1.0, 1e300] {
        let amount = scalar(value);
        assert_eq!(amount.parts(), (value.abs(), 0.0));
        assert_eq!(amount.to_f64(), value.abs());
    }
}

#[test]
fn invalid_scalar_and_pair_inputs_are_rejected() {
    for value in [-1.0, f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
        assert!(Amount::new(value).is_err());
    }
    for (hi, lo) in [(1.0, f64::NAN), (f64::INFINITY, -1.0), (-1.0, 0.0), (1.0, -2.0)] {
        assert!(Amount::from_parts(hi, lo).is_err());
    }
}

#[test]
fn pair_construction_normalizes_without_discarding_a_small_addend() {
    let tiny = 2.0_f64.powi(-60);
    assert_eq!(Amount::from_parts(1.0, tiny).unwrap().parts(), (1.0, tiny));
    assert_eq!(Amount::from_parts(tiny, 1.0).unwrap().parts(), (1.0, tiny));
    assert_eq!(Amount::from_parts(0.75, 0.25).unwrap().parts(), (1.0, 0.0));
}

#[test]
fn sub_ulp_addition_and_subtraction_retain_the_signed_low_component() {
    let tiny = scalar(2.0_f64.powi(-60));
    assert_eq!(scalar(1.0).checked_add(tiny).unwrap().parts(), (1.0, tiny.to_f64()));
    assert_eq!(scalar(1.0).checked_sub(tiny).unwrap().parts(), (1.0, -tiny.to_f64()));
}

#[test]
fn repeated_sub_ulp_additions_cross_the_scalar_resolution_boundary() {
    let tiny = scalar(2.0_f64.powi(-60));
    let mut amount = scalar(1.0);
    for _ in 0..256 {
        amount = amount.checked_add(tiny).unwrap();
    }
    assert_eq!(amount.parts(), (1.0 + f64::EPSILON, 0.0));
}

#[test]
fn repeated_sub_ulp_debits_and_inverse_credits_cancel() {
    let tiny = scalar(2.0_f64.powi(-60));
    let mut amount = scalar(1.0);
    for _ in 0..1024 {
        amount = amount.checked_sub(tiny).unwrap();
    }
    for _ in 0..1024 {
        amount = amount.checked_add(tiny).unwrap();
    }
    assert_eq!(amount.parts(), (1.0, 0.0));
}

#[test]
fn subtracting_the_large_component_exposes_preserved_small_inventory() {
    let tiny = scalar(2.0_f64.powi(-80));
    let amount = scalar(1.0).checked_add(tiny).unwrap();
    assert_eq!(amount.checked_sub(scalar(1.0)).unwrap(), tiny);
}

#[test]
fn exact_exhaustion_succeeds_but_real_overdraw_is_rejected() {
    let tiny = scalar(2.0_f64.powi(-60));
    let below_one = scalar(1.0).checked_sub(tiny).unwrap();
    assert_eq!(below_one.checked_sub(below_one).unwrap(), scalar(0.0));
    assert!(below_one.checked_sub(scalar(1.0)).is_err());
    assert!(scalar(0.0).checked_sub(scalar(f64::from_bits(1))).is_err());
}

#[test]
fn negative_high_input_can_normalize_to_a_nonnegative_total() {
    assert_eq!(Amount::from_parts(-0.25, 0.75).unwrap(), scalar(0.5));
}

#[test]
fn scale_preserves_low_component_for_exact_power_of_two() {
    let tiny = 2.0_f64.powi(-60);
    let amount = Amount::from_parts(1.0, tiny).unwrap();
    assert_eq!(amount.checked_scale(0.5).unwrap().parts(), (0.5, tiny * 0.5));
    assert_eq!(amount.checked_scale(0.0).unwrap(), scalar(0.0));
    assert_eq!(amount.checked_scale(1.0).unwrap(), amount);
}

#[test]
fn scale_retains_normal_product_rounding_error() {
    let amount = scalar(0.1).checked_scale(0.1).unwrap();
    let (hi, lo) = amount.parts();
    assert_eq!(hi, 0.1_f64 * 0.1);
    assert_eq!(lo, 0.1_f64.mul_add(0.1, -hi));
    assert_ne!(lo, 0.0);
}

#[test]
fn invalid_scale_overflow_and_total_underflow_are_rejected() {
    for factor in [-1.0, f64::NAN, f64::INFINITY] {
        assert!(scalar(1.0).checked_scale(factor).is_err());
    }
    assert!(scalar(f64::MAX).checked_scale(2.0).is_err());
    assert!(scalar(f64::MAX).checked_add(scalar(f64::MAX)).is_err());
    assert!(scalar(f64::from_bits(1)).checked_scale(0.5).is_err());
}

#[test]
fn quantized_subnormal_scale_refuses_but_exact_subnormal_scale_succeeds() {
    let quantum = f64::from_bits(1);
    assert!(scalar(3.0 * quantum).checked_scale(0.75).is_err());
    assert_eq!(scalar(2.0 * quantum).checked_scale(0.5).unwrap(), scalar(quantum));
}

#[test]
fn json_round_trip_preserves_both_components_and_legacy_scalars_load() {
    let amount = Amount::from_parts(1.0, -2.0_f64.powi(-60)).unwrap();
    let encoded = serde_json::to_string(&amount).unwrap();
    assert_eq!(serde_json::from_str::<Amount>(&encoded).unwrap(), amount);
    assert_eq!(serde_json::from_str::<Amount>("0.25").unwrap(), scalar(0.25));
    let value: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(value["hi"], 1.0);
    assert_eq!(value["lo"], -2.0_f64.powi(-60));
}

#[test]
fn persisted_pairs_are_validated_instead_of_silently_renormalized() {
    for document in [
        "-1", "null", "{\"hi\":-1,\"lo\":0}",
        "{\"hi\":1,\"lo\":1}", "{\"hi\":0,\"lo\":1}",
        "{\"hi\":1}", "{\"hi\":1,\"lo\":0,\"surprise\":1}",
    ] {
        assert!(serde_json::from_str::<Amount>(document).is_err(), "{document}");
    }
    assert_eq!(serde_json::from_str::<Amount>("{\"hi\":0,\"lo\":0}").unwrap(), scalar(0.0));
}
