//! Electrochemical kinetics and control modes (ELEC-002 through ELEC-008).
//!
//! Builds on the Nernst/Faraday infrastructure in `displacement.rs`,
//! adding Butler-Volmer kinetics, overpotential models, and electrode
//! surface state.

use serde::{Deserialize, Serialize};

/// Faraday constant, C/mol.
pub const FARADAY: f64 = 96_485.332;
/// Gas constant, J/(mol·K).
pub const R_GAS: f64 = 8.314_462_618;

// ── ELEC-002: Electrode interface ─────────────────────────────────

/// An electrode with explicit material, area, and kinetic parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectrodeState {
    /// Metal or carbon identity.
    pub material: String,
    /// Geometric area, cm².
    pub area_cm2: f64,
    /// Standard reduction potential vs SHE, V.
    pub e0_volts: f64,
    /// Number of electrons in the half-reaction.
    pub electrons: f64,
    /// ELEC-004: exchange current density, A/cm².
    #[serde(default)]
    pub exchange_current_density: f64,
    /// ELEC-004: anodic transfer coefficient α_a.
    #[serde(default = "default_alpha")]
    pub alpha_a: f64,
    /// ELEC-004: cathodic transfer coefficient α_c.
    #[serde(default = "default_alpha")]
    pub alpha_c: f64,
    /// ELEC-006: ohmic resistance, Ω.
    #[serde(default)]
    pub resistance_ohm: f64,
    /// ELEC-006: diffusion-layer thickness, cm.
    #[serde(default = "default_diffusion_layer")]
    pub diffusion_layer_cm: f64,
    /// ELEC-008: fractional surface coverage by deposit (0..1).
    #[serde(default)]
    pub surface_coverage: f64,
    /// ELEC-008: deposit mass in grams.
    #[serde(default)]
    pub deposit_grams: f64,
}

fn default_alpha() -> f64 {
    0.5
}
fn default_diffusion_layer() -> f64 {
    0.05 // 500 µm Nernst diffusion layer
}

impl ElectrodeState {
    pub fn new(material: &str, area_cm2: f64, e0_volts: f64, electrons: f64) -> Self {
        Self {
            material: material.to_string(),
            area_cm2,
            e0_volts,
            electrons,
            exchange_current_density: 0.0,
            alpha_a: 0.5,
            alpha_c: 0.5,
            resistance_ohm: 0.0,
            diffusion_layer_cm: 0.05,
            surface_coverage: 0.0,
            deposit_grams: 0.0,
        }
    }

    /// Nernst equilibrium potential at the given activity and temperature.
    pub fn nernst_potential(&self, activity: f64, temperature_k: f64) -> f64 {
        let slope = R_GAS * temperature_k * 10f64.ln() / (self.electrons * FARADAY);
        self.e0_volts + slope * activity.max(1e-30).log10()
    }
}

// ── ELEC-004: Butler-Volmer kinetics ──────────────────────────────

/// Butler-Volmer current density at a given overpotential.
///
/// j = j₀ [exp(α_a·F·η/RT) − exp(−α_c·F·η/RT)]
///
/// Returns current density in A/cm². Positive = anodic (oxidation).
pub fn butler_volmer(
    exchange_current_density: f64,
    alpha_a: f64,
    alpha_c: f64,
    overpotential_v: f64,
    temperature_k: f64,
) -> f64 {
    let f_over_rt = FARADAY / (R_GAS * temperature_k);
    exchange_current_density
        * ((alpha_a * f_over_rt * overpotential_v).exp()
            - (-alpha_c * f_over_rt * overpotential_v).exp())
}

/// Tafel approximation for high overpotentials (|η| >> RT/F).
///
/// For anodic: η = a + b·log₁₀(j/j₀)  where b = 2.303·RT/(α_a·F)
/// Returns overpotential in volts.
pub fn tafel_overpotential(
    current_density: f64,
    exchange_current_density: f64,
    alpha: f64,
    temperature_k: f64,
) -> f64 {
    let b = 2.303 * R_GAS * temperature_k / (alpha * FARADAY);
    b * (current_density.abs() / exchange_current_density).max(1e-30).log10()
}

// ── ELEC-005: Control modes ───────────────────────────────────────

/// How the electrochemical cell is driven.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ControlMode {
    /// Fixed current (amperes). The potential adjusts.
    Galvanostatic { current_a: f64 },
    /// Fixed potential (volts vs SHE). The current adjusts.
    Potentiostatic { potential_v: f64 },
    /// Open circuit — no current flows.
    OpenCircuit,
}

