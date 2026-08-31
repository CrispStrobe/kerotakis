//! THERMO-002: FluidModel trait — abstracts over ideal and non-ideal VLE.
//!
//! The existing VLE code uses γ = 1 (ideal) or UNIFAC (non-ideal). This
//! trait captures that choice as a pluggable component, so the solver
//! routes the same way regardless of which activity-coefficient model is
//! selected — or whether a cubic equation of state replaces Raoult entirely.

use crate::vle::{Antoine, BubblePoint, DewPoint, FlashResult, Volatile};
use std::fmt;

/// A thermodynamic operation that a [`FluidModel`] may support.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FluidOperation {
    BubblePoint,
    DewPoint,
    TpFlash,
    SaturationPressure,
}

impl fmt::Display for FluidOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::BubblePoint => "bubble point",
            Self::DewPoint => "dew point",
            Self::TpFlash => "TP flash",
            Self::SaturationPressure => "saturation pressure",
        };
        f.write_str(name)
    }
}

/// An inspectable, fail-closed refusal from a fluid-model implementation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FluidModelError {
    /// The selected model does not implement the requested operation.
    UnsupportedOperation {
        model: &'static str,
        operation: FluidOperation,
    },
}

impl FluidModelError {
    pub const fn unsupported(model: &'static str, operation: FluidOperation) -> Self {
        Self::UnsupportedOperation { model, operation }
    }
}

impl fmt::Display for FluidModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedOperation { model, operation } => {
                write!(f, "fluid model '{model}' does not support {operation}")
            }
        }
    }
}

impl std::error::Error for FluidModelError {}

/// Explicit capabilities of a fluid model. Capability discovery is separate
/// from calculation failure: a supported calculation can still return `None`
/// when no numerical solution exists.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FluidCapabilities {
    pub bubble_point: bool,
    pub dew_point: bool,
    pub tp_flash: bool,
    pub saturation_pressure: bool,
}

pub type FluidModelResult<T> = Result<Option<T>, FluidModelError>;

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

    /// Declare the operations implemented by this model.
    fn capabilities(&self) -> FluidCapabilities;

    /// Bubble-point temperature at a given total pressure.
    fn bubble_point(
        &self,
        components: &[Volatile],
        pressure_kpa: f64,
    ) -> FluidModelResult<BubblePoint>;

    /// THERMO-005: Dew-point temperature at a given total pressure.
    fn dew_point(&self, components: &[Volatile], pressure_kpa: f64) -> FluidModelResult<DewPoint>;

    /// THERMO-005: Isothermal TP flash at given temperature and pressure.
    fn tp_flash(
        &self,
        components: &[Volatile],
        pressure_kpa: f64,
        t_celsius: f64,
    ) -> FluidModelResult<FlashResult>;

    /// Saturation pressure for a pure component at a given temperature.
    fn saturation_pressure_kpa(&self, antoine: &Antoine, t_celsius: f64) -> FluidModelResult<f64>;
}

/// The ideal fluid model: Raoult's law with γ = 1.
pub struct IdealFluid;

impl FluidModel for IdealFluid {
    fn name(&self) -> &'static str {
        "ideal Raoult"
    }

    fn capabilities(&self) -> FluidCapabilities {
        FluidCapabilities {
            bubble_point: true,
            dew_point: true,
            tp_flash: true,
            saturation_pressure: true,
        }
    }

    fn bubble_point(
        &self,
        components: &[Volatile],
        pressure_kpa: f64,
    ) -> FluidModelResult<BubblePoint> {
        Ok(crate::vle::bubble_point(components, pressure_kpa))
    }

    fn dew_point(&self, components: &[Volatile], pressure_kpa: f64) -> FluidModelResult<DewPoint> {
        Ok(crate::vle::dew_point(components, pressure_kpa))
    }

    fn tp_flash(
        &self,
        components: &[Volatile],
        pressure_kpa: f64,
        t_celsius: f64,
    ) -> FluidModelResult<FlashResult> {
        Ok(crate::vle::tp_flash(components, pressure_kpa, t_celsius))
    }

    fn saturation_pressure_kpa(&self, antoine: &Antoine, t_celsius: f64) -> FluidModelResult<f64> {
        Ok(antoine.pressure_kpa(t_celsius))
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
        // Water Antoine constants (Stull 1947, kPa form)
        let water = Volatile {
            antoine: Antoine {
                a: 8.07131 - 2.0, // adjusted to kPa
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
        let mixture = [water];
        let direct = crate::vle::bubble_point(&mixture, 101.325);
        assert_eq!(model.bubble_point(&mixture, 101.325), Ok(direct));
        assert_eq!(
            model.capabilities(),
            FluidCapabilities {
                bubble_point: true,
                dew_point: true,
                tp_flash: true,
                saturation_pressure: true,
            }
        );
    }

    #[test]
    fn unsupported_operation_is_a_named_refusal_through_trait_object() {
        struct BubbleOnly;

        impl FluidModel for BubbleOnly {
            fn name(&self) -> &'static str {
                "bubble-only test model"
            }

            fn capabilities(&self) -> FluidCapabilities {
                FluidCapabilities {
                    bubble_point: true,
                    ..FluidCapabilities::default()
                }
            }

            fn bubble_point(
                &self,
                components: &[Volatile],
                pressure_kpa: f64,
            ) -> FluidModelResult<BubblePoint> {
                Ok(crate::vle::bubble_point(components, pressure_kpa))
            }

            fn dew_point(
                &self,
                _components: &[Volatile],
                _pressure_kpa: f64,
            ) -> FluidModelResult<DewPoint> {
                Err(FluidModelError::unsupported(
                    self.name(),
                    FluidOperation::DewPoint,
                ))
            }

            fn tp_flash(
                &self,
                _components: &[Volatile],
                _pressure_kpa: f64,
                _t_celsius: f64,
            ) -> FluidModelResult<FlashResult> {
                Err(FluidModelError::unsupported(
                    self.name(),
                    FluidOperation::TpFlash,
                ))
            }

            fn saturation_pressure_kpa(
                &self,
                _antoine: &Antoine,
                _t_celsius: f64,
            ) -> FluidModelResult<f64> {
                Err(FluidModelError::unsupported(
                    self.name(),
                    FluidOperation::SaturationPressure,
                ))
            }
        }

        let model: &dyn FluidModel = &BubbleOnly;
        assert_eq!(
            model.dew_point(&[], 101.325),
            Err(FluidModelError::UnsupportedOperation {
                model: "bubble-only test model",
                operation: FluidOperation::DewPoint,
            })
        );
        assert!(!model.capabilities().dew_point);
        assert_eq!(
            model.dew_point(&[], 101.325).unwrap_err().to_string(),
            "fluid model 'bubble-only test model' does not support dew point"
        );
    }
}
