use kerotakis_thermo::lle::try_infinite_dilution_gamma;
use kerotakis_thermo::unifac::{
    approved_table, try_activity_coefficients, GroupDecomposition, UnifacError,
};

#[test]
fn checked_refusals_keep_the_invalid_coordinate_typed() {
    let water = GroupDecomposition::from([(16, 1)]);
    let unknown = GroupDecomposition::from([(15, 1), (999_999, 1)]);
    assert!(matches!(
        try_activity_coefficients(
            &approved_table(),
            &[(unknown, 0.5), (water.clone(), 0.5)],
            298.15
        ),
        Err(UnifacError::MissingGroup {
            component: 0,
            group_id: 999_999
        })
    ));
    assert!(matches!(
        try_activity_coefficients(&approved_table(), &[(water.clone(), 1.0)], 0.0),
        Err(UnifacError::InvalidTemperature { kelvin: 0.0 })
    ));
    assert!(matches!(
        try_activity_coefficients(&approved_table(), &[(water, 0.4)], 298.15),
        Err(UnifacError::CompositionNotNormalized { .. })
    ));
}

#[test]
fn checked_infinite_dilution_propagates_a_missing_group_without_mutation() {
    let water = GroupDecomposition::from([(16, 1)]);
    let unknown = GroupDecomposition::from([(15, 1), (999_999, 1)]);
    let original = unknown.clone();
    assert!(matches!(
        try_infinite_dilution_gamma(&unknown, &water, 298.15),
        Err(UnifacError::MissingGroup {
            component: 0,
            group_id: 999_999
        })
    ));
    assert_eq!(unknown, original);
    assert_eq!(water, GroupDecomposition::from([(16, 1)]));
}
