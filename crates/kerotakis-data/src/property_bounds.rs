//! Bounds for the three scalar properties consumed by the runtime registry.
//!
//! The core build script depends on this crate so it uses the same unit
//! normalizer as document validation and runtime loading.
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

/// Interpret a runtime scalar in the canonical SpeciesData unit, retaining
/// the source record's original value and unit outside runtime construction.
/// A matching declared dimension is required before any symbol conversion.
pub fn normalize_runtime_scalar(
    property: RuntimeScalarProperty,
    value: f64,
    unit: &crate::Unit,
) -> Result<f64, String> {
    validate_runtime_scalar(property, value)
        .map_err(|reason| format!("{} {reason}", property.name()))?;
    let expected = match property {
        RuntimeScalarProperty::MolarMass => crate::Dimension::MolarMass,
        RuntimeScalarProperty::MassDensity => crate::Dimension::MassDensity,
        RuntimeScalarProperty::MolarHeatCapacity => crate::Dimension::MolarHeatCapacity,
    };
    if unit.dimension != expected {
        return Err(format!(
            "unit dimension {:?}; expected {expected:?}",
            unit.dimension
        ));
    }
    let normalized = crate::normalize_quantity_for(value, &unit.symbol, &expected)
        .map_err(|error| format!("unit: {error}"))?;
    validate_runtime_scalar(property, normalized.value)
        .map_err(|reason| format!("unit-normalized {} {reason}", property.name()))?;
    Ok(normalized.value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversion_overflow_refuses_even_when_source_value_is_finite() {
        let unit = crate::Unit {
            symbol: "kg/mol".to_string(),
            dimension: crate::Dimension::MolarMass,
        };
        assert!(validate_runtime_scalar(RuntimeScalarProperty::MolarMass, f64::MAX).is_ok());
        let error = normalize_runtime_scalar(RuntimeScalarProperty::MolarMass, f64::MAX, &unit)
            .expect_err("conversion to g/mol must not admit infinity");
        assert!(
            error.contains("unit") && error.contains("not finite"),
            "{error}"
        );
    }

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