/// Result of an electrochemical step.
#[derive(Debug, Clone)]
pub struct ElectrochemicalStep {
    /// Current that flowed, A.
    pub current_a: f64,
    /// Cell potential, V.
    pub potential_v: f64,
    /// Overpotential at the working electrode, V.
    pub overpotential_v: f64,
    /// Moles of metal deposited (positive) or dissolved (negative).
    pub moles_transferred: f64,
    /// Electrical work done, J (positive = energy consumed).
    pub electrical_work_j: f64,
    /// Duration, s.
    pub duration_s: f64,
    /// ELEC-006: ohmic drop, V.
    pub ohmic_drop_v: f64,
    /// ELEC-006: whether diffusion-limited.
    pub diffusion_limited: bool,
}

/// Compute the current for a potentiostatic step using Butler-Volmer.
pub fn potentiostatic_current(
    electrode: &ElectrodeState,
    equilibrium_potential_v: f64,
    applied_potential_v: f64,
    temperature_k: f64,
) -> f64 {
    if electrode.exchange_current_density <= 0.0 {
        return 0.0;
    }
    let eta = applied_potential_v - equilibrium_potential_v - electrode.resistance_ohm * 0.0; // no current yet
    let j = butler_volmer(
        electrode.exchange_current_density,
        electrode.alpha_a,
        electrode.alpha_c,
        eta,
        temperature_k,
    );
    // Scale by coverage: passivated surface reduces active area
    let active_fraction = 1.0 - electrode.surface_coverage;
    j * electrode.area_cm2 * active_fraction
}

/// Run an electrochemical step with the given control mode.
pub fn electrochemical_step(
    electrode: &mut ElectrodeState,
    equilibrium_potential_v: f64,
    mode: ControlMode,
    duration_s: f64,
    temperature_k: f64,
) -> ElectrochemicalStep {
    let (current, potential, eta) = match mode {
        ControlMode::OpenCircuit => (0.0, equilibrium_potential_v, 0.0),
        ControlMode::Galvanostatic { current_a } => {
            let eta = if electrode.exchange_current_density > 0.0 {
                // Invert Butler-Volmer numerically (bisection)
                let target_j = current_a / (electrode.area_cm2 * (1.0 - electrode.surface_coverage).max(1e-15));
                invert_bv(target_j, electrode, temperature_k)
            } else {
                0.0
            };
            let ir_drop = current_a * electrode.resistance_ohm;
            let potential = equilibrium_potential_v + eta + ir_drop;
            (current_a, potential, eta)
        }
        ControlMode::Potentiostatic { potential_v } => {
            let current = potentiostatic_current(
                electrode,
                equilibrium_potential_v,
                potential_v,
                temperature_k,
            );
            let eta = potential_v - equilibrium_potential_v;
            (current, potential_v, eta)
        }
    };

    let coulombs = current.abs() * duration_s;
    let moles = coulombs / (electrode.electrons * FARADAY);
    let moles_signed = if current > 0.0 { -moles } else { moles }; // anodic dissolves
    let ohmic = current * electrode.resistance_ohm;

    // ELEC-008: update deposit state
    if moles_signed > 0.0 {
        electrode.deposit_grams += moles_signed * 63.546; // approximate Cu molar mass
        electrode.surface_coverage = (electrode.surface_coverage + moles_signed * 0.01).min(1.0);
    }

    ElectrochemicalStep {
        current_a: current,
        potential_v: potential,
        overpotential_v: eta,
        moles_transferred: moles_signed,
        electrical_work_j: current.abs() * potential.abs() * duration_s,
        duration_s,
        ohmic_drop_v: ohmic,
        diffusion_limited: false, // TODO: check against limiting current
    }
}

/// Numerically invert Butler-Volmer to find η for a given j.
fn invert_bv(target_j: f64, electrode: &ElectrodeState, temperature_k: f64) -> f64 {
    let (mut lo, mut hi) = (-2.0, 2.0);
    let j0 = electrode.exchange_current_density;
    for _ in 0..100 {
        let mid = 0.5 * (lo + hi);
        let j = butler_volmer(j0, electrode.alpha_a, electrode.alpha_c, mid, temperature_k);
        if (j - target_j).signum() == (butler_volmer(j0, electrode.alpha_a, electrode.alpha_c, lo, temperature_k) - target_j).signum() {
            lo = mid;
        } else {
            hi = mid;
        }
        if hi - lo < 1e-12 {
            break;
        }
    }
    0.5 * (lo + hi)
}

