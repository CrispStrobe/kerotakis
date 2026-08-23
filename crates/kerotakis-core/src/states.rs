//! When the solvent stops being a liquid.
//!
//! The bench used to report liquid water at −7.95 °C, with a pH. `Phase` was
//! assigned when matter was added and never reconsidered, so cooling a
//! beaker past its freezing point changed nothing but the number on the
//! thermometer. That is the same defect as the others found on 2026-08-19:
//! a state the engine does not model, returned as though it were the state.
//!
//! What makes this worth more than a bounds check is that the thresholds
//! *move*, and why they move is core curriculum. Dissolved particles lower
//! the freezing point and raise the boiling point, in proportion to how
//! many particles there are — which is why salt clears icy roads and why
//! seawater freezes below zero.
//!
//! **The van 't Hoff factor is not a fudge here, it is counted.** School
//! books introduce *i* as a correction you look up: 1 for sugar, 2 for
//! NaCl, 3 for CaCl₂. We never assume it. PHREEQC hands us the actual
//! species in solution — Na⁺ and Cl⁻ as separate entries, plus the neutral
//! ion pairs it finds — so summing solute molality *counts the particles*.
//! A solution where ion pairing is significant gets an effective *i* below
//! the textbook integer automatically, because the pairs are really there.
//!
//! **The constants are derived, not tabulated.** The cryoscopic and
//! ebullioscopic constants of water are not independent facts; they follow
//! from its enthalpies of fusion and vaporisation:
//!
//! ```text
//! K_f = R · T_f² · M / ΔH_fus     K_b = R · T_b² · M / ΔH_vap
//! ```
//!
//! which give 1.86 and 0.513 K·kg·mol⁻¹ against the literature's 1.86 and
//! 0.512. So the only curated inputs are two enthalpies, and the numbers a
//! learner is normally told to memorise come out of them.

use serde::{Deserialize, Serialize};

/// Gas constant, J·mol⁻¹·K⁻¹.
const R: f64 = 8.314_462_618;

/// Water's normal melting point at 1 atm, K.
pub const WATER_FREEZING_K: f64 = 273.15;
/// Water's normal boiling point at 1 atm, K.
pub const WATER_BOILING_K: f64 = 373.15;
/// Molar mass of water, kg/mol.
const WATER_MOLAR_MASS_KG: f64 = 0.018_015;
/// Enthalpy of fusion of water, J/mol (CRC Handbook).
pub const WATER_H_FUS: f64 = 6010.0;
/// Enthalpy of vaporisation of water at the boiling point, J/mol (CRC).
pub const WATER_H_VAP: f64 = 40650.0;
/// Lowest temperature at which the linear colligative partial-freezing model
/// is allowed to claim a liquid/ice split.
///
/// 252 K is approximately the sodium-chloride/water eutectic temperature,
/// but this is deliberately a *model boundary*, not a claim that every brine
/// shares that eutectic. Below it the identity and composition of the solid
/// salt phases matter and the linear dilute-solution relation is no longer an
/// adequate phase diagram.
pub const BRINE_MODEL_MIN_K: f64 = 252.0;

/// Cryoscopic constant of water, K·kg·mol⁻¹ — derived, not looked up.
pub fn cryoscopic_constant() -> f64 {
    R * WATER_FREEZING_K.powi(2) * WATER_MOLAR_MASS_KG / WATER_H_FUS
}

/// Particle molality at the stated low-temperature boundary.
pub fn brine_model_max_particle_molality() -> f64 {
    (WATER_FREEZING_K - BRINE_MODEL_MIN_K) / cryoscopic_constant()
}

/// Ebullioscopic constant of water, K·kg·mol⁻¹ — likewise derived.
pub fn ebullioscopic_constant() -> f64 {
    R * WATER_BOILING_K.powi(2) * WATER_MOLAR_MASS_KG / WATER_H_VAP
}

/// The temperatures at which this solution changes state.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transitions {
    /// Freezing point, K, depressed by the dissolved particles.
    pub freezing_k: f64,
    /// Boiling point, K, elevated by them.
    pub boiling_k: f64,
    /// Total solute molality, mol per kg of water — the particle count that
    /// drives both shifts.
    pub solute_molality: f64,
}

impl Transitions {
    /// How far the freezing point has been pushed down, K.
    pub fn freezing_depression(&self) -> f64 {
        WATER_FREEZING_K - self.freezing_k
    }
    /// How far the boiling point has been pushed up, K.
    pub fn boiling_elevation(&self) -> f64 {
        self.boiling_k - WATER_BOILING_K
    }
}

/// Where this solution freezes and boils, given the total molality of
/// dissolved particles.
///
/// Colligative properties depend on how *many* particles are dissolved and
/// not at all on what they are — which is the whole point, and the reason
/// this takes a molality rather than a composition.
pub fn transitions(solute_molality: f64) -> Transitions {
    let m = solute_molality.max(0.0);
    Transitions {
        freezing_k: WATER_FREEZING_K - cryoscopic_constant() * m,
        boiling_k: WATER_BOILING_K + ebullioscopic_constant() * m,
        solute_molality: m,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_constants_come_out_of_the_enthalpies() {
        // Against the literature values a student is told to memorise.
        assert!(
            (cryoscopic_constant() - 1.86).abs() < 0.01,
            "K_f = {}",
            cryoscopic_constant()
        );
        assert!(
            (ebullioscopic_constant() - 0.512).abs() < 0.005,
            "K_b = {}",
            ebullioscopic_constant()
        );
    }

    #[test]
    fn pure_water_freezes_at_zero() {
        let t = transitions(0.0);
        assert!((t.freezing_k - WATER_FREEZING_K).abs() < 1e-9);
        assert!((t.boiling_k - WATER_BOILING_K).abs() < 1e-9);
    }

    #[test]
    fn salt_water_freezes_below_zero() {
        // 1 mol/kg of NaCl dissolves into ~2 mol/kg of particles, so the
        // depression is about 2 × 1.86 K. The factor of two is not applied
        // here — it arrives as molality, because the caller counted Na+ and
        // Cl- separately.
        let t = transitions(2.0);
        assert!(
            (t.freezing_depression() - 3.72).abs() < 0.02,
            "{} K",
            t.freezing_depression()
        );
        assert!(t.freezing_k < WATER_FREEZING_K);
    }

    #[test]
    fn seawater_is_in_the_right_place() {
        // Seawater is ~1.1 mol/kg of dissolved ions and freezes near -1.9 C.
        let t = transitions(1.1);
        let celsius = t.freezing_k - 273.15;
        assert!(
            (-2.3..-1.6).contains(&celsius),
            "seawater freezes at {celsius:.2} C"
        );
    }

    #[test]
    fn boiling_rises_much_less_than_freezing_falls() {
        // K_b is about a quarter of K_f, which is why cooks salting pasta
        // water for a higher boiling point are wasting their time.
        let t = transitions(2.0);
        assert!(t.boiling_elevation() < t.freezing_depression() / 3.0);
    }

    #[test]
    fn brine_boundary_is_finite_and_matches_its_declared_temperature() {
        let maximum = brine_model_max_particle_molality();
        assert!(maximum.is_finite() && maximum > 10.0);
        assert!((transitions(maximum).freezing_k - BRINE_MODEL_MIN_K).abs() < 1e-12);
    }
}
