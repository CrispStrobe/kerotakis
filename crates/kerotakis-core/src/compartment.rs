//! ARCH-005 + ARCH-006: ResolvedState, Compartment, and Environment.
//!
//! ARCH-005: Move aqueous `SolutionInfo`, thermal equilibrium, saturation,
//! and phase interpretation behind an invalidatable derived-state container.
//!
//! ARCH-006: Wrap the current vessel as one well-mixed liquid/solid
//! compartment with the existing open-air behavior expressed as boundary
//! conditions.

use serde::{Deserialize, Serialize};

use crate::species::Phase;
use crate::units::{Kelvin, Moles, Pascal};
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
    /// Electrochemical surfaces exposed to this well-mixed region.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub electrodes: Vec<ElectrodeState>,
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
            electrodes: Vec::new(),
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

// ── ELEC-001: Electrode state ──────────────────────────────────────

/// Explicit electrode state: material, geometry, and surface condition.
/// ELEC-001 requires this to serialize and replay, independently of
/// the derived Nernst potential computed in displacement.rs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectrodeState {
    /// Stable identity for operators and transactional deltas.
    pub label: String,
    /// The metal or conductor material (e.g. "Zn", "Cu", "Pt").
    pub material: String,
    /// Reproducible preparation state used to select surface-sensitive
    /// kinetic records (for example "diamond-polished 1 um"). Unknown is
    /// distinct from any preparation and therefore matches no constrained
    /// record.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surface_preparation: Option<String>,
    /// Finite substrate inventory when the electrode itself may be consumed.
    /// `None` denotes external apparatus whose lifetime is out of scope.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub substrate_moles: Option<f64>,
    /// Geometric area in m².
    pub area_m2: f64,
    /// Surface roughness factor (real area / geometric area). Default 1.0.
    #[serde(default = "default_roughness")]
    pub roughness: f64,
    /// Deposited material on the electrode surface (e.g. from electroplating).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deposits: Vec<ElectrodeDeposit>,
}

fn default_roughness() -> f64 {
    1.0
}

impl ElectrodeState {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.label.trim().is_empty() {
            return Err("electrode label must be named");
        }
        if self.material.trim().is_empty() {
            return Err("electrode material must be named");
        }
        if self
            .surface_preparation
            .as_ref()
            .is_some_and(|preparation| preparation.trim().is_empty())
        {
            return Err("electrode surface preparation must be named when known");
        }
        if self
            .substrate_moles
            .is_some_and(|moles| !moles.is_finite() || moles < 0.0)
        {
            return Err("finite electrode inventory must be non-negative");
        }
        if !self.area_m2.is_finite() || self.area_m2 <= 0.0 {
            return Err("electrode area must be finite and positive");
        }
        self.reactive_surface(1.0)
            .validate()
            .map_err(|_| "electrode area and roughness must be finite and positive")?;
        if self.deposits.iter().any(|deposit| {
            deposit.species.trim().is_empty()
                || !deposit.moles.is_finite()
                || deposit.moles < 0.0
                || deposit
                    .thickness_m
                    .is_some_and(|value| !value.is_finite() || value < 0.0)
                || deposit
                    .coverage_fraction
                    .is_some_and(|value| !value.is_finite() || !(0.0..=1.0).contains(&value))
        }) {
            return Err(
                "electrode deposits require valid identity, amount, thickness and coverage",
            );
        }
        Ok(())
    }

    /// Construct reaction-specific surface geometry. Availability is supplied
    /// per reaction: a conductive deposit can block metal dissolution while
    /// remaining active for a cathodic reaction.
    pub fn reactive_surface(
        &self,
        available_fraction: f64,
    ) -> crate::heterogeneous::ReactiveSurface {
        crate::heterogeneous::ReactiveSurface {
            geometric_area_m2: self.area_m2,
            roughness_factor: self.roughness,
            available_fraction,
        }
    }

    /// Conservative availability implied by every characterised deposit.
    /// Layers with unknown geometry or effect are a model gap, not a clean
    /// surface. The most blocking layer wins; multiplying coverages would
    /// invent statistical independence between stacked films.
    pub fn deposit_available_fraction(&self) -> Result<f64, &'static str> {
        let mut available = 1.0_f64;
        for deposit in self.deposits.iter().filter(|deposit| deposit.moles > 0.0) {
            let coverage = deposit
                .coverage_fraction
                .ok_or("deposit coverage is not characterised")?;
            let effect = deposit
                .effect
                .ok_or("deposit kinetic effect is not characterised")?;
            available = available.min(
                crate::electrochemistry::SurfaceCoverage {
                    theta: coverage,
                    effect,
                }
                .active_fraction(),
            );
        }
        Ok(available)
    }
}

