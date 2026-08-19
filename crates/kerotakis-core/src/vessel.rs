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

/// One aqueous species in the true equilibrium distribution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpeciesDetail {
    /// PHREEQC species name (e.g. "Ag+", "AgCl", "CO3-2").
    pub name: String,
    /// mol/kgw.
    pub molality: f64,
    /// Thermodynamic activity; activity/molality is the activity
    /// coefficient γ.
    pub activity: f64,
}

/// Where an answer came from, so any number can be traced: which engine,
/// which dataset, which model — and, where the dataset records it, the
/// literature its numbers came from (PLAN.md: offer different paths and be
/// open about where each came from).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    /// The solver that produced the answer, e.g. "PHREEQC (IPhreeqc)".
    pub engine: String,
    /// The dataset it consulted, e.g. "wateq4f.dat".
    pub dataset: String,
    /// The model that dataset applies, e.g. "Pitzer specific-ion-interaction".
    pub model: String,
    /// How the dataset itself documents its sources (a sample of the
    /// literature citations carried in the data file).
    #[serde(default)]
    pub dataset_sources: Vec<String>,
    /// Why this path was chosen over the alternatives.
    pub routing: String,
}

/// What an aqueous solver last computed about this vessel's solution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolutionInfo {
    /// Electron activity, −log a(e⁻): the redox axis.
    ///
    /// pe is to electrons what pH is to protons, and the symmetry is worth
    /// making visible rather than describing. A solution has an acidity and
    /// an oxidising power, and school chemistry teaches the first at length
    /// while leaving the second as a table of standard potentials to
    /// memorise. This is computed on every solve; it was simply being
    /// thrown away.
    #[serde(default)]
    pub pe: Option<f64>,
    /// How each redox-active element is split between its oxidation
    /// states. Empty when nothing in the beaker has a redox chemistry.
    ///
    /// This is the observable a redox experiment is *for*: not "there is
    /// iron", but "half of it is iron(II) and half is iron(III)". School
    /// chemistry teaches acidity at length and leaves oxidation state as a
    /// table of standard potentials to memorise, largely because the split
    /// is invisible without an engine to compute it.
    #[serde(default)]
    pub redox: Vec<RedoxState>,
    pub ph: f64,
    /// Ionic strength, mol/kgw.
    pub ionic_strength: f64,
    /// Full species distribution (molality > 1e-9), descending. The expert
    /// register's raw material; empty when the solver did not report it.
    #[serde(default)]
    pub species: Vec<SpeciesDetail>,
    /// Where this answer came from.
    #[serde(default)]
    pub provenance: Option<Provenance>,
}

/// One oxidation state of one element, and how much of it there is.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RedoxState {
    /// "Fe", "Mn".
    pub element: String,
    /// The oxidation number: +2, +3, +7, −3.
    pub oxidation: i32,
    /// mol/kgw.
    pub molality: f64,
}

impl RedoxState {
    /// The state in the notation chemists read: `Fe(III)`.
    pub fn label(&self) -> String {
        format!("{}({})", self.element, roman(self.oxidation))
    }
}

/// Roman numerals, as oxidation states are written. Negative states are
/// written with a sign — nitrogen(−III) — because the Romans had no use
/// for them and chemists do.
fn roman(n: i32) -> String {
    let sign = if n < 0 { "−" } else { "" };
    let mut v = n.unsigned_abs();
    if v == 0 {
        return "0".to_string();
    }
    let table = [
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut out = String::new();
    for (value, sym) in table {
        while v >= value {
            out.push_str(sym);
            v -= value;
        }
    }
    format!("{sign}{out}")
}

impl SolutionInfo {
    /// Redox potential in volts, from pe.
    ///
    /// Eh = pe · 2.303·R·T/F — the same quantity in the units a voltmeter
    /// reads, which is how electrochemistry is taught and measured. The
    /// factor is temperature-dependent (0.05916 V at 25 °C), so it takes
    /// the temperature rather than assuming room conditions.
    pub fn eh_volts(&self, temperature_k: f64) -> Option<f64> {
        const FARADAY: f64 = 96_485.332;
        const R: f64 = 8.314_462_618;
        self.pe
            .map(|pe| pe * std::f64::consts::LN_10 * R * temperature_k / FARADAY)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vessel {
    /// Seconds of bench time this vessel has experienced.
    ///
    /// Time is a state dimension the moment rates are modelled, and it is
    /// per-vessel only for bookkeeping: `wait` advances every vessel at
    /// once, because time is not a per-beaker quantity. That is what makes
    /// a fair test possible — two beakers, one variable, the same thirty
    /// seconds.
    #[serde(default)]
    pub elapsed_seconds: f64,
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
            elapsed_seconds: 0.0,
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

    /// Remove up to `moles` of a species across its portions (any phase).
    /// Returns the amount actually removed.
    pub fn withdraw(&mut self, species: &SpeciesId, moles: Moles) -> Moles {
        let mut remaining = moles.0;
        for p in self.contents.iter_mut() {
            if &p.species == species && remaining > 0.0 {
                let take = p.moles.0.min(remaining);
                p.moles = Moles(p.moles.0 - take);
                remaining -= take;
            }
        }
        self.contents.retain(|p| p.moles.0 > 1e-15);
        Moles(moles.0 - remaining)
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
