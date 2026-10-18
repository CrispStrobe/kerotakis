//! Parsed quantities must survive save/replay, including scaled overflow.
use kerotakis_core::{
    script::{parse_at, parse_energy, parse_op, parse_pressure, parse_volume},
    Operator,
};

#[test]
fn numeric_operator_fields_refuse_nan_infinity_and_overflow() {
    for number in ["NaN", "inf", "-inf", "1e999"] {
        for input in [
            format!("stir v1 {number}rpm 1s"),
            format!("evaporate v1 {number}"),
            format!("decant v1 v2 {number}"),
            format!("distil v1 v2 {number}"),
            format!("distil v1 v2 {number}kJ"),
            format!("mix v1 {number} v2 0.5 into v3"),
            format!("mix v1 0.5 v2 {number} into v3"),
            format!("transport v1 from v2 to v3 steps 1 courant {number}"),
            format!("add v1 I-131 {number}mol"),
            format!("add v1 water 1mol @ {number}K"),
            format!("seal v1 {number}L"),
            format!("sweep v1 {number}Pa"),
        ] {
            assert!(
                parse_op(&input).is_err(),
                "non-finite operator admitted: {input}"
            );
        }
    }
    for input in [
        "heat v1 1e308kJ",
        "sweep v1 1e308bar",
        "distil v1 v2 1e308kJ",
        "add v1 hydrogen_peroxide_3_percent 1e308L",
        "stock hydrogen_peroxide_3_percent 1e308L",
    ] {
        assert!(
            parse_op(input).is_err(),
            "scaled overflow admitted: {input}"
        );
    }
}

#[test]
fn public_unit_readers_check_scaled_results() {
    assert!(parse_energy("1e308kJ").is_err());
    assert!(parse_pressure("1e308bar").is_err());
    assert!(parse_volume("1e999L").is_err());
    assert!(parse_at(&["@", "NaNK"]).is_err());
}

#[test]
fn finite_extremes_and_normal_values_keep_exact_json_roundtrip() {
    for input in [
        "heat v1 1e300J",
        "sweep v1 1e300Pa",
        "seal v1 1e300L",
        "stir v1 500rpm 2s",
        "distil v1 v2 0.5",
        "mix v1 0.25 v2 0.5 into v3",
        "transport v1 from v2 to v3 steps 1 courant 0.5",
        "add v1 I-131 1e-9mol",
    ] {
        let original = parse_op(input).unwrap().unwrap();
        let encoded = serde_json::to_vec(&original).unwrap();
        let decoded: Operator = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(original, decoded, "roundtrip changed {input}");
    }
}
