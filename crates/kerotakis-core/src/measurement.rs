//! MEAS-001: calibrated, persisted, reproducible measurement records.
//!
//! The ideal instruments in [`crate::instrument`] observe state exactly.
//! This layer wraps them with what a real bench adds: a calibration the
//! learner performed, declared resolution and error, a response that takes
//! time to settle, and a record stream whose noise replays bit-identically
//! after a save/load.
//!
//! Deliberate boundaries:
//!
//! * The physical model's validity is NOT folded in here. The record keeps
//!   the instrument's own `in_range` flag and nothing else; a caller who
//!   knows the chemistry behind a reading was outside the model's validated
//!   domain reports that separately rather than through this layer.
//! * Error widths are declared instructional settings, never empirical
//!   manufacturer specifications, and they stay distinct from calibration
//!   offsets and from model inadequacy, which this layer cannot see.
//! * Noise is indexed, not streamed: sample *n* is drawn from a fresh
//!   generator seeded from the device seed and *n*. A record stream is
//!   therefore reproducible after a save/load round trip with no hidden
//!   RNG state, and a failed reading (which never reaches a sample)
//!   consumes none of the stream.
//! * A learner-facing record carries no sample composition and no
//!   instrument internals — only what the experimenter could know.

use serde::{Deserialize, Serialize};

use crate::instrument::InstrumentContract;
use crate::vessel::Vessel;

/// A two-point affine calibration: two references of known value and the
/// raw readings the instrument gave for them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Calibration {
    pub reference_low: f64,
    pub reference_high: f64,
    pub raw_low: f64,
    pub raw_high: f64,
    pub id: String,
}

impl Calibration {
    /// Build from two (reference, raw) pairs. Both spans must be finite and
    /// nonzero: a calibration that does not span anything cannot correct
    /// anything, and a NaN input would silently poison every later record.
    pub fn two_point(
        reference_low: f64,
        reference_high: f64,
        raw_low: f64,
        raw_high: f64,
        id: &str,
    ) -> Result<Self, MeasurementError> {
        for value in [reference_low, reference_high, raw_low, raw_high] {
            if !value.is_finite() {
                return Err(MeasurementError::Config(
                    "calibration points must be finite".into(),
                ));
            }
        }
        if reference_high == reference_low {
            return Err(MeasurementError::Config(
                "the two references must differ".into(),
            ));
        }
        if raw_high == raw_low {
            return Err(MeasurementError::Config(
                "the two raw readings must differ".into(),
            ));
        }
        Ok(Self {
            reference_low,
            reference_high,
            raw_low,
            raw_high,
            id: id.to_string(),
        })
    }

    fn correct(&self, raw: f64) -> f64 {
        self.reference_low
            + (raw - self.raw_low) * (self.reference_high - self.reference_low)
                / (self.raw_high - self.raw_low)
    }
}

/// Everything a restored device needs. Contains no instrument identity and
/// no sample composition — exactly what save/load must persist.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviceState {
    seed: u64,
    sample_index: u64,
    last_elapsed_seconds: Option<f64>,
    response_value: Option<f64>,
    #[serde(default)]
    error_std: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    resolution: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    lag_seconds: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    calibration: Option<Calibration>,
}

/// One learner-facing measurement record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasurementRecord {
    pub observable: String,
    pub value: f64,
    pub unit: String,
    pub precision: Option<f64>,
    pub in_range: bool,
    pub elapsed_seconds: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub calibration_id: Option<String>,
    pub sample_index: u64,
}

/// Why a measurement could not be taken or configured.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum MeasurementError {
    #[error("the instrument produced no reading for this state")]
    NoReading,
    #[error("elapsed time must be finite, got {0}")]
    InvalidTime(f64),
    #[error("time went backwards: {requested} s follows {previous} s")]
    TimeReversed { requested: f64, previous: f64 },
    #[error("{0}")]
    Config(String),
}

/// A real instrument wrapped with calibration, declared error, resolution
/// and lag. The wrapped instrument stays ideal; everything measured through
/// this device is the instrument model's answer, not the state itself.
pub struct MeasurementDevice {
    instrument: Box<dyn InstrumentContract>,
    state: DeviceState,
}

impl MeasurementDevice {
    pub fn new(
        instrument: impl InstrumentContract + 'static,
        seed: u64,
    ) -> Result<Self, MeasurementError> {
        Ok(Self {
            instrument: Box::new(instrument),
            state: DeviceState {
                seed,
                sample_index: 0,
                last_elapsed_seconds: None,
                response_value: None,
                error_std: 0.0,
                resolution: None,
                lag_seconds: None,
                calibration: None,
            },
        })
    }

    /// Rebuild a device from its persisted state. The instrument is supplied
    /// by the caller because the contract is code, not data.
    pub fn from_state(
        instrument: impl InstrumentContract + 'static,
        state: &DeviceState,
    ) -> Result<Self, MeasurementError> {
        let mut device = Self::new(instrument, state.seed)?;
        device = device.with_error_std(state.error_std)?;
        if let Some(resolution) = state.resolution {
            device = device.with_resolution(resolution)?;
        }
        if let Some(lag) = state.lag_seconds {
            device = device.with_lag_seconds(lag)?;
        }
        if let Some(calibration) = &state.calibration {
            device = device.with_calibration(calibration.clone())?;
        }
        if state.last_elapsed_seconds.is_some_and(|t| !t.is_finite())
            || state.response_value.is_some_and(|v| !v.is_finite())
            || state.last_elapsed_seconds.is_some() != state.response_value.is_some()
        {
            return Err(MeasurementError::Config(
                "invalid persisted response state".into(),
            ));
        }
        device.state = state.clone();
        Ok(device)
    }

