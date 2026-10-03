use super::*;

#[test]
fn legacy_e110_phase_failure_is_inside_the_ethanol_switch_gap() {
    let legacy = VapourPressure::Piecewise(ETHANOL_SEGMENTS);
    let mut failed_x = None;
    let result = ethanol_water_still_with_phase(
        5.534276991396059, 0.34252968373526665, StillTake::Fraction(0.1),
        4, ATMOSPHERE_KPA,
        |x, pressure| {
            let answer = bubble_point_with(&[legacy, WATER], &x, pressure, |tk| {
                let (ge, gw) = ethanol_water_activity_pair(x, tk);
                vec![ge, gw]
            });
            if answer.is_none() && failed_x.is_none() { failed_x = Some(x); }
            answer
        },
    );
    assert_eq!(result, Err(StillError::PhaseEvaluation));
    let x = failed_x.expect("capture the first unavailable phase root");
    let (ge, gw) = ethanol_water_activity_pair(x, 80.0 + KELVIN_OFFSET);
    let water = x[1] * gw * WATER.pressure_kpa(80.0).unwrap();
    let below = water + x[0] * ge * ETHANOL_LOW.pressure_kpa(80.0).unwrap();
    let above = water + x[0] * ge * ETHANOL_HIGH.pressure_kpa(80.0).unwrap();
    assert!(below < ATMOSPHERE_KPA && ATMOSPHERE_KPA < above,
        "first failure x={x:?}, switch pressures={below},{above}");
    assert!((below / ATMOSPHERE_KPA - 1.0).abs() > 1e-8);
    assert!((above / ATMOSPHERE_KPA - 1.0).abs() > 1e-8);
    eprintln!("E110 legacy first failed x={x:?}; switch pressures={below},{above} kPa");
}
