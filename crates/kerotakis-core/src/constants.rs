//! CAP-6: Physical constants — CODATA 2018 recommended values.
//!
//! Named constants rather than magic numbers. Every value is the
//! exact CODATA 2018 adjustment, not a rounded textbook approximation.
//!
//! **Except the solvent's, at the bottom, and the exception is the point.**
//! Water's molar mass and its two latent heats are not physical constants of
//! the universe; they are properties of a substance, and this registry has a
//! place for those with a source and an uncertainty attached. They are
//! generated here from that place rather than typed here beside it.

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

// ── The solvent's constants, generated from the registry pack ────────
//
// `build.rs` writes these out of `data/registry/registry-source-v1.json`,
// so the engine reads the records the registry documents instead of
// carrying its own copy beside them:
//
//   * `WATER_MOLAR_MASS_G_PER_MOL`  — `molar-mass/water`
//   * `WATER_MOLAR_MASS_KG_PER_MOL` — the same value in kg/mol
//   * `WATER_ENTHALPY_OF_FUSION_J_PER_MOL`       — `enthalpy-of-fusion/water`
//   * `WATER_ENTHALPY_OF_VAPORISATION_J_PER_MOL` — `enthalpy-of-vaporisation/water`
//
// THIS FILE USED TO CARRY A FOURTEENTH COPY OF THE MOLAR MASS AND THE
// ONLY WRONG ONE. It read `WATER_MOLAR_MASS = 18.015_28` under the comment
// "IUPAC 2021 atomic weights", and 18.01528 is not those weights: it is
// 2 x 1.00794 + 15.9994, the pre-2009 IUPAC values, superseded when CIAAW
// moved hydrogen and oxygen to intervals whose conventional representatives
// are 1.008 and 15.999. Those sum to 18.015, which is what the registry
// record says and what every other site on the bench already used. The
// constant had no callers, so the wrong number never reached an answer
// through this file - but it was the spelling two `map_or` fallbacks in
// `bench.rs` and `scene.rs` reached for, and it is gone.
//
// A NOTE ON WHAT IS NOT HERE. CODATA does not publish a molar mass of
// water and this module is otherwise CODATA's; a molar mass is a
// stoichiometric sum over an atomic-weight table, which is why it belongs
// to the registry and arrives from there.
include!(concat!(env!("OUT_DIR"), "/solvent_generated.rs"));

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
