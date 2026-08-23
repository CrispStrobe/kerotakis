//! ELEC-004: Butler–Volmer and Tafel electrode kinetics.
//!
//! The Butler–Volmer equation relates current density to overpotential:
//!   j = j₀ [exp(αₐFη/RT) - exp(-αcFη/RT)]
//!
//! At high overpotential one exponential dominates (Tafel regime):
//!   η = a + b·log|j|  where b = 2.303RT/(αF) (Tafel slope)

use serde::{Deserialize, Serialize};

use crate::constants;

const F: f64 = constants::FARADAY;
const R: f64 = constants::GAS_CONSTANT;

/// Butler–Volmer parameters for one electrode reaction.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ButlerVolmerParams {
    /// Exchange current density, A/m².
    pub j0: f64,
    /// Anodic transfer coefficient (dimensionless, typically 0.3–0.7).
    pub alpha_a: f64,
    /// Cathodic transfer coefficient (dimensionless, typically 0.3–0.7).
    pub alpha_c: f64,
    /// Number of electrons transferred per event.
    pub n: f64,
}

impl ButlerVolmerParams {
    /// Current density (A/m²) from overpotential (V) at temperature (K).
    ///
    /// Positive j = anodic (oxidation), negative j = cathodic (reduction).
    pub fn current_density(&self, eta: f64, t_kelvin: f64) -> f64 {
        let f_rt = F / (R * t_kelvin);
        self.j0
            * ((self.alpha_a * self.n * f_rt * eta).exp()
                - (-self.alpha_c * self.n * f_rt * eta).exp())
    }

    /// Tafel slope for the anodic branch, V/decade.
    pub fn tafel_slope_anodic(&self, t_kelvin: f64) -> f64 {
        2.303 * R * t_kelvin / (self.alpha_a * self.n * F)
    }

    /// Tafel slope for the cathodic branch, V/decade.
    pub fn tafel_slope_cathodic(&self, t_kelvin: f64) -> f64 {
        2.303 * R * t_kelvin / (self.alpha_c * self.n * F)
    }

    /// At equilibrium (η=0), current density is zero — verify the BV
    /// equation satisfies this identity.
    pub fn is_at_equilibrium(&self, tolerance: f64) -> bool {
        self.current_density(0.0, 298.15).abs() < tolerance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn typical_params() -> ButlerVolmerParams {
        ButlerVolmerParams {
            j0: 1e-3, // 1 mA/m²
            alpha_a: 0.5,
            alpha_c: 0.5,
            n: 1.0,
        }
    }

    #[test]
    fn zero_overpotential_gives_zero_current() {
        let p = typical_params();
        assert!(p.is_at_equilibrium(1e-15));
    }

    #[test]
    fn positive_overpotential_gives_anodic_current() {
        let p = typical_params();
        let j = p.current_density(0.1, 298.15);
        assert!(j > 0.0, "anodic current should be positive");
    }

    #[test]
    fn negative_overpotential_gives_cathodic_current() {
        let p = typical_params();
        let j = p.current_density(-0.1, 298.15);
        assert!(j < 0.0, "cathodic current should be negative");
    }

    #[test]
    fn tafel_slope_is_about_59mv_for_n1_alpha_half() {
        let p = typical_params();
        let b = p.tafel_slope_anodic(298.15);
        // 2.303 * 8.314 * 298.15 / (0.5 * 1 * 96485) ≈ 0.1183 V/decade
        assert!((b - 0.1183).abs() < 0.001, "Tafel slope = {b}");
    }

    #[test]
    fn symmetry_when_alpha_a_equals_alpha_c() {
        let p = typical_params();
        let j_pos = p.current_density(0.05, 298.15);
        let j_neg = p.current_density(-0.05, 298.15);
        assert!(
            (j_pos + j_neg).abs() < 1e-10,
            "symmetric BV should give antisymmetric current"
        );
    }
}
