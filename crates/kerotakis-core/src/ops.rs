//! Operators: everything a person can do to the bench, and everything the
//! bench can report back. The operator log is the save file and the API
//! contract (PLAN.md).

use serde::{Deserialize, Serialize};

use crate::authority::{ReplaySeed, SpillDestination};
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

/// Which side of a threshold a titration is waiting to land on.
///
/// A pH endpoint is a *crossing* — the curve arrives from one side and
/// leaves on the other, so the direction is discovered rather than
/// declared. A redox endpoint is not: past equivalence the potential
/// keeps climbing, and the practical says "titrate until the potential
/// passes X", which is an inequality and has to be written as one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Compare {
    Above,
    AtLeast,
    Below,
    AtMost,
}

impl Compare {
    pub fn holds(self, value: f64, threshold: f64) -> bool {
        match self {
            Compare::Above => value > threshold,
            Compare::AtLeast => value >= threshold,
            Compare::Below => value < threshold,
            Compare::AtMost => value <= threshold,
        }
    }

    /// The token that spells it, for narration and round-tripping.
    pub fn symbol(self) -> &'static str {
        match self {
            Compare::Above => ">",
            Compare::AtLeast => ">=",
            Compare::Below => "<",
            Compare::AtMost => "<=",
        }
    }
}

/// EXP-39: how a titration knows it has arrived.
///
/// CAP-12 could only chase a pH, which is the endpoint of exactly one
/// family of titrations. A redox titration has two of its own, and both
/// are read off state the engine already computes rather than off a new
/// solver: the potentiometric endpoint is the aqueous engine's own pe,
/// and the self-indicating endpoint is the computed colour of the
/// liquid — permanganate is its own indicator because ε(λ) says it is
/// visible at 10⁻⁵ mol/L, not because a constant somewhere says so.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Endpoint {
    /// The pH crosses `Operator::Titrate::target_ph`. CAP-12's endpoint,
    /// and still the default.
    #[default]
    Ph,
    /// Potentiometric: the solver's own pe passes `value`. A step where
    /// the engine withholds pe — at equivalence the electron balance has
    /// no root and printing a number there would republish a bracket
    /// ceiling as a measurement — never satisfies the comparison.
    Pe { compare: Compare, value: f64 },
    /// Self-indicating: the titrant's own colour survives in the flask.
    /// The flask's computed colour word is read before the first drop
    /// and after every one; the endpoint is the first increment whose
    /// colour differs from that baseline. Everything is equilibrated, so
    /// a colour that appears here is by construction a colour that
    /// stays — which is exactly what "persists" means at the bench.
    ColourPersists,
}

