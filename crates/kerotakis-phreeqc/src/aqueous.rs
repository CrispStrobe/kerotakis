//! The L2 aqueous equilibrator: maps a vessel's contents into a PHREEQC
//! problem, equilibrates against the thermodynamic database, and maps the
//! result back — dissolution, precipitation, pH and ionic strength, derived
//! rather than hardcoded.
//!
//! v1 inventory model: dissolved matter is tracked as element-total ions
//! (Na⁺, Cl⁻, Ag⁺, NO₃⁻) — exact for conservation, approximate as speciation
//! (complexes like AgCl(aq) are inside PHREEQC's solution, and the expert
//! register can surface them later). Solids with a known mineral phase enter
//! as amount-limited `EQUILIBRIUM_PHASES`, so partial dissolution at the
//! solubility limit is computed, not scripted.
//!
//! Known limitation, stated: dissolution/precipitation enthalpy is not yet
//! fed into the vessel's energy balance (curated ΔH arrives with the codex).

use kerotakis_core::i18n::Locale;
use kerotakis_core::phrase::{sentence_pair, Phrase, Slot};
use kerotakis_core::{
    species, Equilibrator, Event, ExchangeIon, ExchangeOccupancy, ExchangeSites, Headspace, Kelvin,
    Moles, Phase, Portion, Provenance, SolidSolution, SolidSolutionComponent, SolidSolutionModel,
    SolutionInfo, SolutionScope, SolveError, SpeciesDetail, SpeciesId, SurfaceOccupancy,
    SurfaceSiteKind, SurfaceSites, SurfaceSorbate, ThermalMode, Vessel,
};

#[cfg(feature = "engine")]
const RESET_NUMBERED_REACTANTS: &str = "DELETE\n    -all\nEND\n";

use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use std::sync::OnceLock;

#[cfg(feature = "engine")]
use crate::Phreeqc;
use crate::PhreeqcError;

fn env_dump_input_all() -> bool {
    static V: OnceLock<bool> = OnceLock::new();
    *V.get_or_init(|| std::env::var("KERO_DUMP_INPUT").as_deref() == Ok("all"))
}

fn env_dump_input() -> bool {
    static V: OnceLock<bool> = OnceLock::new();
    *V.get_or_init(|| std::env::var("KERO_DUMP_INPUT").is_ok())
}

fn env_redox() -> bool {
    static V: OnceLock<bool> = OnceLock::new();
    *V.get_or_init(|| std::env::var("KERO_REDOX").is_ok())
}

fn env_readback() -> bool {
    static V: OnceLock<bool> = OnceLock::new();
    *V.get_or_init(|| std::env::var("KERO_READBACK").is_ok())
}

// A solution property must be computed at the temperature the vessel finally
// reports. A loose 0.05 K thermal stop left pH from the preceding trial in the
// state; matched acid/base inventories could then retain their feed history at
// the fourth decimal place. This is an absolute numerical convergence target,
// not a chemistry-specific tolerance.
const THERMAL_FIXED_POINT_TOLERANCE_K: f64 = 1e-6;
// Selected-output amounts have finite decimal precision.  Near a phase
// boundary that can make the otherwise contracting thermal map alternate
// between two adjacent representable states instead of landing on one.  A
// cycle narrower than this is numerical quantisation, not non-convergence;
// wider cycles are still refused.
const THERMAL_FIXED_POINT_QUANTIZATION_CYCLE_K: f64 = 5e-5;
// The hot, near-saturated KCl route contracts more slowly than dilute acid/
// base states.  This is a safety ceiling, not an iteration budget: leave
// enough room for every normally contracting state to meet the same strict
// tolerance, while still refusing oscillation or divergence.
const MAX_THERMAL_FIXED_POINT_PASSES: usize = 64;
/// A physical aqueous/gas reentry must close before its thermal quote can
/// become the accepted state. This is independent of the thermal ceiling.
const MAX_CANONICAL_AQUEOUS_PASSES: usize = 8;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct FirstPoseHint {
    primary_route: Option<&'static str>,
    secondary: SecondaryFirstHint,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum SecondaryFirstHint {
    #[default]
    Auto,
    /// A provisional query already proved that the ordinary selector
    /// declines below its input floor. The next pose must verify this.
    Skip,
    /// Ask a provisional lean question while retaining every output and
    /// capability gate. The following ordinary pose must admit it.
    Probe,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SecondaryDecline {
    AlreadyPrimary,
    Unsupported,
    MissingReadback,
    EngineFailure,
    InvalidResult,
    BelowOutputParticleFloor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SecondaryDisposition {
    /// Private work, never a measured answer or a selector-floor witness.
    Pending,
    Accepted,
    BelowInputFloor,
    SkippedForReplay,
    Declined(SecondaryDecline),
}

struct SecondaryResult {
    provenance: Option<kerotakis_core::vessel::SolventActivityProvenance>,
    disposition: SecondaryDisposition,
}

enum PreparedSecondary {
    Resolved(Box<SecondaryResult>),
    Question(Box<ExactAuxiliaryQuestion>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AuxiliaryPolicy {
    Eager,
    DeferReservoir,
}

/// The exact already-prepared native question, not a problem reconstructed
/// from later stock. The witness associates it with one private primary pose.
struct ExactAuxiliaryQuestion {
    pose: Vessel,
    problem: Problem,
    database: &'static str,
    database_fingerprint: String,
    input: String,
    key: String,
    witness: Option<AuxiliaryPoseWitness>,
}

#[derive(Clone, PartialEq)]
struct AuxiliaryPoseWitness {
    heat_quote_j: f64,
    heat_mode: HeatMode,
    primary_database: &'static str,
    component_owner: ComponentCertificate,
    id: kerotakis_core::VesselId,
    temperature: Kelvin,
    pressure: kerotakis_core::Pascal,
    headspace: Headspace,
    thermal_mode: ThermalMode,
    contents: Vec<Portion>,
    unresolved_materials: Vec<kerotakis_core::vessel::UnresolvedMaterialPortion>,
    solution: Option<SolutionInfo>,
    free_proton: f64,
    free_hydroxide: f64,
    solute_charge: f64,
    co2_partial_pressure_atm: Option<f64>,
    pending_co2_transfer_mol: f64,
    elapsed_seconds: f64,
    excess_enthalpy_j: f64,
    step_start: Option<kerotakis_core::vessel::StepStart>,
    heat_input: Option<kerotakis_core::vessel::HeatInput>,
    aqueous_heat: kerotakis_core::vessel::AqueousHeatOwnership,
    aqueous_routing_said: Option<String>,
}

impl AuxiliaryPoseWitness {
    fn capture(vessel: &Vessel, pass: &AqueousPass) -> Self {
        Self {
            heat_quote_j: pass.heat_quote_j,
            heat_mode: pass.heat_mode,
            primary_database: pass.db_tag,
            component_owner: pass.component_certificate,
            id: vessel.id,
            temperature: vessel.temperature,
            pressure: vessel.pressure,
            headspace: vessel.headspace,
            thermal_mode: vessel.thermal_mode,
            contents: vessel.contents.clone(),
            unresolved_materials: vessel.unresolved_materials.clone(),
            solution: vessel.solution.clone(),
            free_proton: vessel.free_proton,
            free_hydroxide: vessel.free_hydroxide,
            solute_charge: vessel.solute_charge,
            co2_partial_pressure_atm: vessel.co2_partial_pressure_atm,
            pending_co2_transfer_mol: vessel.pending_co2_transfer_mol,
            elapsed_seconds: vessel.elapsed_seconds,
            excess_enthalpy_j: vessel.excess_enthalpy_j,
            step_start: vessel.step_start.clone(),
            heat_input: vessel.heat_input.clone(),
            aqueous_heat: vessel.aqueous_heat,
            aqueous_routing_said: vessel.aqueous_routing_said.clone(),
        }
    }
}

impl ExactAuxiliaryQuestion {
    fn validate_input(&self) -> Result<(), SolveError> {
        let fingerprint =
            crate::native_namespace::fingerprint(self.database).map_err(|detail| {
                canonical_closure_error(AqueousReason::FingerprintUnavailable, detail)
            })?;
        if self.database != "pitzer"
            || fingerprint != self.database_fingerprint
            || self.input != build_input(&self.pose, &self.problem, self.database)
            || self.key
                != format!(
                    "#solvent-activity-v1:{}:{}\n{}",
                    self.database, fingerprint, self.input
                )
        {
            return Err(canonical_closure_error(
                AqueousReason::QueuedInputBinding,
                "queued auxiliary question lost its exact database/input binding",
            ));
        }
        Ok(())
    }

    fn validate_pose(&self, vessel: &Vessel, pass: &AqueousPass) -> Result<(), SolveError> {
        self.validate_input()?;
        if self.witness.as_ref() != Some(&AuxiliaryPoseWitness::capture(vessel, pass))
            || !vessel.surfaces.is_empty()
            || !vessel.exchanges.is_empty()
            || !vessel.solid_solutions.is_empty()
            || !vessel.material_objects.is_empty()
            || !vessel.adsorbed.is_empty()
            || !vessel.electrodes.is_empty()
        {
            return Err(canonical_closure_error(
                AqueousReason::QueuedPoseValidation,
                "queued auxiliary answer belongs to another primary stock/heat/temperature pose",
            ));
        }
        Ok(())
    }
}

impl SecondaryResult {
    fn declined(disposition: SecondaryDisposition) -> Self {
        Self {
            provenance: None,
            disposition,
        }
    }
}

/// Only a measured input-floor transition permits an owner replay. Other
/// missing/unsupported/native failures remain ordinary refusals.
fn secondary_floor_replay(
    before: SecondaryDisposition,
    after: SecondaryDisposition,
) -> Option<SecondaryFirstHint> {
    match (before, after) {
        (SecondaryDisposition::Accepted, SecondaryDisposition::BelowInputFloor) => {
            Some(SecondaryFirstHint::Skip)
        }
        (SecondaryDisposition::BelowInputFloor, SecondaryDisposition::Accepted) => {
            Some(SecondaryFirstHint::Probe)
        }
        _ => None,
    }
}

fn checked_secondary_reentry(
    previous: SecondaryDisposition,
    current: SecondaryDisposition,
    first_hint: SecondaryFirstHint,
    secondary_replayed: &mut bool,
) -> Result<Option<SecondaryFirstHint>, SolveError> {
    if matches!(previous, SecondaryDisposition::Pending)
        || matches!(current, SecondaryDisposition::Pending)
    {
        return Err(canonical_closure_error(
            AqueousReason::PendingSelectorAuthority,
            "pending auxiliary work has no selector authority",
        ));
    }
    if previous == SecondaryDisposition::SkippedForReplay
        && current != SecondaryDisposition::BelowInputFloor
    {
        return Err(canonical_closure_error(
            AqueousReason::SecondarySkipUnverified,
            "ordinary selector did not verify the provisional secondary skip",
        ));
    }
    if first_hint == SecondaryFirstHint::Probe && current != SecondaryDisposition::Accepted {
        return Err(canonical_closure_error(
            AqueousReason::SecondaryProbeUnverified,
            "ordinary selector did not admit the provisional secondary probe",
        ));
    }
    if let Some(next) = secondary_floor_replay(previous, current) {
        if *secondary_replayed {
            return Err(canonical_closure_error(
                AqueousReason::SecondaryChangedAfterReplay,
                "secondary selector changed after its owner replay",
            ));
        }
        *secondary_replayed = true;
        return Ok(Some(next));
    }
    if previous != current
        && !(previous == SecondaryDisposition::SkippedForReplay
            && current == SecondaryDisposition::BelowInputFloor)
    {
        return Err(canonical_closure_error(
            AqueousReason::SecondaryChangedWithoutFloor,
            "secondary solvent selector changed outside a witnessed input-floor replay",
        ));
    }
    Ok(None)
}

/// Parse the actual secondary selected-output column. A valid osmotic
/// coefficient does not by itself certify the reported ionic strength:
/// `from_speciation` uses an ideal fallback when mu is nonpositive.
fn secondary_ionic_strength(rows: &[Vec<String>]) -> Result<f64, SecondaryDecline> {
    let cell = rows
        .first()
        .and_then(|head| head.iter().position(|h| h == "mu"))
        .and_then(|i| rows.last().and_then(|row| row.get(i)))
        .ok_or(SecondaryDecline::MissingReadback)?;
    let mu = cell
        .parse::<f64>()
        .map_err(|_| SecondaryDecline::InvalidResult)?;
    (mu.is_finite() && mu >= 0.0)
        .then_some(mu)
        .ok_or(SecondaryDecline::InvalidResult)
}

use crate::derived::{self, DerivedRole, ATMOSPHERIC, EQUILIBRIUM_GASES};
use crate::enthalpy;

/// Moles of free hydroxide, from PHREEQC's own species distribution.
///
/// `None` speciation means no solve has characterised this vessel yet, and
/// a beaker nobody has solved holds no measured hydroxide — which is the
/// right answer for the step that first adds an alkali, because the base
/// is then still a portion and is priced as one.
fn free_hydroxide_moles(species: Option<&[SpeciesDetail]>, water_kg: f64) -> f64 {
    measured_species_moles(species, "OH-", water_kg)
}

/// Moles of one species, from PHREEQC's own distribution. Absent means the
/// engine did not report it, which for `H+` or `OH-` means negligible.
fn measured_species_moles(species: Option<&[SpeciesDetail]>, name: &str, water_kg: f64) -> f64 {
    species
        .unwrap_or(&[])
        .iter()
        .find(|s| s.name == name)
        .map(|s| s.molality * water_kg)
        .unwrap_or(0.0)
}

/// Say which dataset answers this beaker — once, at the moment it becomes
/// true, and not again while it holds.
///
/// **The rule is #653's and this is the only implementation of it.** That
/// PR built fire-on-change for the direct solve
/// ([`PhreeqcEquilibrator::finalize_solution_info`]) and tested it; the
/// owner's 2026-09-18 ruling extends it to the two paths that were writing
/// a `Provenance` into the vessel and announcing nothing — MIX, and
/// solvent-only characterisation. A vessel filled either way held
/// provenance in its state that no reader could ever reach, because the
/// drawer reads `event.provenance` and no event carried it.
///
/// The comparison is [`Provenance::source_key`] — the engine, the dataset,
/// the model, and the SHAPE of the routing recipe, with every measurement
/// taken out of it. A number moving inside a reason is not a new reason:
/// the concentrated-brine route carries the molality in its own sentence
/// and that moves with every spoonful of salt.
///
/// It is compared against [`Vessel::aqueous_routing_said`] — what this
/// vessel has been TOLD — and not against `solution.provenance`, which is
/// the current source and which a later solver in the stack nests its own
/// clause into. That field moves for reasons a reader was never told
/// about; this one is the record of the telling.
///
/// **Why the state lives on the vessel.** The thermal fixed point solves a
/// CLONE of the vessel up to eight times per step and keeps only the last
/// one's events. Memory held in the solver would announce on the first
/// pass and be silent on the pass whose events actually survive.
fn announce_routing(vessel: &mut Vessel, provenance: &Provenance, events: &mut Vec<Event>) {
    let said = provenance.source_key();
    if vessel.aqueous_routing_said.as_deref() == Some(said.as_str()) {
        return;
    }
    vessel.aqueous_routing_said = Some(said);
    events.push(Event::SolutionRouted {
        vessel: vessel.id,
        provenance: provenance.clone(),
    });
}

/// What to call the dataset a solve ran on — the file NAME in a slot, and
/// the clauses around it as the part a catalogue translates.
///
/// Not simply `{tag}.dat`, because minteq.v4 is not run as vendored: this
/// lab adds one reviewed lactate definition to it (see
/// `databases::minteq_v4`). Reporting the bare filename would name a
/// database we are not running, in the field whose entire job is to let a
/// reader trace where a number came from.
///
/// **And that is exactly why this is a recipe and not a string.** What it
/// used to build was one sentence — *wateq4f.dat plus USBM IC 9429
/// reference-temperature complexes, with the reviewed Sander HBr
/// gas-uptake slice* — in which the first token is a FILE NAME that no
/// language translates and everything after it is English that every
/// language must. A German reader met the whole of it in English.
///
/// Two levels, and the nesting is what keeps the German honest: the
/// Sander clause is said ONCE, in one catalogue row, and wraps whatever
/// the base was. A flat row per dataset would have asked a translator for
/// the same clause three times and let the three drift.
///
/// `{file}` carries the file through both levels, which is how
/// [`Provenance::dataset_file`] gets the name back out of the sentence
/// without cutting the sentence up.
fn dataset_phrase(db_tag: &str) -> Phrase {
    let base = match db_tag {
        "minteq.v4" => Slot::phrase(Phrase::new(
            "provenance.dataset.plus-lactate-and-reference-complexes",
            "{file} plus reviewed lactate and USBM IC 9429 reference-temperature complexes",
            vec![(
                Provenance::DATASET_FILE_SLOT.to_string(),
                Slot::text("minteq.v4.dat"),
            )],
        )),
        "wateq4f" => Slot::phrase(Phrase::new(
            "provenance.dataset.plus-reference-complexes",
            "{file} plus USBM IC 9429 reference-temperature complexes",
            vec![(
                Provenance::DATASET_FILE_SLOT.to_string(),
                Slot::text("wateq4f.dat"),
            )],
        )),
        // A dataset run exactly as vendored is a NAME and nothing else, so
        // it goes straight into the wrapper's slot as text.
        other => Slot::text(format!("{other}.dat")),
    };
    Phrase::new(
        "provenance.dataset.with-gas-uptake-slice",
        "{file}, with the reviewed Sander HBr gas-uptake slice",
        vec![(Provenance::DATASET_FILE_SLOT.to_string(), base)],
    )
}

fn dataset_sources(db_tag: &str) -> Vec<String> {
    let mut sources: Vec<_> = derived::index_for(db_tag)
        .citations
        .iter()
        .take(3)
        .cloned()
        .collect();
    if matches!(db_tag, "wateq4f" | "minteq.v4") {
        sources.push(crate::complexation::SOURCE_ID.into());
    }
    sources.push(crate::aqueous_gases::SOURCE_ID.into());
    sources
}

fn reference_complex_boundary(vessel: &Vessel, distribution: &[SpeciesDetail]) -> Option<Event> {
    let thiocyanate = distribution
        .iter()
        .any(|s| s.molality > TRACE && (s.name.contains("Thiocyanate") || s.name.contains("SCN")));
    let complexes = distribution
        .iter()
        .any(|s| s.molality > TRACE && crate::complexation::uses_reference_complexes(&s.name));
    if !complexes && !thiocyanate {
        return None;
    }
    // Each caveat is a WHOLE sentence with its own key, and the space
    // between two of them is `look.sentence-join` rather than a
    // `push_str` — which is what the `.trim()` below used to be
    // apologising for. Three shapes fall out of two booleans, and a
    // language that joins sentences differently gets all three right from
    // one catalogue row.
    let temperature_note = if crate::complexation::temperature_covered(vessel.temperature.0) {
        Phrase::bare(
            "not-modeled.at-the-reference-temperature",
            "The solution is at the tabulated reference temperature.",
        )
    } else {
        Phrase::bare(
            "not-modeled.away-from-the-reference-temperature",
            "At this vessel temperature these are reference-temperature approximations, not validated temperature-dependent constants.",
        )
    };
    let complex_note = complexes.then(|| {
        Phrase::new(
            "not-modeled.usbm-reference-complexes",
            "Copper–ammine / iron–thiocyanate speciation uses public-domain USBM IC 9429 cumulative constants at 25 °C and zero ionic strength, with the selected database's activity corrections. {temperature} Complex-formation enthalpies, rate laws, and complex spectra are not supplied by this data slice; reaction heat and colour are incomplete where those contributions matter.",
            vec![("temperature".to_string(), Slot::phrase(temperature_note))],
        )
    });
    let thiocyanate_note = thiocyanate.then(|| {
        Phrase::bare(
            "not-modeled.thiocyanate-slice",
            "Thiocyanate is treated as a conserved SCN- ligand. HNCS protonation, ligand redox, and binding to metals other than Fe(III) are not covered by this slice; in particular it is not an exhaustive strong-acid thiocyanate model. KSCN is an aqueous analytical equivalent, not a solid dissolution model.",
        )
    });
    let reason = match (complex_note, thiocyanate_note) {
        (Some(complex), Some(scn)) => sentence_pair(complex, scn),
        (Some(only), None) | (None, Some(only)) => only,
        // Unreachable: the guard above returned on neither.
        (None, None) => return None,
    };
    Some(Event::not_modeled(
        vessel.id,
        kerotakis_core::ops::NotModelledCause::ModelBoundary,
        reason,
    ))
}

/// Native equilibrium includes bound ammonia, while the canonical NH3/NH4
/// portions still partition total reduced nitrogen using only free-ion weights.
fn silver_ammine_inventory_boundary(
    vessel: &Vessel,
    distribution: &[SpeciesDetail],
) -> Option<Event> {
    if !distribution.iter().any(|species| {
        species.molality > 0.0 && matches!(species.name.as_str(), "AgNH3+" | "Ag(NH3)2+")
    }) {
        return None;
    }
    Some(Event::not_modeled(
        vessel.id,
        kerotakis_core::ops::NotModelledCause::ModelBoundary,
        Phrase::bare(
            "not-modeled.silver-ammine-ligand-inventory",
            "Silver-ammonia equilibrium complexes and free ligand are represented in the solution speciation. The canonical NH3/NH4 portions remain analytical total-nitrogen carriers, including complex-bound ammonia; they are not measured free NH3 amounts. Their use for downstream ammonia volatilisation or curated reaction availability is not yet reconciled with bound ligand. This limitation does not withdraw the reported equilibrium complex distribution",
        ),
    ))
}

fn reactive_gas_boundary(vessel: &Vessel, problem: &Problem) -> Option<Event> {
    if !problem.gases.iter().any(|(_, species, _)| species == "HBr")
        && !problem
            .external_gases
            .iter()
            .any(|gas| gas.species == "HBr")
    {
        return None;
    }
    let temperature = if crate::aqueous_gases::temperature_covered(vessel.temperature.0) {
        Phrase::bare(
            "not-modeled.hbr-at-the-reference-temperature",
            "This state is at the 25 °C reference temperature.",
        )
    } else {
        Phrase::bare(
            "not-modeled.hbr-away-from-the-reference-temperature",
            "Away from 25 °C this uses a local constant-enthalpy van't Hoff approximation, not a validated broad-temperature correlation.",
        )
    };
    Some(Event::not_modeled(
        vessel.id,
        kerotakis_core::ops::NotModelledCause::ModelBoundary,
        Phrase::new(
            "not-modeled.hbr-gas-uptake",
            "HBr gas uptake uses the dissociative Henry constant from the CC-BY Sander compilation, converted from concentration to the dilute molal standard state. The reaction enthalpy is derived from its local temperature slope and solvent-density correction, not an independent calorimetric measurement. {temperature} Gas transfer is an equilibrium boundary, not a time-dependent absorption model.",
            vec![("temperature".to_string(), Slot::phrase(temperature))],
        ),
    ))
}

fn redox_distribution(
    problem: &Problem,
    db_tag: &str,
    value: &dyn Fn(&str) -> Option<f64>,
) -> Vec<kerotakis_core::RedoxState> {
    // The redox split, read back from the per-valence totals asked for
    // in `build_input`. A state at zero is kept out: "0 mol of Mn(VII)"
    // is true and is not what anyone means by a distribution.
    let mut redox: Vec<kerotakis_core::RedoxState> = Vec::new();
    for column in valence_totals(problem, db_tag) {
        let Some(moles) = value(&column) else {
            continue;
        };
        // A state that would render as "0%" is noise, not a
        // distribution: reporting "100% Fe(II), 0% Fe(III)" tells the
        // reader less than "all iron as Fe(II)" does.
        if moles <= 1e-12 {
            continue;
        }
        let Some((element, rest)) = column.split_once('(') else {
            continue;
        };
        let Ok(oxidation) = rest.trim_end_matches(')').parse::<i32>() else {
            continue;
        };
        redox.push(kerotakis_core::RedoxState {
            element: element.to_string(),
            oxidation,
            molality: moles,
        });
    }
    redox.sort_by(|a, b| {
        a.element
            .cmp(&b.element)
            .then(b.molality.total_cmp(&a.molality))
    });
    redox
}

/// Molar mass of water, g/mol — `molar-mass/water` in the registry pack,
/// reached through the constant `kerotakis-core` generates from it.
///
/// This file carried its own `18.015` until 2026-09-15. It happened to agree
/// with the record, which is precisely what made it invisible: nothing kept
/// the two equal, and two of the eighteen sites that carried a copy had
/// drifted onto a different number.
///
/// The two conversions this constant drives are inverses of each other -
/// moles of solvent to kilograms on the way into PHREEQC, kilograms back to
/// moles on the way out - so a single value is not a tidiness argument here.
/// It is what makes the round trip exact.
const WATER_MOLAR_MASS: f64 = kerotakis_core::constants::WATER_MOLAR_MASS_G_PER_MOL;
const TRACE: f64 = 1e-12;
/// Dissolved particle molality below which asking a second dataset for the
/// solvent's activity cannot buy anything.
///
/// Preserve the existing dilute-model admission boundary. Water activity now
/// comes from high-precision selected output, but that numerical improvement
/// does not independently validate a broader solvent-activity model domain.
const SECOND_OPINION_MIN_MOLALITY: f64 = 0.05;
const UNTRACKED_EXCHANGE_ELEMENTS: &[&str] = &[
    "Al", "Ba", "Cd", "Cu", "Fe", "K", "Li", "Mn", "Pb", "Sr", "Zn",
];

/// What one dataset says about the same vessel.
#[derive(Debug, Clone)]
pub struct PathResult {
    /// The dataset claim in English — see [`kerotakis_core::Provenance::dataset`].
    pub dataset: String,
    /// The model claim in English.
    pub model: String,
    /// The dataset claim as a recipe. `kero explain --compare` prints this
    /// beside three answers, and it carried the same welded English the
    /// provenance field did.
    pub dataset_phrase: Option<Phrase>,
    /// The model claim as a recipe.
    pub model_phrase: Option<Phrase>,
    pub outcome: PathOutcome,
}

impl PathResult {
    /// The dataset that answered, in the reader's language.
    #[must_use]
    pub fn dataset_in(&self, locale: Locale) -> String {
        self.dataset_phrase
            .as_ref()
            .map_or_else(|| self.dataset.clone(), |phrase| phrase.render(locale))
    }

    /// The model it applies, in the reader's language.
    #[must_use]
    pub fn model_in(&self, locale: Locale) -> String {
        self.model_phrase
            .as_ref()
            .map_or_else(|| self.model.clone(), |phrase| phrase.render(locale))
    }

    /// One dataset's answer, with both claims said as recipes.
    #[cfg(feature = "engine")]
    fn new(db_tag: &str, model: crate::dbindex::ActivityModel, outcome: PathOutcome) -> PathResult {
        let dataset = dataset_phrase(db_tag);
        let model = model.phrase();
        PathResult {
            dataset: dataset.render(Locale::EN),
            model: model.render(Locale::EN),
            dataset_phrase: Some(dataset),
            model_phrase: Some(model),
            outcome,
        }
    }
}

#[derive(Debug, Clone)]
pub enum PathOutcome {
    Solved {
        ph: f64,
        ionic_strength: f64,
        /// Phase amounts this dataset predicts, mol.
        phases: Vec<(String, f64)>,
    },
    /// The dataset does not carry the chemistry the question needs.
    CannotExpress { missing_elements: Vec<String> },
    /// A positive supplied solid cannot be represented on this dataset.
    /// Removing it would compare a different material inventory.
    CannotExpressPhases { missing_phases: Vec<String> },
    /// It tried and could not answer — honest, first-class.
    Failed { detail: String },
}

/// One cached solver result, keyed by database + canonical input.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub rows: Vec<Vec<String>>,
    pub species: Vec<SpeciesDetail>,
    /// (phase, saturation index) for every phase the database could form
    /// from the elements present — including phases this lab cannot name.
    #[serde(default)]
    pub saturation: Vec<(String, f64)>,
    /// Whether PHREEQC solved for pe rather than using the value it was
    /// handed. Its report says so in as many words.
    #[serde(default)]
    pub redox_adjusted: bool,
    /// Set when the electron balance was struck but did not *pin* pe — the
    /// equivalence point of a titration, where the potential is undefined.
    ///
    /// Stored the negative way round for legacy wire compatibility:
    /// `#[serde(default)]` gives `false` for an entry written before this
    /// field existed. That default is not electron authority. A produced
    /// witness or the actual current open-oxygen pose must qualify pe.
    #[serde(default)]
    pub pe_undetermined: bool,
    /// Optional source-produced proof of the electron boundary that owned
    /// this cached result. Old cache JSON has no witness and cannot acquire
    /// one merely because `pe_undetermined` defaulted to false.
    #[serde(default)]
    pub pe_authority: PeAuthority,
}

impl serde::Serialize for CacheEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Named JSON can omit a legacy witness. Postcard is positional and
        // must carry this field even when it is Unwitnessed.
        #[derive(serde::Serialize)]
        struct Human<'a> {
            key: &'a str,
            rows: &'a [Vec<String>],
            species: &'a [SpeciesDetail],
            saturation: &'a [(String, f64)],
            redox_adjusted: bool,
            pe_undetermined: bool,
            #[serde(skip_serializing_if = "PeAuthority::is_unwitnessed")]
            pe_authority: PeAuthority,
        }
        #[derive(serde::Serialize)]
        struct Binary<'a> {
            key: &'a str,
            rows: &'a [Vec<String>],
            species: &'a [SpeciesDetail],
            saturation: &'a [(String, f64)],
            redox_adjusted: bool,
            pe_undetermined: bool,
            pe_authority: PeAuthority,
        }
        if serializer.is_human_readable() {
            Human {
                key: &self.key,
                rows: &self.rows,
                species: &self.species,
                saturation: &self.saturation,
                redox_adjusted: self.redox_adjusted,
                pe_undetermined: self.pe_undetermined,
                pe_authority: self.pe_authority,
            }
            .serialize(serializer)
        } else {
            Binary {
                key: &self.key,
                rows: &self.rows,
                species: &self.species,
                saturation: &self.saturation,
                redox_adjusted: self.redox_adjusted,
                pe_undetermined: self.pe_undetermined,
                pe_authority: self.pe_authority,
            }
            .serialize(serializer)
        }
    }
}

/// How supersaturated a phase must be before we admit we are ignoring it,
/// in log units. Small positive indices are ordinary in natural waters and
/// reporting them would be noise; +1.0 is a tenfold excess over saturation,
/// which is a solution a chemist would not expect to survive.
const SUPERSATURATION_REPORTING_SI: f64 = 1.0;

/// A shippable pre-warmed cache.
#[derive(Debug, Clone, Default)]
pub struct CacheData {
    pub entries: Vec<CacheEntry>,
    /// Thermochemical values derived from the same databases as the cached
    /// equilibria. A cache-only device needs these too: otherwise it can
    /// reproduce the composition but not the temperature, and its next
    /// content-addressed lookup describes a different state.
    pub neutralisation_kj_per_mol: Vec<(String, f64)>,
}

// Existing pre-authority Postcard bytes have no fixed entry boundary at the
// optional field. They cannot be decoded safely. Regenerate prewarmed binary
// packs from the current source; named legacy JSON remains readable.
const CACHE_BINARY_MAGIC: [u8; 16] = *b"KEROTAKIS-CACHE2";
const CACHE_BINARY_VERSION: u8 = 1;

#[derive(serde::Serialize)]
struct CacheBinaryHeader {
    magic: [u8; 16],
    version: u8,
}

impl<'de> serde::Deserialize<'de> for CacheBinaryHeader {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (magic, version) = <([u8; 16], u8) as serde::Deserialize>::deserialize(deserializer)?;
        if magic != CACHE_BINARY_MAGIC || version != CACHE_BINARY_VERSION {
            return Err(serde::de::Error::custom(
                "incompatible prewarmed cache binary; regenerate this cache",
            ));
        }
        Ok(Self { magic, version })
    }
}

impl serde::Serialize for CacheData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(serde::Serialize)]
        struct Human<'a> {
            entries: &'a [CacheEntry],
            neutralisation_kj_per_mol: &'a [(String, f64)],
        }
        #[derive(serde::Serialize)]
        struct Binary<'a> {
            header: CacheBinaryHeader,
            entries: &'a [CacheEntry],
            neutralisation_kj_per_mol: &'a [(String, f64)],
        }
        if serializer.is_human_readable() {
            Human {
                entries: &self.entries,
                neutralisation_kj_per_mol: &self.neutralisation_kj_per_mol,
            }
            .serialize(serializer)
        } else {
            Binary {
                header: CacheBinaryHeader {
                    magic: CACHE_BINARY_MAGIC,
                    version: CACHE_BINARY_VERSION,
                },
                entries: &self.entries,
                neutralisation_kj_per_mol: &self.neutralisation_kj_per_mol,
            }
            .serialize(serializer)
        }
    }
}

impl<'de> serde::Deserialize<'de> for CacheData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct Human {
            entries: Vec<CacheEntry>,
            #[serde(default)]
            neutralisation_kj_per_mol: Vec<(String, f64)>,
        }
        #[derive(serde::Deserialize)]
        struct Binary {
            header: CacheBinaryHeader,
            entries: Vec<CacheEntry>,
            neutralisation_kj_per_mol: Vec<(String, f64)>,
        }
        if deserializer.is_human_readable() {
            let decoded = Human::deserialize(deserializer)?;
            Ok(Self {
                entries: decoded.entries,
                neutralisation_kj_per_mol: decoded.neutralisation_kj_per_mol,
            })
        } else {
            let decoded = Binary::deserialize(deserializer)?;
            let _ = decoded.header;
            Ok(Self {
                entries: decoded.entries,
                neutralisation_kj_per_mol: decoded.neutralisation_kj_per_mol,
            })
        }
    }
}

pub struct PhreeqcEquilibrator {
    /// wateq4f: inorganic natural-water chemistry, valid to high ionic
    /// strength — the default.
    #[cfg(feature = "engine")]
    inorganic: Phreeqc,
    /// minteq.v4: adds organic ligands (acetate), but its activity model is
    /// poor for concentrated brines (halite solubility comes out ~3.7
    /// instead of ~6.1 mol/kgw) — used only when the problem needs organics.
    /// Databases have validity domains; routing by problem is the honest
    /// answer.
    #[cfg(feature = "engine")]
    organic: Phreeqc,
    /// pitzer.dat: the specific-ion-interaction model, the right tool for
    /// Instances whose loaded database has been *pinned*: an input that
    /// redefines a couple (the state pins of `build_input_at`) rewrites
    /// the loaded database for the lifetime of the IPhreeqc instance, so
    /// those inputs run here and the pristine instances above never see a
    /// redefinition — the coupled bisection depends on the database's own
    /// couples. Every pinned input re-emits the pins for each fast-redox
    /// element it contains, so a stale pin for an element present is
    /// overwritten and a stale pin for an absent element has nothing to
    /// act on. Lazily created on first pinned input.
    #[cfg(feature = "engine")]
    pinned_inorganic: Option<Phreeqc>,
    #[cfg(feature = "engine")]
    pinned_organic: Option<Phreeqc>,
    #[cfg(feature = "engine")]
    pinned_brine: Option<Phreeqc>,
    /// concentrated brines — but it only knows the major-ion elements.
    #[cfg(feature = "engine")]
    brine: Phreeqc,
    /// Content-addressed result cache: same species set, T and P is the
    /// same answer (PLAN.md, P2). Keyed by database + canonical input.
    /// Rc-wrapped so cache hits avoid deep-cloning the selected-output
    /// rows, speciation and saturation vectors (OPT-3).
    cache: std::collections::HashMap<String, Rc<CachedSolve>>,
    cache_hits: usize,
    /// Heat of neutralisation per database tag, kJ/mol, asked of the
    /// database the first time that database is used.
    neutralisation: std::collections::HashMap<String, f64>,
    /// Why the most recent `solve_once` could not price its whole-step
    /// heat, if it could not. Read by the thermal fixed point: a step whose
    /// heat ledger is incomplete is characterised at its supplied
    /// temperature and says so, rather than applying a partial ledger.
    last_heat_unpriced: Option<enthalpy::Unpriced>,
    /// An outside solver, for builds that cannot link IPhreeqc themselves.
    ///
    /// `wasm32-unknown-unknown` cannot host PHREEQC's C++, so the browser
    /// build has always carried pre-warmed results and reported a stated
    /// miss for anything nobody computed in advance. That is honest and it
    /// is also not a laboratory: "try things" quietly degrades to "replay
    /// what we prepared" in the one distribution channel schools can
    /// actually use. IPhreeqc *does* build for Emscripten — the module and
    /// its test have existed since P0 — so what was missing was a way to
    /// hand it the question.
    ///
    /// The hook takes the database tag and the canonical input and returns
    /// the engine's two outputs as JSON. Everything downstream — routing,
    /// caching, parsing, the temperature fixed point — is unchanged, so the
    /// browser gets the same answers by the same path rather than a second
    /// implementation that could drift.
    hook: Option<SolveHook>,
    /// Cache of raw coupled-trial runs, keyed by database tag + exact input
    /// text. The redox bisection asks the engine dozens of nearly identical
    /// questions per equilibration, and the temperature fixed point re-asks
    /// many of them verbatim on its way to convergence; identical text is an
    /// identical answer. The key is the full text, deliberately: a hash key
    /// would trade a collision — however improbable — for a wrong chemical
    /// answer, and that is not a trade this codebase makes.
    trial_cache: std::collections::HashMap<(String, String), Rc<SolveOutput>>,
    trial_cache_hits: usize,
    /// Second solves made for a solvent activity alone, split by whether
    /// the engine ran. This is the COST of
    /// [`Self::solvent_activity_second_opinion`], and it is counted rather
    /// than estimated because it is the whole argument for that feature
    /// being conditional: an unconditional one would be
    /// `solvent_activity_asked == every aqueous step`.
    solvent_activity_engine_calls: usize,
    solvent_activity_cache_hits: usize,
    /// The electron activity the last successfully *bracketed* coupled solve
    /// converged to, carried across the temperature fixed point's
    /// iterations: the pe root barely moves between temperature guesses, so
    /// the next bisection starts from a narrow window around it instead of
    /// the full water-stability bracket. Reset per equilibration; a stale
    /// value costs one narrow pass that falls back to the full bracket.
    warm_pe: Option<f64>,
    /// Raw engine invocations since construction — OPT-7's before/after
    /// number, kept as a counter so a test can hold the budget.
    engine_calls: usize,
}

/// What one pe-bisection pass saw, for the caller to judge. The pass
/// narrows a bracket; whether that narrowing was a *struck balance* — root
/// bracketed, residual paid — is a separate question, answered in
/// `solve_coupled` exactly once, whichever bracket produced the numbers.
struct PeSearch {
    best: Option<Rc<SolveOutput>>,
    last_sum: Option<f64>,
    saw_below: bool,
    saw_above: bool,
    mid: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PeAuthority {
    /// A native pe column or a legacy cache's default boolean is not an
    /// electron boundary. Imported entries start here until the actual pose
    /// or a new coupled solve witnesses one.
    #[default]
    Unwitnessed,
    /// The reviewed fast-redox electron budget paid its residual and found
    /// a bracketed root, rather than merely returning a trial distribution.
    CoupledRoot,
    /// The electron budget settled at equivalence without a finite root.
    CoupledUnpinned,
    /// The actual canonical input imposed the open atmospheric O2 poise.
    OpenOxygen,
}

impl PeAuthority {
    fn is_unwitnessed(&self) -> bool {
        *self == Self::Unwitnessed
    }

    fn publishes_pe(self) -> bool {
        matches!(self, Self::CoupledRoot | Self::OpenOxygen)
    }
}

/// An atmospheric headspace label alone does not pose an electron boundary:
/// check the exact branch that wrote the native input as well. Finite sealed
/// O2 carried in GAS_PHASE_RAW is material stock, not this imposed poise.
fn open_oxygen_pe_authority(vessel: &Vessel, db_tag: &str, input: &str) -> PeAuthority {
    if vessel.uses_atmospheric_reservoir()
        && derived::index_for(db_tag).has_phase(ATMOSPHERIC_OXYGEN)
        && input.lines().any(|candidate| {
            candidate.split_whitespace().collect::<Vec<_>>()
                == ["pe", "4", ATMOSPHERIC_OXYGEN, ATMOSPHERIC_LOG_PO2]
        })
    {
        PeAuthority::OpenOxygen
    } else {
        PeAuthority::Unwitnessed
    }
}

fn current_pe_authority(
    recorded: PeAuthority,
    coupled_now: bool,
    coupling_failed: bool,
    vessel: &Vessel,
    db_tag: &str,
    input: &str,
) -> PeAuthority {
    match recorded {
        PeAuthority::CoupledRoot | PeAuthority::CoupledUnpinned
            if coupled_now && !coupling_failed =>
        {
            recorded
        }
        PeAuthority::OpenOxygen | PeAuthority::Unwitnessed if !coupled_now && !coupling_failed => {
            open_oxygen_pe_authority(vessel, db_tag, input)
        }
        _ => PeAuthority::Unwitnessed,
    }
}

#[cfg(test)]
mod pe_authority_tests {
    use super::*;
    use kerotakis_core::{Liters, VesselId};

    fn ammonia_state(sealed: bool) -> Vessel {
        let mut vessel = Vessel::new(VesselId(0), "one reduced-nitrogen state");
        vessel.deposit(
            SpeciesId::new("water"),
            Moles(3.873_993_893_977_241_6),
            Phase::Liquid,
        );
        vessel.deposit(SpeciesId::new("NH4+"), Moles(0.00008), Phase::Aqueous);
        vessel.deposit(SpeciesId::new("Cl-"), Moles(0.00008), Phase::Aqueous);
        if sealed {
            vessel.headspace = Headspace::Sealed {
                volume: Liters(0.035),
            };
            vessel.deposit(SpeciesId::new("N2"), Moles(0.00113), Phase::Gas);
            vessel.deposit(SpeciesId::new("O2"), Moles(0.0003), Phase::Gas);
        }
        vessel
    }

    #[test]
    fn only_the_actual_open_oxygen_input_or_paid_coupled_root_witnesses_pe() {
        let open = ammonia_state(false);
        let sealed = ammonia_state(true);
        let open_problem = partition(&open).expect("represented aqueous ammonia");
        let sealed_problem = partition(&sealed).expect("represented finite ammonia gas");
        assert!(redox_coupling(&open_problem, "wateq4f").is_none());
        assert!(redox_coupling(&sealed_problem, "wateq4f").is_none());
        let open_input = build_input(&open, &open_problem, "wateq4f");
        let sealed_input = build_input(&sealed, &sealed_problem, "wateq4f");
        assert_eq!(
            current_pe_authority(
                PeAuthority::Unwitnessed,
                false,
                false,
                &open,
                "wateq4f",
                &open_input
            ),
            PeAuthority::OpenOxygen
        );
        assert_eq!(
            current_pe_authority(
                PeAuthority::Unwitnessed,
                false,
                false,
                &sealed,
                "wateq4f",
                &sealed_input
            ),
            PeAuthority::Unwitnessed,
            "finite inert O2 gas cannot stand in for the open pe input"
        );
        assert_eq!(
            current_pe_authority(
                PeAuthority::OpenOxygen,
                false,
                false,
                &sealed,
                "wateq4f",
                &sealed_input
            ),
            PeAuthority::Unwitnessed,
            "a cached open marker cannot change the current sealed pose"
        );
        assert_eq!(
            current_pe_authority(
                PeAuthority::Unwitnessed,
                false,
                false,
                &open,
                "pitzer",
                &open_input
            ),
            PeAuthority::Unwitnessed,
            "a dataset without atmospheric O2 cannot borrow another route's input"
        );
        assert_eq!(
            current_pe_authority(
                PeAuthority::CoupledRoot,
                true,
                false,
                &sealed,
                "wateq4f",
                &sealed_input
            ),
            PeAuthority::CoupledRoot
        );
        assert_eq!(
            current_pe_authority(
                PeAuthority::CoupledUnpinned,
                true,
                false,
                &sealed,
                "wateq4f",
                &sealed_input
            ),
            PeAuthority::CoupledUnpinned
        );
        assert!(!PeAuthority::CoupledUnpinned.publishes_pe());
        assert_eq!(
            current_pe_authority(
                PeAuthority::CoupledRoot,
                true,
                true,
                &sealed,
                "wateq4f",
                &sealed_input
            ),
            PeAuthority::Unwitnessed,
            "a failed electron root cannot become authoritative on fallback"
        );
        assert_eq!(
            current_pe_authority(
                PeAuthority::CoupledRoot,
                false,
                false,
                &sealed,
                "wateq4f",
                &sealed_input
            ),
            PeAuthority::Unwitnessed,
            "a cached root from a different current problem is not a witness"
        );
    }

    #[test]
    fn legacy_default_false_is_not_a_witness_but_new_cache_metadata_round_trips() {
        let old = serde_json::json!({
            "key": "old canonical input", "rows": [], "species": [],
            "pe_undetermined": false
        });
        let old: CacheEntry = serde_json::from_value(old).expect("old cache wire shape");
        assert_eq!(old.pe_authority, PeAuthority::Unwitnessed);
        let old_json = serde_json::to_value(&old).unwrap();
        assert!(old_json.get("pe_authority").is_none());

        let mut current = old;
        current.pe_authority = PeAuthority::CoupledRoot;
        let encoded = serde_json::to_value(&current).unwrap();
        assert_eq!(encoded["pe_authority"], "coupled_root");
        let decoded: CacheEntry = serde_json::from_value(encoded).unwrap();
        assert_eq!(decoded.pe_authority, PeAuthority::CoupledRoot);
        assert!(!decoded.pe_undetermined);
    }

    #[test]
    fn checked_postcard_cache_preserves_every_authority_and_thermochemical_entry() {
        let variants = [
            PeAuthority::Unwitnessed,
            PeAuthority::CoupledRoot,
            PeAuthority::CoupledUnpinned,
            PeAuthority::OpenOxygen,
        ];
        let entries = variants
            .into_iter()
            .enumerate()
            .map(|(index, pe_authority)| CacheEntry {
                key: format!("controlled-transport-{index}"),
                rows: vec![vec!["pH".into()], vec!["7".into()]],
                species: vec![],
                saturation: vec![("Halite".into(), index as f64)],
                redox_adjusted: matches!(
                    pe_authority,
                    PeAuthority::CoupledRoot | PeAuthority::CoupledUnpinned
                ),
                pe_undetermined: pe_authority == PeAuthority::CoupledUnpinned,
                pe_authority,
            })
            .collect();
        let cache = CacheData {
            entries,
            neutralisation_kj_per_mol: vec![("wateq4f".into(), -57.3), ("minteq.v4".into(), -55.1)],
        };
        let bytes = postcard::to_allocvec(&cache).expect("new binary layout");
        assert_eq!(&bytes[..CACHE_BINARY_MAGIC.len()], &CACHE_BINARY_MAGIC);
        let replay: CacheData = postcard::from_bytes(&bytes).expect("same source cache transport");
        assert_eq!(replay.entries.len(), 4);
        for (entry, authority) in replay.entries.iter().zip(variants) {
            assert_eq!(entry.pe_authority, authority);
            assert_eq!(
                entry.pe_undetermined,
                authority == PeAuthority::CoupledUnpinned
            );
            assert_eq!(entry.rows[1][0], "7");
        }
        assert_eq!(
            replay.neutralisation_kj_per_mol,
            cache.neutralisation_kj_per_mol
        );
        assert_eq!(replay.entries[3].saturation[0], ("Halite".into(), 3.0));

        let named = serde_json::to_value(&cache).expect("existing named shape");
        assert!(named.get("header").is_none());
        assert!(named["entries"][0].get("pe_authority").is_none());
        assert_eq!(named["entries"][1]["pe_authority"], "coupled_root");
        let from_named: CacheData = serde_json::from_value(named).unwrap();
        assert_eq!(from_named.entries[0].pe_authority, PeAuthority::Unwitnessed);
        assert_eq!(from_named.entries[3].pe_authority, PeAuthority::OpenOxygen);
        let legacy: CacheData = serde_json::from_value(serde_json::json!({
            "entries": [{"key": "old", "rows": [], "species": [], "pe_undetermined": false}]
        }))
        .unwrap();
        assert_eq!(legacy.entries[0].pe_authority, PeAuthority::Unwitnessed);
        assert!(legacy.neutralisation_kj_per_mol.is_empty());
    }

    #[test]
    fn checked_postcard_cache_rejects_old_unknown_and_truncated_layouts() {
        #[derive(serde::Serialize)]
        struct OldEntry<'a> {
            key: &'a str,
            rows: &'a [Vec<String>],
            species: &'a [SpeciesDetail],
            saturation: &'a [(String, f64)],
            redox_adjusted: bool,
            pe_undetermined: bool,
        }
        #[derive(serde::Serialize)]
        struct OldData<'a> {
            entries: Vec<OldEntry<'a>>,
            neutralisation_kj_per_mol: Vec<(&'a str, f64)>,
        }
        let rows = vec![vec!["pH".into()], vec!["7".into()]];
        let old = OldData {
            entries: vec![
                OldEntry {
                    key: "old-a",
                    rows: &rows,
                    species: &[],
                    saturation: &[],
                    redox_adjusted: false,
                    pe_undetermined: false,
                },
                OldEntry {
                    key: "old-b",
                    rows: &rows,
                    species: &[],
                    saturation: &[],
                    redox_adjusted: false,
                    pe_undetermined: false,
                },
            ],
            neutralisation_kj_per_mol: vec![("wateq4f", -57.3)],
        };
        let old_bytes = postcard::to_allocvec(&old).unwrap();
        assert!(postcard::from_bytes::<CacheData>(&old_bytes).is_err());

        let new_bytes = postcard::to_allocvec(&CacheData {
            entries: vec![],
            neutralisation_kj_per_mol: vec![("wateq4f".into(), -57.3)],
        })
        .unwrap();
        for cutoff in [0, CACHE_BINARY_MAGIC.len(), new_bytes.len() - 1] {
            assert!(postcard::from_bytes::<CacheData>(&new_bytes[..cutoff]).is_err());
        }
        let mut unknown = new_bytes.clone();
        unknown[CACHE_BINARY_MAGIC.len()] = CACHE_BINARY_VERSION + 1;
        assert!(postcard::from_bytes::<CacheData>(&unknown).is_err());
        let mut other_format = new_bytes;
        other_format[0] ^= 1;
        assert!(postcard::from_bytes::<CacheData>(&other_format).is_err());
    }
}

#[derive(Clone)]
struct CachedSolve {
    rows: Vec<Vec<String>>,
    speciation: Vec<SpeciesDetail>,
    saturation: Vec<(String, f64)>,
    redox_adjusted: bool,
    pe_determined: bool,
    pe_authority: PeAuthority,
}

/// An outside aqueous solver: database tag and canonical input in, the
/// engine's two outputs out.
pub type SolveHook = Box<dyn FnMut(&str, &str) -> Result<SolveOutput, String>>;

/// What an outside solver must return: exactly what the linked engine
/// produces, so the code that reads it does not know the difference.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SolveOutput {
    /// Selected-output rows, header first.
    pub selected: Vec<Vec<String>>,
    /// The full run report, which carries the species distribution and the
    /// saturation indices.
    pub report: String,
    /// Set when the electron balance was struck but did not *pin* pe — the
    /// equivalence point of a titration, where the potential is undefined.
    ///
    /// Negative, defaulted, and for the same reason as `CacheEntry`'s copy
    /// of it: this struct is deserialised from whatever the browser's
    /// solver hook hands back, and that JSON is written by
    /// `web/kerotakis.mjs`, which knows nothing about electron balances. A
    /// required field here broke the demo outright; a field defaulting to
    /// "undetermined" would have been worse, because the demo would have
    /// kept working while quietly never reporting a potential again.
    #[serde(default)]
    pub pe_undetermined: bool,
}

/// Heat of neutralisation, kJ per mole of water formed, from the routed
/// database rather than from a constant of ours.
///
/// `H⁺ + OH⁻ → H₂O` is the reverse of the reaction that defines `OH-` in
/// every PHREEQC database, so its enthalpy is the negative of PHREEQC's
/// native species reaction-enthalpy calculation for `OH-`. The three datasets answer
/// 55.91, 55.81 and 56.36 kJ/mol, against a literature -55.8 for the ionic
/// reaction. (The -57.3 that school textbooks quote for "the enthalpy of
/// neutralisation" is the strong-acid/strong-base figure including dilution
/// effects, which is a different measurement.)
///
/// Read rather than curated on purpose: it cannot go stale, it moves with
/// the dataset the router chose, and the disagreement between the three is
/// the same disagreement the bench already shows for everything else.
#[cfg(feature = "engine")]
fn neutralisation_enthalpy(engine: &mut Phreeqc) -> Option<f64> {
    // Establish the temperature and pressure state explicitly rather than
    // relying on PHREEQC's constructor defaults.
    let probe = "SOLUTION 1\n    temp 25\n    pH 7\nEND\n";
    engine.run(probe).ok()?;
    let v = engine.species_delta_h("OH-").ok()?;
    (v.is_finite() && v > 0.0).then_some(-v)
}

#[cfg(feature = "engine")]
fn load_native_database(tag: &str) -> Result<Phreeqc, PhreeqcError> {
    let database = crate::native_namespace::database(tag).map_err(PhreeqcError::Engine)?;
    Phreeqc::with_database(&database.database)
}

impl PhreeqcEquilibrator {
    /// Settle the physical aqueous/gas answer at one thermal guess. The
    /// caller owns the candidate vessel; no intermediate answer is public.
    fn solve_at_temperature(
        &mut self,
        trial: &mut Vessel,
    ) -> Result<(Vec<Event>, f64, bool, f64), SolveError> {
        Self::solve_at_temperature_using(trial, AuxiliaryPolicy::Eager, |work| {
            self.perform_aqueous_work(work)
        })
    }

    fn perform_aqueous_work(&mut self, work: AqueousWork<'_>) -> Result<AqueousPass, SolveError> {
        match work {
            AqueousWork::Pose {
                candidate,
                past,
                owner,
                hint,
                auxiliary_policy,
            } => self.solve_once(candidate, past, owner, hint, auxiliary_policy),
            AqueousWork::Fulfil { candidate, pass } => self.fulfil_aqueous_pass(candidate, pass),
        }
    }

    fn first_pose_at_temperature(
        &mut self,
        owner: &Vessel,
    ) -> Result<AqueousFirstPose, SolveError> {
        Self::first_pose_at_temperature_with(owner, |trial| {
            self.solve_once(
                trial,
                &[],
                ComponentCertificate::Uncertified,
                FirstPoseHint::default(),
                AuxiliaryPolicy::DeferReservoir,
            )
        })
    }

    fn first_pose_at_temperature_with<F>(
        owner: &Vessel,
        solve: F,
    ) -> Result<AqueousFirstPose, SolveError>
    where
        F: FnOnce(&mut Vessel) -> Result<AqueousPass, SolveError>,
    {
        let owner_start = owner.clone();
        let mut trial = owner_start.clone();
        trial.step_start = Some(
            owner_start
                .step_start
                .clone()
                .unwrap_or_else(|| kerotakis_core::vessel::StepStart::capture(&owner_start)),
        );
        let pass = solve(&mut trial)?;
        Ok(AqueousFirstPose {
            owner_start,
            trial,
            pass,
        })
    }

    fn resume_first_pose(
        &mut self,
        output: &mut Vessel,
        seed: AqueousFirstPose,
    ) -> Result<(Vec<Event>, f64, bool, f64), SolveError> {
        Self::resume_first_pose_using(output, seed, |work| self.perform_aqueous_work(work))
    }

    #[cfg(test)]
    fn resume_first_pose_with<F>(
        output: &mut Vessel,
        seed: AqueousFirstPose,
        mut solve: F,
    ) -> Result<(Vec<Event>, f64, bool, f64), SolveError>
    where
        F: FnMut(
            &mut Vessel,
            &[Event],
            ComponentCertificate,
            FirstPoseHint,
        ) -> Result<AqueousPass, SolveError>,
    {
        Self::resume_first_pose_using(output, seed, |work| match work {
            AqueousWork::Pose {
                candidate,
                past,
                owner,
                hint,
                ..
            } => solve(candidate, past, owner, hint),
            AqueousWork::Fulfil { .. } => Err(SolveError::NotConverged {
                reason: None,
                solver: "phreeqc-canonical".into(),
                detail: "controlled pose adapter cannot invent an auxiliary answer".into(),
            }),
        })
    }

    fn resume_first_pose_using<F>(
        output: &mut Vessel,
        seed: AqueousFirstPose,
        mut perform: F,
    ) -> Result<(Vec<Event>, f64, bool, f64), SolveError>
    where
        F: for<'a> FnMut(AqueousWork<'a>) -> Result<AqueousPass, SolveError>,
    {
        let AqueousFirstPose {
            mut owner_start,
            trial,
            pass,
        } = seed;
        let pose_temperature = owner_start.temperature;
        let pose_headspace = owner_start.headspace;
        let heat_origin = owner_start
            .step_start
            .clone()
            .unwrap_or_else(|| kerotakis_core::vessel::StepStart::capture(&owner_start));
        let mut first = Some((trial, pass));
        let answer = Self::solve_at_temperature_using(
            &mut owner_start,
            AuxiliaryPolicy::DeferReservoir,
            |work| {
                let AqueousWork::Pose {
                    candidate,
                    past,
                    owner,
                    hint,
                    auxiliary_policy,
                } = work
                else {
                    return perform(work);
                };
                if let Some((trial, pass)) = first.take() {
                    // The driver still counts this native first pose as pass1.
                    // Replays use its immutable original owner and the normal
                    // callback; no post-solve stock becomes a new heat origin.
                    if !past.is_empty()
                        || owner != ComponentCertificate::Uncertified
                        || hint != FirstPoseHint::default()
                        || candidate.id != trial.id
                        || candidate.temperature != pose_temperature
                        || trial.temperature != pose_temperature
                        || candidate.headspace != pose_headspace
                        || candidate.step_start.as_ref() != Some(&heat_origin)
                    {
                        return Err(canonical_closure_error(
                            AqueousReason::FirstPoseSeedValidation,
                            "first aqueous pose does not match its original temperature/owner",
                        ));
                    }
                    *candidate = trial;
                    Ok(pass)
                } else {
                    perform(AqueousWork::Pose {
                        candidate,
                        past,
                        owner,
                        hint,
                        auxiliary_policy,
                    })
                }
            },
        )?;
        *output = owner_start;
        Ok(answer)
    }

    /// The closure is private to this module. Tests can supply a controlled
    /// native-answer sequence to prove cap exhaustion and rollback without
    /// changing a production route or executing a native engine locally.
    #[cfg(test)]
    fn solve_at_temperature_with<F>(
        vessel: &mut Vessel,
        mut solve: F,
    ) -> Result<(Vec<Event>, f64, bool, f64), SolveError>
    where
        F: FnMut(
            &mut Vessel,
            &[Event],
            ComponentCertificate,
            FirstPoseHint,
        ) -> Result<AqueousPass, SolveError>,
    {
        Self::solve_at_temperature_using(vessel, AuxiliaryPolicy::Eager, |work| match work {
            AqueousWork::Pose {
                candidate,
                past,
                owner,
                hint,
                ..
            } => solve(candidate, past, owner, hint),
            AqueousWork::Fulfil { .. } => Err(SolveError::NotConverged {
                reason: None,
                solver: "phreeqc-canonical".into(),
                detail: "controlled pose adapter cannot invent an auxiliary answer".into(),
            }),
        })
    }

    fn solve_at_temperature_using<F>(
        vessel: &mut Vessel,
        mut auxiliary_policy: AuxiliaryPolicy,
        mut perform: F,
    ) -> Result<(Vec<Event>, f64, bool, f64), SolveError>
    where
        F: for<'a> FnMut(AqueousWork<'a>) -> Result<AqueousPass, SolveError>,
    {
        let owner_start = vessel.clone();
        let caller_snapshot = owner_start.step_start.clone();
        let heat_origin = caller_snapshot
            .clone()
            .unwrap_or_else(|| kerotakis_core::vessel::StepStart::capture(&owner_start));
        let mut calls = 0;
        let mut first_hint = FirstPoseHint::default();
        let mut secondary_replayed = false;
        'replay: loop {
            if calls >= MAX_CANONICAL_AQUEOUS_PASSES {
                return Err(canonical_closure_error(
                    AqueousReason::PassCap,
                    format!(
                        "physical aqueous/gas answer did not close within {MAX_CANONICAL_AQUEOUS_PASSES} passes"
                    ),
                ));
            }
            // A route change invalidates the entire hypothetical answer,
            // including its outward gas and spent clock parcel. Begin again
            // with the immutable owner input and heat origin.
            let mut trial = owner_start.clone();
            trial.step_start = Some(heat_origin.clone());
            calls += 1;
            let first = perform(AqueousWork::Pose {
                candidate: &mut trial,
                past: &[],
                owner: ComponentCertificate::Uncertified,
                hint: first_hint,
                auxiliary_policy,
            })?;
            if first.has_pending_secondary()?
                && (auxiliary_policy != AuxiliaryPolicy::DeferReservoir
                    || first.component_certificate != ComponentCertificate::ReservoirWithOpaque
                    || trial
                        .solution
                        .as_ref()
                        .is_none_or(|info| info.scope != SolutionScope::Complete))
            {
                return Err(canonical_closure_error(
                    AqueousReason::PendingPoseIneligible,
                    "only a complete private reservoir pose may retain pending auxiliary work",
                ));
            }
            if let Some(tag) = first_hint.primary_route {
                let info = trial
                    .solution
                    .as_ref()
                    .filter(|info| info.scope == SolutionScope::Complete);
                let measured = info.map(|info| info.ionic_strength);
                let in_domain = match tag {
                    "wateq4f" => {
                        measured.is_some_and(|mu| mu.is_finite() && (0.0..=1.0).contains(&mu))
                    }
                    "pitzer" => measured.is_some_and(|mu| mu.is_finite() && mu >= 0.0),
                    _ => false,
                };
                if first.db_tag != tag || !in_domain {
                    return Err(canonical_closure_error(
                        AqueousReason::ForcedRouteValidation,
                        "forced replay route did not produce a complete answer inside its activity-model domain",
                    ));
                }
            }
            match first_hint.secondary {
                SecondaryFirstHint::Skip
                    if first.secondary_disposition != SecondaryDisposition::SkippedForReplay =>
                {
                    return Err(canonical_closure_error(
                        AqueousReason::SecondarySkipMismatch,
                        "secondary replay did not skip its first pose",
                    ));
                }
                SecondaryFirstHint::Probe
                    if first.secondary_disposition != SecondaryDisposition::Accepted =>
                {
                    return Err(canonical_closure_error(
                        AqueousReason::SecondaryProbeMismatch,
                        "secondary replay did not produce a valid provisional auxiliary answer",
                    ));
                }
                _ => {}
            }
            if first.component_certificate.is_certified()
                && first
                    .events
                    .iter()
                    .any(|event| matches!(event, Event::SolverFailed { .. }))
            {
                return Err(canonical_closure_error(
                    AqueousReason::CertifiedSolverFailure,
                    "first gas-aware aqueous answer reported a solver failure",
                ));
            }
            // Projection alone does not attest a current native answer. A
            // clock-owned opaque recipe stays outside the represented
            // solution and gets a separate immutable-stock certificate;
            // the represented solution itself must still be Complete.
            if !first.component_certificate.is_certified()
                || trial
                    .solution
                    .as_ref()
                    .is_none_or(|info| info.scope != SolutionScope::Complete)
            {
                if first.clock_co2_applied_mol != 0.0 || first_hint != FirstPoseHint::default() {
                    return Err(canonical_closure_error(
                        AqueousReason::CertifiedAnswerMissing,
                        "clock CO2 or model replay lacks a complete certified native answer",
                    ));
                }
                trial.step_start = caller_snapshot;
                *vessel = trial;
                return Ok((first.events, first.heat_quote_j, false, 0.0));
            }
            let route = first.db_tag;
            let component_owner = first.component_certificate;
            let clock_applied = first.clock_co2_applied_mol;
            if component_owner == ComponentCertificate::ClockWithOpaque && clock_applied == 0.0 {
                return Err(canonical_closure_error(
                    AqueousReason::ClockBoundaryMissing,
                    "opaque aqueous owner has no spent clock boundary",
                ));
            }
            if !clock_applied.is_finite()
                || (clock_applied != 0.0
                    && (owner_start.pending_co2_transfer_mol.abs()
                        <= kerotakis_core::clock::CO2_TRANSFER_FLOOR_MOL
                        || owner_start.pending_co2_transfer_mol != clock_applied))
            {
                return Err(canonical_closure_error(
                    AqueousReason::ClockParcelMismatch,
                    "clock CO2 parcel did not match its typed input",
                ));
            }
            let clock = crate::inventory::ClockCo2Boundary {
                vessel: trial.id,
                applied_mol: clock_applied,
            };
            // Only this private candidate spends the clock parcel. Every outer
            // temperature guess still starts from the original pending input.
            if clock_applied != 0.0 {
                trial.pending_co2_transfer_mol = 0.0;
            }
            let mut reservoirs = first.reservoirs.iter().cloned().collect::<BTreeSet<_>>();
            // Earlier native crossings retain their original reservoir owner
            // even if exact outward clock removal leaves no carbon candidate on
            // a later reentry.
            let mut owned_reservoirs = reservoirs.clone();
            let reservoir_refs = owned_reservoirs
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>();
            let mut all_events = first.events.clone();
            let mut own_gas_out = BTreeMap::<SpeciesId, f64>::new();
            signed_external_gas(&trial, &first.events, &mut own_gas_out)?;
            if component_owner == ComponentCertificate::ReservoirWithOpaque {
                crate::inventory::certify_reservoir_opaque_inventory(
                    &owner_start,
                    &trial,
                    &net_gas_events(&trial, &own_gas_out),
                    &reservoir_refs,
                )?;
            } else {
                crate::inventory::certify_gas_aware_inventory(
                    &owner_start,
                    &trial,
                    &net_gas_events(&trial, &own_gas_out),
                    &reservoir_refs,
                    clock,
                )?;
                if component_owner == ComponentCertificate::ClockWithOpaque {
                    crate::inventory::certify_opaque_clock_stock(&owner_start, &trial)?;
                }
            }
            let mut previous_state = trial.clone();
            let mut previous_pass = first;
            while calls < MAX_CANONICAL_AQUEOUS_PASSES {
                let mut private_snapshot = heat_origin.clone();
                for (species, signed) in &own_gas_out {
                    private_snapshot.note_gas_out(species, *signed);
                }
                if clock_applied != 0.0 {
                    private_snapshot.note_gas_out(&SpeciesId::new("CO2"), -clock_applied);
                }
                trial.step_start = Some(private_snapshot);
                calls += 1;
                let mut pass = perform(AqueousWork::Pose {
                    candidate: &mut trial,
                    past: &all_events,
                    owner: component_owner,
                    hint: FirstPoseHint::default(),
                    auxiliary_policy,
                })?;
                if !pass.component_certificate.is_certified()
                    || pass.component_certificate != component_owner
                    || pass.clock_co2_applied_mol != 0.0
                    || pass
                        .events
                        .iter()
                        .any(|event| matches!(event, Event::SolverFailed { .. }))
                {
                    return Err(canonical_closure_error(
                        AqueousReason::ReentryCertificateBoundaryCheck,
                        "gas-aware aqueous route or ownership changed during reentry",
                    ));
                }
                if trial
                    .solution
                    .as_ref()
                    .is_none_or(|info| info.scope != SolutionScope::Complete)
                {
                    return Err(canonical_closure_error(
                        AqueousReason::ReentryAnswerIncomplete,
                        "route-changing aqueous reentry lacked a complete native answer",
                    ));
                }
                let previous_pending = previous_pass.has_pending_secondary()?;
                let current_pending = pass.has_pending_secondary()?;
                if current_pending && component_owner != ComponentCertificate::ReservoirWithOpaque {
                    return Err(canonical_closure_error(
                        AqueousReason::PendingOwnerNotReservoir,
                        "non-reservoir owner retained pending auxiliary work",
                    ));
                }
                if pass.db_tag != route {
                    if !matches!(
                        (route, pass.db_tag),
                        ("pitzer", "wateq4f") | ("wateq4f", "pitzer")
                    ) {
                        return Err(canonical_closure_error(
                            AqueousReason::ModelTransitionUnsupported,
                            "unsupported aqueous model transition",
                        ));
                    }
                    if previous_pending || current_pending {
                        auxiliary_policy = AuxiliaryPolicy::Eager;
                    }
                    first_hint.primary_route = Some(pass.db_tag);
                    first_hint.secondary = SecondaryFirstHint::Auto;
                    continue 'replay;
                }
                if !previous_pending && !current_pending {
                    // Eager owners retain the original selector-before-ledger
                    // ordering: a hypothetical rejected route is discarded
                    // before its speculative stock/gas can be certified.
                    if let Some(next) = checked_secondary_reentry(
                        previous_pass.secondary_disposition,
                        pass.secondary_disposition,
                        first_hint.secondary,
                        &mut secondary_replayed,
                    )? {
                        first_hint.secondary = next;
                        continue 'replay;
                    }
                }
                if previous_pending != current_pending {
                    // A capability/input-floor transition is not an auxiliary
                    // answer. Replay eagerly from the original owner without
                    // refunding any physical poses already spent.
                    auxiliary_policy = AuxiliaryPolicy::Eager;
                    continue 'replay;
                }
                let next_reservoirs = pass.reservoirs.iter().cloned().collect::<BTreeSet<_>>();
                let only_clock_co2_emerged = clock_applied != 0.0
                    && !reservoirs.contains("CO2")
                    && next_reservoirs.len() == reservoirs.len() + 1
                    && next_reservoirs.contains("CO2")
                    && reservoirs.is_subset(&next_reservoirs);
                let only_physical_co2_exhausted =
                    reviewed_co2_exhaustion(&reservoirs, &next_reservoirs, &trial);
                if next_reservoirs != reservoirs
                    && !only_clock_co2_emerged
                    && !only_physical_co2_exhausted
                {
                    return Err(canonical_closure_error(
                        AqueousReason::ReservoirSetChanged,
                        "gas-aware aqueous route or ownership changed during reentry",
                    ));
                }
                if only_clock_co2_emerged || only_physical_co2_exhausted {
                    reservoirs = next_reservoirs;
                    owned_reservoirs.extend(reservoirs.iter().cloned());
                }
                let reservoir_refs = owned_reservoirs
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>();
                signed_external_gas(&trial, &pass.events, &mut own_gas_out)?;
                if component_owner == ComponentCertificate::ReservoirWithOpaque {
                    crate::inventory::certify_reservoir_opaque_inventory(
                        &owner_start,
                        &trial,
                        &net_gas_events(&trial, &own_gas_out),
                        &reservoir_refs,
                    )?;
                } else {
                    crate::inventory::certify_gas_aware_inventory(
                        &owner_start,
                        &trial,
                        &net_gas_events(&trial, &own_gas_out),
                        &reservoir_refs,
                        clock,
                    )?;
                    if component_owner == ComponentCertificate::ClockWithOpaque {
                        crate::inventory::certify_opaque_clock_stock(&owner_start, &trial)?;
                    }
                }
                if previous_pending {
                    if canonical_primary_progress(&previous_state, &trial, &previous_pass, &pass)?
                        == PrimaryProgress::Moving
                    {
                        all_events.extend(pass.events.iter().cloned());
                        previous_state = trial.clone();
                        previous_pass = pass;
                        continue;
                    }
                    previous_pass = perform(AqueousWork::Fulfil {
                        candidate: &mut previous_state,
                        pass: previous_pass,
                    })?;
                    pass = perform(AqueousWork::Fulfil {
                        candidate: &mut trial,
                        pass,
                    })?;
                    if previous_pass.has_pending_secondary()? || pass.has_pending_secondary()? {
                        return Err(canonical_closure_error(
                            AqueousReason::PairedAuxiliaryIncomplete,
                            "paired auxiliary work was not fulfilled",
                        ));
                    }
                    if previous_pass.secondary_disposition != SecondaryDisposition::Accepted
                        || pass.secondary_disposition != SecondaryDisposition::Accepted
                    {
                        // Preserve native failure/decline as its actual typed
                        // result. The old eager owner decides its disposition;
                        // a missing result can never be promoted to Accepted.
                        auxiliary_policy = AuxiliaryPolicy::Eager;
                        continue 'replay;
                    }
                }
                if previous_pending {
                    if let Some(next) = checked_secondary_reentry(
                        previous_pass.secondary_disposition,
                        pass.secondary_disposition,
                        first_hint.secondary,
                        &mut secondary_replayed,
                    )? {
                        first_hint.secondary = next;
                        continue 'replay;
                    }
                }
                let closed = canonical_physical_delta_closed(
                    &previous_state,
                    &trial,
                    &previous_pass,
                    &pass,
                )?;
                if previous_pending && !closed {
                    // Primary progress was ready, but the two actual auxiliary
                    // answers did not close. Replay the original eager owner
                    // now; do not spend another speculative primary pose or
                    // refund the already-spent global physical-pass budget.
                    auxiliary_policy = AuxiliaryPolicy::Eager;
                    continue 'replay;
                }
                all_events.extend(pass.events.iter().cloned());
                if closed {
                    let events =
                        closed_aqueous_events(&owner_start, &trial, &all_events, &own_gas_out)?;
                    trial.step_start = caller_snapshot;
                    *vessel = trial;
                    return Ok((events, pass.heat_quote_j, true, clock_applied));
                }
                previous_state = trial.clone();
                previous_pass = pass;
            }
            return Err(canonical_closure_error(
                AqueousReason::PassCap,
                format!(
                    "physical aqueous/gas answer did not close within {MAX_CANONICAL_AQUEOUS_PASSES} passes"
                ),
            ));
        }
    }

    /// Install an outside solver. See [`SolveOutput`].
    pub fn set_hook(&mut self, hook: SolveHook) {
        self.hook = Some(hook);
    }

    /// Run one input and hand back the engine's two outputs.
    ///
    /// The single place the linked engine and an attached outside one are
    /// told apart, so everything above this — routing, the redox
    /// bisection, the caching — is written once and behaves identically in
    /// a terminal and in a browser tab.
    /// The instance an input runs on: pinned inputs (they carry a
    /// `SOLUTION_SPECIES` redefinition) get the pinned sibling of the
    /// routed database's instance — see the field docs.
    #[cfg(feature = "engine")]
    fn engine_for(&mut self, db_tag: &str, input: &str) -> Result<&mut Phreeqc, SolveError> {
        if input.contains("SOLUTION_SPECIES") {
            let slot = match db_tag {
                "minteq.v4" => &mut self.pinned_organic,
                "pitzer" => &mut self.pinned_brine,
                _ => &mut self.pinned_inorganic,
            };
            if slot.is_none() {
                *slot =
                    Some(
                        load_native_database(db_tag).map_err(|e| SolveError::NotConverged {
                            reason: Some(Box::new(
                                AqueousReason::PinnedInstanceLoadFailed.phrase(),
                            )),
                            solver: "phreeqc-aqueous".to_string(),
                            detail: format!("could not load a pinned engine instance: {e}"),
                        })?,
                    );
            }
            Ok(slot.as_mut().expect("just created"))
        } else {
            Ok(match db_tag {
                "minteq.v4" => &mut self.organic,
                "pitzer" => &mut self.brine,
                _ => &mut self.inorganic,
            })
        }
    }

    pub(crate) fn run_raw(&mut self, db_tag: &str, input: &str) -> Result<SolveOutput, SolveError> {
        self.engine_calls += 1;
        let namespace = crate::native_namespace::database(db_tag).map_err(|detail| {
            SolveError::NotConverged {
                reason: Some(Box::new(
                    AqueousReason::ComponentNamespaceUnavailable.phrase(),
                )),
                solver: "phreeqc-component-isolation".into(),
                detail,
            }
        })?;
        let native_input = crate::native_namespace::input(namespace, input);
        #[cfg(feature = "engine")]
        {
            let engine = self.engine_for(db_tag, input)?;
            engine
                .run(RESET_NUMBERED_REACTANTS)
                .map_err(|e| SolveError::NotConverged {
                    reason: Some(Box::new(AqueousReason::ReusedEngineResetFailed.phrase())),
                    solver: "phreeqc-aqueous".to_string(),
                    detail: format!("could not reset reused IPhreeqc state: {e}"),
                })?;
            engine.run(&native_input).map_err(|e| {
                // The input is the whole question; when the engine refuses
                // it, being able to see it is the difference between a
                // diagnosis and a guess.
                if env_dump_input() {
                    eprintln!("--- PHREEQC input that failed ---\n{native_input}---");
                }
                SolveError::NotConverged {
                    reason: None,
                    solver: "phreeqc-aqueous".to_string(),
                    detail: e.to_string(),
                }
            })?;
            Ok(SolveOutput {
                pe_undetermined: false,
                selected: crate::native_namespace::selected(namespace, engine.selected_output()),
                report: crate::native_namespace::report(namespace, &engine.output_string()),
            })
        }
        #[cfg(not(feature = "engine"))]
        {
            let Some(hook) = self.hook.as_mut() else {
                return Err(SolveError::NotConverged {
                    reason: Some(Box::new(AqueousReason::CacheMiss.phrase())),
                    solver: "phreeqc-aqueous (cache-only build)".to_string(),
                    detail: "this state is not in the shipped results and there is no solver here to compute it".to_string(),
                });
            };
            let mut output = hook(db_tag, &native_input).map_err(|e| SolveError::NotConverged {
                reason: None,
                solver: "phreeqc-aqueous (external engine)".to_string(),
                detail: e,
            })?;
            output.selected = crate::native_namespace::selected(namespace, output.selected);
            output.report = crate::native_namespace::report(namespace, &output.report);
            Ok(output)
        }
    }

    /// Solve the vessel with its redox elements *coupled*: find the
    /// electron activity at which the electrons that went in are the
    /// electrons that come out.
    ///
    /// Naming an oxidation state pins that element in its own mass balance,
    /// so an oxidant and a reductant entered that way never react. Removing
    /// the tags lets them react but leaves pe undetermined — and pe is
    /// exactly what the electron budget fixes. Σ(oxidation × moles) rises
    /// monotonically with pe, so this is a bisection, and it is the same
    /// move `equilibrate_hp` makes when it bisects temperature to conserve
    /// enthalpy.
    fn solve_coupled(
        &mut self,
        vessel: &Vessel,
        problem: &Problem,
        db_tag: &str,
        coupling: &RedoxCoupling,
    ) -> Result<SolveOutput, SolveError> {
        // pe outside roughly −12..22 is past the stability field of water
        // at any pH, so nothing chemical lives there.
        // Bracketed by the stability field of water rather than by
        // arithmetic convenience. Above about pe 17 water itself is being
        // oxidised and chloride goes with it, which is where the solver
        // starts refusing — and a bracket that reaches into that region
        // lets a run of failures march the search into it.
        const FULL: (f64, f64) = (-10.0, 17.0);
        // The tolerance the balance is finally judged by; the search is
        // allowed to stop the moment it is met, because grinding the
        // bracket to 1e-6 pe after the residual is paid only re-prices an
        // answer already bought (OPT-7).
        let residual_tol = 1e-9_f64.max(1e-4 * coupling.scale);
        // Warm start (OPT-7): across the temperature fixed point's
        // iterations the pe root barely moves, so a narrow window around
        // the last *bracketed* answer usually contains it. The narrow pass
        // must prove itself by the same two standards as any other — root
        // seen from both sides, residual paid — and anything less falls
        // back to the full bracket, so refusals and undetermined-pe
        // verdicts are only ever pronounced on the full-bracket evidence
        // they were designed for.
        let search = match self.warm_pe {
            Some(pe) => {
                let warm = self.bisect_pe(
                    vessel,
                    problem,
                    db_tag,
                    coupling,
                    (pe - 0.75).max(FULL.0),
                    (pe + 0.75).min(FULL.1),
                    residual_tol,
                )?;
                let warm_residual = warm
                    .last_sum
                    .map_or(f64::INFINITY, |s| (s - coupling.target).abs());
                if warm.saw_below && warm.saw_above && warm_residual <= residual_tol {
                    warm
                } else {
                    self.bisect_pe(
                        vessel,
                        problem,
                        db_tag,
                        coupling,
                        FULL.0,
                        FULL.1,
                        residual_tol,
                    )?
                }
            }
            None => self.bisect_pe(
                vessel,
                problem,
                db_tag,
                coupling,
                FULL.0,
                FULL.1,
                residual_tol,
            )?,
        };
        let PeSearch {
            best,
            last_sum,
            saw_below,
            saw_above,
            mid,
        } = search;
        // A narrowed bracket is not a struck balance. When the root lies
        // outside the water-stability window the interval simply collapses
        // onto an edge, and the last trial — an ordinary-looking
        // distribution — was being returned as though it were the answer.
        //
        // Excess oxidant is what does this. 0.0015 mol of permanganate
        // against 0.005 mol of iron(II) needs 0.0075 mol of electrons and
        // the iron can supply 0.005; the missing 0.0025 mol would have to
        // come from oxidising the water itself. (Chloride would go too, but
        // it is not what decides this: the same beaker made up with sulfuric
        // acid instead of hydrochloric is refused with an identical 2.500e-3
        // residual.) PHREEQC will do exactly
        // that, but the bench does not carry the oxygen it makes, so the
        // books came out 12% short while the beaker reported every last
        // manganese as Mn(II) — a colourless answer to the one titration
        // whose entire point is that the excess stays purple.
        //
        // Refusing here is the honest move and it costs nothing: the stack
        // carries on past a solver that declines, and the vessel is
        // reported as unmodelled rather than as solved.
        let residual = last_sum.map_or(f64::INFINITY, |sum| (sum - coupling.target).abs());
        if residual > residual_tol {
            return Err(SolveError::NotConverged {
                reason: Some(Box::new(if last_sum.is_some() {
                    AqueousReason::RedoxBalanceOutsideWater.phrase()
                } else {
                    AqueousReason::RedoxReadbackUnavailable.phrase()
                })),
                solver: "phreeqc-aqueous (redox)".to_string(),
                detail: format!(
                    "no electron activity in the stability field of water balances this \
                     beaker: {residual:.3e} mol of electron-equivalents are \
                     unaccounted for out of {:.3e}. An oxidant in excess of what \
                     the reductants can supply has to take its remaining electrons \
                     from the solvent itself, which this bench does not model",
                    coupling.scale,
                ),
            });
        }
        // The balance can be struck without pe being pinned by it.
        //
        // At exact equivalence both couple members are spent, so the sum is
        // flat in pe and approaches the target asymptotically instead of
        // crossing it: 1.699941e-2, 1.699992e-2, 1.699999e-2 against a
        // target of 1.7e-2, never once on the other side. The root is at
        // infinite pe — the last trace of iron(II) is only consumed in the
        // limit — so the search marches to the top of the bracket and the
        // residual there is a passing 1e-8.
        //
        // Reporting that as "pe 17.00 (+1.006 V)" would be publishing the
        // bracket ceiling as a measurement, which is the same fault the
        // residual check was added to remove, wearing a convergence as a
        // disguise. The distribution is right and is kept; the potential is
        // withheld, because at equivalence a redox potential genuinely is
        // undefined — that steepness is why the endpoint is detectable at
        // all.
        let rc = best.ok_or_else(|| SolveError::NotConverged {
            reason: Some(Box::new(AqueousReason::RedoxSearchDidNotConverge.phrase())),
            solver: "phreeqc-aqueous (redox)".to_string(),
            detail: "the electron balance did not converge".to_string(),
        })?;
        let mut out = Rc::unwrap_or_clone(rc);
        out.pe_undetermined = !(saw_below && saw_above);
        if !out.pe_undetermined {
            // Only a genuinely bracketed root seeds the next warm start; an
            // asymptotic equivalence point has no pe worth starting from.
            self.warm_pe = Some(mid);
        }
        Ok(out)
    }

    /// One bisection pass of the electron-balance search over `[lo, hi]`.
    ///
    /// Returns what it saw and leaves the judging — was the balance
    /// struck, was the root bracketed — to `solve_coupled`, so the warm
    /// and full brackets are held to identical standards.
    #[allow(clippy::too_many_arguments)]
    fn bisect_pe(
        &mut self,
        vessel: &Vessel,
        problem: &Problem,
        db_tag: &str,
        coupling: &RedoxCoupling,
        mut lo: f64,
        mut hi: f64,
        residual_tol: f64,
    ) -> Result<PeSearch, SolveError> {
        let mut best: Option<Rc<SolveOutput>> = None;
        let mut last_sum: Option<f64> = None;
        // Whether the residual was ever seen on both sides of zero. A
        // bisection that only ever approaches from one side never bracketed
        // a root: it walked to an edge, and the edge is not a measurement.
        let (mut saw_below, mut saw_above) = (false, false);
        let mut mid = 0.5 * (lo + hi);
        let mut interior_probe: u32 = 1;
        // Failed evaluations have no residual sign. Keep the valid bounds
        // unchanged and try dyadic interior points: midpoint, quarters,
        // eighths, sixteenths. Both sides of a numerical hole are explored.
        // Reset to the midpoint after each successful bound update. The
        // total evaluation budget and per-interval probe budget are bounded.
        for _ in 0..68 {
            let level = 31 - interior_probe.leading_zeros();
            let denominator = 1_u32 << (level + 1);
            let numerator = 2 * (interior_probe - (1_u32 << level)) + 1;
            let trial_pe = lo + (hi - lo) * numerator as f64 / denominator as f64;
            let input = build_input_at(vessel, problem, db_tag, Some((trial_pe, coupling)));
            let Ok(out) = self.run_trial(db_tag, &input) else {
                if interior_probe == 15 {
                    break;
                }
                interior_probe += 1;
                continue;
            };
            let Some(sum) = oxidation_sum(&out.selected, &coupling.columns, problem.kgw) else {
                return Err(SolveError::NotConverged {
                    reason: Some(Box::new(AqueousReason::RedoxTotalsMissing.phrase())),
                    solver: "phreeqc-aqueous (redox)".to_string(),
                    detail: "the coupled run reported no oxidation-state totals".to_string(),
                });
            };
            mid = trial_pe;
            interior_probe = 1;
            if env_redox() {
                eprintln!("  pe={mid:.3} sum={sum:.6e} target={:.6e}", coupling.target);
            }
            last_sum = Some(sum);
            best = Some(out);
            if sum < coupling.target {
                saw_below = true;
                lo = mid;
            } else {
                saw_above = true;
                hi = mid;
            }
            // Residual break (OPT-7): stopping here is judged by the same
            // tolerance the post-check enforces, so nothing that would have
            // been refused gets waved through — the search just stops
            // paying for precision the verdict cannot use.
            if (sum - coupling.target).abs() <= residual_tol {
                break;
            }
            if hi - lo < 1e-6 {
                break;
            }
        }
        Ok(PeSearch {
            best,
            last_sum,
            saw_below,
            saw_above,
            mid,
        })
    }

    /// Whether this equilibrator can compute a state nobody pre-computed.
    pub fn can_solve(&self) -> bool {
        cfg!(feature = "engine") || self.hook.is_some()
    }
}

impl PhreeqcEquilibrator {
    #[cfg(feature = "engine")]
    pub fn new() -> Result<Self, PhreeqcError> {
        let mut inorganic = load_native_database("wateq4f")?;
        let mut organic = load_native_database("minteq.v4")?;
        let mut brine = load_native_database("pitzer")?;
        // Asked once, of each dataset, rather than written down by us. A
        // dataset that declines to answer simply contributes no
        // neutralisation heat, which is the state the bench was in before.
        let mut neutralisation = std::collections::HashMap::new();
        for (tag, engine) in [
            ("wateq4f", &mut inorganic),
            ("minteq.v4", &mut organic),
            ("pitzer", &mut brine),
        ] {
            if let Some(dh) = neutralisation_enthalpy(engine) {
                neutralisation.insert(tag.to_string(), dh);
            }
        }
        Ok(PhreeqcEquilibrator {
            inorganic,
            organic,
            brine,
            pinned_inorganic: None,
            pinned_organic: None,
            pinned_brine: None,
            cache: std::collections::HashMap::new(),
            cache_hits: 0,
            trial_cache: std::collections::HashMap::new(),
            trial_cache_hits: 0,
            solvent_activity_engine_calls: 0,
            solvent_activity_cache_hits: 0,
            warm_pe: None,
            engine_calls: 0,
            hook: None,
            neutralisation,
            last_heat_unpriced: None,
        })
    }

    /// A cache-only equilibrator: no engine, answers come from shipped
    /// results. This is the wasm/mobile path where a C++ library cannot be
    /// linked — and the honest failure mode is a stated cache miss, never a
    /// guess.
    #[cfg(not(feature = "engine"))]
    pub fn new() -> Result<Self, PhreeqcError> {
        Ok(PhreeqcEquilibrator {
            cache: std::collections::HashMap::new(),
            cache_hits: 0,
            trial_cache: std::collections::HashMap::new(),
            trial_cache_hits: 0,
            solvent_activity_engine_calls: 0,
            solvent_activity_cache_hits: 0,
            warm_pe: None,
            engine_calls: 0,
            hook: None,
            neutralisation: std::collections::HashMap::new(),
            last_heat_unpriced: None,
        })
    }

    /// Cache hits so far (content-addressed on the canonical PHREEQC input,
    /// which is a deterministic function of the vessel state).
    pub fn cache_hits(&self) -> usize {
        self.cache_hits
    }

    /// Raw engine invocations since construction. This is OPT-7's
    /// measurement: a coupled equilibration used to cost up to ~272 of
    /// these; the trial cache, the warm-started bracket and the residual
    /// break exist to shrink this number, and a test holds the budget.
    pub fn engine_calls(&self) -> usize {
        self.engine_calls
    }

    /// Bisection trials answered from the trial cache instead of the
    /// engine.
    pub fn trial_cache_hits(&self) -> usize {
        self.trial_cache_hits
    }

    /// Solvent-activity second opinions that reached the engine, and ones
    /// answered from the content-addressed cache.
    ///
    /// The pair is the measurement, not the first number alone. The lean
    /// problem this feature poses drops everything about a vessel that does
    /// not change its solution's composition, so two steps that differ only
    /// in how much undissolved solid is at the bottom ask the same solvent
    /// question and the second is free — and the ratio here is how anyone
    /// checks that claim instead of believing it.
    pub fn solvent_activity_solves(&self) -> (usize, usize) {
        (
            self.solvent_activity_engine_calls,
            self.solvent_activity_cache_hits,
        )
    }

    /// One bisection trial, answered from the trial cache when the exact
    /// same question was asked before. Only the coupled search comes
    /// through here: its trials are the repetitive traffic. The main
    /// content-addressed cache stays where it is — it caches *parsed*
    /// results one level up.
    fn run_trial(&mut self, db_tag: &str, input: &str) -> Result<Rc<SolveOutput>, SolveError> {
        let key = (db_tag.to_string(), input.to_string());
        if let Some(hit) = self.trial_cache.get(&key) {
            self.trial_cache_hits += 1;
            return Ok(Rc::clone(hit));
        }
        let out = Rc::new(self.run_raw(db_tag, input)?);
        if self.trial_cache.len() >= 2048 {
            self.trial_cache.clear();
        }
        self.trial_cache.insert(key, Rc::clone(&out));
        Ok(out)
    }

    /// Export the cache for shipping (PLAN.md: pre-warmed cache — every
    /// vessel state reachable in the curated lessons computed at build time,
    /// so guided content never waits for a solver, on any device).
    pub fn export_cache(&self) -> CacheData {
        let mut neutralisation_kj_per_mol: Vec<_> = self
            .neutralisation
            .iter()
            .map(|(database, enthalpy)| (database.clone(), *enthalpy))
            .collect();
        neutralisation_kj_per_mol.sort_by(|a, b| a.0.cmp(&b.0));
        CacheData {
            entries: self
                .cache
                .iter()
                .map(|(k, c)| CacheEntry {
                    key: k.clone(),
                    rows: c.rows.clone(),
                    species: c.speciation.clone(),
                    saturation: c.saturation.clone(),
                    redox_adjusted: c.redox_adjusted,
                    pe_undetermined: !c.pe_determined,
                    pe_authority: c.pe_authority,
                })
                .collect(),
            neutralisation_kj_per_mol,
        }
    }

    /// Load a pre-warmed cache. Entries already present are kept.
    pub fn import_cache(&mut self, data: CacheData) -> usize {
        let before = self.cache.len();
        for mut e in data.entries {
            // A serialized witness must agree with the old endpoint bit.
            // An absent witness is valid old JSON, but never proves pe.
            if (matches!(
                e.pe_authority,
                PeAuthority::CoupledRoot | PeAuthority::OpenOxygen
            ) && e.pe_undetermined)
                || (e.pe_authority == PeAuthority::CoupledUnpinned && !e.pe_undetermined)
            {
                continue;
            }
            // New canonical inputs demand numerical water activity on import
            // too. Legacy keys remain distinct and cannot answer these inputs.
            let numerical = e.key.lines().any(|line| line.trim() == "-activities H2O")
                || e.rows
                    .first()
                    .is_some_and(|header| header.iter().any(|name| name == "la_H2O"));
            if numerical && apply_numerical_water_activity(&mut e.species, &e.rows).is_err() {
                continue;
            }
            let census = e
                .key
                .lines()
                .any(|line| line.trim() == "-headings particle_molality")
                || e.rows
                    .first()
                    .is_some_and(|header| header.iter().any(|name| name == "particle_molality"));
            if census && numerical_particle_molality(&e.rows).is_err() {
                continue;
            }
            self.cache.entry(e.key).or_insert_with(|| {
                Rc::new(CachedSolve {
                    rows: e.rows,
                    speciation: e.species,
                    saturation: e.saturation,
                    redox_adjusted: e.redox_adjusted,
                    pe_determined: !e.pe_undetermined,
                    pe_authority: e.pe_authority,
                })
            });
        }
        for (database, enthalpy) in data.neutralisation_kj_per_mol {
            self.neutralisation.entry(database).or_insert(enthalpy);
        }
        self.cache.len() - before
    }

    /// How many results are cached.
    pub fn cache_len(&self) -> usize {
        self.cache.len()
    }

    #[cfg(feature = "engine")]
    /// Answer the same question from **every** dataset that can express it,
    /// so the paths can be compared rather than one being asserted
    /// (PLAN.md: offer different paths, be open about where each came from).
    ///
    /// Datasets that lack an element or a phase the problem needs are
    /// reported as such rather than silently skipped — a dataset declining
    /// to answer is itself information.
    pub fn compare_paths(&mut self, vessel: &Vessel) -> Vec<PathResult> {
        let Some(problem) = partition(vessel) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for db_tag in derived::DB_TAGS {
            let idx = derived::index_for(db_tag);
            let missing: Vec<String> = problem
                .elements
                .iter()
                .filter(|el| !idx.has_element(el))
                .cloned()
                .collect();
            if !missing.is_empty() {
                out.push(PathResult::new(
                    db_tag,
                    idx.activity_model,
                    PathOutcome::CannotExpress {
                        missing_elements: missing,
                    },
                ));
                continue;
            }
            let scoped = match comparison_problem(&problem, db_tag) {
                Ok(scoped) => scoped,
                Err(missing_phases) => {
                    out.push(PathResult::new(
                        db_tag,
                        idx.activity_model,
                        PathOutcome::CannotExpressPhases { missing_phases },
                    ));
                    continue;
                }
            };
            let input = build_input(vessel, &scoped, db_tag);
            let engine = match self.engine_for(db_tag, &input) {
                Ok(engine) => engine,
                Err(e) => {
                    out.push(PathResult::new(
                        db_tag,
                        idx.activity_model,
                        PathOutcome::Failed {
                            detail: e.to_string(),
                        },
                    ));
                    continue;
                }
            };
            let outcome = match engine
                .run(RESET_NUMBERED_REACTANTS)
                .and_then(|()| engine.run(&input))
            {
                Err(e) => PathOutcome::Failed {
                    detail: e.to_string(),
                },
                Ok(()) => {
                    let rows = engine.selected_output();
                    let value = |col: &str| -> Option<f64> {
                        let i = rows.first()?.iter().position(|h| h == col)?;
                        rows.last()?.get(i)?.parse().ok()
                    };
                    match (value("pH"), value("mu")) {
                        (Some(ph), Some(mu)) => PathOutcome::Solved {
                            ph,
                            ionic_strength: mu,
                            phases: scoped
                                .phases
                                .iter()
                                .filter_map(|(name, ..)| value(name).map(|m| (name.clone(), m)))
                                .collect(),
                        },
                        _ => PathOutcome::Failed {
                            detail: "expected columns missing from the result".to_string(),
                        },
                    }
                }
            };
            out.push(PathResult::new(db_tag, idx.activity_model, outcome));
        }
        out
    }
}

/// A comparison may omit unavailable zero-inventory precipitation candidates.
/// Positive supplied solids must use the same reviewed phase admission as the
/// direct solver, or the dataset must decline to compare this inventory.
#[cfg(any(feature = "engine", test))]
fn comparison_problem(problem: &Problem, db_tag: &str) -> Result<Problem, Vec<String>> {
    let mut scoped = problem.clone();
    scoped.phases.clear();
    let mut missing = Vec::new();
    // Preserve the direct solver's native-only domain for coupled redox and
    // typed surfaces. Comparison must not introduce an untracked solid pool.
    let native_only = redox_coupling_totals(&phase_inclusive_totals(problem), db_tag).is_some()
        || !problem.surfaces.is_empty();
    for (name, moles, si) in &problem.phases {
        let admitted = if native_only {
            derived::index_for(db_tag)
                .has_phase(name)
                .then_some(name.as_str())
        } else {
            posed_phase(name, db_tag)
        };
        if let Some(posed) = admitted {
            scoped.phases.push((posed.to_string(), *moles, *si));
        } else if *moles > 0.0 {
            missing.push(name.clone());
        }
    }
    if missing.is_empty() {
        Ok(scoped)
    } else {
        missing.sort();
        missing.dedup();
        Err(missing)
    }
}

#[derive(Clone)]
struct Problem {
    kgw: f64,
    /// True when PHREEQC is being asked only for the solvent's
    /// autoionisation state. Neutral or otherwise unrepresented spectators
    /// are deliberately absent from the input and must not be rebuilt as if
    /// the aqueous engine had modelled them.
    solvent_only: bool,
    /// Exact off-ledger CO2 parcel added to `totals[C]` by this pose.
    /// It is a clock boundary, not a native gas phase or a cached answer.
    clock_co2_applied_mol: f64,
    /// Element totals in solution, mol.
    totals: Vec<(String, f64)>,
    /// Phases: (name, initial moles, target saturation index).
    phases: Vec<(String, f64, f64)>,
    /// Finite-headspace components: PHREEQC phase, registry species, and
    /// initial partial pressure in atmospheres.
    gases: Vec<(String, String, f64)>,
    /// Gas phases owned by an external boundary rather than the vessel.
    /// They are still amount-limited in PHREEQC so an added dose can be
    /// consumed by the liquid instead of only appearing as vented gas.
    external_gases: Vec<ExternalGas>,
    /// Finite oxide interfaces. The first supported model is pooled into one
    /// PHREEQC SURFACE assemblage and split back onto these ledgers by each
    /// interface's share of strong and weak capacity.
    surfaces: Vec<SurfaceSites>,
    /// Finite cation exchangers, pooled for PHREEQC and split back by each
    /// named interface's share of total charge-equivalent capacity.
    exchanges: Vec<ExchangeSites>,
    /// Finite mixed crystalline phases. The initial reviewed slice permits
    /// one non-ideal aragonite-strontianite assemblage.
    solid_solutions: Vec<SolidSolution>,
    /// Every element to read back: dissolved totals plus the elements of all
    /// involved phases (a dissolving solid puts its elements into solution
    /// even when none started there).
    elements: Vec<String>,
}

/// Evaluate the water-dissociation relation shipped in PHREEQC's own
/// database without starting the native solver.
///
/// `phreeqc.dat` defines `H2O = OH- + H+` with the six-coefficient
/// temperature expression below. In a solvent-only problem electroneutrality
/// gives equal proton and hydroxide activities, so pH is exactly half pKw in
/// this ideal-dilute limit. A full engine call cannot add another degree of
/// freedom to that one-equation problem; it only makes routine water setup
/// hundreds of times more expensive.
fn water_autoionisation_log_k(temperature_k: f64) -> Option<f64> {
    if !temperature_k.is_finite() || temperature_k <= 0.0 {
        return None;
    }
    // vendor/iphreeqc/database/phreeqc.dat, aqueous species, OH-.
    const A: [f64; 6] = [
        293.29227,
        0.1360833,
        -10576.913,
        -123.73158,
        0.0,
        -6.996455e-5,
    ];
    let t = temperature_k;
    let log_k = A[0] + A[1] * t + A[2] / t + A[3] * t.log10() + A[4] / (t * t) + A[5] * t * t;
    log_k.is_finite().then_some(log_k)
}

/// A homogeneous analytical component input is not necessarily a complete
/// physical reagent. The legacy adapter charge-balances its non-H/O totals
/// through pH; never describe that convention as supplied counterions.
// Reporting resolution of the existing canonical readback, in mol of charge.
// Finite native gas/element convergence leaves nanomole-scale residuals; a
// neutral gas leaving can retain that residual after most ions have gone.
// This allowance changes neither inventory nor conservation acceptance.
const ANALYTICAL_CHARGE_READBACK_ALLOWANCE: f64 = 1e-8;

fn analytical_charge_note(vessel: &Vessel) -> Option<Event> {
    // These models own fixed/interfacial charge outside the bulk ledger.
    // Their admission and countercharge remain with their reviewed routes.
    if !vessel.surfaces.is_empty()
        || !vessel.exchanges.is_empty()
        || !vessel.electrodes.is_empty()
        || !vessel.contents.iter().any(|portion| {
            portion.species.0 == "water" && portion.phase == Phase::Liquid && portion.moles.0 > 0.0
        })
    {
        return None;
    }
    let mut charge = 0.0;
    let mut absolute_charge = 0.0;
    for portion in &vessel.contents {
        let Some(formula) = species::lookup(&portion.species)
            .and_then(|data| kerotakis_core::stoich::parse_formula(data.formula).ok())
        else {
            continue;
        };
        let contribution = formula.charge * portion.moles.0;
        charge += contribution;
        absolute_charge += contribution.abs();
    }
    // Current ionic magnitude alone is not a numerical error budget: opening
    // carbonated water vents neutral CO2 while retaining its earlier native
    // element/readback residual. Do not call that an uncompensated user dose.
    // Bare H/O input refusal remains separate and has no dose floor.
    if charge.abs() <= ANALYTICAL_CHARGE_READBACK_ALLOWANCE + absolute_charge * 1e-7 {
        return None;
    }
    Some(Event::not_modeled(
        vessel.id,
        kerotakis_core::ops::NotModelledCause::ModelBoundary,
        Phrase::bare(
            "not-modeled.analytical-component-charge",
            "the supplied inventory has uncompensated ionic charge. This adapter interprets non-H/O element totals as an analytical component problem and adjusts pH to impose electroneutrality; H/O basis entries are conservation coordinates, not prescribed free-ion concentrations. A reported pH or speciation therefore describes that proton-compensated analytical problem, not a prediction for a complete physical mixture of these ions. Supply the actual counterions or use complete acid, base or salt reagents; no sodium, chloride or other counterion is inferred",
        ),
    ))
}

fn has_solvent_only_basis(vessel: &Vessel) -> bool {
    vessel.contents.iter().any(|portion| {
        portion.phase == Phase::Aqueous
            && species::is_acid_base_basis(&portion.species.0)
            && portion.moles.0 > 0.0
    }) && partition(vessel).is_some_and(|problem| problem.solvent_only)
}

fn characterize_solvent_only(vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
    let Some(log_k) = water_autoionisation_log_k(vessel.temperature.0) else {
        return Err(SolveError::NotConverged {
            reason: Some(Box::new(AqueousReason::WaterRelationUndefined.phrase())),
            solver: "water-autoionisation".to_string(),
            detail: format!(
                "the water-dissociation relation is undefined at {:.6} K",
                vessel.temperature.0
            ),
        });
    };
    let ph = -0.5 * log_k;
    let activity = 10.0_f64.powf(-ph);
    let solvent_kg = partition(vessel).map(|problem| problem.kgw);

    // The provenance, composed before the record it goes into, because a
    // reader has to be told about it and the telling is an event.
    //
    // This path answers WITHOUT invoking IPhreeqc — it evaluates the
    // vendored water-dissociation relation directly — so its engine, its
    // dataset and its model are all different from the ones a solve
    // reports. That is exactly the kind of thing the routing line exists
    // to say, and until now it was written into `vessel.solution` and
    // never announced: the drawer reads `event.provenance`, and no event
    // carried this one. Ruled in by the owner on 2026-09-18.
    let provenance = Provenance::new(
        "Kerotakis analytic equilibrium evaluator",
        // "vendored USGS" is two English words in front of a file
        // name, and LV1 used to announce them AS the dataset, because
        // `dataset_file` could only take the first token. The name is
        // in the slot now, so the register that wants the file gets
        // `phreeqc.dat` and the register that wants the sentence gets
        // it in German.
        Phrase::new(
            "provenance.dataset.vendored-usgs",
            "vendored USGS {file}",
            vec![(
                Provenance::DATASET_FILE_SLOT.to_string(),
                Slot::text("phreeqc.dat"),
            )],
        ),
        Phrase::new(
            "provenance.model.water-autoionisation",
            "ideal-dilute water autoionisation ({engine} six-coefficient log K relation)",
            vec![("engine".to_string(), Slot::text("PHREEQC"))],
        ),
        vec!["USGS PHREEQC thermodynamic database".to_string()],
        Phrase::bare(
            "routing.solvent-relation-only",
            "no represented acid, base, salt, surface, exchanger, gas transfer, or reactive aqueous solute; evaluated the solvent relation without invoking IPhreeqc, while preserving all spectator inventory",
        ),
    );
    // Fire-on-change, the same rule and the same implementation the direct
    // solve uses. **Solvent-only characterisation is MOST
    // characterisations**, and this is what keeps that from becoming a
    // line on every step: a beaker of water is routed here every time the
    // stack runs, and the answer is the same every time, so it is said
    // once — when this vessel first becomes a solvent the lab has
    // characterised, and again only if it ever leaves this path and comes
    // back.
    let mut events = Vec::new();
    announce_routing(vessel, &provenance, &mut events);
    vessel.free_proton = activity * solvent_kg.unwrap_or(0.0);
    vessel.free_hydroxide = activity * solvent_kg.unwrap_or(0.0);
    vessel.solution = Some(SolutionInfo {
        solvent_activity: None,
        scope: SolutionScope::SolventOnly,
        solvent_kg,
        particle_molality: None,
        pe: None,
        redox: Vec::new(),
        ph,
        // For equal monovalent H+ and OH-, I = 1/2 (m + m) = m.
        ionic_strength: activity,
        species: vec![
            SpeciesDetail {
                name: "H+".to_string(),
                molality: activity,
                activity,
            },
            SpeciesDetail {
                name: "OH-".to_string(),
                molality: activity,
                activity,
            },
        ],
        provenance: Some(provenance),
    });
    vessel.refresh_pressure();
    // This is support state, not a reaction result. The ordinary honesty
    // rung runs after us and retains ownership of any spectator
    // diagnostics.
    //
    // **The list used to be empty, and the reason it could not stay
    // empty.** A `Provenance` was written into the vessel here and no
    // event carried it, so a beaker of plain water held a claim about
    // which relation answered it that no reader could reach. The routing
    // announcement is the only thing in this list and it fires on CHANGE.
    //
    // The old comment said an empty list is what stops coverage filing
    // routine water setup as a computed answer, and that was true while
    // the route's `event_count` was the raw length of this vector. It is
    // `solve::answer_event_count` now — a routing announcement says WHERE
    // an answer came from and is not one — so the property survives the
    // line: `chemistry_applies` stays false for a solvent-only vessel, and
    // the count this contributes stays zero.
    Ok(events)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ExternalGasKind {
    /// A boundary phase that starts outside the vessel (currently the
    /// atmospheric/swept sink used for outward transfer).
    Reservoir,
    /// A finite gas amount explicitly added through an external boundary.
    Dose,
}

#[derive(Clone)]
struct ExternalGas {
    phase: String,
    species: String,
    initial_moles: f64,
    kind: ExternalGasKind,
}

/// Partition the vessel into a PHREEQC problem, or None if this vessel is
/// not an aqueous problem this mapper fully understands.
/// Re-express a hopelessly concentrated problem as solid plus brine.
///
/// PHREEQC speciates the `SOLUTION` block *before* it looks at
/// `EQUILIBRIUM_PHASES`, so a beaker whose salt cannot possibly all be
/// dissolved is asked an impossible question first and never reaches the
/// step that would precipitate it. Evaporating brine to 1 mL hands the
/// database 100 mol/kgw nominal and all three refuse it — even though the
/// state they are being asked about, mostly solid beside a saturated
/// brine, is comfortably inside pitzer's range.
///
/// The same equilibrium can be posed the other way round: put most of the
/// salt in as solid and let it dissolve to saturation. Same elements, same
/// answer, and a starting point the engine can actually speciate. The
/// probe that settled it: 0.1 mol of NaCl in 1 mL fails as totals and
/// solves as `Halite 0 9.46e-2` beside `Na 5.0`, returning I = 6.13 m.
///
/// Only reached after a failure, so nothing that already solves is
/// disturbed by it.
fn condense_supersaturated(problem: &Problem) -> Option<Problem> {
    /// Past every shipped model's domain — pitzer, the widest, is good to
    /// about 6 mol/kgw — so nothing that could have solved trips this.
    const TRIGGER: f64 = 12.0;
    /// Where to leave the solution: high enough to stay saturated, low
    /// enough to speciate.
    const TARGET: f64 = 5.0;

    if problem.kgw <= 0.0 {
        return None;
    }
    if !problem
        .totals
        .iter()
        .any(|(_, n)| n / problem.kgw > TRIGGER)
    {
        return None;
    }

    let mut out = problem.clone();
    let ceiling = TARGET * problem.kgw;
    // Allocations are computed against a running copy of the totals so two
    // phases sharing an element cannot each claim all of it.
    let mut moved: Vec<(String, f64)> = Vec::new();
    for (name, _, _) in &out.phases {
        let Some(dp) = derived::phase_by_name(name) else {
            continue;
        };
        if dp.elements.is_empty() || dp.waters != 0.0 {
            continue;
        }
        let take = dp
            .elements
            .iter()
            .map(|(el, coeff)| {
                let have = out
                    .totals
                    .iter()
                    .find(|(e, _)| e == el)
                    .map(|(_, n)| *n)
                    .unwrap_or(0.0);
                ((have - ceiling) / coeff).max(0.0)
            })
            .fold(f64::INFINITY, f64::min);
        if take.is_finite() && take > 0.0 {
            for (el, coeff) in &dp.elements {
                if let Some(entry) = out.totals.iter_mut().find(|(e, _)| e == el) {
                    entry.1 -= take * coeff;
                }
            }
            moved.push((name.clone(), take));
        }
    }
    if moved.is_empty() {
        return None;
    }
    for (name, take) in moved {
        if let Some(entry) = out.phases.iter_mut().find(|(n, ..)| *n == name) {
            entry.1 += take;
        }
    }
    Some(out)
}

/// Below this much solvent, "mol per kilogram of water" is a ratio against
/// a residue rather than a statement about a solution.
///
/// One millilitre. It is the smallest volume the bench's own glassware
/// graduates, and it is deliberately the largest floor that disturbs
/// nothing already written down here: `condense_supersaturated`'s
/// documented probe is 0.1 mol of NaCl in exactly 1 mL, and that case has
/// to keep handing the router the same 100 mol/kgw it always did.
const MIN_SOLVENT_KG: f64 = 1e-3;

/// What one solid phase could put into solution, in moles, capped by what
/// `kgw` of water could actually hold.
///
/// `None` where the registry has reviewed no aqueous solubility for the
/// solid this phase is booked as — the caller then keeps the old
/// pessimism for it. See [`potential_molality`] for why that asymmetry is
/// deliberate.
fn saturation_moles(phase: &str, kgw: f64, temperature_k: f64) -> Option<f64> {
    let derived_phase = derived::phase_by_name(phase)?;
    let data = species::lookup_key(derived_phase.species)?;
    let g_per_100_ml = data.aqueous_solubility_at(temperature_k)?;
    if !(data.molar_mass.is_finite() && data.molar_mass > 0.0) {
        return None;
    }
    // g/100 mL → g/L → mol/L, read as mol/kgw. The two differ by the
    // solution's density, and a routing estimate that is already rounded
    // to "is this above 1 molal" does not need that correction.
    let saturation_molal = g_per_100_ml * 10.0 / data.molar_mass;
    Some(saturation_molal * kgw)
}

/// A rough ceiling on the molality of the solution PHREEQC is about to be
/// handed: what is dissolved, plus what the solid phases could dissolve
/// (each formula unit taken as about two ions).
///
/// One number with two jobs. It SELECTS the dataset — above 1 mol/kgw the
/// Debye-Hückel datasets are out of their validity domain, so the router
/// reaches for pitzer, or, when pitzer cannot hold the chemistry, for the
/// `routing.activity-model-fallback` caveat — and it is also READ ALOUD to
/// the reader inside that caveat. Which is why it is computed once, here:
/// a bound applied to the printed copy alone would leave the sentence
/// agreeing with the reader and disagreeing with the engine, which is the
/// same defect one level down.
///
/// Two bounds, and each one is here because of a live transcript
/// (2026-09-18) in which a vessel that had boiled dry printed
/// *~71046.6 mol/kgw* one line above the solver's own *I = 0.0004
/// mol/kgw*. Two faults compounded to produce those eight orders of
/// magnitude:
///
/// * **The denominator was an evaporating solvent.** A beaker boiling dry
///   keeps its solid inventory and loses its water, so a fixed numerator
///   was divided by a residue — about a milligram of it. The solvent mass
///   is floored at [`MIN_SOLVENT_KG`].
///
///   *What that gives up:* below a millilitre of water this estimate now
///   UNDER-states concentration, so a genuinely two-molal microlitre
///   droplet would be routed to the default dataset rather than to
///   pitzer. That is the safer direction of the two — the fallback caveat
///   says of itself that it is not a validated concentrated-mixture
///   prediction — and a millilitre is small enough that the brines the
///   pitzer route exists for, which carry moles of salt rather than
///   millimoles, still clear 1.0 by orders of magnitude.
///
/// * **The numerator counted solids that will never dissolve.** Every
///   equilibrium phase contributed two ions per formula unit. That vessel
///   held chalk, whose own `Event::Inert` sentence three lines earlier
///   said it dissolves to 0.0013 g per 100 mL, "below anything a beaker
///   would show". Pessimism is defensible; two halves of one answer
///   contradicting each other is not. A phase now contributes at most
///   what the water present could hold, read from the SAME reviewed
///   solubility that composes that sentence, so the two cannot disagree
///   about the same solid again.
///
///   *What that gives up:* a solid for which the registry has reviewed NO
///   solubility is still counted in full, deliberately. That is what
///   keeps halite and sylvite routing a real brine to pitzer, which is
///   the case that route exists for, and it is the conservative reading
///   of "could dissolve". The price is that an insoluble solid the
///   registry has not reviewed — manganese dioxide and silver chloride
///   are both in that transcript — still inflates the estimate. Closing
///   that is registry data with a source behind it, not arithmetic here.
fn potential_molality(problem: &Problem, temperature_k: f64) -> f64 {
    let kgw = problem.kgw.max(MIN_SOLVENT_KG);
    let dissolved: f64 = problem.totals.iter().map(|(_, n)| n).sum();
    let from_phases: f64 = problem
        .phases
        .iter()
        .map(|(name, moles, _)| {
            2.0 * saturation_moles(name, kgw, temperature_k).map_or(*moles, |cap| moles.min(cap))
        })
        .sum();
    // A mixed crystal is a reviewed assemblage of end members rather than
    // one registry solid, so there is no single solubility to cap it with
    // and it keeps the pessimistic reading.
    let from_solid_solutions: f64 = problem
        .solid_solutions
        .iter()
        .map(|solid_solution| 2.0 * solid_solution.total_moles().0)
        .sum();
    (dissolved + from_phases + from_solid_solutions) / kgw
}

/// Every derived candidate phase whose elements can reach solution, for a
/// beaker holding `elements`.
///
/// Split out of `partition` because the element set a candidate list is
/// derived over is not always one vessel's. Two solutions combined by
/// fraction make a beaker whose elements only meet on mixing, and the
/// solid that pair can grow — Halite, from an iron *chloride* poured into
/// a *sodium* hydroxide — is proposed by neither source list. The MIX
/// path derives over the merged set through the same rule rather than
/// unioning two answers to a different question.
fn append_candidate_phases(
    vessel: &Vessel,
    elements: &[String],
    phases: &mut Vec<(String, f64, f64)>,
) {
    // Every derived candidate phase whose elements can reach solution can
    // precipitate, amount 0 if no solid exists yet.
    //
    // Whether a redox conversion is available: two or more FAST_REDOX
    // elements means the coupled pe solve owns electron transfers between
    // them (mirrors `redox_coupling`); a lone one keeps the state it was
    // added in.
    let redox_partnered = {
        let mut bases: Vec<&str> = elements
            .iter()
            .map(|e| e.split('(').next().unwrap_or(e))
            .filter(|b| FAST_REDOX.contains(b))
            .collect();
        bases.sort_unstable();
        bases.dedup();
        bases.len() >= 2
    };
    for cand in derived::candidate_phases() {
        // A mixed crystal and a pure phase cannot both own the same mineral
        // formula in this first slice. PHREEQC's documented example poses
        // CaCO3 through the aragonite end member; allowing the normal
        // calcite candidate beside it would silently move calcium carbonate
        // out of the typed solid-solution ledger.
        let owned_by_solid_solution = vessel.solid_solutions.iter().any(|solid_solution| {
            solid_solution
                .components
                .iter()
                .any(|component| component.component.species().0 == cand.species)
        });
        if owned_by_solid_solution {
            continue;
        }
        // The typed HFO surface model books zinc as dissolved or bound —
        // its transport ledger and the PHREEQC oracle it is verified
        // against model sorption only. Zinc hydroxide (a candidate since
        // EXP-30) precipitating mid-column put 6.1e-6 mol of zinc in a
        // third pool that ledger does not carry. A reviewed scope
        // statement, not chemistry: the surface lessons run where the
        // hydroxide barely matters, and a surface-plus-precipitation
        // model would need its own review.
        if !vessel.surfaces.is_empty() && cand.species == "Zn(OH)2" {
            continue;
        }
        // Compare *base* elements. Once a dissolved species carries its
        // oxidation state — copper(II) is "Cu(2)", not "Cu" — a phase whose
        // elements are written plainly stops matching, and copper hydroxide
        // silently left the candidate list: 0.01 mol of Cu(2+) sat at
        // pH 9.9 in a solution that cannot exist.
        let all_present = cand.elements.iter().all(|(el, _)| {
            let want = el.split('(').next().unwrap_or(el);
            elements
                .iter()
                .any(|e| e.split('(').next().unwrap_or(e) == want)
        });
        // An element added in one oxidation state does not reach a phase
        // that holds it in another unless a redox partner is present — an
        // uncoupled element keeps its state (see FAST_REDOX), and posing a
        // ferric solid against ferrous totals invites PHREEQC's reaction
        // step to run the very conversion that curation stood down: iron(II)
        // plus lye precipitated iron(III) hydroxide with no oxidant in the
        // beaker.
        let state_reachable = cand.elements.iter().all(|(el, _)| {
            let base = el.split('(').next().unwrap_or(el);
            let Some(cand_state) = tagged_state(el) else {
                return true;
            };
            if redox_partnered && FAST_REDOX.contains(&base) {
                return true;
            }
            elements
                .iter()
                .filter(|e| e.split('(').next().unwrap_or(e) == base)
                .any(|e| match tagged_state(e) {
                    Some(s) => s == cand_state,
                    // A bare key books at a curated charge ("when we say
                    // iron, we mean iron(II)"); undeterminable states stay
                    // permissive rather than silently veto a phase.
                    None => derived::booking_ion(base)
                        .and_then(|ion| kerotakis_core::stoich::parse_formula(ion).ok())
                        .map(|f| f.charge as i32 == cand_state)
                        .unwrap_or(true),
                })
        });
        let listed = phases.iter().any(|(name, ..)| name == &cand.name);
        // A phase with a kinetic barrier is withheld below its threshold.
        // Equilibrium alone would hand back the *stable* copper solid,
        // tenorite, and a beaker of copper sulfate and lye visibly gives
        // the metastable blue hydroxide instead. Which phase is reachable
        // is a statement about rates, so it lives in the registry as data
        // rather than as a branch in this solver.
        let reachable = species::lookup_key(cand.species)
            .and_then(|d| d.forms_only_above_k)
            .is_none_or(|floor| vessel.temperature.0 >= floor);
        if all_present && state_reachable && !listed && reachable {
            phases.push((cand.name.clone(), 0.0, 0.0));
        }
    }
}

/// Element coverage alone cannot express ligand coverage: wateq4f knows
/// both Ag and N(-3), but lacks silver ammines. Nitrate N(5) is not ammonia.
fn needs_silver_ammine(elements: &[String]) -> bool {
    elements.iter().any(|element| element == "Ag")
        && elements.iter().any(|element| element == "N(-3)")
}

fn partition(vessel: &Vessel) -> Option<Problem> {
    let mut kgw = 0.0;
    let mut totals: Vec<(String, f64)> = Vec::new();
    let mut phases: Vec<(String, f64, f64)> = Vec::new();
    let mut gases: Vec<(String, String, f64)> = Vec::new();
    let mut external_gases: Vec<ExternalGas> = Vec::new();
    let mut solutes = 0;
    let mut clock_co2_applied_mol = 0.0;

    let mut add_total = |el: &str, moles: f64| {
        if let Some(entry) = totals.iter_mut().find(|(e, _)| e == el) {
            entry.1 += moles;
        } else {
            totals.push((el.to_string(), moles));
        }
    };

    let mut elements: Vec<String> = Vec::new();
    let mut note_element = |el: &str| {
        if !elements.iter().any(|e| e == el) {
            elements.push(el.to_string());
        }
    };

    // Equivalent inventories must pose byte-identical nonlinear problems.
    // Vessel contents retain learner feed order, which is meaningful for the
    // narrative but not for an equilibrium state function. Accumulating and
    // emitting in that order changed PHREEQC's iteration path at trace scale.
    let mut contents: Vec<_> = vessel.contents.iter().collect();
    contents.sort_by(|a, b| {
        a.species
            .0
            .cmp(&b.species.0)
            .then_with(|| a.phase.cmp(&b.phase))
            .then_with(|| a.moles.0.total_cmp(&b.moles.0))
    });
    for p in contents {
        if p.phase == Phase::Gas {
            // Explicit gas boundaries are handled before analytical solutes.
            if let Some(volume) = vessel.headspace_volume() {
                // Only gases whose liquid exchange this adapter explicitly
                // supports enter PHREEQC. Nitrogen and oxygen are retained
                // as inert pressure/mass inventory: unrestricted equilibrium
                // turns room air into nitrate, a thermodynamic endpoint that
                // is kinetically impossible on a bench. Reviewed reactive
                // gases use the database's own dissolution equilibrium.
                let gas_phase = if p.species.0 == "NH3" {
                    Some("NH3(g)")
                } else {
                    EQUILIBRIUM_GASES
                        .iter()
                        .find(|(_, species)| *species == p.species.0)
                        .map(|(phase, _)| *phase)
                };
                let Some(phase) = gas_phase else {
                    continue;
                };
                // This is a conversion into PHREEQC's input convention,
                // not the lab's physical gas constant. Its native initializer
                // divides PV by R_LITER_ATM=0.0820597 (global_structures.h).
                // Using CODATA here silently changes supplied gas moles.
                const NATIVE_R_LITRE_ATM: f64 = 0.082_059_7;
                // A registry display formula may include a phase suffix
                // (NH3(aq)); it is not a native database phase identifier.
                // Use the reviewed capability mapping for supplied gas just
                // as we do for an initially empty gas component below.
                let partial_pressure =
                    p.moles.0 * NATIVE_R_LITRE_ATM * vessel.temperature.0 / volume.0;
                gases.push((phase.to_string(), p.species.0.clone(), partial_pressure));
                if p.species.0 == "NH3" {
                    // Gas and aqueous ammonia share reduced nitrogen. Bare
                    // N would admit nitrate/redox conversion of the dose.
                    note_element("N(-3)");
                } else if let Some(formula) =
                    crate::dbindex::parse_formula(phase.trim_end_matches("(g)"))
                {
                    for element in formula
                        .keys()
                        .filter(|element| *element != "H" && *element != "O")
                    {
                        note_element(element);
                    }
                }
                solutes += 1;
            } else if let Some((phase, species)) = EQUILIBRIUM_GASES
                .iter()
                .find(|(_, species)| *species == p.species.0)
            {
                // A gas explicitly added to an external boundary is a
                // finite dose passing through the liquid. Pure CO2 sets SI
                // 0 while it is available; whatever remains then vents.
                match external_gases
                    .iter_mut()
                    .find(|exchange| exchange.phase == *phase)
                {
                    Some(exchange) => exchange.initial_moles += p.moles.0,
                    None => external_gases.push(ExternalGas {
                        phase: phase.to_string(),
                        species: species.to_string(),
                        initial_moles: p.moles.0,
                        kind: ExternalGasKind::Dose,
                    }),
                }
                let gas_formula = phase.trim_end_matches("(g)");
                if let Some(formula) = crate::dbindex::parse_formula(gas_formula) {
                    for element in formula
                        .keys()
                        .filter(|element| *element != "H" && *element != "O")
                    {
                        note_element(element);
                    }
                }
                solutes += 1;
            }
            continue;
        }
        // A species this engine cannot place must not take the rest of the
        // vessel down with it. Bailing out here meant that adding one salt
        // PHREEQC has never heard of — thiosulfate, say — silently withdrew
        // the pH of the acid sitting beside it, and the vessel simply
        // stopped having a solution at all. The honesty pass names the
        // unmodelled substance; the acid still gets its answer.
        // Crystals of a curve salt deposited onto an already saturated
        // solution belong to the crystallisation ledger
        // (`kerotakis_core::crystallisation`); entering them as a freely
        // dissolving solid would undo the crystallisation on this step.
        if p.phase == Phase::Solid
            && kerotakis_core::crystallisation::holds_solid(vessel, &p.species.0)
        {
            continue;
        }
        let Some(role) = derived::role(&p.species.0) else {
            continue;
        };
        match role {
            // Ice is a separate, pure compartment. Only liquid solvent
            // defines the solution mass and its molalities.
            DerivedRole::Solvent if p.phase == Phase::Liquid => {
                kgw += p.moles.0 * WATER_MOLAR_MASS / 1000.0
            }
            DerivedRole::Solvent => {}
            DerivedRole::Dissolves(_) if species::is_acid_base_basis(&p.species.0) => {}
            DerivedRole::Dissolves(els) => {
                solutes += 1;
                for (el, coeff) in els {
                    add_total(el, p.moles.0 * coeff);
                    note_element(el);
                }
            }
            DerivedRole::Mineral {
                phase,
                elements: els,
            } => {
                solutes += 1;
                for (el, _) in els {
                    note_element(el);
                }
                if p.phase == Phase::Solid {
                    if let Some(entry) = phases.iter_mut().find(|(name, ..)| name == phase) {
                        entry.1 += p.moles.0;
                    } else {
                        phases.push((phase.clone(), p.moles.0, 0.0));
                    }
                } else {
                    for (el, coeff) in els {
                        add_total(el, p.moles.0 * coeff);
                    }
                }
            }
        }
    }

    for surface in &vessel.surfaces {
        solutes += 1;
        // SURFACE is reconstructed from its neutral Hfo_*OH reference on
        // every PHREEQC pass. Return water previously released by ligand
        // exchange to that interface reference before solving again, just
        // as the bound sorbates below are returned to the element totals.
        kgw -= surface.water_release.0 * WATER_MOLAR_MASS / 1000.0;
        for occupied in &surface.occupancy {
            let sorbate = occupied.sorbate.species();
            let Some(DerivedRole::Dissolves(els)) = derived::role(&sorbate.0) else {
                continue;
            };
            for (el, coeff) in els {
                add_total(el, occupied.moles.0 * coeff);
                note_element(el);
            }
        }
    }

    for exchange in &vessel.exchanges {
        solutes += 1;
        // Unlike a SURFACE, an EXCHANGE assemblage can be reconstructed from
        // its bound complexes directly (`NaX`, `CaX2`, ...). Do not return
        // these cations to SOLUTION totals as well or they would be counted
        // twice. They are still noted so dissolved readback and candidate
        // phases include every material element the exchanger can release.
        for occupied in &exchange.occupancy {
            if occupied.ion == ExchangeIon::Hydrogen {
                continue;
            }
            let ion = occupied.ion.species();
            let Some(DerivedRole::Dissolves(els)) = derived::role(&ion.0) else {
                continue;
            };
            for (element, _) in els {
                note_element(element);
            }
        }
    }

    for solid_solution in &vessel.solid_solutions {
        solutes += 1;
        // SOLID_SOLUTIONS owns these formula units directly. Do not also
        // return them to SOLUTION totals or a pure EQUILIBRIUM_PHASE.
        for component in &solid_solution.components {
            for element in solid_solution_component_elements(component.component) {
                note_element(element);
            }
        }
    }

    // EXP-57: the carbon the gas-exchange clock decided crossed the
    // surface this step, applied to the ELEMENT TOTALS.
    //
    // The totals are the one representation both halves of this adapter
    // already agree on: the input writes them into `SOLUTION`, and the
    // readback books every dissolved element back through
    // `derived::booking_ion`. Carbon that arrived from the room is
    // therefore speciated and KEPT, with no new machinery on either side.
    //
    // Both of the more obvious routes were tried first and both failed
    // silently, which is why they are written down:
    //
    //   * A PHREEQC `REACTION` block. The engine accepted it and speciated
    //     it correctly, and the readback then dropped every atom — that
    //     rebuild walks the vessel's input PORTIONS, so an element with no
    //     portion of its own has nowhere to be booked. Measured: a beaker
    //     of hydroxide took up 7.6e-4 mol over thirty simulated days,
    //     announced every parcel, and came back holding water and sodium.
    //     The events and the vessel disagreed, and the vessel is what the
    //     next step reads.
    //   * Depositing a `CO2` portion into the ledger. `role("CO2")` is
    //     deliberately `None` — carbon dioxide is a GAS to this bench, not
    //     a dissolved solute — so the portion contributed nothing to the
    //     totals and never reached the engine at all.
    //
    // Outward transfer is clamped to the carbon actually present: a
    // solution cannot give back more than it holds, and an element total
    // driven negative is not a small error underneath PHREEQC.
    if vessel.pending_co2_transfer_mol.abs() > kerotakis_core::clock::CO2_TRANSFER_FLOOR_MOL {
        let carbon_now: f64 = totals
            .iter()
            .filter(|(el, _)| el == "C" || el.starts_with("C("))
            .map(|(_, moles)| *moles)
            .sum();
        let applied = vessel.pending_co2_transfer_mol.max(-carbon_now);
        clock_co2_applied_mol = applied;
        if applied != 0.0 {
            match totals.iter_mut().find(|(el, _)| el == "C") {
                Some(entry) => entry.1 += applied,
                None => totals.push(("C".to_string(), applied)),
            }
            if !elements.iter().any(|e| e == "C") {
                elements.push("C".to_string());
            }
            // A beaker of plain water holds no solutes at all, and the
            // refusal below would decline to solve it — so carbon the room
            // is delivering has to count as one, or the vessel that needs
            // this most is the one vessel that never gets a solve.
            solutes += 1;
        }
    }

    // Unrepresented acids and ionic solutes must never inherit pure water's
    // pH. Neutral spectators may: they do not contribute to the acid/base
    // balance, and the solvent's autoionisation is still a real solution
    // state. `Problem::solvent_only` keeps that deliberately partial solve
    // from rebuilding or silencing the unrepresented material.
    if kgw <= 0.0
        || (solutes == 0 && (holds_unspeciated_acid(vessel) || holds_unspeciated_solute(vessel)))
    {
        return None;
    }
    let solvent_only = solutes == 0;
    append_candidate_phases(vessel, &elements, &mut phases);
    if vessel.owns_headspace_gas() {
        // A finite headspace must admit gases that can form from the
        // solution even when none was initially present.
        for (phase, species) in EQUILIBRIUM_GASES {
            let gas_formula = phase.trim_end_matches("(g)");
            let required = crate::dbindex::parse_formula(gas_formula).unwrap_or_default();
            let all_present = required
                .keys()
                .filter(|el| *el != "O" && *el != "H")
                .all(|el| elements.iter().any(|e| e == el));
            if all_present && !gases.iter().any(|(name, ..)| name == phase) {
                gases.push((phase.to_string(), species.to_string(), 0.0));
            }
        }
        // Couple finite NH3 to the SAME native solve as NH3/NH4+
        // protonation. Core Henry partitioning runs before this aqueous
        // tail and remains the fallback without a native wet solver; it
        // cannot determine acid trapping on its own. This capability is
        // finite-only: it introduces no open/swept ammonia reservoir.
        if elements.iter().any(|element| element == "N(-3)")
            && !gases.iter().any(|(_, species, _)| species == "NH3")
        {
            gases.push(("NH3(g)".to_string(), "NH3".to_string(), 0.0));
        }
    } else {
        // Explicit finite doses are admitted by the reviewed gas capability,
        // not by atmospheric abundance. Each is capped by supplied moles;
        // no persistent reservoir is created for an absent atmospheric gas.
        for exchange in &external_gases {
            phases.push((exchange.phase.clone(), exchange.initial_moles, 0.0));
        }
        // Reservoir boundaries do not own a gas inventory. An open vessel
        // sees room-air partial pressures; an inert nitrogen sweep drives
        // volatile products toward a near-zero partial pressure.
        let target = match vessel.headspace {
            Headspace::Open => None,
            Headspace::Swept { .. } => Some(SWEPT_LOG_PARTIAL_PRESSURE),
            _ => unreachable!("finite gas boundaries handled above"),
        };
        for (phase, species, target_si) in ATMOSPHERIC {
            if let Some(exchange) = external_gases
                .iter()
                .find(|exchange| exchange.phase == *phase)
            {
                let _ = exchange; // the finite dose was added above
                continue;
            }
            // EXP-57 changed NOTHING here, and the reader is owed that
            // plainly, because the direction of travel invites the
            // opposite reading.
            //
            // This phase is offered holding ZERO moles. PHREEQC can
            // precipitate into it and cannot dissolve from it, so for an
            // open vessel it is a one-way valve: carbon above the room's
            // partial pressure leaves within the step, and carbon below it
            // does nothing at all. That outward half stays equilibrium,
            // exactly as it was — see `GasExchangeClock::advance` for why
            // making it a rate needs a decision about fizz that this
            // change does not take.
            //
            // What EXP-57 adds is the INWARD half, and it does not come
            // through this phase, because it cannot: uptake needs a source
            // of moles and this has none. `GasExchangeClock` sizes it and
            // the element-totals block above delivers it.
            let gas_formula = phase.trim_end_matches("(g)");
            let required = crate::dbindex::parse_formula(gas_formula).unwrap_or_default();
            let all_present = required
                .keys()
                .filter(|el| *el != "O" && *el != "H")
                .all(|el| elements.iter().any(|e| e == el));
            let listed = phases.iter().any(|(name, ..)| name == phase);
            // EXP-57: the gate STAYS, and the reason is worth recording
            // because the task that led here asked for it to go.
            //
            // This phase carries zero moles, so PHREEQC can only ever take
            // carbon OUT through it — and carbon that is not dissolved
            // cannot be removed. Offering it to a vessel with no carbon
            // buys nothing and costs a great deal: `SOLUTION` is posed with
            // `pH 7 charge`, so pH is the charge-balancing variable, and
            // handing it a carbon reservoir to balance against moved 0.1 M
            // ammonium chloride from pH 5.2 to 3.54 and lost nitrogen
            // across the ammonia/ammonium split.
            //
            // What the gate was blocking was UPTAKE, and uptake no longer
            // comes through this phase at all: `GasExchangeClock` sizes it
            // and `partition` adds it to the element totals above. The
            // clock asks no question about carbon being present — a
            // solution holding none simply has the largest driving force
            // there is — so the beaker of hydroxide is answered without
            // touching this.
            if all_present && !listed {
                phases.push((phase.to_string(), 0.0, target.unwrap_or(*target_si)));
                external_gases.push(ExternalGas {
                    phase: phase.to_string(),
                    species: species.to_string(),
                    initial_moles: 0.0,
                    kind: ExternalGasKind::Reservoir,
                });
            }
        }
    }
    totals.sort_by(|a, b| a.0.cmp(&b.0));
    phases.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| a.1.total_cmp(&b.1))
            .then_with(|| a.2.total_cmp(&b.2))
    });
    gases.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| a.1.cmp(&b.1))
            .then_with(|| a.2.total_cmp(&b.2))
    });
    external_gases.sort_by(|a, b| {
        a.phase
            .cmp(&b.phase)
            .then_with(|| a.species.cmp(&b.species))
            .then_with(|| a.initial_moles.total_cmp(&b.initial_moles))
    });
    elements.sort();

    Some(Problem {
        kgw,
        solvent_only,
        clock_co2_applied_mol,
        totals,
        phases,
        gases,
        external_gases,
        surfaces: vessel.surfaces.clone(),
        exchanges: vessel.exchanges.clone(),
        solid_solutions: vessel.solid_solutions.clone(),
        elements,
    })
}

/// Split the pooled PHREEQC surface result back across identical physical
/// interfaces in proportion to the capacity each contributed. The solver
/// sees one thermodynamic HFO assemblage; the vessel keeps ownership and
/// finite capacity on each named interface.
fn distribute_surface_occupancy(
    surfaces: &mut [SurfaceSites],
    site: SurfaceSiteKind,
    sorbate: SurfaceSorbate,
    total_moles: f64,
) {
    let total_capacity: f64 = surfaces
        .iter()
        .map(|surface| surface.capacity(site).0)
        .sum();
    if total_capacity <= 0.0 || total_moles <= TRACE {
        return;
    }
    for surface in surfaces {
        let share = total_moles * surface.capacity(site).0 / total_capacity;
        if share > TRACE {
            surface.occupancy.push(SurfaceOccupancy {
                site,
                sorbate,
                moles: Moles(share),
            });
        }
    }
}

/// Return a site-reaction water transfer to the physical interfaces that
/// supplied those pooled sites. Positive values are water released into the
/// solution and therefore mass removed from the interface reference state.
fn distribute_surface_water_release(
    surfaces: &mut [SurfaceSites],
    site: SurfaceSiteKind,
    total_moles: f64,
) {
    let total_capacity: f64 = surfaces
        .iter()
        .map(|surface| surface.capacity(site).0)
        .sum();
    if total_capacity <= 0.0 || total_moles <= TRACE {
        return;
    }
    for surface in surfaces {
        surface.water_release.0 += total_moles * surface.capacity(site).0 / total_capacity;
    }
}

/// Return one pooled exchanger complex to each named physical exchanger in
/// proportion to the charge-equivalent capacity it contributed.
fn distribute_exchange_occupancy(
    exchanges: &mut [ExchangeSites],
    ion: ExchangeIon,
    total_moles: f64,
) {
    let total_capacity: f64 = exchanges.iter().map(|exchange| exchange.capacity.0).sum();
    if total_capacity <= 0.0 || total_moles <= TRACE {
        return;
    }
    for exchange in exchanges {
        let share = total_moles * exchange.capacity.0 / total_capacity;
        if share > TRACE {
            exchange.occupancy.push(ExchangeOccupancy {
                ion,
                moles: Moles(share),
            });
        }
    }
}

fn phreeqc_exchange_species(ion: ExchangeIon) -> &'static str {
    match ion {
        ExchangeIon::Hydrogen => "HX",
        ExchangeIon::Sodium => "NaX",
        ExchangeIon::Calcium => "CaX2",
        ExchangeIon::Magnesium => "MgX2",
    }
}

fn phreeqc_solid_solution_component(component: SolidSolutionComponent) -> &'static str {
    match component {
        SolidSolutionComponent::CalciumCarbonate => "Aragonite",
        SolidSolutionComponent::StrontiumCarbonate => "Strontianite",
    }
}

fn solid_solution_component_elements(component: SolidSolutionComponent) -> &'static [&'static str] {
    match component {
        SolidSolutionComponent::CalciumCarbonate => &["Ca", "C"],
        SolidSolutionComponent::StrontiumCarbonate => &["Sr", "C"],
    }
}

// These are adapter acceptance ceilings, not validated chemical accuracy.
const CRYSTAL_NATIVE_RELATIVE_CAP: f64 = 1e-7;
const CRYSTAL_OUTPUT_RELATIVE_CAP: f64 = 5e-12;

#[derive(Clone, Copy)]
enum CrystalReason {
    AmountInvalid,
    SolventMassInvalid,
    AmountOverflow,
    ReductionUnsupported,
    SurfaceExchangeCoowner,
    InventoryInvalid,
    ExternalPhaseUnknown,
    ExternalBoundaryUnsupported,
    PrimaryPhaseUnknown,
    PhaseOverdraw,
    RemainderInvalid,
    AqueousInventoryMissing,
    AqueousResidualUnexplained,
}

impl CrystalReason {
    fn phrase(self) -> Phrase {
        let (key, en) = match self {
            Self::AmountInvalid => (
                "solver-reason.aqueous.crystal-amount-invalid",
                "A mixed-crystal amount is invalid.",
            ),
            Self::SolventMassInvalid => (
                "solver-reason.aqueous.crystal-solvent-mass-invalid",
                "The aqueous solvent mass is non-finite or non-positive.",
            ),
            Self::AmountOverflow => (
                "solver-reason.aqueous.crystal-amount-overflow",
                "A mixed-crystal amount sum overflowed.",
            ),
            Self::ReductionUnsupported => (
                "solver-reason.aqueous.crystal-reduction-unsupported",
                "The mixed-crystal arithmetic reduction is unsupported.",
            ),
            Self::SurfaceExchangeCoowner => (
                "solver-reason.aqueous.crystal-interface-coowner",
                "Surface or exchange ownership lacks an aqueous witness.",
            ),
            Self::InventoryInvalid => (
                "solver-reason.aqueous.crystal-inventory-invalid",
                "The typed crystal inventory is invalid.",
            ),
            Self::ExternalPhaseUnknown => (
                "solver-reason.aqueous.crystal-external-phase-unknown",
                "An external phase has no reviewed formula.",
            ),
            Self::ExternalBoundaryUnsupported => (
                "solver-reason.aqueous.crystal-external-boundary-unsupported",
                "An external phase lacks a reviewed element boundary.",
            ),
            Self::PrimaryPhaseUnknown => (
                "solver-reason.aqueous.crystal-primary-phase-unknown",
                "A primary phase has no reviewed formula.",
            ),
            Self::PhaseOverdraw => (
                "solver-reason.aqueous.crystal-phase-overdraw",
                "The final owned phase inventory exceeds the available amount.",
            ),
            Self::RemainderInvalid => (
                "solver-reason.aqueous.crystal-remainder-invalid",
                "The checked phase remainder is invalid.",
            ),
            Self::AqueousInventoryMissing => (
                "solver-reason.aqueous.crystal-aqueous-inventory-missing",
                "The aqueous readback is missing an owned elemental remainder.",
            ),
            Self::AqueousResidualUnexplained => (
                "solver-reason.aqueous.crystal-aqueous-residual-unexplained",
                "The aqueous readback does not match the checked phase remainder.",
            ),
        };
        Phrase::bare(key, en)
    }
}

fn crystal_balance_error(reason: CrystalReason, detail: impl Into<String>) -> SolveError {
    SolveError::NotConverged {
        reason: Some(Box::new(reason.phrase())),
        solver: "phreeqc-aqueous".into(),
        detail: format!("mixed-crystal ownership: {}", detail.into()),
    }
}

fn checked_crystal_sum(
    values: &[f64],
    owner: &str,
) -> Result<kerotakis_core::amount::Amount, SolveError> {
    use kerotakis_core::amount::Amount;
    values
        .iter()
        .try_fold(Amount::new(0.0).unwrap(), |sum, value| {
            let amount = Amount::new(*value).map_err(|_| {
                crystal_balance_error(
                    CrystalReason::AmountInvalid,
                    format!("{owner}: invalid amount {value}"),
                )
            })?;
            sum.checked_add(amount).map_err(|_| {
                crystal_balance_error(
                    CrystalReason::AmountOverflow,
                    format!("{owner}: amount sum overflow"),
                )
            })
        })
}

fn crystal_roundoff_cap(terms: usize) -> Result<f64, SolveError> {
    let roundoff = (terms as f64 + 4.0) * f64::EPSILON;
    if !roundoff.is_finite() || roundoff >= 1.0 {
        return Err(crystal_balance_error(
            CrystalReason::ReductionUnsupported,
            "unsupported arithmetic reduction size",
        ));
    }
    Ok(roundoff / (1.0 - roundoff))
}

fn crystal_relative_cap(terms: usize) -> Result<f64, SolveError> {
    Ok(CRYSTAL_NATIVE_RELATIVE_CAP + CRYSTAL_OUTPUT_RELATIVE_CAP + crystal_roundoff_cap(terms)?)
}

/// Reconcile only witnessed exclusive aqueous totals and explicit Ca/Sr/C
/// owners. Inclusive selected totals and interface co-owners require a separate
/// same-solve witness; pre-solve budgets cannot establish that representation.
fn checked_crystal_reconciliation(
    vessel: &Vessel,
    problem: &Problem,
    rows: &[Vec<String>],
    ions: &[(String, f64)],
    crystals: &[SolidSolution],
    value: &dyn Fn(&str) -> Option<f64>,
) -> Result<Vec<(String, f64)>, SolveError> {
    if !problem.surfaces.is_empty() || !problem.exchanges.is_empty() {
        return Err(crystal_balance_error(
            CrystalReason::SurfaceExchangeCoowner,
            "surface/exchange co-ownership lacks an aqueous witness",
        ));
    }
    if problem
        .solid_solutions
        .iter()
        .chain(crystals)
        .any(|phase| !phase.has_valid_state())
    {
        return Err(crystal_balance_error(
            CrystalReason::InventoryInvalid,
            "invalid typed crystal inventory",
        ));
    }
    for (element, amount) in ions {
        checked_crystal_sum(&[*amount], &format!("raw aqueous {element}"))?;
    }
    // Validate every owned gas column before using carbon or clamping anything.
    let gases: Vec<(&str, f64)> = problem
        .gases
        .iter()
        .map(|(phase, species, _)| {
            let column = format!("g_{phase}");
            let amount = value(&column).ok_or_else(|| missing(rows, &column))?;
            checked_crystal_sum(&[amount], &column)?;
            Ok((species.as_str(), amount))
        })
        .collect::<Result<_, SolveError>>()?;
    let mut corrected = ions.to_vec();
    for element in ["Ca", "Sr", "C"] {
        if element == "C"
            && problem
                .external_gases
                .iter()
                .any(|gas| gas.species == "CO2")
        {
            continue; // Explicitly open carbon; Ca/Sr remain closed.
        }
        let mut available: Vec<f64> = problem
            .totals
            .iter()
            .filter(|(key, _)| key.split('(').next().unwrap_or(key) == element)
            .map(|(_, amount)| *amount)
            .collect();
        let mut owned: Vec<f64> = Vec::new();
        for (phases, amounts) in [
            (problem.solid_solutions.as_slice(), &mut available),
            (crystals, &mut owned),
        ] {
            amounts.extend(
                phases
                    .iter()
                    .flat_map(|phase| &phase.components)
                    .filter(|amount| {
                        solid_solution_component_elements(amount.component).contains(&element)
                    })
                    .map(|amount| amount.moles.0),
            );
        }
        // Pure phases are independent owners, absent from SOLUTION totals.
        for (phase, initial, _) in &problem.phases {
            if let Some(external) = problem
                .external_gases
                .iter()
                .find(|gas| gas.phase == *phase)
            {
                let final_amount = value(phase).ok_or_else(|| missing(rows, phase))?;
                checked_crystal_sum(&[*initial], phase)?;
                checked_crystal_sum(&[external.initial_moles], phase)?;
                checked_crystal_sum(&[final_amount], phase)?;

                // Reviewed gas boundaries have no Ca/Sr owner. Only CO2 opens
                // carbon; HBr and atmospheric O2/N2 leave its budget closed.
                let formula = crate::dbindex::parse_formula(phase.trim_end_matches("(g)"))
                    .ok_or_else(|| {
                        crystal_balance_error(
                            CrystalReason::ExternalPhaseUnknown,
                            format!("unreviewed external phase {phase}"),
                        )
                    })?;
                if formula.contains_key(element) {
                    return Err(crystal_balance_error(
                        CrystalReason::ExternalBoundaryUnsupported,
                        format!("external {phase} lacks a reviewed {element} boundary"),
                    ));
                }
                continue;
            }

            let definition = derived::phase_by_name(phase).ok_or_else(|| {
                crystal_balance_error(
                    CrystalReason::PrimaryPhaseUnknown,
                    format!("unreviewed primary phase {phase}"),
                )
            })?;
            let final_amount = value(phase).ok_or_else(|| missing(rows, phase))?;
            checked_crystal_sum(&[*initial], phase)?;
            checked_crystal_sum(&[final_amount], phase)?;
            let coefficients: Vec<f64> = definition
                .elements
                .iter()
                .filter(|(key, _)| key.split('(').next().unwrap_or(key) == element)
                .map(|(_, coefficient)| *coefficient)
                .collect();
            let coefficient = checked_crystal_sum(&coefficients, phase)?.to_f64();
            available.push(initial * coefficient);
            owned.push(final_amount * coefficient);
        }
        if element == "C" {
            available.extend(
                vessel
                    .contents
                    .iter()
                    .filter(|portion| portion.phase == Phase::Gas && portion.species.0 == "CO2")
                    .map(|portion| portion.moles.0),
            );
            owned.extend(
                gases
                    .iter()
                    .filter(|(species, _)| *species == "CO2")
                    .map(|(_, amount)| *amount),
            );
        }
        let available_amount = checked_crystal_sum(&available, &format!("available {element}"))?;
        let total_available = available_amount.to_f64();
        let owned_amount = checked_crystal_sum(&owned, &format!("final owned {element}"))?;
        let total_owned = owned_amount.to_f64();
        let cap = crystal_relative_cap(available.len() + owned.len())?;
        let scale = total_available.max(total_owned);
        if total_owned > total_available && (total_owned - total_available) / scale > cap {
            return Err(crystal_balance_error(
                CrystalReason::PhaseOverdraw,
                format!(
                    "{element} phase overdraw: available={total_available:e}, owned={total_owned:e}"
                ),
            ));
        }
        // Only a checked within-cap residual may reach this nonnegative bound.
        let target = match available_amount.checked_sub(owned_amount) {
            Ok(remainder) => remainder.to_f64(),
            Err(kerotakis_core::amount::AmountError::Negative) => 0.0,
            Err(error) => {
                return Err(crystal_balance_error(
                    CrystalReason::RemainderInvalid,
                    format!("{element} remainder: {error}"),
                ));
            }
        };
        let raw_parts: Vec<f64> = ions
            .iter()
            .filter(|(key, _)| key.split('(').next().unwrap_or(key) == element)
            .map(|(_, amount)| *amount)
            .collect();
        let raw = checked_crystal_sum(&raw_parts, &format!("raw aqueous {element}"))?.to_f64();
        let raw_cap = crystal_relative_cap(raw_parts.len() + available.len() + owned.len())?;
        let discrepancy = (raw - target).abs();
        if raw == 0.0 && target > 0.0 {
            return Err(crystal_balance_error(
                CrystalReason::AqueousInventoryMissing,
                format!("{element} missing aqueous inventory: remainder={target:e}"),
            ));
        }
        if raw != target && discrepancy / raw.max(target) > raw_cap {
            // Scalar owner products and residual projection have a separate
            // arithmetic budget at owner scale. Preserve the aqueous witness,
            // never amplify a trace to match an ill-conditioned subtraction.
            let roundoff = crystal_roundoff_cap(raw_parts.len() + available.len() + owned.len())?;
            if scale > 0.0 && discrepancy / scale <= roundoff {
                continue;
            }
            return Err(crystal_balance_error(
                CrystalReason::AqueousResidualUnexplained,
                format!(
                    "{element} unexplained aqueous residual: raw={raw:e}, remainder={target:e}"
                ),
            ));
        }
        if raw > 0.0 {
            let factor = target / raw;
            for (key, amount) in &mut corrected {
                if key.split('(').next().unwrap_or(key) == element {
                    *amount *= factor;
                }
            }
        }
    }
    Ok(corrected)
}

impl kerotakis_core::aqueous_continuation::TrustedAqueousReadback for PhreeqcEquilibrator {
    fn characterize_optional_current(
        &mut self,
        vessel: &mut Vessel,
    ) -> Result<
        (
            Vec<Event>,
            Option<kerotakis_core::aqueous_continuation::CurrentAqueousAnswer>,
        ),
        SolveError,
    > {
        let input = vessel.clone();
        let mut candidate = input.clone();
        candidate.solution = None;
        candidate.resolved.invalidate();
        candidate.free_proton = 0.0;
        candidate.free_hydroxide = 0.0;
        let events = self.equilibrate(&mut candidate)?;
        let answer = if candidate
            .solution
            .as_ref()
            .is_some_and(|s| s.scope == kerotakis_core::vessel::SolutionScope::Complete)
        {
            Some(
                kerotakis_core::aqueous_continuation::CurrentAqueousAnswer::attest_computed(
                    &input, &candidate,
                )?,
            )
        } else {
            None
        };
        *vessel = candidate;
        Ok((events, answer))
    }

    fn characterize_current(
        &mut self,
        vessel: &mut Vessel,
    ) -> Result<
        (
            Vec<Event>,
            kerotakis_core::aqueous_continuation::CurrentAqueousAnswer,
        ),
        SolveError,
    > {
        // Concrete native/hook/cache adapter: no generic Equilibrator may
        // manufacture a freshness receipt by restoring an old SolutionInfo.
        let input = vessel.clone();
        let mut candidate = input.clone();
        candidate.solution = None;
        candidate.resolved.invalidate();
        candidate.free_proton = 0.0;
        candidate.free_hydroxide = 0.0;
        let events = self.equilibrate(&mut candidate)?;
        let receipt = kerotakis_core::aqueous_continuation::CurrentAqueousAnswer::attest_computed(
            &input, &candidate,
        )?;
        *vessel = candidate;
        Ok((events, receipt))
    }
}

impl Equilibrator for PhreeqcEquilibrator {
    fn trusted_aqueous_readback(
        &mut self,
    ) -> Option<&mut dyn kerotakis_core::aqueous_continuation::TrustedAqueousReadback> {
        Some(self)
    }

    fn name(&self) -> &'static str {
        "phreeqc-aqueous"
    }

    fn applies(&self, vessel: &Vessel) -> bool {
        // Above the ceiling the databases' temperature expressions have
        // ended and a solve is an extrapolated crash waiting to happen
        // (superheated water, curiosity th-022). The honesty pass names
        // the boundary when this gate declines.
        let inside_validity = vessel.temperature.0
            <= kerotakis_core::solve::AQUEOUS_MODEL_CEILING_K
            && kerotakis_core::nonaqueous::water_fraction_among_solvents(vessel)
                .is_none_or(|x| x >= kerotakis_core::nonaqueous::AQUEOUS_WATER_FRACTION_FLOOR);

        // "Has this solver anything to say about this vessel?" — and having
        // to say *no* is an answer, not a reason to stand down.
        //
        // `partition` declines a beaker of malic acid and water, because
        // nothing in it speciates: the acid dissolves on the neutral-solute
        // rung and the databases have no malate. Standing down there meant
        // the stack skipped this solver entirely, so the one place that
        // knows the acidity is missing never got to say so. The refusal is
        // the whole point of carrying the substance at all.
        inside_validity
            && (partition(vessel).is_some()
                || holds_unspeciated_acid(vessel)
                || holds_unspeciated_solute(vessel))
    }

    fn chemistry_applies(&self, vessel: &Vessel) -> bool {
        // Characterising water's own H+/OH- equilibrium is useful support
        // state for meters, but it is not an answer that anything in the
        // beaker reacted. Keep that distinction in the machine-readable
        // route: coverage must not call an unrelated experiment "computed"
        // merely because its setup included water.
        self.applies(vessel)
            && (has_solvent_only_basis(vessel)
                || partition(vessel).is_some_and(|problem| !problem.solvent_only)
                || holds_unspeciated_acid(vessel)
                || holds_unspeciated_solute(vessel))
    }

    /// Solve, and keep solving until the temperature stops moving.
    ///
    /// Solubility depends on temperature and dissolution changes the
    /// temperature, so the two have to be settled together. Solving once
    /// and then applying the heat produces a vessel whose stated
    /// composition and stated temperature describe *different* states:
    /// 60 g of KCl in 100 mL used to report 0.4777 mol dissolved beside a
    /// thermometer reading of 6.05 °C, when only ~0.40 mol will dissolve at
    /// 6 °C. Twenty per cent out, at saturation, which is exactly where a
    /// solubility lesson lives.
    ///
    /// The iteration is the physical statement T* = T₀ + q(T*)/c_p, and it
    /// contracts: an endothermic salt that cools the beaker dissolves less,
    /// which cools it less. Two or three passes settle it.
    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        // Each equilibration is its own question; a pe carried over from a
        // different vessel is a guess, not a memory. The warm start exists
        // for the temperature fixed point *inside* this call.
        self.warm_pe = None;
        validate_clock_boundary(vessel, partition(vessel).as_ref())?;
        if let Some(boundary) = kerotakis_core::nonaqueous::peroxide_medium_boundary(vessel) {
            return Ok(vec![boundary]);
        }
        if has_solvent_only_basis(vessel) {
            // The water relation cannot price a finite H+/OH- reaction or
            // normalize its water inventory. A neutral reading would hide
            // that missing chemistry, even for a charge-balanced pair.
            vessel.solution = None;
            vessel.free_proton = 0.0;
            vessel.free_hydroxide = 0.0;
            vessel.resolved.invalidate();
            return Ok(vec![Event::not_modeled(
                vessel.id,
                kerotakis_core::ops::NotModelledCause::ModelBoundary,
                Phrase::bare(
                    "not-modeled.bare-acid-base-basis",
                    "this solvent-only inventory contains explicit acid/base equivalents. Their neutralisation, water balance and reaction heat are not implemented by the solvent autoionisation path. The inventory is retained and no solution reading is assigned; use complete acid/base reagents for a supported neutralisation experiment",
                ),
            )]);
        }
        if partition(vessel).is_some_and(|problem| problem.solvent_only) {
            return characterize_solvent_only(vessel);
        }
        let component_charge_note = analytical_charge_note(vessel);
        if let Some(surface) = vessel
            .surfaces
            .iter()
            .find(|surface| !surface.has_valid_capacity())
        {
            return Err(SolveError::NotConverged {
                reason: Some(Box::new(AqueousReason::SurfaceCapacityInvalid.phrase())),
                solver: self.name().to_string(),
                detail: format!(
                    "surface '{}' has non-positive geometry, invalid capacity, or occupancy above capacity",
                    surface.label
                ),
            });
        }
        if let Some(exchange) = vessel
            .exchanges
            .iter()
            .find(|exchange| !exchange.has_valid_capacity())
        {
            return Err(SolveError::NotConverged {
                reason: Some(Box::new(AqueousReason::ExchangeCapacityInvalid.phrase())),
                solver: self.name().to_string(),
                detail: format!(
                    "exchange '{}' has non-positive support/capacity or occupancy that does not exactly counter-balance its finite capacity",
                    exchange.label
                ),
            });
        }
        if vessel.solid_solutions.len() > 1 {
            return Err(SolveError::NotConverged {
                reason: Some(Box::new(AqueousReason::MixedCrystalUnsupported.phrase())),
                solver: self.name().to_string(),
                detail: "the first typed solid-solution slice supports one mixed crystalline phase per vessel"
                    .to_string(),
            });
        }
        if let Some(solid_solution) = vessel
            .solid_solutions
            .iter()
            .find(|solid_solution| !solid_solution.has_valid_state())
        {
            return Err(SolveError::NotConverged {
                reason: Some(Box::new(AqueousReason::SolidSolutionStateInvalid.phrase())),
                solver: self.name().to_string(),
                detail: format!(
                    "solid solution '{}' must contain one finite, non-negative amount of each reviewed end member",
                    solid_solution.label
                ),
            });
        }
        // Each thermal guess starts with the same unspent clock input. The
        // private aqueous answer records what was actually posed and spent;
        // only that settled answer may produce the one public clock event.
        let start = vessel.clone();
        let t0 = start.temperature.0;
        let mut guess = t0;
        let mut volume_guess = start.headspace_volume();
        let mut settled: Option<(Vessel, Vec<Event>, f64, bool, f64)> = None;
        let mut settled_unpriced: Option<enthalpy::Unpriced> = None;
        let mut fixed_point_converged = false;
        let mut last_temperature_residual = f64::INFINITY;
        let mut previous_guess: Option<f64> = None;
        let mut previous_was_provisional = false;
        let mut full_strict_thermal = false;

        for _ in 0..MAX_THERMAL_FIXED_POINT_PASSES {
            let mut trial = start.clone();
            trial.temperature = Kelvin(guess);
            if let (Headspace::PressureControlled { pressure, .. }, Some(volume)) =
                (trial.headspace, volume_guess)
            {
                trial.headspace = Headspace::PressureControlled { pressure, volume };
            }
            let mut deferred_seed = None;
            self.last_heat_unpriced = None;
            let (mut events, q_joules, mut physical_canonical_closed, mut clock_applied) =
                if full_strict_thermal {
                    self.solve_at_temperature(&mut trial)?
                } else {
                    let seed = self.first_pose_at_temperature(&trial)?;
                    if seed.pass.component_certificate == ComponentCertificate::ReservoirWithOpaque
                    {
                        // Only the new open-reservoir opaque class defers its
                        // physical reentries. Every other owner retains the
                        // original strict schedule, reusing this first pose.
                        trial = seed.trial.clone();
                        let provisional = (
                            seed.pass.events.clone(),
                            seed.pass.heat_quote_j,
                            false,
                            seed.pass.clock_co2_applied_mol,
                        );
                        deferred_seed = Some(seed);
                        provisional
                    } else {
                        self.resume_first_pose(&mut trial, seed)?
                    }
                };
            // Enthalpy, not temperature, is what survives a solve.
            //
            // Dissolved matter carries no heat capacity in this model — the
            // ions are all Cp 0 — so the vessel's Cp *drops* when a solute
            // speciates. Computing `t0 + q/cp` then quietly destroys the
            // sensible heat the pre-solve heat capacity was holding, and the
            // loss shows up as a broken Hess's law: hydrochloric acid into
            // a beaker already warmed by caustic soda ended 0.19 K below the
            // same two reagents added the other way round. The acid was
            // credited with a heat capacity while it was still a liquid, and
            // stripped of it the moment the solver called it chloride.
            //
            // Balancing enthalpy instead makes the relabelling free, which
            // is what it physically is. Working both orders through by hand,
            // each lands on T₀ + q/Cp(water): the mixing term that cools the
            // second beaker is exactly the term the enthalpy balance gives
            // back.
            //
            // And it is the INTEGRAL that has to balance, not the rectangle.
            // `Cp(start)·(t₀ − t_ref)` and `Cp(trial)·(t − t_ref)` read one
            // curve at two temperatures and then treat both as flat, and the
            // error that leaves depends on where the beaker was standing when
            // the reagent arrived — so it is a different error for each order
            // they arrive in, and Hess's law breaks again in the fifth
            // decimal. Acid-then-base passes through 25 °C and base-then-acid
            // through 35.7 °C, and the two rectangles landed 7.25e-5 K apart
            // on the same final solution. The area under each side's own
            // Cp(T) is a state function and puts them back together.
            // A vessel held at a bath temperature is held: only an adiabatic
            // one moves with the heat its own chemistry released, and letting
            // the fixed-point loop walk a thermostatted vessel away from t0
            // made the bath a suggestion.
            // Unknown-heat continuation is characterisation at the supplied
            // temperature, not an adiabatic prediction. Even a Cp/water basis
            // change must not reconstruct omitted endpoint heat indirectly.
            let mut heat_unpriced = self.last_heat_unpriced.clone();
            let mut next =
                aqueous_temperature_target(&start, &trial, q_joules, heat_unpriced.is_none());
            let mut next_volume = trial.headspace_volume();
            last_temperature_residual = (next - guess).abs();
            let provisional_thermal = deferred_seed.is_some();
            let mut converged = aqueous_thermal_pose_converged(
                &start,
                guess,
                next,
                volume_guess,
                next_volume,
                previous_guess.filter(|_| provisional_thermal == previous_was_provisional),
            );
            let mut reset_thermal_history = false;
            if converged {
                if let Some(seed) = deferred_seed {
                    let (strict_events, strict_heat, strict_closed, strict_clock) =
                        self.resume_first_pose(&mut trial, seed)?;
                    if !strict_closed {
                        return Err(canonical_closure_error(
                            AqueousReason::StrictClosureMissing,
                            "deferred reservoir answer lacks strict physical closure",
                        ));
                    }
                    events = strict_events;
                    physical_canonical_closed = strict_closed;
                    clock_applied = strict_clock;
                    heat_unpriced = self.last_heat_unpriced.clone();
                    next = aqueous_temperature_target(
                        &start,
                        &trial,
                        strict_heat,
                        heat_unpriced.is_none(),
                    );
                    next_volume = trial.headspace_volume();
                    last_temperature_residual = (next - guess).abs();
                    // A provisional two-cycle cannot witness the strict heat
                    // function. If final closure moves T or volume, continue
                    // permanently with the original full-strict thermal map.
                    converged = aqueous_thermal_pose_converged(
                        &start,
                        guess,
                        next,
                        volume_guess,
                        next_volume,
                        None,
                    );
                    if !converged {
                        full_strict_thermal = true;
                        reset_thermal_history = true;
                    }
                }
            }
            // `trial` was solved at `guess`, so preserve that exact
            // temperature with its speciation.  `next` is the enthalpy-map
            // residual used only to decide convergence; storing it beside a
            // solution computed at `guess` was the original feed-order bug.
            settled = Some((
                trial,
                events,
                guess,
                physical_canonical_closed,
                clock_applied,
            ));
            settled_unpriced = heat_unpriced;
            previous_guess = if reset_thermal_history {
                None
            } else {
                Some(guess)
            };
            previous_was_provisional = provisional_thermal;
            guess = next;
            volume_guess = next_volume;
            if converged {
                fixed_point_converged = true;
                break;
            }
        }

        if !fixed_point_converged {
            return Err(SolveError::NotConverged {
                reason: Some(Box::new(AqueousReason::ThermalPassCap.phrase())),
                solver: self.name().to_string(),
                detail: format!(
                    "coupled aqueous temperature did not converge within {MAX_THERMAL_FIXED_POINT_PASSES} passes (last residual {:.6e} K)",
                    last_temperature_residual
                ),
            });
        }

        let Some((solved, mut events, t_final, physical_canonical_closed, clock_applied)) = settled
        else {
            let mut notes = unspeciated_acid_notes(&start);
            notes.extend(unspeciated_solute_notes(&start));
            notes.extend(component_charge_note);
            return Ok(notes);
        };
        *vessel = solved;
        // The final state, posed canonically, before the last word on it.
        if !physical_canonical_closed {
            match self.recharacterise_canonically(vessel, &mut events) {
                Ok(()) => {}
                Err(error) => {
                    *vessel = start;
                    return Err(error);
                }
            }
        }
        events.extend(unspeciated_acid_notes(vessel));
        events.extend(unspeciated_solute_notes(vessel));
        events.extend(component_charge_note);
        if let Some(unpriced) = settled_unpriced.filter(|_| {
            matches!(vessel.thermal_mode, ThermalMode::Adiabatic)
                && kerotakis_core::aqueous_continuation::whole_step_heat_available(&start)
        }) {
            events.push(unpriced_heat_note(vessel.id, &unpriced));
        }
        let ph_now = vessel.solution.as_ref().map(|s| s.ph);
        events.extend(milk_buffer_notes(vessel, ph_now));
        // Observability gates narration, never the physical state. Dropping
        // sub-0.01 K updates loses energy and makes pouring order matter.
        if matches!(vessel.thermal_mode, ThermalMode::Adiabatic) {
            vessel.temperature = Kelvin(t_final.max(0.0));
        }
        if matches!(vessel.thermal_mode, ThermalMode::Adiabatic) && (t_final - t0).abs() > 0.01 {
            // From where the vessel actually started, not from the last
            // trial temperature the iteration happened to stop on.
            let from = Kelvin(t0);
            let to = Kelvin(t_final.max(0.0));
            vessel.temperature = to;
            events.push(Event::TemperatureChanged {
                vessel: vessel.id,
                from,
                to,
            });
        }
        // The last chemistry pass ran at the converged temperature guess,
        // while the assignment above records the final enthalpy-balanced
        // temperature. Keep the derived ideal-gas pressure—and the event
        // that reports it—on that same final state.
        vessel.refresh_pressure();
        if let Some(Event::HeadspaceEquilibrated {
            pressure,
            total_moles,
            ..
        }) = events
            .iter_mut()
            .rev()
            .find(|event| matches!(event, Event::HeadspaceEquilibrated { .. }))
        {
            *pressure = vessel.pressure;
            *total_moles = vessel.gas_moles();
        }
        if clock_applied != 0.0 {
            // The private candidate already spent this exact parcel. Native
            // reservoir crossings are separate, collected by the driver.
            events.push(if clock_applied > 0.0 {
                Event::GasAbsorbed {
                    vessel: vessel.id,
                    species: SpeciesId::new("CO2"),
                    moles: Moles(clock_applied),
                }
            } else {
                Event::GasEvolved {
                    vessel: vessel.id,
                    species: SpeciesId::new("CO2"),
                    moles: Moles(-clock_applied),
                }
            });
        }
        Ok(events)
    }

    fn mix(
        &mut self,
        vessel: &mut Vessel,
        soln_a: &Vessel,
        frac_a: f64,
        soln_b: &Vessel,
        frac_b: f64,
    ) -> Option<Result<Vec<Event>, SolveError>> {
        // This optimization serializes dissolved totals and zero-inventory
        // precipitation candidates only. Supplied solids, retained gas and
        // typed interfaces need the ordinary target solve: MIX would otherwise
        // omit their matter and rebuild the target from an incomplete ledger.
        if [vessel as &Vessel, soln_a, soln_b].iter().any(|source| {
            source.contents.iter().any(|portion| {
                portion.moles.0 > 0.0 && matches!(portion.phase, Phase::Solid | Phase::Gas)
            }) || !source.surfaces.is_empty()
                || !source.exchanges.is_empty()
                || !source.solid_solutions.is_empty()
                || source.pending_co2_transfer_mol != 0.0
                || !source.pending_co2_transfer_mol.is_finite()
        }) {
            return None;
        }
        // Native MIX has its own whole-step thermal bookkeeping. An active
        // unknown-heat continuation must use the ordinary guarded readback.
        if !kerotakis_core::aqueous_continuation::whole_step_heat_available(vessel) {
            return None;
        }
        // Neither source nor target may bypass the ordinary analytical
        // component/basis reporting boundary through the MIX optimization.
        if [vessel as &Vessel, soln_a, soln_b].iter().any(|source| {
            has_solvent_only_basis(source) || analytical_charge_note(source).is_some()
        }) {
            return None;
        }
        // Both source vessels must be solvable aqueous problems.
        let problem_a = partition(soln_a)?;
        let problem_b = partition(soln_b)?;
        // Let the ordinary mixed-vessel equilibrium path handle these. It
        // has a zero-engine fast path when the result is still solvent-only,
        // while a real solute in the other source still receives a complete
        // aqueous solve. Native MIX would turn every pour of plain water into
        // an expensive engine call for no additional information.
        if problem_a.solvent_only || problem_b.solvent_only {
            return None;
        }

        // Route both to the same database; refuse if they disagree.
        let needs_extended_a = problem_a
            .elements
            .iter()
            .any(|e| !derived::index_for("wateq4f").has_element(e) || e == "P");
        let needs_extended_b = problem_b
            .elements
            .iter()
            .any(|e| !derived::index_for("wateq4f").has_element(e) || e == "P");
        let db_tag = if needs_extended_a || needs_extended_b {
            "minteq.v4"
        } else {
            "wateq4f"
        };

        // Merged elements for readback.
        let mut merged_elements: Vec<String> = Vec::new();
        for el in problem_a.elements.iter().chain(problem_b.elements.iter()) {
            if !merged_elements.contains(el) {
                merged_elements.push(el.clone());
            }
        }

        // Separate sources can each use wateq4f while their combination
        // needs silver ammines. Leave that new ligand problem to the
        // ordinary target solve, which owns capability routing/provenance.
        if needs_silver_ammine(&merged_elements) {
            return None;
        }

        // Candidate phases for the mixed solution, reconciled with the
        // routed database by the same `posed_phase` the direct path uses in
        // `setup_problem`. Keeping only the names the database defines
        // *natively* was the A4 divergence: polymorph translation and
        // reviewed foreign-phase injection are chemistry the direct path
        // applies, and a beaker reached by combining two solutions could
        // not grow a precipitate the same beaker reached directly grew.
        // The posed name is carried into `merged_problem` below, because it
        // is also the selected-output column the readback asks for.
        //
        // The union of the two lists is not the mixture's list, either.
        // `partition` derives candidates from one vessel's element set, so a
        // solid whose elements only meet on mixing — Halite, from an iron
        // *chloride* poured into a *sodium* hydroxide — is proposed by
        // neither source and could not form on the MIX solve, while the
        // direct path poses it. The merged set goes through the same
        // derivation rather than through a union of two answers to a
        // different question.
        let merged_phases: Vec<(String, f64, f64)> = {
            let mut candidates: Vec<(String, f64, f64)> = Vec::new();
            for (name, _, si) in problem_a.phases.iter().chain(problem_b.phases.iter()) {
                if !candidates.iter().any(|(listed, ..)| listed == name) {
                    candidates.push((name.clone(), 0.0, *si));
                }
            }
            append_candidate_phases(vessel, &merged_elements, &mut candidates);
            let mut phases: Vec<(String, f64, f64)> = Vec::new();
            for (name, _, si) in &candidates {
                let Some(posed) = posed_phase(name, db_tag) else {
                    continue;
                };
                if !phases.iter().any(|(candidate, ..)| candidate == posed) {
                    phases.push((posed.to_string(), 0.0, *si));
                }
            }
            phases
        };

        // The merged Problem describes the beaker the MIX makes: the element
        // set and candidate phases the input poses, and the totals the
        // readback reconciles against. It is built before the input because
        // the input is built *from* it — the pin block and the
        // per-oxidation-state `-totals` split are both statements about the
        // mixture, not about either source solution.
        let merged_problem = Problem {
            kgw: problem_a.kgw * frac_a + problem_b.kgw * frac_b,
            solvent_only: false,
            clock_co2_applied_mol: 0.0,
            totals: {
                let mut t: Vec<(String, f64)> = Vec::new();
                for (el, moles) in &problem_a.totals {
                    if let Some(entry) = t.iter_mut().find(|(e, _)| e == el) {
                        entry.1 += moles * frac_a;
                    } else {
                        t.push((el.clone(), moles * frac_a));
                    }
                }
                for (el, moles) in &problem_b.totals {
                    if let Some(entry) = t.iter_mut().find(|(e, _)| e == el) {
                        entry.1 += moles * frac_b;
                    } else {
                        t.push((el.clone(), moles * frac_b));
                    }
                }
                t
            },
            phases: merged_phases,
            gases: Vec::new(),
            external_gases: Vec::new(),
            surfaces: Vec::new(),
            exchanges: Vec::new(),
            solid_solutions: Vec::new(),
            elements: merged_elements,
        };

        // Native MIX does not run the closed electron-budget root. Let the
        // ordinary equilibrium owner compute coupled redox from the already
        // merged inventory; accepting MIX here would choose a different
        // electron reservoir merely because the user poured two solutions.
        if redox_coupling(&merged_problem, db_tag).is_some() {
            return None;
        }

        let input = build_mix_input(
            soln_a,
            &problem_a,
            soln_b,
            &problem_b,
            frac_a,
            frac_b,
            db_tag,
            &merged_problem,
            phase_input_precision(vessel),
        );

        if env_dump_input_all() {
            eprintln!("--- PHREEQC MIX input ---\n{input}---");
        }

        // `Bench` treats a failed MIX as advisory and silently re-solves the
        // target through the direct path, so an abandoned MIX is invisible
        // except in the engine calls it costs. Say why, when asked.
        let abandoned = |e: SolveError| -> Option<Result<Vec<Event>, SolveError>> {
            if env_dump_input() {
                eprintln!("--- PHREEQC MIX abandoned: {e} ---\n{input}---");
            }
            Some(Err(e))
        };

        let out = match self.run_raw(db_tag, &input) {
            Ok(out) => out,
            Err(e) => return abandoned(e),
        };

        let speciation = match numerical_species_distribution(&out.report, &out.selected) {
            Ok(speciation) => speciation,
            Err(error) => return abandoned(error),
        };
        let cached = Rc::new(CachedSolve {
            pe_determined: !out.pe_undetermined,
            pe_authority: PeAuthority::Unwitnessed,
            rows: out.selected,
            speciation,
            saturation: parse_saturation_indices(&out.report),
            redox_adjusted: out.report.contains("Adjusted to redox equilibrium"),
        });

        let value = |column: &str| -> Option<f64> {
            let idx = cached.rows.first()?.iter().position(|h| h == column)?;
            cached.rows.last()?.get(idx)?.parse().ok()
        };

        let readback = self.readback_raw_values(&merged_problem, db_tag, &cached.rows, &value);
        let (
            solvent_kgw_out,
            mut new_surfaces,
            new_exchanges,
            new_solid_solutions,
            mut new_ions,
            unnameable,
            protonation,
        ) = match readback {
            Ok(v) => v,
            Err(e) => return abandoned(e),
        };

        if unnameable.iter().any(|(_, n)| *n > 1e-10) {
            return abandoned(SolveError::NotConverged {
                reason: Some(Box::new(AqueousReason::ReadbackUnrepresented.phrase())),
                solver: "phreeqc-inventory".into(),
                detail: format!("cannot commit unrepresented elemental states: {unnameable:?}"),
            });
        }

        let balance = Self::apply_balance_corrections(
            vessel,
            &merged_problem,
            &cached.rows,
            &mut new_ions,
            &mut new_surfaces,
            &new_exchanges,
            &new_solid_solutions,
            &value,
            ComponentCertificate::Uncertified,
            false,
        );
        let (new_phases, new_gases, ph, mu, gas_components_certified) = match balance {
            Ok(v) => v,
            Err(e) => return abandoned(e),
        };
        if gas_components_certified {
            if let Err(error) = certify_projected_protonation(&new_ions, &protonation) {
                return abandoned(error);
            }
        }

        let (mut events, contents) = Self::rebuild_contents_and_events(
            vessel,
            &merged_problem,
            &[],
            solvent_kgw_out,
            &new_ions,
            &new_phases,
            &new_gases,
            &new_solid_solutions,
            &protonation,
        );

        let before_inventory = vessel.clone();
        vessel.contents = contents;
        vessel.surfaces = new_surfaces;
        vessel.exchanges = new_exchanges;
        vessel.solid_solutions = new_solid_solutions;
        let reservoirs: Vec<&str> = merged_problem
            .external_gases
            .iter()
            .filter(|g| matches!(g.kind, ExternalGasKind::Reservoir))
            .map(|g| g.species.as_str())
            .collect();
        if let Err(error) = crate::inventory::complete_basis(
            &before_inventory,
            vessel,
            &events,
            &reservoirs,
            crate::inventory::ClockCo2Boundary::none(before_inventory.id),
        ) {
            *vessel = before_inventory;
            return abandoned(error);
        }
        if gas_components_certified {
            if let Err(error) = crate::inventory::certify_gas_aware_inventory(
                &before_inventory,
                vessel,
                &events,
                &reservoirs,
                crate::inventory::ClockCo2Boundary::none(vessel.id),
            ) {
                *vessel = before_inventory;
                return abandoned(error);
            }
        }
        if let Err(error) =
            refresh_measured_coordinates(vessel, &cached.rows, &cached.speciation, solvent_kgw_out)
        {
            *vessel = before_inventory;
            return abandoned(error);
        }
        vessel.refresh_pressure();

        events.extend(reference_complex_boundary(vessel, &cached.speciation));
        events.extend(silver_ammine_inventory_boundary(vessel, &cached.speciation));
        // Composed BEFORE the record it goes into, because a reader has
        // to be told about it and the telling is an event.
        //
        // A MIX writes a provenance of its own — a different dataset is
        // reachable here than the one either source used, and the ROUTING
        // is `MIX: two solved solutions combined by fraction`, which is
        // not a sentence any other path composes. Until now it went into
        // `vessel.solution` and stopped there: the drawer reads
        // `event.provenance`, no event carried this one, and a beaker
        // filled by pouring two others held provenance nobody could see.
        // Ruled in by the owner on 2026-09-18, after #653 scoped it out.
        let provenance = Provenance::new(
            "PHREEQC (IPhreeqc, USGS)",
            dataset_phrase(db_tag),
            derived::index_for(db_tag).activity_model.phrase(),
            dataset_sources(db_tag),
            Phrase::bare(
                "routing.mix-by-fraction",
                "MIX: two solved solutions combined by fraction",
            ),
        );
        // After the events that say what the pour DID, and before the
        // characterisation: the reader is told what happened, then where
        // the numbers for it came from.
        announce_routing(vessel, &provenance, &mut events);
        vessel.solution = Some(SolutionInfo {
            solvent_activity: None,
            scope: Default::default(),
            solvent_kg: value("mass_H2O"),
            particle_molality: value("particle_molality"),
            // MIX itself does not solve the lab's electron-budget root.
            pe: None,
            redox: redox_distribution(&merged_problem, db_tag, &value),
            ph,
            ionic_strength: mu,
            species: cached.speciation.clone(),
            provenance: Some(provenance),
        });

        events.push(Event::SolutionCharacterized {
            vessel: vessel.id,
            ph,
            ionic_strength: mu,
        });

        Some(Ok(events))
    }
}

struct SolveSetup {
    problem: Problem,
    db_tag: &'static str,
    /// Why this dataset, as a RECIPE rather than a finished sentence.
    ///
    /// It is composed here in three pieces — the base choice, an optional
    /// activity-model caveat, and two more clauses added in
    /// `finalize_solution_info` — and every one of them used to be welded
    /// on with `format!`/`push_str`. A reader who does not read English
    /// met all of it in English.
    routing: Phrase,
    freed_phases: Vec<(String, f64)>,
    input: String,
    key: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HeatMode {
    Inactive,
    Priced,
    /// The whole-step state-function balance declined. No partial heat is
    /// charged and the step is characterised at its supplied temperature.
    Unpriced,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ComponentCertificate {
    Uncertified,
    Represented,
    ClockWithOpaque,
    /// An open, reviewed gas-reservoir route beside exactly immutable opaque
    /// recipe matter and optical reporters. Actual gas crossings remain owned.
    ReservoirWithOpaque,
}

impl ComponentCertificate {
    fn is_certified(self) -> bool {
        self != Self::Uncertified
    }
}

/// One native answer at exactly one original owner temperature/volume. It may
/// guide a private thermal guess, but earns no canonical acceptance until the
/// unchanged strict driver consumes it as its first counted pass.
struct AqueousFirstPose {
    owner_start: Vessel,
    trial: Vessel,
    pass: AqueousPass,
}

/// The typed refusal for a step whose whole-step heat ledger is incomplete.
/// The chemistry of the step stands; only its heat is withheld, and the
/// temperature is the one the step was supplied at.
fn unpriced_heat_note(vessel: kerotakis_core::VesselId, unpriced: &enthalpy::Unpriced) -> Event {
    Event::not_modeled(
        vessel,
        kerotakis_core::ops::NotModelledCause::NoReviewedDatum,
        Phrase::new(
            "not-modeled.aqueous-heat-unpriced",
            "the reaction heat of this step is not modelled: {species} changed amount and has no reviewed enthalpy on this route. No partial heat is charged; the temperature stays where the step began, and the composition, pH and speciation are unaffected",
            vec![("species".to_string(), Slot::text(unpriced.species.clone()))],
        ),
    )
}

fn aqueous_temperature_target(
    start: &Vessel,
    trial: &Vessel,
    q_joules: f64,
    heat_priced: bool,
) -> f64 {
    let t0 = start.temperature.0;
    let t_ref = Kelvin::STANDARD.0;
    let held = start.energy_between(t_ref, t0) + q_joules;
    // An unpriced step keeps its supplied temperature. Re-reading the same
    // enthalpy through the trial's lower heat capacity (dissolved ions carry
    // Cp 0 here) would release the solid's sensible heat above 25 °C as if
    // it were reaction heat while the dissolution enthalpy itself stayed
    // unowned: hot potassium nitrate then WARMED its solution by 8.8 K.
    if !heat_priced
        || !matches!(start.thermal_mode, ThermalMode::Adiabatic)
        || !kerotakis_core::aqueous_continuation::whole_step_heat_available(start)
    {
        t0
    } else if trial.heat_capacity() > 0.0 {
        trial.temperature_after_from(t_ref, held)
    } else {
        t0
    }
}

fn aqueous_thermal_pose_converged(
    start: &Vessel,
    guess: f64,
    next: f64,
    volume_guess: Option<kerotakis_core::Liters>,
    next_volume: Option<kerotakis_core::Liters>,
    previous_guess: Option<f64>,
) -> bool {
    let volume_converged = match (volume_guess, next_volume) {
        (Some(before), Some(after)) if start.owns_headspace_gas() => {
            (after.0 - before.0).abs() < (before.0.abs() * 1e-7).max(1e-9)
        }
        _ => true,
    };
    let residual = (next - guess).abs();
    let quantized_two_cycle = previous_guess.is_some_and(|previous| {
        (next - previous).abs() <= 1e-12 && residual < THERMAL_FIXED_POINT_QUANTIZATION_CYCLE_K
    });
    volume_converged && (residual < THERMAL_FIXED_POINT_TOLERANCE_K || quantized_two_cycle)
}

fn secondary_routing(
    routing: Phrase,
    solvent_activity: Option<&kerotakis_core::vessel::SolventActivityProvenance>,
) -> Phrase {
    match solvent_activity {
        None => routing,
        Some(second) => Phrase::new(
            "routing.second-speciation-for-solvent",
            "{routing}; the solvent's activity is NOT from this dataset — it reports PHREEQC's hard-coded 1 - 0.017*Sum(m) placeholder rather than a model — but from a second speciation of the same solution on {dataset} ({model}), posed with no phases, gas or interfaces and asked for a_w alone",
            vec![
                ("routing".to_string(), Slot::phrase(routing)),
                // Neither of these is a bare name. A dataset is a FILE
                // NAME with clauses welded to it and a model is a
                // PERSON'S NAME with a reliability range welded to it,
                // so each travels as its RECIPE — the names inside it
                // still in text slots, which are the ones no catalogue
                // touches. They were `Slot::text` of the finished
                // English, which put two English clauses in the middle
                // of a German routing paragraph.
                ("dataset".to_string(), second.dataset_slot()),
                ("model".to_string(), second.model_slot()),
            ],
        ),
    }
}

struct AqueousPass {
    events: Vec<Event>,
    /// Priced mode is a whole-step state-function quote. Unpriced mode
    /// charges nothing: a partial per-event ledger is never applied.
    heat_quote_j: f64,
    heat_mode: HeatMode,
    component_certificate: ComponentCertificate,
    db_tag: &'static str,
    reservoirs: Vec<String>,
    /// The actual clock parcel posed in this pass's analytical totals.
    /// Native reservoir events remain separate.
    clock_co2_applied_mol: f64,
    secondary_disposition: SecondaryDisposition,
    pending_secondary: Option<Box<ExactAuxiliaryQuestion>>,
}

impl AqueousPass {
    fn has_pending_secondary(&self) -> Result<bool, SolveError> {
        let pending = self.secondary_disposition == SecondaryDisposition::Pending;
        if pending != self.pending_secondary.is_some() {
            return Err(canonical_closure_error(
                AqueousReason::PendingAuthorityInconsistent,
                "inconsistent queued auxiliary authority",
            ));
        }
        Ok(pending)
    }
}

enum AqueousWork<'a> {
    Pose {
        candidate: &'a mut Vessel,
        past: &'a [Event],
        owner: ComponentCertificate,
        hint: FirstPoseHint,
        auxiliary_policy: AuxiliaryPolicy,
    },
    Fulfil {
        candidate: &'a mut Vessel,
        pass: AqueousPass,
    },
}

#[derive(Debug, PartialEq, Eq)]
enum PrimaryProgress {
    Moving,
    ReadyForAuxiliary,
}

fn validate_clock_boundary(vessel: &Vessel, problem: Option<&Problem>) -> Result<(), SolveError> {
    let pending = vessel.pending_co2_transfer_mol;
    if !pending.is_finite() {
        return Err(canonical_closure_error(
            AqueousReason::ClockParcelNonfinite,
            "nonfinite pending clock CO2 parcel",
        ));
    }
    let spendable = pending.abs() > kerotakis_core::clock::CO2_TRANSFER_FLOOR_MOL;
    let applied = problem.map_or(0.0, |problem| problem.clock_co2_applied_mol);
    if !applied.is_finite()
        || (spendable && (problem.is_none() || applied != pending))
        || (!spendable && applied != 0.0)
    {
        return Err(canonical_closure_error(
            AqueousReason::ClockBoundaryInvalid,
            "pending clock CO2 parcel is unsupported or exceeds available carbon",
        ));
    }
    if applied != 0.0
        && problem.is_some_and(|problem| {
            problem
                .totals
                .iter()
                .filter(|(element, _)| element == "C" || element.starts_with("C("))
                .count()
                != 1
                || !problem.totals.iter().any(|(element, _)| element == "C")
        })
    {
        return Err(canonical_closure_error(
            AqueousReason::ClockCarbonAmbiguous,
            "clock CO2 cannot be assigned to an ambiguous carbon component",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod clock_co2_boundary_tests {
    use super::*;
    use kerotakis_core::VesselId;

    #[test]
    fn clock_pose_keeps_parked_parcel_and_refuses_overdraw_or_ambiguous_carbon() {
        let mut vessel = Vessel::new(VesselId(4), "clock input");
        vessel.deposit(SpeciesId::new("water"), Moles(5.0), Phase::Liquid);
        vessel.pending_co2_transfer_mol = 1e-7;
        let parked = partition(&vessel);
        assert_eq!(parked.as_ref().unwrap().clock_co2_applied_mol, 0.0);
        assert!(validate_clock_boundary(&vessel, parked.as_ref()).is_ok());

        vessel.pending_co2_transfer_mol = 2e-6;
        let posed = partition(&vessel);
        assert_eq!(posed.as_ref().unwrap().clock_co2_applied_mol, 2e-6);
        assert!(validate_clock_boundary(&vessel, posed.as_ref()).is_ok());
        let mut ambiguous = posed.unwrap();
        ambiguous.totals.push(("C(4)".into(), 1e-7));
        assert!(validate_clock_boundary(&vessel, Some(&ambiguous)).is_err());

        vessel.pending_co2_transfer_mol = -2e-6;
        assert!(validate_clock_boundary(&vessel, partition(&vessel).as_ref()).is_err());
        vessel.pending_co2_transfer_mol = f64::NAN;
        assert!(validate_clock_boundary(&vessel, partition(&vessel).as_ref()).is_err());
    }
}

#[derive(Clone, Copy)]
enum AqueousReason {
    QueuedInputBinding,
    QueuedPoseValidation,
    PendingSelectorAuthority,
    SecondarySkipUnverified,
    SecondaryProbeUnverified,
    SecondaryChangedAfterReplay,
    SecondaryChangedWithoutFloor,
    FirstPoseSeedValidation,
    PassCap,
    PendingPoseIneligible,
    ForcedRouteValidation,
    SecondarySkipMismatch,
    SecondaryProbeMismatch,
    CertifiedSolverFailure,
    CertifiedAnswerMissing,
    ClockBoundaryMissing,
    ClockParcelMismatch,
    ReentryCertificateBoundaryCheck,
    ReservoirSetChanged,
    ReentryAnswerIncomplete,
    PendingOwnerNotReservoir,
    ModelTransitionUnsupported,
    PairedAuxiliaryIncomplete,
    StrictClosureMissing,
    PendingAuthorityInconsistent,
    ClockParcelNonfinite,
    ClockBoundaryInvalid,
    ClockCarbonAmbiguous,
    PhysicalPortionInvalid,
    DiagnosticMissing,
    DiagnosticNonfinite,
    PendingAcceptanceBlocked,
    PrimaryProgressPrecondition,
    OwnershipDatasetHeatChanged,
    CompleteStateMissing,
    ProvenanceMissing,
    RouteChanged,
    RedoxCoordinateInvalid,
    CoordinateNonfinite,
    DiagnosticRequiredMissing,
    SolventProvenanceChangedOrMissing,
    SecondOpinionNonfinite,
    SecondOpinionProvenanceMissing,
    StockNonfinite,
    HeatNonfinite,
    GasWrongVessel,
    GasAmountInvalid,
    NestedSolverFailure,
    AcceptedProvenanceMissing,
    MineralTransferNonfinite,
    NeutralisationNonfinite,
    ClockProjectionMissing,
    ReservoirOwnerChanged,
    ReplayRepresentationUnsupported,
    QuestionMissing,
    AuxiliaryResultAuthority,
    PrimarySolutionMissing,
    PrimaryProvenanceMissing,
    PrimaryRoutingMissing,
    FingerprintUnavailable,
    ComponentNamespaceUnavailable,
    ComponentFingerprintUnavailable,
    #[cfg(feature = "engine")]
    PinnedInstanceLoadFailed,
    #[cfg(feature = "engine")]
    ReusedEngineResetFailed,
    #[cfg(not(feature = "engine"))]
    CacheMiss,
    RedoxBalanceOutsideWater,
    RedoxReadbackUnavailable,
    RedoxSearchDidNotConverge,
    RedoxTotalsMissing,
    WaterRelationUndefined,
    SurfaceCapacityInvalid,
    ExchangeCapacityInvalid,
    MixedCrystalUnsupported,
    SolidSolutionStateInvalid,
    ThermalPassCap,
    ActivityModelUnavailable,
    InterfaceLedgerUnsupported,
    DiluteExchangeDomain,
    DiluteCrystalDomain,
    SolidSolutionPhaseUnavailable,
    GasPhaseUnavailable,
    ReadbackUnrepresented,
    ExchangeReadbackInvalid,
    ComponentReadbackInvalid,
    SelectedNumberInvalid,
    SolventMassInvalid,
    MeasuredCoordinateInvalid,
    Co2PressureInvalid,
    WaterActivityInvalid,
}

impl AqueousReason {
    fn phrase(self) -> Phrase {
        let (key, en) = match self {
            Self::QueuedInputBinding => (
                "solver-reason.aqueous.aux-input-binding",
                "The queued auxiliary input no longer matches its exact database and input binding.",
            ),
            Self::QueuedPoseValidation => (
                "solver-reason.aqueous.aux-pose-validation",
                "The queued auxiliary answer failed exact primary-pose or component-owner validation.",
            ),
            Self::PendingSelectorAuthority => (
                "solver-reason.aqueous.aux-pending-selector",
                "Pending auxiliary work has no selector authority.",
            ),
            Self::SecondarySkipUnverified => (
                "solver-reason.aqueous.secondary-skip-unverified",
                "The ordinary selector did not verify the provisional secondary skip.",
            ),
            Self::SecondaryProbeUnverified => (
                "solver-reason.aqueous.secondary-probe-unverified",
                "The ordinary selector did not admit the provisional secondary probe.",
            ),
            Self::SecondaryChangedAfterReplay => (
                "solver-reason.aqueous.secondary-changed-after-replay",
                "The secondary selector changed after its owner replay.",
            ),
            Self::SecondaryChangedWithoutFloor => (
                "solver-reason.aqueous.secondary-changed-without-floor",
                "The secondary solvent selector changed without a witnessed input-floor replay.",
            ),
            Self::FirstPoseSeedValidation => (
                "solver-reason.aqueous.first-pose-seed-validation",
                "The first aqueous seed failed its witnessed pose and ownership contract.",
            ),
            Self::PassCap => (
                "solver-reason.aqueous.pass-cap",
                "The physical aqueous and gas answer did not close within the canonical pass limit.",
            ),
            Self::PendingPoseIneligible => (
                "solver-reason.aqueous.pending-pose-ineligible",
                "Only a complete private reservoir pose may retain pending auxiliary work.",
            ),
            Self::ForcedRouteValidation => (
                "solver-reason.aqueous.forced-route-validation",
                "The forced replay did not satisfy the requested route and complete model-domain checks.",
            ),
            Self::SecondarySkipMismatch => (
                "solver-reason.aqueous.secondary-skip-mismatch",
                "The secondary replay did not skip its first pose.",
            ),
            Self::SecondaryProbeMismatch => (
                "solver-reason.aqueous.secondary-probe-mismatch",
                "The secondary replay did not return a valid provisional auxiliary answer.",
            ),
            Self::CertifiedSolverFailure => (
                "solver-reason.aqueous.certified-solver-failure",
                "The first gas-aware aqueous answer reported a solver failure.",
            ),
            Self::CertifiedAnswerMissing => (
                "solver-reason.aqueous.certified-answer-missing",
                "Clock CO2 or model replay lacks a complete certified native answer.",
            ),
            Self::ClockBoundaryMissing => (
                "solver-reason.aqueous.clock-boundary-missing",
                "The opaque aqueous owner has no spent clock boundary.",
            ),
            Self::ClockParcelMismatch => (
                "solver-reason.aqueous.clock-parcel-mismatch",
                "The clock CO2 parcel does not match its typed input.",
            ),
            Self::ReentryCertificateBoundaryCheck => (
                "solver-reason.aqueous.reentry-certificate-boundary-check",
                "The aqueous reentry failed its certificate, clock-boundary, or nested-answer check.",
            ),
            Self::ReservoirSetChanged => (
                "solver-reason.aqueous.reservoir-set-changed",
                "The aqueous reservoir set changed unexpectedly during reentry.",
            ),
            Self::ReentryAnswerIncomplete => (
                "solver-reason.aqueous.reentry-answer-incomplete",
                "The route-changing aqueous reentry lacks a complete native answer.",
            ),
            Self::PendingOwnerNotReservoir => (
                "solver-reason.aqueous.pending-owner-not-reservoir",
                "A non-reservoir owner retained pending auxiliary work.",
            ),
            Self::ModelTransitionUnsupported => (
                "solver-reason.aqueous.model-transition-unsupported",
                "The aqueous model transition is unsupported.",
            ),
            Self::PairedAuxiliaryIncomplete => (
                "solver-reason.aqueous.paired-auxiliary-incomplete",
                "The paired auxiliary work was not fulfilled.",
            ),
            Self::StrictClosureMissing => (
                "solver-reason.aqueous.strict-closure-missing",
                "The deferred reservoir answer lacks strict physical closure.",
            ),
            Self::PendingAuthorityInconsistent => (
                "solver-reason.aqueous.pending-authority-inconsistent",
                "The queued auxiliary authority is inconsistent.",
            ),
            Self::ClockParcelNonfinite => (
                "solver-reason.aqueous.clock-parcel-nonfinite",
                "The pending clock CO2 parcel is non-finite.",
            ),
            Self::ClockBoundaryInvalid => (
                "solver-reason.aqueous.clock-boundary-invalid",
                "The pending clock CO2 parcel is unsupported or exceeds available carbon.",
            ),
            Self::ClockCarbonAmbiguous => (
                "solver-reason.aqueous.clock-carbon-ambiguous",
                "Clock CO2 cannot be assigned to one unambiguous carbon component.",
            ),
            Self::PhysicalPortionInvalid => (
                "solver-reason.aqueous.physical-portion-invalid",
                "A physical portion in aqueous reentry is invalid.",
            ),
            Self::DiagnosticMissing => (
                "solver-reason.aqueous.diagnostic-missing",
                "A required current aqueous diagnostic is missing.",
            ),
            Self::DiagnosticNonfinite => (
                "solver-reason.aqueous.diagnostic-nonfinite",
                "A current aqueous diagnostic is non-finite.",
            ),
            Self::PendingAcceptanceBlocked => (
                "solver-reason.aqueous.pending-acceptance-blocked",
                "Pending auxiliary authority cannot earn physical acceptance.",
            ),
            Self::PrimaryProgressPrecondition => (
                "solver-reason.aqueous.primary-progress-precondition",
                "Primary progress requires two pending reservoir poses.",
            ),
            Self::OwnershipDatasetHeatChanged => (
                "solver-reason.aqueous.ownership-dataset-heat-changed",
                "Aqueous ownership, dataset, or heat mode changed during canonical reentry.",
            ),
            Self::CompleteStateMissing => (
                "solver-reason.aqueous.complete-state-missing",
                "A complete aqueous state is missing during physical comparison.",
            ),
            Self::ProvenanceMissing => (
                "solver-reason.aqueous.provenance-missing",
                "Aqueous provenance is missing during physical comparison.",
            ),
            Self::RouteChanged => (
                "solver-reason.aqueous.route-changed",
                "The aqueous engine, dataset, or model changed during canonical reentry.",
            ),
            Self::RedoxCoordinateInvalid => (
                "solver-reason.aqueous.redox-coordinate-invalid",
                "A typed aqueous redox coordinate is invalid.",
            ),
            Self::CoordinateNonfinite => (
                "solver-reason.aqueous.coordinate-nonfinite",
                "A canonical aqueous coordinate is non-finite.",
            ),
            Self::DiagnosticRequiredMissing => (
                "solver-reason.aqueous.diagnostic-required-missing",
                "A required aqueous diagnostic is missing.",
            ),
            Self::SolventProvenanceChangedOrMissing => (
                "solver-reason.aqueous.solvent-provenance-changed-or-missing",
                "Solvent-activity provenance is missing or its dataset or model changed during canonical reentry.",
            ),
            Self::SecondOpinionNonfinite => (
                "solver-reason.aqueous.second-opinion-nonfinite",
                "A second-opinion aqueous diagnostic is non-finite.",
            ),
            Self::SecondOpinionProvenanceMissing => (
                "solver-reason.aqueous.second-opinion-provenance-missing",
                "Solvent activity provenance is missing.",
            ),
            Self::StockNonfinite => ("solver-reason.aqueous.stock-nonfinite", "Physical stock is non-finite."),
            Self::HeatNonfinite => ("solver-reason.aqueous.heat-nonfinite", "Whole-step aqueous heat is non-finite."),
            Self::GasWrongVessel => (
                "solver-reason.aqueous.gas-wrong-vessel",
                "A gas crossing belongs to a different vessel.",
            ),
            Self::GasAmountInvalid => (
                "solver-reason.aqueous.gas-amount-invalid",
                "A native gas crossing amount is invalid.",
            ),
            Self::NestedSolverFailure => (
                "solver-reason.aqueous.nested-solver-failure",
                "A nested aqueous solver reported failure.",
            ),
            Self::AcceptedProvenanceMissing => (
                "solver-reason.aqueous.accepted-provenance-missing",
                "The accepted aqueous state lacks provenance.",
            ),
            Self::MineralTransferNonfinite => (
                "solver-reason.aqueous.mineral-transfer-nonfinite",
                "Cumulative mineral transfer is non-finite.",
            ),
            Self::NeutralisationNonfinite => (
                "solver-reason.aqueous.neutralisation-nonfinite",
                "Whole-call neutralisation is non-finite.",
            ),
            Self::ClockProjectionMissing => (
                "solver-reason.aqueous.clock-projection-missing",
                "Clock CO2 lacks a certified native component projection.",
            ),
            Self::ReservoirOwnerChanged => (
                "solver-reason.aqueous.reservoir-owner-changed",
                "The reservoir opaque aqueous owner changed after native readback.",
            ),
            Self::ReplayRepresentationUnsupported => (
                "solver-reason.aqueous.replay-representation-unsupported",
                "The replay route cannot represent the original analytical and phase owners.",
            ),
            Self::QuestionMissing => (
                "solver-reason.aqueous.question-missing",
                "There is no queued auxiliary question to fulfil.",
            ),
            Self::AuxiliaryResultAuthority => (
                "solver-reason.aqueous.auxiliary-result-authority",
                "The native auxiliary result is still pending or its provenance does not match its status.",
            ),
            Self::PrimarySolutionMissing => (
                "solver-reason.aqueous.primary-solution-missing",
                "The queued auxiliary pose lacks its primary solution.",
            ),
            Self::PrimaryProvenanceMissing => (
                "solver-reason.aqueous.primary-provenance-missing",
                "The queued auxiliary pose lacks its primary provenance.",
            ),
            Self::PrimaryRoutingMissing => (
                "solver-reason.aqueous.primary-routing-missing",
                "The queued primary routing lacks its native recipe.",
            ),
            Self::FingerprintUnavailable => (
                "solver-reason.aqueous.fingerprint-unavailable",
                "The exact auxiliary database fingerprint is unavailable.",
            ),
            Self::ComponentNamespaceUnavailable => (
                "solver-reason.aqueous.component-namespace-unavailable",
                "The selected database namespace for component isolation could not be prepared.",
            ),
            Self::ComponentFingerprintUnavailable => (
                "solver-reason.aqueous.component-fingerprint-unavailable",
                "The selected database fingerprint for component isolation could not be prepared.",
            ),
            #[cfg(feature = "engine")]
            Self::PinnedInstanceLoadFailed => (
                "solver-reason.aqueous.pinned-instance-load-failed",
                "The requested pinned native database instance could not be loaded.",
            ),
            #[cfg(feature = "engine")]
            Self::ReusedEngineResetFailed => (
                "solver-reason.aqueous.reused-engine-reset-failed",
                "The reused native engine could not be reset before the next solve.",
            ),
            #[cfg(not(feature = "engine"))]
            Self::CacheMiss => (
                "solver-reason.aqueous.cache-only-miss",
                "This solver has no live engine for the requested state, and no cached result is available.",
            ),
            Self::RedoxBalanceOutsideWater => (
                "solver-reason.aqueous.redox-balance-outside-water",
                "No electron activity in the admitted water stability field balances the typed redox inventory.",
            ),
            Self::RedoxReadbackUnavailable => (
                "solver-reason.aqueous.redox-readback-unavailable",
                "The redox search did not obtain a valid oxidation-state balance readback.",
            ),
            Self::RedoxSearchDidNotConverge => (
                "solver-reason.aqueous.redox-search-did-not-converge",
                "The coupled electron-balance search did not converge.",
            ),
            Self::RedoxTotalsMissing => (
                "solver-reason.aqueous.redox-totals-missing",
                "The coupled aqueous run returned no oxidation-state totals.",
            ),
            Self::WaterRelationUndefined => (
                "solver-reason.aqueous.water-relation-undefined",
                "The water-dissociation relation is undefined at this temperature.",
            ),
            Self::SurfaceCapacityInvalid => (
                "solver-reason.aqueous.surface-capacity-invalid",
                "The typed surface geometry, capacity, or occupancy is invalid.",
            ),
            Self::ExchangeCapacityInvalid => (
                "solver-reason.aqueous.exchange-capacity-invalid",
                "The typed exchange support, capacity, or occupancy is invalid.",
            ),
            Self::MixedCrystalUnsupported => (
                "solver-reason.aqueous.mixed-crystal-unsupported",
                "This solver supports one mixed crystalline phase per vessel.",
            ),
            Self::SolidSolutionStateInvalid => (
                "solver-reason.aqueous.solid-solution-state-invalid",
                "The typed solid solution lacks a finite nonnegative amount for each reviewed end member.",
            ),
            Self::ThermalPassCap => (
                "solver-reason.aqueous.thermal-pass-cap",
                "The coupled aqueous temperature did not converge within the thermal pass limit.",
            ),
            Self::ActivityModelUnavailable => (
                "solver-reason.aqueous.activity-model-unavailable",
                "No approved dataset currently covers this activity model together with the typed interface owner.",
            ),
            Self::InterfaceLedgerUnsupported => (
                "solver-reason.aqueous.interface-ledger-unsupported",
                "The selected interface complexes cannot be retained on the typed ledger.",
            ),
            Self::DiluteExchangeDomain => (
                "solver-reason.aqueous.dilute-exchange-domain",
                "Typed cation exchange is supported only for dilute inorganic water through the reviewed dataset.",
            ),
            Self::DiluteCrystalDomain => (
                "solver-reason.aqueous.dilute-crystal-domain",
                "The typed solid-solution model is supported only for dilute inorganic water through the reviewed dataset.",
            ),
            Self::SolidSolutionPhaseUnavailable => (
                "solver-reason.aqueous.solid-solution-phase-unavailable",
                "The routed database does not define a reviewed solid-solution end member.",
            ),
            Self::GasPhaseUnavailable => (
                "solver-reason.aqueous.gas-phase-unavailable",
                "The routed database does not define a reviewed phase for a gas in this headspace.",
            ),
            Self::ReadbackUnrepresented => (
                "solver-reason.aqueous.readback-unrepresented",
                "The native readback contains elemental states that cannot be represented by the typed aqueous inventory.",
            ),
            Self::ExchangeReadbackInvalid => (
                "solver-reason.aqueous.exchange-readback-invalid",
                "The native exchange occupancy is inconsistent with its finite capacity.",
            ),
            Self::ComponentReadbackInvalid => (
                "solver-reason.aqueous.component-readback-invalid",
                "The native readback returned an invalid typed component amount.",
            ),
            Self::SelectedNumberInvalid => (
                "solver-reason.aqueous.selected-number-invalid",
                "A required selected-output number is missing, malformed, non-finite, or negative.",
            ),
            Self::SolventMassInvalid => (
                "solver-reason.aqueous.solvent-mass-invalid",
                "The numerical solvent mass is invalid.",
            ),
            Self::MeasuredCoordinateInvalid => (
                "solver-reason.aqueous.measured-coordinate-invalid",
                "A measured aqueous coordinate is invalid.",
            ),
            Self::Co2PressureInvalid => (
                "solver-reason.aqueous.co2-pressure-invalid",
                "The numerical CO2 pressure is invalid.",
            ),
            Self::WaterActivityInvalid => (
                "solver-reason.aqueous.water-activity-invalid",
                "The numerical water activity is missing or invalid.",
            ),
        };
        Phrase::bare(key, en)
    }
}

fn canonical_closure_error(reason: AqueousReason, detail: impl Into<String>) -> SolveError {
    SolveError::NotConverged {
        reason: Some(Box::new(reason.phrase())),
        solver: "phreeqc-canonical".into(),
        detail: detail.into(),
    }
}

/// A CO2 owner can disappear only when the *native inorganic C component*
/// has reached the existing raw portion floor. Organic analytical keys such
/// as Acetate and Lactate contain carbon atoms but do not pose native C.
/// Positive mineral and retained-gas inventories are checked separately;
/// the later pass must still certify cumulative physical C/H/O/charge.
fn reviewed_co2_exhaustion(
    previous: &BTreeSet<String>,
    current: &BTreeSet<String>,
    vessel: &Vessel,
) -> bool {
    if !previous.contains("CO2")
        || current.contains("CO2")
        || current.len() + 1 != previous.len()
        || !current.is_subset(previous)
    {
        return false;
    }
    let Some(problem) = partition(vessel) else {
        return false;
    };
    let is_native_carbon = |key: &str| key == "C" || key.starts_with("C(");
    let mut inorganic_carbon = 0.0;
    for (key, amount) in &problem.totals {
        if is_native_carbon(key) {
            if !amount.is_finite() || *amount < 0.0 {
                return false;
            }
            inorganic_carbon += amount;
        }
    }
    if !inorganic_carbon.is_finite() || !(0.0..=TRACE).contains(&inorganic_carbon) {
        return false;
    }
    for portion in &vessel.contents {
        if !portion.moles.0.is_finite() || portion.moles.0 < 0.0 {
            return false;
        }
        if portion.moles.0 <= TRACE {
            continue;
        }
        match portion.phase {
            Phase::Solid => match derived::role(&portion.species.0) {
                Some(DerivedRole::Mineral { phase, .. }) => {
                    let Some(mineral) = derived::phase_by_name(phase) else {
                        return false;
                    };
                    if mineral
                        .elements
                        .iter()
                        .any(|(key, amount)| is_native_carbon(key) && *amount > 0.0)
                    {
                        return false;
                    }
                }
                Some(DerivedRole::Dissolves(elements)) => {
                    if elements
                        .iter()
                        .any(|(key, amount)| is_native_carbon(key) && *amount > 0.0)
                    {
                        return false;
                    }
                }
                Some(DerivedRole::Solvent) => {}
                None => return false,
            },
            Phase::Gas => {
                let Some(formula) = species::lookup(&portion.species)
                    .and_then(|row| kerotakis_core::stoich::parse_formula(row.formula).ok())
                else {
                    return false;
                };
                if formula.counts.get("C").is_some_and(|count| *count > 0.0) {
                    return false;
                }
            }
            _ => {}
        }
    }
    true
}

fn physical_stock(vessel: &Vessel) -> Result<BTreeMap<(SpeciesId, Phase), f64>, SolveError> {
    let mut stock = BTreeMap::new();
    for portion in &vessel.contents {
        if !portion.moles.0.is_finite() || portion.moles.0 < 0.0 {
            return Err(canonical_closure_error(
                AqueousReason::PhysicalPortionInvalid,
                "invalid physical portion in aqueous reentry",
            ));
        }
        *stock
            .entry((portion.species.clone(), portion.phase))
            .or_insert(0.0) += portion.moles.0;
    }
    Ok(stock)
}

fn stable_optional_coordinate(
    before: Option<f64>,
    after: Option<f64>,
    tolerance: f64,
) -> Result<bool, SolveError> {
    match (before, after) {
        (Some(a), Some(b)) if a.is_finite() && b.is_finite() => Ok((b - a).abs() <= tolerance),
        (None, None) => Ok(true),
        (Some(_), Some(_)) => Err(canonical_closure_error(
            AqueousReason::DiagnosticNonfinite,
            "missing or nonfinite current aqueous diagnostic",
        )),
        _ => Err(canonical_closure_error(
            AqueousReason::DiagnosticMissing,
            "missing or nonfinite current aqueous diagnostic",
        )),
    }
}

fn canonical_physical_delta_closed(
    before: &Vessel,
    after: &Vessel,
    before_heat: &AqueousPass,
    after_heat: &AqueousPass,
) -> Result<bool, SolveError> {
    if before_heat.has_pending_secondary()? || after_heat.has_pending_secondary()? {
        return Err(canonical_closure_error(
            AqueousReason::PendingAcceptanceBlocked,
            "pending auxiliary authority cannot earn physical acceptance",
        ));
    }
    canonical_physical_delta(before, after, before_heat, after_heat, true)
}

/// Scheduling evidence only. This never clears Pending or grants acceptance;
/// both exact auxiliary answers and the full predicate remain mandatory.
fn canonical_primary_progress(
    before: &Vessel,
    after: &Vessel,
    before_heat: &AqueousPass,
    after_heat: &AqueousPass,
) -> Result<PrimaryProgress, SolveError> {
    if before_heat.component_certificate != ComponentCertificate::ReservoirWithOpaque
        || after_heat.component_certificate != ComponentCertificate::ReservoirWithOpaque
        || !before_heat.has_pending_secondary()?
        || !after_heat.has_pending_secondary()?
    {
        return Err(canonical_closure_error(
            AqueousReason::PrimaryProgressPrecondition,
            "primary progress requires two pending reservoir poses",
        ));
    }
    Ok(
        if canonical_physical_delta(before, after, before_heat, after_heat, false)? {
            PrimaryProgress::ReadyForAuxiliary
        } else {
            PrimaryProgress::Moving
        },
    )
}

fn canonical_physical_delta(
    before: &Vessel,
    after: &Vessel,
    before_heat: &AqueousPass,
    after_heat: &AqueousPass,
    compare_auxiliary: bool,
) -> Result<bool, SolveError> {
    if before.id != after.id
        || before.pending_co2_transfer_mol != after.pending_co2_transfer_mol
        || !before.surfaces.is_empty()
        || !after.surfaces.is_empty()
        || !before.exchanges.is_empty()
        || !after.exchanges.is_empty()
        || !before.solid_solutions.is_empty()
        || !after.solid_solutions.is_empty()
        || !before.material_objects.is_empty()
        || !after.material_objects.is_empty()
        || !before.adsorbed.is_empty()
        || !after.adsorbed.is_empty()
        || !before.electrodes.is_empty()
        || !after.electrodes.is_empty()
        || before_heat.db_tag != after_heat.db_tag
        || before_heat.heat_mode != after_heat.heat_mode
    {
        return Err(canonical_closure_error(
            AqueousReason::OwnershipDatasetHeatChanged,
            "aqueous ownership, dataset or heat mode changed during canonical reentry",
        ));
    }
    if before_heat.component_certificate == ComponentCertificate::ReservoirWithOpaque {
        crate::inventory::certify_opaque_reservoir_stock(before, after)?;
    } else if !before.unresolved_materials.is_empty() || !after.unresolved_materials.is_empty() {
        crate::inventory::certify_opaque_clock_stock(before, after)?;
    }
    let before_info = before
        .solution
        .as_ref()
        .filter(|info| info.scope == SolutionScope::Complete)
        .ok_or_else(|| {
            canonical_closure_error(
                AqueousReason::CompleteStateMissing,
                "missing complete preceding aqueous state",
            )
        })?;
    let after_info = after
        .solution
        .as_ref()
        .filter(|info| info.scope == SolutionScope::Complete)
        .ok_or_else(|| {
            canonical_closure_error(
                AqueousReason::CompleteStateMissing,
                "missing complete current aqueous state",
            )
        })?;
    let before_route = before_info.provenance.as_ref().ok_or_else(|| {
        canonical_closure_error(
            AqueousReason::ProvenanceMissing,
            "missing preceding aqueous provenance",
        )
    })?;
    let after_route = after_info.provenance.as_ref().ok_or_else(|| {
        canonical_closure_error(
            AqueousReason::ProvenanceMissing,
            "missing current aqueous provenance",
        )
    })?;
    if before_route.engine != after_route.engine
        || before_route.dataset != after_route.dataset
        || before_route.model != after_route.model
    {
        return Err(canonical_closure_error(
            AqueousReason::RouteChanged,
            "aqueous engine, dataset or model changed during canonical reentry",
        ));
    }
    let activity = |info: &SolutionInfo| {
        info.species
            .iter()
            .find(|species| species.name == "H2O")
            .map(|species| species.activity)
    };
    let mut stable = true;
    stable &= stable_optional_coordinate(before_info.pe, after_info.pe, 1e-6)?;
    let redox_coordinates =
        |info: &SolutionInfo| -> Result<BTreeMap<(String, i32), f64>, SolveError> {
            let mut coordinates = BTreeMap::new();
            for state in &info.redox {
                if !state.molality.is_finite()
                    || state.molality < 0.0
                    || coordinates
                        .insert((state.element.clone(), state.oxidation), state.molality)
                        .is_some()
                {
                    return Err(canonical_closure_error(
                        AqueousReason::RedoxCoordinateInvalid,
                        "invalid typed aqueous redox coordinate",
                    ));
                }
            }
            Ok(coordinates)
        };
    let previous_redox = redox_coordinates(before_info)?;
    let current_redox = redox_coordinates(after_info)?;
    for key in previous_redox.keys().chain(current_redox.keys()) {
        let previous = previous_redox.get(key).copied().unwrap_or(0.0);
        let current = current_redox.get(key).copied().unwrap_or(0.0);
        stable &= (current - previous).abs() <= 1e-10;
    }
    for (a, b, tolerance) in [
        (before.free_proton, after.free_proton, 1e-10),
        (before.free_hydroxide, after.free_hydroxide, 1e-10),
        (before.solute_charge, after.solute_charge, 1e-10),
        (before_info.ph, after_info.ph, 1e-6),
        (before_info.ionic_strength, after_info.ionic_strength, 1e-10),
    ] {
        if !a.is_finite() || !b.is_finite() {
            return Err(canonical_closure_error(
                AqueousReason::CoordinateNonfinite,
                "nonfinite canonical aqueous coordinate",
            ));
        }
        stable &= (b - a).abs() <= tolerance;
    }
    for (a, b, tolerance) in [
        (before_info.solvent_kg, after_info.solvent_kg, 1e-10),
        (
            before_info.particle_molality,
            after_info.particle_molality,
            1e-10,
        ),
        (activity(before_info), activity(after_info), 1e-10),
        (
            before.co2_partial_pressure_atm,
            after.co2_partial_pressure_atm,
            1e-10,
        ),
    ] {
        if a.is_none() || b.is_none() {
            return Err(canonical_closure_error(
                AqueousReason::DiagnosticRequiredMissing,
                "missing required aqueous diagnostic",
            ));
        }
        stable &= stable_optional_coordinate(a, b, tolerance)?;
    }
    if compare_auxiliary {
        if before_info
            .solvent_activity
            .as_ref()
            .map(|a| (&a.dataset, &a.model))
            != after_info
                .solvent_activity
                .as_ref()
                .map(|a| (&a.dataset, &a.model))
        {
            return Err(canonical_closure_error(
                AqueousReason::SolventProvenanceChangedOrMissing,
                "solvent activity model changed during canonical reentry",
            ));
        }
        match (&before_info.solvent_activity, &after_info.solvent_activity) {
            (Some(a), Some(b)) => {
                for (left, right) in [
                    (a.water_activity, b.water_activity),
                    (a.particle_molality, b.particle_molality),
                    (a.ionic_strength, b.ionic_strength),
                ] {
                    if !left.is_finite() || !right.is_finite() {
                        return Err(canonical_closure_error(
                            AqueousReason::SecondOpinionNonfinite,
                            "nonfinite second-opinion aqueous diagnostic",
                        ));
                    }
                    stable &= (right - left).abs() <= 1e-10;
                }
            }
            (None, None) => {}
            _ => {
                return Err(canonical_closure_error(
                    AqueousReason::SecondOpinionProvenanceMissing,
                    "missing solvent activity provenance",
                ));
            }
        }
    }
    let previous = physical_stock(before)?;
    let current = physical_stock(after)?;
    for key in previous.keys().chain(current.keys()) {
        let a = previous.get(key).copied().unwrap_or(0.0);
        let b = current.get(key).copied().unwrap_or(0.0);
        if !a.is_finite() || !b.is_finite() {
            return Err(canonical_closure_error(
                AqueousReason::StockNonfinite,
                "nonfinite physical stock",
            ));
        }
        stable &= (b - a).abs() <= 1e-10;
    }
    for heat in [before_heat.heat_quote_j, after_heat.heat_quote_j] {
        if !heat.is_finite() {
            return Err(canonical_closure_error(
                AqueousReason::HeatNonfinite,
                "nonfinite whole-step aqueous heat",
            ));
        }
    }
    stable &= (after_heat.heat_quote_j - before_heat.heat_quote_j).abs() <= 0.001;
    Ok(stable)
}

fn signed_external_gas(
    vessel: &Vessel,
    events: &[Event],
    gas_out: &mut BTreeMap<SpeciesId, f64>,
) -> Result<(), SolveError> {
    for event in events {
        let (id, moles, sign) = match event {
            Event::GasEvolved {
                vessel: owner,
                species,
                moles,
            } if *owner == vessel.id => (species, moles.0, 1.0),
            Event::GasAbsorbed {
                vessel: owner,
                species,
                moles,
            } if *owner == vessel.id => (species, moles.0, -1.0),
            Event::GasEvolved { .. } | Event::GasAbsorbed { .. } => {
                return Err(canonical_closure_error(
                    AqueousReason::GasWrongVessel,
                    "gas crossing has wrong vessel owner",
                ));
            }
            _ => continue,
        };
        if !moles.is_finite() || moles < 0.0 {
            return Err(canonical_closure_error(
                AqueousReason::GasAmountInvalid,
                "invalid native gas crossing",
            ));
        }
        *gas_out.entry(id.clone()).or_insert(0.0) += sign * moles;
    }
    Ok(())
}

fn net_gas_events(vessel: &Vessel, gas_out: &BTreeMap<SpeciesId, f64>) -> Vec<Event> {
    gas_out
        .iter()
        .filter_map(|(species, signed)| {
            if *signed > TRACE {
                Some(Event::GasEvolved {
                    vessel: vessel.id,
                    species: species.clone(),
                    moles: Moles(*signed),
                })
            } else if *signed < -TRACE {
                Some(Event::GasAbsorbed {
                    vessel: vessel.id,
                    species: species.clone(),
                    moles: Moles(-signed),
                })
            } else {
                None
            }
        })
        .collect()
}

fn closed_aqueous_events(
    before: &Vessel,
    after: &Vessel,
    passes: &[Event],
    gas_out: &BTreeMap<SpeciesId, f64>,
) -> Result<Vec<Event>, SolveError> {
    let mut final_events = Vec::new();
    let mut dissolved = BTreeMap::<SpeciesId, f64>::new();
    let mut characterised = None;
    let mut headspace = None;
    for event in passes {
        match event {
            Event::SolverFailed { .. } => {
                return Err(canonical_closure_error(
                    AqueousReason::NestedSolverFailure,
                    "nested aqueous solver failure",
                ));
            }
            Event::GasEvolved { .. }
            | Event::GasAbsorbed { .. }
            | Event::GasContained { .. }
            | Event::Neutralised { .. } => {}
            Event::Dissolved { species, moles, .. } => {
                *dissolved.entry(species.clone()).or_insert(0.0) += moles.0;
            }
            Event::Precipitated { species, moles, .. } => {
                *dissolved.entry(species.clone()).or_insert(0.0) -= moles.0;
            }
            Event::SolutionRouted { .. } => {}
            Event::SolutionCharacterized { .. } => characterised = Some(event.clone()),
            Event::HeadspaceEquilibrated { .. } => headspace = Some(event.clone()),
            _ => {
                if !final_events.contains(event) {
                    final_events.push(event.clone());
                }
            }
        }
    }
    // Inner routing explanations are provisional. The accepted physical
    // state owns the one explanation a reader can receive for this call.
    let final_provenance = after
        .solution
        .as_ref()
        .filter(|info| info.scope == SolutionScope::Complete)
        .and_then(|info| info.provenance.as_ref())
        .ok_or_else(|| {
            canonical_closure_error(
                AqueousReason::AcceptedProvenanceMissing,
                "accepted aqueous state lacks provenance",
            )
        })?;
    let final_source_key = final_provenance.source_key();
    if before.aqueous_routing_said.as_deref() != Some(final_source_key.as_str()) {
        final_events.push(Event::SolutionRouted {
            vessel: after.id,
            provenance: final_provenance.clone(),
        });
    }
    for (species, signed) in dissolved {
        if !signed.is_finite() {
            return Err(canonical_closure_error(
                AqueousReason::MineralTransferNonfinite,
                "nonfinite cumulative mineral transfer",
            ));
        }
        if signed > TRACE {
            final_events.push(Event::Dissolved {
                vessel: after.id,
                species,
                moles: Moles(signed),
            });
        } else if signed < -TRACE {
            final_events.push(Event::Precipitated {
                vessel: after.id,
                species,
                moles: Moles(-signed),
                dry: false,
            });
        }
    }
    final_events.extend(net_gas_events(after, gas_out));
    let original = physical_stock(before)?;
    let current = physical_stock(after)?;
    for ((species, phase), amount) in &current {
        if *phase != Phase::Gas {
            continue;
        }
        let added = amount
            - original
                .get(&(species.clone(), Phase::Gas))
                .copied()
                .unwrap_or(0.0);
        if added > TRACE {
            final_events.push(Event::GasContained {
                vessel: after.id,
                species: species.clone(),
                moles: Moles(added),
            });
        }
    }
    // The native inference is a whole-call charge cancellation, discounted
    // by actual CO2 that left or entered the retained headspace. Recompute
    // it once instead of summing iteration-specific guesses.
    let a_before = before.solute_charge;
    let a_after = after.solute_charge;
    let mut neutralised =
        0.5 * (a_before.abs() + (a_after - a_before).abs() - a_after.abs()).max(0.0);
    let escaped_co2 = gas_out
        .iter()
        .filter(|(species, _)| species.0 == "CO2")
        .map(|(_, signed)| (*signed).max(0.0))
        .sum::<f64>();
    let contained_co2 = final_events
        .iter()
        .filter_map(|event| match event {
            Event::GasContained { species, moles, .. } if species.0 == "CO2" => Some(moles.0),
            _ => None,
        })
        .sum::<f64>();
    neutralised -= neutralised.min(escaped_co2 + contained_co2);
    let supplied_base: f64 = before
        .contents
        .iter()
        .map(|portion| {
            enthalpy::DISSOCIATION
                .iter()
                .find(|(key, _)| *key == portion.species.0)
                .map(|(_, products)| {
                    products
                        .iter()
                        .filter(|(key, _)| *key == "OH-")
                        .map(|(_, coefficient)| coefficient * portion.moles.0)
                        .sum::<f64>()
                })
                .unwrap_or(0.0)
        })
        .sum();
    neutralised = neutralised
        .min((before.free_hydroxide.max(0.0) + supplied_base - after.free_hydroxide).max(0.0));
    if !neutralised.is_finite() {
        return Err(canonical_closure_error(
            AqueousReason::NeutralisationNonfinite,
            "nonfinite whole-call neutralisation",
        ));
    }
    if neutralised > 0.0 {
        final_events.push(Event::Neutralised {
            vessel: after.id,
            moles: Moles(neutralised),
        });
    }
    if let Some(mut event) = characterised {
        if let (
            Event::SolutionCharacterized {
                ph, ionic_strength, ..
            },
            Some(info),
        ) = (&mut event, &after.solution)
        {
            *ph = info.ph;
            *ionic_strength = info.ionic_strength;
        }
        final_events.push(event);
    }
    if let Some(mut event) = headspace {
        if let Event::HeadspaceEquilibrated {
            pressure,
            total_moles,
            ..
        } = &mut event
        {
            *pressure = after.pressure;
            *total_moles = after.gas_moles();
        }
        final_events.push(event);
    }
    Ok(final_events)
}

#[cfg(test)]
mod canonical_physical_closure_tests {
    use super::*;
    use kerotakis_core::{material::MaterialBasis, vessel::UnresolvedMaterialPortion, VesselId};

    fn state() -> Vessel {
        let mut vessel = Vessel::new(VesselId(0), "canonical control");
        vessel.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
        vessel.deposit(SpeciesId::new("HCO3-"), Moles(0.01), Phase::Aqueous);
        vessel.free_proton = 1e-10;
        vessel.free_hydroxide = 1e-7;
        vessel.solute_charge = -0.01;
        vessel.co2_partial_pressure_atm = Some(0.0004);
        vessel.solution = Some(SolutionInfo {
            scope: SolutionScope::Complete,
            solvent_kg: Some(0.09998),
            particle_molality: Some(0.1),
            pe: None,
            redox: Vec::new(),
            ph: 8.3,
            ionic_strength: 0.1,
            species: vec![SpeciesDetail {
                name: "H2O".into(),
                molality: 55.51,
                activity: 0.999,
            }],
            provenance: Some(Provenance::new(
                "PHREEQC",
                "wateq4f.dat",
                "Davies",
                Vec::new(),
                Phrase::bare("test.canonical", "canonical test route"),
            )),
            solvent_activity: None,
        });
        vessel
    }

    fn quote(joules: f64) -> AqueousPass {
        AqueousPass {
            events: Vec::new(),
            heat_quote_j: joules,
            heat_mode: HeatMode::Priced,
            component_certificate: ComponentCertificate::Represented,
            db_tag: "wateq4f",
            reservoirs: vec!["CO2".into()],
            clock_co2_applied_mol: 0.0,
            secondary_disposition: SecondaryDisposition::Declined(SecondaryDecline::Unsupported),
            pending_secondary: None,
        }
    }

    fn pending_state() -> Vessel {
        let mut vessel = state();
        vessel.unresolved_materials.push(UnresolvedMaterialPortion {
            material: "whole_milk".into(),
            recipe_id: "household/whole-milk-surrogate".into(),
            recipe_version: 1,
            basis: MaterialBasis::MassFraction,
            amount: 12.0,
            enzyme_hydrolysis: None,
            protein_denatured_fraction: 0.0,
        });
        vessel.step_start = Some(kerotakis_core::vessel::StepStart::capture(&vessel));
        vessel
    }

    /// Controlled protocol answer, not a native chemistry prediction. Even its
    /// queued question uses the actual preparer's capability/input/key logic.
    fn pending_pass(vessel: &Vessel, heat: f64) -> AqueousPass {
        let problem = partition(vessel).unwrap();
        let value = |column: &str| {
            if column == "mass_H2O" {
                Some(0.09998)
            } else {
                problem
                    .elements
                    .iter()
                    .any(|name| name == column)
                    .then_some(0.1)
            }
        };
        let PreparedSecondary::Question(question) = PhreeqcEquilibrator::prepare_secondary(
            vessel,
            &problem,
            "wateq4f",
            &value,
            SecondaryFirstHint::Auto,
        ) else {
            panic!("controlled supported carbon question must be prepared");
        };
        let mut pass = quote(heat);
        pass.component_certificate = ComponentCertificate::ReservoirWithOpaque;
        pass.secondary_disposition = SecondaryDisposition::Pending;
        pass.pending_secondary = Some(question);
        let witness = AuxiliaryPoseWitness::capture(vessel, &pass);
        pass.pending_secondary.as_mut().unwrap().witness = Some(witness);
        pass
    }

    fn controlled_secondary(activity: f64) -> SecondaryResult {
        let mut provenance = secondary_provenance();
        provenance.water_activity = activity;
        SecondaryResult {
            provenance: Some(provenance),
            disposition: SecondaryDisposition::Accepted,
        }
    }

    #[test]
    fn primary_progress_never_clears_pending_or_grants_full_acceptance() {
        let vessel = pending_state();
        let before = pending_pass(&vessel, 0.0);
        let after = pending_pass(&vessel, 0.0);
        assert_eq!(
            canonical_primary_progress(&vessel, &vessel, &before, &after).unwrap(),
            PrimaryProgress::ReadyForAuxiliary
        );
        assert!(before.has_pending_secondary().unwrap() && after.has_pending_secondary().unwrap());
        assert!(canonical_physical_delta_closed(&vessel, &vessel, &before, &after).is_err());

        let mut caller = vessel.clone();
        let original = serde_json::to_vec(&caller).unwrap();
        let error = PhreeqcEquilibrator::solve_at_temperature_using(
            &mut caller,
            AuxiliaryPolicy::DeferReservoir,
            |work| match work {
                AqueousWork::Pose { candidate, .. } => {
                    let mut pass = pending_pass(candidate, 0.0);
                    pass.component_certificate = ComponentCertificate::Represented;
                    Ok(pass)
                }
                AqueousWork::Fulfil { .. } => panic!("wrong owner must fail before fulfilment"),
            },
        )
        .expect_err("a represented owner cannot retain a queued reservoir answer");
        assert!(matches!(error, SolveError::NotConverged { .. }));
        assert_eq!(serde_json::to_vec(&caller).unwrap(), original);
    }

    #[test]
    fn queued_auxiliary_refuses_wrong_temperature_before_native_dispatch() {
        let vessel = pending_state();
        let pass = pending_pass(&vessel, 0.0);
        let question = pass.pending_secondary.as_ref().unwrap();
        question.validate_pose(&vessel, &pass).unwrap();
        let mut different = vessel.clone();
        different.temperature.0 += 0.01;
        assert!(question.validate_pose(&different, &pass).is_err());
    }

    #[test]
    fn queued_auxiliary_refuses_wrong_stock_and_whole_step_heat_origin() {
        let vessel = pending_state();
        let mut pass = pending_pass(&vessel, 0.0);
        let mut different = vessel.clone();
        different.contents[0].moles.0 += 1e-8;
        assert!(pass
            .pending_secondary
            .as_ref()
            .unwrap()
            .validate_pose(&different, &pass)
            .is_err());
        let mut different_origin = vessel.clone();
        different_origin
            .step_start
            .as_mut()
            .unwrap()
            .gas_out
            .push((SpeciesId::new("CO2"), Moles(1e-8)));
        assert!(pass
            .pending_secondary
            .as_ref()
            .unwrap()
            .validate_pose(&different_origin, &pass)
            .is_err());
        let mut different_heat_owner = vessel.clone();
        different_heat_owner.aqueous_heat =
            kerotakis_core::vessel::AqueousHeatOwnership::UnmodeledContinuation;
        assert!(pass
            .pending_secondary
            .as_ref()
            .unwrap()
            .validate_pose(&different_heat_owner, &pass)
            .is_err());
        pass.heat_quote_j = 1.0;
        assert!(pass
            .pending_secondary
            .as_ref()
            .unwrap()
            .validate_pose(&vessel, &pass)
            .is_err());
    }

    #[test]
    fn queued_auxiliary_refuses_database_fingerprint_input_or_key_drift() {
        let vessel = pending_state();
        for field in ["fingerprint", "input", "key"] {
            let mut pass = pending_pass(&vessel, 0.0);
            let question = pass.pending_secondary.as_mut().unwrap();
            match field {
                "fingerprint" => question.database_fingerprint.push('0'),
                "input" => question.input.push_str("# different question\n"),
                "key" => question.key.push('0'),
                _ => unreachable!(),
            }
            assert!(question.validate_input().is_err());
        }
    }

    #[test]
    fn both_adjacent_queued_answers_are_completed_before_full_acceptance_and_routing() {
        let mut vessel = pending_state();
        let mut operations = Vec::new();
        let (events, _, closed, _) = PhreeqcEquilibrator::solve_at_temperature_using(
            &mut vessel,
            AuxiliaryPolicy::DeferReservoir,
            |work| match work {
                AqueousWork::Pose {
                    candidate,
                    auxiliary_policy,
                    ..
                } => {
                    assert_eq!(auxiliary_policy, AuxiliaryPolicy::DeferReservoir);
                    operations.push("pose");
                    Ok(pending_pass(candidate, 0.0))
                }
                AqueousWork::Fulfil { candidate, pass } => {
                    operations.push("fulfil");
                    PhreeqcEquilibrator::complete_secondary_result(
                        candidate,
                        pass,
                        controlled_secondary(0.9989),
                    )
                }
            },
        )
        .unwrap();
        assert!(closed);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, Event::SolutionRouted { .. }))
                .count(),
            1
        );
        assert_eq!(operations, ["pose", "pose", "fulfil", "fulfil"]);
        let info = vessel.solution.as_ref().unwrap();
        assert_eq!(
            info.solvent_activity.as_ref().unwrap().water_activity,
            0.9989
        );
        let provenance = info.provenance.as_ref().unwrap();
        assert!(provenance.routing.contains("second speciation"));
        assert_eq!(provenance.routing, provenance.routing_in(Locale::EN));
        assert_eq!(vessel.aqueous_routing_said, Some(provenance.source_key()));
    }

    #[test]
    fn actual_paired_auxiliary_delta_failure_replays_eagerly_and_keeps_global_cap_rollback() {
        for coordinate in ["water_activity", "particle_molality", "ionic_strength"] {
            let mut vessel = pending_state();
            let original = serde_json::to_vec(&vessel).unwrap();
            let mut poses: usize = 0;
            let mut fulfilments: usize = 0;
            let mut eager_poses: usize = 0;
            let failure = PhreeqcEquilibrator::solve_at_temperature_using(
                &mut vessel,
                AuxiliaryPolicy::DeferReservoir,
                |work| match work {
                    AqueousWork::Pose {
                        candidate,
                        auxiliary_policy,
                        ..
                    } => {
                        poses += 1;
                        if auxiliary_policy == AuxiliaryPolicy::DeferReservoir {
                            candidate.solution.as_mut().unwrap().solvent_activity = None;
                            Ok(pending_pass(candidate, 0.0))
                        } else {
                            eager_poses += 1;
                            candidate.free_proton =
                                if poses.is_multiple_of(2) { 0.0 } else { 1e-9 };
                            let mut pass = quote(0.0);
                            pass.component_certificate = ComponentCertificate::ReservoirWithOpaque;
                            Ok(pass)
                        }
                    }
                    AqueousWork::Fulfil { candidate, pass } => {
                        fulfilments += 1;
                        let mut result = controlled_secondary(0.9989);
                        if fulfilments == 2 {
                            let auxiliary = result.provenance.as_mut().unwrap();
                            match coordinate {
                                "water_activity" => auxiliary.water_activity += 2e-9,
                                "particle_molality" => auxiliary.particle_molality += 2e-9,
                                "ionic_strength" => auxiliary.ionic_strength += 2e-9,
                                _ => unreachable!(),
                            }
                        }
                        PhreeqcEquilibrator::complete_secondary_result(candidate, pass, result)
                    }
                },
            )
            .expect_err("primary stability cannot excuse an actual auxiliary mismatch");
            assert!(matches!(failure, SolveError::NotConverged { .. }));
            assert_eq!(poses, MAX_CANONICAL_AQUEOUS_PASSES);
            assert_eq!(fulfilments, 2);
            assert!(eager_poses > 0);
            assert_eq!(serde_json::to_vec(&vessel).unwrap(), original);
        }
    }

    #[test]
    fn pending_floor_transition_replays_eagerly_from_original_owner() {
        let mut vessel = pending_state();
        let original = vessel.clone();
        let mut poses: usize = 0;
        let (_, _, closed, _) = PhreeqcEquilibrator::solve_at_temperature_using(
            &mut vessel,
            AuxiliaryPolicy::DeferReservoir,
            |work| match work {
                AqueousWork::Pose {
                    candidate,
                    past,
                    owner,
                    hint,
                    auxiliary_policy,
                } => {
                    poses += 1;
                    if poses == 1 {
                        return Ok(pending_pass(candidate, 0.0));
                    }
                    if poses == 2 {
                        let mut pass = quote(0.0);
                        pass.component_certificate = ComponentCertificate::ReservoirWithOpaque;
                        pass.secondary_disposition = SecondaryDisposition::BelowInputFloor;
                        return Ok(pass);
                    }
                    assert_eq!(auxiliary_policy, AuxiliaryPolicy::Eager);
                    if poses == 3 {
                        assert_eq!(owner, ComponentCertificate::Uncertified);
                        assert_eq!(hint, FirstPoseHint::default());
                        assert!(past.is_empty());
                        assert_eq!(candidate.contents, original.contents);
                        assert_eq!(candidate.step_start, original.step_start);
                    }
                    let mut pass = quote(0.0);
                    pass.component_certificate = ComponentCertificate::ReservoirWithOpaque;
                    Ok(pass)
                }
                AqueousWork::Fulfil { .. } => panic!("a floor decision is not an invented answer"),
            },
        )
        .unwrap();
        assert!(closed);
        assert_eq!(poses, 4);
        assert_eq!(vessel.step_start, original.step_start);
    }

    #[test]
    fn old_controlled_adapter_cannot_invent_queued_auxiliary_answers() {
        let mut vessel = pending_state();
        let original = serde_json::to_vec(&vessel).unwrap();
        let seed = PhreeqcEquilibrator::first_pose_at_temperature_with(&vessel, |candidate| {
            Ok(pending_pass(candidate, 0.0))
        })
        .unwrap();
        let failure =
            PhreeqcEquilibrator::resume_first_pose_with(&mut vessel, seed, |candidate, _, _, _| {
                Ok(pending_pass(candidate, 0.0))
            })
            .expect_err("the old pose-only adapter has no authority to fulfil a native question");
        assert!(matches!(failure, SolveError::NotConverged { .. }));
        assert_eq!(serde_json::to_vec(&vessel).unwrap(), original);
    }

    #[test]
    fn auxiliary_decline_remains_typed_and_missing_result_cannot_be_accepted() {
        let mut vessel = pending_state();
        let pass = pending_pass(&vessel, 0.0);
        let resolved = PhreeqcEquilibrator::complete_secondary_result(
            &mut vessel,
            pass,
            SecondaryResult::declined(SecondaryDisposition::Declined(
                SecondaryDecline::EngineFailure,
            )),
        )
        .unwrap();
        assert_eq!(
            resolved.secondary_disposition,
            SecondaryDisposition::Declined(SecondaryDecline::EngineFailure)
        );
        assert!(!resolved.has_pending_secondary().unwrap());
        assert!(vessel.solution.as_ref().unwrap().solvent_activity.is_none());
        let pass = pending_pass(&vessel, 0.0);
        let original = serde_json::to_vec(&vessel).unwrap();
        assert!(PhreeqcEquilibrator::complete_secondary_result(
            &mut vessel,
            pass,
            SecondaryResult {
                provenance: None,
                disposition: SecondaryDisposition::Accepted
            },
        )
        .is_err());
        assert_eq!(serde_json::to_vec(&vessel).unwrap(), original);
    }

    #[test]
    fn resumed_first_pose_counts_toward_the_same_eight_pass_cap_and_rolls_back() {
        let mut vessel = state();
        vessel.step_start = Some(kerotakis_core::vessel::StepStart::capture(&vessel));
        let original = serde_json::to_vec(&vessel).unwrap();
        let mut calls: usize = 0;
        let mut solve = |candidate: &mut Vessel,
                         _past: &[Event],
                         _owner: ComponentCertificate,
                         _hint: FirstPoseHint| {
            calls += 1;
            candidate.free_proton = if calls.is_multiple_of(2) { 0.0 } else { 1e-9 };
            Ok(quote(0.0))
        };
        let seed = PhreeqcEquilibrator::first_pose_at_temperature_with(&vessel, |candidate| {
            solve(
                candidate,
                &[],
                ComponentCertificate::Uncertified,
                FirstPoseHint::default(),
            )
        })
        .unwrap();
        let error = PhreeqcEquilibrator::resume_first_pose_with(&mut vessel, seed, solve)
            .expect_err("the seed must not buy a ninth physical pose");
        assert!(matches!(error, SolveError::NotConverged { .. }));
        assert_eq!(calls, MAX_CANONICAL_AQUEOUS_PASSES);
        assert_eq!(serde_json::to_vec(&vessel).unwrap(), original);
    }

    #[test]
    fn resumed_first_pose_never_repeats_native_first_input_or_recaptures_heat_origin() {
        let mut vessel = state();
        vessel.step_start = Some(kerotakis_core::vessel::StepStart::capture(&vessel));
        let origin = vessel.step_start.clone();
        let mut calls = 0;
        let mut solve = |candidate: &mut Vessel,
                         past: &[Event],
                         owner: ComponentCertificate,
                         hint: FirstPoseHint| {
            calls += 1;
            assert!(past.is_empty());
            assert_eq!(hint, FirstPoseHint::default());
            assert_eq!(candidate.step_start, origin);
            assert_eq!(
                owner,
                if calls == 1 {
                    ComponentCertificate::Uncertified
                } else {
                    ComponentCertificate::Represented
                }
            );
            Ok(quote(0.0))
        };
        let seed = PhreeqcEquilibrator::first_pose_at_temperature_with(&vessel, |candidate| {
            solve(
                candidate,
                &[],
                ComponentCertificate::Uncertified,
                FirstPoseHint::default(),
            )
        })
        .unwrap();
        let (_, heat, closed, clock) =
            PhreeqcEquilibrator::resume_first_pose_with(&mut vessel, seed, solve).unwrap();
        assert_eq!(calls, 2);
        assert!(closed);
        assert_eq!(heat, 0.0);
        assert_eq!(clock, 0.0);
        assert_eq!(vessel.step_start, origin);
    }

    #[test]
    fn mismatched_first_pose_temperature_is_refused_without_committing() {
        let mut vessel = state();
        let original = serde_json::to_vec(&vessel).unwrap();
        let mut seed =
            PhreeqcEquilibrator::first_pose_at_temperature_with(&vessel, |_| Ok(quote(0.0)))
                .unwrap();
        seed.trial.temperature.0 += 0.01;
        let error = PhreeqcEquilibrator::resume_first_pose_with(&mut vessel, seed, |_, _, _, _| {
            panic!("a mismatched seed must not issue a reentry")
        })
        .expect_err("a seed belongs to exactly its original temperature");
        assert!(matches!(error, SolveError::NotConverged { .. }));
        assert_eq!(serde_json::to_vec(&vessel).unwrap(), original);
    }

    #[test]
    fn provisional_two_cycle_cannot_witness_final_strict_thermal_convergence() {
        let start = state();
        let guess = start.temperature.0;
        let next = guess + 2e-5;
        assert!(aqueous_thermal_pose_converged(
            &start,
            guess,
            next,
            None,
            None,
            Some(next),
        ));
        assert!(!aqueous_thermal_pose_converged(
            &start, guess, next, None, None, None,
        ));
    }

    #[test]
    fn final_strict_heat_and_volume_both_have_to_satisfy_the_thermal_map() {
        let mut start = state();
        start.headspace = Headspace::PressureControlled {
            pressure: kerotakis_core::Pascal(101325.0),
            volume: kerotakis_core::Liters(0.1),
        };
        let guess = start.temperature.0;
        let volume = start.headspace_volume();
        let provisional_next = aqueous_temperature_target(&start, &start, 0.0, true);
        assert!(aqueous_thermal_pose_converged(
            &start,
            guess,
            provisional_next,
            volume,
            volume,
            None,
        ));
        let strict_next = aqueous_temperature_target(&start, &start, 1.0, true);
        assert!(!aqueous_thermal_pose_converged(
            &start,
            guess,
            strict_next,
            volume,
            volume,
            None,
        ));
        assert!(!aqueous_thermal_pose_converged(
            &start,
            guess,
            provisional_next,
            volume,
            Some(kerotakis_core::Liters(0.11)),
            None,
        ));
    }

    #[test]
    fn reservoir_certificate_requires_current_native_numerical_coordinates() {
        // Pure admission predicate, not a synthetic chemistry result.
        let mut vessel = state();
        assert!(complete_reservoir_diagnostics(&vessel));
        vessel.solution.as_mut().unwrap().particle_molality = None;
        assert!(!complete_reservoir_diagnostics(&vessel));
        vessel.solution.as_mut().unwrap().particle_molality = Some(0.1);
        vessel.solution.as_mut().unwrap().scope = SolutionScope::SolventOnly;
        assert!(!complete_reservoir_diagnostics(&vessel));
        vessel.solution.as_mut().unwrap().scope = SolutionScope::Complete;
        vessel.solution.as_mut().unwrap().species.clear();
        assert!(!complete_reservoir_diagnostics(&vessel));
    }

    #[test]
    fn reservoir_opaque_owner_spends_global_eight_pass_cap_and_rolls_back() {
        // Controlled answer sequence exercises the same production bounded
        // driver that native reservoir answers enter. It is not a chemistry
        // forecast: alternating diagnostics are intentionally nonphysical.
        let mut vessel = state();
        vessel.unresolved_materials.push(UnresolvedMaterialPortion {
            material: "red_cabbage_juice".into(),
            recipe_id: "household/red-cabbage-juice-surrogate".into(),
            recipe_version: 1,
            basis: MaterialBasis::MassFraction,
            amount: 0.197,
            enzyme_hydrolysis: None,
            protein_denatured_fraction: 0.0,
        });
        let original = serde_json::to_vec(&vessel).unwrap();
        let mut calls: usize = 0;
        let failure = PhreeqcEquilibrator::solve_at_temperature_with(
            &mut vessel,
            |candidate, _, owner, _| {
                calls += 1;
                assert_eq!(
                    owner,
                    if calls == 1 {
                        ComponentCertificate::Uncertified
                    } else {
                        ComponentCertificate::ReservoirWithOpaque
                    }
                );
                candidate.free_proton = if calls.is_multiple_of(2) { 0.0 } else { 1e-9 };
                let mut pass = quote(0.0);
                pass.component_certificate = ComponentCertificate::ReservoirWithOpaque;
                Ok(pass)
            },
        )
        .expect_err("alternating physical diagnostics cannot close a reservoir answer");
        assert!(matches!(failure, SolveError::NotConverged { .. }));
        assert_eq!(calls, MAX_CANONICAL_AQUEOUS_PASSES);
        assert_eq!(serde_json::to_vec(&vessel).unwrap(), original);
    }

    #[cfg(feature = "engine")]
    #[test]
    fn original_bicarbonate_boundary_keeps_the_one_molal_route_selection() {
        let solver = PhreeqcEquilibrator::new().expect("native dataset constructor");
        let make_input = |bicarbonate_mol: f64| {
            let mut vessel = Vessel::new(VesselId(0), "original bicarbonate route boundary");
            vessel.deposit(
                SpeciesId::new("water"),
                Moles(5.534_276_991_396_059),
                Phase::Liquid,
            );
            vessel.deposit(
                SpeciesId::new("NaHCO3"),
                Moles(bicarbonate_mol),
                Phase::Solid,
            );
            vessel
        };
        let dilute = make_input(0.049);
        let concentrated = make_input(0.05);
        let dilute_problem = partition(&dilute).expect("represented dilute bicarbonate");
        let concentrated_problem =
            partition(&concentrated).expect("represented boundary bicarbonate");
        assert!((concentrated_problem.kgw - 0.0997).abs() < 1e-12);
        let dilute_potential = potential_molality(&dilute_problem, dilute.temperature.0);
        let concentrated_potential =
            potential_molality(&concentrated_problem, concentrated.temperature.0);
        assert!(
            dilute_potential < 1.0,
            "unchanged potential-load rule: {dilute_potential}"
        );
        assert!(
            concentrated_potential > 1.0,
            "unchanged potential-load rule: {concentrated_potential}"
        );
        let dilute_setup = solver.setup_problem(&dilute, None).unwrap().unwrap();
        let concentrated_setup = solver.setup_problem(&concentrated, None).unwrap().unwrap();
        assert_eq!(dilute_setup.db_tag, "wateq4f");
        assert_eq!(concentrated_setup.db_tag, "pitzer");
        assert!(concentrated_setup
            .problem
            .phases
            .iter()
            .any(|(phase, moles, _)| phase == "Nahcolite" && *moles > 0.0));
        assert_eq!(
            solver.engine_calls(),
            0,
            "test must inspect setup, not solve the state"
        );
    }

    #[test]
    fn changed_explanation_keeps_physical_authority_and_publishes_only_final_route() {
        let mut before = state();
        let first_provenance = before
            .solution
            .as_ref()
            .unwrap()
            .provenance
            .as_ref()
            .unwrap()
            .clone();
        before.aqueous_routing_said = Some(first_provenance.source_key());
        let mut after = before.clone();
        after
            .solution
            .as_mut()
            .unwrap()
            .provenance
            .as_mut()
            .unwrap()
            .say_routing(Phrase::bare(
                "test.canonical-final",
                "accepted final explanation",
            ));
        let final_provenance = after
            .solution
            .as_ref()
            .unwrap()
            .provenance
            .as_ref()
            .unwrap()
            .clone();
        after.aqueous_routing_said = Some(final_provenance.source_key());
        assert_ne!(first_provenance.source_key(), final_provenance.source_key());
        assert!(
            canonical_physical_delta_closed(&before, &after, &quote(0.0), &quote(0.0)).unwrap()
        );

        let provisional = Event::SolutionRouted {
            vessel: before.id,
            provenance: first_provenance,
        };
        let events =
            closed_aqueous_events(&before, &after, &[provisional], &BTreeMap::new()).unwrap();
        let routed = events
            .iter()
            .filter_map(|event| match event {
                Event::SolutionRouted { provenance, .. } => Some(provenance),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(routed, vec![&final_provenance]);

        let same_final = closed_aqueous_events(&after, &after, &[], &BTreeMap::new()).unwrap();
        assert!(!same_final
            .iter()
            .any(|event| matches!(event, Event::SolutionRouted { .. })));
    }

    #[test]
    fn actual_model_or_typed_electron_authority_cannot_change_under_narrative_cover() {
        let before = state();
        let mut changed = before.clone();
        changed
            .solution
            .as_mut()
            .unwrap()
            .provenance
            .as_mut()
            .unwrap()
            .model = "another model".into();
        assert!(
            canonical_physical_delta_closed(&before, &changed, &quote(0.0), &quote(0.0)).is_err()
        );

        let mut changed = before.clone();
        changed.solution.as_mut().unwrap().pe = Some(3.0);
        assert!(
            canonical_physical_delta_closed(&before, &changed, &quote(0.0), &quote(0.0)).is_err()
        );

        let mut before_redox = before.clone();
        before_redox
            .solution
            .as_mut()
            .unwrap()
            .redox
            .push(kerotakis_core::RedoxState {
                element: "Fe".into(),
                oxidation: 2,
                molality: 0.001,
            });
        let mut after_redox = before_redox.clone();
        after_redox.solution.as_mut().unwrap().redox[0].molality += 2e-10;
        assert!(!canonical_physical_delta_closed(
            &before_redox,
            &after_redox,
            &quote(0.0),
            &quote(0.0)
        )
        .unwrap());
    }

    #[test]
    fn only_exhausted_native_inorganic_carbon_can_remove_the_co2_owner() {
        let previous = BTreeSet::from(["CO2".to_string(), "O2".to_string()]);
        let without_co2 = BTreeSet::from(["O2".to_string()]);
        let without_o2 = BTreeSet::from(["CO2".to_string()]);
        let mut vessel = state();
        assert!(!reviewed_co2_exhaustion(&previous, &without_co2, &vessel));
        vessel
            .contents
            .retain(|portion| portion.species.0 != "HCO3-");
        vessel.deposit(SpeciesId::new("CH3COOH"), Moles(0.005), Phase::Liquid);
        let acetate_problem = partition(&vessel).unwrap();
        assert!(acetate_problem
            .totals
            .iter()
            .any(|(key, _)| key == "Acetate"));
        assert!(!acetate_problem
            .totals
            .iter()
            .any(|(key, _)| key == "C" || key.starts_with("C(")));
        assert!(reviewed_co2_exhaustion(&previous, &without_co2, &vessel));
        assert!(!reviewed_co2_exhaustion(&previous, &without_o2, &vessel));
        vessel.deposit(SpeciesId::new("HCO3-"), Moles(2.0 * TRACE), Phase::Aqueous);
        assert!(!reviewed_co2_exhaustion(&previous, &without_co2, &vessel));
        vessel
            .contents
            .retain(|portion| portion.species.0 != "HCO3-");
        vessel.deposit(SpeciesId::new("CaCO3"), Moles(2.0 * TRACE), Phase::Solid);
        assert!(!reviewed_co2_exhaustion(&previous, &without_co2, &vessel));
        vessel
            .contents
            .retain(|portion| portion.species.0 != "CaCO3");
        vessel.deposit(SpeciesId::new("CO2"), Moles(2.0 * TRACE), Phase::Gas);
        assert!(!reviewed_co2_exhaustion(&previous, &without_co2, &vessel));
        vessel
            .contents
            .retain(|portion| !(portion.species.0 == "CO2" && portion.phase == Phase::Gas));
        vessel.deposit(SpeciesId::new("CO2"), Moles(2.0 * TRACE), Phase::Solid);
        assert!(!reviewed_co2_exhaustion(&previous, &without_co2, &vessel));
    }

    #[test]
    fn individual_stock_and_global_heat_both_have_to_close() {
        let before = state();
        let mut small = before.clone();
        small.contents[0].moles.0 += 5e-11;
        small.solute_charge += 5e-11;
        small.solution.as_mut().unwrap().ph += 5e-7;
        assert!(
            canonical_physical_delta_closed(&before, &small, &quote(1.0), &quote(1.0005)).unwrap()
        );
        let mut moving = small.clone();
        moving.contents[1].moles.0 += 2e-10;
        assert!(
            !canonical_physical_delta_closed(&before, &moving, &quote(1.0), &quote(1.0005))
                .unwrap()
        );
        assert!(
            !canonical_physical_delta_closed(&before, &small, &quote(1.0), &quote(1.002)).unwrap()
        );
        small.solution.as_mut().unwrap().particle_molality = None;
        assert!(
            canonical_physical_delta_closed(&before, &small, &quote(1.0), &quote(1.0)).is_err()
        );
    }

    #[test]
    fn signed_gas_keeps_all_native_inner_passes() {
        let vessel = state();
        let evolved = Event::GasEvolved {
            vessel: vessel.id,
            species: SpeciesId::new("CO2"),
            moles: Moles(2e-8),
        };
        let absorbed = Event::GasAbsorbed {
            vessel: vessel.id,
            species: SpeciesId::new("CO2"),
            moles: Moles(1e-8),
        };
        let mut gas = BTreeMap::new();
        signed_external_gas(&vessel, &[evolved, absorbed], &mut gas).unwrap();
        assert_eq!(
            net_gas_events(&vessel, &gas),
            vec![Event::GasEvolved {
                vessel: vessel.id,
                species: SpeciesId::new("CO2"),
                moles: Moles(1e-8),
            }]
        );
    }

    #[test]
    fn bounded_driver_exhausts_and_restores_caller_on_unclosed_answers() {
        let mut vessel = state();
        vessel.pending_co2_transfer_mol = 5e-7;
        vessel.step_start = Some(kerotakis_core::vessel::StepStart::capture(&vessel));
        let original = serde_json::to_vec(&vessel).unwrap();
        let original_snapshot = vessel.step_start.clone();
        let mut calls: usize = 0;
        let failure = PhreeqcEquilibrator::solve_at_temperature_with(
            &mut vessel,
            |candidate, _prior_events, _certified_reentry, _forced_route| {
                calls += 1;
                candidate.free_proton = if calls.is_multiple_of(2) { 0.0 } else { 1e-9 };
                Ok(quote(0.0))
            },
        )
        .expect_err("alternating free-ion answers cannot close");
        assert!(matches!(failure, SolveError::NotConverged { .. }));
        assert_eq!(calls, MAX_CANONICAL_AQUEOUS_PASSES);
        assert_eq!(serde_json::to_vec(&vessel).unwrap(), original);
        assert_eq!(vessel.step_start, original_snapshot);
        assert_eq!(vessel.pending_co2_transfer_mol, 5e-7);
    }

    #[test]
    fn opaque_clock_owner_rejects_second_pass_mutation_and_restores_caller() {
        // Controlled answer sequence exercises the actual private driver,
        // not a native-chemistry prediction. The first answer owns its clock
        // carbon; the next answer changes only opaque recipe mass.
        let mut vessel = state();
        vessel.unresolved_materials.push(UnresolvedMaterialPortion {
            material: "whole_milk".into(),
            recipe_id: "household/whole-milk-surrogate".into(),
            recipe_version: 1,
            basis: MaterialBasis::MassFraction,
            amount: 12.0,
            enzyme_hydrolysis: None,
            protein_denatured_fraction: 0.0,
        });
        vessel.pending_co2_transfer_mol = 2e-6;
        vessel.step_start = Some(kerotakis_core::vessel::StepStart::capture(&vessel));
        let original = serde_json::to_vec(&vessel).unwrap();
        let mut calls = 0;
        let error = PhreeqcEquilibrator::solve_at_temperature_with(
            &mut vessel,
            |candidate, _past, owner, _route| {
                calls += 1;
                let mut pass = quote(0.0);
                pass.component_certificate = ComponentCertificate::ClockWithOpaque;
                if calls == 1 {
                    assert_eq!(owner, ComponentCertificate::Uncertified);
                    candidate.deposit(SpeciesId::new("CO2(aq)"), Moles(2e-6), Phase::Aqueous);
                    pass.clock_co2_applied_mol = 2e-6;
                } else {
                    assert_eq!(owner, ComponentCertificate::ClockWithOpaque);
                    candidate.unresolved_materials[0].amount += 0.001;
                }
                Ok(pass)
            },
        )
        .expect_err("opaque mass cannot change during canonical reentry");
        assert!(matches!(error, SolveError::NotConverged { .. }));
        assert_eq!(calls, 2);
        assert_eq!(serde_json::to_vec(&vessel).unwrap(), original);
    }

    #[test]
    fn route_replay_discards_first_models_gas_and_clock_then_starts_from_original_owner() {
        // Controlled driver sequence, not a claim about native chemistry.
        let mut vessel = state();
        vessel.deposit(SpeciesId::new("CO2(aq)"), Moles(1e-8), Phase::Aqueous);
        vessel.pending_co2_transfer_mol = 2e-6;
        vessel.step_start = Some(kerotakis_core::vessel::StepStart::capture(&vessel));
        let original = vessel.clone();
        let mut calls = 0;
        let (events, _, closed, clock) = PhreeqcEquilibrator::solve_at_temperature_with(
            &mut vessel,
            |candidate, prior, certified_reentry, hint| {
                calls += 1;
                let mut pass = quote(0.0);
                match calls {
                    1 => {
                        assert_eq!(certified_reentry, ComponentCertificate::Uncertified);
                        assert_eq!(hint, FirstPoseHint::default());
                        candidate
                            .contents
                            .retain(|portion| portion.species.0 != "CO2(aq)");
                        candidate.deposit(SpeciesId::new("CO2(aq)"), Moles(2e-6), Phase::Aqueous);
                        pass.db_tag = "pitzer";
                        pass.clock_co2_applied_mol = 2e-6;
                        pass.events.push(Event::GasEvolved {
                            vessel: candidate.id,
                            species: SpeciesId::new("CO2"),
                            moles: Moles(1e-8),
                        });
                    }
                    2 => {
                        assert!(certified_reentry.is_certified());
                        assert_eq!(hint, FirstPoseHint::default());
                        assert_eq!(candidate.pending_co2_transfer_mol, 0.0);
                        assert_eq!(prior.len(), 1);
                        assert_eq!(candidate.step_start.as_ref().unwrap().gas_out.len(), 1);
                        // WATEQ was discovered only on the Pitzer-modified
                        // hypothetical state. This answer must be discarded.
                    }
                    3 => {
                        assert_eq!(certified_reentry, ComponentCertificate::Uncertified);
                        assert_eq!(hint.primary_route, Some("wateq4f"));
                        assert_eq!(hint.secondary, SecondaryFirstHint::Auto);
                        assert!(prior.is_empty());
                        assert_eq!(candidate.contents, original.contents);
                        assert_eq!(
                            candidate.pending_co2_transfer_mol,
                            original.pending_co2_transfer_mol
                        );
                        assert_eq!(candidate.step_start, original.step_start);
                        candidate.deposit(SpeciesId::new("CO2(aq)"), Moles(2e-6), Phase::Aqueous);
                        pass.clock_co2_applied_mol = 2e-6;
                    }
                    4 => {
                        assert!(certified_reentry.is_certified());
                        assert_eq!(hint, FirstPoseHint::default());
                        assert!(prior.is_empty());
                        assert_eq!(candidate.pending_co2_transfer_mol, 0.0);
                        assert_eq!(candidate.step_start.as_ref().unwrap().gas_out.len(), 1);
                    }
                    _ => panic!("replay exceeded the expected four controlled queries"),
                }
                Ok(pass)
            },
        )
        .unwrap();
        assert_eq!(calls, 4);
        assert!(closed);
        assert_eq!(clock, 2e-6);
        assert_eq!(vessel.pending_co2_transfer_mol, 0.0);
        assert_eq!(vessel.step_start, original.step_start);
        assert!(!events
            .iter()
            .any(|event| matches!(event, Event::GasEvolved { .. })));
    }

    #[test]
    fn resumed_seed_replay_discards_gas_and_clock_and_uses_original_owner() {
        // Controlled driver sequence, not a claim about native chemistry.
        let mut vessel = state();
        vessel.deposit(SpeciesId::new("CO2(aq)"), Moles(1e-8), Phase::Aqueous);
        vessel.pending_co2_transfer_mol = 2e-6;
        vessel.step_start = Some(kerotakis_core::vessel::StepStart::capture(&vessel));
        let original = vessel.clone();
        let mut calls = 0;
        let mut solve = |candidate: &mut Vessel,
                         prior: &[Event],
                         certified_reentry: ComponentCertificate,
                         hint: FirstPoseHint| {
            calls += 1;
            let mut pass = quote(0.0);
            match calls {
                1 => {
                    assert_eq!(certified_reentry, ComponentCertificate::Uncertified);
                    assert_eq!(hint, FirstPoseHint::default());
                    candidate
                        .contents
                        .retain(|portion| portion.species.0 != "CO2(aq)");
                    candidate.deposit(SpeciesId::new("CO2(aq)"), Moles(2e-6), Phase::Aqueous);
                    pass.db_tag = "pitzer";
                    pass.clock_co2_applied_mol = 2e-6;
                    pass.events.push(Event::GasEvolved {
                        vessel: candidate.id,
                        species: SpeciesId::new("CO2"),
                        moles: Moles(1e-8),
                    });
                }
                2 => {
                    assert!(certified_reentry.is_certified());
                    assert_eq!(hint, FirstPoseHint::default());
                    assert_eq!(candidate.pending_co2_transfer_mol, 0.0);
                    assert_eq!(prior.len(), 1);
                    assert_eq!(candidate.step_start.as_ref().unwrap().gas_out.len(), 1);
                    // WATEQ was discovered only on the Pitzer-modified
                    // hypothetical state. This answer must be discarded.
                }
                3 => {
                    assert_eq!(certified_reentry, ComponentCertificate::Uncertified);
                    assert_eq!(hint.primary_route, Some("wateq4f"));
                    assert_eq!(hint.secondary, SecondaryFirstHint::Auto);
                    assert!(prior.is_empty());
                    assert_eq!(candidate.contents, original.contents);
                    assert_eq!(
                        candidate.pending_co2_transfer_mol,
                        original.pending_co2_transfer_mol
                    );
                    assert_eq!(candidate.step_start, original.step_start);
                    candidate.deposit(SpeciesId::new("CO2(aq)"), Moles(2e-6), Phase::Aqueous);
                    pass.clock_co2_applied_mol = 2e-6;
                }
                4 => {
                    assert!(certified_reentry.is_certified());
                    assert_eq!(hint, FirstPoseHint::default());
                    assert!(prior.is_empty());
                    assert_eq!(candidate.pending_co2_transfer_mol, 0.0);
                    assert_eq!(candidate.step_start.as_ref().unwrap().gas_out.len(), 1);
                }
                _ => panic!("replay exceeded the expected four controlled queries"),
            }
            Ok(pass)
        };
        let seed = PhreeqcEquilibrator::first_pose_at_temperature_with(&vessel, |candidate| {
            solve(
                candidate,
                &[],
                ComponentCertificate::Uncertified,
                FirstPoseHint::default(),
            )
        })
        .unwrap();
        let (events, _, closed, clock) =
            PhreeqcEquilibrator::resume_first_pose_with(&mut vessel, seed, solve).unwrap();
        assert_eq!(calls, 4);
        assert!(closed);
        assert_eq!(clock, 2e-6);
        assert_eq!(vessel.pending_co2_transfer_mol, 0.0);
        assert_eq!(vessel.step_start, original.step_start);
        assert!(!events
            .iter()
            .any(|event| matches!(event, Event::GasEvolved { .. })));
    }

    #[test]
    fn route_oscillation_spends_the_same_global_eight_query_budget_and_rolls_back() {
        let mut vessel = state();
        vessel.pending_co2_transfer_mol = 5e-7;
        vessel.step_start = Some(kerotakis_core::vessel::StepStart::capture(&vessel));
        let original = serde_json::to_vec(&vessel).unwrap();
        let mut calls = 0;
        let mut active_route = "pitzer";
        let failure = PhreeqcEquilibrator::solve_at_temperature_with(
            &mut vessel,
            |candidate, _, certified_reentry, hint| {
                calls += 1;
                let tag = if certified_reentry.is_certified() {
                    if active_route == "pitzer" {
                        "wateq4f"
                    } else {
                        "pitzer"
                    }
                } else {
                    hint.primary_route.unwrap_or("pitzer")
                };
                if !certified_reentry.is_certified() {
                    active_route = tag;
                }
                candidate.solution.as_mut().unwrap().ionic_strength =
                    if tag == "pitzer" { 1.2 } else { 0.1 };
                let mut pass = quote(0.0);
                pass.db_tag = tag;
                Ok(pass)
            },
        )
        .expect_err("alternating routed models cannot spend a fresh budget on each replay");
        assert!(matches!(failure, SolveError::NotConverged { .. }));
        assert_eq!(calls, MAX_CANONICAL_AQUEOUS_PASSES);
        assert_eq!(serde_json::to_vec(&vessel).unwrap(), original);
    }

    fn secondary_provenance() -> kerotakis_core::vessel::SolventActivityProvenance {
        kerotakis_core::vessel::SolventActivityProvenance {
            dataset: "pitzer.dat".into(),
            model: "Pitzer".into(),
            dataset_phrase: None,
            model_phrase: None,
            water_activity: 0.9989,
            particle_molality: 0.06,
            ionic_strength: 0.03,
        }
    }

    fn secondary_answer(
        candidate: &mut Vessel,
        disposition: SecondaryDisposition,
        heat_j: f64,
    ) -> AqueousPass {
        candidate.solution.as_mut().unwrap().solvent_activity =
            (disposition == SecondaryDisposition::Accepted).then(secondary_provenance);
        let mut pass = quote(heat_j);
        pass.secondary_disposition = disposition;
        pass
    }

    #[test]
    fn auxiliary_floor_replay_discards_provisional_gas_clock_and_heat() {
        let mut vessel = state();
        vessel.pending_co2_transfer_mol = 2e-6;
        vessel.step_start = Some(kerotakis_core::vessel::StepStart::capture(&vessel));
        let original = vessel.clone();
        let mut calls = 0;
        let (events, heat, closed, clock) = PhreeqcEquilibrator::solve_at_temperature_with(
            &mut vessel,
            |candidate, prior, reentry, hint| {
                calls += 1;
                match calls {
                    1 => {
                        assert_eq!(reentry, ComponentCertificate::Uncertified);
                        assert!(prior.is_empty());
                        assert_eq!(hint, FirstPoseHint::default());
                        candidate.deposit(
                            SpeciesId::new("CO2(aq)"),
                            Moles(2e-6 - 1e-8),
                            Phase::Aqueous,
                        );
                        let mut pass =
                            secondary_answer(candidate, SecondaryDisposition::Accepted, 99.0);
                        pass.clock_co2_applied_mol = 2e-6;
                        pass.events.push(Event::GasEvolved {
                            vessel: candidate.id,
                            species: SpeciesId::new("CO2"),
                            moles: Moles(1e-8),
                        });
                        Ok(pass)
                    }
                    2 => {
                        assert_eq!(reentry, ComponentCertificate::Represented);
                        assert_eq!(prior.len(), 1);
                        assert_eq!(candidate.pending_co2_transfer_mol, 0.0);
                        Ok(secondary_answer(
                            candidate,
                            SecondaryDisposition::BelowInputFloor,
                            99.0,
                        ))
                    }
                    3 => {
                        assert_eq!(reentry, ComponentCertificate::Uncertified);
                        assert!(prior.is_empty());
                        assert_eq!(hint.secondary, SecondaryFirstHint::Skip);
                        assert_eq!(candidate.contents, original.contents);
                        assert_eq!(candidate.pending_co2_transfer_mol, 2e-6);
                        assert_eq!(candidate.step_start, original.step_start);
                        candidate.deposit(SpeciesId::new("CO2(aq)"), Moles(2e-6), Phase::Aqueous);
                        let mut pass = secondary_answer(
                            candidate,
                            SecondaryDisposition::SkippedForReplay,
                            0.0,
                        );
                        pass.clock_co2_applied_mol = 2e-6;
                        Ok(pass)
                    }
                    4 => {
                        assert_eq!(reentry, ComponentCertificate::Represented);
                        assert!(prior.is_empty());
                        assert_eq!(hint, FirstPoseHint::default());
                        Ok(secondary_answer(
                            candidate,
                            SecondaryDisposition::BelowInputFloor,
                            0.0,
                        ))
                    }
                    _ => panic!("secondary replay exceeded its four controlled poses"),
                }
            },
        )
        .expect("ordinary floor agrees after fresh owner replay");
        assert_eq!(calls, 4);
        assert!(closed);
        assert_eq!(heat, 0.0);
        assert_eq!(clock, 2e-6);
        assert_eq!(vessel.pending_co2_transfer_mol, 0.0);
        assert_eq!(vessel.step_start, original.step_start);
        assert!(!events
            .iter()
            .any(|event| matches!(event, Event::GasEvolved { .. })));
    }

    #[test]
    fn reverse_floor_replay_probes_only_first_pose_then_requires_ordinary_admission() {
        let mut vessel = state();
        let mut calls = 0;
        let (_, _, closed, _) = PhreeqcEquilibrator::solve_at_temperature_with(
            &mut vessel,
            |candidate, _, reentry, hint| {
                calls += 1;
                let disposition = match calls {
                    1 => {
                        assert_eq!(reentry, ComponentCertificate::Uncertified);
                        assert_eq!(hint, FirstPoseHint::default());
                        SecondaryDisposition::BelowInputFloor
                    }
                    2 => {
                        assert_eq!(reentry, ComponentCertificate::Represented);
                        assert_eq!(hint, FirstPoseHint::default());
                        SecondaryDisposition::Accepted
                    }
                    3 => {
                        assert_eq!(reentry, ComponentCertificate::Uncertified);
                        assert_eq!(hint.secondary, SecondaryFirstHint::Probe);
                        SecondaryDisposition::Accepted
                    }
                    4 => {
                        assert_eq!(reentry, ComponentCertificate::Represented);
                        assert_eq!(hint, FirstPoseHint::default());
                        SecondaryDisposition::Accepted
                    }
                    _ => panic!("secondary replay exceeded its four controlled poses"),
                };
                Ok(secondary_answer(candidate, disposition, 0.0))
            },
        )
        .expect("ordinary selector agrees with provisional probe");
        assert!(closed);
        assert_eq!(calls, 4);
        assert!(vessel.solution.as_ref().unwrap().solvent_activity.is_some());
    }

    #[test]
    fn auxiliary_other_declines_never_authorize_replay_and_floor_spends_global_cap() {
        for reason in [
            SecondaryDecline::AlreadyPrimary,
            SecondaryDecline::Unsupported,
            SecondaryDecline::MissingReadback,
            SecondaryDecline::EngineFailure,
            SecondaryDecline::InvalidResult,
            SecondaryDecline::BelowOutputParticleFloor,
        ] {
            assert_eq!(
                secondary_floor_replay(
                    SecondaryDisposition::Accepted,
                    SecondaryDisposition::Declined(reason)
                ),
                None
            );
            assert_eq!(
                secondary_floor_replay(
                    SecondaryDisposition::Declined(reason),
                    SecondaryDisposition::Accepted
                ),
                None
            );
        }
        let mut vessel = state();
        vessel.pending_co2_transfer_mol = 5e-7;
        vessel.step_start = Some(kerotakis_core::vessel::StepStart::capture(&vessel));
        let original = serde_json::to_vec(&vessel).unwrap();
        let mut calls: usize = 0;
        let error =
            PhreeqcEquilibrator::solve_at_temperature_with(&mut vessel, |candidate, _, _, hint| {
                calls += 1;
                let disposition = match calls {
                    1 => SecondaryDisposition::Accepted,
                    2 => SecondaryDisposition::BelowInputFloor,
                    3 => {
                        assert_eq!(hint.secondary, SecondaryFirstHint::Skip);
                        SecondaryDisposition::SkippedForReplay
                    }
                    _ => SecondaryDisposition::BelowInputFloor,
                };
                if calls >= 3 {
                    candidate.free_proton = if calls.is_multiple_of(2) { 1e-9 } else { 0.0 };
                }
                Ok(secondary_answer(candidate, disposition, 0.0))
            })
            .expect_err("discarded selector poses count against the same eight-pose cap");
        assert!(matches!(error, SolveError::NotConverged { .. }));
        assert_eq!(calls, MAX_CANONICAL_AQUEOUS_PASSES);
        assert_eq!(serde_json::to_vec(&vessel).unwrap(), original);
    }

    #[test]
    fn auxiliary_engine_or_capability_change_cannot_replay_or_commit() {
        for initial in [
            SecondaryDisposition::Accepted,
            SecondaryDisposition::BelowInputFloor,
        ] {
            for reason in [
                SecondaryDecline::EngineFailure,
                SecondaryDecline::Unsupported,
                SecondaryDecline::InvalidResult,
            ] {
                let mut vessel = state();
                let original = serde_json::to_vec(&vessel).unwrap();
                let mut calls = 0;
                let error = PhreeqcEquilibrator::solve_at_temperature_with(
                    &mut vessel,
                    |candidate, _, _, hint| {
                        assert_eq!(hint, FirstPoseHint::default());
                        calls += 1;
                        let disposition = if calls == 1 {
                            initial
                        } else {
                            SecondaryDisposition::Declined(reason)
                        };
                        Ok(secondary_answer(candidate, disposition, 0.0))
                    },
                )
                .expect_err("optional engine or model authority cannot change during reentry");
                assert!(matches!(error, SolveError::NotConverged { .. }));
                assert_eq!(calls, 2);
                assert_eq!(serde_json::to_vec(&vessel).unwrap(), original);
            }
        }
    }

    #[test]
    fn secondary_selected_output_mu_must_be_finite_and_nonnegative() {
        let read = |value: &str| {
            secondary_ionic_strength(&[
                vec!["pH".into(), "mu".into()],
                vec!["7".into(), value.into()],
            ])
        };
        assert_eq!(read("0"), Ok(0.0));
        assert_eq!(read("0.031"), Ok(0.031));
        for value in ["-0.01", "NaN", "inf", "-inf"] {
            assert_eq!(read(value), Err(SecondaryDecline::InvalidResult));
            assert_eq!(
                secondary_floor_replay(
                    SecondaryDisposition::Accepted,
                    SecondaryDisposition::Declined(SecondaryDecline::InvalidResult),
                ),
                None,
            );
        }
        assert_eq!(read("not-a-number"), Err(SecondaryDecline::InvalidResult));
        assert_eq!(
            secondary_ionic_strength(&[vec!["pH".into()], vec!["7".into()]]),
            Err(SecondaryDecline::MissingReadback)
        );
    }
}

impl PhreeqcEquilibrator {
    /// Ask the settled state again, posed from the INVENTORY rather than
    /// from the operation that produced it.
    ///
    /// # The defect this exists for
    ///
    /// `solution.solvent_kg` was path-dependent. The same beaker — the
    /// corpus row `aq-023`, 100 mL of water, 0.01 mol of calcium chloride
    /// and 5 g of a carbonate laundry powder — read `0.0997010580` kg when
    /// the salt went in first and `0.0996909590` kg when the powder did.
    /// One part in 1e4, about 10 mg of water in 100 g. Every molality is
    /// per kg of that mass, so the residue propagates into
    /// `ionic_strength` and from there into every activity coefficient,
    /// every saturation index and the pH.
    ///
    /// Both numbers come from the same line of the same function, so this
    /// was never two sources disagreeing. It is one source asked two
    /// different questions:
    ///
    ///   * `partition` takes the solvent mass from the vessel's **water
    ///     portion**, and that portion is whatever the previous step's
    ///     [`crate::inventory::complete_basis`] wrote — the water implied
    ///     by conserved H and O once every solute is booked. That figure
    ///     is order-free: the two orderings agree on it to one part in
    ///     4e8, because conservation does not care what order matter
    ///     arrived in.
    ///   * `mass_H2O` is PHREEQC's own solvent mass, and it comes back
    ///     from the solve tracking the water the INPUT declared. The input
    ///     to the last operation's solve is the *intermediate* vessel plus
    ///     one reagent — and the intermediate vessels of the two orderings
    ///     are different solutions, holding different amounts of their
    ///     hydrogen and oxygen inside species rather than inside water.
    ///     So the last solve is handed the same final contents in two
    ///     different representations and answers each faithfully.
    ///
    /// PHREEQC's `mass_H2O` is not representation-invariant — `aqueous.rs`
    /// already says exactly that of the surface-complexation readback —
    /// and that is a fact about the solver, not a bug in it.
    ///
    /// # What "canonically" means here, decided explicitly
    ///
    /// A canonical pose is one built from **what the vessel holds**, never
    /// from what just happened to it:
    ///
    ///   1. the **settled contents**, after the readback and after
    ///      `complete_basis` has put the H/O residue back into water, acid
    ///      and base equivalents — the one representation of this state
    ///      that conservation pins;
    ///   2. through the ordinary [`partition`], which already sorts the
    ///      portions and accumulates **aggregated element totals**, so
    ///      feed order cannot reach the accumulation;
    ///   3. and then through the ordinary [`Self::setup_problem`] and
    ///      `build_input`, so the routing, the candidate phases and the
    ///      formatting are the ones every other solve gets.
    ///
    /// Two orderings of the same reagents therefore hand this pass the
    /// same contents, which produce the same `Problem`, which prints
    /// almost the same input string — and what survives into the text is
    /// worth being exact about, because it is what the invariance is
    /// actually worth. Measured on the `aq-023` chemistry, salt-first
    /// against powder-first:
    ///
    /// ```text
    ///                       solvent_kg     ionic strength   pH
    ///   before, salt   0.0997010580   0.2759516202   10.9521249278
    ///   before, powder 0.0996909590   0.2759782269   10.9521322284
    ///   after,  salt   0.0996939730   0.2759702852   10.9521300644
    ///   after,  powder 0.0996939730   0.2759702847   10.9521300643
    ///
    ///   apart, relative  solvent_kg   ionic strength   pH (absolute)
    ///   before             1.0129e-4       9.6418e-5       7.3007e-6
    ///   after             1.0023e-13       1.8361e-9       1.1000e-10
    /// ```
    ///
    /// The measurements above describe the historical nine-decimal input.
    /// Current inputs preserve solvent mass with sixteen scientific digits
    /// so the native total is the written molality times the same water
    /// mass used to calculate it. Cache keys include that full input. The
    /// canonical pose still derives from settled contents, and no displayed
    /// number is used as an inventory correction.
    ///
    /// # What it deliberately does NOT do
    ///
    /// It does not rebuild the vessel. Contents, phases, surfaces and
    /// temperature stay exactly as the operation left them; this pass
    /// refreshes reported solution fields and the measured free-ion and CO2
    /// pressure coordinates from the same numerical answer. It corrects
    /// the `SolutionCharacterized` event that carries pH and ionic strength.
    ///
    /// Overwriting `solvent_kg` with the vessel's own water inventory was
    /// the obvious alternative and is not available: molalities are per kg
    /// of the solver's `mass_H2O`, so substituting a different mass would
    /// leave `n = m × kg` false by exactly the discrepancy it repaired.
    /// The invariance has to come from what the solver is ASKED.
    ///
    /// # Cost
    ///
    /// One `dispatch_solve` per **equilibration**, not per pass of the
    /// thermal fixed point (which re-solves a clone up to eight times) and
    /// not per solver in the stack. It is placed here, after `settled`,
    /// for that reason. Some of those calls never reach the engine at all:
    /// a re-pose whose input the content-addressed cache already holds is
    /// answered from it.
    ///
    /// Measured on the two `aq-023` orderings, which are three
    /// equilibrations each (one of them the solvent-only opening water,
    /// which this pass declines): **8 engine calls became 9, and 6 became
    /// 8**. At most one more call per equilibration, which is what the
    /// ruling accepted, and sometimes none. `tests/order_invariance.rs`
    /// pins both the invariance and the ceiling.
    ///
    /// # When it declines
    ///
    /// A vessel with no solution, a solvent-only vessel (there is no
    /// engine call behind that answer to repeat) and any re-pose the
    /// engine refuses are all left exactly as they were. A refusal here
    /// must not turn a step that worked into a step that failed — but it
    /// does silently restore the path dependence, which is why the test
    /// asserts the invariance on the live path rather than trusting this
    /// branch never to be taken.
    fn recharacterise_canonically(
        &mut self,
        vessel: &mut Vessel,
        events: &mut [Event],
    ) -> Result<(), SolveError> {
        let Some(scope) = vessel.solution.as_ref().map(|info| info.scope) else {
            return Ok(());
        };
        if scope == SolutionScope::SolventOnly {
            return Ok(());
        }
        let Ok(Some(setup)) = self.setup_problem(vessel, None) else {
            return Ok(());
        };
        if setup.problem.solvent_only {
            return Ok(());
        }
        // The route has to be the one the record already names.
        //
        // `setup_problem` picks the dataset from the problem it is handed,
        // and this one is built from the settled contents rather than from
        // the contents plus the reagent. The two agree in every case this
        // bench has, because a readback moves the totals by parts in 1e6
        // and the thresholds it crosses — an element the default dataset
        // lacks, a molality past 1 mol/kgw — are nowhere near that. If a
        // vessel ever does sit on one of those edges, reporting a pH from
        // one dataset beside a provenance naming another is worse than
        // reporting a path-dependent one, so this declines instead.
        let routed_file = format!("{}.dat", setup.db_tag);
        let same_route = vessel
            .solution
            .as_ref()
            .and_then(|info| info.provenance.as_ref())
            .is_some_and(|provenance| provenance.dataset.contains(&routed_file));
        if !same_route {
            return Ok(());
        }
        let Ok((cached, _)) = self.dispatch_solve(
            vessel,
            &setup.problem,
            setup.db_tag,
            &setup.input,
            setup.key,
        ) else {
            return Ok(());
        };
        let value = |column: &str| -> Option<f64> {
            let idx = cached.rows.first()?.iter().position(|h| h == column)?;
            cached.rows.last()?.get(idx)?.parse().ok()
        };
        let (Some(solvent_kg), Some(ph), Some(mu)) = (value("mass_H2O"), value("pH"), value("mu"))
        else {
            return Ok(());
        };
        refresh_measured_coordinates(vessel, &cached.rows, &cached.speciation, solvent_kg)?;
        let Some(info) = vessel.solution.as_mut() else {
            return Ok(());
        };
        info.solvent_kg = Some(solvent_kg);
        info.particle_molality = value("particle_molality");
        info.ph = ph;
        info.ionic_strength = mu;
        info.species = cached.speciation.clone();
        // The event carries the same two numbers and must not disagree
        // with the field. Patched in place, exactly as the headspace
        // pressure below is patched once the final temperature is known.
        if let Some(Event::SolutionCharacterized {
            ph: said_ph,
            ionic_strength: said_mu,
            ..
        }) = events
            .iter_mut()
            .rev()
            .find(|event| matches!(event, Event::SolutionCharacterized { .. }))
        {
            *said_ph = ph;
            *said_mu = mu;
        }
        Ok(())
    }

    /// One pass at the vessel's current temperature. Returns the reaction
    /// heat rather than applying it, so the caller can iterate temperature
    /// and composition to a common answer instead of reporting one solved
    /// before the other.
    fn solve_once(
        &mut self,
        vessel: &mut Vessel,
        prior_physical_events: &[Event],
        owner: ComponentCertificate,
        hint: FirstPoseHint,
        auxiliary_policy: AuxiliaryPolicy,
    ) -> Result<AqueousPass, SolveError> {
        self.last_heat_unpriced = None;
        if hint.primary_route.is_some() {
            // A provisional answer on another database must not reuse an
            // electron-search guess from the abandoned model.
            self.warm_pe = None;
        }
        let Some(SolveSetup {
            problem,
            db_tag,
            routing,
            freed_phases,
            input,
            key,
        }) = self.setup_problem(vessel, hint.primary_route)?
        else {
            return Ok(AqueousPass {
                events: Vec::new(),
                heat_quote_j: 0.0,
                heat_mode: HeatMode::Inactive,
                component_certificate: ComponentCertificate::Uncertified,
                db_tag: "",
                reservoirs: Vec::new(),
                clock_co2_applied_mol: 0.0,
                secondary_disposition: SecondaryDisposition::Declined(
                    SecondaryDecline::Unsupported,
                ),
                pending_secondary: None,
            });
        };
        // Only the ordinary native owner may opt this reviewed open-reservoir
        // subset into canonical closure. MIX cannot inherit that authority.
        let reservoir_requested = reservoir_opaque_component_owner(vessel, &problem, owner);

        // The state this step starts from.
        //
        // Preferring the bench's `step_start` over this call's own opening
        // state is what makes the balance a property of the STEP rather
        // than of one solver. Solvers above the tail change what the vessel
        // holds: the curated `NaHCO3 + CH3COOH` row consumes the
        // bicarbonate and gives off the carbon dioxide itself, so a balance
        // beginning at the tail's call-start would price a step in which
        // the bicarbonate had never been there and the carbon simply
        // stopped existing.
        //
        // The fallback is the call-start, unchanged, for hosts that never
        // set the snapshot — the wasm bench and the cache-only path.
        let (before_contents, before_oh, mut gas_out) = match &vessel.step_start {
            Some(start) => (
                start.contents.clone(),
                start.free_hydroxide,
                start
                    .gas_out
                    .iter()
                    .map(|(species, moles)| (species.0.clone(), moles.0))
                    .collect::<Vec<_>>(),
            ),
            None => (vessel.contents.clone(), vessel.free_hydroxide, Vec::new()),
        };

        let (cached, coupling_failed) =
            self.dispatch_solve(vessel, &problem, db_tag, &input, key)?;

        let value = |column: &str| -> Option<f64> {
            let idx = cached.rows.first()?.iter().position(|h| h == column)?;
            cached.rows.last()?.get(idx)?.parse().ok()
        };

        let (
            solvent_kgw_out,
            mut new_surfaces,
            new_exchanges,
            new_solid_solutions,
            mut new_ions,
            unnameable,
            protonation,
        ) = self.readback_raw_values(&problem, db_tag, &cached.rows, &value)?;

        if unnameable.iter().any(|(_, amount)| *amount > 1e-10) {
            return Err(SolveError::NotConverged {
                reason: Some(Box::new(AqueousReason::ReadbackUnrepresented.phrase())),
                solver: "phreeqc-inventory".into(),
                detail: format!("cannot commit unrepresented elemental states: {unnameable:?}"),
            });
        }

        let (new_phases, new_gases, ph, mu, gas_components_certified) =
            Self::apply_balance_corrections(
                vessel,
                &problem,
                &cached.rows,
                &mut new_ions,
                &mut new_surfaces,
                &new_exchanges,
                &new_solid_solutions,
                &value,
                owner,
                reservoir_requested,
            )?;

        if gas_components_certified {
            certify_projected_protonation(&new_ions, &protonation)?;
        } else if problem.clock_co2_applied_mol != 0.0 {
            return Err(canonical_closure_error(
                AqueousReason::ClockProjectionMissing,
                "clock CO2 parcel lacks a certified native component projection",
            ));
        }

        let (mut events, contents) = Self::rebuild_contents_and_events(
            vessel,
            &problem,
            &freed_phases,
            solvent_kgw_out,
            &new_ions,
            &new_phases,
            &new_gases,
            &new_solid_solutions,
            &protonation,
        );

        // Neutralisation: the heat of the reaction the engine cannot see.
        //
        // PHREEQC is handed element totals, so it cannot tell an acid just
        // added from one that was always there — and `H⁺ + OH⁻ → H₂O` never
        // appears as a reaction it reports. Its heat was simply missing:
        // 0.1 mol of hydrochloric acid neutralised by caustic soda left the
        // beaker 13.7 K below where a real one lands, silently, in the most
        // common thermochemistry experiment in school.
        //
        // The extent is recoverable from the solutes' net charge. A beaker
        // holding chloride and nothing else is holding exactly that much
        // free acid; one holding sodium is holding that much free base.
        // What cancels when the opposite arrives is the overlap of the two,
        // which is `(|A_before| + |ΔA| − |A_after|) / 2` — a quantity that
        // is zero for adding more of what is already there, and correct
        // when the sign flips straight past neutral (0.1 mol of acid met by
        // 0.2 mol of base still neutralises 0.1).
        let a_before = vessel.solute_charge;
        let a_after: f64 = contents
            .iter()
            .filter(|p| p.phase == Phase::Aqueous && !species::is_acid_base_basis(&p.species.0))
            .filter_map(|p| {
                let d = species::lookup(&p.species)?;
                let f = kerotakis_core::stoich::parse_formula(d.formula).ok()?;
                Some(f.charge * p.moles.0)
            })
            .sum();
        let mut neutralised =
            0.5 * (a_before.abs() + (a_after - a_before).abs() - a_after.abs()).max(0.0);
        vessel.solute_charge = a_after;

        // But charge cancelling is not the same event as water forming, and
        // the enthalpy above belongs to the second one.
        //
        // A carbonate takes a proton too: `HCO₃⁻ + H⁺ → H₂O + CO₂↑`. The
        // charge arithmetic cannot tell that from `H⁺ + OH⁻ → H₂O`, because
        // in both a negative solute and an acid disappear together — so the
        // strong-acid-strong-base enthalpy was being claimed for it, at the
        // wrong magnitude and, once the gas leaving is counted, the wrong
        // SIGN. Vinegar poured onto dissolved baking soda came out WARMER,
        // and vinegar and baking soda is one of the few reactions a child
        // can put a hand on the beaker and feel: it gets cold.
        //
        // Every mole of CO₂ that left in this same step is a mole of acid
        // that went to a carbonate rather than to hydroxide. Discount it.
        // What remains is water-forming neutralisation and is charged at
        // the neutralisation enthalpy. What was discounted has an enthalpy
        // this lab does not hold, and goes uncharged rather than borrowing
        // the nearest number that happens to be in the file.
        //
        // Nothing is announced here, deliberately. The first version raised
        // a `NotYetModeled` naming the missing heat, and it fired on four
        // corpus rows that have no acid in them at all — opening a
        // carbonated bottle, sweeping CO₂ out of water. Carbonic acid
        // leaving as gas cancels charge exactly as a carbonate taking a
        // proton does, so this discount cannot tell a reaction from a
        // degassing, and a note that says "acid was taken by a carbonate"
        // would have been false on every one of them. Declining to charge
        // heat we do not have is right in both cases; describing why is
        // only right in one, and this site cannot tell which it is in.
        let carbonate_route: f64 = events
            .iter()
            .filter_map(|e| match e {
                Event::GasEvolved { species, moles, .. }
                | Event::GasContained { species, moles, .. }
                    if species.0 == "CO2" =>
                {
                    Some(moles.0)
                }
                _ => None,
            })
            .sum();
        let to_carbonate = neutralised.min(carbonate_route);
        neutralised -= to_carbonate;
        // A change in weak-acid speciation can cancel analytical charge
        // without consuming free hydroxide. In particular, a preceding
        // saponification has already spent its base. Bound the narration by
        // the hydroxide still measured here plus that supplied by actual
        // undissolved hydroxide reagents, not analytical H/O coordinates.
        let supplied_base: f64 = vessel
            .contents
            .iter()
            .map(|p| {
                enthalpy::DISSOCIATION
                    .iter()
                    .find(|(key, _)| *key == p.species.0)
                    .map(|(_, products)| {
                        products
                            .iter()
                            .filter(|(key, _)| *key == "OH-")
                            .map(|(_, coefficient)| coefficient * p.moles.0)
                            .sum::<f64>()
                    })
                    .unwrap_or(0.0)
            })
            .sum();
        let remaining_base = value("m_OH-")
            .map(|m| m * solvent_kgw_out)
            .unwrap_or_else(|| free_hydroxide_moles(Some(&cached.speciation), solvent_kgw_out));
        neutralised = neutralised
            .min((vessel.free_hydroxide.max(0.0) + supplied_base - remaining_base).max(0.0));
        // The extent has been computed here for a while to get the heat
        // right, and then discarded. It is a reaction that happened, so it
        // belongs in the ledger — and it is what the net ionic equation
        // `H⁺ + OH⁻ → H₂O` is derived from (GUI-092). The observability
        // floor is applied at the moment of telling, as everywhere else.
        if neutralised > 0.0 {
            events.push(Event::Neutralised {
                vessel: vessel.id,
                moles: Moles(neutralised),
            });
        }
        let before_inventory = vessel.clone();
        vessel.contents = contents;
        vessel.surfaces = new_surfaces;
        vessel.exchanges = new_exchanges;
        vessel.solid_solutions = new_solid_solutions;
        let reservoirs: Vec<&str> = problem
            .external_gases
            .iter()
            .filter(|g| matches!(g.kind, ExternalGasKind::Reservoir))
            .map(|g| g.species.as_str())
            .collect();
        let clock = crate::inventory::ClockCo2Boundary {
            vessel: vessel.id,
            applied_mol: problem.clock_co2_applied_mol,
        };
        crate::inventory::complete_basis(&before_inventory, vessel, &events, &reservoirs, clock)?;
        if gas_components_certified {
            if reservoir_requested {
                crate::inventory::certify_reservoir_opaque_inventory(
                    &before_inventory,
                    vessel,
                    &events,
                    &reservoirs,
                )?;
            } else {
                crate::inventory::certify_gas_aware_inventory(
                    &before_inventory,
                    vessel,
                    &events,
                    &reservoirs,
                    clock,
                )?;
            }
        }
        vessel.refresh_pressure();
        if vessel.owns_headspace_gas() && !problem.gases.is_empty() {
            events.push(Event::HeadspaceEquilibrated {
                vessel: vessel.id,
                pressure: vessel.pressure,
                total_moles: vessel.gas_moles(),
            });
        }

        // Reaction heat: curated dissolution enthalpies feed the energy
        // balance (PLAN.md). Dissolution of an endothermic salt cools the
        // vessel; precipitation releases the corresponding heat. v1 applies
        // the temperature change once rather than iterating solver ↔ T; the
        // shifts at teaching concentrations are small against the ~25–100 °C
        // range of the database.
        // Reaction heat is one state-function balance over everything that
        // moved — see `enthalpy`. It is computed at the BOTTOM of this
        // function, where the full event list exists: gas that has left the
        // liquid is priced as gas, and the gas events are not assembled
        // until the readback further down.
        //
        // Two separate charges used to live right here: the neutralisation
        // enthalpy, against an extent derived from how much the solutes'
        // net charge cancelled, and a dissolution enthalpy per `Dissolved`
        // and `Precipitated` event. They are now terms of one sum rather
        // than paths beside it, which is what stops them disagreeing —
        // `H+ + OH- -> H2O` is a hydroxide going to water in the inventory
        // and returns the same kJ/mol the engine used to be asked for.
        //
        // The charge-cancellation extent is still computed above, because
        // `Event::Neutralised` is the net ionic equation GUI-092 renders.
        // It no longer carries any heat.
        let mut q_joules = 0.0; // heat released into the vessel
        let mut heat_mode = HeatMode::Inactive;

        let idx = derived::index_for(db_tag);
        // pe is reported only when the beaker contains a redox couple the
        // user actually put there. Left to itself PHREEQC reports the value
        // it was handed — 4.0 by default — and printing that beside a
        // computed pH would dress an assumption as a measurement.
        //
        // Hydrogen and oxygen are excluded even though the database gives
        // them oxidation states, because they are in every aqueous solution
        // and their presence says nothing about whether anything is being
        // oxidised.
        //
        // KNOWN LIMITATION, stated rather than hidden: this is necessary
        // but not sufficient. A beaker of permanganate with nothing to
        // reduce it contains a redox-active element and still does not
        // *determine* an electron activity — PHREEQC will report its
        // default and we will show it. The engine annotates its report with
        // "Adjusted to redox equilibrium", which looked like the right
        // signal until it turned out to fire on the water couple in plain
        // brine as well. Until that is understood, a test that can be
        // explained is better than one that cannot: `redox_adjusted` is
        // parsed and cached, and is where the eventual fix will hook.
        let redox_constrained = problem.elements.iter().any(|el| {
            // Element totals are keyed by valence where one is known —
            // "Mn(7)" rather than "Mn" — while the redox set is canonical.
            let canonical = el.split('(').next().unwrap_or(el);
            canonical != "H" && canonical != "O" && idx.redox_elements.contains(canonical)
        });
        let _ = cached.redox_adjusted;

        // One number the dataset that answered the chemistry could not
        // supply. See `solvent_activity_second_opinion` for the band it is
        // asked in and what it costs.
        let (secondary, pending_secondary) = if auxiliary_policy == AuxiliaryPolicy::DeferReservoir
            && reservoir_requested
            && hint == FirstPoseHint::default()
        {
            match Self::prepare_secondary(vessel, &problem, db_tag, &value, hint.secondary) {
                PreparedSecondary::Resolved(result) => (*result, None),
                PreparedSecondary::Question(question) => (
                    SecondaryResult {
                        provenance: None,
                        disposition: SecondaryDisposition::Pending,
                    },
                    Some(question),
                ),
            }
        } else {
            (
                self.solvent_activity_second_opinion(
                    vessel,
                    &problem,
                    db_tag,
                    &value,
                    hint.secondary,
                ),
                None,
            )
        };
        let solvent_activity = secondary.provenance;

        // An imported legacy raw answer can be reused for chemistry, but
        // its default `pe_undetermined=false` is not an electron witness.
        // The exact current input can still prove an open oxygen poise.
        // Coupled imports were recomputed by dispatch_solve above.
        let pe_authority = current_pe_authority(
            cached.pe_authority,
            redox_coupling(&problem, db_tag).is_some(),
            coupling_failed.is_some(),
            vessel,
            db_tag,
            &input,
        );

        Self::finalize_solution_info(
            vessel,
            &problem,
            db_tag,
            routing,
            solvent_activity,
            cached.speciation.clone(),
            &cached.saturation,
            coupling_failed,
            cached.pe_determined,
            pe_authority,
            redox_constrained,
            &value,
            &unnameable,
            ph,
            mu,
            &mut events,
        );

        // Gas that LEFT the liquid is no longer in `contents` and is
        // priced as gas on the after side. Gas still held in a headspace is
        // already a `Phase::Gas` portion, so only outward transfers are
        // collected here or it would be counted twice.
        // What the engine actually measured, carried forward for the next
        // step's balance. `solution` is wiped at the top of every step, so
        // without this the previous state's hydroxide is unrecoverable and
        // the only persistent record would be the solutes' net charge —
        // which is free base ONLY in a vessel of strong electrolytes. A
        // bicarbonate carries its charge excess as carbonate alkalinity and
        // a beaker handed a bare cation carries it as nothing; both read as
        // hydroxide, and both then invent a neutralisation at 55.81 kJ a
        // mole. One of them reached MINUS 27 K.
        refresh_measured_coordinates(vessel, &cached.rows, &cached.speciation, solvent_kgw_out)?;
        let measured_oh = vessel.free_hydroxide;
        // Written beside it and unused by this balance — H+ is a master
        // species and carries no enthalpy — but a gate above the tail has
        // no other way to read it once `solution` has been cleared. See
        // the field docs for why it is not `unspent_acidity`.
        // The final selected row also persists H+, OH- and the tiny CO2
        // molality that the rounded species report may not display. These
        // coordinates belong to the current native answer, including hits
        // in the content-addressed cache.
        if matches!(vessel.thermal_mode, ThermalMode::Adiabatic)
            && kerotakis_core::aqueous_continuation::whole_step_heat_available(vessel)
        {
            // Gas this solver gave off, added to whatever the solvers
            // above it already booked into the snapshot.
            for e in &events {
                let (species, signed) = match e {
                    Event::GasEvolved { species, moles, .. } => (species, moles.0),
                    Event::GasAbsorbed { species, moles, .. } => (species, -moles.0),
                    _ => continue,
                };
                match gas_out.iter_mut().find(|(k, _)| *k == species.0) {
                    Some((_, n)) => *n += signed,
                    None => gas_out.push((species.0.clone(), signed)),
                }
            }
            // The clock parcel was already posed in SOLUTION totals, so it
            // is not among native gas events. The whole-step heat boundary
            // nevertheless owes this inlet/outlet exactly once.
            if problem.clock_co2_applied_mol != 0.0 {
                match gas_out.iter_mut().find(|(key, _)| key == "CO2") {
                    Some((_, amount)) => *amount -= problem.clock_co2_applied_mol,
                    None => gas_out.push(("CO2".into(), -problem.clock_co2_applied_mol)),
                }
            }
            match enthalpy::heat_released_j(
                &before_contents,
                before_oh,
                &vessel.contents,
                measured_oh,
                &gas_out,
                db_tag,
            ) {
                Ok(j) => {
                    q_joules = j;
                    heat_mode = HeatMode::Priced;
                }
                Err(unpriced) => {
                    // The balance could not price something that MOVED.
                    //
                    // This used to fall back to charging the registry's
                    // dissolution enthalpy per observed `Dissolved` /
                    // `Precipitated` event. That ledger is partial by
                    // construction — it exists only for steps the
                    // state-function balance could not close — and a
                    // partial ledger is not a smaller error, it can be the
                    // wrong SIGN: slaking quicklime on wateq4f charged the
                    // portlandite precipitation and not the lime it came
                    // from, and the beaker cooled for one of the most
                    // exothermic reactions on the bench.
                    //
                    // So the heat is refused for the whole step, and the
                    // thermal fixed point keeps the supplied temperature
                    // (`aqueous_temperature_target`) rather than letting a
                    // heat-capacity relabelling reconstruct a heat no one
                    // owns. The refusal is named once, after settling.
                    heat_mode = HeatMode::Unpriced;
                    q_joules = 0.0;
                    let _ = prior_physical_events;
                    self.last_heat_unpriced = Some(unpriced);
                }
            }
        }

        let component_certificate = if !gas_components_certified {
            ComponentCertificate::Uncertified
        } else if reservoir_requested {
            if !reservoir_opaque_component_owner(vessel, &problem, owner) {
                return Err(canonical_closure_error(
                    AqueousReason::ReservoirOwnerChanged,
                    "reservoir opaque aqueous owner changed after native readback",
                ));
            }
            if heat_mode == HeatMode::Priced && complete_reservoir_diagnostics(vessel) {
                ComponentCertificate::ReservoirWithOpaque
            } else {
                ComponentCertificate::Uncertified
            }
        } else if vessel.unresolved_materials.is_empty() {
            ComponentCertificate::Represented
        } else {
            ComponentCertificate::ClockWithOpaque
        };
        let mut pass = AqueousPass {
            events,
            heat_quote_j: q_joules,
            heat_mode,
            component_certificate,
            db_tag,
            reservoirs: reservoirs.into_iter().map(str::to_string).collect(),
            clock_co2_applied_mol: problem.clock_co2_applied_mol,
            secondary_disposition: secondary.disposition,
            pending_secondary,
        };
        if pass.pending_secondary.is_some() {
            let witness = AuxiliaryPoseWitness::capture(vessel, &pass);
            pass.pending_secondary
                .as_mut()
                .expect("present question")
                .witness = Some(witness);
            if component_certificate != ComponentCertificate::ReservoirWithOpaque {
                // An unpriced/incomplete first answer cannot retain private
                // deferred authority; preserve the ordinary eager schedule.
                pass = self.fulfil_aqueous_pass(vessel, pass)?;
            }
        }
        Ok(pass)
    }

    fn setup_problem(
        &self,
        vessel: &Vessel,
        forced_route: Option<&'static str>,
    ) -> Result<Option<SolveSetup>, SolveError> {
        let partitioned = partition(vessel);
        validate_clock_boundary(vessel, partitioned.as_ref())?;
        let Some(mut problem) = partitioned else {
            return Ok(None);
        };

        // Route by validity domain: minteq.v4 when its extended chemistry
        // is needed — organics (it alone has acetate), the two couples
        // `databases::minteq_v4()` borrows into it (lactate and
        // hypochlorite), or phosphate (wateq4f
        // lacks the free H3PO4 species, so the first proton would come out
        // artificially strong); pitzer for concentrated major-ion brines
        // (the ion-interaction model is built for them: halite saturates at
        // the textbook 6.13 mol/kgw where wateq4f gives 6.50); wateq4f
        // otherwise.
        // Silver ammines are a separate ligand capability: Ag and N(-3)
        // exist in wateq4f, but their complexes are present only in minteq.
        // "Extended": any element wateq4f does not know (derived — e.g.
        // Acetate), plus phosphate as a documented exception (wateq4f has
        // the P element but no free H3PO4 species).
        let needs_extended = problem
            .elements
            .iter()
            .any(|e| !derived::index_for("wateq4f").has_element(e) || e == "P");
        let potential_molality = potential_molality(&problem, vessel.temperature.0);
        let pitzer_capable = problem
            .elements
            .iter()
            .all(|el| derived::index_for("pitzer").has_element(el));
        let (mut db_tag, mut routing) = if needs_silver_ammine(&problem.elements) {
            (
                "minteq.v4",
                Phrase::bare(
                    "routing.silver-ammine",
                    "chosen because silver and reduced nitrogen require the silver-ammonia complexes present in minteq.v4.dat (NIST46.4 entries 204901 and 204902); elemental coverage in the default dataset does not include these ligand species",
                ),
            )
        } else if needs_extended {
            (
                "minteq.v4",
                Phrase::bare(
                    "routing.extended-chemistry",
                    "chosen because the problem needs chemistry the default dataset lacks (organic ligands, the borrowed hypochlorite couple, or free phosphoric acid)",
                ),
            )
        } else if potential_molality > 1.0
            && pitzer_capable
            && problem.surfaces.is_empty()
            && problem.exchanges.is_empty()
            && problem.solid_solutions.is_empty()
        {
            (
                "pitzer",
                Phrase::new(
                    "routing.concentrated-ion-interaction",
                    "chosen because the solution is concentrated (~{molality} mol/kgw), where the ion-interaction model is the valid one",
                    // A measured quantity, not a word: German writes
                    // ~16,0 mol/kgw and the separator is the reader's.
                    vec![(
                        "molality".to_string(),
                        Slot::number(format!("{potential_molality:.1}")),
                    )],
                ),
            )
        } else {
            (
                "wateq4f",
                Phrase::bare(
                    "routing.default-inorganic",
                    "the default inorganic aqueous dataset",
                ),
            )
        };
        if let Some(requested) = forced_route {
            let transition_is_reviewed = matches!(requested, "pitzer" | "wateq4f")
                && (db_tag == requested
                    || matches!(
                        (db_tag, requested),
                        ("pitzer", "wateq4f") | ("wateq4f", "pitzer")
                    ));
            let target = derived::index_for(requested);
            let original_is_representable = problem
                .elements
                .iter()
                .all(|element| target.has_element(element))
                && problem
                    .phases
                    .iter()
                    .filter(|(_, moles, _)| *moles > 0.0)
                    .all(|(phase, _, _)| posed_phase(phase, requested).is_some())
                && problem
                    .gases
                    .iter()
                    .all(|(phase, _, _)| target.has_phase(phase))
                && problem
                    .external_gases
                    .iter()
                    .all(|gas| target.has_phase(&gas.phase));
            if !transition_is_reviewed
                || !original_is_representable
                || needs_extended
                || needs_silver_ammine(&problem.elements)
                || !problem.surfaces.is_empty()
                || !problem.exchanges.is_empty()
                || !problem.solid_solutions.is_empty()
                || !vessel.unresolved_materials.is_empty()
                || !vessel.material_objects.is_empty()
                || !vessel.adsorbed.is_empty()
                || !vessel.electrodes.is_empty()
            {
                return Err(canonical_closure_error(
                    AqueousReason::ReplayRepresentationUnsupported,
                    "replay route cannot represent the original analytical and phase owners",
                ));
            }
            db_tag = requested;
            routing = Phrase::bare(
                "routing.provisional-model-replay",
                "provisional model replay from the original material input; final routing is verified by a fresh ordinary aqueous pose",
            );
        }
        if potential_molality > 1.0 && db_tag != "pitzer" {
            // The caveat is a clause AROUND the choice, not a string glued
            // after it: a language that puts the qualification first can
            // write "{routing}" wherever it wants in its own template.
            routing = Phrase::new(
                "routing.activity-model-fallback",
                "{routing}; the input has a potentially concentrated solute load (~{molality} mol/kgw, not a measured ionic strength), but the Pitzer route cannot represent all requested chemistry. This activity-model fallback is not a validated concentrated-mixture prediction",
                vec![
                    ("routing".to_string(), Slot::phrase(routing)),
                    (
                        "molality".to_string(),
                        Slot::number(format!("{potential_molality:.1}")),
                    ),
                ],
            );
            // Saturation and crystallisation of a posed phase at this load
            // are outside the Debye-Hueckel route's activity domain: the
            // hosted validation of b720949 put minteq.v4 Halite at -40%
            // against measurement. Say so as a typed clause rather than
            // let the phase amounts read as a validated prediction.
            let posed: Vec<&str> = problem
                .phases
                .iter()
                .map(|(name, _, _)| name.as_str())
                .collect();
            if !posed.is_empty() {
                routing = Phrase::new(
                    "routing.crystallisation-outside-activity-domain",
                    "{routing}. Saturation and crystallisation of {phases} at this load lie outside this route's validated activity domain (Debye-Hueckel, up to 1 mol/kgw), so those phase amounts are not a validated prediction",
                    vec![
                        ("routing".to_string(), Slot::phrase(routing)),
                        ("phases".to_string(), Slot::text(posed.join(", "))),
                    ],
                );
            }
        }
        if !problem.surfaces.is_empty() {
            if potential_molality > 1.0 && db_tag != "minteq.v4" {
                return Err(SolveError::NotConverged {
                    reason: Some(Box::new(AqueousReason::ActivityModelUnavailable.phrase())),
                    solver: self.name().to_string(),
                    detail: "no approved dataset currently covers both concentrated-solution activity and typed zinc HFO adsorption".to_string(),
                });
            }
            let untracked = if db_tag == "minteq.v4" {
                UNTRACKED_HFO_MINTEQ_ELEMENTS
            } else {
                UNTRACKED_HFO_WATEQ_ELEMENTS
            };
            let mut unsupported: Vec<&str> = problem
                .elements
                .iter()
                .map(|element| element.split('(').next().unwrap_or(element))
                .filter(|element| untracked.contains(element))
                .collect();
            unsupported.sort_unstable();
            unsupported.dedup();
            if !unsupported.is_empty() {
                return Err(SolveError::NotConverged {
                    reason: Some(Box::new(AqueousReason::InterfaceLedgerUnsupported.phrase())),
                    solver: self.name().to_string(),
                    detail: format!(
                        "the {db_tag} HFO model can adsorb {}, but this version cannot yet retain those complexes on its typed interface ledger",
                        unsupported.join(", ")
                    ),
                });
            }
        }
        if !problem.exchanges.is_empty() {
            if db_tag != "wateq4f" || potential_molality > 1.0 {
                return Err(SolveError::NotConverged {
                    reason: Some(Box::new(AqueousReason::DiluteExchangeDomain.phrase())),
                    solver: self.name().to_string(),
                    detail: "typed cation exchange is currently validated only for dilute inorganic water through wateq4f.dat"
                        .to_string(),
                });
            }
            let mut unsupported: Vec<&str> = problem
                .elements
                .iter()
                .map(|element| element.split('(').next().unwrap_or(element))
                .filter(|element| UNTRACKED_EXCHANGE_ELEMENTS.contains(element))
                .collect();
            unsupported.sort_unstable();
            unsupported.dedup();
            if !unsupported.is_empty() {
                return Err(SolveError::NotConverged {
                    reason: Some(Box::new(AqueousReason::InterfaceLedgerUnsupported.phrase())),
                    solver: self.name().to_string(),
                    detail: format!(
                        "the wateq4f exchanger can bind {}, but this version retains only H, Na, Ca and Mg on its typed exchanger ledger",
                        unsupported.join(", ")
                    ),
                });
            }
        }
        if !problem.solid_solutions.is_empty() {
            if db_tag != "wateq4f" || potential_molality > 1.0 {
                return Err(SolveError::NotConverged {
                    reason: Some(Box::new(AqueousReason::DiluteCrystalDomain.phrase())),
                    solver: self.name().to_string(),
                    detail: "typed aragonite-strontianite solid solution is currently validated only for dilute inorganic water through wateq4f.dat"
                        .to_string(),
                });
            }
            for component in SolidSolutionComponent::ALL {
                let phase = phreeqc_solid_solution_component(component);
                if !derived::index_for(db_tag).has_phase(phase) {
                    return Err(SolveError::NotConverged {
                        reason: Some(Box::new(
                            AqueousReason::SolidSolutionPhaseUnavailable.phrase(),
                        )),
                        solver: self.name().to_string(),
                        detail: format!(
                            "the routed database does not define the reviewed solid-solution end member {phase}"
                        ),
                    });
                }
            }
        }
        // Phases the routed database does not define must not reach the
        // input. Zero-amount candidates are dropped; a solid-backed
        // anhydrous phase (e.g. solid KCl on the wateq4f route — Sylvite is
        // pitzer-only) dissolves into element totals instead, i.e. is
        // treated as freely soluble in that database's domain. Hydrate
        // phases are kept so the engine errors honestly rather than the
        // ledger losing their crystal water silently.
        let idx = derived::index_for(db_tag);
        // Every gas posed to the solve must be defined on the routed
        // database, natively or through the consistency-checked import in
        // `gas_phases::reconcile`. Silently dropping a component would leave
        // a stale headspace beside a seemingly complete acid/base
        // equilibrium, so refuse before mutation instead of borrowing
        // unreviewed phase data (docs/gas-phase-coverage-20261027.md).
        if let Some((phase, ..)) = problem
            .gases
            .iter()
            .find(|(phase, ..)| !idx.has_phase(phase))
        {
            return Err(SolveError::NotConverged {
                reason: Some(Box::new(AqueousReason::GasPhaseUnavailable.phrase())),
                solver: self.name().to_string(),
                detail: format!(
                    "finite gas/liquid equilibrium requires {phase}, which the routed {db_tag} database does not define and no reviewed vendored database could supply consistently; this headspace is not modeled"
                ),
            });
        }
        problem.gases.retain(|(phase, ..)| idx.has_phase(phase));
        let mut freed: Vec<(String, f64)> = Vec::new();
        // What was freed, as phases rather than loose elements — the heat
        // is owed to the *substance* that dissolved, not to its atoms.
        let mut freed_phases: Vec<(String, f64)> = Vec::new();
        // A soluble sorbate feed must have the same canonical input on its
        // first and later surface solves. Presenting ZnSO4 first as a finite
        // equilibrium phase but later as aqueous Zn/S totals changed HFO
        // occupancy by about eight percent on an otherwise no-op stir. Fold
        // this reviewed soluble reagent into totals up front, retain the
        // zero-amount phase so precipitation remains possible, and retain
        // the phase identity separately for its dissolution event/enthalpy.
        if !problem.surfaces.is_empty() {
            for (name, moles, _) in &mut problem.phases {
                if *moles <= 0.0 {
                    continue;
                }
                let Some(phase) = derived::phase_by_name(name) else {
                    continue;
                };
                if phase.species != "ZnSO4" {
                    continue;
                }
                for (element, coefficient) in &phase.elements {
                    freed.push((element.clone(), coefficient * *moles));
                }
                freed_phases.push((name.clone(), *moles));
                *moles = 0.0;
            }
        }
        let coupled_now =
            redox_coupling_totals(&phase_inclusive_totals(&problem), db_tag).is_some();
        problem.phases.retain_mut(|(name, moles, _)| {
            if idx.has_phase(name) {
                return true;
            }
            // A reviewed alternate/foreign phase bounds supplied solid just as
            // it bounds a zero-inventory precipitation candidate. Freeing it
            // first made successive solves alternate solid and dissolved stock.
            // Preserve the coupled-redox and typed-surface admission domains.
            if *moles > 0.0 && !coupled_now && problem.surfaces.is_empty() {
                if let Some(posed) = posed_phase(name, db_tag).map(str::to_string) {
                    *name = posed;
                    return true;
                }
            }
            if *moles > 0.0 {
                if let Some(p) = derived::phase_by_name(name) {
                    if p.waters == 0.0 {
                        for (el, c) in &p.elements {
                            freed.push((el.clone(), *c * *moles));
                        }
                        freed_phases.push((name.clone(), *moles));
                        return false;
                    }
                }
                return true;
            }
            // A zero-amount candidate whose species the routed database
            // holds under a different polymorph name (the global dedupe
            // keeps one name per species across databases) is renamed to
            // that database's polymorph, not dropped — otherwise a
            // precipitate like Fe(OH)3 is silently impossible on any route
            // whose database spells it differently.
            // A coupled or surface-bearing solve keeps the native-only
            // phase set it was validated with. Coupled: a translated or
            // injected phase would let a coupled element precipitate
            // mid-bisect, and the electron budget is summed over
            // *dissolved* states — with mass in a solid, that budget can
            // never close and the coupling dies. Surfaces: the typed HFO
            // ledger carries dissolved and bound inventories only, and a
            // newly precipitable hydroxide put 6.1e-6 mol of zinc in a
            // third pool the transport ledger does not have.
            if coupled_now || !problem.surfaces.is_empty() {
                return false;
            }
            // Translated to the routed database's polymorph, or kept under
            // its own name if it is a reviewed foreign phase the input will
            // define; anything else is dropped. `posed_phase` is the whole
            // rule, and the MIX builder now applies the same one.
            match posed_phase(name, db_tag).map(str::to_string) {
                Some(posed) => {
                    *name = posed;
                    true
                }
                None => false,
            }
        });
        problem.external_gases.retain(|exchange| {
            problem
                .phases
                .iter()
                .any(|(phase, ..)| phase == &exchange.phase)
        });
        for (el, n) in freed {
            if let Some(entry) = problem.totals.iter_mut().find(|(e, _)| *e == el) {
                entry.1 += n;
            } else {
                problem.totals.push((el, n));
            }
        }
        let input = build_input(vessel, &problem, db_tag);
        // Old caches predate independent native N/S balances and the reviewed
        // ligand slice. They must miss, never masquerade as current chemistry.
        let database_hash = crate::native_namespace::fingerprint(db_tag).map_err(|detail| {
            SolveError::NotConverged {
                reason: Some(Box::new(
                    AqueousReason::ComponentFingerprintUnavailable.phrase(),
                )),
                solver: "phreeqc-component-isolation".into(),
                detail,
            }
        })?;
        let key = format!("#aqueous-isolated-v1:{db_tag}:{database_hash}\n{input}");

        Ok(Some(SolveSetup {
            problem,
            db_tag,
            routing,
            freed_phases,
            input,
            key,
        }))
    }

    /// Ask `pitzer.dat` for the solvent's activity, and nothing else.
    ///
    /// **Why a second solve exists at all.** The router sends a solution to
    /// the ion-interaction dataset only when it is CONCENTRATED — above
    /// 1 mol/kgw, where the Debye-Hückel datasets are out of their validity
    /// domain for the chemistry. Below that the chemistry is fine on
    /// `wateq4f.dat` and the solvent's activity is not: PHREEQC does not
    /// model a_w on a Debye-Hückel database, it prints the hard-coded
    /// `1 - 0.017*Sum(m)` placeholder, which carries no information about
    /// what is dissolved. `states::SolventActivity` therefore declines it
    /// and Raoult's law stands in, which put a tenth-molal brine at
    /// -0.371 °C: a dissolved salt's whole effect on its solvent taken as
    /// a mole-fraction correction, with nothing in it about the salt. That
    /// is the same defect, in the same direction and for the same reason,
    /// as the one at one molal that 2026-09-11 fixed at the concentrated
    /// end. No measured value is quoted here or asserted anywhere in this
    /// feature's tests; `colligative_numbers.rs` says why, and PLAN.md
    /// records the world-facing anchor as still owed.
    ///
    /// So the gap is not in the chemistry and does not want the chemistry
    /// re-solved. It is one number, and this asks one dataset for it.
    ///
    /// **What is posed.** Element totals and the solvent mass, and nothing
    /// else: no phases, no gas phase, no surfaces, no exchangers, no solid
    /// solutions. That is deliberate in both directions. It is cheap — the
    /// engine has no phase assemblage to iterate against — and it is
    /// *cache-dense*, because the key is the input string and this input
    /// drops everything about the vessel that does not change the solution's
    /// composition. Two steps that differ only in how much undissolved
    /// solid is sitting at the bottom pose the same solvent question and
    /// the second is free.
    ///
    /// **When it is asked.** Only inside a band where the answer would both
    /// differ from Raoult's law and be trustworthy, and only where posing
    /// the lean problem cannot misrepresent the solution:
    ///
    /// 1. the chemistry did NOT route to `pitzer` — if it did, the `H2O`
    ///    row of that solve is already the activity and a second one would
    ///    be the same solve twice;
    /// 2. `pitzer.dat` carries every element, or it cannot answer;
    /// 3. no surfaces, exchangers or solid solutions, and no gas phase —
    ///    each of those takes ions out of (or puts them into) the solution,
    ///    so the totals handed to the lean problem would not be the
    ///    solution's;
    /// 4. no solid phase survived the main solve with a positive amount,
    ///    for the same reason: a saturated solution's dissolved totals are
    ///    not its input totals, and the lean problem has no phase to hold
    ///    the excess;
    /// 5. dissolved particles are at or above
    ///    [`SECOND_OPINION_MIN_MOLALITY`]. This preserves the previously
    ///    admitted band. Numerical water activity removes report rounding,
    ///    but is not evidence for widening the solvent model's domain.
    ///
    /// A failure is not an error: a dataset that cannot answer leaves
    /// `None`, the vessel keeps the ideal route it had before this existed,
    /// and nothing about the chemistry is disturbed.
    ///
    /// **That fallback is also the one thing to watch on the wasm side.** A
    /// cache-only build has no engine, so this solve is a cache lookup and
    /// nothing else; a key the shipped cache does not carry silently leaves
    /// the vessel on the ideal route while the native bench takes the
    /// ion-interaction one, and that is the 2026-08 native/wasm one-value
    /// hazard wearing a new coat. It does not bite, because the cache is
    /// generated by replaying the same lessons through this same code path
    /// (`kero prewarm`), so a vessel whose main solve is in the cache has
    /// its solvent question in there beside it — both keys are written by
    /// the same run. The invariant to keep is that pairing: anything that
    /// warms one key must warm the other, and "The two wasm halves,
    /// together" is the gate that would notice if it stopped.
    ///
    /// **How a reader tells which happened.** Two ways, and they answer
    /// different questions. `SolutionInfo::solvent_activity` is `Some` on
    /// exactly the vessels that cost two solves, and names the dataset the
    /// activity came from — which is not the dataset in `provenance` that
    /// the pH and the speciation came from. And `Provenance::routing`,
    /// which is rendered wherever provenance is, gains a sentence saying so
    /// in prose. A vessel that took one solve has neither.
    fn solvent_activity_second_opinion(
        &mut self,
        vessel: &Vessel,
        problem: &Problem,
        db_tag: &str,
        value: &dyn Fn(&str) -> Option<f64>,
        hint: SecondaryFirstHint,
    ) -> SecondaryResult {
        match Self::prepare_secondary(vessel, problem, db_tag, value, hint) {
            PreparedSecondary::Resolved(result) => *result,
            PreparedSecondary::Question(question) => {
                if question.validate_input().is_err() {
                    return SecondaryResult::declined(SecondaryDisposition::Declined(
                        SecondaryDecline::InvalidResult,
                    ));
                }
                self.fulfil_secondary_question(&question)
            }
        }
    }

    fn prepare_secondary(
        vessel: &Vessel,
        problem: &Problem,
        db_tag: &str,
        value: &dyn Fn(&str) -> Option<f64>,
        hint: SecondaryFirstHint,
    ) -> PreparedSecondary {
        const SECOND: &str = "pitzer";
        // `KERO_TRACE_SOLVENT_ACTIVITY=1` says which of the gates below
        // turned a vessel away, in the same spirit as `KERO_DUMP_INPUT`.
        // Every gate here is a silent `None`, and a silent `None` that is
        // one gate too wide looks exactly like a feature that does not
        // work: the first version of this declined every OPEN beaker on a
        // condition that read as "no gas boundary", which is most of this
        // bench, and nothing said so.
        let decline = |disposition: SecondaryDisposition, reason: &str| -> PreparedSecondary {
            if std::env::var("KERO_TRACE_SOLVENT_ACTIVITY").is_ok() {
                eprintln!("solvent-activity second opinion declined: {reason}");
            }
            PreparedSecondary::Resolved(Box::new(SecondaryResult::declined(disposition)))
        };
        if hint == SecondaryFirstHint::Skip {
            return PreparedSecondary::Resolved(Box::new(SecondaryResult::declined(
                SecondaryDisposition::SkippedForReplay,
            )));
        }
        if db_tag == SECOND {
            return decline(
                SecondaryDisposition::Declined(SecondaryDecline::AlreadyPrimary),
                "the chemistry already routed to pitzer, so its own H2O row is the activity",
            );
        }
        if !problem.surfaces.is_empty()
            || !problem.exchanges.is_empty()
            || !problem.solid_solutions.is_empty()
            || !problem.gases.is_empty()
        {
            return decline(
                SecondaryDisposition::Declined(SecondaryDecline::Unsupported),
                "a surface, exchanger, solid solution or finite gas headspace owns part of this composition",
            );
        }
        // An external gas RESERVOIR is not a reason to decline, and reading
        // it as one is what made the first version of this feature fire on
        // almost nothing. `partition` gives every OPEN vessel an
        // atmospheric reservoir — `O2(g)` unconditionally, because its
        // element requirement outside H and O is empty and therefore
        // vacuously met — so "no external gases" meant "not an open
        // beaker", which is most of this bench. The reservoir is a phase
        // offered holding ZERO moles: PHREEQC can precipitate into it and
        // cannot dissolve from it, so it can only take matter OUT, and
        // anything it took out shows up as a positive amount in the phase
        // check below. Uptake from the room does not come through it at
        // all — `GasExchangeClock` sizes that and `partition` has already
        // put it in the element totals this lean problem is posed from.
        //
        // A DOSE is different and is still declined. A finite dose is
        // carried in `phases` with its moles, not in `totals`, so stripping
        // the phase list would silently delete matter that may have gone
        // into solution.
        if problem
            .external_gases
            .iter()
            .any(|gas| !matches!(gas.kind, ExternalGasKind::Reservoir))
        {
            return decline(
                SecondaryDisposition::Declined(SecondaryDecline::Unsupported),
                "an external gas DOSE is carried as a phase, not as a total",
            );
        }
        let brine = derived::index_for(SECOND);
        if !problem.elements.iter().all(|el| brine.has_element(el)) {
            return decline(
                SecondaryDisposition::Declined(SecondaryDecline::Unsupported),
                "pitzer.dat cannot express every element here",
            );
        }
        // A phase that came out of the solve holding matter is a phase the
        // lean problem has nowhere to put. For a mineral that means the
        // solution is saturated against it and its dissolved totals are not
        // its input totals; for an atmospheric reservoir it means gas left
        // the liquid within the step. Either way the lean problem, which
        // has no phases at all, would be posed from totals that are not the
        // solution's — so it is not asked.
        // The lean problem is posed from the solution AS SOLVED, not from
        // the problem as posed, and that distinction is the whole of why
        // the first three versions of this declined a plain beaker of
        // brine. `partition` does not put a dissolving SOLID in `totals`:
        // sodium chloride added to water arrives as a `Halite` phase
        // carrying its moles, and `totals` for that beaker is empty. A
        // lean problem built by clearing `phases` therefore threw the
        // entire solute away and asked pitzer for the activity of pure
        // water, which is below any floor and was declined as too dilute —
        // a correct-looking refusal for a completely wrong reason.
        //
        // The selected output already carries the right numbers.
        // `build_input` asks for `-totals` over every element, so
        // `value(el)` is that element's DISSOLVED total in mol/kgw after
        // the solve: the salt that went into solution is in it, the solid
        // still sitting at the bottom is not, and the room's uptake is
        // already folded in. Posing from those is both correct and more
        // general than posing from the input — a saturated solution's
        // dissolved totals are its solved totals, so no phase list is
        // needed to hold the excess and none of this has to decline a
        // solution merely for having a solid in the beaker.
        let solved_kgw = value("mass_H2O").unwrap_or(problem.kgw);
        if !(solved_kgw.is_finite() && solved_kgw > 0.0) {
            return decline(
                SecondaryDisposition::Declined(SecondaryDecline::MissingReadback),
                "no solvent mass",
            );
        }
        // A valence-tagged element would need `build_input`'s own merge to
        // be repeated here, and every such problem is a redox one that the
        // gate below declines anyway. Declining early keeps one rule.
        if problem.elements.iter().any(|el| el.contains('(')) {
            return decline(
                SecondaryDisposition::Declined(SecondaryDecline::Unsupported),
                "a valence-tagged element belongs to a redox problem",
            );
        }
        let mut dissolved: Vec<(String, f64)> = Vec::new();
        for el in &problem.elements {
            match value(el) {
                Some(molality) => {
                    if !molality.is_finite() || molality < 0.0 {
                        return decline(
                            SecondaryDisposition::Declined(SecondaryDecline::InvalidResult),
                            &format!("negative or nonfinite dissolved total read back for {el}"),
                        );
                    }
                    if molality > 0.0 {
                        dissolved.push((el.clone(), molality * solved_kgw));
                    }
                }
                // An element with no readback column would be silently
                // dropped from the lean problem, which is matter quietly
                // disappearing — the one thing this engine keeps having to
                // root out. Decline instead.
                None => {
                    return decline(
                        SecondaryDisposition::Declined(SecondaryDecline::MissingReadback),
                        &format!("no dissolved total read back for {el}"),
                    );
                }
            }
        }
        let posed: f64 = dissolved.iter().map(|(_, n)| n).sum::<f64>() / solved_kgw;
        if !posed.is_finite() {
            return decline(
                SecondaryDisposition::Declined(SecondaryDecline::InvalidResult),
                "nonfinite dissolved element total molality",
            );
        }
        let mut lean = problem.clone();
        lean.totals = dissolved;
        lean.kgw = solved_kgw;
        lean.phases.clear();
        lean.gases.clear();
        lean.external_gases.clear();
        lean.surfaces.clear();
        lean.exchanges.clear();
        lean.solid_solutions.clear();
        // A coupled solve bisects for pe and is the most expensive shape this
        // engine has. It is also not what is being asked for: the question is
        // the solvent's activity, and a beaker whose electron budget needs
        // solving can keep the route it had. Declining here is what keeps the
        // worst case of this feature one ordinary solve rather than a search.
        if redox_coupling(&lean, SECOND).is_some() {
            return decline(
                SecondaryDisposition::Declined(SecondaryDecline::Unsupported),
                "the lean problem would be redox-coupled",
            );
        }

        let input = build_input(vessel, &lean, SECOND);
        let Ok(database_hash) = crate::native_namespace::fingerprint(SECOND) else {
            return decline(
                SecondaryDisposition::Declined(SecondaryDecline::Unsupported),
                "pitzer.dat has no fingerprint",
            );
        };
        // Classify the floor only after every non-native capability gate.
        // A typed floor witness must mean the lean question would otherwise
        // be admissible, rather than hiding an unsupported redox owner.
        if posed < SECOND_OPINION_MIN_MOLALITY && hint != SecondaryFirstHint::Probe {
            return decline(
                SecondaryDisposition::BelowInputFloor,
                "dissolved element totals are below the molality floor",
            );
        }
        let key = format!("#solvent-activity-v1:{SECOND}:{database_hash}\n{input}");
        PreparedSecondary::Question(Box::new(ExactAuxiliaryQuestion {
            pose: vessel.clone(),
            problem: lean,
            database: SECOND,
            database_fingerprint: database_hash.to_string(),
            input,
            key,
            witness: None,
        }))
    }

    fn fulfil_secondary_question(&mut self, question: &ExactAuxiliaryQuestion) -> SecondaryResult {
        const SECOND: &str = "pitzer";
        let vessel = &question.pose;
        let lean = &question.problem;
        let input = &question.input;
        let key = question.key.clone();
        let brine = derived::index_for(SECOND);
        let decline = |disposition: SecondaryDisposition, reason: &str| -> SecondaryResult {
            if std::env::var("KERO_TRACE_SOLVENT_ACTIVITY").is_ok() {
                eprintln!("solvent-activity second opinion declined: {reason}");
            }
            SecondaryResult::declined(disposition)
        };
        let engine_before = self.engine_calls;
        let cached = match self.dispatch_solve(vessel, lean, SECOND, input, key) {
            Ok((cached, _)) => cached,
            Err(error) => {
                return decline(
                    SecondaryDisposition::Declined(SecondaryDecline::EngineFailure),
                    &format!("the second solve refused: {error}"),
                );
            }
        };
        if self.engine_calls > engine_before {
            self.solvent_activity_engine_calls += 1;
        } else {
            self.solvent_activity_cache_hits += 1;
        }

        let Some(water) = cached.speciation.iter().find(|sp| sp.name == "H2O") else {
            return decline(
                SecondaryDisposition::Declined(SecondaryDecline::MissingReadback),
                "the second speciation reported no H2O row",
            );
        };
        let Ok(particle_molality) = numerical_particle_molality(&cached.rows) else {
            return decline(
                SecondaryDisposition::Declined(SecondaryDecline::MissingReadback),
                "the second solve reported no numerical particle census",
            );
        };
        if !particle_molality.is_finite() || particle_molality < SECOND_OPINION_MIN_MOLALITY {
            return decline(
                SecondaryDisposition::Declined(SecondaryDecline::BelowOutputParticleFloor),
                "the second speciation reported too few particles",
            );
        }
        let ionic_strength = match secondary_ionic_strength(&cached.rows) {
            Ok(mu) => mu,
            Err(reason) => {
                return decline(
                    SecondaryDisposition::Declined(reason),
                    "the second solve reported missing, negative or nonfinite ionic strength",
                );
            }
        };
        // The same plausibility gate `SolventActivity::from_speciation`
        // applies, applied here too so a useless answer is not recorded as
        // provenance. An activity that fails it would leave the vessel on
        // the ideal route regardless, and a `Some` that meant "ideal" would
        // be the most misleading thing this field could hold.
        if kerotakis_core::states::SolventActivity::from_speciation(
            water.activity,
            particle_molality,
            ionic_strength,
        )
        .osmotic_coefficient
        .is_none()
        {
            return decline(
                SecondaryDisposition::Declined(SecondaryDecline::InvalidResult),
                &format!(
                    "the activity is not one an osmotic model produced: a_w = {} at {particle_molality} mol/kgw",
                    water.activity
                ),
            );
        }

        let dataset = dataset_phrase(SECOND);
        let model = brine.activity_model.phrase();
        SecondaryResult {
            provenance: Some(kerotakis_core::vessel::SolventActivityProvenance {
                dataset: dataset.render(Locale::EN),
                model: model.render(Locale::EN),
                dataset_phrase: Some(dataset),
                model_phrase: Some(model),
                water_activity: water.activity,
                particle_molality,
                ionic_strength,
            }),
            disposition: SecondaryDisposition::Accepted,
        }
    }

    fn fulfil_aqueous_pass(
        &mut self,
        vessel: &mut Vessel,
        pass: AqueousPass,
    ) -> Result<AqueousPass, SolveError> {
        if !pass.has_pending_secondary()? {
            return Err(canonical_closure_error(
                AqueousReason::QuestionMissing,
                "no queued auxiliary question to fulfil",
            ));
        }
        let question = pass.pending_secondary.as_ref().expect("checked question");
        question.validate_pose(vessel, &pass)?;
        let result = self.fulfil_secondary_question(question);
        Self::complete_secondary_result(vessel, pass, result)
    }

    fn complete_secondary_result(
        vessel: &mut Vessel,
        mut pass: AqueousPass,
        result: SecondaryResult,
    ) -> Result<AqueousPass, SolveError> {
        if !pass.has_pending_secondary()? {
            return Err(canonical_closure_error(
                AqueousReason::QuestionMissing,
                "no queued auxiliary question to fulfil",
            ));
        }
        pass.pending_secondary
            .as_ref()
            .expect("checked question")
            .validate_pose(vessel, &pass)?;
        let question = pass.pending_secondary.take().expect("checked question");
        if result.disposition == SecondaryDisposition::Pending
            || (result.disposition == SecondaryDisposition::Accepted) != result.provenance.is_some()
        {
            return Err(canonical_closure_error(
                AqueousReason::AuxiliaryResultAuthority,
                "native auxiliary answer remained pending",
            ));
        }
        let info = vessel.solution.as_mut().ok_or_else(|| {
            canonical_closure_error(
                AqueousReason::PrimarySolutionMissing,
                "queued auxiliary pose lost its primary solution",
            )
        })?;
        let mut provenance = info.provenance.clone().ok_or_else(|| {
            canonical_closure_error(
                AqueousReason::PrimaryProvenanceMissing,
                "queued auxiliary pose lost its primary provenance",
            )
        })?;
        let routing = provenance.routing_phrase.clone().ok_or_else(|| {
            canonical_closure_error(
                AqueousReason::PrimaryRoutingMissing,
                "queued primary routing lacks its native recipe",
            )
        })?;
        provenance.say_routing(secondary_routing(routing, result.provenance.as_ref()));
        info.solvent_activity = result.provenance;
        info.provenance = Some(provenance.clone());
        // The pending explanation was private. Recompose exactly the eager
        // announcement against its original routing history, before acceptance.
        pass.events
            .retain(|event| !matches!(event, Event::SolutionRouted { .. }));
        vessel.aqueous_routing_said = question.pose.aqueous_routing_said.clone();
        announce_routing(vessel, &provenance, &mut pass.events);
        pass.secondary_disposition = result.disposition;
        Ok(pass)
    }

    fn dispatch_solve(
        &mut self,
        vessel: &Vessel,
        problem: &Problem,
        db_tag: &str,
        input: &str,
        key: String,
    ) -> Result<(Rc<CachedSolve>, Option<String>), SolveError> {
        // Content-addressed cache: database + input string is a
        // deterministic canonicalisation of (species set, amounts, T) — same
        // state, same answer, no engine call.
        let mut coupling_failed: Option<String> = None;
        let coupling = redox_coupling(problem, db_tag);
        let coupled_problem = coupling.is_some();
        // A legacy imported cache has no proof that its pe came from a
        // bracketed electron root. Recompute that one result rather than
        // treating the old default `pe_undetermined=false` as a witness.
        let cached = if let Some(hit) = self.cache.get(&key).filter(|hit| {
            !coupled_problem
                || matches!(
                    hit.pe_authority,
                    PeAuthority::CoupledRoot | PeAuthority::CoupledUnpinned
                )
        }) {
            self.cache_hits += 1;
            Rc::clone(hit)
        } else {
            // Redox elements that equilibrate on a bench timescale are
            // coupled and pe is solved for; everything else keeps the
            // oxidation state it was added in. Which is which is curated —
            // see FAST_REDOX — because it is a claim about rates.
            // `KERO_DUMP_INPUT=1` shows the input only when the engine
            // refuses it; `=all` shows every one. A wrong answer the engine
            // was perfectly happy to give needs the same view as a refused
            // one — diffing two of these is what found the missing
            // dissolution enthalpy below.
            if env_dump_input_all() {
                eprintln!("--- PHREEQC input ---\n{input}---");
            }
            let (out, pe_authority) = match coupling {
                Some(coupling) => match self.solve_coupled(vessel, problem, db_tag, &coupling) {
                    Ok(out) => {
                        let authority = if out.pe_undetermined {
                            PeAuthority::CoupledUnpinned
                        } else {
                            PeAuthority::CoupledRoot
                        };
                        (out, authority)
                    }
                    // A coupled solve can fail where an uncoupled one
                    // succeeds, and the reason is chemistry rather than
                    // arithmetic: iron(II) oxidised near neutral pH wants
                    // to be iron(III), which is insoluble there, and no
                    // iron hydroxide is in our registry for it to become.
                    // Falling back keeps the rest of the answer — pH,
                    // speciation, everything not about electrons — and the
                    // failure is said out loud rather than swallowed.
                    Err(e) => {
                        coupling_failed = Some(e.to_string());
                        (self.run_raw(db_tag, input)?, PeAuthority::Unwitnessed)
                    }
                },
                None => match self.run_raw(db_tag, input) {
                    Ok(out) => (out, open_oxygen_pe_authority(vessel, db_tag, input)),
                    // A refusal may be about how the question was posed
                    // rather than about the chemistry. Ask it the other way
                    // round — most of the salt as solid, dissolving to
                    // saturation — before accepting the failure.
                    //
                    // Read back against the *original* problem, so the
                    // precipitation the vessel is told about is measured
                    // from the solid it actually had, not from the solid we
                    // invented to make the question answerable.
                    Err(e) => match condense_supersaturated(problem) {
                        Some(recast) => {
                            let recast_input = build_input(vessel, &recast, db_tag);
                            let authority = open_oxygen_pe_authority(vessel, db_tag, &recast_input);
                            (
                                self.run_raw(db_tag, &recast_input).map_err(|_| e)?,
                                authority,
                            )
                        }
                        None => return Err(e),
                    },
                },
            };
            let speciation = numerical_species_distribution(&out.report, &out.selected)?;
            let cached = Rc::new(CachedSolve {
                pe_determined: !out.pe_undetermined,
                pe_authority,
                rows: out.selected,
                speciation,
                saturation: parse_saturation_indices(&out.report),
                redox_adjusted: out.report.contains("Adjusted to redox equilibrium"),
            });
            if self.cache.len() >= 10_000 {
                self.cache.clear(); // simple bound; refine when profiling says so
            }
            // A failed coupled root has a real warning on this route. Do
            // not cache its raw fallback without that warning and let a
            // later hit silently turn the fallback into a coupled success.
            if coupling_failed.is_none() {
                self.cache.insert(key, Rc::clone(&cached));
            }
            cached
        };
        Ok((cached, coupling_failed))
    }

    #[allow(clippy::type_complexity)]
    fn readback_raw_values(
        &self,
        problem: &Problem,
        db_tag: &str,
        rows: &[Vec<String>],
        value: &dyn Fn(&str) -> Option<f64>,
    ) -> Result<
        (
            f64,
            Vec<SurfaceSites>,
            Vec<ExchangeSites>,
            Vec<SolidSolution>,
            Vec<(String, f64)>,
            Vec<(String, f64)>,
            BTreeMap<String, Vec<(&'static str, f64)>>,
        ),
        SolveError,
    > {
        // Read back: element totals (mol/kgw) and phase amounts (mol).
        // Molalities are per kg of *equilibrated* water (mass_H2O), which
        // differs slightly from the input water mass through speciation.
        let kgw_out = value("mass_H2O").ok_or_else(|| missing(rows, "mass_H2O"))?;
        if !problem.solid_solutions.is_empty() {
            if !kgw_out.is_finite() || kgw_out <= 0.0 {
                return Err(crystal_balance_error(
                    CrystalReason::SolventMassInvalid,
                    "invalid aqueous solvent mass",
                ));
            }
            let states = valence_totals(problem, db_tag);
            for column in problem.elements.iter().chain(&states) {
                let base = column.split('(').next().unwrap_or(column);
                if !["Ca", "Sr", "C"].contains(&base) {
                    continue;
                }
                for name in [column.as_str(), base] {
                    let raw = value(name).ok_or_else(|| missing(rows, name))?;
                    checked_crystal_sum(&[raw], &format!("raw selected {name}"))?;
                    checked_crystal_sum(&[raw * kgw_out], &format!("aqueous {name}"))?;
                }
            }
        }
        let mut solvent_kgw_out = kgw_out;
        let mut new_surfaces = problem.surfaces.clone();
        if !new_surfaces.is_empty() {
            let strong =
                value("m_Hfo_sOZn+").ok_or_else(|| missing(rows, "m_Hfo_sOZn+"))? * kgw_out;
            let weak = value("m_Hfo_wOZn+").ok_or_else(|| missing(rows, "m_Hfo_wOZn+"))? * kgw_out;
            // The reviewed HFO datasets also bind the sulfate
            // counterion used by the zinc-sulfate lesson. Both weak-site
            // complexes contain one sulfate and occupy one site; failing to
            // round-trip them made sulfur (and its mass) disappear from the
            // vessel after every equilibrium pass.
            let weak_sulfate_water =
                value("m_Hfo_wSO4-").ok_or_else(|| missing(rows, "m_Hfo_wSO4-"))? * kgw_out;
            let weak_sulfate_hydroxyl =
                value("m_Hfo_wOHSO4-2").ok_or_else(|| missing(rows, "m_Hfo_wOHSO4-2"))? * kgw_out;
            let weak_sulfate = weak_sulfate_water + weak_sulfate_hydroxyl;
            // MINTEQ additionally defines the equivalent strong-site pair.
            let (strong_sulfate, strong_sulfate_water) = if db_tag == "minteq.v4" {
                let water =
                    value("m_Hfo_sSO4-").ok_or_else(|| missing(rows, "m_Hfo_sSO4-"))? * kgw_out;
                let hydroxyl = value("m_Hfo_sOHSO4-2")
                    .ok_or_else(|| missing(rows, "m_Hfo_sOHSO4-2"))?
                    * kgw_out;
                (water + hydroxyl, water)
            } else {
                (0.0, 0.0)
            };
            for surface in &mut new_surfaces {
                surface.occupancy.clear();
                surface.water_release = Moles(0.0);
            }
            distribute_surface_occupancy(
                &mut new_surfaces,
                SurfaceSiteKind::Strong,
                SurfaceSorbate::Zinc,
                strong,
            );
            distribute_surface_occupancy(
                &mut new_surfaces,
                SurfaceSiteKind::Weak,
                SurfaceSorbate::Zinc,
                weak,
            );
            distribute_surface_occupancy(
                &mut new_surfaces,
                SurfaceSiteKind::Strong,
                SurfaceSorbate::Sulfate,
                strong_sulfate,
            );
            distribute_surface_occupancy(
                &mut new_surfaces,
                SurfaceSiteKind::Weak,
                SurfaceSorbate::Sulfate,
                weak_sulfate,
            );
            // Surface complex amounts are the authoritative *surface*
            // transfer. `mass_H2O` is not representation-invariant here: a
            // first solve fed by an amount-limited ZnSO4 phase includes this
            // water, while the identical state rebuilt from aqueous totals
            // does not. Booking the named ligand-exchange complexes against
            // the neutral input reference makes both paths converge to the
            // same typed interface and solvent ledgers.
            let modeled_water = strong_sulfate_water + weak_sulfate_water;
            if modeled_water > TRACE {
                distribute_surface_water_release(
                    &mut new_surfaces,
                    SurfaceSiteKind::Strong,
                    strong_sulfate_water,
                );
                distribute_surface_water_release(
                    &mut new_surfaces,
                    SurfaceSiteKind::Weak,
                    weak_sulfate_water,
                );
                solvent_kgw_out = problem.kgw + modeled_water * WATER_MOLAR_MASS / 1000.0;
            }
        }
        let mut new_exchanges = problem.exchanges.clone();
        if !new_exchanges.is_empty() {
            for exchange in &mut new_exchanges {
                exchange.occupancy.clear();
            }
            for ion in [
                ExchangeIon::Hydrogen,
                ExchangeIon::Sodium,
                ExchangeIon::Calcium,
                ExchangeIon::Magnesium,
            ] {
                let column = format!("m_{}", phreeqc_exchange_species(ion));
                let moles = value(&column).ok_or_else(|| missing(rows, &column))? * kgw_out;
                distribute_exchange_occupancy(&mut new_exchanges, ion, moles);
            }
            if let Some(exchange) = new_exchanges
                .iter()
                .find(|exchange| !exchange.has_valid_capacity())
            {
                return Err(SolveError::NotConverged {
                    reason: Some(Box::new(AqueousReason::ExchangeReadbackInvalid.phrase())),
                    solver: self.name().to_string(),
                    detail: format!(
                        "PHREEQC returned exchange occupancy inconsistent with finite capacity for '{}'",
                        exchange.label
                    ),
                });
            }
        }
        let mut new_solid_solutions = problem.solid_solutions.clone();
        if let Some(solid_solution) = new_solid_solutions.first_mut() {
            for amount in &mut solid_solution.components {
                let phase = phreeqc_solid_solution_component(amount.component);
                let column = format!("s_{phase}");
                let raw = value(&column).ok_or_else(|| missing(rows, &column))?;
                if !raw.is_finite() || raw < 0.0 {
                    return Err(SolveError::NotConverged {
                        reason: Some(Box::new(AqueousReason::ComponentReadbackInvalid.phrase())),
                        solver: self.name().to_string(),
                        detail: format!(
                            "PHREEQC returned an invalid finite nonnegative component amount for '{column}': {raw}"
                        ),
                    });
                }
                amount.moles = Moles(raw);
            }
            if !solid_solution.has_valid_state() {
                return Err(SolveError::NotConverged {
                    reason: Some(Box::new(AqueousReason::ComponentReadbackInvalid.phrase())),
                    solver: self.name().to_string(),
                    detail: format!(
                        "PHREEQC returned an invalid component inventory for solid solution '{}'",
                        solid_solution.label
                    ),
                });
            }
        }
        let mut new_ions: Vec<(String, f64)> = Vec::new();
        let mut unnameable: Vec<(String, f64)> = Vec::new();
        if env_readback() {
            eprintln!("readback: kgw_out={kgw_out:.12e}");
            if let (Some(h), Some(r)) = (rows.first(), rows.last()) {
                for (name, v) in h.iter().zip(r.iter()) {
                    if !name.is_empty() {
                        eprintln!("   {name:<12} {v}");
                    }
                }
            }
            eprintln!("   elements={:?}", problem.elements);
        }
        // A coupled element no longer sits in the one oxidation state it
        // was added in — that is the point — so reading back only the state
        // it came in as *loses the rest*. Permanganate in brine settles at
        // 96% Mn(VII), 3% Mn(VI), 1% Mn(II), and reading the Mn(7) column
        // alone quietly discarded 4% of the manganese on every step. Each
        // state is booked as its own ion; a state with no name in the
        // registry is reported rather than dropped.
        // Not only the *coupled* elements: PHREEQC equilibrates an
        // element's own oxidation states against pe whether or not it may
        // trade electrons with anything else, so permanganate alone in
        // brine still settles at 96/3/1 across Mn(VII), Mn(VI) and Mn(II).
        // Every redox element present therefore has to be read state by
        // state.
        let coupled: Vec<String> = valence_totals(problem, db_tag);
        let coupled_bases: Vec<&str> = coupled.iter().filter_map(|c| c.split('(').next()).collect();
        for el in &problem.elements {
            let base = el.split('(').next().unwrap_or(el);
            if coupled_bases.contains(&base) {
                continue; // handled per oxidation state below
            }
            let molality =
                checked_component_readback(el, value(el).ok_or_else(|| missing(rows, el))?)?;
            new_ions.push((el.clone(), molality * kgw_out));
        }
        // Conservation by construction: the element's own total is
        // authoritative and its oxidation states only decide how to name
        // it. Booking the states directly made their rounding the vessel's
        // arithmetic, and the error compounded on every step — nitrogen
        // drifted 0.25% each time the beaker was touched.
        let bases: std::collections::BTreeSet<&str> = coupled_bases.iter().copied().collect();
        for base in bases {
            let split: Vec<(&String, f64)> = coupled
                .iter()
                .filter(|c| c.split('(').next() == Some(base))
                .filter_map(|c| value(c).map(|m| checked_component_readback(c, m).map(|m| (c, m))))
                .collect::<Result<_, _>>()?;
            let sum: f64 = split.iter().map(|(_, m)| m).sum();
            let total = checked_component_readback(base, value(base).unwrap_or(sum))? * kgw_out;
            if total <= TRACE {
                continue;
            }
            // An uncoupled element keeps the oxidation state it was added
            // in — the FAST_REDOX law — but PHREEQC's reaction step still
            // redistributes a trace across states against pe. Booking that
            // trace made it real: iron(II) sulfate alone in water read back
            // a sliver of iron(III), the next solve saw iron(III) present
            // and admitted a ferric phase for it, and the ferric solid then
            // pulled the whole inventory across — lye precipitated
            // iron(III) hydroxide from a ferrous salt with no oxidant in
            // the beaker. With no redox partner, the dissolved total goes
            // back to the vessel in the state distribution it went in as.
            // N/S are independently conserved by the native component
            // problem. Reassigning those totals here would erase legitimate
            // mixed-state feeds entering through a mineral or MIX.
            if FAST_REDOX.contains(&base) && redox_coupling(problem, db_tag).is_none() {
                let inputs: Vec<(&String, f64)> = problem
                    .totals
                    .iter()
                    .filter(|(k, _)| k.split('(').next().unwrap_or(k) == base)
                    .map(|(k, n)| (k, *n))
                    .collect();
                let total_in: f64 = inputs.iter().map(|(_, n)| n).sum();
                if total_in > 0.0 {
                    for (key, n_in) in inputs {
                        let moles = total * n_in / total_in;
                        if moles <= TRACE {
                            continue;
                        }
                        if derived::booking_ion(key).is_some() {
                            new_ions.push((key.clone(), moles));
                        } else {
                            unnameable.push((key.clone(), moles));
                        }
                    }
                    continue;
                }
            }
            // An element can be redox-active and still have no *tagged*
            // state carrying anything: the databases name carbon(IV) as
            // plain "C", so bicarbonate's only tagged state is C(−IV) and
            // it is empty. Skipping on a zero split threw the carbon away —
            // every carbonate solution lost all of it. With nothing to
            // distribute, the element is simply booked as itself.
            if sum <= 0.0 {
                match derived::booking_ion(base) {
                    Some(_) => new_ions.push((base.to_string(), total)),
                    None => unnameable.push((base.to_string(), total)),
                }
                continue;
            }
            for (column, molality) in split {
                let moles = total * molality / sum;
                if moles <= TRACE {
                    continue;
                }
                // A specific oxidation state may have no name of its own
                // while the element does: carbon(IV)'s master is CO3(2−),
                // which this registry does not carry because it names
                // dissolved carbonate as bicarbonate at teaching pH. The
                // fallback is guarded — only for a state that *is*
                // essentially the whole element — so a trace of manganese
                // (VI) can never be quietly relabelled as manganese(II).
                // Without it, every carbonate solution lost all of its
                // carbon.
                let dominant = molality / sum > 0.99;
                let named = derived::booking_ion(column).is_some();
                if named {
                    new_ions.push((column.clone(), moles));
                } else if dominant && derived::booking_ion(base).is_some() {
                    new_ions.push((base.to_string(), moles));
                } else {
                    unnameable.push((column.clone(), moles));
                }
            }
        }
        // How each protonation-split state divides between the registry
        // species that carry it, from the molalities the solve returned.
        // Empty is the honest default and the safe one: a state with no
        // columns (pitzer knows no nitrogen) falls through to its single
        // booking ion exactly as before.
        let mut protonation: BTreeMap<String, Vec<(&'static str, f64)>> = BTreeMap::new();
        for (el, _) in &new_ions {
            let Some(split) = derived::protonation_split(el) else {
                continue;
            };
            let weights: Vec<(&'static str, f64)> = split
                .iter()
                .filter_map(|(species, key)| {
                    let column = format!("m_{species}");
                    value(&column)
                        .map(|m| checked_component_readback(&column, m).map(|m| (*key, m)))
                })
                .collect::<Result<_, _>>()?;
            // All or nothing. A partial split would divide the total by a
            // denominator missing one of its terms, which is not a
            // rounding error — it is the whole of the missing species
            // silently reassigned to the one that was found.
            if weights.len() != split.len() {
                continue;
            }
            let sum: f64 = weights.iter().map(|(_, m)| m).sum();
            checked_component_readback(el, sum)?;
            if sum <= 0.0 {
                continue;
            }
            protonation.insert(
                el.clone(),
                weights.into_iter().map(|(k, m)| (k, m / sum)).collect(),
            );
        }
        Ok((
            solvent_kgw_out,
            new_surfaces,
            new_exchanges,
            new_solid_solutions,
            new_ions,
            unnameable,
            protonation,
        ))
    }

    #[allow(clippy::type_complexity, clippy::too_many_arguments)]
    fn apply_balance_corrections(
        vessel: &Vessel,
        problem: &Problem,
        rows: &[Vec<String>],
        new_ions: &mut Vec<(String, f64)>,
        new_surfaces: &mut [SurfaceSites],
        new_exchanges: &[ExchangeSites],
        new_solid_solutions: &[SolidSolution],
        value: &dyn Fn(&str) -> Option<f64>,
        owner: ComponentCertificate,
        reservoir_requested: bool,
    ) -> Result<
        (
            Vec<(String, f64)>,
            Vec<(String, String, f64)>,
            f64,
            f64,
            bool,
        ),
        SolveError,
    > {
        // PHREEQC selected totals are not consistent about whether a
        // SURFACE contribution is included: the first zinc solve reports
        // dissolved Zn, while a repeated solve can report the full
        // analytical Zn total alongside non-zero Hfo_*OZn columns. The
        // material balance is unambiguous. Bound typed inventory is owned by
        // the interface, so aqueous readback cannot exceed analytical input
        // minus that bound amount. `min` preserves any still-lower dissolved
        // result caused by precipitation.
        for sorbate in [SurfaceSorbate::Zinc, SurfaceSorbate::Sulfate] {
            let Some(DerivedRole::Dissolves(elements)) = derived::role(&sorbate.species().0) else {
                continue;
            };
            let mut bound: f64 = new_surfaces
                .iter()
                .map(|surface| surface.bound(sorbate).0)
                .sum();
            if bound <= 0.0 {
                continue;
            }
            for (element, coefficient) in elements {
                let base = element.split('(').next().unwrap_or(element);
                // Analytical inventory can enter either as a SOLUTION
                // total or as an amount-limited EQUILIBRIUM_PHASE.  The
                // latter is the normal first pass for a registry solid such
                // as ZnSO4; looking only at `problem.totals` therefore made
                // its ceiling zero and discarded all dissolved zinc.
                let solution_inventory: f64 = problem
                    .totals
                    .iter()
                    .filter(|(candidate, _)| {
                        candidate.split('(').next().unwrap_or(candidate) == base
                    })
                    .map(|(_, moles)| moles)
                    .sum();
                let phase_inventory: f64 = problem
                    .phases
                    .iter()
                    .filter_map(|(phase, moles, _)| {
                        let derived = derived::phase_by_name(phase)?;
                        let in_phase: f64 = derived
                            .elements
                            .iter()
                            .filter(|(candidate, _)| {
                                candidate.split('(').next().unwrap_or(candidate) == base
                            })
                            .map(|(_, phase_coefficient)| phase_coefficient)
                            .sum();
                        Some(moles * in_phase)
                    })
                    .sum();
                let analytical = solution_inventory + phase_inventory;
                // Selected surface-species molalities can exceed the
                // analytical sorbate available at low loading. Capping only
                // the aqueous side still creates matter because the typed
                // interface owns that impossible excess. Scale this
                // sorbate's occupancy to the hard material ceiling first;
                // sulfate ligand exchange carries its released-water ledger
                // with the same complexes.
                let maximum_bound = (analytical / coefficient).max(0.0);
                if bound > maximum_bound && bound > 0.0 {
                    let scale = maximum_bound / bound;
                    for surface in new_surfaces.iter_mut() {
                        for entry in &mut surface.occupancy {
                            if entry.sorbate == sorbate {
                                entry.moles = Moles(entry.moles.0 * scale);
                            }
                        }
                        if sorbate == SurfaceSorbate::Sulfate {
                            surface.water_release = Moles(surface.water_release.0 * scale);
                        }
                    }
                    bound = maximum_bound;
                }
                let ceiling = (analytical - bound * coefficient).max(0.0);
                // Redox-active elements may be returned as several tagged
                // entries. Cap their aggregate, not every entry separately,
                // or each oxidation state could independently retain the
                // full ceiling.
                let aqueous: f64 = new_ions
                    .iter()
                    .filter(|(candidate, _)| {
                        candidate.split('(').next().unwrap_or(candidate) == base
                    })
                    .map(|(_, moles)| moles)
                    .sum();
                if aqueous > ceiling && aqueous > 0.0 {
                    let scale = ceiling / aqueous;
                    for (candidate, moles) in new_ions.iter_mut() {
                        if candidate.split('(').next().unwrap_or(candidate) == base {
                            *moles *= scale;
                        }
                    }
                }
            }
        }
        // Some selected-output paths include exchange-bound cations in an
        // analytical total while others report solution only. The typed
        // exchanger owns its complexes, so cap the aqueous ledger to the
        // incoming solution-plus-exchanger inventory minus the new bound
        // amount. This makes both representations converge to one material
        // balance and prevents a no-op stir from duplicating cations.
        if !problem.exchanges.is_empty() {
            for ion in [
                ExchangeIon::Sodium,
                ExchangeIon::Calcium,
                ExchangeIon::Magnesium,
            ] {
                let Some(DerivedRole::Dissolves(elements)) = derived::role(&ion.species().0) else {
                    continue;
                };
                let initial_bound: f64 = problem
                    .exchanges
                    .iter()
                    .map(|exchange| exchange.bound(ion).0)
                    .sum();
                let final_bound: f64 = new_exchanges
                    .iter()
                    .map(|exchange| exchange.bound(ion).0)
                    .sum();
                for (element, coefficient) in elements {
                    let base = element.split('(').next().unwrap_or(element);
                    let solution_inventory: f64 = problem
                        .totals
                        .iter()
                        .filter(|(candidate, _)| {
                            candidate.split('(').next().unwrap_or(candidate) == base
                        })
                        .map(|(_, moles)| moles)
                        .sum();
                    let phase_inventory: f64 = problem
                        .phases
                        .iter()
                        .filter_map(|(phase, moles, _)| {
                            let derived = derived::phase_by_name(phase)?;
                            let in_phase: f64 = derived
                                .elements
                                .iter()
                                .filter(|(candidate, _)| {
                                    candidate.split('(').next().unwrap_or(candidate) == base
                                })
                                .map(|(_, phase_coefficient)| phase_coefficient)
                                .sum();
                            Some(moles * in_phase)
                        })
                        .sum();
                    let ceiling =
                        (solution_inventory + initial_bound * coefficient + phase_inventory
                            - final_bound * coefficient)
                            .max(0.0);
                    let aqueous: f64 = new_ions
                        .iter()
                        .filter(|(candidate, _)| {
                            candidate.split('(').next().unwrap_or(candidate) == base
                        })
                        .map(|(_, moles)| moles)
                        .sum();
                    if aqueous > ceiling && aqueous > 0.0 {
                        let scale = ceiling / aqueous;
                        for (candidate, moles) in new_ions.iter_mut() {
                            if candidate.split('(').next().unwrap_or(candidate) == base {
                                *moles *= scale;
                            }
                        }
                    }
                }
            }
        }
        if !problem.solid_solutions.is_empty() {
            *new_ions = checked_crystal_reconciliation(
                vessel,
                problem,
                rows,
                new_ions,
                new_solid_solutions,
                value,
            )?;
        }
        let mut new_phases: Vec<(String, f64)> = Vec::new();
        for (phase, ..) in &problem.phases {
            let moles = value(phase).ok_or_else(|| missing(rows, phase))?;
            new_phases.push((phase.clone(), moles));
        }
        let mut new_gases: Vec<(String, String, f64)> = Vec::new();
        for (phase, species, _) in &problem.gases {
            let column = format!("g_{phase}");
            let moles = value(&column).ok_or_else(|| missing(rows, &column))?;
            if !moles.is_finite() || moles < 0.0 {
                return Err(component_projection_error(
                    ProjectionReason::GasColumnInvalid,
                    format!("invalid raw finite gas column {column}: {moles}"),
                ));
            }
            new_gases.push((phase.clone(), species.clone(), moles));
        }

        // In a homogeneous closed solution, analytical element totals are
        // conserved coordinates, not solver observations. Reusing PHREEQC's
        // rounded readback as the next step's input introduced a few parts in
        // 1e9 of artificial charge imbalance. Near neutral pH that is enough
        // to remember whether acid or base arrived first. Project only tiny
        // readback residuals back onto the exact input totals; any larger
        // discrepancy remains visible, and phase/interface/gas ownership uses
        // the explicit balance paths above instead of this simple projection.
        let homogeneous = problem.surfaces.is_empty()
            && problem.exchanges.is_empty()
            && problem.solid_solutions.is_empty()
            && problem.gases.is_empty()
            && problem.external_gases.is_empty()
            && new_phases.iter().all(|(_, amount)| *amount <= TRACE);
        if homogeneous {
            let mut targets = BTreeMap::<String, f64>::new();
            for (element, amount) in &problem.totals {
                *targets
                    .entry(element.split('(').next().unwrap_or(element).to_string())
                    .or_default() += amount;
            }
            for (element, target) in targets {
                let current: f64 = new_ions
                    .iter()
                    .filter(|(candidate, _)| {
                        candidate.split('(').next().unwrap_or(candidate) == element
                    })
                    .map(|(_, amount)| amount)
                    .sum();
                let tolerance = 1e-14_f64.max(target.abs() * 1e-7);
                if current > 0.0 && (current - target).abs() <= tolerance {
                    let scale = target / current;
                    for (candidate, amount) in new_ions.iter_mut() {
                        if candidate.split('(').next().unwrap_or(candidate) == element {
                            *amount *= scale;
                        }
                    }
                }
            }
        }
        let gas_components_certified = project_gas_aware_components(
            vessel,
            problem,
            new_ions,
            &mut new_phases,
            &new_gases,
            owner,
            reservoir_requested,
        )?;
        let ph = value("pH").ok_or_else(|| missing(rows, "pH"))?;
        let mu = value("mu").ok_or_else(|| missing(rows, "mu"))?;
        Ok((new_phases, new_gases, ph, mu, gas_components_certified))
    }

    #[allow(clippy::too_many_arguments)]
    fn rebuild_contents_and_events(
        vessel: &Vessel,
        problem: &Problem,
        freed_phases: &[(String, f64)],
        solvent_kgw_out: f64,
        new_ions: &[(String, f64)],
        new_phases: &[(String, f64)],
        new_gases: &[(String, String, f64)],
        new_solid_solutions: &[SolidSolution],
        protonation: &BTreeMap<String, Vec<(&'static str, f64)>>,
    ) -> (Vec<Event>, Vec<Portion>) {
        // Rebuild the vessel inventory: water stays; solutes are replaced by
        // the computed state.

        let mut events = Vec::new();
        let mut contents = Vec::new();
        for p in &vessel.contents {
            if p.phase == Phase::Gas
                && problem
                    .external_gases
                    .iter()
                    .any(|exchange| exchange.species == p.species.0)
            {
                // This portion was a finite dose through an external
                // boundary. The equilibrium delta below decides how much
                // entered the condensed inventory and how much vented.
                continue;
            }
            // Held crystals of a curve salt were never handed to the
            // solver (`partition`), so they pass through unchanged.
            if p.phase == Phase::Solid
                && kerotakis_core::crystallisation::holds_solid(vessel, &p.species.0)
            {
                contents.push(p.clone());
                continue;
            }
            match derived::role(&p.species.0) {
                // The solvent is rebuilt on the *equilibrated* water mass,
                // not left at what went in. Speciation consumes and
                // releases water — hydration, hydrolysis — so `mass_H2O`
                // differs from the water poured in, and every dissolved
                // amount below is `molality × mass_H2O`. Keeping the old
                // water beside the new ions made the vessel inconsistent
                // with itself, and the error *compounded*: nitrogen grew
                // 0.25% per step, 0.010000 → 0.010025 → 0.010072, for as
                // long as the beaker was touched.
                Some(DerivedRole::Solvent) if p.phase == Phase::Liquid => contents.push(Portion {
                    species: p.species.clone(),
                    moles: Moles(solvent_kgw_out * 1000.0 / WATER_MOLAR_MASS),
                    phase: Phase::Liquid,
                }),
                // Solid water is pure ice owned by the phase ledger, not
                // PHREEQC's solution solvent. Preserve it byte-for-byte.
                Some(DerivedRole::Solvent) => contents.push(p.clone()),
                // Matter this engine does not model passes through
                // untouched. The rebuild replaces the vessel's contents
                // with the computed state, so anything without a role used
                // to be *destroyed* here — not ignored, deleted. It was
                // invisible only because an unmodelled species also made
                // `partition` decline, so the solver never ran at all.
                None => {
                    if p.phase == Phase::Gas
                        && problem
                            .gases
                            .iter()
                            .any(|(_, species, _)| species == &p.species.0)
                    {
                        continue;
                    }
                    // Freely soluble but unspeciated: it goes into
                    // solution, and that is the whole claim.
                    let dissolves = species::lookup(&p.species).is_some_and(|d| {
                        d.dissolves_without_speciation
                            && d.aqueous_solubility_g_per_100_ml.is_none()
                    });
                    if dissolves && p.phase == Phase::Solid {
                        contents.push(Portion {
                            species: p.species.clone(),
                            moles: p.moles,
                            phase: Phase::Aqueous,
                        });
                    } else {
                        contents.push(p.clone());
                    }
                }
                // A headspace share of a solute this problem never took —
                // the partition step skips gas portions outside its own
                // gas list, so their moles are in neither the totals nor
                // `new_gases`. Ammonia that Henry's law moved into the
                // headspace (`kerotakis_core::volatility`) is the live
                // case; dropping it here destroyed nitrogen.
                Some(_)
                    if p.phase == Phase::Gas
                        && !problem
                            .gases
                            .iter()
                            .any(|(_, species, _)| species == &p.species.0) =>
                {
                    contents.push(p.clone())
                }
                Some(_) => {}
            }
        }
        // Moles of an *element* are not moles of the ion that carries it
        // unless the ion holds exactly one atom of it. Nitrogen booked as
        // N2 counted twice, so a beaker of silver nitrate gained a quarter
        // of a percent of nitrogen every time it was touched — the element
        // total went in as a molecule count and came back out as an atom
        // count.
        let atoms_per_ion = |ion: &str, base: &str| -> f64 {
            species::lookup_key(ion)
                .and_then(|d| kerotakis_core::stoich::parse_formula(d.formula).ok())
                .and_then(|f| f.counts.get(base).copied())
                .filter(|n| *n > 0.0)
                .unwrap_or(1.0)
        };
        for (el, moles) in new_ions {
            if *moles > TRACE {
                let base = el.split('(').next().unwrap_or(el);
                // A state whose registry name is a protonation question is
                // booked as the species the solve found, in the proportions
                // it found them. Reduced nitrogen is the case: one number
                // came back, and ammonia-or-ammonium is not something that
                // number knows. The element total stays authoritative — the
                // fractions only decide how to name it — so this cannot
                // create or destroy nitrogen however the split falls.
                if let Some(split) = protonation.get(el) {
                    for (ion, fraction) in split {
                        let share = *moles * fraction;
                        // The solved element total stays authoritative even
                        // when one canonical protonation share is below TRACE.
                        if !share.is_finite() || share <= 0.0 {
                            continue;
                        }
                        contents.push(Portion {
                            species: SpeciesId::new(ion),
                            moles: Moles(share / atoms_per_ion(ion, base)),
                            phase: Phase::Aqueous,
                        });
                    }
                    continue;
                }
                let ion = derived::booking_ion(el).expect("booking ion covered by tests");
                contents.push(Portion {
                    species: SpeciesId::new(ion),
                    moles: Moles(*moles / atoms_per_ion(ion, base)),
                    phase: Phase::Aqueous,
                });
            }
        }
        for (_, species, moles) in new_gases {
            if *moles > TRACE {
                contents.push(Portion {
                    species: SpeciesId::new(species),
                    moles: Moles(*moles),
                    phase: Phase::Gas,
                });
            }
            let before = vessel
                .contents
                .iter()
                .filter(|portion| portion.phase == Phase::Gas && portion.species.0 == *species)
                .map(|portion| portion.moles.0)
                .sum::<f64>();
            let formed = *moles - before;
            if formed > TRACE {
                events.push(Event::GasContained {
                    vessel: vessel.id,
                    species: SpeciesId::new(species),
                    moles: Moles(formed),
                });
            }
        }
        for (phase, moles) in new_phases {
            if let Some(dp) = derived::phase_by_name(phase) {
                let (species, waters) = (dp.species, dp.waters);
                // Baseline is the phase's INPUT amount, not the vessel's
                // solids: a freely-soluble solid (e.g. KCl) contributes to
                // the totals, not the phase, and comparing against vessel
                // solids double-counted its dissolution (and its heat).
                let before = problem
                    .phases
                    .iter()
                    .find(|(name, ..)| name == phase)
                    .map(|(_, m, _)| *m)
                    .unwrap_or(0.0);
                if *moles > TRACE {
                    contents.push(Portion {
                        species: SpeciesId::new(species),
                        moles: Moles(*moles),
                        phase: Phase::Solid,
                    });
                }
                // Bookkeeping keeps every trace; observation does not.
                let delta = moles - before;
                if delta >= kerotakis_core::OBSERVABLE_MOLES {
                    events.push(Event::Precipitated {
                        vessel: vessel.id,
                        species: SpeciesId::new(species),
                        moles: Moles(delta),
                        dry: false,
                    });
                } else if delta <= -kerotakis_core::OBSERVABLE_MOLES {
                    events.push(Event::Dissolved {
                        vessel: vessel.id,
                        species: SpeciesId::new(species),
                        moles: Moles(-delta),
                    });
                }
                // Waters of crystallisation move between the liquid and the
                // solid (gypsum: 2 H2O per formula unit) — and PHREEQC has
                // already moved them. `mass_H2O` is the water left *after*
                // the crystal took its share, and the solvent is rebuilt
                // from it, so subtracting them again here bound four waters
                // per formula instead of two. The chemistry is unchanged;
                // it is now counted once.
                let _ = waters;
            } else if let Some(exchange) = problem
                .external_gases
                .iter()
                .find(|exchange| exchange.phase == *phase)
            {
                match exchange.kind {
                    ExternalGasKind::Reservoir => {
                        let transferred = *moles - exchange.initial_moles;
                        if transferred > TRACE {
                            events.push(Event::GasEvolved {
                                vessel: vessel.id,
                                species: SpeciesId::new(&exchange.species),
                                moles: Moles(transferred),
                            });
                        } else if transferred < -TRACE {
                            events.push(Event::GasAbsorbed {
                                vessel: vessel.id,
                                species: SpeciesId::new(&exchange.species),
                                moles: Moles(-transferred),
                            });
                        }
                    }
                    ExternalGasKind::Dose => {
                        events.push(Event::not_modeled(
                            vessel.id,
                            kerotakis_core::ops::NotModelledCause::RateNotModelled,
                            Phrase::new(
                                "not-modeled.finite-gas-dose",
                                "{species} finite gas dose: uptake is an instantaneous equilibrium calculation, not a mass-transfer rate. After the dose, only explicitly configured atmospheric species exchange with an external reservoir; no new reservoir is inferred from the dose. Waiting time does not parameterise degassing",
                                vec![(
                                    "species".to_string(),
                                    Slot::term("species", exchange.species.clone()),
                                )],
                            ),
                        ));
                        let absorbed = exchange.initial_moles - moles;
                        if absorbed > TRACE {
                            events.push(Event::GasAbsorbed {
                                vessel: vessel.id,
                                species: SpeciesId::new(&exchange.species),
                                moles: Moles(absorbed),
                            });
                        }
                        // Whatever remains from the finite dose is outside
                        // the vessel after it has bubbled through. If the
                        // solution produced additional gas, it is included
                        // in this same outward amount.
                        if *moles > TRACE {
                            events.push(Event::GasEvolved {
                                vessel: vessel.id,
                                species: SpeciesId::new(&exchange.species),
                                moles: Moles(*moles),
                            });
                        }
                    }
                }
            }
        }
        for solid_solution in new_solid_solutions {
            let before = problem
                .solid_solutions
                .iter()
                .find(|candidate| candidate.label == solid_solution.label);
            for amount in &solid_solution.components {
                let initial = before
                    .map(|candidate| candidate.moles_of(amount.component).0)
                    .unwrap_or(0.0);
                let delta = amount.moles.0 - initial;
                if delta >= kerotakis_core::OBSERVABLE_MOLES {
                    events.push(Event::Precipitated {
                        vessel: vessel.id,
                        species: amount.component.species(),
                        moles: Moles(delta),
                        dry: false,
                    });
                } else if delta <= -kerotakis_core::OBSERVABLE_MOLES {
                    events.push(Event::Dissolved {
                        vessel: vessel.id,
                        species: amount.component.species(),
                        moles: Moles(-delta),
                    });
                }
            }
        }
        // A phase the routed database does not define was dissolved into
        // element totals above. The chemistry was right either way, but the
        // *event* was never recorded — and the dissolution enthalpy rides on
        // the event, so the vessel silently skipped the heat whenever the
        // router happened to pick a database lacking that mineral.
        //
        // Sylvite is pitzer-only, and the router chooses by ionic strength,
        // so potassium chloride cooled the beaker only once something else
        // had already made the solution concentrated enough to route there.
        // Dissolving KCl and then NaCl ended 0.82 K warmer than the same two
        // salts in the other order: enthalpy had stopped being a state
        // function. That temperature gap was also the whole of the
        // order-dependent pH we chased for two sessions — equalising the
        // temperature drops the pH difference from 1.36e-2 to 2.8e-10.
        //
        // Recorded whatever the size, as with the escaping gas above:
        // `Event::is_observable` decides what is shown, but the energy
        // balance must see all of it.
        for (phase, moles) in freed_phases {
            if *moles > TRACE {
                if let Some(dp) = derived::phase_by_name(phase) {
                    events.push(Event::Dissolved {
                        vessel: vessel.id,
                        species: SpeciesId::new(dp.species),
                        moles: Moles(*moles),
                    });
                }
            }
        }
        // Freely-soluble solids (no mineral phase) dissolved entirely.
        for p in &vessel.contents {
            if p.phase == Phase::Solid
                && matches!(derived::role(&p.species.0), Some(DerivedRole::Dissolves(_)))
                && !kerotakis_core::crystallisation::holds_solid(vessel, &p.species.0)
            {
                events.push(Event::Dissolved {
                    vessel: vessel.id,
                    species: p.species.clone(),
                    moles: p.moles,
                });
            }
        }
        (events, contents)
    }

    #[allow(clippy::too_many_arguments)]
    fn finalize_solution_info(
        vessel: &mut Vessel,
        problem: &Problem,
        db_tag: &str,
        routing: Phrase,
        solvent_activity: Option<kerotakis_core::vessel::SolventActivityProvenance>,
        speciation: Vec<SpeciesDetail>,
        saturation: &[(String, f64)],
        coupling_failed: Option<String>,
        pe_determined: bool,
        pe_authority: PeAuthority,
        redox_constrained: bool,
        value: &dyn Fn(&str) -> Option<f64>,
        unnameable: &[(String, f64)],
        ph: f64,
        mu: f64,
        events: &mut Vec<Event>,
    ) {
        let idx = derived::index_for(db_tag);
        let mut redox = redox_distribution(problem, db_tag, value);
        // Say what the input does to redox, because it is load-bearing and
        // otherwise invisible. Naming an oxidation state in a PHREEQC
        // solution *decouples* that element: it gets its own mass balance
        // and exchanges electrons with nothing. That is how this bench can
        // show permanganate and iron(II) sitting in one beaker — a solution
        // that cannot exist. Each state is right; their coexistence is not,
        // and the difference has to be visible rather than inferred.
        let redox_note = match (&coupling_failed, redox.len()) {
            (Some(why), _) => Some(Phrase::new(
                "routing.redox-not-coupled",
                "the redox elements here could not be coupled, so each is shown in the oxidation state it was added in and they have not reacted with each other — {why}",
                // The solver's own diagnostic travels in a text slot, as
                // it does in the event one screen above: the frame around
                // it is what a catalogue translates.
                vec![("why".to_string(), Slot::text(why.clone()))],
            )),
            (None, n) if n > 1 && redox_coupling(problem, db_tag).is_none() => {
                // Coupled elements are settled by the electron balance;
                // this note is for the ones deliberately left pinned.
                Some(Phrase::bare(
                    "routing.slow-couples-held-as-added",
                    "some elements here keep the oxidation state they were added in: only the couples that equilibrate on a bench timescale exchange electrons, and the slow ones — sulfate, nitrate, carbonate — are held as added",
                ))
            }
            _ => None,
        };
        // Say it in the stream, not only in `explain`.
        //
        // A beaker whose coupling stood down shows its elements in the
        // states they were added in — permanganate still purple, iron still
        // iron(II) — which is a perfectly ordinary-looking answer. The
        // reason it is not one lives in the routing, and a reader has to
        // think to ask for `explain` to find it. Whoever is most likely to
        // be misled is exactly whoever does not know to ask.
        //
        // The state itself was already honest; the discoverability was
        // asymmetric. Same precedent as a flame held to ethanol reporting
        // that no solver looked, rather than reporting that nothing
        // happened.
        if let Some(why) = &coupling_failed {
            events.push(Event::not_modeled(
                vessel.id,
                kerotakis_core::ops::NotModelledCause::NoSolver,
                Phrase::new(
                    "not-modeled.redox-coupling-stood-down",
                    "these elements have not reacted with each other — they are shown in \
                     the oxidation states they were added in, which is not what the beaker \
                     would do: {why}",
                    // The solver's own diagnostic, not a sentence written
                    // for a learner: it travels in a text slot and the
                    // frame around it is what a catalogue translates.
                    vec![("why".to_string(), Slot::text(why.clone()))],
                ),
            ));
        }

        redox.sort_by(|a, b| {
            a.element
                .cmp(&b.element)
                .then(b.molality.total_cmp(&a.molality))
        });

        events.extend(reference_complex_boundary(vessel, &speciation));
        events.extend(silver_ammine_inventory_boundary(vessel, &speciation));
        events.extend(reactive_gas_boundary(vessel, problem));
        // A pe nothing constrains is not a small pe, it is a number the
        // solver happened to leave behind.
        //
        // `redox_constrained` asks whether any element in the problem COULD
        // carry more than one oxidation state in this dataset, and carbon
        // can — so bicarbonate and hydrochloric acid satisfied it, and the
        // contract published a pe for a beaker that holds no couple at all.
        // Measured on `th-100` (water, NaHCO3, HCl, sealed): swapping the
        // two reagents leaves pH agreeing to four decimals, ionic strength
        // to six and the gauge to a part in a million, and moves pe from
        // 12.780243 to -0.055944 — about 760 mV of nothing.
        //
        // A redox row alone is only a reported valence inventory: the
        // sealed ammonia case has one N(-3) row while native pe wanders
        // between otherwise identical physical states. The input/solve
        // branch must also witness an electron potential. FeCl3 in an open
        // vessel retains its actually posed atmospheric O2 boundary; a
        // paid, bracketed Fe/Mn electron balance has its own authority.
        // An imported cache's old default boolean cannot create either.
        let pe_constrained =
            redox_constrained && pe_determined && pe_authority.publishes_pe() && !redox.is_empty();
        // The whole routing line, composed before the record is built.
        //
        // Three clauses, each of them optional, each of them a sentence in
        // its own right: why this dataset, what redox did, and whether a
        // second dataset was asked for the solvent's activity. They used
        // to be a `format!` and two `push_str`s, which is a finished
        // English paragraph on a field a reader reads.
        let routing = match redox_note {
            None => routing,
            Some(note) => Phrase::new(
                "routing.with-redox-note",
                "{routing}. {note}",
                vec![
                    ("routing".to_string(), Slot::phrase(routing)),
                    ("note".to_string(), Slot::phrase(note)),
                ],
            ),
        };
        // Said in prose as well as in the field, because `routing` is what
        // is rendered beside the numbers and "two datasets answered this
        // beaker" is exactly the kind of thing a reader should not have to
        // infer.
        let routing = secondary_routing(routing, solvent_activity.as_ref());
        let provenance = Provenance::new(
            "PHREEQC (IPhreeqc, USGS)",
            dataset_phrase(db_tag),
            idx.activity_model.phrase(),
            dataset_sources(db_tag),
            routing,
        );
        // Which dataset answers this beaker, said once, when it changes.
        //
        // The provenance a learner would most want was the one the web
        // could not reach: the drawer reads `event.provenance`, and until
        // `Event::SolutionRouted` the only event carrying one was the
        // combustion `ThermalEquilibrium`. This is the aqueous half, on
        // the wire.
        //
        // **On CHANGE, and that is the whole of the design.** This
        // function runs far more often than a reader wants a line — three
        // commands of `aq-023` reach it five times, because the thermal
        // fixed point above re-solves a clone of the vessel until the
        // temperature settles. An event per characterisation would bury
        // the log and tell nobody anything. The news is never *the solver
        // ran*; it is *the answer to this beaker now comes from somewhere
        // else* — the ionic strength went past where the default dataset
        // is reliable and the Pitzer file took over, or a second dataset
        // was asked for the solvent's activity, or the redox couples
        // stood down. Those are the moments, and between them the
        // sentence stands and says nothing.
        //
        // The rule itself now lives in `announce_routing`, because two
        // other paths write a provenance and had to obey it: see there for
        // what is compared and why it is held on the vessel.
        announce_routing(vessel, &provenance, events);
        let info = SolutionInfo {
            scope: Default::default(),
            solvent_kg: value("mass_H2O"),
            particle_molality: value("particle_molality"),
            redox,
            pe: pe_constrained.then(|| value("pe")).flatten(),
            ph,
            ionic_strength: mu,
            species: speciation,
            provenance: Some(provenance),
            solvent_activity,
        };
        let changed = vessel
            .solution
            .as_ref()
            .map(|prev| (prev.ph - ph).abs() > 0.01 || (prev.ionic_strength - mu).abs() > 1e-4)
            .unwrap_or(true);
        vessel.solution = Some(info);
        if changed {
            events.push(Event::SolutionCharacterized {
                vessel: vessel.id,
                ph,
                ionic_strength: mu,
            });
        }

        // Matter the readback could not name. It is a small fraction of a
        // minor oxidation state, and losing it silently is exactly the kind
        // of quiet subtraction this engine keeps having to root out.
        for (column, moles) in unnameable {
            events.push(Event::not_modeled(
                vessel.id,
                kerotakis_core::ops::NotModelledCause::PhaseNotInRegistry,
                Phrase::new(
                    "not-modeled.unnameable-oxidation-state",
                    "{moles} mol settled as {column}, an oxidation state this lab has no name for — it is not in the vessel's inventory, so there is slightly less of that element in the glass than went in",
                    vec![
                        ("moles".to_string(), Slot::number(format!("{moles:.3e}"))),
                        // A PHREEQC valence column — `Mn(7)` — is
                        // notation, and the reader traces it back to a
                        // database rather than reading it as a word.
                        ("column".to_string(), Slot::text(column.clone())),
                    ],
                ),
            ));
        }

        let offered: Vec<&str> = problem.phases.iter().map(|(p, ..)| p.as_str()).collect();
        events.extend(crate::phase_diagnostics::events(
            vessel.id,
            vessel.temperature.0,
            db_tag,
            saturation,
            &offered,
            SUPERSATURATION_REPORTING_SI,
        ));
    }
}

#[derive(Clone, Copy)]
enum ProjectionReason {
    GasColumnInvalid,
    TotalInvalid,
    ProtonationSplitMissing,
    ProtonationSplitInvalid,
    ComponentInvalid,
    GasReadbackInvalid,
    PhaseReadbackInvalid,
    PhaseReadbackMissing,
    PhaseAmountInvalid,
    MineralAmountNegative,
    GasReadbackMissing,
    GasAmountInvalid,
    InitialGasInvalid,
    ReservoirEndpointResidual,
    ComponentTargetInvalid,
    ComponentBoundExceeded,
}

impl ProjectionReason {
    fn phrase(self) -> Phrase {
        let (key, en) = match self {
            Self::GasColumnInvalid => (
                "solver-reason.aqueous.projection-gas-column-invalid",
                "A native gas column contains an invalid amount.",
            ),
            Self::TotalInvalid => (
                "solver-reason.aqueous.projection-total-invalid",
                "A native analytical component contains an invalid amount.",
            ),
            Self::ProtonationSplitMissing => (
                "solver-reason.aqueous.projection-protonation-split-missing",
                "A required native protonation split is missing.",
            ),
            Self::ProtonationSplitInvalid => (
                "solver-reason.aqueous.projection-protonation-split-invalid",
                "The native protonation split is incomplete or invalid.",
            ),
            Self::ComponentInvalid => (
                "solver-reason.aqueous.projection-component-invalid",
                "An aqueous component amount is invalid.",
            ),
            Self::GasReadbackInvalid => (
                "solver-reason.aqueous.projection-gas-readback-invalid",
                "A finite gas readback is invalid.",
            ),
            Self::PhaseReadbackInvalid => (
                "solver-reason.aqueous.projection-phase-readback-invalid",
                "A phase readback is invalid.",
            ),
            Self::PhaseReadbackMissing => (
                "solver-reason.aqueous.projection-phase-readback-missing",
                "A required phase readback is missing.",
            ),
            Self::PhaseAmountInvalid => (
                "solver-reason.aqueous.projection-phase-amount-invalid",
                "An initial or returned phase amount is invalid.",
            ),
            Self::MineralAmountNegative => (
                "solver-reason.aqueous.projection-mineral-amount-negative",
                "A returned mineral amount is negative.",
            ),
            Self::GasReadbackMissing => (
                "solver-reason.aqueous.projection-gas-readback-missing",
                "A required finite gas readback is missing.",
            ),
            Self::GasAmountInvalid => (
                "solver-reason.aqueous.projection-gas-amount-invalid",
                "A returned finite gas amount is invalid.",
            ),
            Self::InitialGasInvalid => (
                "solver-reason.aqueous.projection-initial-gas-invalid",
                "An initial gas amount is invalid.",
            ),
            Self::ReservoirEndpointResidual => (
                "solver-reason.aqueous.projection-reservoir-endpoint-invalid",
                "An exhausted reservoir endpoint has a non-finite or out-of-bound residual.",
            ),
            Self::ComponentTargetInvalid => (
                "solver-reason.aqueous.projection-component-target-invalid",
                "A component target or readback amount is invalid.",
            ),
            Self::ComponentBoundExceeded => (
                "solver-reason.aqueous.projection-component-readback-rejected",
                "The component readback failed the existing acceptance checks.",
            ),
        };
        Phrase::bare(key, en)
    }
}

fn component_projection_error(reason: ProjectionReason, detail: impl Into<String>) -> SolveError {
    SolveError::NotConverged {
        reason: Some(Box::new(reason.phrase())),
        solver: "phreeqc-inventory".into(),
        detail: format!("gas-aware analytical component balance: {}", detail.into()),
    }
}

fn checked_component_readback(column: &str, raw: f64) -> Result<f64, SolveError> {
    if !raw.is_finite() || raw < 0.0 {
        return Err(component_projection_error(
            ProjectionReason::TotalInvalid,
            format!("invalid raw analytical component {column}: {raw}"),
        ));
    }
    Ok(raw)
}

fn certify_projected_protonation(
    ions: &[(String, f64)],
    protonation: &BTreeMap<String, Vec<(&'static str, f64)>>,
) -> Result<(), SolveError> {
    for (key, amount) in ions {
        let Some(split) = derived::protonation_split(key) else {
            continue;
        };
        if *amount == 0.0 {
            continue;
        }
        let fractions = protonation.get(key).ok_or_else(|| {
            component_projection_error(
                ProjectionReason::ProtonationSplitMissing,
                format!("missing native protonation split for {key}"),
            )
        })?;
        // Several native species can book to the same registry species:
        // CO2 and H2CO3 both retain their separate shares under CO2(aq).
        // Certify the declared multiplicity, not a one-to-one name mapping.
        let mut expected_keys = BTreeMap::<&str, usize>::new();
        for (_, species) in split {
            *expected_keys.entry(species).or_default() += 1;
        }
        let mut actual_keys = BTreeMap::<&str, usize>::new();
        for (species, _) in fractions {
            *actual_keys.entry(species).or_default() += 1;
        }
        if fractions.len() != split.len()
            || actual_keys != expected_keys
            || fractions
                .iter()
                .any(|(_, fraction)| !fraction.is_finite() || *fraction < 0.0)
            || (fractions.iter().map(|(_, fraction)| fraction).sum::<f64>() - 1.0).abs() > 1e-12
        {
            return Err(component_projection_error(
                ProjectionReason::ProtonationSplitInvalid,
                format!("incomplete native protonation split for {key}"),
            ));
        }
    }
    Ok(())
}

/// Add an actual gas transfer to the dissolved analytical components. The
/// native element key is selected by a unique base, never by replacing a
/// tagged oxidation state with its untagged neighbour.
fn add_gas_component_transfer(
    targets: &mut BTreeMap<String, f64>,
    unique_keys: &BTreeMap<String, String>,
    phase: &str,
    dissolved_delta: f64,
) -> bool {
    let Some(formula) = crate::dbindex::parse_formula(phase.trim_end_matches("(g)")) else {
        return false;
    };
    for (element, count) in formula {
        if element == "H" || element == "O" {
            continue; // The H/O basis and gas-boundary certificate own these.
        }
        let Some(key) = unique_keys.get(&element) else {
            return false;
        };
        *targets.entry(key.clone()).or_default() += dissolved_delta * count;
    }
    true
}

/// A reservoir phase can certify an exhausted aqueous endpoint when its
/// independently bounded amount closes the tiny analytical residual. The
/// correction belongs to the gas transfer, never to an invented aqueous ion.
fn exhausted_reservoir_endpoint_correction(
    vessel: &Vessel,
    problem: &Problem,
    new_ions: &[(String, f64)],
    new_phases: &[(String, f64)],
) -> Option<(usize, String, f64)> {
    if problem.external_gases.len() != 1
        || !problem.gases.is_empty()
        || problem.phases.len() != 1
        || new_phases.len() != 1
        || problem.clock_co2_applied_mol != 0.0
        || vessel.owns_headspace_gas()
        || vessel
            .contents
            .iter()
            .any(|portion| portion.phase == Phase::Gas)
        || !problem.surfaces.is_empty()
        || !problem.exchanges.is_empty()
        || !problem.solid_solutions.is_empty()
        || !vessel.unresolved_materials.is_empty()
        || !vessel.material_objects.is_empty()
        || !vessel.adsorbed.is_empty()
        || !vessel.electrodes.is_empty()
    {
        return None;
    }
    let gas = &problem.external_gases[0];
    if gas.kind != ExternalGasKind::Reservoir
        || !ATMOSPHERIC
            .iter()
            .any(|(phase, species, _)| *phase == gas.phase && *species == gas.species)
        || gas.phase != format!("{}(g)", gas.species)
        || problem.phases[0].0 != gas.phase
        || new_phases[0].0 != gas.phase
    {
        return None;
    }
    let registry = species::lookup_key(&gas.species)?;
    let formula = kerotakis_core::stoich::parse_formula(registry.formula).ok()?;
    let phase_formula = crate::dbindex::parse_formula(gas.phase.trim_end_matches("(g)"))?;
    if formula.charge != 0.0 || formula.counts != phase_formula {
        return None;
    }
    let mut physical_elements = formula.counts.iter().filter(|(element, count)| {
        element.as_str() != "H" && element.as_str() != "O" && **count > 0.0
    });
    let (element, coefficient) = physical_elements.next()?;
    if physical_elements.next().is_some() || !coefficient.is_finite() || *coefficient <= 0.0 {
        return None;
    }
    let totals: Vec<_> = problem
        .totals
        .iter()
        .filter(|(key, _)| key != "H" && key != "O")
        .collect();
    if totals.len() != 1 || totals[0].0.as_str() != element.as_str() {
        return None;
    }
    // Tagged or duplicate native rows would make the endpoint's component
    // owner ambiguous even if they happen to share the same base element.
    if new_ions
        .iter()
        .any(|(key, _)| key.split('(').next().unwrap_or(key) == element && key != element)
    {
        return None;
    }
    let current: f64 = new_ions
        .iter()
        .filter(|(key, _)| key == element)
        .map(|(_, amount)| *amount)
        .sum();
    let input_total = totals[0].1;
    let coefficient = *coefficient;
    let target_before = input_total + (gas.initial_moles - new_phases[0].1) * coefficient;
    let initial = gas.initial_moles;
    let final_amount = new_phases[0].1;
    if !initial.is_finite()
        || initial < 0.0
        || !final_amount.is_finite()
        || final_amount <= initial
        || !current.is_finite()
        || current != 0.0
        || !target_before.is_finite()
        || target_before <= 0.0
    {
        return None;
    }
    let correction = target_before / coefficient;
    let gas_readback_bound = 1e-14_f64.max(initial.abs().max(final_amount.abs()) * 1e-7);
    if !correction.is_finite() || correction <= 0.0 || correction > gas_readback_bound {
        return None;
    }
    for count in formula.counts.values() {
        let moved = correction * count;
        if !moved.is_finite() || moved.abs() > 1e-10 {
            return None;
        }
    }
    let moved_charge = correction * formula.charge;
    if !moved_charge.is_finite() || moved_charge.abs() > 1e-10 {
        return None;
    }
    let corrected_amount = final_amount + correction;
    let residual = input_total + (initial - corrected_amount) * coefficient;
    if !corrected_amount.is_finite()
        || corrected_amount < initial
        || !residual.is_finite()
        || residual.abs() > 1e-14
    {
        return None;
    }
    Some((0, element.clone(), corrected_amount))
}

/// An open vessel may pose a known aqueous subproblem beside conserved opaque
/// recipe matter. Only the reviewed atmospheric reservoir may exchange gas;
/// the existing signed gas ledger owns each crossing. Pigment and unresolved
/// recipe matter remain exact immutable spectators, not acid/base chemistry.
fn reservoir_opaque_component_owner(
    vessel: &Vessel,
    problem: &Problem,
    owner: ComponentCertificate,
) -> bool {
    matches!(
        owner,
        ComponentCertificate::Uncertified | ComponentCertificate::ReservoirWithOpaque
    ) && problem.clock_co2_applied_mol == 0.0
        && !problem.solvent_only
        && !vessel.unresolved_materials.is_empty()
        && vessel.pending_co2_transfer_mol == 0.0
        && matches!(&vessel.headspace, Headspace::Open)
        && !vessel.owns_headspace_gas()
        && !vessel
            .contents
            .iter()
            .any(|portion| portion.phase == Phase::Gas)
        && problem.gases.is_empty()
        && !problem.external_gases.is_empty()
        && problem.external_gases.iter().all(|gas| {
            gas.kind == ExternalGasKind::Reservoir
                && gas.initial_moles.is_finite()
                && gas.initial_moles == 0.0
                && ATMOSPHERIC.iter().any(|(phase, species, target)| {
                    gas.phase == *phase
                        && gas.species == *species
                        && problem
                            .phases
                            .iter()
                            .any(|(candidate, initial, actual_target)| {
                                candidate.as_str() == *phase
                                    && *initial == 0.0
                                    && actual_target.is_finite()
                                    && *actual_target == *target
                            })
                })
        })
        && problem.surfaces.is_empty()
        && problem.exchanges.is_empty()
        && problem.solid_solutions.is_empty()
        && vessel.surfaces.is_empty()
        && vessel.exchanges.is_empty()
        && vessel.solid_solutions.is_empty()
        && vessel.material_objects.is_empty()
        && vessel.adsorbed.is_empty()
        && vessel.electrodes.is_empty()
        && problem.kgw.is_finite()
        && problem.kgw > 0.0
        && !problem.totals.is_empty()
        && problem
            .totals
            .iter()
            .all(|(_, amount)| amount.is_finite() && *amount >= 0.0)
        && problem
            .phases
            .iter()
            .all(|(_, amount, target)| amount.is_finite() && *amount >= 0.0 && target.is_finite())
        && crate::inventory::valid_opaque_clock_stock(vessel)
        && crate::inventory::valid_reservoir_immutable_spectators(vessel)
        && !holds_unspeciated_acid(vessel)
        && !holds_unspeciated_solute(vessel)
}

/// The first reservoir-owned answer must already be a complete *native* represented
/// answer. A missing selected-output column cannot become a certificate just
/// because a solver hook returned `Some(solution)`.
fn complete_reservoir_diagnostics(vessel: &Vessel) -> bool {
    let Some(info) = vessel
        .solution
        .as_ref()
        .filter(|info| info.scope == SolutionScope::Complete)
    else {
        return false;
    };
    info.provenance.as_ref().is_some_and(|route| {
        !route.engine.is_empty() && !route.dataset.is_empty() && !route.model.is_empty()
    }) && info.ph.is_finite()
        && info.ionic_strength.is_finite()
        && info.ionic_strength >= 0.0
        && info
            .solvent_kg
            .is_some_and(|water| water.is_finite() && water > 0.0)
        && info
            .particle_molality
            .is_some_and(|particles| particles.is_finite() && particles >= 0.0)
        && info
            .species
            .iter()
            .find(|species| species.name == "H2O")
            .is_some_and(|water| water.activity.is_finite() && water.activity > 0.0)
        && vessel.free_proton.is_finite()
        && vessel.free_proton >= 0.0
        && vessel.free_hydroxide.is_finite()
        && vessel.free_hydroxide >= 0.0
        && vessel.solute_charge.is_finite()
        && vessel
            .co2_partial_pressure_atm
            .is_some_and(|pressure| pressure.is_finite() && pressure >= 0.0)
}

fn opaque_clock_component_owner(
    vessel: &Vessel,
    problem: &Problem,
    owner: ComponentCertificate,
) -> bool {
    !vessel.unresolved_materials.is_empty()
        && (problem.clock_co2_applied_mol != 0.0 || owner == ComponentCertificate::ClockWithOpaque)
        && crate::inventory::valid_opaque_clock_stock(vessel)
        && crate::inventory::valid_clock_immutable_spectators(vessel)
        && !holds_unspeciated_acid(vessel)
        && !holds_unspeciated_solute(vessel)
        && vessel.contents.iter().all(|portion| {
            let amount = portion.moles.0;
            if !amount.is_finite() || amount < 0.0 {
                return false;
            }
            if amount == 0.0 {
                return true;
            }
            if portion.phase == Phase::Gas {
                return problem
                    .gases
                    .iter()
                    .any(|(_, species, _)| species == &portion.species.0)
                    || (vessel.owns_headspace_gas()
                        && matches!(portion.species.0.as_str(), "N2" | "O2"));
            }
            derived::role(&portion.species.0).is_some()
                || (portion.phase == Phase::Aqueous
                    && species::is_acid_base_basis(&portion.species.0))
                || crate::inventory::clock_spectator(portion)
        })
}

/// Project only numerical native readback residue onto exact, independently
/// owned analytical components. This narrow class has no typed interface or
/// finite external dose; a gas may cross an admitted reservoir or redistribute
/// into a finite headspace. Candidate minerals use their derived component
/// vector. Ambiguous valence bases retain their existing route disposition.
fn project_gas_aware_components(
    vessel: &Vessel,
    problem: &Problem,
    new_ions: &mut [(String, f64)],
    new_phases: &mut [(String, f64)],
    new_gases: &[(String, String, f64)],
    owner: ComponentCertificate,
    reservoir_requested: bool,
) -> Result<bool, SolveError> {
    if problem.external_gases.is_empty()
        && problem.gases.is_empty()
        && problem.clock_co2_applied_mol == 0.0
        && !owner.is_certified()
    {
        return Ok(false);
    }
    if owner == ComponentCertificate::ReservoirWithOpaque
        && !reservoir_opaque_component_owner(vessel, problem, owner)
    {
        return Ok(false);
    }
    if !problem.surfaces.is_empty()
        || !problem.exchanges.is_empty()
        || !problem.solid_solutions.is_empty()
        || (!vessel.unresolved_materials.is_empty()
            && !opaque_clock_component_owner(vessel, problem, owner)
            && !(reservoir_requested && reservoir_opaque_component_owner(vessel, problem, owner)))
        || !vessel.material_objects.is_empty()
        || !vessel.adsorbed.is_empty()
        || !vessel.electrodes.is_empty()
        || problem
            .external_gases
            .iter()
            .any(|gas| gas.kind == ExternalGasKind::Dose)
    {
        return Ok(false);
    }
    if new_ions
        .iter()
        .any(|(_, amount)| !amount.is_finite() || *amount < 0.0)
    {
        return Err(component_projection_error(
            ProjectionReason::ComponentInvalid,
            "invalid individual aqueous component",
        ));
    }
    if new_gases
        .iter()
        .any(|(_, _, amount)| !amount.is_finite() || *amount < 0.0)
    {
        return Err(component_projection_error(
            ProjectionReason::GasReadbackInvalid,
            "invalid finite gas readback",
        ));
    }
    for (phase, amount) in new_phases.iter() {
        if !amount.is_finite()
            || (*amount < 0.0
                && !problem.external_gases.iter().any(|gas| {
                    gas.phase.as_str() == phase.as_str() && gas.kind == ExternalGasKind::Reservoir
                }))
        {
            return Err(component_projection_error(
                ProjectionReason::PhaseReadbackInvalid,
                format!("invalid phase readback {phase}"),
            ));
        }
    }
    let exhausted_endpoint =
        exhausted_reservoir_endpoint_correction(vessel, problem, new_ions, new_phases);
    let mut keys = BTreeSet::<String>::new();
    keys.extend(problem.totals.iter().map(|(key, _)| key.clone()));
    keys.extend(new_ions.iter().map(|(key, _)| key.clone()));
    // `elements` is a readback request and can name the untagged base of a
    // tagged total. Use it only when no exact analytical/output key exists.
    for element in &problem.elements {
        let base = element.split('(').next().unwrap_or(element);
        if !keys
            .iter()
            .any(|key| key.split('(').next().unwrap_or(key) == base)
        {
            keys.insert(element.clone());
        }
    }
    let mut unique_keys = BTreeMap::<String, String>::new();
    for key in &keys {
        let base = key.split('(').next().unwrap_or(key);
        if unique_keys.insert(base.into(), key.clone()).is_some() {
            return Ok(false); // An independent oxidation state needs its own owner.
        }
    }
    let mut targets = BTreeMap::<String, f64>::new();
    for (key, amount) in &problem.totals {
        *targets.entry(key.clone()).or_default() += amount;
    }
    for (phase_index, (phase, initial, _)) in problem.phases.iter().enumerate() {
        let Some((_, raw_final_amount)) = new_phases
            .iter()
            .find(|(candidate, _)| candidate.as_str() == phase.as_str())
        else {
            return Err(component_projection_error(
                ProjectionReason::PhaseReadbackMissing,
                format!("missing phase readback {phase}"),
            ));
        };
        if !initial.is_finite() || *initial < 0.0 || !raw_final_amount.is_finite() {
            return Err(component_projection_error(
                ProjectionReason::PhaseAmountInvalid,
                format!("invalid phase amount {phase}"),
            ));
        }
        let final_amount = exhausted_endpoint
            .as_ref()
            .filter(|(endpoint_index, _, _)| *endpoint_index == phase_index)
            .map_or(*raw_final_amount, |(_, _, corrected)| *corrected);
        if let Some(gas) = problem
            .external_gases
            .iter()
            .find(|gas| gas.phase.as_str() == phase.as_str())
        {
            if !add_gas_component_transfer(
                &mut targets,
                &unique_keys,
                phase,
                gas.initial_moles - final_amount,
            ) {
                return Ok(false);
            }
        } else if let Some(mineral) = derived::phase_by_name(phase) {
            if final_amount < 0.0 {
                return Err(component_projection_error(
                    ProjectionReason::MineralAmountNegative,
                    format!("negative mineral amount {phase}"),
                ));
            }
            for (key, coefficient) in &mineral.elements {
                *targets.entry(key.clone()).or_default() +=
                    (*initial - final_amount) * *coefficient;
            }
        } else {
            // Even a trace with no identified owner is outside this
            // projection's phase/component map.
            return Ok(false);
        }
    }
    for (phase, species, _) in &problem.gases {
        let Some((_, _, final_amount)) = new_gases.iter().find(|(name, key, _)| {
            name.as_str() == phase.as_str() && key.as_str() == species.as_str()
        }) else {
            return Err(component_projection_error(
                ProjectionReason::GasReadbackMissing,
                format!("missing finite gas readback {phase}"),
            ));
        };
        if !final_amount.is_finite() || *final_amount < 0.0 {
            return Err(component_projection_error(
                ProjectionReason::GasAmountInvalid,
                format!("invalid finite gas amount {phase}"),
            ));
        }
        let initial: f64 = vessel
            .contents
            .iter()
            .filter(|portion| {
                portion.phase == Phase::Gas && portion.species.0.as_str() == species.as_str()
            })
            .map(|portion| portion.moles.0)
            .sum();
        if !initial.is_finite() || initial < 0.0 {
            return Err(component_projection_error(
                ProjectionReason::InitialGasInvalid,
                format!("invalid initial gas amount {phase}"),
            ));
        }
        if !add_gas_component_transfer(&mut targets, &unique_keys, phase, initial - *final_amount) {
            return Ok(false);
        }
    }
    // A new state key absent from both input and a physical phase transfer is
    // a new valence/component problem, not a rounding residual to erase.
    if new_ions
        .iter()
        .any(|(key, amount)| !targets.contains_key(key) && *amount > 0.0)
    {
        return Ok(false);
    }
    for (key, mut target) in targets {
        if exhausted_endpoint
            .as_ref()
            .is_some_and(|(_, endpoint_key, _)| endpoint_key == &key)
        {
            if !target.is_finite() || target.abs() > 1e-14 {
                return Err(component_projection_error(
                    ProjectionReason::ReservoirEndpointResidual,
                    format!("exhausted reservoir endpoint did not close {key}: residual={target}"),
                ));
            }
            // Exact zero is assigned only after the registered gas-phase
            // witness has been corrected and the constrained total recomputed.
            target = 0.0;
        }
        let current: f64 = new_ions
            .iter()
            .filter(|(candidate, _)| candidate.as_str() == key.as_str())
            .map(|(_, amount)| *amount)
            .sum();
        if !target.is_finite() || target < 0.0 || !current.is_finite() || current < 0.0 {
            return Err(component_projection_error(
                ProjectionReason::ComponentTargetInvalid,
                format!("invalid component amount {key}: target={target}, readback={current}"),
            ));
        }
        let tolerance = 1e-14_f64.max(target.abs() * 1e-7);
        if (current - target).abs() > tolerance || (target > 0.0 && current == 0.0) {
            return Err(component_projection_error(
                ProjectionReason::ComponentBoundExceeded,
                format!(
                    "component {key} exceeds readback bound: target={target}, readback={current}, bound={tolerance}"
                ),
            ));
        }
        if current > 0.0 {
            let scale = target / current;
            for (candidate, amount) in new_ions.iter_mut() {
                if candidate.as_str() == key.as_str() {
                    *amount *= scale;
                }
            }
        }
    }
    if let Some((phase_index, _, corrected_amount)) = exhausted_endpoint {
        new_phases[phase_index].1 = corrected_amount;
    }
    Ok(true)
}

#[cfg(test)]
mod gas_component_projection_tests {
    use super::*;
    use kerotakis_core::{
        material::MaterialBasis,
        vessel::{MaterialObject, MaterialObjectState, UnresolvedMaterialPortion},
        Grams, SurfaceModel, VesselId,
    };

    fn open_problem() -> Problem {
        Problem {
            kgw: 0.1,
            solvent_only: false,
            clock_co2_applied_mol: 0.0,
            totals: vec![
                ("Na".into(), 0.02),
                ("Acetate".into(), 0.005),
                ("C".into(), 0.02),
            ],
            phases: vec![("CO2(g)".into(), 0.0, 0.0)],
            gases: vec![],
            external_gases: vec![ExternalGas {
                phase: "CO2(g)".into(),
                species: "CO2".into(),
                initial_moles: 0.0,
                kind: ExternalGasKind::Reservoir,
            }],
            surfaces: vec![],
            exchanges: vec![],
            solid_solutions: vec![],
            elements: vec!["Na".into(), "Acetate".into(), "C".into()],
        }
    }

    fn readback(carbon: f64) -> Vec<(String, f64)> {
        vec![
            ("Na".into(), 0.02 + 1e-10),
            ("Acetate".into(), 0.005 - 2e-10),
            ("C".into(), carbon),
        ]
    }

    fn exhausted_co2_problem(input_carbon: f64, reservoir_initial: f64) -> Problem {
        Problem {
            kgw: 0.0997,
            solvent_only: false,
            clock_co2_applied_mol: 0.0,
            totals: vec![("C".into(), input_carbon)],
            phases: vec![("CO2(g)".into(), 0.0, -12.0)],
            gases: vec![],
            external_gases: vec![ExternalGas {
                phase: "CO2(g)".into(),
                species: "CO2".into(),
                initial_moles: reservoir_initial,
                kind: ExternalGasKind::Reservoir,
            }],
            surfaces: vec![],
            exchanges: vec![],
            solid_solutions: vec![],
            elements: vec!["C".into()],
        }
    }

    fn project(
        vessel: &Vessel,
        problem: &Problem,
        new_ions: &mut [(String, f64)],
        new_phases: &[(String, f64)],
        new_gases: &[(String, String, f64)],
        certified_reentry: bool,
    ) -> Result<bool, SolveError> {
        let mut phases = new_phases.to_vec();
        super::project_gas_aware_components(
            vessel,
            problem,
            new_ions,
            &mut phases,
            new_gases,
            if certified_reentry {
                ComponentCertificate::ClockWithOpaque
            } else {
                ComponentCertificate::Uncertified
            },
            false,
        )
    }

    fn reservoir_problem() -> Problem {
        Problem {
            kgw: 0.10949,
            solvent_only: false,
            clock_co2_applied_mol: 0.0,
            totals: vec![("C".into(), 0.03774), ("Na".into(), 0.07548)],
            phases: vec![("CO2(g)".into(), 0.0, -3.408)],
            gases: vec![],
            external_gases: vec![ExternalGas {
                phase: "CO2(g)".into(),
                species: "CO2".into(),
                initial_moles: 0.0,
                kind: ExternalGasKind::Reservoir,
            }],
            surfaces: vec![],
            exchanges: vec![],
            solid_solutions: vec![],
            elements: vec!["C".into(), "Na".into()],
        }
    }

    fn reservoir_vessel() -> Vessel {
        let mut vessel = Vessel::new(VesselId(0), "open optical reporter mechanism");
        vessel.deposit(SpeciesId::new("water"), Moles(6.0), Phase::Liquid);
        vessel.deposit(SpeciesId::new("Na+"), Moles(0.07548), Phase::Aqueous);
        vessel.deposit(SpeciesId::new("CO3-2"), Moles(0.03774), Phase::Aqueous);
        vessel.deposit(
            SpeciesId::new("anthocyanin"),
            Moles(6.690_753_378_830_455e-6),
            Phase::Aqueous,
        );
        vessel.unresolved_materials.push(UnresolvedMaterialPortion {
            material: "red_cabbage_juice".into(),
            recipe_id: "household/red-cabbage-juice-surrogate".into(),
            recipe_version: 1,
            basis: MaterialBasis::MassFraction,
            amount: 0.197,
            enzyme_hydrolysis: None,
            protein_denatured_fraction: 0.0,
        });
        vessel
    }

    #[test]
    fn reservoir_opaque_owner_requires_explicit_native_opt_in_and_keeps_clock_distinct() {
        // Source eligibility and shared projector controls, not mock chemistry.
        let vessel = reservoir_vessel();
        let problem = reservoir_problem();
        assert!(reservoir_opaque_component_owner(
            &vessel,
            &problem,
            ComponentCertificate::Uncertified,
        ));
        assert!(reservoir_opaque_component_owner(
            &vessel,
            &problem,
            ComponentCertificate::ReservoirWithOpaque,
        ));
        assert!(!reservoir_opaque_component_owner(
            &vessel,
            &problem,
            ComponentCertificate::ClockWithOpaque,
        ));
        let mut ions = problem.totals.clone();
        let mut phases = vec![("CO2(g)".into(), 0.0)];
        assert!(!project_gas_aware_components(
            &vessel,
            &problem,
            &mut ions,
            &mut phases,
            &[],
            ComponentCertificate::Uncertified,
            false,
        )
        .unwrap());
        assert!(project_gas_aware_components(
            &vessel,
            &problem,
            &mut ions,
            &mut phases,
            &[],
            ComponentCertificate::Uncertified,
            true,
        )
        .unwrap());

        let mut changed = vessel.clone();
        changed
            .contents
            .iter_mut()
            .find(|p| p.species.0 == "anthocyanin")
            .unwrap()
            .phase = Phase::Solid;
        assert!(!reservoir_opaque_component_owner(
            &changed,
            &problem,
            ComponentCertificate::Uncertified,
        ));
        changed = vessel.clone();
        changed.deposit(SpeciesId::new("malic_acid"), Moles(0.001), Phase::Aqueous);
        assert!(!reservoir_opaque_component_owner(
            &changed,
            &problem,
            ComponentCertificate::Uncertified,
        ));
        changed = vessel.clone();
        changed.deposit(SpeciesId::new("KIO3"), Moles(0.001), Phase::Aqueous);
        assert!(!reservoir_opaque_component_owner(
            &changed,
            &problem,
            ComponentCertificate::Uncertified,
        ));
        changed = vessel.clone();
        changed.pending_co2_transfer_mol = 2e-6;
        assert!(!reservoir_opaque_component_owner(
            &changed,
            &problem,
            ComponentCertificate::Uncertified,
        ));
        let mut dose = problem.clone();
        dose.external_gases[0].kind = ExternalGasKind::Dose;
        assert!(!reservoir_opaque_component_owner(
            &vessel,
            &dose,
            ComponentCertificate::Uncertified,
        ));
        let mut unreviewed = problem.clone();
        unreviewed.external_gases[0].species = "HBr".into();
        assert!(!reservoir_opaque_component_owner(
            &vessel,
            &unreviewed,
            ComponentCertificate::Uncertified,
        ));
        let mut missing = problem.clone();
        missing.external_gases.clear();
        assert!(!reservoir_opaque_component_owner(
            &vessel,
            &missing,
            ComponentCertificate::Uncertified,
        ));
    }

    #[test]
    fn milk_and_culture_recipe_components_have_derived_owners_or_are_explicitly_opaque() {
        let milk = kerotakis_core::material::lookup("whole_milk", None).expect("milk recipe");
        let expanded = milk.expand(103.0, 0).expect("100 mL milk expansion");
        let components: BTreeSet<_> = expanded
            .components
            .iter()
            .map(|component| component.species_id.as_str())
            .collect();
        assert_eq!(
            components,
            BTreeSet::from([
                "water",
                "K+",
                "Na+",
                "Ca+2",
                "Cl-",
                "H2PO4-",
                "C6H5O7-3",
                "octacalcium_phosphate",
            ])
        );
        for key in components.into_iter().chain(["lactic_acid", "lactate"]) {
            assert!(derived::role(key).is_some(), "{key} needs a native owner");
        }
        assert!(expanded.unresolved_amount > 0.0);
        let culture = kerotakis_core::material::lookup("yoghurt_culture", None)
            .expect("starter culture recipe");
        let expanded_culture = culture.expand(1.0, 0).expect("one gram culture");
        assert!(expanded_culture.components.is_empty());
        assert!(expanded_culture.unresolved_amount > 0.0);
    }

    #[test]
    fn opaque_owner_is_clock_only_and_requires_every_represented_component() {
        let mut vessel = Vessel::new(VesselId(0), "known serum plus opaque recipe");
        vessel.deposit(SpeciesId::new("water"), Moles(5.0), Phase::Liquid);
        vessel.deposit(SpeciesId::new("Na+"), Moles(0.02), Phase::Aqueous);
        vessel.unresolved_materials.push(UnresolvedMaterialPortion {
            material: "whole_milk".into(),
            recipe_id: "household/whole-milk-surrogate".into(),
            recipe_version: 1,
            basis: MaterialBasis::MassFraction,
            amount: 12.0,
            enzyme_hydrolysis: None,
            protein_denatured_fraction: 0.0,
        });
        let mut problem = open_problem();
        let mut ions = readback(0.02 + 1e-10);
        assert!(!project(&vessel, &problem, &mut ions, &[], &[], false,).unwrap());
        problem.clock_co2_applied_mol = 1e-6;
        assert!(opaque_clock_component_owner(
            &vessel,
            &problem,
            ComponentCertificate::Uncertified
        ));
        let mut ions = readback(0.02 + 1e-10);
        assert!(project(
            &vessel,
            &problem,
            &mut ions,
            &[("CO2(g)".into(), 0.0)],
            &[],
            false,
        )
        .unwrap());
        problem.clock_co2_applied_mol = 0.0;
        assert!(opaque_clock_component_owner(
            &vessel,
            &problem,
            ComponentCertificate::ClockWithOpaque
        ));
        assert!(!opaque_clock_component_owner(
            &vessel,
            &problem,
            ComponentCertificate::Uncertified
        ));

        problem.clock_co2_applied_mol = 1e-6;
        vessel.deposit(
            SpeciesId::new("unmapped_reactive_ion"),
            Moles(0.001),
            Phase::Aqueous,
        );
        assert!(!opaque_clock_component_owner(
            &vessel,
            &problem,
            ComponentCertificate::Uncertified
        ));
        vessel
            .contents
            .retain(|portion| portion.species.0 != "unmapped_reactive_ion");
        // The current registered ionic gap is iodate: KIO3 has a real
        // formula but no derived role on the routed native datasets.
        assert!(species::lookup_key("KIO3").is_some());
        assert!(derived::role("KIO3").is_none());
        vessel.deposit(SpeciesId::new("KIO3"), Moles(0.001), Phase::Aqueous);
        assert!(!opaque_clock_component_owner(
            &vessel,
            &problem,
            ComponentCertificate::Uncertified
        ));
        let mut ions = readback(0.02 + 1e-10);
        assert!(!project(&vessel, &problem, &mut ions, &[], &[], false,).unwrap());
        vessel
            .contents
            .retain(|portion| portion.species.0 != "KIO3");
        problem.external_gases[0].kind = ExternalGasKind::Dose;
        let mut ions = readback(0.02 + 1e-10);
        assert!(!project(&vessel, &problem, &mut ions, &[], &[], false,).unwrap());
        problem.external_gases[0].kind = ExternalGasKind::Reservoir;
        problem.surfaces.push(SurfaceSites {
            label: "unsupported interface".into(),
            model: SurfaceModel::HydrousFerricOxide,
            mass: Grams(0.09),
            specific_area_m2_per_g: 600.0,
            strong_capacity: Moles(5e-6),
            weak_capacity: Moles(2e-4),
            occupancy: vec![],
            water_release: Moles(0.0),
        });
        let mut ions = readback(0.02 + 1e-10);
        assert!(!project(&vessel, &problem, &mut ions, &[], &[], false,).unwrap());
        problem.surfaces.clear();
        vessel.material_objects.push(MaterialObject {
            material: "prepared spectator".into(),
            recipe_id: "test/prepared".into(),
            recipe_version: 1,
            mass_g: 1.0,
            components: vec![],
            state: MaterialObjectState::default(),
        });
        let mut ions = readback(0.02 + 1e-10);
        assert!(!project(&vessel, &problem, &mut ions, &[], &[], false,).unwrap());
        vessel.material_objects.clear();
        vessel.unresolved_materials[0].amount = f64::NAN;
        assert!(!opaque_clock_component_owner(
            &vessel,
            &problem,
            ComponentCertificate::Uncertified
        ));
    }

    #[test]
    fn clock_overdraw_cannot_be_admitted_by_opaque_owner() {
        let mut vessel = Vessel::new(VesselId(0), "clock overdraw");
        vessel.deposit(SpeciesId::new("water"), Moles(5.0), Phase::Liquid);
        vessel.deposit(SpeciesId::new("Na+"), Moles(1e-4), Phase::Aqueous);
        vessel.deposit(SpeciesId::new("HCO3-"), Moles(1e-4), Phase::Aqueous);
        vessel.unresolved_materials.push(UnresolvedMaterialPortion {
            material: "whole_milk".into(),
            recipe_id: "household/whole-milk-surrogate".into(),
            recipe_version: 1,
            basis: MaterialBasis::MassFraction,
            amount: 12.0,
            enzyme_hydrolysis: None,
            protein_denatured_fraction: 0.0,
        });
        vessel.pending_co2_transfer_mol = 2e-6;
        let supported = partition(&vessel).expect("represented carbonate and water");
        assert!(validate_clock_boundary(&vessel, Some(&supported)).is_ok());
        assert!(opaque_clock_component_owner(
            &vessel,
            &supported,
            ComponentCertificate::Uncertified
        ));
        vessel.pending_co2_transfer_mol = -1e-3;
        let problem = partition(&vessel).expect("represented carbonate and water");
        assert!(validate_clock_boundary(&vessel, Some(&problem)).is_err());
    }

    #[test]
    fn clock_only_component_projection_requires_typed_first_or_reentry_owner() {
        let vessel = Vessel::new(VesselId(0), "clock-only projection");
        let mut problem = open_problem();
        problem.phases.clear();
        problem.external_gases.clear();
        let mut first = readback(0.02 + 1e-10);
        problem.clock_co2_applied_mol = 1e-6;
        assert!(project(&vessel, &problem, &mut first, &[], &[], false,).unwrap());
        assert!((amount(&first, "C") - 0.02).abs() < 1e-16);

        problem.clock_co2_applied_mol = 0.0;
        let mut ordinary = readback(0.02 + 1e-10);
        assert!(!project(&vessel, &problem, &mut ordinary, &[], &[], false,).unwrap());
        assert!(project(&vessel, &problem, &mut ordinary, &[], &[], true,).unwrap());
        let mut invalid = readback(0.021);
        assert!(project(&vessel, &problem, &mut invalid, &[], &[], true,).is_err());
    }

    fn amount(ions: &[(String, f64)], key: &str) -> f64 {
        ions.iter()
            .find(|(candidate, _)| candidate == key)
            .unwrap()
            .1
    }

    #[test]
    fn reservoir_out_and_in_preserve_independent_native_components() {
        let vessel = Vessel::new(VesselId(0), "pure bookkeeping");
        let problem = open_problem();
        let mut outward = readback(0.015 + 3e-10);
        assert!(project(
            &vessel,
            &problem,
            &mut outward,
            &[("CO2(g)".into(), 0.005)],
            &[],
            false,
        )
        .unwrap());
        assert!((amount(&outward, "Na") - 0.02).abs() < 1e-16);
        assert!((amount(&outward, "Acetate") - 0.005).abs() < 1e-16);
        assert!((amount(&outward, "C") - 0.015).abs() < 1e-16);

        let mut inward = vec![
            ("Na".into(), 0.02),
            ("Acetate".into(), 0.005),
            ("C".into(), 0.023 + 4e-10),
        ];
        assert!(project(
            &vessel,
            &problem,
            &mut inward,
            &[("CO2(g)".into(), -0.003)],
            &[],
            false,
        )
        .unwrap());
        assert!((amount(&inward, "C") - 0.023).abs() < 1e-16);
        assert!((amount(&inward, "Acetate") - 0.005).abs() < 1e-16);
    }

    #[test]
    fn exhausted_registered_reservoir_closes_only_gas_amount_at_native_precision() {
        // Frozen aq-064 target from the saved capture: native dissolved C is
        // exactly zero while the external CO2 reservoir carries the residue.
        let residual = 1.806_367_555_534_649_2e-14;
        let input = 0.034_050_065_360_08;
        let raw_gas_out = input - residual;
        let problem = exhausted_co2_problem(input, 0.0);
        let vessel = Vessel::new(VesselId(0), "exhausted CO2 endpoint");
        let mut ions = vec![("C".into(), 0.0)];
        let mut phases = vec![("CO2(g)".into(), raw_gas_out)];
        let original_endpoint =
            problem.totals[0].1 + problem.external_gases[0].initial_moles - phases[0].1;
        assert!((original_endpoint - residual).abs() < 1e-17);

        assert!(super::project_gas_aware_components(
            &vessel,
            &problem,
            &mut ions,
            &mut phases,
            &[],
            ComponentCertificate::Uncertified,
            false,
        )
        .unwrap());
        assert_eq!(ions, vec![("C".into(), 0.0)]);
        assert!(phases[0].1 > raw_gas_out, "outward flow remains outward");
        let constrained =
            problem.totals[0].1 + problem.external_gases[0].initial_moles - phases[0].1;
        assert!(constrained.abs() <= 1e-14, "{constrained:.17e}");
        assert!(phases[0].1.is_finite() && phases[0].1 >= 0.0);

        let co2 = species::lookup_key("CO2").expect("registered neutral gas");
        let formula = kerotakis_core::stoich::parse_formula(co2.formula).unwrap();
        assert_eq!(formula.charge, 0.0);
        let moved_gas = phases[0].1 - raw_gas_out;
        for count in formula.counts.values() {
            assert!((moved_gas * *count).abs() <= 1e-10);
        }
        assert!((moved_gas * formula.charge).abs() <= 1e-10);
    }

    #[test]
    fn exhausted_endpoint_requires_one_outward_registered_reservoir_only() {
        let residual = 1.806_367_555_534_649_2e-14;
        let input = 0.034_050_065_360_08;
        let raw_gas_out = input - residual;
        let vessel = Vessel::new(VesselId(0), "endpoint negative witnesses");

        let assert_no_correction =
            |problem: Problem, phases: Vec<(String, f64)>, ions: &mut Vec<(String, f64)>| {
                assert!(super::exhausted_reservoir_endpoint_correction(
                    &vessel, &problem, ions, &phases,
                )
                .is_none());
            };

        let mut ions = vec![("C".into(), 0.0)];
        let mut dose = exhausted_co2_problem(input, 0.0);
        dose.external_gases[0].kind = ExternalGasKind::Dose;
        assert_no_correction(dose, vec![("CO2(g)".into(), raw_gas_out)], &mut ions);
        let mut dose = exhausted_co2_problem(input, 0.0);
        dose.external_gases[0].kind = ExternalGasKind::Dose;
        let phases = vec![("CO2(g)".into(), raw_gas_out)];
        assert!(!project(&vessel, &dose, &mut ions, &phases, &[], false).unwrap());

        let mut ions = vec![("C".into(), 0.0)];
        let mut competing_gas = exhausted_co2_problem(input, 0.0);
        competing_gas.external_gases.push(ExternalGas {
            phase: "HBr(g)".into(),
            species: "HBr".into(),
            initial_moles: 0.0,
            kind: ExternalGasKind::Reservoir,
        });
        assert_no_correction(
            competing_gas,
            vec![("CO2(g)".into(), raw_gas_out)],
            &mut ions,
        );

        let mut ions = vec![("C".into(), 0.0)];
        let mut competing_mineral = exhausted_co2_problem(input, 0.0);
        competing_mineral.phases.push(("Calcite".into(), 0.0, 0.0));
        assert_no_correction(
            competing_mineral,
            vec![("CO2(g)".into(), raw_gas_out), ("Calcite".into(), 0.0)],
            &mut ions,
        );

        let mut ions = vec![("C".into(), 0.0)];
        let too_large = exhausted_co2_problem(input + 2e-10, 0.0);
        let correction = too_large.totals[0].1 - raw_gas_out;
        assert!(correction > 1e-10);
        assert_no_correction(too_large, vec![("CO2(g)".into(), raw_gas_out)], &mut ions);

        let mut ions = vec![("C".into(), 0.0)];
        let insufficient_precision = exhausted_co2_problem(1e-13, 0.0);
        assert_no_correction(
            insufficient_precision,
            vec![("CO2(g)".into(), 5e-14)],
            &mut ions,
        );

        let mut ions = vec![("C".into(), 0.0), ("Na".into(), 1e-6)];
        let problem = exhausted_co2_problem(input, 0.0);
        let mut phases = vec![("CO2(g)".into(), raw_gas_out)];
        assert!(!super::project_gas_aware_components(
            &vessel,
            &problem,
            &mut ions,
            &mut phases,
            &[],
            ComponentCertificate::Uncertified,
            false,
        )
        .unwrap());
        assert_eq!(phases[0].1, raw_gas_out, "declined projection is atomic");

        let inward_problem = exhausted_co2_problem(input, input + 1e-6);
        let inward = vec![("CO2(g)".into(), 1e-6)];
        let mut inward_ions = vec![("C".into(), 0.0)];
        assert_no_correction(inward_problem, inward, &mut inward_ions);

        let mut ions = vec![("C".into(), -1e-30)];
        let mut valid = exhausted_co2_problem(input, 0.0);
        let mut phases = vec![("CO2(g)".into(), raw_gas_out)];
        assert!(super::project_gas_aware_components(
            &vessel,
            &valid,
            &mut ions,
            &mut phases,
            &[],
            ComponentCertificate::Uncertified,
            false,
        )
        .is_err());
        valid.external_gases[0].initial_moles = f64::NAN;
        let ions = vec![("C".into(), 0.0)];
        assert!(
            exhausted_reservoir_endpoint_correction(&vessel, &valid, &ions, &phases,).is_none()
        );
    }

    #[test]
    fn raw_analytical_components_cannot_hide_invalid_valence_readback() {
        for key in ["Na", "C", "N(-3)", "N(5)", "m_HCO3-", "m_H2CO3"] {
            for raw in [-1e-30, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
                assert!(checked_component_readback(key, raw).is_err());
            }
            for raw in [0.0, 1e-30, 0.02] {
                assert_eq!(checked_component_readback(key, raw).unwrap(), raw);
            }
        }
    }

    #[test]
    fn certified_components_require_complete_native_protonation() {
        let ions = vec![("N(-3)".into(), 0.01)];
        assert!(certify_projected_protonation(&ions, &BTreeMap::new()).is_err());
        let mut fractions = BTreeMap::from([("N(-3)".into(), vec![("NH3", 0.25), ("NH4+", 0.75)])]);
        assert!(certify_projected_protonation(&ions, &fractions).is_ok());
        fractions.get_mut("N(-3)").unwrap().pop();
        assert!(certify_projected_protonation(&ions, &fractions).is_err());
        fractions.insert("N(-3)".into(), vec![("NH3", f64::NAN), ("NH4+", 1.0)]);
        assert!(certify_projected_protonation(&ions, &fractions).is_err());
    }

    #[test]
    fn declared_native_alias_multiplicity_preserves_all_keyed_fractions() {
        let ions = vec![("C".into(), 0.02)];
        // Synthetic shares test mapping semantics, not a measured distribution.
        let shares = vec![
            ("CO2(aq)", 0.2),
            ("CO2(aq)", 0.1),
            ("HCO3-", 0.3),
            ("CO3-2", 0.4),
        ];
        let check =
            |shares| certify_projected_protonation(&ions, &BTreeMap::from([("C".into(), shares)]));
        assert!(check(shares.clone()).is_ok());
        let mut permuted = shares.clone();
        permuted.reverse();
        assert!(check(permuted).is_ok());
        let mut missing_alias = shares.clone();
        missing_alias.remove(0);
        assert!(check(missing_alias).is_err());
        let mut replaced_alias = shares.clone();
        replaced_alias[1].0 = "HCO3-";
        assert!(check(replaced_alias).is_err());
        let mut wrong_duplicate = shares;
        wrong_duplicate[2].0 = "CO2(aq)";
        assert!(check(wrong_duplicate).is_err());
    }

    #[test]
    fn raw_finite_gas_readback_refuses_invalid_values_before_projection() {
        let vessel = Vessel::new(VesselId(0), "raw gas boundary");
        let mut problem = open_problem();
        problem.phases.clear();
        problem.external_gases.clear();
        problem.gases.push(("CO2(g)".into(), "CO2".into(), 0.0));
        for raw in [-1e-12, f64::NAN, f64::INFINITY] {
            let mut ions = readback(0.02);
            let result = PhreeqcEquilibrator::apply_balance_corrections(
                &vessel,
                &problem,
                &[], // Controlled numeric reader below; no native selected-output rows.
                &mut ions,
                &mut [],
                &[],
                &[],
                &|column| match column {
                    "g_CO2(g)" => Some(raw),
                    "pH" => Some(7.0),
                    "mu" => Some(0.1),
                    _ => None,
                },
                ComponentCertificate::Uncertified,
                false,
            );
            let error = result.expect_err("invalid raw gas must not be clamped");
            assert!(error.to_string().contains("invalid raw finite gas column"));
        }
    }

    #[test]
    fn finite_gas_redistribution_uses_owned_initial_and_final_amounts() {
        let mut vessel = Vessel::new(VesselId(0), "finite gas bookkeeping");
        vessel.deposit(SpeciesId::new("CO2"), Moles(0.01), Phase::Gas);
        let mut problem = open_problem();
        problem.phases.clear();
        problem.external_gases.clear();
        problem.gases.push(("CO2(g)".into(), "CO2".into(), 0.0));
        let mut ions = readback(0.024 + 1e-10);
        assert!(project(
            &vessel,
            &problem,
            &mut ions,
            &[],
            &[("CO2(g)".into(), "CO2".into(), 0.006)],
            false,
        )
        .unwrap());
        assert!((amount(&ions, "C") - 0.024).abs() < 1e-16);
    }

    #[test]
    fn tagged_state_stays_tagged_and_protonation_fractions_stay_native() {
        let mut vessel = Vessel::new(VesselId(0), "split bookkeeping");
        vessel.deposit(SpeciesId::new("water"), Moles(5.0), Phase::Liquid);
        let mut problem = open_problem();
        problem.totals = vec![("N(-3)".into(), 0.02), ("Cl".into(), 0.02)];
        problem.elements = vec!["N".into(), "Cl".into()];
        problem.phases.clear();
        problem.external_gases.clear();
        problem.gases.push(("NH3(g)".into(), "NH3".into(), 0.0));
        vessel.deposit(SpeciesId::new("NH3"), Moles(0.001), Phase::Gas);
        let mut ions = vec![("N(-3)".into(), 0.021 + 2e-10), ("Cl".into(), 0.02)];
        assert!(project(
            &vessel,
            &problem,
            &mut ions,
            &[],
            &[("NH3(g)".into(), "NH3".into(), 0.0)],
            false,
        )
        .unwrap());
        assert!((amount(&ions, "N(-3)") - 0.021).abs() < 1e-16);
        assert!(!ions.iter().any(|(key, _)| key == "N"));
        let fractions = BTreeMap::from([("N(-3)".into(), vec![("NH3", 0.25), ("NH4+", 0.75)])]);
        let (_, contents) = PhreeqcEquilibrator::rebuild_contents_and_events(
            &vessel,
            &problem,
            &[],
            0.1,
            &ions,
            &[],
            &[("NH3(g)".into(), "NH3".into(), 0.0)],
            &[],
            &fractions,
        );
        let nh3 = contents
            .iter()
            .find(|p| p.species.0 == "NH3" && p.phase == Phase::Aqueous)
            .unwrap()
            .moles
            .0;
        let nh4 = contents
            .iter()
            .find(|p| p.species.0 == "NH4+")
            .unwrap()
            .moles
            .0;
        assert!((nh3 / (nh3 + nh4) - 0.25).abs() < 1e-15);
    }

    #[test]
    fn oversized_invalid_and_unsupported_inputs_never_project() {
        let vessel = Vessel::new(VesselId(0), "negative controls");
        let mut problem = open_problem();
        for carbon in [0.016, f64::NAN, -0.01] {
            let mut ions = readback(carbon);
            assert!(project(
                &vessel,
                &problem,
                &mut ions,
                &[("CO2(g)".into(), 0.005)],
                &[],
                false,
            )
            .is_err());
        }
        let mut ions = readback(0.015);
        problem.external_gases[0].kind = ExternalGasKind::Dose;
        assert!(!project(
            &vessel,
            &problem,
            &mut ions,
            &[("CO2(g)".into(), 0.005)],
            &[],
            false,
        )
        .unwrap());
        problem.external_gases[0].kind = ExternalGasKind::Reservoir;
        problem.surfaces.push(SurfaceSites {
            label: "outside projection".into(),
            model: SurfaceModel::HydrousFerricOxide,
            mass: Grams(0.09),
            specific_area_m2_per_g: 600.0,
            strong_capacity: Moles(5e-6),
            weak_capacity: Moles(2e-4),
            occupancy: vec![],
            water_release: Moles(0.0),
        });
        assert!(!project(
            &vessel,
            &problem,
            &mut ions,
            &[("CO2(g)".into(), 0.005)],
            &[],
            false,
        )
        .unwrap());
        problem.surfaces.clear();
        problem.totals.push(("C(-4)".into(), 1e-4));
        assert!(!project(
            &vessel,
            &problem,
            &mut ions,
            &[("CO2(g)".into(), 0.005)],
            &[],
            false,
        )
        .unwrap());
    }
}

/// An acid in the glass that is not in the pH.
///
/// A sugar dissolving unspeciated is harmless: glucose really does just go
/// into solution, and nothing about the answer is changed by our not
/// having a species for it. An *acid* doing the same thing is not
/// harmless, because whatever pH is published then looks exactly like a pH
/// that accounted for it. A beaker of malic acid reading 7.00 is not a
/// missing feature; it is a wrong answer delivered with a straight face,
/// and the reader most likely to be misled is the one who does not know
/// to go and check which database was routed to.
///
/// This runs outside the solve rather than inside it, because the case
/// that matters most is the one where no solve happens at all: malic acid
/// and water alone give `partition` no speciating solute, so it declines,
/// and a caveat living in the solution-reporting path would never be
/// reached — silence exactly where the claim is most misleading.
///
/// The note fires wherever the acid is present in whatever phase: solid on
/// the bottom of the beaker is still an acid that is going to dissolve.
/// Is there an acid in this vessel that no shipped database can speciate,
/// in water for it to be an acid in? Malic acid in a dry jar has nothing
/// to be wrong about.
fn holds_unspeciated_acid(vessel: &Vessel) -> bool {
    let has_water = vessel.contents.iter().any(|portion| {
        portion.species.0 == "water" && portion.phase == Phase::Liquid && portion.moles.0 > TRACE
    });
    has_water
        && derived::UNSPECIATED_ACIDS.iter().any(|(key, _)| {
            vessel
                .contents
                .iter()
                .any(|portion| portion.species.0 == *key && portion.moles.0 > TRACE)
        })
}

/// The same argument as `holds_unspeciated_acid`, for the substances that
/// get no aqueous role at all rather than a partial one.
fn holds_unspeciated_solute(vessel: &Vessel) -> bool {
    let has_water = vessel.contents.iter().any(|portion| {
        portion.species.0 == "water" && portion.phase == Phase::Liquid && portion.moles.0 > TRACE
    });
    has_water
        && (derived::UNSPECIATED_SOLUTES.iter().any(|(key, _)| {
            vessel
                .contents
                .iter()
                .any(|portion| portion.species.0 == *key && portion.moles.0 > TRACE)
        }) || !unmapped_ionic_solutes(vessel).is_empty())
}

/// A soluble ionic feed cannot inherit pure water's pH merely because no
/// database contribution was derived for it. Neutral sugars/alcohols and
/// insoluble objects do not trip this boundary.
fn unmapped_ionic_solutes(vessel: &Vessel) -> Vec<&str> {
    vessel
        .contents
        .iter()
        .filter_map(|portion| {
            // H/O analytical coordinates are handled by the aqueous balance,
            // not by element contributions. They are not unsupported solutes.
            if portion.moles.0 <= TRACE
                || species::is_acid_base_basis(&portion.species.0)
                || derived::role(&portion.species.0).is_some()
            {
                return None;
            }
            let data = species::lookup(&portion.species)?;
            if portion.phase != Phase::Aqueous && !data.dissolves_without_speciation {
                return None;
            }
            let formula = kerotakis_core::stoich::parse_formula(data.formula).ok()?;
            let ionic = formula.charge != 0.0
                || formula.counts.keys().any(|el| {
                    matches!(
                        el.as_str(),
                        "Li" | "Na"
                            | "K"
                            | "Rb"
                            | "Cs"
                            | "Mg"
                            | "Ca"
                            | "Sr"
                            | "Ba"
                            | "Al"
                            | "Fe"
                            | "Cu"
                            | "Zn"
                            | "Ag"
                            | "Cd"
                            | "Pb"
                            | "Co"
                            | "Ni"
                            | "Mn"
                            | "Cr"
                    )
                });
            ionic.then_some(portion.species.0.as_str())
        })
        .collect()
}

fn unspeciated_solute_notes(vessel: &Vessel) -> Vec<Event> {
    if !vessel
        .contents
        .iter()
        .any(|p| p.species.0 == "water" && p.phase == Phase::Liquid && p.moles.0 > TRACE)
    {
        return Vec::new();
    }
    let mut notes: Vec<(&str, &str)> = derived::UNSPECIATED_SOLUTES
        .iter()
        .filter(|(key, _)| {
            vessel
                .contents
                .iter()
                .any(|portion| portion.species.0 == *key && portion.moles.0 > TRACE)
        })
        .map(|(key, why)| (*key, *why))
        .collect();
    notes.sort_unstable();
    notes.dedup();
    let mut events: Vec<_> = notes
        .into_iter()
        .map(|(key, why)| {
            let name = kerotakis_core::species::lookup_key(key)
                .map(|d| d.name)
                .unwrap_or(key);
            Event::not_modeled(
                vessel.id,
                // Not in our gift, and not in anybody's — which is the
                // distinction this cause exists to carry.
                kerotakis_core::ops::NotModelledCause::NotInAnyDatabase,
                Phrase::new(
                    "not-modeled.dissolved-and-unspeciated",
                    "{name} is dissolved and unspeciated: {why}",
                    vec![
                        ("name".to_string(), Slot::term("species", name)),
                        (
                            // Keyed by the ROW of `UNSPECIATED_SOLUTES`,
                            // not by its English: the reason is curated
                            // prose with no holes, and a key built out of
                            // its text orphans every translation of it the
                            // moment somebody rewords the row.
                            "why".to_string(),
                            Slot::phrase(Phrase::bare(&format!("unspeciated-solute.{key}"), why)),
                        ),
                    ],
                ),
            )
        })
        .collect();
    let mut unknown = unmapped_ionic_solutes(vessel);
    unknown.sort_unstable();
    unknown.dedup();
    for key in unknown {
        if derived::UNSPECIATED_SOLUTES
            .iter()
            .any(|(known, _)| *known == key)
        {
            continue;
        }
        events.push(Event::not_modeled(
            vessel.id,
            kerotakis_core::ops::NotModelledCause::NotSpeciated,
            Phrase::new(
                "not-modeled.no-component-mapping",
                "{name} is an ionic solute without an aqueous component mapping in this lab. Its ion distribution, acidity, conductivity, and reactions are not included; any solution reading describes only the represented components, not the complete mixture.",
                vec![("name".to_string(), Slot::term("species", key))],
            ),
        ));
    }
    events
}

fn unspeciated_acid_notes(vessel: &Vessel) -> Vec<Event> {
    if !holds_unspeciated_acid(vessel) {
        return Vec::new();
    }
    // The KEY is carried through now, not dropped: the curated half of
    // the sentence is keyed by the ROW of `UNSPECIATED_ACIDS` it came
    // from, and the row is the only thing that names it.
    let mut notes: Vec<(&str, &str)> = derived::UNSPECIATED_ACIDS
        .iter()
        .filter(|(key, _)| {
            vessel
                .contents
                .iter()
                .any(|portion| portion.species.0 == *key && portion.moles.0 > TRACE)
        })
        .map(|(key, why)| (*key, *why))
        .collect();
    notes.sort_unstable();
    notes.dedup();
    notes
        .into_iter()
        .map(|(key, why)| {
            Event::not_modeled(
                vessel.id,
                kerotakis_core::ops::NotModelledCause::NotSpeciated,
                Phrase::new(
                    "not-modeled.unspeciated-acidity",
                    "this solution holds an acid whose acidity is not modelled: {why}. \
                     Whatever pH is shown is the pH of everything else in the glass, and \
                     the real solution is more acidic than it says",
                    vec![(
                        "why".to_string(),
                        Slot::phrase(Phrase::bare(&format!("unspeciated-acid.{key}"), why)),
                    )],
                ),
            )
        })
        .collect()
}

/// The buffer a milk vessel is missing, in the recipe's own words.
///
/// Milk's serum minerals are in the recipe — citrate, phosphate, and the
/// diffusible K/Na/Ca/Cl — and they do real work: an unbuffered lactic
/// fermentation of this size would sit near pH 2.6 and with them it reads
/// 3.8. But casein is conserved as unresolved solids, and the recipe is
/// explicit that its buffering is "the larger part of milk's buffer
/// capacity between pH 6.6 and pH 5.0, which is exactly the interval a
/// yoghurt fermentation crosses". So an acidified milk reads LOW, by a
/// knowable amount, and the beaker says so.
///
/// The sentence is QUOTED from the recipe's own `lot_assumptions` rather
/// than written again here. Two copies of a caveat drift, and the one in
/// the recipe is the one a reader can check against its sources.
fn milk_buffer_notes(vessel: &Vessel, ph: Option<f64>) -> Vec<Event> {
    // Only where the missing buffer actually bites, and BELOW the pH this
    // recipe's own fresh milk sits at. The recipe names 6.6 to 5.0 as the
    // interval casein's buffering dominates, but its fresh milk reads 6.56
    // — so a 6.6 threshold fires on a glass of ordinary milk that nobody
    // has acidified and that is not being under-read. 6.0 is clear of it
    // and still well inside the interval that matters.
    let Some(ph) = ph.filter(|p| *p < 6.0) else {
        return Vec::new();
    };
    let holds_milk = vessel
        .unresolved_materials
        .iter()
        .any(|portion| portion.recipe_id == "household/whole-milk-surrogate");
    if !holds_milk {
        return Vec::new();
    }
    let Some(recipe) = kerotakis_core::material::lookup("whole_milk", None) else {
        return Vec::new();
    };
    // The first SENTENCE of the assumption, not the paragraph. The whole
    // of it is four hundred words of sourcing that belongs where a reader
    // can go and check it — `explain material` prints it in full — and
    // repeating it on every solve of every milk vessel would bury the
    // number it is a caveat about.
    let Some(sentence) = recipe
        .lot_assumptions
        .iter()
        .find(|a| a.contains("CASEIN IS STILL NOT MODELLED"))
        .and_then(|a| a.split_once(", which").map(|(head, _)| head.to_string()))
    else {
        return Vec::new();
    };
    // HALF a buffer, said as half.
    //
    // The colloidal calcium phosphate landed on 2026-09-18 and casein did
    // not, and this note is the owner's condition on shipping one without
    // the other: a half-corrected buffer that reads as a fully-corrected
    // one is worse than the uncorrected one, because nobody re-checks a
    // number that looks right. It looks right now — with the acid a cited
    // yoghurt carries this beaker reads 4.60 against that yoghurt's
    // measured 4.6, where it read 3.94 before — and the agreement is a
    // coincidence of where the modelled colloid runs out rather than
    // evidence that the buffer is complete. So the sentence says which
    // half is here, which half is not, what the missing half is worth, and
    // where the two swap over.
    vec![Event::not_modeled(
        vessel.id,
        kerotakis_core::ops::NotModelledCause::NoReviewedDatum,
        Phrase::new(
            "not-modeled.milk-casein-buffer-residual",
            "this milk has been acidified to pH {ph}, and only half of its \
             buffer is modelled. Its colloidal calcium phosphate is here and \
             dissolves as the acid arrives; casein's own buffering is not here \
             at all, and that is about a third of milk's buffer capacity. \
             Above pH 5 what is missing costs little; below it the colloid is \
             spent and this beaker has nothing left to resist with, where a \
             real one still has casein — so the number is a lower bound again, \
             and by more than it was. {assumption} (the recipe's full \
             assumption is in `explain material whole_milk`)",
            vec![
                ("ph".to_string(), Slot::number(format!("{ph:.2}"))),
                (
                    // The recipe's own words, keyed by the PLACE they are
                    // quoted from. The sentence lives in the material
                    // registry rather than in this source, so a catalogue
                    // has to be able to reach it by a name that survives
                    // the recipe being reworded — which is what a key
                    // built out of its English would not do.
                    "assumption".to_string(),
                    Slot::phrase(Phrase::bare(
                        "material-assumption.whole_milk-casein",
                        &sentence,
                    )),
                ),
            ],
        ),
    )]
}

#[derive(Clone, Copy)]
enum SelectedValueFailure {
    ColumnOrCellMissing,
    NumberMalformed,
    ReaderUnavailable,
}

impl SelectedValueFailure {
    fn phrase(self, column: &str) -> Phrase {
        let (key, en) = match self {
            Self::ColumnOrCellMissing => (
                "solver-reason.aqueous.selected-column-or-cell-missing",
                "The required selected-output column or row cell {column} is missing.",
            ),
            Self::NumberMalformed => (
                "solver-reason.aqueous.selected-number-malformed",
                "The selected-output value in column {column} is present but cannot be parsed as a number.",
            ),
            Self::ReaderUnavailable => (
                "solver-reason.aqueous.selected-reader-unavailable",
                "The selected-output reader did not provide the available value in column {column}.",
            ),
        };
        Phrase::new(key, en, vec![("column".to_string(), Slot::text(column))])
    }
}

fn selected_value_failure(rows: &[Vec<String>], column: &str) -> SelectedValueFailure {
    let Some(index) = rows
        .first()
        .and_then(|header| header.iter().position(|name| name == column))
    else {
        return SelectedValueFailure::ColumnOrCellMissing;
    };
    if rows.len() < 2 || rows.last().and_then(|row| row.get(index)).is_none() {
        return SelectedValueFailure::ColumnOrCellMissing;
    }
    match rows
        .last()
        .and_then(|row| row.get(index))
        .unwrap()
        .parse::<f64>()
    {
        Err(_) => SelectedValueFailure::NumberMalformed,
        Ok(_) => SelectedValueFailure::ReaderUnavailable,
    }
}

fn missing(rows: &[Vec<String>], column: &str) -> SolveError {
    SolveError::NotConverged {
        reason: Some(Box::new(
            selected_value_failure(rows, column).phrase(column),
        )),
        solver: "phreeqc-aqueous".to_string(),
        detail: format!("selected output lacks column '{column}'"),
    }
}

#[cfg(test)]
mod selected_value_failure_tests {
    use super::{missing, selected_value_failure, SelectedValueFailure};
    use kerotakis_core::phrase::{Phrase, Slot};
    use kerotakis_core::SolveError;

    fn reason_and_detail(rows: &[Vec<String>]) -> (Phrase, String, String) {
        match missing(rows, "mu") {
            SolveError::NotConverged {
                solver,
                detail,
                reason: Some(reason),
            } => (*reason, solver, detail),
            other => panic!("expected an owned readback reason, got {other:?}"),
        }
    }

    #[test]
    fn selected_value_failures_distinguish_absence_malformed_and_reader_policy() {
        let absent_header = Vec::<Vec<String>>::new();
        let missing_cell = vec![vec!["mu".to_string()], vec![]];
        let malformed = vec![vec!["mu".to_string()], vec!["not-a-number".to_string()]];
        let reader_unavailable = vec![vec!["mu".to_string()], vec!["0.25".to_string()]];

        assert!(matches!(
            selected_value_failure(&absent_header, "mu"),
            SelectedValueFailure::ColumnOrCellMissing
        ));
        assert!(matches!(
            selected_value_failure(&missing_cell, "mu"),
            SelectedValueFailure::ColumnOrCellMissing
        ));
        assert!(matches!(
            selected_value_failure(&malformed, "mu"),
            SelectedValueFailure::NumberMalformed
        ));
        assert!(matches!(
            selected_value_failure(&reader_unavailable, "mu"),
            SelectedValueFailure::ReaderUnavailable
        ));

        for (rows, expected_key) in [
            (
                absent_header.as_slice(),
                "solver-reason.aqueous.selected-column-or-cell-missing",
            ),
            (
                missing_cell.as_slice(),
                "solver-reason.aqueous.selected-column-or-cell-missing",
            ),
            (
                malformed.as_slice(),
                "solver-reason.aqueous.selected-number-malformed",
            ),
            (
                reader_unavailable.as_slice(),
                "solver-reason.aqueous.selected-reader-unavailable",
            ),
        ] {
            let (reason, solver, detail) = reason_and_detail(rows);
            assert_eq!(reason.key, expected_key);
            assert!(matches!(
                reason.slots.as_slice(),
                [(name, Slot::Text { text })] if name == "column" && text == "mu"
            ));
            assert_eq!(solver, "phreeqc-aqueous");
            assert_eq!(detail, "selected output lacks column 'mu'");
        }
    }
}

/// Elements whose redox couples equilibrate fast enough to matter on a
/// bench, and are therefore allowed to exchange electrons with each other.
///
/// This is curated kinetics, exactly like the thermal solver's 500 K
/// stand-down and copper's metastability threshold, and for the same
/// reason: thermodynamics alone gives the wrong answer about a beaker.
/// Couple everything and a strong reductant drags pe down far enough to
/// reduce sulfate to sulfide, which is real chemistry that takes bacteria
/// and geological time and does not happen in a lesson. Couple nothing and
/// permanganate sits placidly beside iron(II), which is what this engine
/// did until now.
///
/// So: the couples a school actually titrates with are coupled, and the
/// famously sluggish ones — sulfate/sulfide, nitrate/ammonium,
/// carbonate/methane — keep the oxidation state they were added in and say
/// so. Editorial judgement (Kerotakis): the membership of this list is a
/// statement about rates that we do not compute.
const FAST_REDOX: &[&str] = &["Fe", "Mn", "Cu", "Cr"];

/// Elements with reviewed HFO complexes in the routed databases whose bound
/// forms do not yet have a typed `SurfaceSorbate` readback. Refusing these
/// combinations is essential: letting PHREEQC bind them would remove their
/// dissolved totals while leaving no owned inventory in the vessel.
const UNTRACKED_HFO_WATEQ_ELEMENTS: &[&str] = &[
    "Ag", "As", "B", "Ba", "Ca", "Cd", "Cu", "F", "Fe", "Mg", "Mn", "Ni", "P", "Pb", "Se", "Sr",
    "U",
];
const UNTRACKED_HFO_MINTEQ_ELEMENTS: &[&str] = &[
    "Ag", "As", "B", "Ba", "Be", "Ca", "Cd", "Co", "Cr", "Cu", "Hg", "Mg", "Mo", "Ni", "P", "Pb",
    "Sb", "Se", "Sn", "V",
];

/// The gas phase an open vessel's redox state is set by, and its share of
/// the atmosphere. log10(0.21) = −0.68.
const ATMOSPHERIC_OXYGEN: &str = "O2(g)";
const ATMOSPHERIC_LOG_PO2: &str = "-0.68";
/// Effective volatile-gas partial pressure under an ideal continuous purge.
/// It is small rather than zero because equilibrium constants are logarithmic.
const SWEPT_LOG_PARTIAL_PRESSURE: f64 = -12.0;

/// What the electrons in a vessel add up to, and the input that lets
/// PHREEQC redistribute them.
#[derive(Debug, Clone)]
struct RedoxCoupling {
    /// Σ (oxidation state × moles) over the coupled elements, as added.
    /// Conserved by any real redox reaction, so it is what pe must be
    /// solved to reproduce.
    target: f64,
    /// The coupled elements, for reading the answer back.
    columns: Vec<String>,
    /// Σ |oxidation state × moles| as added — the size of the redox
    /// inventory, so the balance residual can be judged against how much
    /// there was to balance rather than against an absolute number.
    scale: f64,
}

/// The charge of a master species that is the bare cation of `base` —
/// `None` for anything else (oxyanions, complexes): the couple synthesis
/// in `build_input_at` must not touch those.
fn simple_cation_charge(base: &str, formula: &str) -> Option<i32> {
    let parsed = kerotakis_core::stoich::parse_formula_with(
        formula,
        kerotakis_core::stoich::FormulaDialect::PhreeqcMaster,
    )
    .ok()?;
    if parsed.counts.len() == 1
        && parsed.counts.get(base).copied() == Some(1.0)
        && parsed.charge.fract() == 0.0
        && parsed.charge != 0.0
    {
        Some(parsed.charge as i32)
    } else {
        None
    }
}

/// The half-reaction defining a tagged oxidation state's master species
/// against the element's primary master, balanced with water and protons:
/// `Fe+2 = Fe+3 + 1 e-`, `Mn+2 + 4 H2O = MnO4- + 8 H+ + 5 e-`. `None`
/// when the master species contains anything besides the element, oxygen
/// and hydrogen, or when the electron count disagrees with the tagged
/// state — the synthesis refuses rather than write a wrong reaction.
fn pin_equation(
    base: &str,
    q_prim: i32,
    primary: &str,
    state: i32,
    master: &str,
) -> Option<String> {
    let parsed = kerotakis_core::stoich::parse_formula_with(
        master,
        kerotakis_core::stoich::FormulaDialect::PhreeqcMaster,
    )
    .ok()?;
    if parsed.counts.get(base).copied() != Some(1.0) {
        return None;
    }
    let o = parsed.counts.get("O").copied().unwrap_or(0.0);
    let h = parsed.counts.get("H").copied().unwrap_or(0.0);
    let expected = 1 + usize::from(o > 0.0) + usize::from(h > 0.0);
    if parsed.counts.len() != expected {
        return None;
    }
    if parsed.charge.fract() != 0.0 || o.fract() != 0.0 || h.fract() != 0.0 {
        return None;
    }
    let q = parsed.charge as i32;
    let water = o as i32;
    let protons = 2 * water - h as i32;
    if protons < 0 {
        return None;
    }
    let electrons = q + protons - q_prim;
    if electrons != state - q_prim || electrons == 0 {
        return None;
    }
    let mut lhs = primary.to_string();
    if water > 0 {
        lhs += &format!(" + {water} H2O");
    }
    let mut rhs = master.to_string();
    if protons > 0 {
        rhs += &format!(" + {protons} H+");
    }
    if electrons > 0 {
        rhs += &format!(" + {electrons} e-");
    } else {
        lhs += &format!(" + {} e-", -electrons);
    }
    Some(format!("{lhs} = {rhs}"))
}

/// The oxidation state written into an element key: `Mn(7)` → 7, `Fe` → None.
fn tagged_state(key: &str) -> Option<i32> {
    let (_, rest) = key.split_once('(')?;
    rest.trim_end_matches(')').parse().ok()
}

/// Whether this vessel has a redox problem worth solving, and what the
/// electron budget is.
///
/// Needs at least two coupled elements *in different oxidation states than
/// each other would settle at* — in practice, two coupled elements at all.
/// One element on its own has nothing to react with here, because water's
/// own redox is deliberately not part of the budget.
fn redox_coupling(problem: &Problem, db_tag: &str) -> Option<RedoxCoupling> {
    redox_coupling_totals(&problem.totals, db_tag)
}

/// Oxidation-state inventory used for admission and pinning, not solution input.
/// Supplied solids remain in EQUILIBRIUM_PHASES; adding them to SOLUTION would
/// count their matter twice. Zero-amount precipitation candidates add no stock.
fn phase_inclusive_totals(problem: &Problem) -> Vec<(String, f64)> {
    let mut totals = problem.totals.clone();
    for (name, moles, _) in &problem.phases {
        if *moles <= 0.0 {
            continue;
        }
        let Some(phase) = derived::phase_by_name(name) else {
            continue;
        };
        for (element, coefficient) in &phase.elements {
            let amount = coefficient * moles;
            if let Some((_, existing)) = totals.iter_mut().find(|(key, _)| key == element) {
                *existing += amount;
            } else {
                totals.push((element.clone(), amount));
            }
        }
    }
    totals
}

fn redox_coupling_totals(totals: &[(String, f64)], db_tag: &str) -> Option<RedoxCoupling> {
    let idx = derived::index_for(db_tag);
    let mut target = 0.0;
    let mut scale = 0.0;
    let mut coupled: Vec<&str> = Vec::new();
    for (key, moles) in totals {
        let base = key.split('(').next().unwrap_or(key);
        if !FAST_REDOX.contains(&base) || !idx.redox_elements.contains(base) {
            continue;
        }
        // The oxidation state this element went in at. Usually the key
        // carries it — `Mn(7)` from permanganate — but a simple salt books
        // as a bare element, and then the state is the charge on the ion it
        // books as: iron(II) sulfate gives `Fe`, which books as Fe+2.
        // Without this, adding permanganate to iron(II) sulfate found no
        // electron budget at all and the two sat side by side unreacted.
        let state = match tagged_state(key) {
            Some(n) => n,
            None => derived::booking_ion(base)
                .and_then(|ion| kerotakis_core::stoich::parse_formula(ion).ok())
                .map(|f| f.charge as i32)?,
        };
        target += state as f64 * moles;
        scale += (state as f64 * moles).abs();
        if !coupled.contains(&base) {
            coupled.push(base);
        }
    }
    if coupled.len() < 2 {
        return None;
    }
    let mut columns: Vec<String> = Vec::new();
    for master in idx.masters.keys() {
        let Some((base, _)) = master.split_once('(') else {
            continue;
        };
        if coupled.contains(&base) && !columns.contains(master) {
            columns.push(master.clone());
        }
    }
    Some(RedoxCoupling {
        target,
        columns,
        scale,
    })
}

/// Σ (oxidation state × moles) read back from a solved distribution.
fn oxidation_sum(rows: &[Vec<String>], columns: &[String], kgw: f64) -> Option<f64> {
    if rows.len() < 2 || columns.is_empty() || !kgw.is_finite() || kgw <= 0.0 {
        return None;
    }
    let header = rows.first()?;
    let last = rows.last()?;
    let mut sum = 0.0;
    for (requested, column) in columns.iter().enumerate() {
        // A partial or ambiguous projection cannot close an electron ledger.
        if columns[..requested].contains(column) {
            return None;
        }
        let i = header.iter().position(|h| h == column)?;
        if header[i + 1..].contains(column) {
            return None;
        }
        let molality: f64 = last.get(i)?.parse().ok()?;
        if !molality.is_finite() || molality < 0.0 {
            return None;
        }
        let state = tagged_state(column)? as f64;
        sum += state * molality * kgw;
        if !sum.is_finite() {
            return None;
        }
    }
    Some(sum)
}

/// The per-oxidation-state totals worth asking for.
///
/// PHREEQC will report an element split across its oxidation states if you
/// name them — `-totals Fe(2) Fe(3)` — and that split *is* the observable
/// of every redox experiment: half the iron oxidised, all the manganese
/// reduced. The states themselves come from the database's master-species
/// block rather than a list of ours, so a database that knows more redox
/// chemistry than we do is not silently cut down to what we thought of.
fn valence_totals(problem: &Problem, db_tag: &str) -> Vec<String> {
    let idx = derived::index_for(db_tag);
    let present: Vec<&str> = problem
        .elements
        .iter()
        .map(|el| el.split('(').next().unwrap_or(el))
        .filter(|el| idx.redox_elements.contains(*el))
        // Hydrogen and oxygen have oxidation states in every database and
        // are in every aqueous solution; reporting their split would be
        // noise, not chemistry.
        .filter(|el| *el != "H" && *el != "O")
        .collect();
    let mut out: Vec<String> = Vec::new();
    for master in idx.masters.keys() {
        let Some((base, _)) = master.split_once('(') else {
            continue;
        };
        if present.contains(&base) && !out.contains(master) {
            out.push(master.clone());
        }
    }
    out
}

fn build_input(vessel: &Vessel, problem: &Problem, db_tag: &str) -> String {
    build_input_at(vessel, problem, db_tag, None)
}

/// The name `db_tag` must be asked about to pose the solid the candidate
/// list calls `name`, or `None` when that database cannot represent it at
/// all.
///
/// The candidate list is database-blind on purpose; the input is not. A
/// name the routed database defines passes through — this includes gases
/// like `CO2(g)`, which the mineral map never holds. A name it lacks is
/// translated to that database's own polymorph of the same solid: the fix
/// for iron hydroxide sitting at SI +27 while the input asked wateq4f for a
/// phase only minteq defines. No polymorph but a reviewed foreign
/// definition (injected by `foreign_phase_definitions`) → posed as itself.
/// No definition at all → `None`, and that is exactly the case where the
/// honesty pass's supersaturation note is the right answer.
///
/// Every path that poses a phase goes through here. They did not used to:
/// the MIX builder kept only names the routed database defines natively, so
/// two solutions combined by fraction could not grow a precipitate the same
/// reagents in one beaker did.
fn posed_phase<'a>(name: &'a str, db_tag: &str) -> Option<&'a str> {
    if derived::index_for(db_tag).has_phase(name) {
        Some(name)
    } else if let Some(alt) = derived::phase_in_db(name, db_tag) {
        Some(alt)
    } else if derived::foreign_phase_definition(name, db_tag).is_some() {
        Some(name)
    } else {
        None
    }
}

/// The `PHASES` blocks a routed database needs before anything in the input
/// may reference a reviewed foreign phase.
///
/// PHREEQC accepts PHASES blocks in the input stream, and the injected
/// definition carries the home database's log K (see
/// `derived::foreign_phase_definition`). This is how ferrous hydroxide is
/// posable on wateq4f, which does not define it, without rerouting the
/// whole solve — rerouting made the answer depend on the order reagents
/// were added in.
fn foreign_phase_definitions(phases: &[(String, f64, f64)], db_tag: &str) -> String {
    let mut block = String::new();
    for (phase, ..) in phases {
        if let Some(definition) = derived::foreign_phase_definition(phase, db_tag) {
            if !block.contains(definition.as_str()) {
                block.push_str(&definition);
            }
        }
    }
    block
}

fn phase_input_precision(vessel: &Vessel) -> bool {
    vessel.temperature.0 <= kerotakis_core::states::WATER_FREEZING_K
        || vessel
            .contents
            .iter()
            .any(|portion| portion.species.0 == "water" && portion.phase == Phase::Solid)
}

/// Build a PHREEQC input that defines two solutions and mixes them.
///
/// The input defines SOLUTION 1 from vessel A and SOLUTION 2 from vessel B,
/// each with their own END block so they are speciated independently. Then
/// a MIX block combines them by the given fractions into solution 3, which
/// is saved and read back through SELECTED_OUTPUT.
///
/// `merged` is the mixture's own `Problem`: its phases have already been
/// reconciled with the routed database by `posed_phase`, exactly as
/// `setup_problem` reconciles the direct path's candidates, and the names
/// there are the names PHREEQC is asked about and the names the readback
/// reads back.
#[allow(clippy::too_many_arguments)]
fn build_mix_input(
    vessel_a: &Vessel,
    problem_a: &Problem,
    vessel_b: &Vessel,
    problem_b: &Problem,
    frac_a: f64,
    frac_b: f64,
    db_tag: &str,
    merged: &Problem,
    phase_precision: bool,
) -> String {
    use std::fmt::Write;
    let mut input = String::new();
    // One cold/icy source or receiver makes the entire MIX transaction
    // phase-sensitive: preserve both sources' analytical water budgets.
    let phase_precision =
        phase_precision || phase_input_precision(vessel_a) || phase_input_precision(vessel_b);
    // A reviewed foreign phase is defined before anything references it —
    // the same injection the direct path makes.
    input.push_str(&foreign_phase_definitions(&merged.phases, db_tag));
    // And the same pin, over the merged totals. Posing a phase is what
    // makes it necessary — PHREEQC's reaction step redistributes an
    // uncoupled element across its states against pe on any step that has
    // one — so this path acquired the gap the moment it started posing
    // phases at all: iron poured in as iron(III) came back part iron(II),
    // and that part was outside the mass balance the readback closes.
    // Where the merged problem would be *coupled* the direct path pins
    // nothing and bisects pe instead; MIX has no coupled solve, so it
    // leaves that case exactly as it was rather than pinning an answer the
    // direct path would have solved for.
    if redox_coupling(merged, db_tag).is_none() {
        input.push_str(&fast_redox_pin_block(&merged.totals, db_tag));
    }

    // SELECTED_OUTPUT is stated *before* the first simulation, and that
    // placement is load-bearing rather than cosmetic.
    //
    // A `SELECTED_OUTPUT` definition outlives the input that made it: it
    // survives `DELETE -all` (which clears numbered reactants, not output
    // definitions) and therefore the whole reset the engine pool does
    // between runs. This input is three simulations in one run, and with
    // the block written last the first two punched their rows under
    // *whatever the previous solve on this engine instance had defined* —
    // so the accumulated selected output opened with a stale heading row
    // and only later carried this input's own. `rows.first()` is the
    // heading and `rows.last()` is the answer, and they came from
    // different definitions: the readback then asked for a column by the
    // wrong name ("selected output lacks column 'Na'") or, worse, read a
    // real number from the wrong column. Stated up front, one definition
    // covers every punch in the run and heading and answer agree.
    //
    // This is why the MIX solve had never once completed: `Bench` treats a
    // failed MIX as advisory and re-solves the target through the direct
    // path, so the only symptom was a second engine call.
    input.push_str(&mix_selected_output(merged, db_tag));

    // SOLUTION 1 — vessel A.
    let temp_a_c = vessel_a.temperature.to_celsius();
    writeln!(input, "SOLUTION 1").unwrap();
    writeln!(input, "    units     mol/kgw").unwrap();
    if phase_precision {
        writeln!(input, "    temp      {temp_a_c:.16e}").unwrap();
    } else {
        writeln!(input, "    temp      {temp_a_c:.4}").unwrap();
    }
    writeln!(input, "    pH        7  charge").unwrap();
    // Molalities below use the unrounded kgw; write that same solvent mass.
    writeln!(input, "    water     {:.16e}", problem_a.kgw).unwrap();
    if vessel_a.uses_atmospheric_reservoir()
        && derived::index_for(db_tag).has_phase(ATMOSPHERIC_OXYGEN)
    {
        writeln!(
            input,
            "    pe        4  {ATMOSPHERIC_OXYGEN}  {ATMOSPHERIC_LOG_PO2}"
        )
        .unwrap();
    }
    for (el, moles) in &problem_a.totals {
        writeln!(input, "    {el} {:.12e}", moles / problem_a.kgw).unwrap();
    }
    writeln!(input, "END").unwrap();

    // SOLUTION 2 — vessel B.
    let temp_b_c = vessel_b.temperature.to_celsius();
    writeln!(input, "SOLUTION 2").unwrap();
    writeln!(input, "    units     mol/kgw").unwrap();
    if phase_precision {
        writeln!(input, "    temp      {temp_b_c:.16e}").unwrap();
    } else {
        writeln!(input, "    temp      {temp_b_c:.4}").unwrap();
    }
    writeln!(input, "    pH        7  charge").unwrap();
    writeln!(input, "    water     {:.16e}", problem_b.kgw).unwrap();
    if vessel_b.uses_atmospheric_reservoir()
        && derived::index_for(db_tag).has_phase(ATMOSPHERIC_OXYGEN)
    {
        writeln!(
            input,
            "    pe        4  {ATMOSPHERIC_OXYGEN}  {ATMOSPHERIC_LOG_PO2}"
        )
        .unwrap();
    }
    for (el, moles) in &problem_b.totals {
        writeln!(input, "    {el} {:.12e}", moles / problem_b.kgw).unwrap();
    }
    writeln!(input, "END").unwrap();

    // MIX — combine them by fraction.
    writeln!(input, "MIX 1").unwrap();
    writeln!(input, "    1  {frac_a:.12e}").unwrap();
    writeln!(input, "    2  {frac_b:.12e}").unwrap();
    writeln!(input, "SAVE solution 3").unwrap();

    // Candidate equilibrium phases for the mixed solution.
    if !merged.phases.is_empty() {
        writeln!(input, "EQUILIBRIUM_PHASES 1").unwrap();
        for (phase, ..) in &merged.phases {
            writeln!(input, "    {phase} 0 0").unwrap();
        }
    }

    writeln!(input, "END").unwrap();
    input
}

/// The `SELECTED_OUTPUT` block the MIX readback reads: the mixture's own
/// columns, asked for by the names `merged` carries.
fn mix_selected_output(merged: &Problem, db_tag: &str) -> String {
    use std::fmt::Write;
    let mut block = String::new();
    writeln!(block, "SELECTED_OUTPUT").unwrap();
    writeln!(block, "    -reset    false").unwrap();
    writeln!(block, "    -high_precision true").unwrap();
    writeln!(block, "    -ph       true").unwrap();
    writeln!(block, "    -pe       true").unwrap();
    writeln!(block, "    -ionic_strength true").unwrap();
    writeln!(block, "    -water    true").unwrap();
    // Phase equilibrium needs numerical activity, not the four-digit report.
    writeln!(block, "    -activities H2O").unwrap();
    // Element totals, plus the per-oxidation-state split for any redox
    // element present — the same columns the direct path asks for. Without
    // the split, an element that moves between states inside the solve is
    // read back short by exactly the mass that moved.
    let mut totals: Vec<String> = Vec::new();
    for e in merged
        .elements
        .iter()
        .cloned()
        .chain(valence_totals(merged, db_tag))
    {
        if !totals.contains(&e) {
            totals.push(e);
        }
    }
    if !totals.is_empty() {
        writeln!(block, "    -totals   {}", totals.join(" ")).unwrap();
    }
    // The protonation split, on the same terms as the direct path: a
    // mixture that carries reduced nitrogen has to come back knowing
    // whether it is ammonia or ammonium, or decanting one beaker into
    // another would rename what is in it.
    let mut molalities: Vec<&str> = vec!["H+", "OH-"];
    for total in &totals {
        for (species, _) in derived::protonation_split(total).unwrap_or(&[]) {
            if !molalities.contains(species) {
                molalities.push(species);
            }
        }
    }
    if !molalities.is_empty() {
        writeln!(block, "    -molalities {}", molalities.join(" ")).unwrap();
    }
    if !merged.phases.is_empty() {
        let names: Vec<&str> = merged.phases.iter().map(|(p, ..)| p.as_str()).collect();
        writeln!(block, "    -equilibrium_phases {}", names.join(" ")).unwrap();
    }
    append_particle_census(&mut block);
    block
}

/// The `SOLUTION_SPECIES` block that keeps an uncoupled fast-redox element
/// in the state it was added in, or the empty string when nothing needs
/// pinning.
///
/// An uncoupled element keeps the oxidation state it was added in (see
/// FAST_REDOX) — but PHREEQC's reaction step redistributes a trace
/// across states against pe whenever any equilibrium phase is posed,
/// and the trace is not free: iron(III) formed this way hydrolyses,
/// and the water it consumed leaked from the ledger on every solve —
/// 7.6e-5 mol per pass, caught by the displacement metamorphic test as
/// an answer that depended on the order reagents were added in. The
/// state is pinned *inside* the solve instead: the couple's own log K
/// is redefined in the input stream to ±50, far enough that the other
/// state cannot carry mass. Only simple cation↔cation couples are
/// synthesized; an oxyanion state (permanganate) keeps its database
/// reaction, because writing its equation needs O/H bookkeeping this
/// synthesis does not do — and its natural log K already keeps phantom
/// formation negligible.
///
/// Taken over the *problem's* totals rather than a vessel's, because the
/// MIX path needs the same pin over the merged totals of two solutions and
/// emitted none: it began posing phases before it carried this, and a
/// posed phase is exactly what makes the redistribution fire.
fn fast_redox_pin_block(totals: &[(String, f64)], db_tag: &str) -> String {
    use std::fmt::Write;
    let idx = derived::index_for(db_tag);
    let mut pinned: Vec<&str> = Vec::new();
    let mut block = String::new();
    for (el, _) in totals {
        let base = el.split('(').next().unwrap_or(el);
        if !FAST_REDOX.contains(&base)
            || !idx.redox_elements.contains(base)
            || pinned.contains(&base)
        {
            continue;
        }
        pinned.push(base);
        // The states this element was added in, over every totals key
        // of the base. More than one distinct state without a redox
        // partner is left to the database's own equilibrium.
        let mut added: Vec<i32> = totals
            .iter()
            .filter(|(k, _)| k.split('(').next().unwrap_or(k) == base)
            .filter_map(|(k, _)| match tagged_state(k) {
                Some(s) => Some(s),
                None => derived::booking_ion(base)
                    .and_then(|ion| kerotakis_core::stoich::parse_formula(ion).ok())
                    .map(|f| f.charge as i32),
            })
            .collect();
        added.sort_unstable();
        added.dedup();
        let [s_add] = added[..] else { continue };
        // The primary master carries the element's mole balance; the
        // tagged states are defined against it by electron count.
        let Some(primary) = idx.masters.get(base) else {
            continue;
        };
        let Some(q_prim) = simple_cation_charge(base, &primary.species) else {
            continue;
        };
        for (key, master) in &idx.masters {
            let Some((b, _)) = key.split_once('(') else {
                continue;
            };
            if b != base {
                continue;
            }
            let Some(s) = tagged_state(key) else { continue };
            if s == q_prim {
                continue;
            }
            let Some(equation) = pin_equation(base, q_prim, &primary.species, s, &master.species)
            else {
                continue;
            };
            let log_k = if s == s_add { 50.0 } else { -50.0 };
            writeln!(block, "    {equation}\n        log_k {log_k}").unwrap();
        }
    }
    if block.is_empty() {
        return String::new();
    }
    format!("SOLUTION_SPECIES\n{block}")
}

/// The input, optionally with the fast-redox elements coupled at a trial pe.
///
/// Coupling is done by *removing* the valence tag: `Mn(7) 5e-4` pins
/// manganese in its own mass balance, while `Mn 5e-4` lets pe decide where
/// it sits. The elements left tagged stay pinned on purpose — see
/// `FAST_REDOX`.
fn build_input_at(
    vessel: &Vessel,
    problem: &Problem,
    db_tag: &str,
    couple: Option<(f64, &RedoxCoupling)>,
) -> String {
    use std::fmt::Write;
    let mut input = String::new();
    // A reviewed foreign phase is defined before anything references it.
    input.push_str(&foreign_phase_definitions(&problem.phases, db_tag));
    // An uncoupled element keeps the oxidation state it was added in; a
    // coupled solve is bisecting pe and must not have its own answer pinned
    // out from under it. See `fast_redox_pin_block`.
    if couple.is_none() {
        input.push_str(&fast_redox_pin_block(
            &phase_inclusive_totals(problem),
            db_tag,
        ));
    }
    let temp_c = vessel.temperature.to_celsius();
    writeln!(input, "SOLUTION 1").unwrap();
    writeln!(input, "    units     mol/kgw").unwrap();
    let phase_precision = phase_input_precision(vessel);
    if phase_precision {
        writeln!(input, "    temp      {temp_c:.16e}").unwrap();
    } else {
        writeln!(input, "    temp      {temp_c:.4}").unwrap();
    }
    writeln!(input, "    pH        7  charge").unwrap();
    // Every total below is n/kgw. Rounding only kgw changes the amount
    // supplied to PHREEQC before equilibrium, including warm open water.
    writeln!(input, "    water     {:.16e}", problem.kgw).unwrap();
    match couple {
        // Solving for the electron balance: pe is the unknown being
        // bisected, so it is stated outright.
        Some((pe, _)) => writeln!(input, "    pe        {pe:.6}").unwrap(),
        // Otherwise the beaker is open to the room, and the air above it is
        // what sets its redox state. PHREEQC's default of pe 4 is not a
        // measurement, it is a placeholder, and it is mildly *reducing*:
        // left at 4 the engine turned copper(II) into copper(I) and reduced
        // sulfate to sulfide — real chemistry that needs bacteria and
        // geological time, reported as the contents of a beaker. Fixing pe
        // from atmospheric oxygen gives pe ≈ 19.6 at pH 1, copper stays
        // copper(II) and sulfate stays sulfate, which is what is in the
        // glass. log P(O2) = −0.68 is the atmosphere's own partial
        // pressure, and it is the same reservoir the gas-venting path
        // already equilibrates against.
        // Only where the database defines the phase. pitzer.dat is a
        // major-ion brine model and carries no O2(g) at all — asking it for
        // one is an error, and it has no redox chemistry to get wrong
        // anyway.
        None if vessel.uses_atmospheric_reservoir()
            && derived::index_for(db_tag).has_phase(ATMOSPHERIC_OXYGEN) =>
        {
            writeln!(
                input,
                "    pe        4  {ATMOSPHERIC_OXYGEN}  {ATMOSPHERIC_LOG_PO2}"
            )
            .unwrap()
        }
        None => {}
    }
    // Merge totals that lose their valence tag: Fe(2) and Fe(3) both become
    // Fe, and PHREEQC takes one line per element.
    let mut totals: Vec<(String, f64)> = Vec::new();
    for (el, moles) in &problem.totals {
        let base = el.split('(').next().unwrap_or(el);
        let key = match couple {
            Some(_) if FAST_REDOX.contains(&base) => base.to_string(),
            _ => el.clone(),
        };
        match totals.iter_mut().find(|(k, _)| *k == key) {
            Some(entry) => entry.1 += moles,
            None => totals.push((key, *moles)),
        }
    }
    for (el, moles) in &totals {
        writeln!(input, "    {el} {:.12e}", moles / problem.kgw).unwrap();
    }
    if !problem.phases.is_empty() {
        writeln!(input, "EQUILIBRIUM_PHASES 1").unwrap();
        for (phase, moles, target_si) in &problem.phases {
            // The candidate list is database-blind; the input is not.
            // `posed_phase` is that reconciliation, and a `None` is the
            // case the honesty note keeps.
            let Some(posed) = posed_phase(phase, db_tag) else {
                continue;
            };
            writeln!(input, "    {posed} {target_si} {moles:.12e}").unwrap();
        }
    }
    if !problem.gases.is_empty() {
        // GAS_PHASE's pressure initializer uses the database's gas EOS.
        // P=nRT/V therefore supplies the wrong finite moles when a phase
        // carries Peng-Robinson parameters (CO2 in pitzer.dat). Raw native
        // gas state accepts owned component moles directly and still lets
        // the solver apply its reviewed EOS during equilibrium.
        let volume = match vessel.headspace {
            Headspace::Sealed { volume } | Headspace::PressureControlled { volume, .. } => volume.0,
            _ => unreachable!("gas components require a material-closed headspace"),
        };
        let amounts: Vec<_> = problem
            .gases
            .iter()
            .map(|(phase, species, _)| {
                let moles: f64 = vessel
                    .contents
                    .iter()
                    .filter(|p| p.phase == Phase::Gas && p.species.0 == *species)
                    .map(|p| p.moles.0)
                    .sum();
                (phase, moles)
            })
            .collect();
        let total_moles: f64 = amounts.iter().map(|(_, moles)| moles).sum();
        // total_p is required by the raw reader; it is a starting estimate,
        // not an inventory initializer or a reported physical prediction.
        let pressure_guess: f64 = problem.gases.iter().map(|(_, _, p)| p).sum();
        writeln!(input, "GAS_PHASE_RAW 1").unwrap();
        writeln!(input, "    -type 1").unwrap(); // native GP_VOLUME
        writeln!(input, "    -total_p {pressure_guess:.12e}").unwrap();
        writeln!(input, "    -volume {volume:.12e}").unwrap();
        // The native raw-state temperature is Kelvin, unlike GAS_PHASE.
        writeln!(input, "    -temperature {:.12e}", vessel.temperature.0).unwrap();
        writeln!(input, "    -new_def 0").unwrap();
        writeln!(input, "    -solution_equilibria 0").unwrap();
        writeln!(input, "    -total_moles {total_moles:.12e}").unwrap();
        for (phase, moles) in amounts {
            writeln!(input, "    -component {phase}\n        -moles {moles:.12e}").unwrap();
        }
        writeln!(input, "USE gas_phase 1").unwrap();
    }

    if !problem.surfaces.is_empty() {
        let total_mass: f64 = problem.surfaces.iter().map(|surface| surface.mass.0).sum();
        let total_area: f64 = problem
            .surfaces
            .iter()
            .map(|surface| surface.mass.0 * surface.specific_area_m2_per_g)
            .sum();
        let specific_area = total_area / total_mass;
        let strong_capacity: f64 = problem
            .surfaces
            .iter()
            .map(|surface| surface.strong_capacity.0)
            .sum();
        let weak_capacity: f64 = problem
            .surfaces
            .iter()
            .map(|surface| surface.weak_capacity.0)
            .sum();
        writeln!(input, "SURFACE 1").unwrap();
        writeln!(
            input,
            "    Hfo_sOH {:.12e} {:.12e} {:.12e}",
            strong_capacity, specific_area, total_mass
        )
        .unwrap();
        writeln!(input, "    Hfo_wOH {:.12e}", weak_capacity).unwrap();
        writeln!(input, "    -equilibrate 1").unwrap();
    }
    if !problem.exchanges.is_empty() {
        writeln!(input, "EXCHANGE 1").unwrap();
        for ion in [
            ExchangeIon::Hydrogen,
            ExchangeIon::Sodium,
            ExchangeIon::Calcium,
            ExchangeIon::Magnesium,
        ] {
            let moles: f64 = problem
                .exchanges
                .iter()
                .map(|exchange| exchange.bound(ion).0)
                .sum();
            if moles > TRACE {
                writeln!(
                    input,
                    "    {} {:.12e}",
                    phreeqc_exchange_species(ion),
                    moles
                )
                .unwrap();
            }
        }
    }
    if let Some(solid_solution) = problem.solid_solutions.first() {
        writeln!(input, "SOLID_SOLUTIONS 1").unwrap();
        writeln!(input, "    Kerotakis_CaSrCO3").unwrap();
        match solid_solution.model {
            SolidSolutionModel::AragoniteStrontianite => {
                let calcium = solid_solution
                    .moles_of(SolidSolutionComponent::CalciumCarbonate)
                    .0;
                let strontium = solid_solution
                    .moles_of(SolidSolutionComponent::StrontiumCarbonate)
                    .0;
                writeln!(input, "        -comp1 Aragonite {calcium:.12e}").unwrap();
                writeln!(input, "        -comp2 Strontianite {strontium:.12e}").unwrap();
                // PHREEQC example 10's dimensionless Guggenheim parameters
                // for the reviewed non-ideal Ca(x)Sr(1-x)CO3 pair.
                writeln!(input, "        -Gugg_nondimensional 3.43 -1.82").unwrap();
            }
        }
    }
    writeln!(input, "SELECTED_OUTPUT").unwrap();
    writeln!(input, "    -reset    false").unwrap();
    // Default selected-output prints ~5 significant digits, which leaks into
    // the mass balance; high precision prints 12.
    writeln!(input, "    -high_precision true").unwrap();
    writeln!(input, "    -ph       true").unwrap();
    // pe is computed on every solve and was being discarded. It is the
    // redox axis — the electron analogue of pH — and without it the lab
    // cannot say anything about oxidation and reduction at all.
    writeln!(input, "    -pe       true").unwrap();
    writeln!(input, "    -ionic_strength true").unwrap();
    writeln!(input, "    -water    true").unwrap();
    // Phase equilibrium needs numerical activity, not the four-digit report.
    writeln!(input, "    -activities H2O").unwrap();
    let mut elements: Vec<String> = problem.elements.to_vec();
    // Per-oxidation-state totals for anything redox-active: the split
    // between them is what a redox experiment is *about*.
    elements.extend(valence_totals(problem, db_tag));
    // Element totals, plus the per-oxidation-state split for any redox
    // element present. The split is the observable of every redox
    // experiment and PHREEQC will report it if asked by name.
    let mut totals: Vec<String> = Vec::new();
    for e in elements
        .iter()
        .map(|e| e.to_string())
        .chain(valence_totals(problem, db_tag))
    {
        if !totals.contains(&e) {
            totals.push(e);
        }
    }
    if !totals.is_empty() {
        writeln!(input, "    -totals   {}", totals.join(" ")).unwrap();
    }
    if !problem.phases.is_empty() {
        let phases: Vec<&str> = problem.phases.iter().map(|(p, ..)| p.as_str()).collect();
        writeln!(input, "    -equilibrium_phases {}", phases.join(" ")).unwrap();
    }
    if !problem.gases.is_empty() {
        let gases: Vec<&str> = problem
            .gases
            .iter()
            .map(|(phase, ..)| phase.as_str())
            .collect();
        writeln!(input, "    -gases    {}", gases.join(" ")).unwrap();
    }
    // One `-molalities` line, not one per reason to want species columns.
    // Three separate needs had grown three separate lines in the same
    // block, and whether PHREEQC accumulates or replaces a repeated
    // identifier is not something this input should be resting on: a
    // vessel with both a surface and an exchanger would have been staking
    // its readback on the answer. Columns are read by name, so a single
    // combined list is right under either reading.
    // Heat and catalyst gates need numerical output, not the rounded text
    // report used for human-readable speciation.
    let mut molalities: Vec<&str> = vec!["H+", "OH-"];
    if !problem.surfaces.is_empty() {
        molalities.extend(["Hfo_sOZn+", "Hfo_wOZn+", "Hfo_wSO4-", "Hfo_wOHSO4-2"]);
        if db_tag == "minteq.v4" {
            molalities.extend(["Hfo_sSO4-", "Hfo_sOHSO4-2"]);
        }
    }
    if !problem.exchanges.is_empty() {
        molalities.extend(["HX", "NaX", "CaX2", "MgX2"]);
    }
    // The protonation split needs the individual species, not the state
    // total: N(-3) is one number and ammonia-or-ammonium is the question
    // it cannot answer. Asked for only when that state is actually in the
    // problem, so no other input grows a column.
    for total in &totals {
        for (species, _) in derived::protonation_split(total).unwrap_or(&[]) {
            if !molalities.contains(species) {
                molalities.push(species);
            }
        }
    }
    if !molalities.is_empty() {
        writeln!(input, "    -molalities {}", molalities.join(" ")).unwrap();
    }
    if !problem.solid_solutions.is_empty() {
        writeln!(input, "    -solid_solutions Aragonite Strontianite").unwrap();
    }
    append_particle_census(&mut input);
    writeln!(input, "END").unwrap();
    input
}

/// Numerical aqueous particle census on the native solvent-mass basis.
/// USGS SYS("aq", ...) enumerates aqueous species, excluding solvent water,
/// surface/exchange species and held solids (pinned IPhreeqc system_total_aq).
/// A scalar avoids using rounded report molalities or a second engine solve.
fn append_particle_census(input: &mut String) {
    input.push_str("USER_PUNCH 1\n    -headings particle_molality\n    -start\n10 particles = SYS(\"aq\", count, names$, types$, amounts)\n20 PUNCH particles / TOT(\"water\")\n    -end\n");
}

/// Require one finite nonnegative numerical particle census from a new solve.
/// Rounded report rows remain presentation data, never this scalar's source.
fn numerical_particle_molality(rows: &[Vec<String>]) -> Result<f64, SolveError> {
    let invalid = || {
        SolveError::NotConverged {
        reason: Some(Box::new(AqueousReason::SelectedNumberInvalid.phrase())),
        solver: "phreeqc-aqueous".to_string(),
        detail: "numerical aqueous particle molality is missing or invalid in the final selected-output row".to_string(),
    }
    };
    let header = rows.first().ok_or_else(invalid)?;
    let columns: Vec<_> = header
        .iter()
        .enumerate()
        .filter_map(|(index, name)| (name == "particle_molality").then_some(index))
        .collect();
    if columns.len() != 1 || rows.len() < 2 {
        return Err(invalid());
    }
    rows.last()
        .and_then(|row| row.get(columns[0]))
        .and_then(|cell| cell.parse::<f64>().ok())
        .filter(|value| value.is_finite() && *value >= 0.0)
        .ok_or_else(invalid)
}

/// Parse the report's "Saturation indices" block into (phase, SI) pairs.
///
/// This is the one place PHREEQC volunteers information about phases we did
/// *not* ask about: it lists every mineral the loaded database could build
/// from the elements in solution, saturated or not. That makes it exactly
/// the right source for the honesty question "is the solution you are
/// looking at one that could not survive contact with a phase this lab
/// cannot name?".
///
/// Block shape, stable across PHREEQC 3.x:
/// `Phase   SI**  log IAP   log K(298 K, 1 atm)  [formula]`
fn parse_saturation_indices(output: &str) -> Vec<(String, f64)> {
    let Some(start) = output.rfind("Saturation indices") else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for line in output[start..].lines().skip(1) {
        let trimmed = line.trim();
        if trimmed.starts_with("-----") && !out.is_empty() {
            break;
        }
        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        // name, SI, log IAP, log K — headers and rules have neither shape.
        if tokens.len() < 4 || tokens[0] == "Phase" {
            continue;
        }
        let Ok(si) = tokens[1].parse::<f64>() else {
            continue;
        };
        out.push((tokens[0].to_string(), si));
    }
    out
}

/// Keep the report as presentation, but obtain the phase solver's water
/// activity and particle census from final numerical selected output. Four
/// significant report digits create a discontinuous liquidus and freeze/melt
/// cycles, even when every individual phase transfer conserves enthalpy.
/// Native direct, recast, redox and MIX results all pass this boundary.
fn numerical_species_distribution(
    report: &str,
    rows: &[Vec<String>],
) -> Result<Vec<SpeciesDetail>, SolveError> {
    let mut species = parse_species_distribution(report);
    apply_numerical_water_activity(&mut species, rows)?;
    numerical_particle_molality(rows)?;
    Ok(species)
}

/// A missing column is allowed for an imported, older answer; a present but
/// malformed numerical answer must never be replaced by a rounded report.
fn selected_nonnegative_molality(
    rows: &[Vec<String>],
    name: &str,
) -> Result<Option<f64>, SolveError> {
    let invalid = || SolveError::NotConverged {
        reason: Some(Box::new(AqueousReason::SelectedNumberInvalid.phrase())),
        solver: "phreeqc-aqueous".to_string(),
        detail: format!("invalid final numerical {name} molality"),
    };
    let header = rows.first().ok_or_else(invalid)?;
    let mut indices = header
        .iter()
        .enumerate()
        .filter_map(|(index, column)| (column == name).then_some(index));
    let Some(index) = indices.next() else {
        return Ok(None);
    };
    if indices.next().is_some() {
        return Err(invalid());
    }
    let amount = rows
        .last()
        .filter(|_| rows.len() >= 2)
        .and_then(|row| row.get(index))
        .and_then(|cell| cell.parse::<f64>().ok())
        .filter(|amount| amount.is_finite() && *amount >= 0.0)
        .ok_or_else(invalid)?;
    Ok(Some(amount))
}

/// Persist meters from the same precise native row as the accepted solve.
/// The inventory's solute charge and physical portions remain untouched.
fn refresh_measured_coordinates(
    vessel: &mut Vessel,
    rows: &[Vec<String>],
    report_species: &[SpeciesDetail],
    solvent_kg: f64,
) -> Result<(), SolveError> {
    if !solvent_kg.is_finite() || solvent_kg <= 0.0 {
        return Err(SolveError::NotConverged {
            reason: Some(Box::new(AqueousReason::SolventMassInvalid.phrase())),
            solver: "phreeqc-aqueous".to_string(),
            detail: "invalid numerical solvent mass for measured coordinates".to_string(),
        });
    }
    let hydroxide = selected_nonnegative_molality(rows, "m_OH-")?.map_or_else(
        || free_hydroxide_moles(Some(report_species), solvent_kg),
        |molality| molality * solvent_kg,
    );
    let proton = selected_nonnegative_molality(rows, "m_H+")?.map_or_else(
        || measured_species_moles(Some(report_species), "H+", solvent_kg),
        |molality| molality * solvent_kg,
    );
    let co2 = selected_nonnegative_molality(rows, "m_CO2")?;
    let hydrated_co2 = selected_nonnegative_molality(rows, "m_H2CO3")?;
    let co2_molality = if co2.is_some() || hydrated_co2.is_some() {
        co2.unwrap_or(0.0) + hydrated_co2.unwrap_or(0.0)
    } else {
        // Older imported rows may lack both columns. Their report remains
        // the only available diagnostic, with its original precision limit.
        (measured_species_moles(Some(report_species), "CO2", solvent_kg)
            + measured_species_moles(Some(report_species), "H2CO3", solvent_kg))
            / solvent_kg
    };
    if !hydroxide.is_finite()
        || hydroxide < 0.0
        || !proton.is_finite()
        || proton < 0.0
        || !co2_molality.is_finite()
        || co2_molality < 0.0
    {
        return Err(SolveError::NotConverged {
            reason: Some(Box::new(AqueousReason::MeasuredCoordinateInvalid.phrase())),
            solver: "phreeqc-aqueous".to_string(),
            detail: "nonfinite measured aqueous coordinates".to_string(),
        });
    }
    let pressure = kerotakis_core::properties::henry_lookup("CO2").map(|coeff| {
        co2_molality / kerotakis_core::properties::henry_at_t(coeff, vessel.temperature.0).value
    });
    if pressure.is_some_and(|value| !value.is_finite() || value < 0.0) {
        return Err(SolveError::NotConverged {
            reason: Some(Box::new(AqueousReason::Co2PressureInvalid.phrase())),
            solver: "phreeqc-aqueous".to_string(),
            detail: "invalid numerical CO2 pressure".to_string(),
        });
    }
    vessel.free_hydroxide = hydroxide;
    vessel.free_proton = proton;
    vessel.co2_partial_pressure_atm = pressure;
    Ok(())
}

fn apply_numerical_water_activity(
    species: &mut [SpeciesDetail],
    rows: &[Vec<String>],
) -> Result<(), SolveError> {
    let invalid = || SolveError::NotConverged {
        reason: Some(Box::new(AqueousReason::WaterActivityInvalid.phrase())),
        solver: "phreeqc-aqueous".to_string(),
        detail: "numerical H2O activity is missing or invalid in the final selected-output row"
            .to_string(),
    };
    let header = rows.first().ok_or_else(invalid)?;
    let columns: Vec<usize> = header
        .iter()
        .enumerate()
        .filter_map(|(index, name)| (name == "la_H2O").then_some(index))
        .collect();
    if columns.len() != 1 || rows.len() < 2 {
        return Err(invalid());
    }
    let log_activity: f64 = rows
        .last()
        .and_then(|row| row.get(columns[0]))
        .and_then(|cell| cell.parse().ok())
        .filter(|value: &f64| value.is_finite())
        .ok_or_else(invalid)?;
    let activity = 10.0_f64.powf(log_activity);
    if !activity.is_finite() || activity <= 0.0 {
        return Err(invalid());
    }
    let water = species
        .iter_mut()
        .find(|species| species.name == "H2O")
        .ok_or_else(invalid)?;
    water.activity = activity;
    Ok(())
}

/// Parse the last "Distribution of species" block of a PHREEQC output
/// report into (name, molality, activity) triples, molality > 1e-9,
/// descending. The block's shape is stable across PHREEQC 3.x: a header,
/// element-total lines (2 columns), and species lines (>= 6 columns:
/// name, molality, activity, log m, log a, log gamma[, volume]).
pub(crate) fn parse_species_distribution(output: &str) -> Vec<SpeciesDetail> {
    let Some(start) = output.rfind("Distribution of species") else {
        return Vec::new();
    };
    let mut result: Vec<SpeciesDetail> = Vec::new();
    for line in output[start..].lines().skip(1) {
        let trimmed = line.trim();
        // The next report section begins with a dashed rule.
        if trimmed.starts_with("-----") {
            break;
        }
        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        if tokens.len() < 6 {
            continue;
        }
        let (Ok(molality), Ok(activity)) = (tokens[1].parse::<f64>(), tokens[2].parse::<f64>())
        else {
            continue;
        };
        // Log columns must also parse, or this is a header/stray line.
        if tokens[3].parse::<f64>().is_err() {
            continue;
        }
        // A species appears once per element section it contains (AgCl is
        // listed under both Ag and Cl); keep it once.
        if molality > 1e-9 && !result.iter().any(|r| r.name == tokens[0]) {
            result.push(SpeciesDetail {
                name: tokens[0].to_string(),
                molality,
                activity,
            });
        }
    }
    result.sort_by(|a, b| b.molality.total_cmp(&a.molality));
    result
}

#[cfg(test)]
mod numerical_water_activity_tests {
    use super::{
        numerical_species_distribution, ExternalGas, ExternalGasKind, PhreeqcEquilibrator, Problem,
        SecondaryDisposition, SecondaryFirstHint,
    };
    use kerotakis_core::{Moles, Phase, Portion, SpeciesId, Vessel, VesselId};

    const REPORT: &str = "Distribution of species\nH2O 5.551e+01 9.500e-01 1.744 -0.022 0.0\nNa+ 1.000e+00 0.7000 0 -0.15 -0.15\n-----\n";

    fn rows(value: &str) -> Vec<Vec<String>> {
        vec![
            vec!["la_H2O".into(), "particle_molality".into()],
            vec![value.into(), "1.0".into()],
        ]
    }

    #[test]
    fn numerical_particle_census_uses_final_precise_native_value() {
        let mut selected = rows("-0.1");
        selected.push(vec!["-0.2".into(), "1.234567891234".into()]);
        assert_eq!(
            super::numerical_particle_molality(&selected).unwrap(),
            1.234567891234
        );
        assert_eq!(
            super::parse_species_distribution(REPORT)
                .iter()
                .filter(|species| species.name != "H2O")
                .map(|species| species.molality)
                .sum::<f64>(),
            1.0
        );
        assert_eq!(
            super::numerical_particle_molality(&[
                vec!["particle_molality".into()],
                vec!["0".into()]
            ])
            .unwrap(),
            0.0
        );
    }

    #[test]
    fn invalid_particle_census_refuses_without_report_fallback() {
        for bad in ["NaN", "inf", "-inf", "-0.001", "invalid"] {
            let mut selected = rows("-0.1");
            selected[1][1] = bad.into();
            assert!(
                numerical_species_distribution(REPORT, &selected).is_err(),
                "{bad}"
            );
        }
        for selected in [
            vec![],
            vec![vec!["particle_molality".into()]],
            vec![vec!["particle_molality".into()], vec![]],
            vec![
                vec!["particle_molality".into(), "particle_molality".into()],
                vec!["1".into(), "2".into()],
            ],
            vec![vec!["la_H2O".into()], vec!["0".into()]],
        ] {
            assert!(super::numerical_particle_molality(&selected).is_err());
        }
    }

    #[test]
    fn numerical_activity_keeps_distinct_states_with_the_same_rounded_report() {
        for activity in [0.950001234567_f64, 0.950004567891_f64] {
            let precise =
                numerical_species_distribution(REPORT, &rows(&activity.log10().to_string()))
                    .expect("numerical water activity");
            let water = precise.iter().find(|s| s.name == "H2O").unwrap();
            assert!((water.activity - activity).abs() < 2e-15);
            assert_eq!(water.molality, 55.51);
            assert_eq!(
                precise.iter().find(|s| s.name == "Na+").unwrap().activity,
                0.7
            );
        }
    }

    #[test]
    fn numerical_activity_uses_the_final_native_solution_row() {
        let mut selected = rows("-0.1");
        selected.push(vec!["-0.2".into(), "1.0".into()]);
        let result = numerical_species_distribution(REPORT, &selected).unwrap();
        assert!(
            (result.iter().find(|s| s.name == "H2O").unwrap().activity - 10.0_f64.powf(-0.2)).abs()
                < 2e-15
        );
    }

    #[test]
    fn measured_coordinates_use_final_numerical_row_below_report_floor() {
        let species = super::parse_species_distribution(REPORT);
        let mut first = Vessel::new(VesselId(0), "numerical meters");
        let mut second = first.clone();
        let header = vec!["m_H+", "m_OH-", "m_CO2", "m_H2CO3"]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();
        let rows = |carbon: &str| {
            vec![
                header.clone(),
                vec!["1.23456789e-8", "2.34567891e-7", carbon, "0"]
                    .into_iter()
                    .map(str::to_string)
                    .collect(),
            ]
        };
        super::refresh_measured_coordinates(&mut first, &rows("4.12e-13"), &species, 0.1).unwrap();
        super::refresh_measured_coordinates(&mut second, &rows("8.24e-13"), &species, 0.1).unwrap();
        assert!((first.free_proton - 1.23456789e-9).abs() < 1e-23);
        assert!((first.free_hydroxide - 2.34567891e-8).abs() < 1e-22);
        let pressure = first.co2_partial_pressure_atm.unwrap();
        assert!(pressure > 0.0, "sub-report-floor CO2 must drive a clock");
        assert!((second.co2_partial_pressure_atm.unwrap() / pressure - 2.0).abs() < 1e-12);
        assert_eq!(first.contents, second.contents);
    }

    #[test]
    fn measured_coordinates_reject_invalid_present_cells_but_allow_legacy_absence() {
        let species = super::parse_species_distribution(REPORT);
        let mut vessel = Vessel::new(VesselId(0), "legacy meters");
        super::refresh_measured_coordinates(
            &mut vessel,
            &[vec!["unrelated".into()], vec!["0".into()]],
            &species,
            0.1,
        )
        .unwrap();
        assert_eq!(vessel.co2_partial_pressure_atm, Some(0.0));
        let mut invalid_legacy = species.clone();
        let invalid_hydroxide = invalid_legacy
            .iter_mut()
            .find(|species| species.name == "Na+")
            .unwrap();
        invalid_hydroxide.name = "OH-".to_string();
        invalid_hydroxide.molality = -1e-7;
        assert!(super::refresh_measured_coordinates(
            &mut vessel,
            &[vec!["unrelated".into()], vec!["0".into()]],
            &invalid_legacy,
            0.1,
        )
        .is_err());
        for bad in ["NaN", "inf", "-1", "not-a-number"] {
            let rows = vec![vec!["m_CO2".into()], vec![bad.into()]];
            assert!(
                super::refresh_measured_coordinates(&mut vessel, &rows, &species, 0.1).is_err(),
                "{bad}"
            );
        }
    }

    #[test]
    fn pure_solvent_floor_is_not_misclassified_as_unsupported_or_queried() {
        let mut solver = PhreeqcEquilibrator::new().expect("equilibrator fixture");
        let vessel = Vessel::new(VesselId(0), "pure solvent selector fixture");
        let pure_solvent = Problem {
            kgw: 0.1,
            solvent_only: true,
            clock_co2_applied_mol: 0.0,
            totals: vec![],
            phases: vec![],
            gases: vec![],
            external_gases: vec![],
            surfaces: vec![],
            exchanges: vec![],
            solid_solutions: vec![],
            elements: vec![],
        };
        let no_readback = |_column: &str| None;

        let floor = solver.solvent_activity_second_opinion(
            &vessel,
            &pure_solvent,
            "wateq4f",
            &no_readback,
            SecondaryFirstHint::Auto,
        );
        assert_eq!(floor.disposition, SecondaryDisposition::BelowInputFloor);
        assert_eq!(solver.engine_calls(), 0);
        assert_eq!(solver.cache_hits(), 0);
        assert_eq!(solver.solvent_activity_engine_calls, 0);
        assert_eq!(solver.solvent_activity_cache_hits, 0);

        let mut interface = pure_solvent.clone();
        interface
            .surfaces
            .push(kerotakis_core::vessel::SurfaceSites {
                label: "actual interface ownership".into(),
                model: kerotakis_core::SurfaceModel::HydrousFerricOxide,
                mass: kerotakis_core::Grams(0.09),
                specific_area_m2_per_g: 600.0,
                strong_capacity: Moles(5e-6),
                weak_capacity: Moles(2e-4),
                occupancy: vec![],
                water_release: Moles(0.0),
            });
        let unsupported_interface = solver.solvent_activity_second_opinion(
            &vessel,
            &interface,
            "wateq4f",
            &no_readback,
            SecondaryFirstHint::Auto,
        );
        assert_eq!(
            unsupported_interface.disposition,
            SecondaryDisposition::Declined(super::SecondaryDecline::Unsupported)
        );

        let mut dose = pure_solvent;
        dose.external_gases.push(ExternalGas {
            phase: "CO2(g)".into(),
            species: "CO2".into(),
            initial_moles: 1e-3,
            kind: ExternalGasKind::Dose,
        });
        let unsupported_dose = solver.solvent_activity_second_opinion(
            &vessel,
            &dose,
            "wateq4f",
            &no_readback,
            SecondaryFirstHint::Auto,
        );
        assert_eq!(
            unsupported_dose.disposition,
            SecondaryDisposition::Declined(super::SecondaryDecline::Unsupported)
        );
        assert_eq!(solver.engine_calls(), 0);
        assert_eq!(solver.cache_hits(), 0);
        assert_eq!(solver.solvent_activity_engine_calls, 0);
        assert_eq!(solver.solvent_activity_cache_hits, 0);
    }

    #[test]
    fn protonation_rebuild_preserves_positive_trace_share_of_authoritative_total() {
        use std::collections::BTreeMap;

        let total_n = 0.02;
        let nh3_fraction = 2.5e-11;
        let mut vessel = Vessel::new(VesselId(0), "water and ammonium chloride");
        vessel.contents.push(Portion {
            species: SpeciesId::new("water"),
            moles: Moles(55.508),
            phase: Phase::Liquid,
        });
        vessel.deposit(SpeciesId::new("NH4+"), Moles(total_n), Phase::Aqueous);
        vessel.deposit(SpeciesId::new("Cl-"), Moles(total_n), Phase::Aqueous);
        let problem = Problem {
            kgw: 1.0,
            solvent_only: false,
            clock_co2_applied_mol: 0.0,
            totals: vec![("N(-3)".into(), total_n), ("Cl".into(), total_n)],
            phases: vec![],
            gases: vec![],
            external_gases: vec![],
            surfaces: vec![],
            exchanges: vec![],
            solid_solutions: vec![],
            elements: vec!["Cl".into(), "N".into()],
        };
        let new_ions = [("N(-3)".into(), total_n), ("Cl".into(), total_n)];
        let protonation = BTreeMap::from([(
            "N(-3)".to_owned(),
            vec![("NH3", nh3_fraction), ("NH4+", 1.0 - nh3_fraction)],
        )]);

        let (_, contents) = PhreeqcEquilibrator::rebuild_contents_and_events(
            &vessel,
            &problem,
            &[],
            1.0,
            &new_ions,
            &[],
            &[],
            &[],
            &protonation,
        );
        let nh3 = contents
            .iter()
            .find(|portion| portion.species.0 == "NH3")
            .expect("positive ammonia share is retained")
            .moles
            .0;
        let reconstructed_n: f64 = contents
            .iter()
            .filter(|portion| matches!(portion.species.0.as_str(), "NH3" | "NH4+"))
            .map(|portion| portion.moles.0)
            .sum();

        assert!((nh3 - 5e-13).abs() < 1e-27);
        assert!((reconstructed_n - total_n).abs() < 1e-17);
    }

    #[test]
    fn cold_and_icy_inputs_preserve_both_source_solvent_budgets_and_temperature() {
        use kerotakis_core::{Kelvin, Moles, Phase, SpeciesId, Vessel, VesselId};
        for (temperature, ice) in [(260.12345678912345, false), (285.12345678912345, true)] {
            let mut cold = Vessel::new(VesselId(0), "phase-sensitive input");
            cold.temperature = Kelvin(temperature);
            cold.deposit(
                SpeciesId::new("water"),
                Moles(1.234_567_891_234_568),
                Phase::Liquid,
            );
            cold.deposit(SpeciesId::new("NaCl"), Moles(0.017), Phase::Solid);
            if ice {
                cold.deposit(SpeciesId::new("water"), Moles(0.1), Phase::Solid);
            }
            let mut warm = Vessel::new(VesselId(1), "warm source");
            warm.temperature = Kelvin(298.1500375477956);
            warm.deposit(
                SpeciesId::new("water"),
                Moles(2.345678912345678),
                Phase::Liquid,
            );
            warm.deposit(SpeciesId::new("NaCl"), Moles(0.011), Phase::Solid);
            let pa = super::partition(&cold).unwrap();
            let pb = super::partition(&warm).unwrap();
            let direct = super::build_input(&cold, &pa, "wateq4f");
            let fields = |input: &str, field: &str| -> Vec<f64> {
                input
                    .lines()
                    .filter_map(|line| line.trim().strip_prefix(field))
                    .map(|value| value.trim().parse().unwrap())
                    .collect()
            };
            assert_eq!(
                fields(&direct, "temp "),
                vec![cold.temperature.to_celsius()]
            );
            assert_eq!(fields(&direct, "water "), vec![pa.kgw]);
            let mut combined = cold.clone();
            for portion in &warm.contents {
                combined.deposit(portion.species.clone(), portion.moles, portion.phase);
            }
            let merged = super::partition(&combined).unwrap();
            let mixed =
                super::build_mix_input(&cold, &pa, &warm, &pb, 1.0, 1.0, "wateq4f", &merged, false);
            assert_eq!(
                fields(&mixed, "temp "),
                vec![cold.temperature.to_celsius(), warm.temperature.to_celsius()]
            );
            assert_eq!(fields(&mixed, "water "), vec![pa.kgw, pb.kgw]);
            assert!(super::phase_input_precision(&cold));
            assert!(!super::phase_input_precision(&warm));
            // Receiver-only sensitivity must preserve both warm inputs too.
            let mut warm_combined = warm.clone();
            for portion in &warm.contents {
                warm_combined.deposit(portion.species.clone(), portion.moles, portion.phase);
            }
            let warm_merged = super::partition(&warm_combined).unwrap();
            let receiver_cold = super::build_mix_input(
                &warm,
                &pb,
                &warm,
                &pb,
                1.0,
                1.0,
                "wateq4f",
                &warm_merged,
                true,
            );
            assert_eq!(
                fields(&receiver_cold, "temp "),
                vec![warm.temperature.to_celsius(); 2]
            );
            assert_eq!(fields(&receiver_cold, "water "), vec![pb.kgw; 2]);
            let ordinary = super::build_mix_input(
                &warm,
                &pb,
                &warm,
                &pb,
                1.0,
                1.0,
                "wateq4f",
                &warm_merged,
                false,
            );
            assert_eq!(
                fields(&ordinary, "temp "),
                vec![
                    format!("{:.4}", warm.temperature.to_celsius())
                        .parse::<f64>()
                        .unwrap();
                    2
                ]
            );
            assert_eq!(fields(&ordinary, "water "), vec![pb.kgw; 2]);
        }
    }

    fn cache_entry(value: &str) -> super::CacheEntry {
        super::CacheEntry {
            key: "test-current-input\n    -activities H2O\n    -headings particle_molality\n"
                .into(),
            rows: rows(value),
            species: super::parse_species_distribution(REPORT),
            saturation: vec![],
            redox_adjusted: false,
            pe_undetermined: false,
            pe_authority: super::PeAuthority::Unwitnessed,
        }
    }

    #[test]
    fn current_cache_cannot_bypass_numeric_particle_validation() {
        let mut solver = super::PhreeqcEquilibrator::new().unwrap();
        for bad in ["NaN", "inf", "-1", "invalid"] {
            let mut entry = cache_entry("-0.1");
            entry.rows[1][1] = bad.into();
            assert_eq!(
                solver.import_cache(super::CacheData {
                    entries: vec![entry],
                    ..Default::default()
                }),
                0
            );
        }
        let mut entry = cache_entry("-0.1");
        entry.rows = vec![vec!["la_H2O".into()], vec!["-0.1".into()]];
        assert_eq!(
            solver.import_cache(super::CacheData {
                entries: vec![entry],
                ..Default::default()
            }),
            0
        );
    }

    #[test]
    fn imported_current_cache_normalizes_water_from_numerical_rows() {
        let mut solver = super::PhreeqcEquilibrator::new().unwrap();
        let activity = 0.950001234567_f64;
        let data = super::CacheData {
            entries: vec![cache_entry(&activity.log10().to_string())],
            ..Default::default()
        };
        assert_eq!(solver.import_cache(data), 1);
        let exported = solver.export_cache();
        let water = exported.entries[0]
            .species
            .iter()
            .find(|s| s.name == "H2O")
            .unwrap();
        assert!((water.activity - activity).abs() < 2e-15);
    }

    #[test]
    fn imported_current_cache_rejects_missing_or_invalid_numerical_activity() {
        let mut solver = super::PhreeqcEquilibrator::new().unwrap();
        for value in ["NaN", "-999", "999"] {
            assert_eq!(
                solver.import_cache(super::CacheData {
                    entries: vec![cache_entry(value)],
                    ..Default::default()
                }),
                0
            );
        }
        let mut entry = cache_entry("0");
        entry.rows = vec![];
        assert_eq!(
            solver.import_cache(super::CacheData {
                entries: vec![entry],
                ..Default::default()
            }),
            0
        );
    }

    #[test]
    fn missing_or_invalid_numerical_activity_refuses_instead_of_using_the_report() {
        for value in ["NaN", "inf", "-inf", "invalid", "999", "-999"] {
            assert!(
                numerical_species_distribution(REPORT, &rows(value)).is_err(),
                "{value}"
            );
        }
        for selected in [
            vec![],
            vec![vec!["la_H2O".into()]],
            vec![vec!["pH".into()], vec!["7".into()]],
            vec![vec!["la_H2O".into()], vec![]],
            vec![
                vec!["la_H2O".into(), "la_H2O".into()],
                vec!["0".into(), "0".into()],
            ],
        ] {
            assert!(numerical_species_distribution(REPORT, &selected).is_err());
        }
        assert!(numerical_species_distribution("", &rows("0")).is_err());
    }
}

#[cfg(test)]
mod oxidation_sum_tests {
    use super::{build_input, oxidation_sum, partition};
    use kerotakis_core::{Moles, Phase, SpeciesId, Vessel, VesselId};

    fn rows(header: &[&str], values: &[&str]) -> Vec<Vec<String>> {
        [header, values]
            .into_iter()
            .map(|row| row.iter().map(|s| (*s).to_string()).collect())
            .collect()
    }

    #[test]
    fn invalid_or_partial_native_ledgers_are_not_balances() {
        let columns = vec!["Fe(2)".into(), "Fe(3)".into()];
        for value in ["NaN", "inf", "-inf", "-0.001", "invalid", "1e308"] {
            assert_eq!(
                oxidation_sum(&rows(&["Fe(2)", "Fe(3)"], &[value, "0"]), &columns, 1.0),
                None,
                "{value}"
            );
        }
        for data in [
            vec![],
            vec![vec!["Fe(2)".into(), "Fe(3)".into()]],
            rows(&["pH"], &["7"]),
            rows(&["Fe(2)"], &["0.1"]),
            rows(&["Fe(2)", "Fe(3)"], &["0.1"]),
            rows(&["Fe(2)", "Fe(2)", "Fe(3)"], &["0.1", "0.2", "0.3"]),
        ] {
            assert_eq!(oxidation_sum(&data, &columns, 1.0), None, "{data:?}");
        }
        let data = rows(&["Fe(2)", "Fe(3)"], &["0.1", "0.2"]);
        for kg in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(oxidation_sum(&data, &columns, kg), None);
        }
        assert_eq!(oxidation_sum(&data, &[], 1.0), None);
        assert_eq!(
            oxidation_sum(&data, &["Fe(2)".into(), "Fe(2)".into()], 1.0),
            None
        );
        assert_eq!(
            oxidation_sum(&rows(&["Fe"], &["0.1"]), &["Fe".into()], 1.0),
            None
        );
    }

    #[test]
    fn valid_negative_oxidation_sum_and_zero_population_are_preserved() {
        let columns = vec!["S(-2)".into(), "S(6)".into()];
        let data = rows(&["S(-2)", "S(6)"], &["0.5", "0"]);
        assert_eq!(oxidation_sum(&data, &columns, 2.0), Some(-2.0));
        assert_eq!(
            oxidation_sum(&rows(&["S(-2)", "S(6)"], &["0", "0"]), &columns, 1.0),
            Some(0.0)
        );
    }

    #[test]
    fn equilibrium_input_is_independent_of_feed_order() {
        fn silver_chloride_feed(order: &[&str]) -> Vessel {
            let mut vessel = Vessel::new(VesselId(0), "feed order");
            vessel.deposit(SpeciesId::new("water"), Moles(16.65), Phase::Liquid);
            for species in order {
                vessel.deposit(SpeciesId::new(species), Moles(0.0012), Phase::Solid);
            }
            vessel
        }

        let forward = silver_chloride_feed(&["NaCl", "AgNO3"]);
        let reverse = silver_chloride_feed(&["AgNO3", "NaCl"]);
        let forward_problem = partition(&forward).expect("aqueous problem");
        let reverse_problem = partition(&reverse).expect("aqueous problem");
        assert_eq!(
            build_input(&forward, &forward_problem, "minteq.v4"),
            build_input(&reverse, &reverse_problem, "minteq.v4")
        );
    }
}

#[cfg(test)]
mod dataset_claim_tests {
    use super::{dataset_phrase, Locale};
    use kerotakis_core::Provenance;

    /// The recipe renders the sentence that used to be written out.
    ///
    /// `Provenance.dataset` is asserted verbatim by the dataset tests and
    /// filed verbatim by `tools/chemistry-audit`, so the English is a
    /// contract. These are the exact strings `dataset_name` built before
    /// it became a recipe.
    #[test]
    fn the_recipe_renders_the_english_it_replaced() {
        assert_eq!(
            dataset_phrase("wateq4f").render(Locale::EN),
            "wateq4f.dat plus USBM IC 9429 reference-temperature complexes, with the reviewed Sander HBr gas-uptake slice"
        );
        assert_eq!(
            dataset_phrase("minteq.v4").render(Locale::EN),
            "minteq.v4.dat plus reviewed lactate and USBM IC 9429 reference-temperature complexes, with the reviewed Sander HBr gas-uptake slice"
        );
        assert_eq!(
            dataset_phrase("pitzer").render(Locale::EN),
            "pitzer.dat, with the reviewed Sander HBr gas-uptake slice"
        );
    }

    /// The FILE comes back out of the sentence because it was put in a
    /// slot, not because the sentence begins with it.
    ///
    /// Both nestings, and the flat one: `dataset_file` descends into the
    /// nested clause for the two datasets this lab adds to, and reads the
    /// wrapper's own slot for the one it runs as vendored.
    #[test]
    fn the_file_name_survives_the_sentence_around_it() {
        for (tag, file) in [
            ("wateq4f", "wateq4f.dat"),
            ("minteq.v4", "minteq.v4.dat"),
            ("pitzer", "pitzer.dat"),
        ] {
            let provenance = Provenance::new(
                "PHREEQC (IPhreeqc, USGS)",
                dataset_phrase(tag),
                "model",
                Vec::new(),
                kerotakis_core::phrase::Phrase::bare("routing.test", "because"),
            );
            assert_eq!(provenance.dataset_file(), file, "{tag}");
        }
    }

    /// German gets the clauses and keeps the file name.
    #[test]
    fn german_translates_the_clauses_and_not_the_file() {
        let de = Locale::parse("de");
        let said = dataset_phrase("wateq4f").render(de);
        assert!(said.contains("wateq4f.dat"), "{said}");
        assert!(!said.contains("plus USBM"), "{said}");
        assert!(!said.contains("with the reviewed"), "{said}");
    }
}

#[cfg(test)]
mod routing_molality_tests {
    use super::{partition, potential_molality, Problem, MIN_SOLVENT_KG, WATER_MOLAR_MASS};
    use kerotakis_core::{Moles, Phase, SpeciesId, Vessel, VesselId};

    /// The estimate exactly as it stood before 2026-09-18: a fixed solute
    /// inventory over whatever solvent is left, with every solid counted
    /// as two ions per formula unit whatever the bench knows about it.
    ///
    /// Kept in the tests, not in the engine, so the two numbers can be put
    /// beside each other in the assertions below.
    fn unbounded_estimate(problem: &Problem) -> f64 {
        (problem.totals.iter().map(|(_, n)| n).sum::<f64>()
            + 2.0 * problem.phases.iter().map(|(_, n, _)| n).sum::<f64>()
            + 2.0
                * problem
                    .solid_solutions
                    .iter()
                    .map(|solid_solution| solid_solution.total_moles().0)
                    .sum::<f64>())
            / problem.kgw
    }

    fn estimate(vessel: &Vessel) -> (f64, f64, f64) {
        let problem = partition(vessel).expect("an aqueous problem");
        (
            problem.kgw,
            unbounded_estimate(&problem),
            potential_molality(&problem, vessel.temperature.0),
        )
    }

    /// The 2026-09-18 transcript, reduced to the two quantities the
    /// estimate actually reads: a lump of chalk, and the milligram of
    /// water a vessel has left after boiling dry.
    ///
    /// It printed *~71046.6 mol/kgw* one line above the solver's own
    /// *I = 0.0004 mol/kgw*, and three lines below chalk's own sentence
    /// saying it dissolves to 0.0013 g per 100 mL, "below anything a
    /// beaker would show". Those are two halves of one answer disagreeing
    /// by eight orders of magnitude about the same solid.
    #[test]
    fn a_beaker_boiled_dry_over_chalk_is_not_a_concentrated_solution() {
        let mut vessel = Vessel::new(VesselId(0), "boiled dry over chalk");
        // 0.68 mg of water — the transcript's own denominator, reached by
        // evaporation rather than by pouring.
        vessel.deposit(SpeciesId::new("water"), Moles(3.7502e-5), Phase::Liquid);
        vessel.deposit(SpeciesId::new("CaCO3"), Moles(0.024), Phase::Solid);

        let (kgw, before, after) = estimate(&vessel);
        eprintln!("boiled-dry chalk: kgw={kgw:.6e} before={before:.4} after={after:.6}");
        assert!(
            before > 50_000.0,
            "the defect reproduces: {before} mol/kgw over {kgw} kg of water"
        );
        assert!(
            after < 0.01,
            "and the estimate is now the same order as the solver's own ionic strength: {after}"
        );
        // Chalk contributes what a millilitre of water could hold and no
        // more: 0.0013 g/100 mL is about 1.3e-4 mol/kgw, doubled for two
        // ions per formula unit.
        assert!(
            (1e-4..1e-3).contains(&after),
            "the whole estimate is now chalk's saturation, not chalk's inventory: {after}"
        );
    }

    /// The case the pitzer route exists for is untouched — and it now
    /// reads a saturated brine rather than a dissolved mountain.
    ///
    /// Eight moles of salt in a kilogram of water is the vessel
    /// `Provenance::source_key`'s doc quotes at ~16.0 mol/kgw. That figure
    /// was the PESSIMISTIC reading: the registry had reviewed no
    /// solubility for halite, so the estimate counted every mole of the
    /// eight as if all of it could dissolve. Halite now carries the Earl
    /// of Berkeley's 1904 gravimetric figure, so the cap that already
    /// bounded chalk bounds this too, and the estimate is what a kilogram
    /// of water could actually hold: 6.13 mol/kgw, doubled for two ions
    /// per formula unit.
    ///
    /// THE ROUTING DECISION IS UNCHANGED AND THAT IS WHAT THIS TEST IS
    /// FOR. 12.26 is what the pitzer route exists for exactly as 16.0 was
    /// — an order of magnitude above the 1.0 where the Debye-Hückel
    /// datasets leave their validity domain.
    #[test]
    fn a_real_brine_still_reads_as_concentrated() {
        let mut vessel = Vessel::new(VesselId(0), "brine");
        vessel.deposit(SpeciesId::new("water"), Moles(55.51), Phase::Liquid);
        vessel.deposit(SpeciesId::new("NaCl"), Moles(8.0), Phase::Solid);

        let (kgw, before, after) = estimate(&vessel);
        eprintln!("brine: kgw={kgw:.6e} before={before:.4} after={after:.4}");
        assert!(
            (15.0..17.0).contains(&before),
            "the pessimistic reading is the ~16.0 mol/kgw the provenance doc quotes: {before}"
        );
        assert!(
            (12.0..13.0).contains(&after),
            "and the capped reading is what the water could hold: {after}"
        );
        assert!(
            after > 1.0,
            "either way it is routed as concentrated: {after}"
        );
    }

    /// And so is the millilitre probe `condense_supersaturated` documents
    /// — 0.1 mol of NaCl in exactly 1 mL — which is why the floor is one
    /// millilitre and not more.
    #[test]
    fn the_documented_millilitre_probe_is_unchanged() {
        let mut vessel = Vessel::new(VesselId(0), "one millilitre");
        // 1 mL of water, to the mole.
        vessel.deposit(
            SpeciesId::new("water"),
            Moles(1000.0 * MIN_SOLVENT_KG / WATER_MOLAR_MASS),
            Phase::Liquid,
        );
        vessel.deposit(SpeciesId::new("NaCl"), Moles(0.1), Phase::Solid);

        let (kgw, before, after) = estimate(&vessel);
        eprintln!("1 mL probe: kgw={kgw:.6e} before={before:.4} after={after:.4}");
        assert!(
            (kgw - MIN_SOLVENT_KG).abs() < 1e-6,
            "the probe really does sit on the floor: {kgw}"
        );
        // Halite's reviewed solubility arrived 2026-09-21, so the cap that
        // already bounded chalk bounds this probe too: a millilitre of
        // water holds 6.13e-3 mol of salt, which is 12.26 mol/kgw doubled
        // for two ions, against the 200 the uncapped inventory read. The
        // floor is still doing its job — what it must not do is let the
        // probe fall BELOW the concentrated route, and it does not.
        assert!(
            after > 1.0,
            "and it is still routed as concentrated: {after}"
        );
        assert!(
            after < before,
            "the cap binds where the floor does not: {before} vs {after}"
        );
    }

    /// The floor is what stops a vanishing solvent from turning a dilute
    /// beaker into a brine on paper. A millimole of dissolved salt is a
    /// millimole whether the water is a litre or a droplet; what changes
    /// is whether "per kilogram of water" is a statement about anything.
    #[test]
    fn the_solvent_floor_binds_only_below_a_millilitre() {
        let mut litre = Vessel::new(VesselId(0), "a litre");
        litre.deposit(SpeciesId::new("water"), Moles(55.51), Phase::Liquid);
        litre.deposit(SpeciesId::new("NaCl"), Moles(0.001), Phase::Solid);
        let (_, before, after) = estimate(&litre);
        assert!(
            (before - after).abs() < 1e-9,
            "a litre is far above the floor: {before} vs {after}"
        );

        let mut droplet = Vessel::new(VesselId(0), "a droplet");
        // A tenth of a millilitre, holding a fifth of a millimole of salt.
        droplet.deposit(SpeciesId::new("water"), Moles(0.005551), Phase::Liquid);
        droplet.deposit(SpeciesId::new("NaCl"), Moles(0.0002), Phase::Solid);
        let (kgw, before, after) = estimate(&droplet);
        eprintln!("droplet: kgw={kgw:.6e} before={before:.4} after={after:.4}");
        assert!(before > after, "the floor binds: {before} vs {after}");
        assert!(
            after < 1.0 && before > 1.0,
            "and this is the price: a genuinely concentrated droplet of a \
             tenth of a millilitre is routed as dilute — {before} becomes {after}"
        );
    }
}

// Frozen source-informed raw mixed-crystal readback contracts.
#[cfg(test)]
use kerotakis_core::VesselId;
#[cfg(test)]
#[path = "../tests/solid_solution_raw_readback/contracts_complete_columns.rs"]
mod solid_solution_raw_readback_contracts;

#[cfg(test)]
#[path = "../tests/solid_solution_phase_budget/contracts_closed_boundary.rs"]
mod solid_solution_raw_readback_contracts_budget;

#[cfg(test)]
#[path = "../tests/solid_solution_phase_budget/supplement_closed_boundary.rs"]
mod solid_solution_raw_readback_contracts_budget_supplement;

#[cfg(test)]
#[path = "../tests/solid_solution_phase_budget/cancellation.rs"]
mod solid_solution_raw_readback_contracts_cancellation;

#[cfg(test)]
#[path = "../tests/solid_solution_phase_budget/raw_carbon.rs"]
mod solid_solution_raw_readback_contracts_raw_carbon;

#[cfg(test)]
#[path = "../tests/solid_solution_phase_budget/external_boundaries.rs"]
mod solid_solution_raw_readback_contracts_external_boundaries;

#[cfg(test)]
#[path = "../tests/solid_solution_phase_budget/external_raw.rs"]
mod solid_solution_raw_readback_contracts_external_raw;

#[cfg(test)]
#[path = "../tests/solid_solution_phase_budget/refusal_atomicity.rs"]
mod solid_solution_raw_readback_contracts_refusal_atomicity;

#[cfg(test)]
mod finite_gas_temperature_serialization_tests {
    use super::*;

    #[test]
    fn warm_fractional_primary_and_mix_inputs_round_trip_supplied_solutes() {
        let make = |id, kgw: f64, salt| {
            let mut vessel = Vessel::new(kerotakis_core::VesselId(id), "warm input");
            vessel.temperature = Kelvin(298.5820854655798);
            vessel.deposit(
                SpeciesId::new("water"),
                Moles(kgw * 1000.0 / WATER_MOLAR_MASS),
                Phase::Liquid,
            );
            // The source starts with a complete dissolved salt pair. Solid
            // NaCl is posed as Halite, so it has no Na/Cl SOLUTION rows to
            // round-trip in this serialization control.
            vessel.deposit(SpeciesId::new("Na+"), Moles(salt), Phase::Aqueous);
            vessel.deposit(SpeciesId::new("Cl-"), Moles(salt), Phase::Aqueous);
            vessel
        };
        let a = make(0, 0.09979672112345678, 0.020000000000004334);
        let b = make(1, 0.08123456789123456, 0.01112345678901234);
        let pa = partition(&a).unwrap();
        let pb = partition(&b).unwrap();
        assert!(!phase_input_precision(&a) && !phase_input_precision(&b));
        let read = |input: &str, field: &str| -> f64 {
            input
                .lines()
                .find_map(|line| line.trim().strip_prefix(field))
                .unwrap()
                .trim()
                .parse()
                .unwrap()
        };
        for (vessel, problem, supplied) in [
            (&a, &pa, 0.020000000000004334),
            (&b, &pb, 0.01112345678901234),
        ] {
            let input = build_input(vessel, problem, "wateq4f");
            let water = read(&input, "water ");
            assert_eq!(water, problem.kgw);
            assert!((read(&input, "Na ") * water - supplied).abs() < 2e-14);
            assert!((read(&input, "Cl ") * water - supplied).abs() < 2e-14);
        }
        let mut merged = a.clone();
        for portion in &b.contents {
            merged.deposit(portion.species.clone(), portion.moles, portion.phase);
        }
        let pm = partition(&merged).unwrap();
        let mixed = build_mix_input(&a, &pa, &b, &pb, 1.0, 1.0, "wateq4f", &pm, false);
        for (number, problem, supplied) in [
            (1, &pa, 0.020000000000004334),
            (2, &pb, 0.01112345678901234),
        ] {
            let section = mixed
                .split(&format!("SOLUTION {number}\n"))
                .nth(1)
                .unwrap()
                .split("END")
                .next()
                .unwrap();
            let water = read(section, "water ");
            assert_eq!(water, problem.kgw);
            assert!((read(section, "Na ") * water - supplied).abs() < 2e-14);
            assert!((read(section, "Cl ") * water - supplied).abs() < 2e-14);
        }
    }

    #[test]
    fn finite_gas_input_round_trips_fractional_solvent_and_aqueous_carbon() {
        let kgw = 0.099699987654321;
        let aqueous_carbon = 0.00123456789;
        let gas_carbon = 0.02;
        let mut vessel = Vessel::new(kerotakis_core::VesselId(0), "finite carbon input");
        vessel.temperature = Kelvin(298.1500375477956);
        vessel.headspace = Headspace::Sealed {
            volume: kerotakis_core::Liters(0.1),
        };
        vessel.deposit(
            SpeciesId::new("water"),
            Moles(kgw * 1000.0 / WATER_MOLAR_MASS),
            Phase::Liquid,
        );
        vessel.deposit(
            SpeciesId::new("CO2(aq)"),
            Moles(aqueous_carbon),
            Phase::Aqueous,
        );
        vessel.deposit(SpeciesId::new("CO2"), Moles(gas_carbon), Phase::Gas);
        let problem = partition(&vessel).unwrap();
        assert!((problem.kgw - kgw).abs() < 1e-16);
        let input = build_input(&vessel, &problem, "wateq4f");
        let written = |prefix: &str| -> f64 {
            input
                .lines()
                .find_map(|line| line.trim().strip_prefix(prefix))
                .unwrap()
                .trim()
                .parse()
                .unwrap()
        };
        let input_aqueous = written("C ") * written("water ");
        let input_gas = written("-moles ");
        assert!(input.contains("GAS_PHASE_RAW 1"));
        assert!((written("-temperature ") - vessel.temperature.0).abs() < 5e-10);
        let target = aqueous_carbon + gas_carbon;
        // Each twelve-decimal scientific field has <=5e-13 relative
        // quantization. This bound covers both native initialization paths.
        let bound = 5e-12 * target;
        assert!((input_aqueous + input_gas - target).abs() < bound);
        let legacy_water: f64 = format!("{:.9}", problem.kgw).parse().unwrap();
        let legacy_error = written("C ") * legacy_water - aqueous_carbon;
        let predicted_error = aqueous_carbon * (legacy_water / problem.kgw - 1.0);
        assert!(
            legacy_error.abs() > 1e-12,
            "fixture must expose the old picomole-scale input error"
        );
        assert!((legacy_error - predicted_error).abs() < 1e-15);

        // The open route preserves the same physical carbon and water
        // without inheriting a finite gas phase from the closed fixture.
        let mut open = vessel.clone();
        open.headspace = Headspace::Open;
        open.contents.retain(|portion| portion.phase != Phase::Gas);
        let open_problem = partition(&open).unwrap();
        let open_input = build_input(&open, &open_problem, "wateq4f");
        assert!(!open_input.contains("GAS_PHASE_RAW 1"));
        let open_written = |prefix: &str| -> f64 {
            open_input
                .lines()
                .find_map(|line| line.trim().strip_prefix(prefix))
                .unwrap()
                .trim()
                .parse()
                .unwrap()
        };
        assert_eq!(open_written("water "), open_problem.kgw);
        assert!((open_written("C ") * open_written("water ") - aqueous_carbon).abs() < 1e-14);
    }

    #[test]
    fn gas_initializer_round_trips_owned_moles_at_recorded_fractional_temperatures() {
        for temperature in [298.1500375477956, 298.5820854655798, 298.14965445166615] {
            let mut vessel = Vessel::new(kerotakis_core::VesselId(0), "finite gas input");
            vessel.temperature = Kelvin(temperature);
            vessel.headspace = Headspace::Sealed {
                volume: kerotakis_core::Liters(0.1),
            };
            vessel.contents.push(Portion {
                species: SpeciesId::new("water"),
                moles: Moles(5.55),
                phase: Phase::Liquid,
            });
            vessel.contents.push(Portion {
                species: SpeciesId::new("CO2"),
                moles: Moles(0.02),
                phase: Phase::Gas,
            });
            let problem = partition(&vessel).unwrap();
            let input = build_input(&vessel, &problem, "wateq4f");
            let written_kelvin: f64 = input
                .lines()
                .find_map(|line| line.trim().strip_prefix("-temperature "))
                .unwrap()
                .trim()
                .parse()
                .unwrap();
            let initialized: f64 = input
                .lines()
                .find_map(|line| line.trim().strip_prefix("-moles "))
                .unwrap()
                .trim()
                .parse()
                .unwrap();
            assert!((written_kelvin - temperature).abs() < 5e-10);
            assert!(
                (initialized - 0.02).abs() < 1e-14,
                "T={temperature}: {initialized}"
            );
        }
    }
}

#[cfg(test)]
mod comparison_phase_inventory_tests {
    use super::*;
    use kerotakis_core::VesselId;

    fn water_problem() -> Problem {
        let mut vessel = Vessel::new(VesselId(0), "comparison inventory");
        vessel.deposit(SpeciesId::new("water"), Moles(5.0), Phase::Liquid);
        partition(&vessel).expect("liquid water supplies an aqueous problem")
    }

    #[test]
    fn unavailable_positive_solids_refuse_instead_of_disappearing() {
        let mut problem = water_problem();
        problem.phases = vec![("UnreviewedFiniteSolid".into(), 0.25, 0.0)];
        assert_eq!(
            comparison_problem(&problem, "wateq4f")
                .err()
                .expect("must refuse"),
            vec!["UnreviewedFiniteSolid".to_string()]
        );
        // Even an amount below narrative reporting floors is supplied matter.
        problem.phases[0].1 = 1e-15;
        assert!(comparison_problem(&problem, "wateq4f").is_err());
    }

    #[test]
    fn unavailable_zero_inventory_candidates_do_not_refuse_the_comparison() {
        let mut problem = water_problem();
        problem.phases = vec![("UnreviewedFiniteSolid".into(), 0.0, 0.0)];
        let scoped = comparison_problem(&problem, "wateq4f").unwrap();
        assert!(scoped.phases.is_empty());
        assert_eq!(scoped.kgw, problem.kgw);
        assert_eq!(scoped.totals, problem.totals);
    }

    #[test]
    fn foreign_solid_is_not_admitted_into_a_typed_surface_comparison() {
        use kerotakis_core::{Grams, SurfaceModel};
        let mut problem = water_problem();
        problem.phases = vec![("Fe(OH)2".into(), 0.01, 0.0)];
        assert!(comparison_problem(&problem, "wateq4f").is_ok());
        problem.surfaces.push(SurfaceSites {
            label: "finite HFO".into(),
            model: SurfaceModel::HydrousFerricOxide,
            mass: Grams(0.09),
            specific_area_m2_per_g: 600.0,
            strong_capacity: Moles(5e-6),
            weak_capacity: Moles(2e-4),
            occupancy: Vec::new(),
            water_release: Moles(0.0),
        });
        assert_eq!(
            comparison_problem(&problem, "wateq4f")
                .err()
                .expect("supplied phase must be refused"),
            vec!["Fe(OH)2"]
        );
        problem.phases[0].1 = 0.0;
        assert!(comparison_problem(&problem, "wateq4f")
            .unwrap()
            .phases
            .is_empty());
    }

    #[test]
    fn foreign_solid_is_not_admitted_into_a_coupled_redox_comparison() {
        let mut problem = water_problem();
        problem.phases = vec![("Fe(OH)2".into(), 0.01, 0.0)];
        problem.totals = vec![("Fe(2)".into(), 0.001), ("Mn(7)".into(), 0.001)];
        assert!(redox_coupling(&problem, "wateq4f").is_some());
        assert_eq!(
            comparison_problem(&problem, "wateq4f")
                .err()
                .expect("supplied phase must be refused"),
            vec!["Fe(OH)2"]
        );
    }

    #[test]
    fn supplied_phase_states_inform_pinning_without_becoming_dissolved_totals() {
        let mut problem = water_problem();
        problem.phases = vec![("Fe(OH)2".into(), 0.01, 0.0)];
        let analytical = problem.totals.clone();
        let states = phase_inclusive_totals(&problem);
        assert!(states
            .iter()
            .any(|(element, amount)| element == "Fe(2)" && (*amount - 0.01).abs() < 1e-12));
        assert!(!fast_redox_pin_block(&states, "wateq4f").is_empty());
        assert_eq!(
            problem.totals, analytical,
            "solid inventory must not be double counted"
        );
        problem.phases[0].1 = 0.0;
        assert_eq!(phase_inclusive_totals(&problem), analytical);
    }

    #[test]
    fn redox_partner_in_positive_solid_keeps_native_only_admission() {
        let mut problem = water_problem();
        problem.phases = vec![("Fe(OH)2".into(), 0.01, 0.0)];
        problem.totals = vec![("Mn(7)".into(), 0.001)];
        assert!(redox_coupling(&problem, "wateq4f").is_none());
        assert!(redox_coupling_totals(&phase_inclusive_totals(&problem), "wateq4f").is_some());
        assert!(comparison_problem(&problem, "wateq4f").is_err());
    }

    #[test]
    fn reviewed_phase_mappings_preserve_supplied_inventory() {
        let tag = "wateq4f";
        let phase = derived::candidate_phases()
            .iter()
            .find(|phase| {
                !derived::index_for(tag).has_phase(&phase.name)
                    && posed_phase(&phase.name, tag).is_some()
            })
            .expect("a reviewed non-native phase exercises the old retain defect");
        let mut problem = water_problem();
        problem.phases = vec![(phase.name.clone(), 0.002, 0.0)];
        let scoped = comparison_problem(&problem, tag).unwrap();
        assert_eq!(scoped.phases.len(), 1);
        assert_eq!(scoped.phases[0].0, posed_phase(&phase.name, tag).unwrap());
        assert_eq!(scoped.phases[0].1, 0.002);
        assert_eq!(scoped.phases[0].2, 0.0);
    }
}

#[cfg(test)]
mod finite_ammonia_admission_diagnostics {
    use super::*;

    #[test]
    fn ammonia_phase_and_reduced_nitrogen_admission_match_the_actual_database() {
        // minteq.v4 carries the wateq4f Henry phase verbatim, so a sealed
        // ammonium + acetate vessel on the ligand route is solvable.
        let minteq = &derived::index_for("minteq.v4").phases["NH3(g)"];
        let wateq = &derived::index_for("wateq4f").phases["NH3(g)"];
        assert_eq!(minteq.log_k, wateq.log_k);
        assert_eq!(minteq.delta_h_kj, wateq.delta_h_kj);
        for phase in [Phase::Aqueous, Phase::Gas] {
            let mut v = Vessel::new(kerotakis_core::VesselId(0), "beaker");
            v.headspace = Headspace::Sealed {
                volume: kerotakis_core::Liters(0.35),
            };
            v.deposit(SpeciesId::new("water"), Moles(3.87), Phase::Liquid);
            v.deposit(SpeciesId::new("NH3"), Moles(0.0008), phase);
            let problem = partition(&v).unwrap();
            assert!(
                problem.elements.iter().any(|element| element == "N(-3)"),
                "phase {phase:?}, role {:?}, elements {:?}",
                derived::role("NH3"),
                problem.elements
            );
            assert!(
                problem.gases.iter().any(|(name, _, _)| name == "NH3(g)"),
                "phase {phase:?}, gases {:?}",
                problem.gases
            );
        }
    }
}

#[cfg(test)]
mod crystal_diagnostic_amount_contract {
    #[test]
    fn invalid_crystal_amount_is_not_misreported_as_solvent_mass() {
        for value in [-1.0, f64::NAN, f64::INFINITY] {
            let error = super::checked_crystal_sum(&[value], "crystal-diagnostic-fixture")
                .expect_err("the actual checked amount reduction must refuse invalid amounts");
            assert_eq!(
                error.owned_reason().unwrap().key,
                "solver-reason.aqueous.crystal-amount-invalid"
            );
            assert_eq!(
                error.to_string(),
                format!("phreeqc-aqueous could not solve this state: mixed-crystal ownership: crystal-diagnostic-fixture: invalid amount {value}")
            );
        }
    }
}
