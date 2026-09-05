//! Vessel state: contents, temperature, pressure, thermal mode.

use serde::{Deserialize, Serialize};

use crate::enzyme::EnzymeFamily;
use crate::material::MaterialBasis;
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

/// The explicitly unresolved balance of a named material addition.
///
/// This is matter the recipe admits it cannot yet map to canonical species.
/// Keeping it in vessel state prevents later UI and persistence layers from
/// silently pretending that the named material was completely characterized.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnresolvedMaterialPortion {
    pub material: String,
    pub recipe_id: String,
    pub recipe_version: u32,
    pub basis: MaterialBasis,
    pub amount: f64,
    /// Bounded progress within matter that deliberately remains unresolved.
    /// Hydrolysis changes its structure, not its conserved mass.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enzyme_hydrolysis: Option<EnzymeHydrolysisState>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnzymeHydrolysisState {
    /// The catalyst family whose progress this is — or, for a portion that
    /// is an enzyme SOURCE rather than a substrate, the family it carries.
    pub family: EnzymeFamily,
    pub converted_fraction: f64,
    /// Set once this portion has been held above the denaturation
    /// temperature of the enzyme it carries. It is never cleared: a cooked
    /// pineapple does not become raw again when the beaker cools, and that
    /// irreversibility is the entire difference the row asks about.
    #[serde(default)]
    pub carried_enzyme_denatured: bool,
}

/// A coherent prepared object whose resolved ingredients remain owned by the
/// object rather than joining the bulk vessel phases. This is the minimum
/// state needed for membranes and prepared food surfaces: chemistry can inspect
/// the object's inventory without losing its identity. Additive/defaulted for
/// backward-compatible snapshots.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialObject {
    pub material: String,
    pub recipe_id: String,
    pub recipe_version: u32,
    pub mass_g: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub components: Vec<ObjectComponent>,
    #[serde(default)]
    pub state: MaterialObjectState,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectComponent {
    pub species: SpeciesId,
    pub moles: Moles,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MaterialObjectState {
    #[serde(default)]
    pub elapsed_seconds: f64,
    #[serde(default)]
    pub exchanged_water_moles: f64,
    #[serde(default)]
    pub browned_fraction: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SoapScumState {
    pub aggregate_mass_g: f64,
    pub divalent_ion_moles: f64,
    pub soap_equivalent_moles: f64,
}

/// Geometry/appearance state for the one reviewed invisible-ink system.
/// Matter remains in the ordinary paper and lemon-juice ledgers; this records
/// only whether that specific mark has dried and darkened.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct LemonPaperMarkState {
    pub lemon_amount_g: f64,
    pub paper_amount_g: f64,
    pub dry: bool,
    pub browned_fraction: f64,
}

/// Persistent visual state for gas trapped by a declared foam stabilizer.
/// Chemistry still owns every gas mole; this only describes a temporary
/// bubble structure while it drains and coalesces.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FoamState {
    pub trapped_gas_liters: f64,
    pub volume_liters: f64,
    pub peak_volume_liters: f64,
}

/// Persistent view of unresolved grains floating at a liquid surface. The
/// values are recipe-declared classroom observables; no grain-scale flow field
/// or molecular surface tension is implied.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SurfaceParticleState {
    pub material: String,
    pub coverage_fraction: f64,
    pub cleared_fraction: f64,
}

/// Resolved dye temporarily localized at an opaque liquid surface. The dye
/// moles remain in `contents`; this state records only their geometry and is
/// therefore also the exact inventory excluded from the homogeneous optical
/// calculation until the surface is disturbed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SurfaceColourSpot {
    pub material: String,
    pub species: SpeciesId,
    pub moles: Moles,
    pub srgb: [u8; 3],
    pub spread_fraction: f64,
}

/// Persistent amount of an unresolved oil layer dispersed as droplets in an
/// aqueous phase. The bulk oil remains in `unresolved_materials`; this state
/// only records its temporary geometry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct EmulsionState {
    pub oil_recipe_id: String,
    pub dispersed_volume_l: f64,
    pub half_life_seconds: f64,
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

