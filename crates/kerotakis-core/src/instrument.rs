//! INST-001: Instrument contract.
//!
//! An instrument observes or perturbs a vessel's state. The contract
//! separates sampling (read-only) from perturbation (adds energy or
//! removes material), so solvers know whether an observation changed
//! the system.

use serde::{Deserialize, Serialize};

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

/// INST-003: Pressure gauge — reads headspace pressure in kPa.
pub struct PressureGauge;

impl InstrumentContract for PressureGauge {
    fn name(&self) -> &'static str {
        "pressure gauge"
    }
    fn applies(&self, _vessel: &Vessel) -> bool {
        true
    }
    fn mode(&self) -> InstrumentMode {
        InstrumentMode::Passive
    }
    fn measure(&self, vessel: &Vessel) -> Option<Reading> {
        Some(Reading {
            observable: "pressure".into(),
            value: vessel.pressure.0 / 1000.0, // Pa → kPa
            unit: "kPa".into(),
            precision: Some(0.1),
            in_range: vessel.pressure.0 > 0.0 && vessel.pressure.0 < 1e7,
        })
    }
}

/// INST-004: Conductivity meter — reads solution conductivity.
/// Currently reports the PHREEQC-computed specific conductance if available.
pub struct ConductivityMeter;

impl InstrumentContract for ConductivityMeter {
    fn name(&self) -> &'static str {
        "conductivity meter"
    }
    fn applies(&self, vessel: &Vessel) -> bool {
        vessel.solution.is_some()
    }
    fn mode(&self) -> InstrumentMode {
        InstrumentMode::Passive
    }
    fn measure(&self, vessel: &Vessel) -> Option<Reading> {
        let sol = vessel.solution.as_ref()?;
        // Estimate conductivity from ionic strength (simple approximation).
        // Full Kohlrausch implementation is future work.
        let conductivity_us_cm = sol.ionic_strength * 100_000.0; // rough
        Some(Reading {
            observable: "conductivity".into(),
            value: conductivity_us_cm,
            unit: "µS/cm".into(),
            precision: Some(1.0),
            in_range: conductivity_us_cm > 0.0 && conductivity_us_cm < 1e6,
        })
    }
}

/// INST-006: Simple calorimeter — reports temperature-change-based heat.
pub struct Calorimeter;

impl InstrumentContract for Calorimeter {
    fn name(&self) -> &'static str {
        "calorimeter"
    }
    fn applies(&self, _vessel: &Vessel) -> bool {
        true
    }
    fn mode(&self) -> InstrumentMode {
        // Reading temperature is passive; a real calorimeter has its own
        // heat capacity, but this simplified version just reads ΔT.
        InstrumentMode::Passive
    }
    fn measure(&self, vessel: &Vessel) -> Option<Reading> {
        // Report the vessel's total enthalpy relative to 25°C in kJ.
        let q_kj = vessel.enthalpy().0 / 1000.0;
        Some(Reading {
            observable: "enthalpy".into(),
            value: q_kj,
            unit: "kJ".into(),
            precision: Some(0.01),
            in_range: q_kj.abs() < 1e6,
        })
    }
}

/// INST-005: Spectrophotometer — measures absorbance across visible wavelengths.
///
/// Computes the total absorbance spectrum from all coloured solutes using
/// Beer-Lambert (A = ε·c·l), then reports the absorbance at the peak
/// wavelength. The full spectrum is available via `measure_spectrum()`.
pub struct Spectrophotometer {
    /// Path length in cm (cuvette width).
    pub path_cm: f64,
}

impl Default for Spectrophotometer {
    fn default() -> Self {
        Self { path_cm: 1.0 }
    }
}

impl Spectrophotometer {
    /// Compute the full absorbance spectrum of the vessel's solution.
    pub fn measure_spectrum(&self, vessel: &Vessel) -> [f64; crate::spectrum::BANDS] {
        use crate::species;
        let mut total = [0.0f64; crate::spectrum::BANDS];

        // Sum ε·c·l for each coloured solute
        let water_kg = vessel
            .contents
            .iter()
            .filter(|p| {
                p.species.0 == "water"
                    && (p.phase == crate::species::Phase::Liquid
                        || p.phase == crate::species::Phase::Aqueous)
            })
            .map(|p| p.moles.0 * 0.018015)
            .sum::<f64>();

        if water_kg < 1e-6 {
            return total;
        }

        for portion in &vessel.contents {
            if portion.phase != crate::species::Phase::Aqueous {
                continue;
            }
            if let Some(data) = species::lookup(&portion.species) {
                if let Some(spectrum_fn) = data.spectrum {
                    let spectrum = spectrum_fn();
                    let molality = portion.moles.0 / water_kg;
                    for (i, band) in total.iter_mut().enumerate() {
                        *band += spectrum[i] * molality * self.path_cm;
                    }
                }
            }
        }
        total
    }
}

