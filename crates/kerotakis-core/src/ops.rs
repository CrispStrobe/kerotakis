//! Operators: everything a person can do to the bench, and everything the
//! bench can report back. The operator log is the save file and the API
//! contract (PLAN.md).

use serde::{Deserialize, Serialize};

use crate::material::MaterialBasis;
use crate::species::{Phase, SpeciesId};
use crate::units::{Joules, Kelvin, Liters, Moles, Pascal};
use crate::vessel::VesselId;

fn one_molar() -> f64 {
    1.0
}

fn one_stage() -> u32 {
    1
}
fn kelvin_zero() -> Kelvin {
    Kelvin(0.0)
}
fn default_stir_rpm() -> f64 {
    500.0
}
fn default_stir_seconds() -> f64 {
    10.0
}

/// A mutating or measuring action. One `Operator` in is one step of the bench
/// loop: L0 safety pass → apply → re-equilibrate → events out.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Operator {
    /// Create a new empty vessel on the bench.
    NewVessel {
        /// Glassware kind ("beaker", "flask", "tube", "cylinder",
        /// "crucible"); absent means beaker. Optional so every log written
        /// before kinds existed still replays.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kind: Option<String>,
    },
    /// Put an empty vessel back into storage. Matter is never silently
    /// discarded through this operation, and the bench keeps one receiver.
    RemoveVessel { vessel: VesselId },
    /// Add an amount of a species to a vessel, entering at `at` temperature
    /// (defaults to standard).
    Add {
        vessel: VesselId,
        species: SpeciesId,
        moles: Moles,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        at: Option<Kelvin>,
    },
    /// Dispense a versioned named mixture/object. The recipe identity, version,
    /// basis amount and sample seed are pinned in the operator so replay never
    /// depends on ambient randomness or whichever recipe version is newest.
    AddMaterial {
        vessel: VesselId,
        material: String,
        recipe_id: String,
        recipe_version: u32,
        total_amount: f64,
        basis: MaterialBasis,
        sample_seed: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        at: Option<Kelvin>,
    },
    /// Put energy into a vessel (burner, heating mantle). Negative energy is
    /// expressed with `Cool`.
    Heat { vessel: VesselId, energy: Joules },
    /// Remove energy from a vessel (ice bath).
    Cool { vessel: VesselId, energy: Joules },
    /// Run a magnetic stirrer. The operation owns the mechanical conditions;
    /// chemistry models may consume them when they support transport/rate
    /// coupling, while clients can already render the computed tip speed.
    Stir {
        vessel: VesselId,
        #[serde(default = "default_stir_rpm")]
        rpm: f64,
        #[serde(default = "default_stir_seconds")]
        seconds: f64,
    },
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
    /// Boil volatile liquid over into another vessel through a condenser:
    /// a Rayleigh batch cut — the vapour composition follows the pot as
    /// it drifts — through `stages` ideal stages at total reflux, with
    /// UNIFAC γ(T) for ethanol–water. Ask for a mole fraction of the
    /// charge or for what a latent-heat budget can lift; non-volatile
    /// matter stays behind, which is why distilling brine makes
    /// distilled water.
    Distil {
        from: VesselId,
        to: VesselId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fraction: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        energy: Option<Joules>,
        #[serde(default = "one_stage")]
        stages: u32,
    },
    /// Open the stopcock of a separating funnel: the lower liquid layer
    /// — and everything dissolved in it — runs into the receiver, and
    /// the upper layer stays behind. Only meaningful when the computed
    /// liquid–liquid equilibrium says there *are* layers; one phase has
    /// nothing to drain separately, and the bench says so.
    Drain { from: VesselId, to: VesselId },
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
    /// Set particle size of a solid in the vessel (for heterogeneous rates).
    Grind {
        vessel: VesselId,
        species: SpeciesId,
        diameter_um: f64,
    },
    /// Turn a light source on or off for photolysis.
    Irradiate {
        vessel: VesselId,
        wavelength_nm: f64,
        irradiance_w_m2: f64,
    },
    /// Add solvent (water) by volume. The pedagogical complement of
    /// `evaporate`: where evaporate concentrates, dilute spreads.
    Dilute { vessel: VesselId, volume: Liters },
    /// Waft the vessel's air toward your nose — the taught technique,
    /// never a direct huff. Reports curated odours of headspace gases
    /// and volatile species; hazardous vapours come with the warning a
    /// real bench would enforce.
    Smell { vessel: VesselId },
    /// Spike a vessel with a tracer amount of a curated radionuclide
    /// (EXP-49). Separate from `Add` because nuclides live in the
    /// nuclide ledger, not the chemical registry.
    SpikeNuclide {
        vessel: VesselId,
        /// Notation like "I-131"; must be in the curated teaching set.
        nuclide: String,
        moles: Moles,
    },
    /// Apply a classical bench gas test to the vessel's headspace:
    /// pop (H₂), glowing splint (O₂), limewater (CO₂), damp litmus (NH₃).
    TestGas {
        vessel: VesselId,
        test: crate::gas_tests::GasTest,
    },
    /// Apply a named curated organic transformation on command:
    /// `react v1 esterification`. Deliberate, not automatic — the
    /// mixture does not do this on its own at the bench's conditions;
    /// see `curated::ORG_REACTIONS`.
    React { vessel: VesselId, reaction: String },
    /// Auto-stepped titration: add `titrant` to `vessel` in increments of
    /// `step` volume, re-equilibrating after each addition, until the pH
    /// crosses `target_ph` or `max_steps` additions are exhausted. Records
    /// (cumulative volume, pH) at every step.
    Titrate {
        vessel: VesselId,
        titrant: SpeciesId,
        /// Concentration of the standard solution in the burette,
        /// mol/L. The burette holds a solution, not the pure substance
        /// — each step delivers `concentration × step` moles of
        /// titrant plus the carrier water of the step volume.
        #[serde(default = "one_molar")]
        concentration: f64,
        step: Liters,
        target_ph: f64,
        max_steps: u32,
    },
    /// Mix fractions of two solved solutions into a third vessel using
    /// PHREEQC's MIX keyword, which combines them at the thermodynamic
    /// level rather than by raw re-dissolution. Falls back to physical
    /// mixing when the solver cannot honour MIX semantics.
    Mix {
        a: VesselId,
        b: VesselId,
        into: VesselId,
        fraction_a: f64,
        fraction_b: f64,
    },
    /// Hold a magnet over the vessel: ferromagnetic solids jump to the
    /// magnet and are dropped into `to`; everything else stays behind.
    Magnet { from: VesselId, to: VesselId },
    /// Push liquid through a 1-D chain of vessels: conservative upwind
    /// transport with an explicit Courant fraction. The inlet provides the
    /// feed composition (unchanged); the effluent collects in the receiver.
    Transport {
        chain: Vec<VesselId>,
        inlet: VesselId,
        receiver: VesselId,
        steps: u32,
        courant: f64,
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
    /// INST-006: Calorimeter — reads enthalpy.
    Calorimeter,
    /// EXP-49: Geiger counter — total activity of the vessel's
    /// nuclide inventory, in becquerels.
    GeigerCounter,
    /// INST-007: Chromatography column — separates dissolved neutral
    /// solutes by their computed partition coefficients and reports the
    /// peak table. Non-destructive here: an analytical injection is an
    /// aliquot too small for the ledger to see.
    Chromatograph,
}