/// The drawable glassware kinds `new` accepts, each with the light path
/// its geometry gives Beer–Lambert. This is why choosing a vessel is a
/// chemistry decision: the same permanganate is pale in a slim tube and
/// deep purple in a beaker, and the colour pipeline uses THIS number.
pub const VESSEL_KINDS: &[(&str, f64)] = &[
    ("beaker", 4.0),
    ("flask", 3.5),
    ("tube", 1.2),
    ("cylinder", 2.2),
    ("crucible", 3.0),
];

/// The Beer–Lambert path for a vessel label; unknown labels read as the
/// classic beaker rather than guessing.
pub fn path_cm_for(label: &str) -> f64 {
    VESSEL_KINDS
        .iter()
        .find(|(k, _)| *k == label)
        .map(|(_, p)| *p)
        .unwrap_or(4.0)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

/// Reviewed thermodynamic models for a mixed crystalline phase.
///
/// A closed enum prevents a save from claiming that an arbitrary pair and
/// arbitrary interaction parameters were validated. New pairs extend the
/// model, component mapping, and live-engine checks together.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SolidSolutionModel {
    /// The non-ideal CaCO3-SrCO3 pair from PHREEQC example 10.
    AragoniteStrontianite,
}

/// One end member whose inventory Kerotakis can round-trip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SolidSolutionComponent {
    CalciumCarbonate,
    StrontiumCarbonate,
}

impl SolidSolutionComponent {
    pub const ALL: [Self; 2] = [Self::CalciumCarbonate, Self::StrontiumCarbonate];

    pub fn species(self) -> SpeciesId {
        match self {
            Self::CalciumCarbonate => SpeciesId::new("CaCO3"),
            Self::StrontiumCarbonate => SpeciesId::new("SrCO3"),
        }
    }

    pub fn molar_mass(self) -> f64 {
        match self {
            Self::CalciumCarbonate => 100.087,
            Self::StrontiumCarbonate => 147.628,
        }
    }
}

/// The amount of one end member in a mixed crystal, in formula-unit moles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolidSolutionAmount {
    pub component: SolidSolutionComponent,
    pub moles: Moles,
}

/// A finite, mass-owning mixed crystalline phase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolidSolution {
    pub label: String,
    pub model: SolidSolutionModel,
    pub components: Vec<SolidSolutionAmount>,
}

impl SolidSolution {
    pub fn aragonite_strontianite(
        label: impl Into<String>,
        calcium_carbonate: Moles,
        strontium_carbonate: Moles,
    ) -> Self {
        Self {
            label: label.into(),
            model: SolidSolutionModel::AragoniteStrontianite,
            components: vec![
                SolidSolutionAmount {
                    component: SolidSolutionComponent::CalciumCarbonate,
                    moles: calcium_carbonate,
                },
                SolidSolutionAmount {
                    component: SolidSolutionComponent::StrontiumCarbonate,
                    moles: strontium_carbonate,
                },
            ],
        }
    }

    pub fn moles_of(&self, component: SolidSolutionComponent) -> Moles {
        Moles(
            self.components
                .iter()
                .filter(|entry| entry.component == component)
                .map(|entry| entry.moles.0)
                .sum(),
        )
    }

    pub fn total_moles(&self) -> Moles {
        Moles(self.components.iter().map(|entry| entry.moles.0).sum())
    }

    pub fn mass(&self) -> Grams {
        Grams(
            self.components
                .iter()
                .map(|entry| entry.moles.0 * entry.component.molar_mass())
                .sum(),
        )
    }

    pub fn has_valid_state(&self) -> bool {
        !self.label.trim().is_empty()
            && self.components.len() == SolidSolutionComponent::ALL.len()
            && SolidSolutionComponent::ALL.iter().all(|component| {
                self.components
                    .iter()
                    .filter(|entry| entry.component == *component)
                    .count()
                    == 1
            })
            && self
                .components
                .iter()
                .all(|entry| entry.moles.0.is_finite() && entry.moles.0 >= 0.0)
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
        self.pe
            .map(|pe| pe * crate::relations::nernst_slope(crate::units::Kelvin(temperature_k)))
    }
}

// ── ARCH-004: MaterialLot ──────────────────────────────────────────

pub const DRY_YEAST_RECIPE_SOURCE: &str = "material recipe household/dry-yeast-catalase-surrogate";