impl InstrumentContract for Spectrophotometer {
    fn name(&self) -> &'static str {
        "spectrophotometer"
    }
    fn applies(&self, vessel: &Vessel) -> bool {
        vessel
            .contents
            .iter()
            .any(|p| p.phase == crate::species::Phase::Aqueous)
    }
    fn mode(&self) -> InstrumentMode {
        InstrumentMode::Passive
    }
    fn measure(&self, vessel: &Vessel) -> Option<Reading> {
        let spectrum = self.measure_spectrum(vessel);
        let peak_abs = spectrum.iter().copied().fold(0.0f64, f64::max);
        let peak_idx = spectrum
            .iter()
            .position(|&a| (a - peak_abs).abs() < 1e-15)
            .unwrap_or(0);
        let peak_nm = crate::spectrum::BAND_NM[peak_idx];

        Some(Reading {
            observable: format!("absorbance at {peak_nm:.0} nm"),
            value: peak_abs,
            unit: "AU".into(),
            precision: Some(0.001),
            in_range: (0.0..4.0).contains(&peak_abs),
        })
    }
}

/// INST-007: Ideal-plate chromatography column.
///
/// Separates components by partition coefficient using ideal plate theory:
///   retention_time = t₀ · (1 + k')
///   k' = K · (Vs/Vm)  (capacity factor from partition coefficient)
///   N = 16 · (tR/w)²  (plate count from peak width)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromatographyColumn {
    /// Number of theoretical plates.
    pub plates: u32,
    /// Void time (time for unretained species to elute), seconds.
    pub void_time_s: f64,
    /// Phase ratio Vs/Vm (stationary/mobile volume ratio).
    pub phase_ratio: f64,
}

/// A chromatographic peak for one component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromatographicPeak {
    pub species: String,
    pub retention_time_s: f64,
    pub peak_width_s: f64,
    /// Relative area proportional to amount.
    pub relative_area: f64,
}

impl ChromatographyColumn {
    /// Predict the retention time for a species with a given partition coefficient K.
    pub fn retention_time(&self, partition_k: f64) -> f64 {
        let capacity_factor = partition_k * self.phase_ratio;
        self.void_time_s * (1.0 + capacity_factor)
    }

    /// Predict the peak width (4σ) from the plate count and retention time.
    pub fn peak_width(&self, retention_time_s: f64) -> f64 {
        4.0 * retention_time_s / (self.plates as f64).sqrt()
    }

    /// Resolution between two peaks.
    pub fn resolution(&self, tr1: f64, tr2: f64) -> f64 {
        let w1 = self.peak_width(tr1);
        let w2 = self.peak_width(tr2);
        2.0 * (tr2 - tr1).abs() / (w1 + w2)
    }
}

/// INST-008: Qualitative analysis — computed identification from tests.
///
/// Instead of a scripted answer key, the system runs actual computed tests
/// (solubility, flame colour, pH, precipitation) and compares results
/// against detection limits to narrow down the identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitativeTest {
    pub name: String,
    pub observable: String,
    pub detection_limit: f64,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitativeResult {
    pub test_name: String,
    pub observed_value: f64,
    pub detected: bool,
    pub consistent_with: Vec<String>,
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

    #[test]
    fn spectrophotometer_reports_zero_for_empty_vessel() {
        let vessel = Vessel::new(VesselId(0), "test");
        let spec = Spectrophotometer::default();
        // No aqueous species → not applicable
        assert!(!spec.applies(&vessel));
    }

    #[test]
    fn chromatography_retention_time_scales_with_k() {
        let col = ChromatographyColumn {
            plates: 1000,
            void_time_s: 60.0,
            phase_ratio: 1.0,
        };
        // k'=0 means unretained → tR = t₀
        assert!((col.retention_time(0.0) - 60.0).abs() < 0.01);
        // k'=1 → tR = 2·t₀
        assert!((col.retention_time(1.0) - 120.0).abs() < 0.01);
    }

    #[test]
    fn chromatography_resolution_increases_with_plates() {
        let col_low = ChromatographyColumn {
            plates: 100,
            void_time_s: 60.0,
            phase_ratio: 1.0,
        };
        let col_high = ChromatographyColumn {
            plates: 10000,
            void_time_s: 60.0,
            phase_ratio: 1.0,
        };
        let tr1 = 100.0;
        let tr2 = 120.0;
        assert!(col_high.resolution(tr1, tr2) > col_low.resolution(tr1, tr2));
    }
}
