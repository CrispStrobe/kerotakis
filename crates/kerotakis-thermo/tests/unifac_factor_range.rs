//! A zero Boltzmann weight produced by floating-point underflow is not a
//! representable positive interaction, even if later arithmetic stays finite.
use kerotakis_thermo::{
    lle,
    unifac::{approved_table, try_activity_coefficients, GroupDecomposition},
};
fn mixture() -> [(GroupDecomposition, f64); 2] {
    [
        (GroupDecomposition::from([(1, 2), (2, 4)]), 0.5),
        (GroupDecomposition::from([(16, 1)]), 0.5),
    ]
}
#[test]
fn underflowed_directional_weights_refuse_even_if_final_gammas_look_finite() {
    let result = try_activity_coefficients(&approved_table(), &mixture(), 1e-300);
    assert!(
        result.is_err(),
        "a weight rounded to zero must not silently contribute a finite coefficient: {result:?}"
    );
}
#[test]
fn underflowed_weights_cannot_supply_a_phase_verdict() {
    let result = lle::try_water_hexane_lle(0.5, 1e-300);
    assert!(
        result.is_err(),
        "phase detection must preserve the interaction-range refusal: {result:?}"
    );
}
#[test]
fn ordinary_directional_weights_keep_the_existing_two_phase_control() {
    let result = lle::try_water_hexane_lle(0.5, 298.15).unwrap();
    match result {
        lle::LleResult::TwoPhase {
            x1_alpha,
            x1_beta,
            phase_fraction_alpha,
        } => {
            assert!(x1_alpha > 0.0 && x1_alpha < 0.5 && x1_beta > 0.5 && x1_beta < 1.0);
            assert!(
                (phase_fraction_alpha * x1_alpha + (1.0 - phase_fraction_alpha) * x1_beta - 0.5)
                    .abs()
                    < 1e-12
            );
        }
        _ => panic!("ordinary water/hexane must preserve its computed two-phase result"),
    }
}
