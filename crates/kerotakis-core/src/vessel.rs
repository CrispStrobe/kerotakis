//! Vessel state: contents, temperature, pressure, thermal mode.

use serde::{Deserialize, Serialize};

use crate::species::{self, Phase, SpeciesId};
use crate::units::{Grams, Joules, Kelvin, Liters, Moles, Pascal};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VesselId(pub usize);

impl std::fmt::Display for VesselId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v{}", self.0 + 1)
    }
}

/// How the vessel exchanges heat with the surroundings between operators.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThermalMode {
    /// No heat exchange: mixing enthalpy and reaction heat change T.
    Adiabatic,
    /// Held at the given temperature (water bath / thermostat).
    Thermostatted(Kelvin),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portion {
    pub species: SpeciesId,
    pub moles: Moles,
    pub phase: Phase,
}

/// What an aqueous solver last computed about this vessel's solution.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SolutionInfo {
    pub ph: f64,
    /// Ionic strength, mol/kgw.
    pub ionic_strength: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vessel {
    pub id: VesselId,
    pub label: String,
    pub contents: Vec<Portion>,
    pub temperature: Kelvin,
    pub pressure: Pascal,
    pub thermal_mode: ThermalMode,
    /// `Some` once an aqueous solver has characterised the solution; `None`
    /// means no solver has — and the honesty pass says so.
    #[serde(default)]
    pub solution: Option<SolutionInfo>,
}

impl Vessel {
    pub fn new(id: VesselId, label: impl Into<String>) -> Self {
        Vessel {
            id,
            label: label.into(),
            contents: Vec::new(),
            temperature: Kelvin::STANDARD,
            pressure: Pascal::ATMOSPHERIC,
            thermal_mode: ThermalMode::Adiabatic,
            solution: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.contents.is_empty()
    }

    pub fn moles_of(&self, species: &SpeciesId) -> Moles {
        Moles(
            self.contents
                .iter()
                .filter(|p| &p.species == species)
                .map(|p| p.moles.0)
                .sum(),
        )
    }

    /// Add matter, merging with an existing portion of the same species and
    /// phase.
    pub fn deposit(&mut self, species: SpeciesId, moles: Moles, phase: Phase) {
        if moles.0 <= 0.0 {
            return;
        }
        if let Some(p) = self
            .contents
            .iter_mut()
            .find(|p| p.species == species && p.phase == phase)
        {
            p.moles = p.moles + moles;
        } else {
            self.contents.push(Portion {
                species,
                moles,
                phase,
            });
        }
    }

    /// Total heat capacity of the contents, J/K. Zero for an empty vessel.
    pub fn heat_capacity(&self) -> f64 {
        self.contents
            .iter()
            .filter_map(|p| species::lookup(&p.species).map(|d| p.moles.0 * d.heat_capacity))
            .sum()
    }

    /// Sensible enthalpy of the contents relative to 298.15 K, J.
    ///
    /// H = Σ n·Cp·(T − T_ref); Cp treated as T-independent at this stage.
    /// The conservation property tests rest on this definition.
    pub fn enthalpy(&self) -> Joules {
        Joules(self.heat_capacity() * (self.temperature.0 - Kelvin::STANDARD.0))
    }

    /// Total mass, g.
    pub fn mass(&self) -> Grams {
        Grams(
            self.contents
                .iter()
                .filter_map(|p| species::lookup(&p.species).map(|d| p.moles.0 * d.molar_mass))
                .sum(),
        )
    }

    /// Approximate liquid volume, additive-volume assumption (surfaced as an
    /// approximation by the renderer). Solids, gases and dissolved species
    /// excluded — the solution's volume is carried by its liquid phase, and
    /// the volume contribution of solutes is not modelled at this stage.
    pub fn liquid_volume(&self) -> Liters {
        Liters(
            self.contents
                .iter()
                .filter(|p| p.phase == Phase::Liquid)
                .filter_map(|p| species::lookup(&p.species).map(|d| d.liters_from_moles(p.moles).0))
                .sum(),
        )
    }
}
