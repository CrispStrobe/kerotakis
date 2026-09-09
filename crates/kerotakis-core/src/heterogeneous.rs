//! Shared geometry and transport arithmetic for finite-rate surface reactions.
//!
//! Electrodes, dissolving solids and heterogeneous catalysts differ in their
//! rate laws, but not in the conversion from flux per reactive area to extent.
//! Keeping that conversion here prevents each chemistry route from inventing
//! its own interpretation of area, roughness or blocking.

use serde::{Deserialize, Serialize};

use crate::constants::FARADAY;

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

/// A stagnant-film mass-transfer model in canonical SI units.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DiffusionLayerTransport {
    pub diffusivity_m2_per_s: f64,
    pub bulk_concentration_mol_per_m3: f64,
    pub diffusion_layer_m: f64,
}

/// Result of the exact lumped diffusion-layer balance under a constant
/// interfacial production flux. A depletion boundary ends the accepted slice;
/// continuing at the requested flux would make concentration negative.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DiffusionLayerAdvance {
    pub surface_concentration_mol_per_m3: f64,
    pub accepted_seconds: f64,
    pub depleted: bool,
}

impl DiffusionLayerTransport {
    pub fn molar_flux_limit(self) -> Result<f64, SurfaceRateError> {
        if !self.diffusivity_m2_per_s.is_finite()
            || self.diffusivity_m2_per_s <= 0.0
            || !self.bulk_concentration_mol_per_m3.is_finite()
            || self.bulk_concentration_mol_per_m3 < 0.0
            || !self.diffusion_layer_m.is_finite()
            || self.diffusion_layer_m <= 0.0
        {
            return Err(SurfaceRateError::InvalidTransport);
        }
        Ok(self.diffusivity_m2_per_s * self.bulk_concentration_mol_per_m3 / self.diffusion_layer_m)
    }

    pub fn current_density_limit(self, electrons_per_mole: f64) -> Result<f64, SurfaceRateError> {
        if !electrons_per_mole.is_finite() || electrons_per_mole <= 0.0 {
            return Err(SurfaceRateError::InvalidTransport);
        }
        Ok(electrons_per_mole * FARADAY * self.molar_flux_limit()?)
    }

    /// Surface concentration under a positive reactant-consumption flux.
    pub fn surface_concentration(
        self,
        consumed_mol_per_m2_s: f64,
    ) -> Result<f64, SurfaceRateError> {
        if !consumed_mol_per_m2_s.is_finite() || consumed_mol_per_m2_s < 0.0 {
            return Err(SurfaceRateError::InvalidTransport);
        }
        Ok((self.bulk_concentration_mol_per_m3
            - consumed_mol_per_m2_s * self.diffusion_layer_m / self.diffusivity_m2_per_s)
            .max(0.0))
    }

    /// Advance the well-mixed Nernst diffusion layer exactly for a constant
    /// net production flux. Positive flux produces the species at the surface;
    /// negative flux consumes it. The model solves
    /// `dc/dt = D (c_bulk - c) / delta^2 + flux / delta`.
    /// This is the Nernst-layer mass balance used by Hankins, Yablonsky & Kiss,
    /// PLoS ONE 12 e0173786 (2017), DOI 10.1371/journal.pone.0173786 (CC BY 4.0).
    pub fn advance_constant_flux(
        self,
        initial_surface_concentration_mol_per_m3: f64,
        net_production_mol_per_m2_s: f64,
        seconds: f64,
    ) -> Result<DiffusionLayerAdvance, SurfaceRateError> {
        if !initial_surface_concentration_mol_per_m3.is_finite()
            || initial_surface_concentration_mol_per_m3 < 0.0
            || !net_production_mol_per_m2_s.is_finite()
            || !seconds.is_finite()
            || seconds < 0.0
        {
            return Err(SurfaceRateError::InvalidTransport);
        }
        // Reuse the steady model's complete physical-domain validation.
        self.molar_flux_limit()?;
        if seconds == 0.0 {
            return Ok(DiffusionLayerAdvance {
                surface_concentration_mol_per_m3: initial_surface_concentration_mol_per_m3,
                accepted_seconds: 0.0,
                depleted: false,
            });
        }
        let relaxation_rate_per_s = self.diffusivity_m2_per_s / self.diffusion_layer_m.powi(2);
        let steady_concentration = self.bulk_concentration_mol_per_m3
            + net_production_mol_per_m2_s * self.diffusion_layer_m / self.diffusivity_m2_per_s;
        let concentration_at = |elapsed: f64| {
            steady_concentration
                + (initial_surface_concentration_mol_per_m3 - steady_concentration)
                    * (-relaxation_rate_per_s * elapsed).exp()
        };
        let final_concentration = concentration_at(seconds);
        if final_concentration >= 0.0 {
            return Ok(DiffusionLayerAdvance {
                surface_concentration_mol_per_m3: final_concentration,
                accepted_seconds: seconds,
                depleted: false,
            });
        }

        let ratio = -steady_concentration
            / (initial_surface_concentration_mol_per_m3 - steady_concentration);
        if !(0.0..=1.0).contains(&ratio) || ratio == 0.0 {
            return Err(SurfaceRateError::InvalidTransport);
        }
        let depletion_seconds = -ratio.ln() / relaxation_rate_per_s;
        if !depletion_seconds.is_finite() || !(0.0..=seconds).contains(&depletion_seconds) {
            return Err(SurfaceRateError::InvalidTransport);
        }
        Ok(DiffusionLayerAdvance {
            surface_concentration_mol_per_m3: 0.0,
            accepted_seconds: depletion_seconds,
            depleted: true,
        })
    }
}

