//! CAP-6: Physical constants — CODATA 2018 recommended values.
//!
//! Named constants rather than magic numbers. Every value is the
//! exact CODATA 2018 adjustment, not a rounded textbook approximation.

/// Avogadro constant, mol⁻¹.
pub const AVOGADRO: f64 = 6.022_140_76e23;

/// Boltzmann constant, J/K.
pub const BOLTZMANN: f64 = 1.380_649e-23;

/// Gas constant R = N_A × k_B, J/(mol·K).
pub const GAS_CONSTANT: f64 = 8.314_462_618_153_24;

/// Faraday constant F = N_A × e, C/mol.
pub const FARADAY: f64 = 96_485.332_12;

/// Planck constant, J·s.
pub const PLANCK: f64 = 6.626_070_15e-34;

/// Speed of light in vacuum, m/s.
pub const SPEED_OF_LIGHT: f64 = 299_792_458.0;

/// Elementary charge, C.
pub const ELEMENTARY_CHARGE: f64 = 1.602_176_634e-19;

/// Standard atmosphere, Pa.
pub const STANDARD_ATMOSPHERE: f64 = 101_325.0;

/// Standard temperature, K (25 °C).
pub const STANDARD_TEMPERATURE: f64 = 298.15;

/// Molar mass of water, g/mol (IUPAC 2021 atomic weights).
pub const WATER_MOLAR_MASS: f64 = 18.015_28;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gas_constant_is_na_times_kb() {
        let computed = AVOGADRO * BOLTZMANN;
        assert!(
            (computed - GAS_CONSTANT).abs() < 1e-6,
            "R = {computed}, expected {GAS_CONSTANT}"
        );
    }

    #[test]
    fn faraday_is_na_times_e() {
        let computed = AVOGADRO * ELEMENTARY_CHARGE;
        assert!(
            (computed - FARADAY).abs() < 0.01,
            "F = {computed}, expected {FARADAY}"
        );
    }
}