// ── ELEC-006: Limiting current ────────────────────────────────────

/// Diffusion-limited current density (Levich / Cottrell).
///
/// j_L = n·F·D·C / δ  where D is diffusivity, C is bulk concentration,
/// δ is the diffusion layer thickness.
pub fn limiting_current_density(
    electrons: f64,
    diffusivity_cm2_per_s: f64,
    bulk_concentration_mol_per_cm3: f64,
    diffusion_layer_cm: f64,
) -> f64 {
    electrons * FARADAY * diffusivity_cm2_per_s * bulk_concentration_mol_per_cm3
        / diffusion_layer_cm
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nernst_at_standard_conditions() {
        let e = ElectrodeState::new("Cu", 1.0, 0.342, 2.0);
        let potential = e.nernst_potential(1.0, 298.15);
        assert!(
            (potential - 0.342).abs() < 1e-10,
            "E at a=1 should be E°, got {}",
            potential
        );
    }

    #[test]
    fn nernst_shifts_with_activity() {
        let e = ElectrodeState::new("Cu", 1.0, 0.342, 2.0);
        let low_activity = e.nernst_potential(0.01, 298.15);
        let high_activity = e.nernst_potential(1.0, 298.15);
        assert!(
            low_activity < high_activity,
            "lower activity should give lower potential"
        );
    }

    #[test]
    fn butler_volmer_zero_overpotential() {
        let j = butler_volmer(0.01, 0.5, 0.5, 0.0, 298.15);
        assert!(j.abs() < 1e-15, "j should be 0 at η=0, got {}", j);
    }

    #[test]
    fn butler_volmer_anodic_positive() {
        let j = butler_volmer(0.01, 0.5, 0.5, 0.1, 298.15);
        assert!(j > 0.0, "anodic current should be positive at η>0");
    }

    #[test]
    fn butler_volmer_cathodic_negative() {
        let j = butler_volmer(0.01, 0.5, 0.5, -0.1, 298.15);
        assert!(j < 0.0, "cathodic current should be negative at η<0");
    }

    #[test]
    fn tafel_slope_agrees_with_bv() {
        // At high overpotential, Tafel and B-V should agree
        let j0 = 0.001;
        let alpha = 0.5;
        let t = 298.15;
        let eta = 0.3; // ~300 mV, well into Tafel region
        let j_bv = butler_volmer(j0, alpha, alpha, eta, t);
        let eta_tafel = tafel_overpotential(j_bv, j0, alpha, t);
        assert!(
            (eta - eta_tafel).abs() < 0.01,
            "Tafel η={:.4} vs B-V η={:.4}",
            eta_tafel,
            eta
        );
    }

    #[test]
    fn galvanostatic_step_deposits_metal() {
        let mut e = ElectrodeState::new("Cu", 1.0, 0.342, 2.0);
        e.exchange_current_density = 0.01;
        let step = electrochemical_step(
            &mut e,
            0.342,
            ControlMode::Galvanostatic { current_a: -0.1 }, // cathodic
            100.0,
            298.15,
        );
        assert!(step.moles_transferred > 0.0, "should deposit metal");
        assert!(step.electrical_work_j > 0.0);
    }

    #[test]
    fn open_circuit_no_current() {
        let mut e = ElectrodeState::new("Cu", 1.0, 0.342, 2.0);
        let step = electrochemical_step(
            &mut e,
            0.342,
            ControlMode::OpenCircuit,
            100.0,
            298.15,
        );
        assert_eq!(step.current_a, 0.0);
        assert_eq!(step.moles_transferred, 0.0);
    }

    #[test]
    fn deposit_increases_coverage() {
        let mut e = ElectrodeState::new("Cu", 1.0, 0.342, 2.0);
        e.exchange_current_density = 0.01;
        let coverage_before = e.surface_coverage;
        electrochemical_step(
            &mut e,
            0.342,
            ControlMode::Galvanostatic { current_a: -0.1 },
            100.0,
            298.15,
        );
        assert!(
            e.surface_coverage > coverage_before,
            "deposition should increase coverage"
        );
    }

    #[test]
    fn limiting_current_density_positive() {
        let j_l = limiting_current_density(2.0, 1e-5, 1e-6, 0.05);
        assert!(j_l > 0.0);
    }
}
