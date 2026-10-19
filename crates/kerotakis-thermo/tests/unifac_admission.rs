//! Negative input admission, independently of the published parameter domain.
use kerotakis_thermo::unifac::{approved_table, try_activity_coefficients, GroupDecomposition};

fn mixture() -> Vec<(GroupDecomposition, f64)> {
    vec![
        (GroupDecomposition::from([(15, 1)]), 0.5),
        (GroupDecomposition::from([(16, 1)]), 0.5),
    ]
}

#[test]
fn unknown_subgroup_cannot_be_silently_dropped_from_a_molecule() {
    let mut components = mixture();
    components[0].0.insert(999_999, 1);
    let result = try_activity_coefficients(&approved_table(), &components, 298.15);
    assert!(
        result.is_err(),
        "a partial molecule was evaluated: {result:?}"
    );
}

#[test]
fn nonpositive_and_nonfinite_temperatures_are_refused() {
    for temperature in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let result = try_activity_coefficients(&approved_table(), &mixture(), temperature);
        assert!(
            result.is_err(),
            "temperature {temperature:?} produced {result:?}"
        );
    }
}

#[test]
fn invalid_mole_fractions_are_refused_before_evaluation() {
    for fractions in [
        [-0.1, 1.1],
        [f64::NAN, 0.5],
        [f64::INFINITY, 0.5],
        [0.0, 0.0],
        [0.2, 0.2],
    ] {
        let mut components = mixture();
        for (component, fraction) in components.iter_mut().zip(fractions) {
            component.1 = fraction;
        }
        let result = try_activity_coefficients(&approved_table(), &components, 298.15);
        assert!(
            result.is_err(),
            "invalid composition {fractions:?} produced {result:?}"
        );
    }
}

#[test]
fn empty_or_zero_count_molecular_decompositions_are_refused() {
    for groups in [
        GroupDecomposition::new(),
        GroupDecomposition::from([(15, 0)]),
    ] {
        let mut components = mixture();
        components[0].0 = groups;
        let result = try_activity_coefficients(&approved_table(), &components, 298.15);
        assert!(
            result.is_err(),
            "an empty molecule was evaluated: {result:?}"
        );
    }
}

#[test]
fn nonfinite_group_and_interaction_parameters_are_refused() {
    let mut table = approved_table();
    table.groups.iter_mut().find(|g| g.id == 15).unwrap().r = f64::NAN;
    let result = try_activity_coefficients(&table, &mixture(), 298.15);
    assert!(result.is_err(), "invalid group data produced {result:?}");

    let mut table = approved_table();
    table
        .interactions
        .iter_mut()
        .find(|p| p.m == 6 && p.n == 7)
        .unwrap()
        .a_mn = f64::NAN;
    let result = try_activity_coefficients(&table, &mixture(), 298.15);
    assert!(
        result.is_err(),
        "invalid interaction data produced {result:?}"
    );
}

#[test]
fn numerical_overflow_is_a_refusal_not_a_successful_vector() {
    let result = try_activity_coefficients(&approved_table(), &mixture(), 1e-300);
    assert!(
        result.is_err(),
        "overflow produced a successful result: {result:?}"
    );
}

#[test]
fn pure_water_and_absent_probe_limits_remain_admitted() {
    let water = GroupDecomposition::from([(16, 1)]);
    let result = try_activity_coefficients(&approved_table(), &[(water, 1.0)], 298.15).unwrap();
    assert!((result[0] - 1.0).abs() < 1e-12);

    let mut components = mixture();
    components[0].1 = 0.0;
    components[1].1 = 1.0;
    let result = try_activity_coefficients(&approved_table(), &components, 298.15).unwrap();
    assert!(result.iter().all(|gamma| gamma.is_finite() && *gamma > 0.0));
    assert!((result[1] - 1.0).abs() < 1e-12);
}
