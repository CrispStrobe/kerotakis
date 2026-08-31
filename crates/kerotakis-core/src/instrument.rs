//! INST-001: Instrument contract.
//!
//! An instrument observes or perturbs a vessel's state. The contract
//! separates sampling (read-only) from perturbation (adds energy or
//! removes material), so solvers know whether an observation changed
//! the system.

use serde::{Deserialize, Serialize};

use crate::species::{Phase, SpeciesId};
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

/// INST-004: Conductivity meter — Kohlrausch sum over the solved
/// speciation (see [`crate::conductivity`]), mean-mobility estimate when
/// the solver reported no speciation. `in_range` is honest about both the
/// instrument's span and the model's own validity: a concentrated
/// solution or an uncovered ion reads as out-of-calibration.
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
        let est = crate::conductivity::specific_conductance(sol);
        Some(Reading {
            observable: "conductivity".into(),
            value: est.microsiemens_per_cm,
            unit: "µS/cm".into(),
            precision: Some(1.0),
            in_range: est.trustworthy()
                && est.microsiemens_per_cm > 0.0
                && est.microsiemens_per_cm < 1e6,
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
                if let Some(spectrum) = data.spectrum {
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
    /// The bench's standard column. The numbers are the ones the
    /// instrument-oracle test works by hand (N = 10⁴ plates, t₀ = 60 s,
    /// β = 0.5), so a learner who checks the worked example against the
    /// bench finds the same column in both places.
    pub fn school() -> Self {
        ChromatographyColumn {
            plates: 10_000,
            void_time_s: 60.0,
            phase_ratio: 0.5,
        }
    }

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

// ── EXP-33: the melting-point apparatus ────────────────────────────

/// Which transition the apparatus is set up to find.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransitionRead {
    /// A capillary of dry solid in the block: the melting point.
    Melting,
    /// A flask of liquid with a thermometer in the vapour: the boiling point.
    Boiling,
}

impl TransitionRead {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Melting => "melting point",
            Self::Boiling => "boiling point",
        }
    }
    /// The phase the technique needs in the sample holder.
    fn wanted_phase(&self) -> Phase {
        match self {
            Self::Melting => Phase::Solid,
            Self::Boiling => Phase::Liquid,
        }
    }
}

/// Why the apparatus did or did not produce a sharp number.
///
/// The refusals are the pedagogy. "Mixture: no sharp point" is the single
/// most useful thing a melting-point apparatus tells a chemist, and a bench
/// that answered a mixture with the pure substance's constant would be
/// teaching the opposite of the lesson.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PurityVerdict {
    /// One substance, and the registry has its transition temperature.
    Pure,
    /// More than one substance in the sample holder.
    Mixture,
    /// One substance, but wet — or dissolved in something. The technique
    /// needs an isolated sample, and packing a damp solid is the classic
    /// way to get a low, broad, wrong answer.
    NotIsolated,
    /// One substance, isolated, but no transition temperature is curated.
    NoData,
    /// Nothing of the right phase is in the vessel at all.
    NothingToTest,
}

/// The threshold above which a second substance stops being a trace and
/// starts being a mixture, as a mole fraction of the sample.
///
/// A stated model choice, deliberately stricter than a real apparatus: a
/// capillary broadens visibly somewhere near a mole per cent, so refusing
/// at a tenth of one errs towards calling a sample impure. The engine would
/// rather withhold a sharp number it is not entitled to than print one.
pub const PURITY_TRACE_FRACTION: f64 = 1e-3;

/// What the apparatus found. Everything the renderer needs, including the
/// citation — a transition temperature is curated data, and a number shown
/// without the book it came from is not evidence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransitionReading {
    pub kind: TransitionRead,
    pub verdict: PurityVerdict,
    /// The one substance in the holder, where there is one.
    pub species: Option<SpeciesId>,
    /// The temperature in °C, where the apparatus is entitled to one.
    pub value_c: Option<f64>,
    /// What the sample actually does there — it is not always melting.
    pub outcome: Option<crate::species::TransitionOutcome>,
    /// Per-record citation for `value_c`.
    pub source: Option<String>,
    /// Everything of the wanted phase, in registry order. For a mixture this
    /// is the answer: these are the things that are in the way.
    pub components: Vec<SpeciesId>,
    /// The lowest pure melting point among the components — the bound the
    /// mixture's own range begins below. Direction only; see `boundary`.
    pub lowest_component_c: Option<f64>,
    /// What the reading does NOT claim.
    pub boundary: Option<String>,
}

/// What this apparatus is and is not, said once so every register can say it.
pub const APPARATUS_BOUNDARY: &str =
    "the block does not simulate the melt: it reports the curated literature constant for \
     the substance the bench knows is in the capillary, to the precision a school apparatus \
     resolves. A real determination is the evidence for an identity; here the identity is \
     the evidence for the number, and only the refusals are computed from the sample";

