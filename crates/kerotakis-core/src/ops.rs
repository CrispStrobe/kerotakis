//! Operators: everything a person can do to the bench, and everything the
//! bench can report back. The operator log is the save file and the API
//! contract (PLAN.md).

use serde::{Deserialize, Serialize};

use crate::species::{Phase, SpeciesId};
use crate::units::{Joules, Kelvin, Moles};
use crate::vessel::VesselId;

/// A mutating or measuring action. One `Operator` in is one step of the bench
/// loop: L0 safety pass → apply → re-equilibrate → events out.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Operator {
    /// Create a new empty vessel on the bench.
    NewVessel,
    /// Add an amount of a species to a vessel, entering at `at` temperature
    /// (defaults to standard).
    Add {
        vessel: VesselId,
        species: SpeciesId,
        moles: Moles,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        at: Option<Kelvin>,
    },
    /// Put energy into a vessel (burner, heating mantle). Negative energy is
    /// expressed with `Cool`.
    Heat { vessel: VesselId, energy: Joules },
    /// Remove energy from a vessel (ice bath).
    Cool { vessel: VesselId, energy: Joules },
    /// Stir. Currently affects nothing the solvers model; logged for the
    /// record and honest about it.
    Stir { vessel: VesselId },
    /// Pour a fraction (0..=1) of the liquid contents into another vessel.
    Decant {
        from: VesselId,
        to: VesselId,
        fraction: f64,
    },
    /// Pour everything through filter paper: liquid and dissolved matter
    /// pass into `to` (the filtrate), solids stay behind in `from`.
    Filter { from: VesselId, to: VesselId },
    /// Apply an ignition source — a match, a spark. If nothing in the
    /// vessel catches, the spark's heat dissipates and the vessel is left
    /// as it was.
    Ignite { vessel: VesselId },
    /// Boil/let evaporate a fraction (0..=1) of the water. Volatile
    /// non-water liquids need L3 (relative volatility) and are honestly
    /// flagged.
    Evaporate { vessel: VesselId, fraction: f64 },
    /// Read an instrument. Never mutates state.
    Measure {
        vessel: VesselId,
        instrument: Instrument,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Instrument {
    Thermometer,
    Balance,
    PhMeter,
    /// Your own eyes. The first instrument anyone uses, and the only one a
    /// young learner needs to start.
    Eyes,
}

/// What one step produced. Everything user-visible derives from this.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum Event {
    VesselCreated {
        vessel: VesselId,
    },
    Added {
        vessel: VesselId,
        species: SpeciesId,
        moles: Moles,
    },
    TemperatureChanged {
        vessel: VesselId,
        from: Kelvin,
        to: Kelvin,
    },
    Transferred {
        from: VesselId,
        to: VesselId,
        fraction: f64,
    },
    /// A solid went into solution (computed by an aqueous solver).
    Dissolved {
        vessel: VesselId,
        species: SpeciesId,
        moles: Moles,
    },
    /// A solid was used up by a reaction.
    Consumed {
        vessel: VesselId,
        species: SpeciesId,
        moles: Moles,
    },
    /// A solid formed out of solution (computed by an aqueous solver).
    Precipitated {
        vessel: VesselId,
        species: SpeciesId,
        moles: Moles,
    },
    /// The aqueous solver characterised the solution (reported when the
    /// values change appreciably).
    SolutionCharacterized {
        vessel: VesselId,
        ph: f64,
        ionic_strength: f64,
    },
    Measured {
        vessel: VesselId,
        instrument: Instrument,
        value: f64,
        unit: String,
    },
    /// What the vessel looks like.
    Observed {
        vessel: VesselId,
        appearance: crate::appearance::Appearance,
    },
    /// L0 recognised a real-world hazard. The simulation proceeds — that is
    /// the pedagogy — but this event always precedes the chemistry.
    HazardWarning {
        severity: crate::solve::Severity,
        hazard: String,
        real_world: String,
    },
    /// L0 refused the operation (product-safety boundary). Nothing was
    /// mutated.
    SafetyVeto {
        reason: String,
    },
    /// Everything liquid passed the filter; solids stayed behind.
    Filtered {
        from: VesselId,
        to: VesselId,
    },
    /// Water left as vapour.
    Evaporated {
        vessel: VesselId,
        moles: Moles,
    },
    /// A gas formed and left the open vessel. The balance notices.
    GasEvolved {
        vessel: VesselId,
        species: SpeciesId,
        moles: Moles,
    },
    /// A curated reaction transformed the contents. The equation is shown at
    /// student register and above; the observations arrive as their own
    /// events.
    ReactionOccurred {
        vessel: VesselId,
        equation: String,
    },
    /// Something caught fire, with the light it gives off.
    Ignited {
        vessel: VesselId,
        /// The flame's colour, where the burning substance has a
        /// characteristic one.
        flame: Option<String>,
    },
    /// An ignition source was applied and nothing caught.
    DidNotIgnite {
        vessel: VesselId,
    },
    /// It would not burn, but it coloured the flame — the flame test.
    FlameTest {
        vessel: VesselId,
        species: SpeciesId,
        colour: String,
    },
    /// A thermal (gas/condensed) equilibrium was computed for this vessel.
    ThermalEquilibrium {
        vessel: VesselId,
        temperature: Kelvin,
        provenance: crate::vessel::Provenance,
    },
    /// The state is one no wired solver models yet. State is unchanged
    /// except for the honest bookkeeping already performed; the renderer says
    /// so at every register.
    NotYetModeled {
        vessel: VesselId,
        what: String,
    },
    /// The solvent changed state: froze, melted or boiled.
    ///
    /// Carries the transition temperature *this* solution has rather than
    /// the pure solvent's, plus how far the dissolved particles moved it.
    /// That shift is the observable content of colligative properties — the
    /// reason salt clears an icy road — so it travels with the event
    /// instead of having to be recomputed by whoever renders it.
    StateChanged {
        vessel: VesselId,
        species: SpeciesId,
        from: Phase,
        to: Phase,
        at: Kelvin,
        /// K away from the pure solvent's transition. Negative when
        /// dissolved particles have lowered it.
        shifted_by: f64,
    },
    /// A solver was asked and could not converge / answer. First-class,
    /// honest, never a crash.
    SolverFailed {
        vessel: VesselId,
        solver: String,
        detail: String,
    },
}

/// One entry of the bench log: the operator plus what it produced.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogEntry {
    pub step: usize,
    pub operator: Operator,
    pub events: Vec<Event>,
}
