//! Bounds for the three scalar properties consumed by the runtime registry.
//!
//! This module deliberately has no crate dependencies: the core build script
//! includes the same source used by document validation and runtime loading.
//! These bounds describe the runtime fallback representation, not every
//! thermodynamic quantity. In particular, zero fallback Cp remains supported
//! for aqueous species whose heat capacity is carried by the solvent.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeScalarProperty {
    MolarMass,
    MassDensity,
    MolarHeatCapacity,
}

impl RuntimeScalarProperty {
    pub fn name(self) -> &'static str {
        match self {
            Self::MolarMass => "molar_mass",
            Self::MassDensity => "mass_density",
            Self::MolarHeatCapacity => "molar_heat_capacity",
        }
    }
}

/// Validate only the scalar bounds required by the runtime representation.
/// Signed energies and independently curated polynomial coefficients are
/// outside this policy.
pub fn validate_runtime_scalar(
    property: RuntimeScalarProperty,
    value: f64,
) -> Result<(), &'static str> {
    match property {
        RuntimeScalarProperty::MolarMass | RuntimeScalarProperty::MassDensity => {
            if !value.is_finite() || value <= 0.0 {
                return Err("must be finite and positive");
            }
        }
        RuntimeScalarProperty::MolarHeatCapacity => {
            if !value.is_finite() || value < 0.0 {
                return Err("must be finite and non-negative");
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonfinite_runtime_scalars_refuse() {
        for property in [
            RuntimeScalarProperty::MolarMass,
            RuntimeScalarProperty::MassDensity,
            RuntimeScalarProperty::MolarHeatCapacity,
        ] {
            for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
                assert!(validate_runtime_scalar(property, value).is_err());
            }
        }
    }

    #[test]
    fn positive_mass_and_density_are_required() {
        for property in [
            RuntimeScalarProperty::MolarMass,
            RuntimeScalarProperty::MassDensity,
        ] {
            for value in [-1.0, -0.0, 0.0] {
                assert!(validate_runtime_scalar(property, value).is_err());
            }
            assert!(validate_runtime_scalar(property, 1.0).is_ok());
        }
    }

    #[test]
    fn zero_fallback_heat_capacity_remains_supported() {
        for value in [-0.0, 0.0, 1.0] {
            assert!(
                validate_runtime_scalar(RuntimeScalarProperty::MolarHeatCapacity, value).is_ok()
            );
        }
        assert!(validate_runtime_scalar(RuntimeScalarProperty::MolarHeatCapacity, -1.0).is_err());
    }
}