    pub fn state(&self) -> &DeviceState {
        &self.state
    }

    /// Declared random-error width in the reading's unit. Zero means an
    /// errorless (but still calibrated and quantized) instrument.
    pub fn with_error_std(mut self, std: f64) -> Result<Self, MeasurementError> {
        if !std.is_finite() || std < 0.0 {
            return Err(MeasurementError::Config(
                "error standard deviation must be finite and non-negative".into(),
            ));
        }
        self.state.error_std = std;
        Ok(self)
    }

    /// Display resolution: recorded values land on this grid.
    pub fn with_resolution(mut self, resolution: f64) -> Result<Self, MeasurementError> {
        if !resolution.is_finite() || resolution <= 0.0 {
            return Err(MeasurementError::Config(
                "resolution must be finite and positive".into(),
            ));
        }
        self.state.resolution = Some(resolution);
        Ok(self)
    }

    /// First-order response time: the display relaxes toward the target
    /// with time constant `lag_seconds`.
    pub fn with_lag_seconds(mut self, seconds: f64) -> Result<Self, MeasurementError> {
        if !seconds.is_finite() || seconds <= 0.0 {
            return Err(MeasurementError::Config(
                "lag time constant must be finite and positive".into(),
            ));
        }
        self.state.lag_seconds = Some(seconds);
        Ok(self)
    }

    pub fn with_calibration(mut self, calibration: Calibration) -> Result<Self, MeasurementError> {
        let calibration = Calibration::two_point(
            calibration.reference_low,
            calibration.reference_high,
            calibration.raw_low,
            calibration.raw_high,
            &calibration.id,
        )?;
        self.state.calibration = Some(calibration);
        Ok(self)
    }

    /// Take one record at the given elapsed bench time. A failed reading
    /// leaves the device's time, sample index and response untouched.
    pub fn record(
        &mut self,
        vessel: &Vessel,
        elapsed_seconds: f64,
    ) -> Result<MeasurementRecord, MeasurementError> {
        if !elapsed_seconds.is_finite() {
            return Err(MeasurementError::InvalidTime(elapsed_seconds));
        }
        if let Some(previous) = self.state.last_elapsed_seconds {
            if elapsed_seconds < previous {
                return Err(MeasurementError::TimeReversed {
                    requested: elapsed_seconds,
                    previous,
                });
            }
        }
        let reading = self
            .instrument
            .measure(vessel)
            .ok_or(MeasurementError::NoReading)?;

        let corrected = match &self.state.calibration {
            Some(calibration) => calibration.correct(reading.value),
            None => reading.value,
        };

        // The lagged response relaxes toward the (calibrated) target. The
        // very first reading displays the target: an instrument switched on
        // has been sitting in the room since before the experiment began.
        let response = match (self.state.response_value, self.state.lag_seconds) {
            (Some(previous), Some(tau)) => {
                let dt = elapsed_seconds - self.state.last_elapsed_seconds.unwrap_or(previous);
                previous + (corrected - previous) * (1.0 - (-dt / tau).exp())
            }
            (Some(previous), None) => {
                let _ = previous;
                corrected
            }
            (None, _) => corrected,
        };

        let noise = if self.state.error_std > 0.0 {
            // Indexed stream: sample n is drawn from a generator seeded by
            // (seed, n). Deterministic, serialisable, and a failed reading
            // (which never reaches here) consumes nothing.
            use rand::{Rng, SeedableRng};
            let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(
                self.state
                    .seed
                    .wrapping_add(self.state.sample_index.wrapping_mul(0x9E37_79B9_7F4A_7C15)),
            );
            rng.sample::<f64, _>(rand_distr::Normal::new(0.0, self.state.error_std).unwrap())
        } else {
            0.0
        };

        let mut value = response + noise;
        if let Some(resolution) = self.state.resolution {
            value = (value / resolution).round() * resolution;
        }

        if !reading.value.is_finite()
            || !corrected.is_finite()
            || !response.is_finite()
            || !value.is_finite()
        {
            return Err(MeasurementError::Config(
                "non-finite instrument response".into(),
            ));
        }
        let next_index = self
            .state
            .sample_index
            .checked_add(1)
            .ok_or_else(|| MeasurementError::Config("sample index exhausted".into()))?;
        let record = MeasurementRecord {
            observable: reading.observable,
            value,
            unit: reading.unit,
            precision: reading.precision,
            in_range: reading.in_range,
            elapsed_seconds,
            calibration_id: self.state.calibration.as_ref().map(|c| c.id.clone()),
            sample_index: self.state.sample_index,
        };

        self.state.response_value = Some(response);
        self.state.last_elapsed_seconds = Some(elapsed_seconds);
        self.state.sample_index = next_index;
        Ok(record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instrument::Thermometer;

    fn warm_vessel() -> Vessel {
        let mut vessel = Vessel::new(crate::VesselId(0), "beaker");
        vessel.temperature = crate::Kelvin::from_celsius(25.0);
        vessel
    }

    #[test]
    fn sample_index_advances_only_on_success() {
        let mut device = MeasurementDevice::new(Thermometer, 3).unwrap();
        // An empty vessel reads through the thermometer just fine, so the
        // stream advances; NoReading is the pH meter's contract.
        let first = device.record(&Vessel::new(crate::VesselId(9), "empty"), 0.0);
        assert!(first.is_ok(), "{first:?}");

        let mut ph = MeasurementDevice::new(crate::instrument::PhMeter, 4).unwrap();
        assert!(matches!(
            ph.record(&warm_vessel(), 0.0),
            Err(MeasurementError::NoReading)
        ));
        assert_eq!(ph.state().last_elapsed_seconds, None);
        assert_eq!(ph.state().sample_index, 0);
    }
}