impl Endpoint {
    /// Whether this is the legacy pH endpoint, and so may be omitted
    /// from the wire entirely.
    pub fn is_ph(&self) -> bool {
        matches!(self, Endpoint::Ph)
    }
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
    /// BRD-002: put a finite amount of one shelf entry in its bottle.
    ///
    /// An operator rather than a bench setter, because the shelf level
    /// decides whether a later `Add` succeeds — and anything a replay's
    /// outcome depends on has to be in the log, or the replay is not one.
    /// A key that is never stocked stays an unlimited supply, which is the
    /// sandbox every script written before this assumed.
    StockShelf { key: String, amount: f64 },
    /// Put energy into a vessel (burner, heating mantle). Negative energy is
    /// expressed with `Cool`.
    ///
    /// `source` names the thing doing the heating, and above all how hot
    /// that thing is. Absent means the bench default, a laboratory burner
    /// with the collar open — so every script written before sources
    /// existed replays unchanged, and `heat v1 40kJ` still parses to this
    /// operator with `source: None`.
    Heat {
        vessel: VesselId,
        energy: Joules,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        source: Option<crate::apparatus::HeatSource>,
    },
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
    /// Pour a liquid fraction into an explicit material-holding surface.
    Spill {
        from: VesselId,
        destination: SpillDestination,
        fraction: f64,
        replay_seed: ReplaySeed,
    },
    /// Submit collision evidence. Breakage is decided by core from vessel kind
    /// and impulse; the renderer cannot directly mutate or destroy a vessel.
    Impact {
        vessel: VesselId,
        impulse_ns: f64,
        destination_if_broken: SpillDestination,
        replay_seed: ReplaySeed,
    },
    /// Recover a fraction of a spill into an intact receiver.
    RecoverSpill {
        destination: SpillDestination,
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
    /// Spin one balanced tube in a mini centrifuge.
    Centrifuge {
        vessel: VesselId,
        rpm: f64,
        seconds: f64,
        rotor_radius_m: f64,
        /// Opposing tube contents in grams. `None` preserves the historical
        /// shorthand and means an exactly matched balance tube.
        #[serde(default)]
        counterbalance_g: Option<f64>,
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
    /// Draw the vessel's particle census — Johnstone's submicroscopic
    /// vertex, at whatever scale the contents earn.
    ///
    /// An operator rather than a session command, which is the whole point
    /// of the change that added it: `particles` answered "what dissolved
    /// ions are present?" perfectly in the REPL and no SCRIPT could ask,
    /// because the corpus lint refuses a session command ("script line 3 is
    /// a session command, not an operator"). The engine could answer and the
    /// script surface could not. It reads state and changes none, like
    /// `Smell`.
    Particles { vessel: VesselId },
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
    /// `step` volume, re-equilibrating after each addition, until the
    /// `endpoint` is reached or `max_steps` additions are exhausted.
    /// Records (cumulative volume, pH) at every step, and (cumulative
    /// volume, pe) wherever the solver pinned a potential.
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
        /// The pH the burette is chasing. Read only by `Endpoint::Ph`,
        /// which is the default and was CAP-12's only endpoint; the
        /// EXP-39 redox endpoints carry their own target inside
        /// `endpoint` and leave this field at the neutral 7 they never
        /// consult. It keeps its name and its meaning so that every
        /// script, log and protocol payload written before EXP-39
        /// deserialises unchanged.
        target_ph: f64,
        max_steps: u32,
        /// EXP-39: what stops the burette. Absent from every payload
        /// written before EXP-39, and absent means the pH target above.
        #[serde(default, skip_serializing_if = "Endpoint::is_ph")]
        endpoint: Endpoint,
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
    /// EXP-33: Melting-point apparatus — a capillary of dry solid in a
    /// heated block. Answers with the curated transition temperature for a
    /// pure sample and with the reason for a refusal otherwise.
    MeltingPointApparatus,
    /// EXP-33: the same instrument set up for a liquid: a distillation head
    /// with the bulb in the vapour.
    BoilingPointApparatus,
    /// KID-19a: a density reader. A hydrometer floats in a liquid and
    /// reads its density; for a single pure solid the same instrument is a
    /// balance and a measuring cylinder, and the answer is the substance's
    /// own density. Density is the property that tells copper from zinc
    /// from aluminium when a balance cannot: five grams of each weighs
    /// five grams.
    Densitometer,
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
    /// KID-9: the same K read as a paper strip reads it — the fraction of
    /// the solvent front's distance this solute travels. A column and a
    /// paper plate are one separation reported two ways, so both numbers
    /// come from one coefficient and cannot disagree.
    #[serde(default)]
    pub rf: f64,
}

/// One solid population's computed travel during a centrifuge run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CentrifugeSeparation {
    pub species: SpeciesId,
    pub particle_diameter_um: f64,
    pub particle_size_assumed: bool,
    pub particle_density_kg_m3: f64,
    pub terminal_speed_m_s: f64,
    pub distance_m: f64,
    pub separated_fraction: f64,
    pub direction: crate::centrifuge::SeparationDirection,
}

/// What one step produced. Everything user-visible derives from this.
/// Why a refusal is a refusal — the kind of gap, beside the sentence.
///
/// `NotYetModeled` carried its subject as prose and nothing else. That reads
/// well and groups not at all: three refusals mentioning water can be three
/// different problems, and the only way to ask "are these the same gap" was
/// to match sentences, which is the defect this programme has spent its time
/// removing everywhere else. The prose stays exactly as it was — it is what a
/// learner reads — and this sits next to it for everything that is not a
/// learner.
///
/// # This is for grouping and diagnosis. It is NOT for scoring.
///
/// Do not key a coverage rule, a disposition or any other verdict on it.
///
/// The reason is specific rather than stylistic. PR #362 tried to decide
/// whether a corpus row had been answered by looking at which event KINDS a
/// step emitted. It moved `mat-096`, which really does answer its question,
/// and `mat-099`, which does not — the iron rusts anyway and the zinc does
/// nothing, so the row demonstrates the opposite of "galvanizing protects
/// iron". Both rows emit the same events. `mat-003` and `mat-006` emit
/// BYTE-IDENTICAL event streams and differ only in what the prompt asks.
///
/// It was closed unmerged, and the sentence worth carrying forward is this:
/// the change shrank the missing column and therefore looked like progress.
/// A cause id makes exactly that mistake easier to make and harder to see,
/// because it looks principled. Deciding whether a question was answered
/// needs the QUESTION, which lives in the prompt and not in the engine.
///
/// If you are reading this because you want to key something on it: the
/// thing you actually want is for the prompt to say what it is asking about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NotModelledCause {
    /// No aqueous solution has been characterised, so an instrument that
    /// reads one has nothing to read.
    NoSolution,
    /// The operation is modelled; the vessel simply does not hold what it
    /// needs. Nothing to evaporate, no substrate for the verb, an empty
    /// beaker under the hydrometer. Not a gap in the lab at all, which is
    /// exactly why it is worth being able to tell apart from one.
    NothingToActOn,
    /// No wired solver covers this state at all.
    NoSolver,
    /// The chemistry is right and the SPEED of it is not modelled.
    RateNotModelled,
    /// A model was asked outside the range it is parameterised for.
    ModelBoundary,
    /// The registry carries no reviewed value for a quantity this needs.
    NoReviewedDatum,
    /// A phase or oxidation state the databases know about is absent from
    /// this lab's registry, so nothing can form, dissolve or be named as it.
    PhaseNotInRegistry,
    /// The substance has no aqueous role that any shipped database can
    /// speciate.
    NotSpeciated,
    /// The matter exists and there is no modelled route between where it is
    /// and where the question looks — dissolved gas and a headspace, say.
    NoTransportPath,
    /// The vessel's boundary cannot do what the operation needs.
    BoundaryMismatch,
    /// A curated table has no row for this, and the general case is not
    /// derivable.
    NotParameterised,
    /// No shipped thermodynamic database defines the species at all, so no
    /// registry entry and no wiring can rescue it.
    ///
    /// Distinct from `PhaseNotInRegistry`, and the distinction is the whole
    /// value of the row: that one is in this lab's gift and this one is
    /// not. Hypochlorite is the example — searched by name across every
    /// `.dat` vendored with iphreeqc, and not one defines it — as is
    /// malate, and sodium acetate as a solid. A reader who groups by cause
    /// can tell "we have not got to it" from "nobody has".
    NotInAnyDatabase,
    /// Recorded before this field existed.
    ///
    /// Exists ONLY so that `serde(default)` can load an operator log or a
    /// golden fixture written earlier. New code must never construct it:
    /// if you find yourself reaching for it, the honest move is a new
    /// variant naming the actual cause. It is deliberately not a general
    /// `Other`, because an `Other` becomes the drawer everything is swept
    /// into and the grouping stops meaning anything. (The trap named by
    /// kerotakis-5f, who hit the serialisation half of it on `ElutedPeak`
    /// the same week.)
    #[default]
    Unclassified,
}

