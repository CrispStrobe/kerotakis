//! LLE with real activity coefficients: oil and water demix because the
//! thermodynamics says so, and alcohol and water provably do not.

use kerotakis_thermo::lle::*;

#[test]
fn water_and_hexane_demix_at_room_temperature() {
    match water_hexane_lle(0.5, 298.15) {
        LleResult::TwoPhase {
            x1_alpha, x1_beta, ..
        } => {
            // One phase nearly pure water, the other nearly pure hexane.
            let (lean, rich) = if x1_alpha < x1_beta {
                (x1_alpha, x1_beta)
            } else {
                (x1_beta, x1_alpha)
            };
            assert!(
                lean < 0.15,
                "the aqueous layer holds little hexane, got x = {lean:.3}"
            );
            assert!(
                rich > 0.85,
                "the organic layer is mostly hexane, got x = {rich:.3}"
            );
        }
        LleResult::SinglePhase => panic!("water and hexane must split"),
    }
}

#[test]
fn water_and_ethanol_stay_one_phase_everywhere() {
    for z in [0.1, 0.3, 0.5, 0.7, 0.9] {
        assert_eq!(
            water_ethanol_lle(z, 298.15),
            LleResult::SinglePhase,
            "ethanol and water are miscible in all proportions (z = {z})"
        );
    }
}
