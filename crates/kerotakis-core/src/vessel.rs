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

/// The gas boundary above a vessel's contents.
///
/// Reservoir and swept boundaries exchange gas with the surroundings. Sealed
/// and pressure-controlled boundaries own their gas portions, which therefore
/// contribute to vessel mass; the latter lets volume move to hold a set
/// pressure.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(tag = "boundary", rename_all = "snake_case")]
pub enum Headspace {
    #[default]
    Open,
    Sealed {
        volume: Liters,
    },
    PressureControlled {
        pressure: Pascal,
        volume: Liters,
    },
    /// An inert nitrogen purge carries volatile products away at the stated
    /// total pressure. No carrier-gas inventory is booked into the vessel.
    Swept {
        pressure: Pascal,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portion {
    pub species: SpeciesId,
    pub moles: Moles,
    pub phase: Phase,
}

/// The approved thermodynamic model for a finite population of surface sites.
///
/// The first slice deliberately names one model rather than accepting an
/// arbitrary PHREEQC keyword. That keeps the saved state engine-independent
/// and makes every supported surface chemistry an explicit reviewed claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceModel {
    HydrousFerricOxide,
}

/// The two site populations in the Dzombak–Morel hydrous-ferric-oxide model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceSiteKind {
    Strong,
    Weak,
}

/// Sorbates whose surface complexes Kerotakis can currently round-trip.
///
/// This is an enum rather than a free-form species string so a state cannot
/// claim that an unsupported complex was equilibrated. Later reviewed
/// surface reactions extend the enum and their readback mapping together.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceSorbate {
    Zinc,
    Sulfate,
}

impl SurfaceSorbate {
    pub fn species(self) -> SpeciesId {
        match self {
            Self::Zinc => SpeciesId::new("Zn+2"),
            Self::Sulfate => SpeciesId::new("SO4-2"),
        }
    }
}

/// One amount of sorbate held on one surface-site population.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SurfaceOccupancy {
    pub site: SurfaceSiteKind,
    pub sorbate: SurfaceSorbate,
    pub moles: Moles,
}

/// A finite, mass-owning oxide interface and its adsorption ledger.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SurfaceSites {
    /// Stable user-facing identity when a vessel carries several interfaces.
    pub label: String,
    pub model: SurfaceModel,
    /// Physical oxide mass used by the electrostatic surface model, g.
    pub mass: Grams,
    /// Specific surface area, m²/g.
    pub specific_area_m2_per_g: f64,
    pub strong_capacity: Moles,
    pub weak_capacity: Moles,
    /// Computed bound inventory. Empty before the first equilibrium pass.
    #[serde(default)]
    pub occupancy: Vec<SurfaceOccupancy>,
    /// Net water transferred from the neutral `Hfo_*OH` site reference into
    /// the solution by the current ligand-exchange state. The aqueous engine
    /// includes it in solvent mass; subtracting the same amount from the
    /// interface ledger prevents that site material from being counted twice.
    #[serde(default = "zero_moles")]
    pub water_release: Moles,
}

fn zero_moles() -> Moles {
    Moles(0.0)
}

impl SurfaceSites {
    pub fn capacity(&self, site: SurfaceSiteKind) -> Moles {
        match site {
            SurfaceSiteKind::Strong => self.strong_capacity,
            SurfaceSiteKind::Weak => self.weak_capacity,
        }
    }

    pub fn occupied(&self, site: SurfaceSiteKind) -> Moles {
        Moles(
            self.occupancy
                .iter()
                .filter(|entry| entry.site == site)
                .map(|entry| entry.moles.0)
                .sum(),
        )
    }

    pub fn bound(&self, sorbate: SurfaceSorbate) -> Moles {
        Moles(
            self.occupancy
                .iter()
                .filter(|entry| entry.sorbate == sorbate)
                .map(|entry| entry.moles.0)
                .sum(),
        )
    }

    pub fn has_valid_capacity(&self) -> bool {
        self.mass.0.is_finite()
            && self.mass.0 > 0.0
            && self.specific_area_m2_per_g.is_finite()
            && self.specific_area_m2_per_g > 0.0
            && self.strong_capacity.0.is_finite()
            && self.strong_capacity.0 > 0.0
            && self.weak_capacity.0.is_finite()
            && self.weak_capacity.0 > 0.0
            && self.water_release.0.is_finite()
            && self.water_release.0 >= 0.0
            && self.water_release.0 <= self.bound(SurfaceSorbate::Sulfate).0 + 1e-12
            && self.occupancy.iter().all(|entry| {
                entry.moles.0.is_finite()
                    && entry.moles.0 >= 0.0
                    && self.occupied(entry.site).0 <= self.capacity(entry.site).0 + 1e-12
            })
    }
}

