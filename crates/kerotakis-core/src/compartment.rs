//! ARCH-005 + ARCH-006: ResolvedState, Compartment, and Environment.
//!
//! ARCH-005: Move aqueous `SolutionInfo`, thermal equilibrium, saturation,
//! and phase interpretation behind an invalidatable derived-state container.
//!
//! ARCH-006: Wrap the current vessel as one well-mixed liquid/solid
//! compartment with the existing open-air behavior expressed as boundary
//! conditions.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::species::Phase;
use crate::units::{Kelvin, Liters, Moles, Pascal};
use crate::vessel::{Headspace, ResolvedState};
use crate::SpeciesId;

// ── ARCH-006: Compartment and Environment ──────────────────────────────

/// How the compartment's volume responds to pressure changes.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VolumeMode {
    /// Volume is fixed (rigid walls). Pressure adjusts.
    #[default]
    Fixed,
    /// Volume adjusts to maintain a set pressure (piston).
    Movable { target_pressure: Pascal },
    /// Volume is unconstrained (open beaker — the current default).
    Open,
}

/// One well-mixed region within a vessel: the liquid/solid contents,
/// their resolved chemistry, and a volume constraint.
///
/// The current `Vessel` is equivalent to a single `Compartment` with
/// `VolumeMode::Open` and an external `Environment`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Compartment {
    pub label: String,
    /// All species portions in this compartment.
    pub contents: Vec<crate::vessel::Portion>,
    /// Temperature of this compartment.
    pub temperature: Kelvin,
    /// Pressure within this compartment.
    pub pressure: Pascal,
    /// How this compartment's volume responds to changes.
    pub volume_mode: VolumeMode,
    /// Derived chemistry state — invalidated on mutation.
    #[serde(default)]
    pub resolved: ResolvedState,
}

impl Default for Compartment {
    fn default() -> Self {
        Self {
            label: "compartment".into(),
            contents: Vec::new(),
            temperature: Kelvin(298.15),
            pressure: Pascal(101325.0),
            volume_mode: VolumeMode::Open,
            resolved: ResolvedState::default(),
        }
    }
}

impl Compartment {
    /// Total moles of a species in a given phase.
    pub fn moles(&self, species: &SpeciesId, phase: Phase) -> Moles {
        Moles(
            self.contents
                .iter()
                .filter(|p| p.species == *species && p.phase == phase)
                .map(|p| p.moles.0)
                .sum(),
        )
    }

    /// Whether any dissolved species are present.
    pub fn has_aqueous(&self) -> bool {
        self.contents.iter().any(|p| p.phase == Phase::Aqueous)
    }
}

/// The boundary conditions that a compartment sits in — the atmosphere,
/// thermostat, or sealed headspace above the contents.
///
/// This replaces the implicit "open beaker in air" assumption with an
/// explicit, serializable boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    /// Ambient pressure outside the vessel.
    pub ambient_pressure: Pascal,
    /// Ambient temperature (for heat exchange if thermostatted).
    pub ambient_temperature: Kelvin,
    /// Gas boundary above the compartment.
    pub headspace: Headspace,
    /// Composition of the atmospheric reservoir (mole fractions).
    /// Default: standard air (N₂ 0.78, O₂ 0.21, Ar 0.01).
    #[serde(default = "default_atmosphere")]
    pub atmosphere: Vec<(String, f64)>,
}

fn default_atmosphere() -> Vec<(String, f64)> {
    vec![
        ("N2".into(), 0.78),
        ("O2".into(), 0.21),
        ("Ar".into(), 0.01),
    ]
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            ambient_pressure: Pascal(101325.0),
            ambient_temperature: Kelvin(298.15),
            headspace: Headspace::Open,
            atmosphere: default_atmosphere(),
        }
    }
}

// ── ARCH-007: Interface ────────────────────────────────────────────────

/// A boundary between two compartments or between a compartment and
/// the environment. Interfaces carry area, permeability, and transfer
/// coefficients — the physical quantities needed to model finite-rate
/// mass and heat transfer.
///
/// Currently data-only; no chemistry is implemented on interfaces yet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interface {
    /// Human-readable label for this interface.
    pub label: String,
    /// What kind of boundary this is.
    pub kind: InterfaceKind,
    /// Contact area in m².
    pub area_m2: f64,
}

/// The type of physical boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterfaceKind {
    /// Gas–liquid boundary (evaporation, absorption).
    GasLiquid,
    /// Liquid–solid boundary (dissolution, precipitation, adsorption).
    LiquidSolid,
    /// Solid–gas boundary (sublimation, deposition).
    SolidGas,
    /// Membrane or filter (selective permeation).
    Membrane,
    /// Electrode surface (electrochemistry).
    Electrode,
}

impl Default for Interface {
    fn default() -> Self {
        Self {
            label: "interface".into(),
            kind: InterfaceKind::GasLiquid,
            area_m2: 1e-3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compartment_default_is_room_conditions() {
        let c = Compartment::default();
        assert!((c.temperature.0 - 298.15).abs() < 0.01);
        assert!((c.pressure.0 - 101325.0).abs() < 1.0);
        assert!(!c.resolved.valid);
    }

    #[test]
    fn environment_default_is_standard_air() {
        let env = Environment::default();
        assert_eq!(env.atmosphere.len(), 3);
        assert_eq!(env.atmosphere[0].0, "N2");
        assert!((env.atmosphere[0].1 - 0.78).abs() < 0.01);
    }

    #[test]
    fn interface_serializes_round_trip() {
        let iface = Interface {
            label: "beaker wall".into(),
            kind: InterfaceKind::LiquidSolid,
            area_m2: 0.005,
        };
        let json = serde_json::to_string(&iface).unwrap();
        let loaded: Interface = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.kind, InterfaceKind::LiquidSolid);
        assert!((loaded.area_m2 - 0.005).abs() < 1e-10);
    }

    #[test]
    fn volume_mode_variants_serialize() {
        let fixed = VolumeMode::Fixed;
        let movable = VolumeMode::Movable {
            target_pressure: Pascal(101325.0),
        };
        let open = VolumeMode::Open;

        for mode in [fixed, movable, open] {
            let json = serde_json::to_string(&mode).unwrap();
            let loaded: VolumeMode = serde_json::from_str(&json).unwrap();
            assert_eq!(loaded, mode);
        }
    }
}