impl NotModelledCause {
    /// A stable kebab-case id, following the safety rule ids. Stable across
    /// releases: it is what a reader groups by.
    pub fn id(self) -> &'static str {
        match self {
            Self::NoSolution => "no-solution",
            Self::NothingToActOn => "nothing-to-act-on",
            Self::NoSolver => "no-solver",
            Self::RateNotModelled => "rate-not-modelled",
            Self::ModelBoundary => "model-boundary",
            Self::NoReviewedDatum => "no-reviewed-datum",
            Self::PhaseNotInRegistry => "phase-not-in-registry",
            Self::NotSpeciated => "not-speciated",
            Self::NoTransportPath => "no-transport-path",
            Self::BoundaryMismatch => "boundary-mismatch",
            Self::NotParameterised => "not-parameterised",
            Self::NotInAnyDatabase => "not-in-any-database",
            Self::Unclassified => "unclassified",
        }
    }
}

/// What a plastic is doing at the temperature it is being held at.
///
/// Three states and no fourth: a polymer object is rigid, or it has gone
/// soft enough to reshape, or it has decomposed. The bench claims nothing
/// between them — no viscosity, no rate of flow, no degree of cure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PolymerState {
    /// Below everything: it holds its shape.
    Rigid,
    /// Above the softening point and below decomposition: the chains
    /// slide, so it can be moulded, and cooling sets it in the new shape.
    Softened,
    /// Past the decomposition temperature. This one does not undo.
    Charred,
}

impl PolymerState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Rigid => "rigid",
            Self::Softened => "softened",
            Self::Charred => "charred",
        }
    }
}