/// Geometry assumed while a deposited phase grows. Parameters are data: the
/// engine computes coverage and thickness but does not guess morphology.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "morphology", rename_all = "snake_case")]
pub enum DepositGrowthModel {
    /// A continuous film covers the surface as soon as it exists.
    Conformal { molar_volume_m3_per_mol: f64 },
    /// Constant-height islands spread laterally until they coalesce, then the
    /// continuous film thickens.
    IslandCoalescence {
        molar_volume_m3_per_mol: f64,
        coalescence_thickness_m: f64,
    },
}

impl DepositGrowthModel {
    pub fn geometry(self, moles: f64, geometric_area_m2: f64) -> Result<(f64, f64), &'static str> {
        let (molar_volume, coalescence) = match self {
            Self::Conformal {
                molar_volume_m3_per_mol,
            } => (molar_volume_m3_per_mol, None),
            Self::IslandCoalescence {
                molar_volume_m3_per_mol,
                coalescence_thickness_m,
            } => (molar_volume_m3_per_mol, Some(coalescence_thickness_m)),
        };
        if !moles.is_finite()
            || moles < 0.0
            || !geometric_area_m2.is_finite()
            || geometric_area_m2 <= 0.0
            || !molar_volume.is_finite()
            || molar_volume <= 0.0
            || coalescence.is_some_and(|height| !height.is_finite() || height <= 0.0)
        {
            return Err("deposit growth parameters must be finite and physical");
        }
        if moles == 0.0 {
            return Ok((0.0, 0.0));
        }
        let volume = moles * molar_volume;
        match coalescence {
            None => Ok((volume / geometric_area_m2, 1.0)),
            Some(height) => {
                let coverage = (volume / (geometric_area_m2 * height)).min(1.0);
                let thickness = volume / (geometric_area_m2 * coverage);
                Ok((thickness, coverage))
            }
        }
    }
}

/// A layer deposited on an electrode surface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectrodeDeposit {
    /// What was deposited (species key).
    pub species: String,
    /// Amount deposited in moles.
    pub moles: f64,
    /// Thickness estimate in metres (if known).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thickness_m: Option<f64>,
    /// Geometric coverage when measured or computed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coverage_fraction: Option<f64>,
    /// Kinetic role of this layer. Absence means no defensible model is known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect: Option<crate::electrochemistry::PassivationEffect>,
}

impl Default for ElectrodeState {
    fn default() -> Self {
        Self {
            label: "electrode".into(),
            material: "Pt".into(),
            surface_preparation: None,
            substrate_moles: None,
            area_m2: 1e-4,
            roughness: 1.0,
            deposits: Vec::new(),
        }
    }
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
    fn electrode_state_round_trips() {
        let electrode = ElectrodeState {
            label: "zinc anode".into(),
            material: "Zn".into(),
            surface_preparation: Some("project test polish".into()),
            substrate_moles: Some(0.01),
            area_m2: 0.001,
            roughness: 1.5,
            deposits: vec![ElectrodeDeposit {
                species: "Cu".into(),
                moles: 0.0001,
                thickness_m: Some(1e-6),
                coverage_fraction: Some(0.25),
                effect: Some(crate::electrochemistry::PassivationEffect::Conductive),
            }],
        };
        let json = serde_json::to_string(&electrode).unwrap();
        let loaded: ElectrodeState = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.material, "Zn");
        assert!(loaded.validate().is_ok());
        assert_eq!(loaded.deposits.len(), 1);
        assert_eq!(loaded.deposits[0].species, "Cu");
    }

    #[test]
    fn island_growth_computes_coverage_then_thickness() {
        let model = DepositGrowthModel::IslandCoalescence {
            molar_volume_m3_per_mol: 1e-5,
            coalescence_thickness_m: 1e-6,
        };
        let (thin, half) = model.geometry(0.5e-3, 0.01).unwrap();
        assert!((thin - 1e-6).abs() < 1e-15);
        assert!((half - 0.5).abs() < 1e-12);
        let (thick, full) = model.geometry(2e-3, 0.01).unwrap();
        assert!((thick - 2e-6).abs() < 1e-15);
        assert_eq!(full, 1.0);
    }

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
