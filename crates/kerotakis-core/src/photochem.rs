//! ADV-002: Photochemistry IR — light-source state and photolysis rate laws.
//!
//! Extends the reaction-network IR with photon-driven rate expressions:
//! rate = Φ · I · ε · c · l  where Φ is quantum yield, I is irradiance,
//! ε is molar absorptivity, c is concentration, l is path length.

use serde::{Deserialize, Serialize};

/// A light source with wavelength and intensity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LightSource {
    /// Wavelength in nm (monochromatic approximation).
    pub wavelength_nm: f64,
    /// Irradiance in W/m² (photon flux × energy per photon).
    pub irradiance_w_m2: f64,
    /// Whether the source is on or off.
    pub active: bool,
}

impl LightSource {
    /// Photon energy in J.
    pub fn photon_energy_j(&self) -> f64 {
        // E = hc/λ
        const H: f64 = 6.626e-34;
        const C: f64 = 2.998e8;
        H * C / (self.wavelength_nm * 1e-9)
    }

    /// Photon flux in photons/(m²·s).
    pub fn photon_flux(&self) -> f64 {
        if self.active {
            self.irradiance_w_m2 / self.photon_energy_j()
        } else {
            0.0
        }
    }
}

/// A photolysis rate law: the rate at which a photon-absorbing species
/// decomposes under irradiation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhotolysisRate {
    /// Species that absorbs the photon.
    pub absorber: String,
    /// Quantum yield Φ (molecules reacted per photon absorbed, 0–1).
    pub quantum_yield: f64,
    /// Molar absorptivity at the source wavelength, L/(mol·cm).
    pub epsilon: f64,
    /// Path length through the solution, cm.
    pub path_cm: f64,
}

impl PhotolysisRate {
    /// Rate of reaction in mol/(L·s) given concentration and photon flux.
    pub fn rate_mol_per_l_s(&self, concentration_mol_l: f64, photon_flux: f64) -> f64 {
        // Absorbed photons per volume = ε · c · l · I₀ (for thin samples)
        // Rate = Φ · absorbed_rate
        self.quantum_yield * self.epsilon * concentration_mol_l * self.path_cm * photon_flux
            / 6.022e23 // convert photons to moles
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uv_lamp_photon_energy() {
        let lamp = LightSource {
            wavelength_nm: 254.0, // mercury lamp
            irradiance_w_m2: 10.0,
            active: true,
        };
        let e = lamp.photon_energy_j();
        // 254 nm → ~7.83e-19 J
        assert!((e - 7.83e-19).abs() < 0.1e-19);
        assert!(lamp.photon_flux() > 0.0);
    }

    #[test]
    fn inactive_lamp_gives_zero_flux() {
        let lamp = LightSource {
            wavelength_nm: 400.0,
            irradiance_w_m2: 100.0,
            active: false,
        };
        assert_eq!(lamp.photon_flux(), 0.0);
    }

    #[test]
    fn photolysis_rate_scales_with_concentration() {
        let rate = PhotolysisRate {
            absorber: "H2O2".into(),
            quantum_yield: 0.5,
            epsilon: 20.0,
            path_cm: 1.0,
        };
        let r1 = rate.rate_mol_per_l_s(0.01, 1e18);
        let r2 = rate.rate_mol_per_l_s(0.02, 1e18);
        assert!((r2 / r1 - 2.0).abs() < 0.01);
    }
}