/// Cations whose exchanger inventory Kerotakis can currently round-trip.
///
/// Keeping this closed prevents a PHREEQC database from binding an ion that
/// the vessel ledger cannot retain. Extend the enum and the engine readback
/// together whenever another ion is reviewed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExchangeIon {
    Hydrogen,
    Sodium,
    Calcium,
    Magnesium,
}

impl ExchangeIon {
    pub fn species(self) -> SpeciesId {
        match self {
            Self::Hydrogen => SpeciesId::new("H+"),
            Self::Sodium => SpeciesId::new("Na+"),
            Self::Calcium => SpeciesId::new("Ca+2"),
            Self::Magnesium => SpeciesId::new("Mg+2"),
        }
    }

    /// Charge equivalents consumed by one mole of exchanger complex.
    pub fn equivalents(self) -> f64 {
        match self {
            Self::Hydrogen | Self::Sodium => 1.0,
            Self::Calcium | Self::Magnesium => 2.0,
        }
    }

    pub fn molar_mass(self) -> f64 {
        match self {
            // The registry represents acidity through analytical reagents,
            // so the bare proton is not a depositable species of its own.
            Self::Hydrogen => 1.007_94,
            _ => species::lookup(&self.species())
                .map(|data| data.molar_mass)
                .expect("reviewed exchange ion is in the species registry"),
        }
    }
}

/// One cation inventory held on a finite exchanger.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExchangeOccupancy {
    pub ion: ExchangeIon,
    /// Moles of cation, not charge equivalents.
    pub moles: Moles,
}

/// A finite, mass-owning cation exchanger such as sodium-form softener resin.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExchangeSites {
    /// Stable user-facing identity when a vessel carries several exchangers.
    pub label: String,
    /// Dry support mass excluding the exchangeable cations, g.
    pub dry_mass: Grams,
    /// Total negative-site capacity, mol charge equivalents.
    pub capacity: Moles,
    /// Bound cation inventory. A valid exchanger is fully counter-balanced.
    pub occupancy: Vec<ExchangeOccupancy>,
}

impl ExchangeSites {
    pub fn bound(&self, ion: ExchangeIon) -> Moles {
        Moles(
            self.occupancy
                .iter()
                .filter(|entry| entry.ion == ion)
                .map(|entry| entry.moles.0)
                .sum(),
        )
    }

    pub fn occupied_equivalents(&self) -> Moles {
        Moles(
            self.occupancy
                .iter()
                .map(|entry| entry.moles.0 * entry.ion.equivalents())
                .sum(),
        )
    }

    pub fn has_valid_capacity(&self) -> bool {
        if !self.dry_mass.0.is_finite()
            || self.dry_mass.0 <= 0.0
            || !self.capacity.0.is_finite()
            || self.capacity.0 <= 0.0
            || self
                .occupancy
                .iter()
                .any(|entry| !entry.moles.0.is_finite() || entry.moles.0 < 0.0)
        {
            return false;
        }
        let tolerance = (self.capacity.0 * 1e-9).max(1e-12);
        (self.occupied_equivalents().0 - self.capacity.0).abs() <= tolerance
    }
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
    let table = [(10, "X"), (9, "IX"), (5, "V"), (4, "IV"), (1, "I")];
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
    /// Open atmosphere or a finite sealed gas volume. Defaulted so existing
    /// saves retain their historical open-beaker behavior.
    #[serde(default)]
    pub headspace: Headspace,
    /// Finite solid/liquid interfaces. Defaulted for old save compatibility.
    #[serde(default)]
    pub surfaces: Vec<SurfaceSites>,
    /// Finite cation-exchange interfaces. Defaulted for old save compatibility.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exchanges: Vec<ExchangeSites>,
    /// Net charge carried by the dissolved solutes, Σ z·n, mol.
    ///
    /// Not a physical excess — the solution is electroneutral, and the
    /// balance is made up by H⁺ or OH⁻. That is exactly what makes this
    /// useful: a beaker holding 0.1 mol of chloride and nothing else is
    /// holding 0.1 mol of free acid, and one holding 0.1 mol of sodium is
    /// holding 0.1 mol of free base. The number is the vessel's *unspent
    /// acidity*, signed.
    ///
    /// Carried between steps because neutralisation is the amount of that
    /// acidity which cancels when the opposite arrives, and the engine sees
    /// only element totals — it cannot tell an acid that was just added
    /// from one that was always there.
    #[serde(default)]
    pub solute_charge: f64,
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
            headspace: Headspace::Open,
            surfaces: Vec::new(),
            exchanges: Vec::new(),
            solute_charge: 0.0,
            solution: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.contents.is_empty() && self.surfaces.is_empty() && self.exchanges.is_empty()
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

