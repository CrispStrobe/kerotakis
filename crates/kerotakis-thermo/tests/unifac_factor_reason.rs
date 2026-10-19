//! Typed numerical software controls; mutated rows are not chemical parameters.
use kerotakis_thermo::unifac::*;
#[test]
fn nonrepresentable_interaction_factor_keeps_its_direction() {
    let groups = [
        (GroupDecomposition::from([(1, 2), (2, 4)]), 0.5),
        (GroupDecomposition::from([(16, 1)]), 0.5),
    ];
    for overflow in [false, true] {
        let mut table = approved_table();
        let temperature = if overflow {
            table
                .interactions
                .iter_mut()
                .find(|row| row.m == 1 && row.n == 7)
                .unwrap()
                .a_mn = -f64::MAX;
            0.5
        } else {
            1e-300
        };
        assert!(matches!(
            try_activity_coefficients(&table, &groups, temperature),
            Err(UnifacError::InteractionFactorRange {
                from_main_group: 1,
                to_main_group: 7
            })
        ));
    }
}