/// ORG-009: How confident the engine is in a claimed result.
///
/// This distinguishes "the thermodynamics say so" from "a template
/// matched but no rate was computed" from "we guessed." Displayed
/// to users so they know which answers are predictions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    /// Thermodynamic equilibrium or exact stoichiometry — the engine's
    /// strongest claim. PHREEQC speciation, CEA equilibrium, exact
    /// precipitation stoichiometry.
    Computed,
    /// A kinetic model ran and produced this trajectory. The mechanism
    /// is validated but the rate constants carry uncertainty.
    Modeled,
    /// A reaction template matched and the products are correct, but
    /// no rate or equilibrium constant was available. The engine reports
    /// "this reacts" but not "how much" or "how fast."
    TemplateMatch,
    /// A curated lookup or editorial estimate. Useful but not computed
    /// from first principles.
    Curated,
    /// The engine could not determine an answer and is reporting the
    /// honest absence of knowledge.
    Unknown,
}

/// One peak in a reported chromatogram: who, when, how wide, how much.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ElutedPeak {
    pub species: SpeciesId,
    /// Retention time t_R = t₀·(1 + K·β), seconds.
    pub retention_time_s: f64,
    /// Baseline width (4σ) from the plate count, seconds.
    pub width_s: f64,
    /// Area relative to the largest peak — proportional to moles
    /// injected, because an ideal detector counts what passes it.
    pub relative_area: f64,
    /// The computed partition coefficient that put the peak there.
    pub partition_k: f64,
}

