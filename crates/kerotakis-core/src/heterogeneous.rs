//! Shared geometry and transport arithmetic for finite-rate surface reactions.
//!
//! Electrodes, dissolving solids and heterogeneous catalysts differ in their
//! rate laws, but not in the conversion from flux per reactive area to extent.
//! Keeping that conversion here prevents each chemistry route from inventing
//! its own interpretation of area, roughness or blocking.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ReactiveSurface {
    /// Projected or otherwise measured macroscopic area.
    pub geometric_area_m2: f64,
    /// Real microscopic area divided by geometric area.
    pub roughness_factor: f64,
    /// Fraction accessible to this particular reaction after coverage.
    pub available_fraction: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceRateError {
    InvalidArea,
    InvalidRoughness,
    InvalidAvailability,
    InvalidFluxOrTime,
    InvalidMass,
    InvalidTransport,
}

impl ReactiveSurface {
    pub fn validate(&self) -> Result<(), SurfaceRateError> {
        if !self.geometric_area_m2.is_finite() || self.geometric_area_m2 < 0.0 {
            return Err(SurfaceRateError::InvalidArea);
        }
        if !self.roughness_factor.is_finite() || self.roughness_factor <= 0.0 {
            return Err(SurfaceRateError::InvalidRoughness);
        }
        if !self.available_fraction.is_finite() || !(0.0..=1.0).contains(&self.available_fraction) {
            return Err(SurfaceRateError::InvalidAvailability);
        }
        Ok(())
    }

    pub fn reactive_area_m2(&self) -> Result<f64, SurfaceRateError> {
        self.validate()?;
        Ok(self.geometric_area_m2 * self.roughness_factor * self.available_fraction)
    }

    pub fn extent_from_flux(
        &self,
        mol_per_m2_s: f64,
        seconds: f64,
    ) -> Result<f64, SurfaceRateError> {
        if !mol_per_m2_s.is_finite() || !seconds.is_finite() || seconds < 0.0 {
            return Err(SurfaceRateError::InvalidFluxOrTime);
        }
        Ok(mol_per_m2_s * self.reactive_area_m2()? * seconds)
    }
}

/// Combine an intrinsic surface flux with a transport ceiling using a smooth
/// resistance-in-series relation. `None` means no measured transport limit.
pub fn transport_limited_flux(intrinsic: f64, limiting: Option<f64>) -> f64 {
    let Some(limit) = limiting else {
        return intrinsic;
    };
    if !intrinsic.is_finite() || !limit.is_finite() || limit <= 0.0 {
        return f64::NAN;
    }
    let magnitude = intrinsic.abs();
    intrinsic.signum() * magnitude * limit / (magnitude + limit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extent_is_extensive_in_area_roughness_and_time() {
        let surface = ReactiveSurface {
            geometric_area_m2: 2.0,
            roughness_factor: 3.0,
            available_fraction: 0.5,
        };
        assert_eq!(surface.reactive_area_m2().unwrap(), 3.0);
        assert_eq!(surface.extent_from_flux(0.25, 4.0).unwrap(), 3.0);
    }

    #[test]
    fn blocking_and_transport_are_monotone() {
        let intrinsic = 10.0;
        let limited = transport_limited_flux(intrinsic, Some(2.0));
        assert!(limited > 0.0 && limited < 2.0);
        assert_eq!(transport_limited_flux(intrinsic, None), intrinsic);
    }

    #[test]
    fn invalid_geometry_is_refused() {
        let surface = ReactiveSurface {
            geometric_area_m2: -1.0,
            roughness_factor: 1.0,
            available_fraction: 1.0,
        };
        assert_eq!(surface.validate(), Err(SurfaceRateError::InvalidArea));
    }
}
