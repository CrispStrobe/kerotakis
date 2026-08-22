//! ELEC-005/006/008: Electrochemical control modes, transport limits,
//! and deposit/passivation tracking.

use serde::{Deserialize, Serialize};

use crate::compartment::ElectrodeState;
use crate::units::Kelvin;

// ── ELEC-005: Galvanostatic and potentiostatic control ─────────────

/// How the electrochemical cell is driven.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum CellControl {
    /// Constant current (galvanostatic): the power supply delivers a
    /// fixed current regardless of voltage.
    Galvanostatic {
        current_amps: f64,
    },
    /// Constant voltage (potentiostatic): the power supply maintains a
    /// fixed potential difference.
    Potentiostatic {
        voltage: f64,
    },
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
}
