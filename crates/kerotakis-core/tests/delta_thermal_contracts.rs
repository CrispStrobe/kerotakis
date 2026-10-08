//! Thermal proposal validity and atomic refusal, independent of solver models.
use kerotakis_core::{
    delta::{StateDelta, ThermalDelta},
    *,
};
fn vessel() -> Vessel {
    let mut v = Vessel::new(VesselId(0), "thermal");
    v.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    v
}
#[test]
fn invalid_absolute_temperature_refuses_the_complete_proposal() {
    for t in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0, 0.0, -0.0] {
        let mut v = vessel();
        let before = format!("{v:?}");
        let d = StateDelta::new("invalid-temperature")
            .with_moles(SpeciesId::new("NaCl"), Phase::Solid, 0.001)
            .with_thermal(ThermalDelta::SetTemperature(Kelvin(t)));
        assert!(d.commit(&mut v).is_err());
        assert_eq!(format!("{v:?}"), before);
    }
}
#[test]
fn nonfinite_energy_refuses_before_material_or_thermal_mutation() {
    for e in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut v = vessel();
        let before = format!("{v:?}");
        let d = StateDelta::new("invalid-energy")
            .with_moles(SpeciesId::new("NaCl"), Phase::Solid, 0.001)
            .with_thermal(ThermalDelta::AddEnergy(Joules(e)));
        assert!(d.commit(&mut v).is_err());
        assert_eq!(format!("{v:?}"), before);
    }
}
#[test]
fn nonfinite_energy_is_invalid_even_without_heat_capacity() {
    for e in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut v = Vessel::new(VesselId(0), "empty");
        let before = format!("{v:?}");
        assert!(StateDelta::new("invalid-empty-energy")
            .with_thermal(ThermalDelta::AddEnergy(Joules(e)))
            .commit(&mut v)
            .is_err());
        assert_eq!(format!("{v:?}"), before);
    }
}
#[test]
fn finite_positive_absolute_temperature_remains_supported() {
    for t in [1.0, 298.15, 1000.0] {
        let mut v = vessel();
        StateDelta::new("supported-temperature")
            .with_thermal(ThermalDelta::SetTemperature(Kelvin(t)))
            .commit(&mut v)
            .unwrap();
        assert_eq!(v.temperature.0, t);
    }
}
#[test]
fn finite_heat_and_cooling_keep_their_direction() {
    for e in [-1.0, 1.0] {
        let mut v = vessel();
        let before = v.temperature.0;
        StateDelta::new("supported-energy")
            .with_thermal(ThermalDelta::AddEnergy(Joules(e)))
            .commit(&mut v)
            .unwrap();
        assert!(v.temperature.0.is_finite() && v.temperature.0 > 0.0);
        assert_eq!(v.temperature.0 > before, e > 0.0);
    }
}
#[test]
fn zero_energy_preserves_complete_state() {
    let mut v = vessel();
    let before = format!("{v:?}");
    StateDelta::new("zero-energy")
        .with_thermal(ThermalDelta::AddEnergy(Joules(0.0)))
        .commit(&mut v)
        .unwrap();
    assert_eq!(format!("{v:?}"), before);
}