/// Levich rotating-disk mass transfer in canonical SI units.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RotatingDiskTransport {
    pub diffusivity_m2_per_s: f64,
    pub bulk_concentration_mol_per_m3: f64,
    pub kinematic_viscosity_m2_per_s: f64,
    pub rotation_rate_rpm: f64,
}

impl RotatingDiskTransport {
    pub fn current_density_limit(self, electrons_per_mole: f64) -> Result<f64, SurfaceRateError> {
        if !electrons_per_mole.is_finite()
            || electrons_per_mole <= 0.0
            || !self.diffusivity_m2_per_s.is_finite()
            || self.diffusivity_m2_per_s <= 0.0
            || !self.bulk_concentration_mol_per_m3.is_finite()
            || self.bulk_concentration_mol_per_m3 < 0.0
            || !self.kinematic_viscosity_m2_per_s.is_finite()
            || self.kinematic_viscosity_m2_per_s <= 0.0
            || !self.rotation_rate_rpm.is_finite()
            || self.rotation_rate_rpm < 0.0
        {
            return Err(SurfaceRateError::InvalidTransport);
        }
        let omega_rad_per_s = self.rotation_rate_rpm * std::f64::consts::TAU / 60.0;
        Ok(0.620
            * electrons_per_mole
            * FARADAY
            * self.diffusivity_m2_per_s.powf(2.0 / 3.0)
            * self.kinematic_viscosity_m2_per_s.powf(-1.0 / 6.0)
            * omega_rad_per_s.sqrt()
            * self.bulk_concentration_mol_per_m3)
    }
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

    #[test]
    fn stagnant_film_limit_and_surface_depletion_share_one_flux() {
        let film = DiffusionLayerTransport {
            diffusivity_m2_per_s: 1e-9,
            bulk_concentration_mol_per_m3: 10.0,
            diffusion_layer_m: 1e-4,
        };
        assert!((film.molar_flux_limit().unwrap() - 1e-4).abs() < 1e-15);
        assert_eq!(film.surface_concentration(1e-4).unwrap(), 0.0);
        assert!((film.surface_concentration(0.5e-4).unwrap() - 5.0).abs() < 1e-12);
    }

    #[test]
    fn transient_layer_relaxes_to_the_same_steady_concentration() {
        let film = DiffusionLayerTransport {
            diffusivity_m2_per_s: 1e-9,
            bulk_concentration_mol_per_m3: 10.0,
            diffusion_layer_m: 1e-4,
        };
        let half_limit = -0.5 * film.molar_flux_limit().unwrap();
        let advanced = film
            .advance_constant_flux(10.0, half_limit, 10_000.0)
            .unwrap();
        assert!(!advanced.depleted);
        assert_eq!(advanced.accepted_seconds, 10_000.0);
        assert!((advanced.surface_concentration_mol_per_m3 - 5.0).abs() < 1e-12);
    }

    #[test]
    fn transient_layer_stops_exactly_at_depletion() {
        let film = DiffusionLayerTransport {
            diffusivity_m2_per_s: 1e-9,
            bulk_concentration_mol_per_m3: 10.0,
            diffusion_layer_m: 1e-4,
        };
        let twice_limit = -2.0 * film.molar_flux_limit().unwrap();
        let advanced = film.advance_constant_flux(10.0, twice_limit, 20.0).unwrap();
        let expected = (2.0_f64).ln() * film.diffusion_layer_m.powi(2) / film.diffusivity_m2_per_s;
        assert!(advanced.depleted);
        assert_eq!(advanced.surface_concentration_mol_per_m3, 0.0);
        assert!((advanced.accepted_seconds - expected).abs() < 1e-12);
    }

    #[test]
    fn transient_layer_is_invariant_to_time_partitioning() {
        let film = DiffusionLayerTransport {
            diffusivity_m2_per_s: 2e-9,
            bulk_concentration_mol_per_m3: 20.0,
            diffusion_layer_m: 2e-4,
        };
        let flux = 0.25 * film.molar_flux_limit().unwrap();
        let whole = film.advance_constant_flux(4.0, flux, 3.0).unwrap();
        let first = film.advance_constant_flux(4.0, flux, 1.0).unwrap();
        let split = film
            .advance_constant_flux(first.surface_concentration_mol_per_m3, flux, 2.0)
            .unwrap();
        assert!(
            (whole.surface_concentration_mol_per_m3 - split.surface_concentration_mol_per_m3).abs()
                < 1e-12
        );
    }

    #[test]
    fn rotating_disk_limit_has_levich_square_root_scaling() {
        let slow = RotatingDiskTransport {
            diffusivity_m2_per_s: 2e-9,
            bulk_concentration_mol_per_m3: 1.0,
            kinematic_viscosity_m2_per_s: 1e-6,
            rotation_rate_rpm: 400.0,
        };
        let fast = RotatingDiskTransport {
            rotation_rate_rpm: 1600.0,
            ..slow
        };
        let ratio =
            fast.current_density_limit(4.0).unwrap() / slow.current_density_limit(4.0).unwrap();
        assert!((ratio - 2.0).abs() < 1e-12);
    }
}