    /// Effective heat capacity of the contents under this vessel's current
    /// mechanical boundary, J/K. Zero for an empty vessel.
    ///
    /// Condensed phases use their tabulated constant-pressure capacity. Gas
    /// does too when a pressure controller may move the boundary. A rigid,
    /// sealed headspace cannot spend heat on `P dV` work, so its ideal-gas
    /// contribution is `Cv = Cp - R`. Open and swept vessels own no ambient
    /// gas inventory; an explicit gas portion there is a finite dose and
    /// carries sensible heat until the chemistry pass absorbs or vents it.
    pub fn heat_capacity(&self) -> f64 {
        const R_JOULE: f64 = 8.314_462_618;
        self.contents
            .iter()
            .filter_map(|portion| {
                let data = species::lookup(&portion.species)?;
                let molar = if portion.phase == Phase::Gas && self.is_sealed() {
                    (data.heat_capacity - R_JOULE).max(0.0)
                } else {
                    data.heat_capacity
                };
                Some(portion.moles.0 * molar)
            })
            .sum()
    }

    /// Sensible energy of the contents relative to 298.15 K, J.
    ///
    /// This is enthalpy for constant-pressure and condensed portions, and
    /// internal energy for gas in a rigid sealed headspace. Heat capacities
    /// are treated as temperature-independent at this stage. The historical
    /// method name remains the public ledger API.
    pub fn enthalpy(&self) -> Joules {
        Joules(self.heat_capacity() * (self.temperature.0 - Kelvin::STANDARD.0))
    }

    /// Total mass, g.
    pub fn mass(&self) -> Grams {
        let contents: f64 = self
            .contents
            .iter()
            .filter_map(|p| species::lookup(&p.species).map(|d| p.moles.0 * d.molar_mass))
            .sum();
        let interfaces: f64 = self
            .surfaces
            .iter()
            .map(|surface| {
                surface.mass.0
                    - surface.water_release.0
                        * species::lookup_key("water")
                            .map(|water| water.molar_mass)
                            .unwrap_or(0.0)
                    + surface
                        .occupancy
                        .iter()
                        .filter_map(|entry| {
                            species::lookup(&entry.sorbate.species())
                                .map(|data| entry.moles.0 * data.molar_mass)
                        })
                        .sum::<f64>()
            })
            .sum();
        let exchangers: f64 = self
            .exchanges
            .iter()
            .map(|exchange| {
                exchange.dry_mass.0
                    + exchange
                        .occupancy
                        .iter()
                        .map(|entry| entry.moles.0 * entry.ion.molar_mass())
                        .sum::<f64>()
            })
            .sum();
        Grams(contents + interfaces + exchangers)
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

    pub fn is_sealed(&self) -> bool {
        matches!(self.headspace, Headspace::Sealed { .. })
    }

    /// Whether gas belongs to the vessel rather than an external reservoir.
    pub fn owns_headspace_gas(&self) -> bool {
        matches!(
            self.headspace,
            Headspace::Sealed { .. } | Headspace::PressureControlled { .. }
        )
    }

    pub fn uses_atmospheric_reservoir(&self) -> bool {
        matches!(self.headspace, Headspace::Open)
    }

    pub fn headspace_volume(&self) -> Option<Liters> {
        match self.headspace {
            Headspace::Open | Headspace::Swept { .. } => None,
            Headspace::Sealed { volume } => Some(volume),
            Headspace::PressureControlled { volume, .. } => Some(volume),
        }
    }

    pub fn gas_moles(&self) -> Moles {
        Moles(
            self.contents
                .iter()
                .filter(|portion| portion.phase == Phase::Gas)
                .map(|portion| portion.moles.0)
                .sum(),
        )
    }

    /// Recompute pressure from the owned gas inventory. The first model is
    /// deliberately the ideal-gas law; PHREEQC owns gas/liquid partitioning,
    /// while this method keeps pressure correct after generic operations.
    pub fn refresh_pressure(&mut self) {
        const R_LITRE_PASCAL: f64 = 8_314.462_618;
        match self.headspace {
            Headspace::Open => self.pressure = Pascal::ATMOSPHERIC,
            Headspace::Sealed { volume } if volume.0 > 0.0 => {
                self.pressure =
                    Pascal(self.gas_moles().0 * R_LITRE_PASCAL * self.temperature.0 / volume.0);
            }
            Headspace::Sealed { .. } => self.pressure = Pascal(0.0),
            Headspace::PressureControlled { pressure, .. } if pressure.0 > 0.0 => {
                let volume =
                    Liters(self.gas_moles().0 * R_LITRE_PASCAL * self.temperature.0 / pressure.0);
                self.headspace = Headspace::PressureControlled { pressure, volume };
                self.pressure = pressure;
            }
            Headspace::PressureControlled { .. } => self.pressure = Pascal(0.0),
            Headspace::Swept { pressure } => self.pressure = pressure,
        }
    }

    /// Put a gas product in an owned headspace. Returns `false` for reservoir
    /// and swept boundaries, whose caller should report the amount as escaped.
    pub fn retain_gas(&mut self, species: SpeciesId, moles: Moles) -> bool {
        if !self.owns_headspace_gas() {
            return false;
        }
        self.deposit(species, moles, Phase::Gas);
        self.refresh_pressure();
        true
    }
}