/// A step that never needed to chunk its delivery took one pass. Logs
/// written before chunked heating existed carry no `passes` field at all,
/// and one is what they meant.
fn one_pass() -> u32 {
    1
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum Event {
    VesselCreated {
        vessel: VesselId,
    },
    VesselRemoved {
        vessel: VesselId,
    },
    SpillCreated {
        destination: SpillDestination,
        source: VesselId,
        fraction: f64,
        replay_seed: ReplaySeed,
    },
    ContainerBroken {
        vessel: VesselId,
        destination: SpillDestination,
        impulse_ns: f64,
        replay_seed: ReplaySeed,
    },
    CollisionWithstood {
        vessel: VesselId,
        impulse_ns: f64,
        replay_seed: ReplaySeed,
    },
    SpillRecovered {
        destination: SpillDestination,
        to: VesselId,
        fraction: f64,
    },
    SpillHazard {
        destination: SpillDestination,
        severity: crate::solve::Severity,
        /// Stable rule id, exactly as on `HazardWarning` (additive; empty
        /// when the warning has no curated matrix rule, e.g. a veto).
        #[serde(default)]
        rule: String,
        hazard: String,
        real_world: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        contributors: Vec<SpeciesId>,
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
    ObjectSpillBoundary {
        vessel: VesselId,
        object_count: usize,
    },
    OsmosisChanged {
        vessel: VesselId,
        material: String,
        water_moles: f64,
        mass_change_g: f64,
    },
    BrowningChanged {
        vessel: VesselId,
        material: String,
        browned_fraction: f64,
    },
    SoapScumFormed {
        vessel: VesselId,
        aggregate_mass_g: f64,
        divalent_ion_moles: f64,
    },
    LemonPaperMarked {
        vessel: VesselId,
        lemon_amount_g: f64,
        paper_amount_g: f64,
    },
    LemonPaperDried {
        vessel: VesselId,
    },
    LemonPaperBrowned {
        vessel: VesselId,
        browned_fraction: f64,
        temperature_k: f64,
    },
    TemperatureChanged {
        vessel: VesselId,
        from: Kelvin,
        to: Kelvin,
    },
    /// Heat actually accepted by or removed from a vessel. The core
    /// currently applies energy instantaneously; no power or elapsed-time
    /// claim is made here.
    ///
    /// `requested_j` minus `delivered_j` is the part of the dose that had
    /// nowhere to go. On heating that is the honest consequence of a
    /// source having a temperature of its own: once the vessel is as hot
    /// as the flame, the only route left for more energy is chemistry that
    /// consumes heat at that temperature, and when that is exhausted the
    /// remainder is simply not delivered.
    EnergyTransferred {
        vessel: VesselId,
        heating: bool,
        requested_j: f64,
        delivered_j: f64,
        time_coupled: bool,
        /// What was doing the heating, and how hot it is. Absent on
        /// cooling, which has no modelled coolant, and on logs written
        /// before heat sources existed.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        source: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ceiling_k: Option<f64>,
        /// Of the delivered energy, how much is warmth the vessel still
        /// holds: Cp·ΔT over the step, measured on the contents as they
        /// ended. The rest went into chemistry, into a phase change, or
        /// left with a gas.
        #[serde(default)]
        sensible_j: f64,
        /// How many delivery passes the step needed. One means the dose
        /// fitted below the ceiling and nothing had to be chunked.
        #[serde(default = "one_pass")]
        passes: u32,
        /// Whether delivery stopped at the pass cap rather than because
        /// the dose ran out or the vessel reached the source.
        #[serde(default)]
        capped: bool,
    },
    /// Mechanical mixing conditions actually delivered by a magnetic
    /// stirrer. Tip speed follows π·bar_length·rpm/60.
    Stirred {
        vessel: VesselId,
        rpm: f64,
        seconds: f64,
        bar_length_m: f64,
        tip_speed_m_s: f64,
        /// Fraction of an available non-metal deposit lifted into suspension,
        /// from accumulated bar travel over a 0.30 m mixing-length scale.
        resuspended_fraction: f64,
        /// Whether rpm/tip speed alter a kinetic rate through mass transfer.
        /// Timed stirring advances the vessel clock independently of this.
        rate_coupled: bool,
    },
    /// A stirred, recipe-declared surfactant changed how much unresolved oil
    /// is temporarily dispersed through the aqueous phase.
    EmulsionChanged {
        vessel: VesselId,
        material: String,
        from_dispersed_fraction: f64,
        to_dispersed_fraction: f64,
        dispersed_volume_l: f64,
        half_life_seconds: f64,
    },
    /// Acetic-acid dose caused a recipe-declared milk colloid to separate
    /// into visible curds and whey. The aggregate mass remains conserved.
    CurdlingChanged {
        vessel: VesselId,
        material: String,
        from_formed_fraction: f64,
        to_formed_fraction: f64,
        separation_progress: f64,
        curd_solids_mass_g: f64,
        acid_species: SpeciesId,
        acid_moles: Moles,
    },
    /// KID-13: a suspension dense enough to argue back.
    ///
    /// Nothing reacts and no mole moves: this reports how the mixture
    /// *responds* to being pushed. Above a packing fraction the particles
    /// cannot slide past one another fast enough, so the harder it is
    /// stirred the more it resists — and it flows again the moment the
    /// stirring stops.
    Thickened {
        vessel: VesselId,
        solid: SpeciesId,
        /// 0 at the onset mixture, 1 at the full one.
        strength: f64,
        solid_mass_fraction: f64,
        tip_speed_m_s: f64,
        /// Sheared hard enough to thicken, rather than merely stirred.
        sheared_hard: bool,
    },
    /// KID-14: the glue stopped being a liquid.
    ///
    /// Nothing is consumed and no matter is created: borate bridges between
    /// polymer chains keep breaking and re-forming, which is why slime
    /// flows slowly and tears quickly. The fraction is how much of the
    /// polymer is bound into the network.
    GelFormed {
        vessel: VesselId,
        polymer: SpeciesId,
        crosslinker: SpeciesId,
        from_gelled_fraction: f64,
        to_gelled_fraction: f64,
        polymer_grams: f64,
        crosslinker_moles: Moles,
    },
    /// A declared superabsorbent network retained water without consuming it.
    PolymerSwelled {
        vessel: VesselId,
        dry_polymer_g: f64,
        retained_water_g: f64,
        swelling_ratio_g_per_g: f64,
        capacity_g_per_g: f64,
        saturated: bool,
    },
    /// Light predicted for the declared luminol/peroxide teaching system.
    /// This is a relative observable, not photon-counting or a glow-stick
    /// formulation claim.
    ChemiluminescenceObserved {
        vessel: VesselId,
        relative_intensity: f64,
        half_life_s: f64,
        elapsed_s: f64,
        temperature: Kelvin,
        oxidant_moles: Moles,
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
    /// A balanced mini centrifuge run, with motion and separation computed
    /// from rotor and material properties rather than a canned animation.
    Centrifuged {
        vessel: VesselId,
        rpm: f64,
        seconds: f64,
        rotor_radius_m: f64,
        rcf: f64,
        sample_mass_g: f64,
        counterbalance_g: f64,
        imbalance_g: f64,
        fluid_density_kg_m3: f64,
        dynamic_viscosity_pa_s: f64,
        separations: Vec<CentrifugeSeparation>,
        /// False until vessel suspension/deposit state consumes the result.
        state_coupled: bool,
    },
    /// Light physically delivered by the lamp. `photolysis_coupled` is an
    /// explicit model boundary: the lamp may run without claiming that the
    /// chemical state changed.
    Irradiated {
        vessel: VesselId,
        wavelength_nm: f64,
        irradiance_w_m2: f64,
        photolysis_coupled: bool,
    },
    /// BRD-014.S05: a named material stood in ultraviolet light and let
    /// only a fraction through — the label's factor for that band, at the
    /// standard film, with the mechanism in words.
    UvAttenuated {
        vessel: VesselId,
        /// The recipe's display name.
        material: String,
        wavelength_nm: f64,
        /// "UV-B" or "UV-A".
        band: String,
        /// Fraction of the incident light transmitted, 0–1.
        transmitted_fraction: f64,
        mechanism: String,
    },
    /// EXP-25: a dissolved volatile moved between the liquid and an owned
    /// headspace to its Henry's-law equilibrium (`volatility.rs`). A
    /// phase distribution, not a reaction: the amount is booked, the heat
    /// deliberately is not (see the module).
    HeadspacePartitioned {
        vessel: VesselId,
        species: SpeciesId,
        /// `true`: liquid → headspace; `false`: headspace → liquid.
        to_gas: bool,
        moles: Moles,
        /// Share of the species' whole inventory now in the headspace, 0–1.
        gas_fraction: f64,
        /// Equilibrium partial pressure, Pa.
        partial_pressure_pa: f64,
        /// Henry's constant at the vessel temperature, mol/(L·atm).
        henry_mol_per_l_atm: f64,
        /// Provenance of the coefficient.
        source: String,
    },
    /// Tracked particles settled under ordinary gravity while bench time
    /// advanced, using the same Stokes model as the centrifuge at 1 g.
    GravitySettled {
        vessel: VesselId,
        seconds: f64,
        separations: Vec<CentrifugeSeparation>,
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
        /// The ion this metal has just finished displacing, where THAT is
        /// the reason nothing more is happening.
        ///
        /// The activity series writes an `Inert` for two opposite
        /// situations and the lv1 sentence has to tell them apart. Silver
        /// in copper sulfate does nothing because the couple runs uphill:
        /// nothing here can take its electrons, and "it does not swap
        /// places with anything dissolved here" is the whole answer. Iron
        /// left over in a beaker whose copper has all plated out is the
        /// mirror image — the couple ran downhill and has finished — and
        /// that sentence read as a claim that iron cannot displace copper,
        /// two lines under the event saying it just had.
        ///
        /// `Some(ion)` names the ion that has been used up. Defaulted so a
        /// log written before the field existed still reads, and skipped
        /// when absent so the uphill case serialises exactly as before.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        spent: Option<SpeciesId>,
    },
    /// BRD-023: the corrosion route's verdict on one metal in one vessel.
    ///
    /// A separate event from `Inert` because the two say opposite things
    /// about the same nail. `Inert` is "nothing is happening to this
    /// solid"; this is "here is what the corrosion cell in this beaker is
    /// doing, and whether this metal is the one paying for it". The
    /// negative form — a metal that is NOT corroding, and which of the
    /// three requirements is missing, or what is protecting it — is as
    /// much a computed result as the positive one, which is why it is the
    /// same event with `corroding: false` rather than a stand-aside.
    ///
    /// It carries no rate, on purpose. The corrosion reactions live in
    /// `kinetics::REGISTRY`, where `wait` drives them and `Reacted`
    /// reports what moved; this event is the verdict that decides which
    /// of them may run at all, and a second number here would be a second
    /// opinion about the same nail.
    Corroded {
        vessel: VesselId,
        species: SpeciesId,
        corroding: bool,
        why: String,
    },
    /// BRD-032: a sorbent solid is holding some of a dissolved species on
    /// its surface, and the beaker's own solution is what is left.
    ///
    /// It carries both halves on purpose. "The charcoal adsorbed the dye"
    /// is the sentence a demonstration wants and is the one that misleads:
    /// a gram of carbon has a finite capacity, and whether it took most of
    /// the dye or a tenth of it is the whole answer to "can charcoal
    /// remove a food dye from water". So the loading and the remainder are
    /// reported together, and neither can be read without the other.
    Adsorbed {
        vessel: VesselId,
        sorbate: SpeciesId,
        sorbent: SpeciesId,
        /// Moles now held on the surface.
        held: Moles,
        /// Milligrams of sorbate per gram of sorbent — the number an
        /// isotherm is quoted in, and the one that says whether the
        /// carbon is saturated.
        loading_mg_per_g: f64,
        /// Moles still in solution, which is what a filtration pours.
        still_dissolved: Moles,
        /// What the curated isotherm does NOT claim.
        boundary: String,
    },
    /// BRD-041: a fuel stands with oxygen, warm, but below its
    /// autoignition temperature and without a spark — so nothing burns,
    /// and that is an answer. The thermal solver says this instead of
    /// equilibrating a mixture that, on a real bench, would sit there.
    BelowAutoignition {
        vessel: VesselId,
        fuel: SpeciesId,
        /// The fuel's autoignition temperature in air.
        autoignition: Kelvin,
        /// Where the vessel actually stands.
        temperature: Kelvin,
    },
    /// BRD-014: what is inside a sealed cell, and what moves while it
    /// discharges. The bench does not open the case — the object is
    /// coherent and its mass is conserved by construction — so this is
    /// the curated sentence that says what the mass is doing, beside a
    /// balance reading that is the evidence for the sealing.
    SealedCell {
        vessel: VesselId,
        /// The recipe's display name.
        material: String,
        /// Nominal open-circuit voltage, V.
        open_circuit_volts: f64,
        /// The balanced discharge reaction, as written. It is NOT run.
        reaction: String,
        why: String,
    },
    /// BRD-023: what heat has done to a named plastic, and which of the
    /// two families it belongs to. One event with three states rather
    /// than three events, because the states are exclusive readings of
    /// one thermometer and the interesting thing is which one holds.
    PolymerHeated {
        vessel: VesselId,
        /// The recipe's display name.
        material: String,
        state: PolymerState,
        /// Where the vessel stands.
        temperature: Kelvin,
        /// The threshold the state turns on: the softening point for a
        /// thermoplastic below or above it, the decomposition temperature
        /// once that is the nearer wall.
        threshold: Kelvin,
        /// Whether the object comes back when it cools. Softening does;
        /// charring does not; and staying rigid has nothing to undo.
        reversible: bool,
        /// True for a cross-linked network, which has no melt to reach.
        cross_linked: bool,
    },
    /// A solid formed out of solution (computed by an aqueous solver).
    Precipitated {
        vessel: VesselId,
        species: SpeciesId,
        moles: Moles,
    },
    /// KID-7: more is dissolved than the water can hold, and nothing has
    /// come out.
    ///
    /// Not a failure and not a rounding artefact — it is the state a cooled
    /// sugar syrup is genuinely in, and the reason rock candy needs a
    /// string to grow on. Precipitating it automatically would erase the
    /// experiment; saying nothing would report a solution the water cannot
    /// actually hold as though it were ordinary. So it is a result.
    Supersaturated {
        vessel: VesselId,
        species: SpeciesId,
        /// What is in solution now.
        dissolved: Moles,
        /// What this much water holds at this temperature.
        capacity: Moles,
    },
    /// Acid met base: this much of the solutes' unspent acidity cancelled.
    ///
    /// `H⁺ + OH⁻ → H₂O` is never a reaction PHREEQC reports, because it is
    /// handed element totals and cannot tell an acid just added from one
    /// that was always there. The extent is recoverable from the solutes'
    /// net charge, and the aqueous solver has been computing it for a
    /// while — to get the heat of neutralisation right — and then throwing
    /// the number away. It is the ledger's business: it is the commonest
    /// reaction in a school lab and it was the only one that happened
    /// without an entry against it.
    Neutralised {
        vessel: VesselId,
        /// Moles of water formed, i.e. moles of acidity cancelled.
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
        /// Applied operating point, retained for physical playback.
        amps: f64,
        seconds: f64,
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
    /// A zinc/acid/copper teaching cell whose voltage is explicitly a bounded
    /// unit-activity estimate, not a fully specified two-ion half-cell.
    AcidMetalCellVoltage {
        anode: VesselId,
        cathode: VesselId,
        volts: f64,
        ph: f64,
    },
    /// Two vessels were wired and no cell exists between them, with the
    /// reason. A computed answer about the beakers, not a gap in the lab.
    NoCell {
        a: VesselId,
        b: VesselId,
        why: String,
    },
    /// EXP-33: the melting-point block's answer, refusals included.
    ///
    /// Carries its own citation because the number is curated data rather
    /// than a solved quantity, and a transition temperature printed without
    /// the book behind it is a claim the learner cannot check.
    TransitionPointRead {
        vessel: VesselId,
        reading: crate::instrument::TransitionReading,
    },
    /// EXP-33: a hydrate gave up its water of crystallisation to heat.
    ///
    /// Every field of the ledger travels: how many formula units broke, how
    /// much water left, and at what temperature — because the crucible
    /// lesson is the arithmetic, not the colour change.
    Dehydrated {
        vessel: VesselId,
        hydrate: SpeciesId,
        anhydrous: SpeciesId,
        formula_units: Moles,
        water: Moles,
        at: Kelvin,
    },
    /// EXP-33: the water went back in and the colour came back with it.
    Hydrated {
        vessel: VesselId,
        anhydrous: SpeciesId,
        hydrate: SpeciesId,
        formula_units: Moles,
        water: Moles,
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
        /// Stable machine identity of the hazard rule (additive; empty on
        /// events from older snapshots or producers without a curated
        /// rule). `hazard`/`real_world` are localized prose — consumers
        /// that must recognise WHICH hazard fired key on this.
        #[serde(default)]
        rule: String,
        hazard: String,
        real_world: String,
    },
    /// L0 refused the operation (product-safety boundary). Nothing was
    /// mutated.
    SafetyVeto {
        reason: String,
    },
    /// BRD-002: a bottle was filled to a finite level. From here on, a
    /// dispense of this key is a withdrawal.
    ShelfStocked {
        key: String,
        amount: f64,
        unit: crate::stock::StockUnit,
    },
    /// BRD-002: the shelf refused a dispense because the bottle does not
    /// hold that much. Nothing was mutated — not the vessel, and not the
    /// bottle, which still holds exactly `remaining`.
    StockExhausted {
        key: String,
        requested: f64,
        remaining: f64,
        unit: crate::stock::StockUnit,
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
    /// The particle census of one vessel, as drawn.
    ///
    /// Carries the whole `Census` rather than a rendered string: the
    /// picture is the same claim at every register, and a host that wants
    /// to draw it rather than print it needs the populations. `Source`
    /// travels with it because a picture from solved speciation is a
    /// different claim from one off the inventory, and the viewer is
    /// entitled to know which they are looking at.
    ParticlesCounted {
        vessel: VesselId,
        census: crate::particles::Census,
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
    /// A reviewed household mixture forms a visible upper layer on water.
    /// Unlike `LayersFormed`, this does not claim molecular identity or a
    /// computed full-composition liquid-liquid equilibrium.
    MaterialLayersFormed {
        vessel: VesselId,
        upper_material: String,
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
    /// KID-13: gas is coming out of the liquid, and there is something in
    /// there dense enough to sink and rough enough for bubbles to cling
    /// to. This is the dancing raisin, and the number is the point: the
    /// attached gas has to be worth `lift_gas_fraction` of the object's
    /// own volume before it goes up.
    BubbleRide {
        vessel: VesselId,
        /// Display name of the material riding the bubbles.
        object: String,
        object_density_g_per_ml: f64,
        liquid_density_g_per_ml: f64,
        /// Attached gas volume as a fraction of the object's own volume,
        /// needed to lift it. Zero means it floats unaided.
        lift_gas_fraction: f64,
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
    /// KID-12: the flame went out because of the air, not the fuel.
    ///
    /// A candle under a jar stops burning while roughly four fifths of
    /// the jar's oxygen is still in it: a flame needs an oxygen
    /// *fraction*, not merely some oxygen. `oxygen_fraction` is what the
    /// surrounding gas had fallen to when the flame quit, so the
    /// learner can be shown the number that contradicts "it used up all
    /// the oxygen".
    FlameStarved {
        vessel: VesselId,
        /// The fuel that was still there and could not go on burning.
        fuel: SpeciesId,
        /// How much of it burned before the air gave out. Zero means the
        /// flame never caught at all — air already too thin to light in,
        /// which is what a carbon-dioxide extinguisher makes.
        burned: Moles,
        oxygen_fraction: f64,
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
        /// What KIND of gap this is, beside the sentence rather than
        /// instead of it. See `NotModelledCause`.
        ///
        /// `serde(default)` because `Event` is serialised: saved operator
        /// logs and the golden lesson fixtures carry refusals written
        /// before this field existed, and a required field would refuse to
        /// load them. The default is `Unclassified`, which means exactly
        /// "written before there was a cause" and nothing else.
        #[serde(default)]
        cause: NotModelledCause,
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
    /// BRD-032: which model set the boiling temperature, and at what
    /// pressure.
    ///
    /// Emitted only where the vessel is **not** at one atmosphere, so an
    /// open beaker's event stream is exactly what it was. It is also
    /// emitted when the pressure route *declined* — that is the point.
    /// A bench that quietly kept the 1 atm answer under a pressure cooker
    /// lid would be indistinguishable from one that had modelled it, and
    /// BRD-032's whole condition is that a fall-through be named.
    BoilingPointRouted {
        vessel: VesselId,
        species: SpeciesId,
        /// The vessel's own pressure, kPa.
        pressure_kpa: f64,
        /// The boiling temperature actually used, including any colligative
        /// elevation.
        boiling: Kelvin,
        /// How much of that came from the pressure alone, K.
        shifted_by: f64,
        /// Which model answered, or why none did.
        route: crate::states::BoilingRoute,
        /// The model's own name, from its pack row.
        model: String,
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
    /// Baker's yeast converted finite dissolved sucrose into ethanol and CO2
    /// during one timed interval using a bounded recipe-level rate response.
    Fermented {
        vessel: VesselId,
        sucrose_moles: Moles,
        ethanol_moles: Moles,
        carbon_dioxide_moles: Moles,
        active_yeast_grams: f64,
        seconds: f64,
    },
    /// A catalyst hydrolysed a bounded substrate fraction inside conserved,
    /// unresolved food material; no named product ledger is implied.
    EnzymeHydrolysed {
        vessel: VesselId,
        family: crate::enzyme::EnzymeFamily,
        material: String,
        substrate: String,
        hydrolysed_mass_g: f64,
        converted_fraction: f64,
        seconds: f64,
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
    /// A recipe-declared surfactant dose pushed an existing floating powder
    /// layer away from the centre of a quiet liquid surface.
    SurfaceSpread {
        vessel: VesselId,
        material: String,
        from_cleared_fraction: f64,
        to_cleared_fraction: f64,
        coverage_fraction: f64,
    },
    /// Detergent spread resolved colourant drops across an opaque colloid's
    /// surface. The fractions are bounded visual geometry, not a CFD field.
    SurfaceColourSpread {
        vessel: VesselId,
        from_spread_fraction: f64,
        to_spread_fraction: f64,
        spot_count: usize,
    },
    /// Mechanical stirring released localized surface dye into the normal
    /// homogeneous Beer–Lambert colour calculation.
    SurfaceColourMixed {
        vessel: VesselId,
        spot_count: usize,
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
        /// EXP-39: the redox half of the same titration — (cumulative mL,
        /// pe) at every step where the aqueous engine *pinned* a
        /// potential.
        ///
        /// Deliberately sparse rather than nullable: a point is missing
        /// exactly where pe is undefined, and at the equivalence point of
        /// a redox titration pe genuinely is undefined — both members of
        /// the couple are spent, the electron balance has no root, and
        /// the engine withholds the number rather than publish the top of
        /// its own search bracket as a measurement. The gap in this curve
        /// is therefore not missing data; it is where the endpoint is.
        ///
        /// Empty for a pH titration of a beaker with no redox chemistry,
        /// which is every payload written before EXP-39.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pe_curve: Vec<(f64, f64)>,
        /// EXP-39: whether the endpoint was actually reached, as opposed
        /// to the step budget running out first. `None` on payloads that
        /// do not state it.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        endpoint_reached: Option<bool>,
        /// EXP-39: which endpoint the burette was chasing. Omitted when
        /// it is the pH endpoint that every payload before EXP-39 meant.
        #[serde(default, skip_serializing_if = "Endpoint::is_ph")]
        endpoint: Endpoint,
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
            | Event::Neutralised { moles, .. }
            | Event::Reacted { moles, .. } => moles.0,
            _ => return true,
        };
        amount >= crate::OBSERVABLE_MOLES
    }
}
