//! Frozen source-informed controls: a pressure jump must not create a root gap.
use kerotakis_thermo::vle::*;

#[test]
fn ethanol_overlap_endpoints_have_no_pressure_jump() {
    for t in [79.65_f64, 80.0] {
        let below = f64::from_bits(t.to_bits() - 1);
        let above = f64::from_bits(t.to_bits() + 1);
        let left = ETHANOL.pressure_kpa(below).unwrap();
        let right = ETHANOL.pressure_kpa(above).unwrap();
        assert!((left / right - 1.0).abs() < 1e-11, "jump at {t}: {left} -> {right}");
    }
}

#[test]
fn pure_ethanol_bubble_and_dew_roots_cover_the_old_switch_gap() {
    let low = ETHANOL_LOW.pressure_kpa(80.0).unwrap();
    let high = ETHANOL_HIGH.pressure_kpa(80.0).unwrap();
    assert!(high > low);
    for weight in [0.1, 0.5, 0.9] {
        let pressure = low + weight * (high - low);
        let bubble = bubble_point_with(&[ETHANOL], &[1.0], pressure, |_| vec![1.0])
            .expect("continuous fitted overlap must contain the bubble root");
        let dew = dew_point_with(&[ETHANOL], &[1.0], pressure, &mut |_, _| vec![1.0])
            .expect("continuous fitted overlap must contain the dew root");
        assert!((79.65..=80.0).contains(&bubble.t_celsius));
        assert!((bubble.t_celsius - dew.t_celsius).abs() < 1e-7);
        assert!((ETHANOL.pressure_kpa(bubble.t_celsius).unwrap() / pressure - 1.0).abs() < 1e-8);
    }
}

#[test]
fn ideal_binary_bubble_root_covers_the_old_switch_gap() {
    let pressure = 0.5 * WATER.pressure_kpa(80.0).unwrap()
        + 0.25 * (ETHANOL_LOW.pressure_kpa(80.0).unwrap()
            + ETHANOL_HIGH.pressure_kpa(80.0).unwrap());
    let bubble = bubble_point_with(&[ETHANOL, WATER], &[0.5, 0.5], pressure, |_| vec![1.0, 1.0])
        .expect("continuous mixture pressure must contain the bubble root");
    assert!((79.65..=80.0).contains(&bubble.t_celsius));
    let accounted = 0.5 * (ETHANOL.pressure_kpa(bubble.t_celsius).unwrap()
        + WATER.pressure_kpa(bubble.t_celsius).unwrap());
    assert!((accounted / pressure - 1.0).abs() < 1e-8);
}
