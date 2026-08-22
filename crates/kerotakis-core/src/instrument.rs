//! INST-001: Instrument contract.
//!
//! An instrument observes or perturbs a vessel's state. The contract
//! separates sampling (read-only) from perturbation (adds energy or
//! removes material), so solvers know whether an observation changed
//! the system.

use serde::{Deserialize, Serialize};

use crate::units::{Joules, Kelvin};
use crate::vessel::Vessel;

/// What an instrument does to the system when it measures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstrumentMode {
    /// Pure observation — no state change.
    Passive,
    /// The measurement perturbs the system (e.g. calorimetry adds energy,
    /// sampling removes material).
    Perturbative,
}

/// A single reading from an instrument.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reading {
    /// What was measured (e.g. "temperature", "pH", "mass", "absorbance").
    pub observable: String,
    /// Numeric value in the instrument's native unit.
    pub value: f64,
    /// Unit symbol (e.g. "°C", "g", "pH").
    pub unit: String,
    /// Precision of the instrument (± in the same unit).
    #[serde(default)]
    pub precision: Option<f64>,
    /// Whether the reading is within the instrument's calibrated range.
    pub in_range: bool,
}

/// The instrument contract: every instrument can describe itself,
/// check applicability, and take a reading.
pub trait InstrumentContract {
    /// Human-readable name (e.g. "thermometer", "pH meter", "balance").
    fn name(&self) -> &'static str;

    /// Whether this instrument applies to the given vessel state.
    fn applies(&self, vessel: &Vessel) -> bool;

    /// Whether the measurement perturbs the system.
    fn mode(&self) -> InstrumentMode;

    /// Take a reading from the vessel.
    fn measure(&self, vessel: &Vessel) -> Option<Reading>;
}

// ── Built-in instruments (INST-002 migration targets) ──────────────

/// A simple thermometer: reads temperature, no perturbation.
pub struct Thermometer;

impl InstrumentContract for Thermometer {
    fn name(&self) -> &'static str {
        "thermometer"
    }

    fn applies(&self, _vessel: &Vessel) -> bool {
        true
    }

    fn mode(&self) -> InstrumentMode {
        InstrumentMode::Passive
    }

    fn measure(&self, vessel: &Vessel) -> Option<Reading> {
        Some(Reading {
            observable: "temperature".into(),
            value: vessel.temperature.0 - 273.15,
            unit: "°C".into(),
            precision: Some(0.1),
            in_range: vessel.temperature.0 > 233.15 && vessel.temperature.0 < 573.15,
        })
    }
}

/// A laboratory balance: reads total mass, no perturbation.
pub struct Balance;

impl InstrumentContract for Balance {
    fn name(&self) -> &'static str {
        "balance"
    }

    fn applies(&self, _vessel: &Vessel) -> bool {
        true
    }

    fn mode(&self) -> InstrumentMode {
        InstrumentMode::Passive
    }

    fn measure(&self, vessel: &Vessel) -> Option<Reading> {
        Some(Reading {
            observable: "mass".into(),
            value: vessel.mass().0,
            unit: "g".into(),
            precision: Some(0.01),
            in_range: true,
        })
    }
}

/// A pH meter: reads pH from the solved aqueous state.
pub struct PhMeter;

impl InstrumentContract for PhMeter {
    fn name(&self) -> &'static str {
        "pH meter"
    }

    fn applies(&self, vessel: &Vessel) -> bool {
        vessel.solution.is_some()
    }

    fn mode(&self) -> InstrumentMode {
        InstrumentMode::Passive
    }

    fn measure(&self, vessel: &Vessel) -> Option<Reading> {
        let sol = vessel.solution.as_ref()?;
        Some(Reading {
            observable: "pH".into(),
            value: sol.ph,
            unit: "pH".into(),
            precision: Some(0.01),
            in_range: sol.ph > 0.0 && sol.ph < 14.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vessel::VesselId;

    #[test]
    fn thermometer_reads_room_temperature() {
        let vessel = Vessel::new(VesselId(0), "test");
        let reading = Thermometer.measure(&vessel).unwrap();
        assert_eq!(reading.observable, "temperature");
        assert!((reading.value - 25.0).abs() < 0.1);
        assert!(reading.in_range);
    }

    #[test]
    fn ph_meter_requires_solution() {
        let vessel = Vessel::new(VesselId(0), "test");
        assert!(!PhMeter.applies(&vessel));
        assert!(PhMeter.measure(&vessel).is_none());
    }

    #[test]
    fn balance_reads_zero_for_empty_vessel() {
        let vessel = Vessel::new(VesselId(0), "test");
        let reading = Balance.measure(&vessel).unwrap();
        assert_eq!(reading.observable, "mass");
        assert!(reading.value.abs() < 0.001);
    }
}
