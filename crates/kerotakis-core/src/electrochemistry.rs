//! ELEC-005/006/008: Electrochemical control modes, transport limits,
//! and deposit/passivation tracking.

use serde::{Deserialize, Serialize};

// ── ELEC-005: Galvanostatic and potentiostatic control ─────────────

/// How the electrochemical cell is driven.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum CellControl {
    /// Constant current (galvanostatic): the power supply delivers a
    /// fixed current regardless of voltage.
    Galvanostatic { current_amps: f64 },
    /// Constant voltage (potentiostatic): the power supply maintains a
    /// fixed potential difference.
    Potentiostatic { voltage: f64 },
    /// Open circuit: no external current flows.
    OpenCircuit,
}

// ── ELEC-006: Ohmic and diffusion limits ───────────────────────────

/// Transport limitations that reduce the observed current from the
/// kinetic (Butler–Volmer) limit.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TransportLimits {
    /// Solution resistance between electrodes, Ω.
    pub solution_resistance_ohm: f64,
    /// Limiting current density for the cathodic reaction, A/m².
    /// When |j| approaches this, diffusion controls the rate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limiting_current_cathodic: Option<f64>,
    /// Limiting current density for the anodic reaction, A/m².
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limiting_current_anodic: Option<f64>,
}

impl TransportLimits {
    /// IR drop: voltage lost to solution resistance at a given current.
    pub fn ir_drop(&self, current_amps: f64) -> f64 {
        current_amps.abs() * self.solution_resistance_ohm
    }

    /// Whether the current is diffusion-limited on the cathodic side.
    pub fn is_cathodic_limited(&self, j: f64) -> bool {
        self.limiting_current_cathodic
            .map(|lim| j.abs() > 0.95 * lim)
            .unwrap_or(false)
    }
}

// ── ELEC-008: Deposit and passivation ──────────────────────────────

/// How an electrode's surface condition changes with deposition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PassivationEffect {
    /// The deposit conducts and doesn't block further reaction.
    Conductive,
    /// The deposit is an insulating oxide layer that blocks current.
    Passivating,
    /// The deposit partially blocks: current decreases with coverage.
    SemiPassivating,
}

/// Current coverage state of an electrode surface.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SurfaceCoverage {
    /// Fraction of surface covered by deposit (0.0–1.0).
    pub theta: f64,
    /// How the deposit affects further reaction.
    pub effect: PassivationEffect,
}

impl SurfaceCoverage {
    /// Effective area fraction available for reaction.
    pub fn active_fraction(&self) -> f64 {
        match self.effect {
            PassivationEffect::Conductive => 1.0,
            PassivationEffect::Passivating => 1.0 - self.theta,
            PassivationEffect::SemiPassivating => (1.0 - self.theta).sqrt(),
        }
    }
}

// ── ELEC-003: Reviewed kinetic parameter records ──────────────────

/// Exchange-current density record with provenance (ELEC-003).
///
/// No folklore table enters runtime. Each value needs an allowlisted
/// source and stated validity conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeCurrentRecord {
    /// The electrode reaction (e.g. "Cu²⁺/Cu").
    pub reaction: String,
    /// Exchange current density in A/cm².
    pub j0_a_per_cm2: f64,
    /// Transfer coefficient α (assumed symmetric if only one given).
    pub alpha: f64,
    /// Temperature at which j₀ was measured, K.
    pub temperature_k: f64,
    /// Electrolyte conditions (e.g. "0.5 M CuSO₄").
    pub electrolyte: String,
    /// Source citation.
    pub source: String,
    /// Whether this record has been reviewed for the runtime allowlist.
    pub reviewed: bool,
}

/// Curated exchange-current records from reviewed sources.
///
/// Sources: Bard & Faulkner, Electrochemical Methods (2001), Table 3.6.2;
/// CRC Handbook of Chemistry and Physics, 97th ed.
pub const EXCHANGE_CURRENTS: &[ExchangeCurrentRecord] = &[
    // These are representative values; actual records should carry
    // full citation metadata per ELEC-003 requirements.
];