/// The mixture refusal's own boundary — the one number this bench will not
/// invent.
pub const MIXTURE_BOUNDARY: &str =
    "the direction is a law and is claimed: a mixture melts below its lowest-melting pure \
     component and over a range, not at a point. The SIZE of the depression is not claimed — \
     that needs the cryoscopic constant and enthalpy of fusion of the major component, and \
     this registry curates neither for solids other than water";

/// Read a melting or boiling point off a vessel.
///
/// The whole method is: find out whether the sample is one substance, and
/// only then look the constant up.
pub fn read_transition(vessel: &Vessel, kind: TransitionRead) -> TransitionReading {
    let wanted = kind.wanted_phase();
    let mut holder: Vec<(SpeciesId, f64)> = Vec::new();
    let mut elsewhere = 0.0f64;
    for p in &vessel.contents {
        if p.moles.0 <= 0.0 {
            continue;
        }
        if p.phase == wanted {
            match holder.iter_mut().find(|(s, _)| *s == p.species) {
                Some((_, n)) => *n += p.moles.0,
                None => holder.push((p.species.clone(), p.moles.0)),
            }
        } else {
            elsewhere += p.moles.0;
        }
    }
    let total: f64 = holder.iter().map(|(_, n)| n).sum();
    let mut reading = TransitionReading {
        kind,
        verdict: PurityVerdict::NothingToTest,
        species: None,
        value_c: None,
        outcome: None,
        source: None,
        components: holder.iter().map(|(s, _)| s.clone()).collect(),
        lowest_component_c: None,
        boundary: None,
    };
    if total <= 0.0 {
        return reading;
    }

    // The lowest pure point among whatever is in the holder — the mixture
    // message's one grounded number, and harmless to compute either way.
    reading.lowest_component_c = holder
        .iter()
        .filter_map(|(s, _)| {
            let t = crate::species::lookup(s)?.transitions?;
            let (k, _) = match kind {
                TransitionRead::Melting => t.melting_reading()?,
                TransitionRead::Boiling => t.boiling_reading()?,
            };
            Some(k - 273.15)
        })
        .fold(None, |acc: Option<f64>, c| {
            Some(acc.map_or(c, |a| a.min(c)))
        });

    let significant: Vec<&(SpeciesId, f64)> = holder
        .iter()
        .filter(|(_, n)| n / total > PURITY_TRACE_FRACTION)
        .collect();
    if significant.len() > 1 {
        reading.verdict = PurityVerdict::Mixture;
        reading.boundary = Some(MIXTURE_BOUNDARY.to_string());
        return reading;
    }
    let (only, _) = significant
        .first()
        .copied()
        .expect("a positive total has at least one significant component");
    reading.species = Some(only.clone());

    // Anything of another phase in the same vessel means the sample was not
    // isolated: a damp solid, or a liquid with something dissolved in it.
    // Both give a low, broad, wrong answer on a real bench, which is why the
    // technique insists on a dry, isolated sample rather than tolerating one.
    if elsewhere > 0.0 {
        reading.verdict = PurityVerdict::NotIsolated;
        reading.boundary = Some(MIXTURE_BOUNDARY.to_string());
        return reading;
    }

    let Some(data) = crate::species::lookup(only) else {
        reading.verdict = PurityVerdict::NoData;
        return reading;
    };
    let found = data.transitions.and_then(|t| match kind {
        TransitionRead::Melting => t.melting_reading().map(|r| (r, t)),
        TransitionRead::Boiling => t.boiling_reading().map(|r| (r, t)),
    });
    match found {
        Some(((k, outcome), t)) => {
            reading.verdict = PurityVerdict::Pure;
            reading.value_c = Some(k - 273.15);
            reading.outcome = Some(outcome);
            reading.source = Some(t.source.to_string());
            reading.boundary = Some(match t.boundary {
                Some(extra) => format!("{APPARATUS_BOUNDARY}. {extra}"),
                None => APPARATUS_BOUNDARY.to_string(),
            });
        }
        None => reading.verdict = PurityVerdict::NoData,
    }
    reading
}

/// The INST-001 face of the apparatus: a scalar reading where there is one.
pub struct MeltingPointApparatus(pub TransitionRead);

impl InstrumentContract for MeltingPointApparatus {
    fn name(&self) -> &'static str {
        match self.0 {
            TransitionRead::Melting => "melting-point apparatus",
            TransitionRead::Boiling => "boiling-point apparatus",
        }
    }

    fn applies(&self, vessel: &Vessel) -> bool {
        !matches!(
            read_transition(vessel, self.0).verdict,
            PurityVerdict::NothingToTest
        )
    }

    fn mode(&self) -> InstrumentMode {
        // The capillary is charged from the bulk and the bulk is untouched:
        // the sample in the tube is an aliquot too small for the ledger.
        InstrumentMode::Passive
    }

    fn measure(&self, vessel: &Vessel) -> Option<Reading> {
        let reading = read_transition(vessel, self.0);
        Some(Reading {
            observable: reading.kind.as_str().to_string(),
            value: reading.value_c?,
            unit: "°C".into(),
            // A school block resolves half a degree; the curated constants
            // are quoted no finer, so claiming more would be theatre.
            precision: Some(0.5),
            in_range: true,
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
