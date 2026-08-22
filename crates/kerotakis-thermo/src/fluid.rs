//! THERMO-002: FluidModel trait — abstracts over ideal and non-ideal VLE.
//!
//! The existing VLE code uses γ = 1 (ideal) or UNIFAC (non-ideal). This
//! trait captures that choice as a pluggable component, so the solver
//! routes the same way regardless of which activity-coefficient model is
//! selected — or whether a cubic equation of state replaces Raoult entirely.

use crate::vle::{Antoine, BubblePoint, DewPoint, FlashResult, Volatile};

/// Activity coefficient model: given composition and temperature, return
/// activity coefficients for each component.
pub trait ActivityModel {
    fn name(&self) -> &'static str;

    /// Compute activity coefficients for each component at the given
    /// mole fractions and temperature (°C). Returns γ_i for each component.
    fn activity_coefficients(&self, mole_fractions: &[f64], t_celsius: f64) -> Vec<f64>;
}

/// Ideal solution: γ_i = 1 for all components at all conditions.
pub struct IdealSolution;

impl ActivityModel for IdealSolution {
    fn name(&self) -> &'static str {
        "ideal (Raoult)"
    }

    fn activity_coefficients(&self, mole_fractions: &[f64], _t_celsius: f64) -> Vec<f64> {
        vec![1.0; mole_fractions.len()]
    }
}

/// A fluid model: vapour pressure correlation + activity coefficients.
/// The solver asks this for bubble/dew points and flash results without
/// knowing which thermodynamic model is behind it.
pub trait FluidModel {
    fn name(&self) -> &'static str;

    /// Bubble-point temperature at a given total pressure.
    fn bubble_point(&self, components: &[Volatile], pressure_kpa: f64) -> Option<BubblePoint>;

    /// THERMO-005: Dew-point temperature at a given total pressure.
    fn dew_point(&self, components: &[Volatile], pressure_kpa: f64) -> Option<DewPoint> {
        crate::vle::dew_point(components, pressure_kpa)
    }

    /// THERMO-005: Isothermal TP flash at given temperature and pressure.
    fn tp_flash(
        &self,
        components: &[Volatile],
        pressure_kpa: f64,
        t_celsius: f64,
    ) -> Option<FlashResult> {
        crate::vle::tp_flash(components, pressure_kpa, t_celsius)
    }

    /// Saturation pressure for a pure component at a given temperature.
    fn saturation_pressure_kpa(&self, antoine: &Antoine, t_celsius: f64) -> Option<f64> {
        antoine.pressure_kpa(t_celsius)
    }
}

/// The ideal fluid model: Raoult's law with γ = 1.
pub struct IdealFluid;

impl FluidModel for IdealFluid {
    fn name(&self) -> &'static str {
        "ideal Raoult"
    }

    fn bubble_point(&self, components: &[Volatile], pressure_kpa: f64) -> Option<BubblePoint> {
        crate::vle::bubble_point(components, pressure_kpa)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ideal_solution_returns_unity_gammas() {
        let gammas = IdealSolution.activity_coefficients(&[0.5, 0.5], 25.0);
        assert_eq!(gammas, vec![1.0, 1.0]);
    }

    #[test]
    fn ideal_fluid_delegates_to_vle() {
        // Water Antoine constants (NIST, kPa form)
        let water = Volatile {
            antoine: Antoine {
                a: 8.07131 - 2.0,   // adjusted to kPa
                b: 1730.63,
                c: 233.426,
                valid_c: (1.0, 100.0),
                source: "test",
            },
            x: 1.0,
            gamma: 1.0,
        };

        let model = IdealFluid;
        assert_eq!(model.name(), "ideal Raoult");
        // Just verify it doesn't panic for a pure-component case
        let _ = model.bubble_point(&[water], 101.325);
    }
}
