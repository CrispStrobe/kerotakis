//! Limit policy control: forcing no integration iterations must never return a partial success.
use super::*;

#[test]
fn an_exhausted_integration_limit_is_distinct_from_phase_failure() {
    let models = [ConstantLatent {
        boiling_k: 350.0,
        latent_kj_mol: 40.0,
        valid_k: (280.0, 430.0),
    }; 2];
    let result = ideal_still_checked_with_limit(
        &[0.5, 1.5],
        &models,
        StillTake::Fraction(0.1),
        1,
        101.325,
        bubble_checked,
        0,
    );
    assert!(matches!(result, Err(StillError::IntegrationLimit)));
}
