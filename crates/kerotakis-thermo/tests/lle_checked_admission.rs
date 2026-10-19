//! Fallible phase stability must propagate model errors at the failing evaluation.
use kerotakis_thermo::{
    lle,
    unifac::{GroupDecomposition, UnifacError},
};
#[test]
fn checked_binary_lle_preserves_invalid_coordinate_and_group_errors() {
    for temperature in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(matches!(
            lle::try_water_hexane_lle(0.5, temperature),
            Err(UnifacError::InvalidTemperature { .. })
        ));
    }
    for fraction in [-0.1, 1.1, f64::NAN] {
        assert!(matches!(
            lle::try_water_hexane_lle(fraction, 298.15),
            Err(UnifacError::InvalidComposition { .. })
        ));
    }
    let missing = GroupDecomposition::from([(u32::MAX, 1)]);
    let water = GroupDecomposition::from([(16, 1)]);
    assert!(matches!(
        lle::try_binary_lle_unifac(&missing, &water, 0.5, 298.15),
        Err(UnifacError::MissingGroup {
            component: 0,
            group_id: u32::MAX
        })
    ));
}
#[test]
fn phase_scan_stops_at_the_first_failed_evaluation() {
    let mut calls = 0;
    let result = lle::try_lle_binary(0.5, &mut |_| {
        calls += 1;
        if calls == 3 {
            Err("third evaluation refused")
        } else {
            Ok((1.0, 1.0))
        }
    });
    assert_eq!(result, Err("third evaluation refused"));
    assert_eq!(
        calls, 3,
        "no fabricated coefficient or later phase scan after refusal"
    );
}
