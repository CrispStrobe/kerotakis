use kerotakis_core::{instrument::Thermometer, measurement::*, *};
#[test]
fn restoring_invalid_device_state_is_rejected() {
    let device = MeasurementDevice::new(Thermometer, 1).unwrap();
    for (field, value) in [
        ("error_std", -1.0),
        ("resolution", 0.0),
        ("lag_seconds", -1.0),
    ] {
        let mut data = serde_json::to_value(device.state()).unwrap();
        data[field] = serde_json::json!(value);
        let state: DeviceState = serde_json::from_value(data).unwrap();
        assert!(MeasurementDevice::from_state(Thermometer, &state).is_err());
    }
}
#[test]
fn invalid_public_calibration_is_rejected() {
    let c = Calibration {
        reference_low: 0.0,
        reference_high: 1.0,
        raw_low: 0.0,
        raw_high: 0.0,
        id: "bad".into(),
    };
    assert!(MeasurementDevice::new(Thermometer, 1)
        .unwrap()
        .with_calibration(c)
        .is_err());
}
#[test]
fn nonfinite_reading_does_not_change_device() {
    let mut d = MeasurementDevice::new(Thermometer, 1).unwrap();
    let before = d.state().clone();
    let mut v = Vessel::new(VesselId(0), "bad");
    v.temperature = Kelvin(f64::INFINITY);
    assert!(d.record(&v, 0.0).is_err());
    assert_eq!(d.state(), &before);
}
