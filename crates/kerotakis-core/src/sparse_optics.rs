//! Reviewed single-wavelength optical data with explicit experimental conditions.
//!
//! This module does not infer chromophore concentration from a vessel. A caller
//! must independently establish free molecular I2 concentration and explicitly
//! assert the experimental medium. Certification evidence is provenance supplied
//! by the caller, not proof verified by this library. Analytical iodine, triiodide,
//! HOI and starch/polyiodide inventories are not substitutes for free I2.
//!
//! The historical point has no validated concentration sweep. Beer–Lambert
//! calculations are finite arithmetic predictions; their mathematical validity
//! does not establish empirical calibration at arbitrary concentrations. In
//! particular, numerical extreme controls test the arithmetic, not spectroscopy.
//! No point is interpolated to a nearby wavelength, temperature or solvent; the
//! existing complete-spectrum and vessel-speciation qualifications remain intact.

use serde::{Deserialize, Serialize};
use std::{fmt, sync::OnceLock};

/// Explicit solvent identity. Only the reviewed aqueous medium is supported.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OpticalSolvent {
    Water,
    Hexane,
}

/// The caller asserts these documented background-medium concentrations.
/// This assertion does not certify unknown additional constituents or mixtures.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExperimentalMedium {
    pub solvent: OpticalSolvent,
    pub perchloric_acid_molar: f64,
    pub potassium_iodate_molar: f64,
}

/// Provenance for a caller-certified free-I2 concentration, with private fields
/// so unchecked negative/nonfinite concentrations cannot enter the calculation.
/// This is deliberately not deserializable or constructible from vessel state.
#[derive(Clone, Debug)]
pub struct CertifiedFreeIodine {
    concentration_molar: f64,
    evidence: String,
}

impl CertifiedFreeIodine {
    /// Assert an independently measured or independently validated free-I2
    /// concentration. Nonempty evidence identifies that work; the library cannot
    /// check its truth or validate chemical speciation from this text.
    pub fn from_independent_measurement(
        concentration_molar: f64,
        evidence: &str,
    ) -> Result<Self, SparseOpticsError> {
        if !concentration_molar.is_finite() || concentration_molar < 0.0 {
            return Err(SparseOpticsError::InvalidConcentration);
        }
        if evidence.trim().is_empty() {
            return Err(SparseOpticsError::MissingCertification);
        }
        Ok(Self {
            concentration_molar,
            evidence: evidence.to_owned(),
        })
    }

    pub fn concentration_molar(&self) -> f64 {
        self.concentration_molar
    }

    pub fn certification_evidence(&self) -> &str {
        &self.evidence
    }
}

/// Provenance-bearing measured point. No spectral shape is implied.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReviewedOpticalPoint {
    pub species: String,
    pub wavelength_nm: f64,
    pub epsilon_l_mol_cm: f64,
    pub temperature_k: f64,
    pub medium: ExperimentalMedium,
    pub source_doi: String,
    pub source_citation: String,
    pub source_url: String,
    pub published_initial_iodine_loading_molar_approx: f64,
    pub validated_concentration_range_molar: Option<[f64; 2]>,
    pub preparation: String,
    pub limitations: Vec<String>,
}

/// Read the bundled machine-readable primary-source record. Loading a point does
/// not close missing-spectrum:I2 or certify concentrations in any vessel.
pub fn reviewed_iodine_record() -> &'static ReviewedOpticalPoint {
    static RECORD: OnceLock<ReviewedOpticalPoint> = OnceLock::new();
    RECORD.get_or_init(|| {
        serde_json::from_str(include_str!("../data/optics/aqueous-iodine-460nm.json"))
            .expect("bundled reviewed optical point must be valid JSON")
    })
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SparseAbsorbance {
    pub absorbance: f64,
    pub wavelength_nm: f64,
    pub temperature_k: f64,
    pub epsilon_l_mol_cm: f64,
    pub free_iodine_molar: f64,
    pub path_cm: f64,
    pub medium: ExperimentalMedium,
    pub certification_evidence: String,
    pub source_doi: String,
    /// Always explicitly qualified: the record contains no calibration sweep.
    pub concentration_validity: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SparseOpticsError {
    InvalidConcentration,
    MissingCertification,
    InvalidPath,
    UnsupportedMedium,
    UnsupportedTemperature,
    UnsupportedWavelength,
    UnrepresentableAbsorbance,
}

impl fmt::Display for SparseOpticsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidConcentration => "free-I2 concentration must be finite and nonnegative",
            Self::MissingCertification => {
                "independent free-I2 concentration provenance is required"
            }
            Self::InvalidPath => "optical path must be finite and positive",
            Self::UnsupportedMedium => "experimental medium has no reviewed molecular-I2 point",
            Self::UnsupportedTemperature => "temperature has no reviewed molecular-I2 point",
            Self::UnsupportedWavelength => "wavelength has no reviewed molecular-I2 point",
            Self::UnrepresentableAbsorbance => {
                "positive absorbance is not representable as a finite number"
            }
        };
        f.write_str(message)
    }
}
impl std::error::Error for SparseOpticsError {}

/// Calculate A=epsilon*c*l at the exact reviewed point and medium. Positive
/// underflow and final overflow are explicit refusals, never a false clear sample.
/// Reordering multiplication can recover a representable final product when one
/// intermediate ordering overflows or underflows. It supplies no calibration
/// guarantee outside the published sample and does not produce a full spectrum.
pub fn measure_free_iodine(
    certified: &CertifiedFreeIodine,
    medium: ExperimentalMedium,
    temperature_k: f64,
    wavelength_nm: f64,
    path_cm: f64,
) -> Result<SparseAbsorbance, SparseOpticsError> {
    if !path_cm.is_finite() || path_cm <= 0.0 {
        return Err(SparseOpticsError::InvalidPath);
    }
    let record = reviewed_iodine_record();
    if medium != record.medium {
        return Err(SparseOpticsError::UnsupportedMedium);
    }
    if temperature_k != record.temperature_k {
        return Err(SparseOpticsError::UnsupportedTemperature);
    }
    if wavelength_nm != record.wavelength_nm {
        return Err(SparseOpticsError::UnsupportedWavelength);
    }
    let c = certified.concentration_molar;
    let epsilon = record.epsilon_l_mol_cm;
    let absorbance = if c == 0.0 {
        0.0
    } else {
        [
            c * epsilon * path_cm,
            c * path_cm * epsilon,
            epsilon * path_cm * c,
        ]
        .into_iter()
        .find(|a| a.is_finite() && *a > 0.0)
        .ok_or(SparseOpticsError::UnrepresentableAbsorbance)?
    };
    if c > 0.0 && absorbance < f64::MIN_POSITIVE {
        // With a subnormal final result, dividing by the smaller input first
        // stays finite and exposes the final rounding quantum without creating
        // another quantized subnormal intermediate.
        let ratio = absorbance / c.min(path_cm) / c.max(path_cm) / epsilon;
        if !ratio.is_finite() || (ratio - 1.0).abs() > 1e-8 {
            return Err(SparseOpticsError::UnrepresentableAbsorbance);
        }
    }
    Ok(SparseAbsorbance {
        absorbance,
        wavelength_nm,
        temperature_k,
        epsilon_l_mol_cm: epsilon,
        free_iodine_molar: c,
        path_cm,
        medium,
        certification_evidence: certified.evidence.clone(),
        source_doi: record.source_doi.clone(),
        concentration_validity: "No validated concentration sweep; Beer-Lambert scaling is mathematical extrapolation beyond the published sample.".to_owned(),
    })
}