/// What one step produced. Everything user-visible derives from this.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum Event {
    VesselCreated {
        vessel: VesselId,
    },
    VesselRemoved {
        vessel: VesselId,
    },
    Added {
        vessel: VesselId,
        species: SpeciesId,
        moles: Moles,
        /// Inventory of this species in the vessel after the dose. Older
        /// event logs did not carry it; clients must tolerate its absence.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        total_after: Option<Moles>,
    },
    /// A named material expanded into canonical species while retaining the
    /// user-facing identity and any chemically unresolved balance.
    MaterialAdded {
        vessel: VesselId,
        material: String,
        recipe_id: String,
        recipe_version: u32,
        total_amount: f64,
        basis: MaterialBasis,
        sample_seed: u64,
        components: Vec<MaterialComponentAdded>,
        unresolved_amount: f64,
    },
    TemperatureChanged {
        vessel: VesselId,
        from: Kelvin,
        to: Kelvin,
    },
    /// Mechanical mixing conditions actually delivered by a magnetic
    /// stirrer. Tip speed follows π·bar_length·rpm/60.
    Stirred {
        vessel: VesselId,
        rpm: f64,
        seconds: f64,
        bar_length_m: f64,
        tip_speed_m_s: f64,
        /// False until kinetics/surface-area models consume this operation.
        rate_coupled: bool,
    },
    /// A mortar changed the mean diameter of a solid powder. Surface area
    /// assumes equal spherical particles: A = 6V/d, using registry density.
    Ground {
        vessel: VesselId,
        species: SpeciesId,
        diameter_um: f64,
        solid_moles: Moles,
        surface_area_m2: f64,
        /// False until a heterogeneous kinetic law consumes this area.
        rate_coupled: bool,
    },
    Transferred {
        from: VesselId,
        to: VesselId,
        fraction: f64,
    },
    /// A solid met an organic solvent and the curated handbook limit
    /// decided how much dissolves (CAP-23 rung 1). Undissociated solute,
    /// no speciation or activity claim — the boundary is the model.
    DissolvedInSolvent {
        vessel: VesselId,
        species: SpeciesId,
        solvent: SpeciesId,
        dissolved: Moles,
        undissolved: Moles,
    },
    /// A metal sat in an organic solvent and the computed answer is
    /// "no reaction at bench conditions", with the reason.
    InertInSolvent {
        vessel: VesselId,
        species: SpeciesId,
        solvent: SpeciesId,
        why: String,
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
    /// Magnetic solids jumped to the magnet; non-magnetic matter stayed.
    MagnetSeparated {
        from: VesselId,
        to: VesselId,
        attracted: Vec<SpeciesId>,
        remained: Vec<SpeciesId>,
    },
    /// Water left as vapour.
    Evaporated {
        vessel: VesselId,
        moles: Moles,
    },
    /// Volatile liquid boiled over into a receiver through a condenser.
    /// `at` and `ended` are the pot's boiling temperature at the start
    /// and end of the cut — the Rayleigh drift made visible; `energy_kj`
    /// is the latent heat the burner paid and the condenser dumped (the
    /// still is externally powered, and this is the bill); when
    /// `azeotropic`, the column ran into the azeotrope and further
    /// stages or boiling no longer enrich.
    Distilled {
        from: VesselId,
        to: VesselId,
        water: Moles,
        ethanol: Moles,
        at: Kelvin,
        #[serde(default = "kelvin_zero")]
        ended: Kelvin,
        #[serde(default = "one_stage")]
        stages: u32,
        #[serde(default)]
        energy_kj: f64,
        azeotropic: bool,
    },
    /// A neutral solute split between the two layers on its computed
    /// partition coefficient — the ratio of its infinite-dilution
    /// activity coefficients in the two solvents.
    Partitioned {
        vessel: VesselId,
        species: SpeciesId,
        /// Fraction of the solute that sat in the lower layer (and so
        /// left with it when the stopcock opened).
        fraction_lower: f64,
    },
    /// What a careful waft noticed: species and their curated odour
    /// words. Empty means "nothing your nose picks out", which for
    /// many gases is exactly the danger worth teaching.
    Smelled {
        vessel: VesselId,
        notes: Vec<(SpeciesId, String)>,
    },
    /// A classical gas test was applied to the vessel's headspace.
    GasTested {
        vessel: VesselId,
        test: crate::gas_tests::GasTest,
        positive: bool,
        notes: String,
    },
    /// A sealed vessel exceeded what glass can hold. The headspace let
    /// go: the seal is gone, the gases vented, and the safety line is
    /// not decorative. The GUI's explosion is THIS event, never a
    /// script.
    Burst {
        vessel: VesselId,
        /// Pressure at failure, Pa.
        at_pa: f64,
        /// The rating it exceeded, Pa.
        rating_pa: f64,
    },
    /// Heat of mixing crossed the observability line: composition
    /// change released (positive) or absorbed (negative) this much
    /// heat, from the state-function difference of UNIFAC-derived
    /// excess enthalpy over the verified-pair allowlist.
    HeatOfMixing {
        vessel: VesselId,
        joules: f64,
    },
    /// A radionuclide tracer entered the vessel's nuclide ledger.
    NuclideSpiked {
        vessel: VesselId,
        nuclide: String,
        moles: Moles,
        /// Initial activity, Bq — the number the Geiger will read.
        activity_bq: f64,
    },
    /// Radioactive decay ran during a wait: a parcel of the parent
    /// became the daughter. Elements do NOT conserve across this event
    /// — nucleons do, and the equation says exactly how.
    Decayed {
        vessel: VesselId,
        parent: String,
        daughter: String,
        mode: String,
        moles: Moles,
        half_life_s: f64,
        equation: String,
    },
    /// A named organic transformation ran to the stated extent. The
    /// boundary line carries what the model does NOT claim.
    OrgReacted {
        vessel: VesselId,
        name: String,
        equation: String,
        /// Reaction extent in moles of the equation as written.
        extent: Moles,
        boundary: String,
    },
    /// The column spoke: each dissolved neutral solute with a curated
    /// group decomposition eluted at the time its partition coefficient
    /// sets, and anything the method cannot see is named rather than
    /// silently dropped. K comes from the same UNIFAC γ∞ ratio the
    /// separating funnel runs on — water as the mobile phase, an alkane
    /// stationary phase — so the funnel and the column must agree about
    /// which solute is the hydrophobic one.
    Chromatographed {
        vessel: VesselId,
        /// Theoretical plates of the column that produced this table.
        plates: u32,
        /// Void time: when an unretained solute reaches the detector.
        void_time_s: f64,
        /// Peaks in elution order.
        peaks: Vec<ElutedPeak>,
        /// Dissolved species the method has no groups for — ions above
        /// all. Stated, because a chromatogram that quietly ignores half
        /// the sample teaches the wrong lesson about what a detector saw.
        outside_method: Vec<SpeciesId>,
    },
    /// The lower layer ran out through the stopcock, solutes and all.
    Drained {
        from: VesselId,
        to: VesselId,
        solvent: SpeciesId,
        moles: Moles,
    },
    /// Two liquid layers formed: mixing these liquids raises the Gibbs
    /// energy instead of lowering it, so they split — computed
    /// liquid–liquid equilibrium, not a solubility table. `upper`
    /// floats on `lower` by density.
    LayersFormed {
        vessel: VesselId,
        upper: SpeciesId,
        lower: SpeciesId,
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
        /// Heat released by the computed reaction at the ignition
        /// temperature, J. `None` means the engaged solver cannot quantify
        /// it; clients must use a restrained fallback rather than inventing
        /// a dramatic flame.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        energy_j: Option<f64>,
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
        /// Chemical enthalpy converted into sensible heat by this solve at
        /// its starting temperature, J. Present for exothermic CEA solves;
        /// absent where the solver cannot make that thermochemical claim.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reaction_energy_j: Option<f64>,
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
    /// Gas yield/rate from a kinetic interval, before any visual mapping.
    GasProduced {
        vessel: VesselId,
        reaction: String,
        species: SpeciesId,
        moles: Moles,
        rate_moles_per_second: f64,
    },
    /// Exothermic energy released by a curated kinetic reaction.
    ReactionHeatReleased {
        vessel: VesselId,
        reaction: String,
        energy_j: f64,
    },
    /// A surfactant recipe temporarily trapped produced gas as foam.
    FoamChanged {
        vessel: VesselId,
        trapped_gas_liters: f64,
        volume_liters: f64,
        height_cm: f64,
        overflow_liters: f64,
        half_life_seconds: f64,
    },
    /// A solver was asked and could not converge / answer. First-class,
    /// honest, never a crash.
    SolverFailed {
        vessel: VesselId,
        solver: String,
        detail: String,
    },
    /// Solvent (water) was added by volume to dilute the contents.
    Diluted {
        vessel: VesselId,
        volume: Liters,
        moles: Moles,
    },
    /// An auto-stepped titration ran to completion (or exhausted its step
    /// budget). The curve carries (cumulative mL, pH) at every step.
    Titrated {
        vessel: VesselId,
        titrant: SpeciesId,
        /// Standard-solution concentration in the burette, mol/L.
        #[serde(default = "one_molar")]
        concentration: f64,
        steps: u32,
        total_volume: Liters,
        final_ph: f64,
        curve: Vec<(f64, f64)>,
    },
    /// Fractions of two solutions were mixed into a third vessel.
    Mixed {
        a: VesselId,
        b: VesselId,
        into: VesselId,
        fraction_a: f64,
        fraction_b: f64,
        /// Temperatures used by the adiabatic balance. Keeping these on the
        /// event lets clients assess and explain the computed outcome without
        /// reconstructing pre-step vessel state.
        temperature_a: Kelvin,
        temperature_b: Kelvin,
        temperature_into: Kelvin,
    },
    /// Liquid flowed through a 1-D column of vessels. The effluent —
    /// what came out the far end — was deposited into the receiver.
    Transported {
        chain: Vec<VesselId>,
        receiver: VesselId,
        steps: u32,
        courant: f64,
        effluent_moles: Vec<(SpeciesId, Moles)>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialComponentAdded {
    pub species: SpeciesId,
    pub basis_amount: f64,
    pub moles: Moles,
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
