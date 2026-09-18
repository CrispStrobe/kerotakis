//! MEAS-001: calibrated, persisted, reproducible measurement records.
//!
//! The ideal instruments in `instrument.rs` observe state exactly. This
//! layer wraps them with what a real bench adds: a calibration the learner
//! performed, declared resolution and error, a response that takes time to
//! settle, and a record stream whose noise replays bit-identically after a
//! save/load. The physical model's validity is NOT folded in here — the
//! instrument range flag stays separate, and the caller supplies any
//! chemical-model validity separately.

use kerotakis_core::instrument::{InstrumentContract, PhMeter, Thermometer};
use kerotakis_core::measurement::{Calibration, MeasurementDevice, MeasurementError};
use kerotakis_core::script::parse_op;
use kerotakis_core::{Bench, Kelvin, SolutionInfo, Vessel, VesselId};

fn bench_with(commands: &[&str]) -> Bench {
    let mut bench = Bench::new();
    for command in commands {
        bench
            .step(parse_op(command).unwrap().unwrap())
            .expect("operator");
    }
    bench
}

fn ph_vessel() -> Vessel {
    let mut vessel = Vessel::new(VesselId(0), "beaker");
    vessel.solution = Some(SolutionInfo {
        solvent_activity: None,
        scope: Default::default(),
        solvent_kg: None,
        pe: None,
        redox: Vec::new(),
        ph: 7.0,
        ionic_strength: 0.0,
        species: Vec::new(),
        provenance: None,
    });
    vessel
}

#[test]
fn two_point_calibration_maps_raw_to_reference_and_is_recorded() {
    let bench = bench_with(&["add v1 water 100mL"]);
    let raw = Thermometer.measure(&bench.vessels[0]).unwrap().value;
    let mut device = MeasurementDevice::new(Thermometer, 7)
        .unwrap()
        .with_calibration(Calibration::two_point(1.0, 101.0, 0.0, 100.0, "cal-a").unwrap())
        .unwrap();
    let record = device.record(&bench.vessels[0], 0.0).unwrap();
    assert_eq!(record.observable, "temperature");
    assert!(
        (record.value - (raw + 1.0)).abs() < 1e-9,
        "{}",
        record.value
    );
    assert_eq!(record.calibration_id.as_deref(), Some("cal-a"));
}

#[test]
fn error_stream_is_reproducible_and_survives_save_load() {
    let bench = bench_with(&["add v1 water 100mL"]);
    let mut device = MeasurementDevice::new(Thermometer, 42)
        .unwrap()
        .with_error_std(0.5)
        .unwrap();
    let first = device.record(&bench.vessels[0], 0.0).unwrap().value;
    let second = device.record(&bench.vessels[0], 10.0).unwrap().value;
    let saved = serde_json::to_string(&device.state()).unwrap();
    let state = serde_json::from_str(&saved).unwrap();
    let mut restored = MeasurementDevice::from_state(Thermometer, &state).unwrap();
    let third = restored.record(&bench.vessels[0], 20.0).unwrap().value;
    let fourth = restored.record(&bench.vessels[0], 30.0).unwrap().value;

    let mut replay = MeasurementDevice::new(Thermometer, 42)
        .unwrap()
        .with_error_std(0.5)
        .unwrap();
    let mut replayed = Vec::new();
    for elapsed in [0.0, 10.0, 20.0, 30.0] {
        replayed.push(replay.record(&bench.vessels[0], elapsed).unwrap().value);
    }
    assert_eq!(replayed, vec![first, second, third, fourth]);

    let mut other_seed = MeasurementDevice::new(Thermometer, 43)
        .unwrap()
        .with_error_std(0.5)
        .unwrap();
    let other: Vec<f64> = [0.0, 10.0]
        .iter()
        .map(|elapsed| {
            other_seed
                .record(&bench.vessels[0], *elapsed)
                .unwrap()
                .value
        })
        .collect();
    assert_ne!(other, vec![first, second]);
}

#[test]
fn resolution_quantizes_the_recorded_value() {
    let bench = bench_with(&["add v1 water 100mL"]);
    let mut device = MeasurementDevice::new(Thermometer, 9)
        .unwrap()
        .with_resolution(0.5)
        .unwrap()
        .with_calibration(Calibration::two_point(0.0, 101.0, 0.0, 100.0, "cal-b").unwrap())
        .unwrap();
    let record = device.record(&bench.vessels[0], 0.0).unwrap();
    let steps = record.value / 0.5;
    assert!(
        (steps - steps.round()).abs() < 1e-9,
        "{} is not on the 0.5 grid",
        record.value
    );
    assert!(
        (record.value - 20.2).abs() > 1e-9,
        "raw 20.2 must be quantized"
    );
}