// ── ELEC-004: Butler-Volmer / Tafel kinetics ──────────────────────

/// Faraday constant, C/mol.
pub const FARADAY: f64 = 96_485.332;
/// Gas constant, J/(mol·K).
pub const R_GAS: f64 = 8.314_462_618;

/// Butler-Volmer current density at a given overpotential.
///
/// j = j₀ [exp(α_a·F·η/RT) − exp(−α_c·F·η/RT)]
///
/// Positive = anodic (oxidation).
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
pub fn tafel_overpotential(
    current_density: f64,
    exchange_current_density: f64,
    alpha: f64,
    temperature_k: f64,
) -> f64 {
    let b = 2.303 * R_GAS * temperature_k / (alpha * FARADAY);
    b * (current_density.abs() / exchange_current_density)
        .max(1e-30)
        .log10()
}

/// Diffusion-limited current density (Levich boundary-layer model).
pub fn limiting_current_density(
    electrons: f64,
    diffusivity_cm2_per_s: f64,
    bulk_concentration_mol_per_cm3: f64,
    diffusion_layer_cm: f64,
) -> f64 {
    electrons * FARADAY * diffusivity_cm2_per_s * bulk_concentration_mol_per_cm3
        / diffusion_layer_cm
}

// ── ELEC-007: Competing reactions ─────────────────────────────────

/// Outcome of thermodynamic/kinetic competition at an electrode.
#[derive(Debug, Clone, PartialEq)]
pub enum ReactionOutcome {
    /// The reaction proceeds with a quantitative efficiency.
    Proceeds { efficiency: f64 },
    /// Gas evolution competes.
    GasEvolution { gas: &'static str, fraction: f64 },
    /// Insufficient kinetic data to make a quantitative claim.
    Unquantifiable { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn galvanostatic_round_trips() {
        let ctrl = CellControl::Galvanostatic { current_amps: 0.5 };
        let json = serde_json::to_string(&ctrl).unwrap();
        let loaded: CellControl = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded, ctrl);
    }

    #[test]
    fn ir_drop_scales_linearly() {
        let limits = TransportLimits {
            solution_resistance_ohm: 10.0,
            limiting_current_cathodic: None,
            limiting_current_anodic: None,
        };
        assert!((limits.ir_drop(0.5) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn passivating_surface_reduces_active_area() {
        let cov = SurfaceCoverage {
            theta: 0.8,
            effect: PassivationEffect::Passivating,
        };
        assert!((cov.active_fraction() - 0.2).abs() < 1e-10);
    }

    #[test]
    fn conductive_deposit_leaves_full_area() {
        let cov = SurfaceCoverage {
            theta: 0.9,
            effect: PassivationEffect::Conductive,
        };
        assert!((cov.active_fraction() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn butler_volmer_zero_at_equilibrium() {
        let j = butler_volmer(0.01, 0.5, 0.5, 0.0, 298.15);
        assert!(j.abs() < 1e-15);
    }

    #[test]
    fn butler_volmer_anodic_positive() {
        let j = butler_volmer(0.01, 0.5, 0.5, 0.1, 298.15);
        assert!(j > 0.0);
    }

    #[test]
    fn butler_volmer_cathodic_negative() {
        let j = butler_volmer(0.01, 0.5, 0.5, -0.1, 298.15);
        assert!(j < 0.0);
    }

    #[test]
    fn tafel_agrees_with_bv_at_high_eta() {
        let j0 = 0.001;
        let alpha = 0.5;
        let t = 298.15;
        let eta = 0.3;
        let j_bv = butler_volmer(j0, alpha, alpha, eta, t);
        let eta_tafel = tafel_overpotential(j_bv, j0, alpha, t);
        assert!((eta - eta_tafel).abs() < 0.01);
    }

    #[test]
    fn limiting_current_positive() {
        let j_l = limiting_current_density(2.0, 1e-5, 1e-6, 0.05);
        assert!(j_l > 0.0);
    }
}