/// A batch of material with its addition provenance (ARCH-004).
///
/// Lots track what was added, when, and from where, independently of
/// how solvers resolve species. Two lots can merge physically without
/// losing provenance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialLot {
    /// What was added (the user-facing name, e.g. "NaCl", "water").
    pub species: SpeciesId,
    /// How much was added, in moles.
    pub moles: Moles,
    /// Which phase was intended.
    pub phase: Phase,
    /// When this lot was added (elapsed seconds at time of addition).
    pub added_at: f64,
    /// First contact with a liquid phase, when known. Enzyme-bearing dry
    /// materials use this to distinguish dry storage from hydration time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hydrated_at: Option<f64>,
    /// Where this came from (e.g. "reagent bottle", "transfer from v2").
    #[serde(default)]
    pub source: Option<String>,
    /// Particle-size metadata for solids, if relevant (mean diameter in µm).
    #[serde(default)]
    pub particle_size_um: Option<f64>,
    /// Fraction of this solid lot currently suspended in its liquid medium.
    /// `None` preserves legacy state whose suspension was never tracked.
    #[serde(default)]
    pub suspended_fraction: Option<f64>,
}

// ── ARCH-005: ResolvedState ───────────────────────────────────────

/// Derived state that is invalidated when primary state changes (ARCH-005).
///
/// Contains everything that solvers compute from the primary contents:
/// aqueous characterization, phase equilibrium, saturation indices.
/// Setting `valid = false` marks the state as stale; solvers must
/// recompute before the next observation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResolvedState {
    /// Whether the resolved state is current (false after any mutation).
    #[serde(default)]
    pub valid: bool,
    /// Aqueous solver output, if any.
    #[serde(default)]
    pub solution: Option<SolutionInfo>,
}

impl ResolvedState {
    pub fn invalidate(&mut self) {
        self.valid = false;
        self.solution = None;
    }
}

/// What a step started from — see [`Vessel::step_start`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepStart {
    /// The ledger with the operator applied, before any solver.
    pub contents: Vec<Portion>,
    pub solute_charge: f64,
    pub temperature: Kelvin,
    /// Gas that has LEFT the vessel so far this step — `GasEvolved` minus
    /// `GasAbsorbed`, by species — accumulated by `SolverStack` between
    /// solvers, so a solver that runs late sees what an earlier one gave
    /// off. Gas kept in a headspace is not here: it is a `Phase::Gas`
    /// portion in `contents` already, and would be counted twice.
    pub gas_out: Vec<(SpeciesId, Moles)>,
}

impl StepStart {
    pub fn capture(vessel: &Vessel) -> Self {
        StepStart {
            contents: vessel.contents.clone(),
            solute_charge: vessel.solute_charge,
            temperature: vessel.temperature,
            gas_out: Vec::new(),
        }
    }