#[test]
fn out_of_range_readings_are_flagged_not_hidden() {
    let mut vessel = Vessel::new(VesselId(0), "beaker");
    vessel.temperature = Kelvin(673.15);
    let mut device = MeasurementDevice::new(Thermometer, 1).unwrap();
    let record = device.record(&vessel, 0.0).unwrap();
    assert!(!record.in_range);
    assert!((record.value - 400.0).abs() < 1e-9, "{}", record.value);
}

#[test]
fn lag_relaxes_toward_the_target_and_time_must_progress() {
    let mut bench = bench_with(&["add v1 water 100mL"]);
    let mut device = MeasurementDevice::new(Thermometer, 5)
        .unwrap()
        .with_lag_seconds(10.0)
        .unwrap();
    let started = device.record(&bench.vessels[0], 0.0).unwrap().value;
    bench.vessels[0].temperature = Kelvin::from_celsius(30.0);
    let raw_target = 30.0f64;
    let relaxed = device.record(&bench.vessels[0], 10.0).unwrap().value;
    let expected = started + (raw_target - started) * (1.0 - (-1.0f64).exp());
    assert!((relaxed - expected).abs() < 1e-9, "{relaxed} vs {expected}");
    assert!(matches!(
        device.record(&bench.vessels[0], 5.0),
        Err(MeasurementError::TimeReversed { .. })
    ));
}

#[test]
fn failed_readings_do_not_advance_the_noise_stream() {
    let dry = Vessel::new(VesselId(0), "beaker");
    let mut noisy = MeasurementDevice::new(PhMeter, 11)
        .unwrap()
        .with_error_std(0.7)
        .unwrap();
    assert!(matches!(
        noisy.record(&dry, 0.0),
        Err(MeasurementError::NoReading)
    ));
    let good = noisy.record(&ph_vessel(), 5.0).unwrap();
    let mut reference = MeasurementDevice::new(PhMeter, 11)
        .unwrap()
        .with_error_std(0.7)
        .unwrap();
    let expected = reference.record(&ph_vessel(), 5.0).unwrap();
    assert_eq!(good.value, expected.value);
}

#[test]
fn records_carry_no_sample_composition() {
    let bench = bench_with(&["add v1 water 100mL"]);
    let mut device = MeasurementDevice::new(Thermometer, 2).unwrap();
    let record = device.record(&bench.vessels[0], 0.0).unwrap();
    let json = serde_json::to_value(&record).unwrap();
    for key in json.as_object().unwrap().keys() {
        assert!(
            [
                "observable",
                "value",
                "unit",
                "precision",
                "in_range",
                "elapsed_seconds",
                "calibration_id",
                "sample_index"
            ]
            .contains(&key.as_str()),
            "unexpected field {key} in a learner-facing record"
        );
    }
}

#[test]
fn invalid_configuration_and_time_are_refused() {
    assert!(Calibration::two_point(0.0, 0.0, 0.0, 100.0, "c").is_err());
    assert!(Calibration::two_point(0.0, 5.0, 100.0, 100.0, "c").is_err());
    assert!(Calibration::two_point(0.0, f64::NAN, 100.0, 101.0, "c").is_err());
    assert!(MeasurementDevice::new(Thermometer, 1)
        .unwrap()
        .with_error_std(-0.1)
        .is_err());
    assert!(MeasurementDevice::new(Thermometer, 1)
        .unwrap()
        .with_error_std(f64::NAN)
        .is_err());
    assert!(MeasurementDevice::new(Thermometer, 1)
        .unwrap()
        .with_resolution(0.0)
        .is_err());
    assert!(MeasurementDevice::new(Thermometer, 1)
        .unwrap()
        .with_lag_seconds(0.0)
        .is_err());

    let bench = bench_with(&["add v1 water 100mL"]);
    let mut device = MeasurementDevice::new(Thermometer, 1).unwrap();
    assert!(matches!(
        device.record(&bench.vessels[0], f64::NAN),
        Err(MeasurementError::InvalidTime(_))
    ));
    device.record(&bench.vessels[0], 10.0).unwrap();
    assert!(matches!(
        device.record(&bench.vessels[0], 5.0),
        Err(MeasurementError::TimeReversed { .. })
    ));
}
