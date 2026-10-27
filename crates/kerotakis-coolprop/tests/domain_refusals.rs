//! Domain-refusal negatives. Run WITHOUT the engine (pure Rust) and with it:
//! the gate runs before any engine call, so the refusal is the same.
use kerotakis_coolprop::*;

#[test]
fn below_triple_point_is_refused() {
    assert!(matches!(
        saturation_at_temperature(273.0),
        Err(CoolPropError::OutOfDomain {
            quantity: Quantity::Temperature,
            ..
        })
    ));
    assert!(matches!(
        saturation_at_pressure(500.0),
        Err(CoolPropError::OutOfDomain {
            quantity: Quantity::Pressure,
            ..
        })
    ));
}

#[test]
fn critical_point_and_above_is_not_a_saturation_state() {
    for t in [TC_K, 700.0] {
        assert!(matches!(
            saturation_at_temperature(t),
            Err(CoolPropError::OutOfDomain { .. })
        ));
    }
    assert!(matches!(
        saturation_at_pressure(PC_PA),
        Err(CoolPropError::OutOfDomain { .. })
    ));
}

#[test]
fn single_phase_box_is_enforced() {
    assert!(matches!(
        water_state(1300.0, 1e5),
        Err(CoolPropError::OutOfDomain { .. })
    ));
    assert!(matches!(
        water_state(300.0, 200e6),
        Err(CoolPropError::OutOfDomain { .. })
    ));
    assert!(matches!(
        water_state(300.0, 100.0),
        Err(CoolPropError::OutOfDomain { .. })
    ));
}

#[test]
fn near_critical_is_refused() {
    assert!(matches!(
        water_state(647.5, 22.5e6),
        Err(CoolPropError::NearCritical { .. })
    ));
}

#[test]
fn non_finite_is_refused() {
    assert!(matches!(
        saturation_at_temperature(f64::NAN),
        Err(CoolPropError::NonFinite { .. })
    ));
    assert!(matches!(
        water_state(300.0, f64::INFINITY),
        Err(CoolPropError::NonFinite { .. })
    ));
}

#[cfg(not(feature = "engine"))]
#[test]
fn in_domain_without_engine_is_engine_not_linked_not_a_value() {
    assert_eq!(
        saturation_at_temperature(373.0),
        Err(CoolPropError::EngineNotLinked)
    );
    assert_eq!(
        water_state(300.0, 101_325.0),
        Err(CoolPropError::EngineNotLinked)
    );
    assert_eq!(
        water_saturation_pressure_kpa(25.0),
        Err(CoolPropError::EngineNotLinked)
    );
}
