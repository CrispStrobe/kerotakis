//! Operators: everything a person can do to the bench, and everything the
//! bench can report back. The operator log is the save file and the API
//! contract (PLAN.md).

use serde::{Deserialize, Serialize};

use crate::species::{Phase, SpeciesId};
use crate::units::{Joules, Kelvin, Liters, Moles, Pascal};
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
    /// Close a vessel over a finite gas volume, trapping the room air that
    /// occupied it at the current temperature.
    Seal {
        vessel: VesselId,
        headspace_volume: Liters,
    },
    /// Close the material boundary under a movable piston. Gas remains in
    /// the vessel while its volume changes to maintain the target pressure.
    Regulate {
        vessel: VesselId,
        pressure: Pascal,
        initial_volume: Liters,
    },
    /// Apply an inert purge that carries volatile products away.
    Sweep { vessel: VesselId, pressure: Pascal },
    /// Open a vessel and release every gas portion to the room.
    Open { vessel: VesselId },
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
    /// Let time pass. Rates need a clock, and this is it.
    ///
    /// Deliberately not per-vessel: every vessel on the bench advances by
    /// the same interval, because time is not something one beaker has more
    /// of than another. Two beakers and one changed variable is the whole
    /// design of a fair test, and it only works if the clock is shared.
    Wait { seconds: f64 },
    /// Read an instrument. Never mutates state.
    Measure {
        vessel: VesselId,
        instrument: Instrument,
    },
    /// Wire two vessels as a galvanic cell and read the voltmeter. Open
    /// circuit: no current flows and nothing changes, which is what the
    /// activity series predicts — the voltage a battery *would* have.
    Cell { a: VesselId, b: VesselId },
    /// Drive a current through a vessel for a time: electrolysis.
    ///
    /// The cell operator asks what voltage a pair *produces*. This asks
    /// what a current *moves*, which is the other half of the same idea and
    /// the one with a number a learner can weigh on a balance.
    Electrolyse {
        vessel: VesselId,
        amps: f64,
        seconds: f64,
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
    /// INST-003: Gas pressure gauge — reads headspace pressure.
    PressureGauge,
    /// INST-003: Gas volume meter — reads headspace volume.
    VolumeMeter,
    /// INST-004: Conductivity meter — reads solution conductivity.
    ConductivityMeter,
    /// INST-005: UV-Vis spectrophotometer — reads absorbance spectrum.
    Spectrophotometer,
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
        /// What is left of that species in the vessel afterwards, where
        /// the emitter knows. The event used to carry only what went,
        /// and the lv1 sentence "is used up" claimed a completeness it
        /// could not see — half a magnesium ribbon beside its plated
        /// copper was reported gone. `None` means the emitter did not
        /// say; the renderer then claims only that it is being used up.
        /// Defaulted so a log written before the field existed, or a
        /// browser that never sends it, still reads.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        remaining: Option<Moles>,
    },
    /// A metal came out of solution onto a more reactive one — displacement
    /// (computed by the activity series over the solver's activities).
    /// Not a precipitate: it grows as a coating on the metal that gave up
    /// the electrons, which is how a learner recognises it.
    Plated {
        vessel: VesselId,
        species: SpeciesId,
        onto: SpeciesId,
        moles: Moles,
    },
    /// A substance was examined by a chemistry model and found not to
    /// react, with the reason. Distinct from `NotYetModeled` on purpose:
    /// copper in dilute acid doing nothing is a computed result about
    /// copper, and reporting it as a gap would be as wrong as reporting a
    /// gap as a result.
    Inert {
        vessel: VesselId,
        species: SpeciesId,
        why: String,
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
    /// The open-circuit voltage between two half-cells, and which way the
    /// electrons would go. Open circuit means: no current drawn, no
    /// internal resistance, nothing in either beaker changed — the
    /// voltmeter reading the moment the wires touch, not the voltage under
    /// load and not how long a torch would run (that is Faraday's
    /// question, answered elsewhere).
    /// Charge went through, and this much metal moved with it.
    ///
    /// Every field is carried because Faraday's law is an arithmetic chain
    /// a learner is expected to walk — coulombs, then moles of electrons,
    /// then moles of substance, then grams — and a register that prints
    /// only the last number teaches the answer instead of the method.
    Electrolysed {
        vessel: VesselId,
        species: SpeciesId,
        coulombs: f64,
        electrons: Moles,
        moles: Moles,
        grams: f64,
        /// Electrons per ion, the `z` that makes the division a chemistry
        /// question rather than an arithmetic one.
        per_ion: f64,
    },
    CellVoltage {
        anode: VesselId,
        cathode: VesselId,
        volts: f64,
        /// The standard cell potential from E° alone, for the comparison
        /// that makes Nernst visible.
        standard_volts: f64,
        /// `Zn | Zn+2 ‖ Cu+2 | Cu`.
        notation: String,
        /// The reaction that would run if the circuit were closed.
        equation: String,
    },
    /// Two vessels were wired and no cell exists between them, with the
    /// reason. A computed answer about the beakers, not a gap in the lab.
    NoCell {
        a: VesselId,
        b: VesselId,
        why: String,
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
    /// A gas formed and left through a reservoir or swept boundary. The
    /// balance notices.
    GasEvolved {
        vessel: VesselId,
        species: SpeciesId,
        moles: Moles,
    },
    /// Gas crossed inward from an external boundary and remains in the
    /// liquid or a condensed phase.
    GasAbsorbed {
        vessel: VesselId,
        species: SpeciesId,
        moles: Moles,
    },
    /// A gas formed but remained in a material-closed headspace.
    GasContained {
        vessel: VesselId,
        species: SpeciesId,
        moles: Moles,
    },
    VesselSealed {
        vessel: VesselId,
        headspace_volume: Liters,
        trapped_air: Moles,
    },
    VesselPressureControlled {
        vessel: VesselId,
        pressure: Pascal,
        initial_volume: Liters,
        trapped_gas: Moles,
    },
    VesselSwept {
        vessel: VesselId,
        pressure: Pascal,
    },
    VesselOpened {
        vessel: VesselId,
    },
    /// Gas/liquid equilibrium settled in a finite headspace.
    HeadspaceEquilibrated {
        vessel: VesselId,
        pressure: Pascal,
        total_moles: Moles,
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
    /// Time passed, and this is what it did.
    Reacted {
        vessel: VesselId,
        /// The kinetic reaction's id.
        reaction: String,
        equation: String,
        /// How far it ran in this interval.
        moles: Moles,
        /// Seconds of bench time that elapsed.
        seconds: f64,
        /// The catalyst in force, if any, and the barrier it provided.
        catalyst: Option<String>,
        /// Activation energy actually used, J/mol.
        activation_energy: f64,
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

impl Event {
    /// Whether a person would notice this.
    ///
    /// The event stream is the *ledger*: it has to account for everything
    /// that happened, including the third of a micromole of carbon dioxide
    /// that leaves an open beaker while nobody is looking. Applying the
    /// observability floor at the moment of *recording* made the books not
    /// balance — matter left the vessel with no entry against it — so the
    /// floor belongs here, at the moment of *telling*, where it started as
    /// a question about what a learner should be shown.
    pub fn is_observable(&self) -> bool {
        let amount = match self {
            Event::Dissolved { moles, .. }
            | Event::Precipitated { moles, .. }
            | Event::GasEvolved { moles, .. }
            | Event::GasAbsorbed { moles, .. }
            | Event::GasContained { moles, .. }
            | Event::Consumed { moles, .. }
            | Event::Plated { moles, .. }
            | Event::Reacted { moles, .. } => moles.0,
            _ => return true,
        };
        amount >= crate::OBSERVABLE_MOLES
    }
}