    /// Book an outward gas transfer (negative moles for gas taken back in).
    pub fn note_gas_out(&mut self, species: &SpeciesId, moles: f64) {
        match self.gas_out.iter_mut().find(|(s, _)| s == species) {
            Some((_, total)) => total.0 += moles,
            None => self.gas_out.push((species.clone(), Moles(moles))),
        }
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
    /// EXP-49: tracer-scale radionuclide inventory, deliberately
    /// separate from the chemical contents (¹⁴C and ¹²C are one
    /// element to the chemistry and two nuclides here). Chemically
    /// inert at tracer scale — the stated v1 boundary.
    #[serde(default)]
    pub nuclides: crate::nuclide::NuclideLedger,
    /// EXP-44: the vessel's last-known total excess enthalpy (J), the
    /// state-function anchor for incremental heat-of-mixing.
    #[serde(default)]
    pub excess_enthalpy_j: f64,
    pub id: VesselId,
    pub label: String,
    pub contents: Vec<Portion>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unresolved_materials: Vec<UnresolvedMaterialPortion>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub material_objects: Vec<MaterialObject>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub soap_scum: Option<SoapScumState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lemon_paper_mark: Option<LemonPaperMarkState>,
    #[serde(default)]
    pub foam: FoamState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surface_particles: Option<SurfaceParticleState>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub surface_colours: Vec<SurfaceColourSpot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emulsion: Option<EmulsionState>,
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
    /// Finite mixed crystalline phases. Defaulted for old save compatibility.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub solid_solutions: Vec<SolidSolution>,
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
    /// The vessel as it stood when the current step's operator had been
    /// applied and no solver had yet run, plus the gas that has left it so
    /// far this step. `Some` only while the bench is inside `step_with`:
    /// set right after the operator, cleared once the solver stack
    /// returns.
    ///
    /// Why it exists: an enthalpy balance is a state function over the
    /// STEP, not over one solver's call. A curated row that runs before the
    /// aqueous tail consumes the bicarbonate and evolves the carbon dioxide
    /// itself; a balance that took its "before" from the tail's own
    /// call-start would price a step in which the bicarbonate had never
    /// been there and the carbon simply ceased to exist. The tail reads
    /// this and falls back to its call-start where a host never sets it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_start: Option<StepStart>,
    /// Free hydroxide the aqueous solver last MEASURED, in moles.
    ///
    /// Kept beside `solute_charge` and for the same reason — the heat
    /// balance needs to know what the beaker was holding at the start of a
    /// step — but it is not the same quantity and must not be derived from
    /// it. Net charge is free base only in a vessel of strong electrolytes:
    /// a bicarbonate solution carries its charge excess as carbonate
    /// alkalinity, and a beaker handed a bare cation carries it as nothing
    /// at all. Reading either as hydroxide invents a neutralisation that
    /// never happened, at 55.81 kJ for every mole of it.
    ///
    /// `solution` is cleared at the top of every step as stale, so the
    /// measurement cannot be recovered from there; this survives because
    /// the tail writes it on the way out.
    #[serde(default)]
    pub free_hydroxide: f64,
    /// Free protons the aqueous solver last MEASURED, in moles. The mirror
    /// of `free_hydroxide`, and persisted for the same reason: `solution`
    /// is cleared at the top of every step, so a solver ABOVE the aqueous
    /// tail — a reaction-family catalyst gate, say — cannot read the
    /// measurement any other way.
    ///
    /// **This is not `displacement::unspent_acidity`, and the difference is
    /// large.** Unspent acidity is the titratable total: every proton the
    /// vessel could eventually give up. This is how many are actually loose
    /// right now. For a strong acid they agree — 0.1 mol of HCl in 100 g of
    /// water measures 0.1 mol of free protons. For a weak one they do not
    /// come close: the same amount of acetic acid is 0.1 mol titratable and
    /// 4.5e-4 mol free, a factor of 220. A gate that asks "is this vessel
    /// acidic enough to catalyse" wants this one; a ledger that asks "how
    /// much acid is there to spend" wants the other. Reaching for whichever
    /// is to hand is how a quantity comes to be computed from something
    /// that merely correlates with it.
    #[serde(default)]
    pub free_proton: f64,
    /// `Some` once an aqueous solver has characterised the solution; `None`
    /// means no solver has — and the honesty pass says so.
    #[serde(default)]
    pub solution: Option<SolutionInfo>,
    /// ARCH-004: Material lots tracking provenance of additions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lots: Vec<MaterialLot>,
    /// ARCH-005: Solver-derived state, invalidated on mutation.
    #[serde(default)]
    pub resolved: ResolvedState,
}

impl Vessel {
    pub fn new(id: VesselId, label: impl Into<String>) -> Self {
        Vessel {
            elapsed_seconds: 0.0,
            nuclides: Default::default(),
            excess_enthalpy_j: 0.0,
            id,
            label: label.into(),
            contents: Vec::new(),
            unresolved_materials: Vec::new(),
            material_objects: Vec::new(),
            soap_scum: None,
            lemon_paper_mark: None,
            foam: FoamState::default(),
            surface_particles: None,
            surface_colours: Vec::new(),
            emulsion: None,
            temperature: Kelvin::STANDARD,
            pressure: Pascal::ATMOSPHERIC,
            thermal_mode: ThermalMode::Adiabatic,
            headspace: Headspace::Open,
            surfaces: Vec::new(),
            exchanges: Vec::new(),
            solid_solutions: Vec::new(),
            solute_charge: 0.0,
            step_start: None,
            free_hydroxide: 0.0,
            free_proton: 0.0,
            solution: None,
            lots: Vec::new(),
            resolved: ResolvedState::default(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.contents.is_empty()
            && self.unresolved_materials.is_empty()
            && self.material_objects.is_empty()
            && self.surfaces.is_empty()
            && self.exchanges.is_empty()
            && self.solid_solutions.is_empty()
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

    /// Deposit with provenance tracking (ARCH-004).
    pub fn deposit_lot(
        &mut self,
        species: SpeciesId,
        moles: Moles,
        phase: Phase,
        source: Option<String>,
        particle_size_um: Option<f64>,
    ) {
        let suspended_fraction = (phase == Phase::Solid).then(|| {
            if self.liquid_volume().0 > 0.0 && !crate::displacement::is_elemental_metal(&species.0)
            {
                1.0
            } else {
                0.0
            }
        });
        self.deposit(species.clone(), moles, phase);
        self.lots.push(MaterialLot {
            species,
            moles,
            phase,
            added_at: self.elapsed_seconds,
            hydrated_at: None,
            source,
            particle_size_um,
            suspended_fraction,
        });
        self.mark_liquid_contact();
        self.resolved.invalidate();
    }

    /// Record first liquid contact without erasing provenance or resetting a
    /// material that was already hydrated before later transfers.
    pub fn mark_liquid_contact(&mut self) {
        if self.liquid_volume().0 <= 0.0 {
            return;
        }
        for lot in &mut self.lots {
            if lot.source.as_deref() == Some(DRY_YEAST_RECIPE_SOURCE) && lot.hydrated_at.is_none() {
                lot.hydrated_at = Some(self.elapsed_seconds);
            }
        }
    }

    /// Mole-weighted suspended fraction for explicitly tracked solid lots.
    /// `None` means this is legacy/solver-created solid state with no split.
    pub fn suspended_fraction_of(&self, species: &SpeciesId) -> Option<f64> {
        let mut tracked_moles = 0.0;
        let mut suspended_moles = 0.0;
        for lot in self
            .lots
            .iter()
            .filter(|lot| lot.species == *species && lot.phase == Phase::Solid)
        {
            let Some(fraction) = lot.suspended_fraction else {
                continue;
            };
            tracked_moles += lot.moles.0;
            suspended_moles += lot.moles.0 * fraction.clamp(0.0, 1.0);
        }
        (tracked_moles > 0.0).then_some((suspended_moles / tracked_moles).clamp(0.0, 1.0))
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

    /// KID-7: remove up to `moles` of a species from one phase only.
    ///
    /// `withdraw` takes from whichever portions it meets first, which is the
    /// right thing when a species is leaving the vessel. Crystallisation is
    /// not that: it moves a solute from the aqueous compartment into the
    /// solid one, and taking the shortfall out of the solid it is trying to
    /// grow would be exactly backwards.
    pub fn withdraw_phase(&mut self, species: &SpeciesId, moles: Moles, phase: Phase) -> Moles {
        let mut remaining = moles.0;
        for p in self.contents.iter_mut() {
            if &p.species == species && p.phase == phase && remaining > 0.0 {
                let take = p.moles.0.min(remaining);
                p.moles = Moles(p.moles.0 - take);
                remaining -= take;
            }
        }
        for lot in self.lots.iter_mut() {
            if &lot.species == species && lot.phase == phase && remaining <= 0.0 {
                break;
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
        self.contents
            .iter()
            .filter_map(|portion| {
                let data = species::lookup(&portion.species)?;
                let molar = if portion.phase == Phase::Gas && self.is_sealed() {
                    (data.heat_capacity - crate::constants::GAS_CONSTANT).max(0.0)
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
        let solid_solutions: f64 = self
            .solid_solutions
            .iter()
            .map(|solid_solution| solid_solution.mass().0)
            .sum();
        let unresolved_materials = crate::material::unresolved_material_mass_g(self);
        let material_objects: f64 = self
            .material_objects
            .iter()
            .map(|object| object.mass_g)
            .sum();
        Grams(
            contents
                + interfaces
                + exchangers
                + solid_solutions
                + unresolved_materials
                + material_objects
                + self
                    .soap_scum
                    .as_ref()
                    .map_or(0.0, |scum| scum.aggregate_mass_g),
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
