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

use kerotakis_core::{
    species, Equilibrator, Event, ExchangeIon, ExchangeOccupancy, ExchangeSites, Headspace, Kelvin,
    Moles, Phase, Portion, Provenance, SolidSolution, SolidSolutionComponent, SolidSolutionModel,
    SolutionInfo, SolveError, SpeciesDetail, SpeciesId, SurfaceOccupancy, SurfaceSiteKind,
    SurfaceSites, SurfaceSorbate, ThermalMode, Vessel,
};

#[cfg(feature = "engine")]
const RESET_NUMBERED_REACTANTS: &str = "DELETE\n    -all\nEND\n";

use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::OnceLock;

use crate::PhreeqcError;
#[cfg(feature = "engine")]
use crate::{databases, Phreeqc};

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

use crate::derived::{self, DerivedRole, ATMOSPHERIC};
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

/// What to call the dataset a solve ran on.
///
/// Not simply `{tag}.dat`, because minteq.v4 is not run as vendored: this
/// lab adds one reviewed lactate definition to it (see
/// `databases::minteq_v4`). Reporting the bare filename would name a
/// database we are not running, in the field whose entire job is to let a
/// reader trace where a number came from.
fn dataset_name(db_tag: &str) -> String {
    match db_tag {
        "minteq.v4" => "minteq.v4.dat plus one reviewed lactate definition".to_string(),
        other => format!("{other}.dat"),
    }
}

const WATER_MOLAR_MASS: f64 = 18.015;
const TRACE: f64 = 1e-12;
const UNTRACKED_EXCHANGE_ELEMENTS: &[&str] = &[
    "Al", "Ba", "Cd", "Cu", "Fe", "K", "Li", "Mn", "Pb", "Sr", "Zn",
];

/// What one dataset says about the same vessel.
#[derive(Debug, Clone)]
pub struct PathResult {
    pub dataset: String,
    pub model: String,
    pub outcome: PathOutcome,
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
    /// It tried and could not answer — honest, first-class.
    Failed { detail: String },
}

/// One cached solver result, keyed by database + canonical input.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
    /// Stored the negative way round on purpose. `#[serde(default)]` gives
    /// `false` for an entry written before this field existed, and `false`
    /// has to mean "nothing unusual": the other polarity would have every
    /// pre-warmed result in the shipped demo quietly stop reporting a
    /// potential.
    #[serde(default)]
    pub pe_undetermined: bool,
}

/// How supersaturated a phase must be before we admit we are ignoring it,
/// in log units. Small positive indices are ordinary in natural waters and
/// reporting them would be noise; +1.0 is a tenfold excess over saturation,
/// which is a solution a chemist would not expect to survive.
const SUPERSATURATION_REPORTING_SI: f64 = 1.0;

/// A shippable pre-warmed cache.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CacheData {
    pub entries: Vec<CacheEntry>,
    /// Thermochemical values derived from the same databases as the cached
    /// equilibria. A cache-only device needs these too: otherwise it can
    /// reproduce the composition but not the temperature, and its next
    /// content-addressed lookup describes a different state.
    #[serde(default)]
    pub neutralisation_kj_per_mol: Vec<(String, f64)>,
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

#[derive(Clone)]
struct CachedSolve {
    rows: Vec<Vec<String>>,
    speciation: Vec<SpeciesDetail>,
    saturation: Vec<(String, f64)>,
    redox_adjusted: bool,
    pe_determined: bool,
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

impl PhreeqcEquilibrator {
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
            let (slot, database): (&mut Option<Phreeqc>, &[u8]) = match db_tag {
                "minteq.v4" => (&mut self.pinned_organic, databases::minteq_v4()),
                "pitzer" => (&mut self.pinned_brine, databases::PITZER),
                _ => (&mut self.pinned_inorganic, databases::WATEQ4F),
            };
            if slot.is_none() {
                *slot = Some(Phreeqc::with_database(database).map_err(|e| {
                    SolveError::NotConverged {
                        solver: "phreeqc-aqueous".to_string(),
                        detail: format!("could not load a pinned engine instance: {e}"),
                    }
                })?);
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
        #[cfg(feature = "engine")]
        {
            let engine = self.engine_for(db_tag, input)?;
            engine
                .run(RESET_NUMBERED_REACTANTS)
                .map_err(|e| SolveError::NotConverged {
                    solver: "phreeqc-aqueous".to_string(),
                    detail: format!("could not reset reused IPhreeqc state: {e}"),
                })?;
            engine.run(input).map_err(|e| {
                // The input is the whole question; when the engine refuses
                // it, being able to see it is the difference between a
                // diagnosis and a guess.
                if env_dump_input() {
                    eprintln!("--- PHREEQC input that failed ---\n{input}---");
                }
                SolveError::NotConverged {
                    solver: "phreeqc-aqueous".to_string(),
                    detail: e.to_string(),
                }
            })?;
            Ok(SolveOutput {
                pe_undetermined: false,
                selected: engine.selected_output(),
                report: engine.output_string(),
            })
        }
        #[cfg(not(feature = "engine"))]
        {
            let Some(hook) = self.hook.as_mut() else {
                return Err(SolveError::NotConverged {
                    solver: "phreeqc-aqueous (cache-only build)".to_string(),
                    detail: "this state is not in the shipped results and there is no solver here to compute it".to_string(),
                });
            };
            hook(db_tag, input).map_err(|e| SolveError::NotConverged {
                solver: "phreeqc-aqueous (external engine)".to_string(),
                detail: e,
            })
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
        for _ in 0..34 {
            mid = 0.5 * (lo + hi);
            let input = build_input_at(vessel, problem, db_tag, Some((mid, coupling)));
            // A single awkward trial must not end the search. PHREEQC will
            // refuse some electron activities outright — a residual of one
            // part in a hundred thousand on chloride is enough — and those
            // are scattered through the range rather than at its edges.
            // Aborting on the first one threw away a titration the bisection
            // had very nearly solved, and reported the reagents as unreacted.
            let Ok(out) = self.run_trial(db_tag, &input) else {
                // Keep moving in the direction the last usable answer
                // pointed, so the bracket steps over the bad patch instead
                // of stalling on it.
                match last_sum {
                    Some(sum) if sum < coupling.target => lo = mid,
                    Some(_) => hi = mid,
                    None => lo = mid,
                }
                continue;
            };
            let Some(sum) = oxidation_sum(&out.selected, &coupling.columns, problem.kgw) else {
                return Err(SolveError::NotConverged {
                    solver: "phreeqc-aqueous (redox)".to_string(),
                    detail: "the coupled run reported no oxidation-state totals".to_string(),
                });
            };
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
        let mut inorganic = Phreeqc::with_database(databases::WATEQ4F)?;
        let mut organic = Phreeqc::with_database(databases::minteq_v4())?;
        let mut brine = Phreeqc::with_database(databases::PITZER)?;
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
            warm_pe: None,
            engine_calls: 0,
            hook: None,
            neutralisation,
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
            warm_pe: None,
            engine_calls: 0,
            hook: None,
            neutralisation: std::collections::HashMap::new(),
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
                })
                .collect(),
            neutralisation_kj_per_mol,
        }
    }

    /// Load a pre-warmed cache. Entries already present are kept.
    pub fn import_cache(&mut self, data: CacheData) -> usize {
        let before = self.cache.len();
        for e in data.entries {
            self.cache.entry(e.key).or_insert_with(|| {
                Rc::new(CachedSolve {
                    rows: e.rows,
                    speciation: e.species,
                    saturation: e.saturation,
                    redox_adjusted: e.redox_adjusted,
                    pe_determined: !e.pe_undetermined,
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
                out.push(PathResult {
                    dataset: dataset_name(db_tag),
                    model: idx.activity_model.describe().to_string(),
                    outcome: PathOutcome::CannotExpress {
                        missing_elements: missing,
                    },
                });
                continue;
            }
            let mut scoped = problem.clone();
            scoped.phases.retain(|(name, ..)| idx.has_phase(name));
            let input = build_input(vessel, &scoped, db_tag);
            let engine = match self.engine_for(db_tag, &input) {
                Ok(engine) => engine,
                Err(e) => {
                    out.push(PathResult {
                        dataset: dataset_name(db_tag),
                        model: idx.activity_model.describe().to_string(),
                        outcome: PathOutcome::Failed {
                            detail: e.to_string(),
                        },
                    });
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
            out.push(PathResult {
                dataset: dataset_name(db_tag),
                model: idx.activity_model.describe().to_string(),
                outcome,
            });
        }
        out
    }
}

#[derive(Clone)]
struct Problem {
    kgw: f64,
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

fn partition(vessel: &Vessel) -> Option<Problem> {
    let mut kgw = 0.0;
    let mut totals: Vec<(String, f64)> = Vec::new();
    let mut phases: Vec<(String, f64, f64)> = Vec::new();
    let mut gases: Vec<(String, String, f64)> = Vec::new();
    let mut external_gases: Vec<ExternalGas> = Vec::new();
    let mut solutes = 0;

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

    for p in &vessel.contents {
        if p.phase == Phase::Gas {
            if let Some(volume) = vessel.headspace_volume() {
                // Only gases whose liquid exchange this adapter explicitly
                // supports enter PHREEQC. Nitrogen and oxygen are retained
                // as inert pressure/mass inventory: unrestricted equilibrium
                // turns room air into nitrate, a thermodynamic endpoint that
                // is kinetically impossible on a bench. CO2 is the first
                // approved gas/liquid model; later gases grow this list.
                if !ATMOSPHERIC
                    .iter()
                    .any(|(_, species, ..)| *species == p.species.0)
                {
                    continue;
                }
                const R_LITRE_ATM: f64 = 0.082_057_366;
                let Some(data) = species::lookup(&p.species) else {
                    continue;
                };
                let phase = format!("{}(g)", data.formula);
                let partial_pressure = p.moles.0 * R_LITRE_ATM * vessel.temperature.0 / volume.0;
                gases.push((phase, p.species.0.clone(), partial_pressure));
                if let Some(formula) = crate::dbindex::parse_formula(data.formula) {
                    for element in formula
                        .keys()
                        .filter(|element| *element != "H" && *element != "O")
                    {
                        note_element(element);
                    }
                }
                solutes += 1;
            } else if let Some((phase, species, ..)) = ATMOSPHERIC
                .iter()
                .find(|(_, species, ..)| *species == p.species.0)
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

    if kgw <= 0.0 || solutes == 0 {
        return None;
    }
    append_candidate_phases(vessel, &elements, &mut phases);
    if vessel.owns_headspace_gas() {
        // A finite headspace must admit gases that can form from the
        // solution even when none was initially present.
        for (phase, species, _) in ATMOSPHERIC {
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
    } else {
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
                phases.push((phase.to_string(), exchange.initial_moles, 0.0));
                continue;
            }
            let gas_formula = phase.trim_end_matches("(g)");
            let required = crate::dbindex::parse_formula(gas_formula).unwrap_or_default();
            let all_present = required
                .keys()
                .filter(|el| *el != "O" && *el != "H")
                .all(|el| elements.iter().any(|e| e == el));
            let listed = phases.iter().any(|(name, ..)| name == phase);
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
    Some(Problem {
        kgw,
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

fn solid_solution_element_inventory(solid_solutions: &[SolidSolution], element: &str) -> f64 {
    solid_solutions
        .iter()
        .flat_map(|solid_solution| &solid_solution.components)
        .filter(|amount| solid_solution_component_elements(amount.component).contains(&element))
        // Both reviewed carbonate end members contain one mole of their
        // cation and one mole of carbon per formula unit.
        .map(|amount| amount.moles.0)
        .sum()
}

impl Equilibrator for PhreeqcEquilibrator {
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
        if let Some(surface) = vessel
            .surfaces
            .iter()
            .find(|surface| !surface.has_valid_capacity())
        {
            return Err(SolveError::NotConverged {
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
                solver: self.name().to_string(),
                detail: format!(
                    "exchange '{}' has non-positive support/capacity or occupancy that does not exactly counter-balance its finite capacity",
                    exchange.label
                ),
            });
        }
        if vessel.solid_solutions.len() > 1 {
            return Err(SolveError::NotConverged {
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
                solver: self.name().to_string(),
                detail: format!(
                    "solid solution '{}' must contain one finite, non-negative amount of each reviewed end member",
                    solid_solution.label
                ),
            });
        }
        let start = vessel.clone();
        let t0 = start.temperature.0;
        let mut guess = t0;
        let mut volume_guess = start.headspace_volume();
        let mut settled: Option<(Vessel, Vec<Event>, f64)> = None;

        for _ in 0..8 {
            let mut trial = start.clone();
            trial.temperature = Kelvin(guess);
            if let (Headspace::PressureControlled { pressure, .. }, Some(volume)) =
                (trial.headspace, volume_guess)
            {
                trial.headspace = Headspace::PressureControlled { pressure, volume };
            }
            let (events, q_joules) = self.solve_once(&mut trial)?;
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
            let cp = trial.heat_capacity();
            let cp_before = start.heat_capacity();
            let t_ref = Kelvin::STANDARD.0;
            let next = if cp > 0.0 {
                t_ref + (cp_before * (t0 - t_ref) + q_joules) / cp
            } else {
                t0
            };
            let next_volume = trial.headspace_volume();
            let volume_converged = match (volume_guess, next_volume) {
                (Some(before), Some(after)) if start.owns_headspace_gas() => {
                    (after.0 - before.0).abs() < (before.0.abs() * 1e-7).max(1e-9)
                }
                _ => true,
            };
            let converged = (next - guess).abs() < 0.05 && volume_converged;
            settled = Some((trial, events, next));
            guess = next;
            volume_guess = next_volume;
            if converged {
                break;
            }
        }

        let Some((solved, mut events, t_final)) = settled else {
            let mut notes = unspeciated_acid_notes(&start);
            notes.extend(unspeciated_solute_notes(&start));
            return Ok(notes);
        };
        *vessel = solved;
        events.extend(unspeciated_acid_notes(vessel));
        events.extend(unspeciated_solute_notes(vessel));
        let ph_now = vessel.solution.as_ref().map(|s| s.ph);
        events.extend(milk_buffer_notes(vessel, ph_now));
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
        // Both source vessels must be solvable aqueous problems.
        let problem_a = partition(soln_a)?;
        let problem_b = partition(soln_b)?;

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

        let input = build_mix_input(
            soln_a,
            &problem_a,
            soln_b,
            &problem_b,
            frac_a,
            frac_b,
            db_tag,
            &merged_problem,
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

        let cached = Rc::new(CachedSolve {
            pe_determined: !out.pe_undetermined,
            rows: out.selected,
            speciation: parse_species_distribution(&out.report),
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
            _,
            protonation,
        ) = match readback {
            Ok(v) => v,
            Err(e) => return abandoned(e),
        };

        let balance = Self::apply_balance_corrections(
            vessel,
            &merged_problem,
            &mut new_ions,
            &mut new_surfaces,
            &new_exchanges,
            &new_solid_solutions,
            &value,
        );
        let (new_phases, new_gases, ph, mu) = match balance {
            Ok(v) => v,
            Err(e) => return abandoned(e),
        };

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

        vessel.contents = contents;
        vessel.surfaces = new_surfaces;
        vessel.exchanges = new_exchanges;
        vessel.solid_solutions = new_solid_solutions;
        vessel.refresh_pressure();

        vessel.solution = Some(SolutionInfo {
            pe: value("pe"),
            redox: Vec::new(),
            ph,
            ionic_strength: mu,
            species: cached.speciation.clone(),
            provenance: Some(Provenance {
                engine: "PHREEQC (IPhreeqc, USGS)".to_string(),
                dataset: dataset_name(db_tag),
                model: derived::index_for(db_tag)
                    .activity_model
                    .describe()
                    .to_string(),
                dataset_sources: derived::index_for(db_tag)
                    .citations
                    .iter()
                    .take(3)
                    .cloned()
                    .collect(),
                routing: "MIX: two solved solutions combined by fraction".to_string(),
            }),
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
    routing: String,
    freed_phases: Vec<(String, f64)>,
    input: String,
    key: String,
}

impl PhreeqcEquilibrator {
    /// One pass at the vessel's current temperature. Returns the reaction
    /// heat rather than applying it, so the caller can iterate temperature
    /// and composition to a common answer instead of reporting one solved
    /// before the other.
    fn solve_once(&mut self, vessel: &mut Vessel) -> Result<(Vec<Event>, f64), SolveError> {
        let Some(SolveSetup {
            problem,
            db_tag,
            routing,
            freed_phases,
            input,
            key,
        }) = self.setup_problem(vessel)?
        else {
            return Ok((Vec::new(), 0.0));
        };

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

        let (new_phases, new_gases, ph, mu) = Self::apply_balance_corrections(
            vessel,
            &problem,
            &mut new_ions,
            &mut new_surfaces,
            &new_exchanges,
            &new_solid_solutions,
            &value,
        )?;

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
            .filter(|p| p.phase == Phase::Aqueous)
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
        vessel.contents = contents;
        vessel.surfaces = new_surfaces;
        vessel.exchanges = new_exchanges;
        vessel.solid_solutions = new_solid_solutions;
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

        Self::finalize_solution_info(
            vessel,
            &problem,
            db_tag,
            routing,
            cached.speciation.clone(),
            &cached.saturation,
            coupling_failed,
            cached.pe_determined,
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
        let measured_oh = free_hydroxide_moles(
            vessel.solution.as_ref().map(|s| s.species.as_slice()),
            problem.kgw,
        );
        vessel.free_hydroxide = measured_oh;
        // Written beside it and unused by this balance — H+ is a master
        // species and carries no enthalpy — but a gate above the tail has
        // no other way to read it once `solution` has been cleared. See
        // the field docs for why it is not `unspent_acidity`.
        vessel.free_proton = measured_species_moles(
            vessel.solution.as_ref().map(|s| s.species.as_slice()),
            "H+",
            problem.kgw,
        );

        if matches!(vessel.thermal_mode, ThermalMode::Adiabatic) {
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
            match enthalpy::heat_released_j(
                &before_contents,
                before_oh,
                &vessel.contents,
                measured_oh,
                &gas_out,
                db_tag,
            ) {
                Ok(j) => q_joules += j,
                Err(_unpriced) => {
                    // THE LAST SECOND PATH, and it is confined to exactly
                    // the steps the balance cannot answer.
                    //
                    // Declining outright loses heats the bench already
                    // knew. A copper sulfate solution precipitates
                    // chalcanthite, which no registry row prices, and the
                    // whole step then declined — throwing away the -73.1
                    // kJ/mol of dissolving the copper sulfate, which is
                    // known and large. The beaker sat at room temperature
                    // instead of reaching 58 °C, and because solubility
                    // follows temperature the lesson's crystals and even
                    // its colour changed with it.
                    //
                    // So where the balance can price a step it owns it
                    // completely and no event charges anything; where it
                    // cannot, this falls back to what the bench charged
                    // before — per observed dissolution, partial by
                    // construction, and no worse than yesterday.
                    // `dissolution_fallback_is_only_for_steps_the_balance_declines`
                    // counts what still lands here.
                    for e in &events {
                        match e {
                            Event::Dissolved {
                                species: sid,
                                moles,
                                ..
                            } => {
                                if let Some(dh) =
                                    species::lookup(sid).and_then(|d| d.dissolution_enthalpy_kj)
                                {
                                    q_joules -= dh * 1000.0 * moles.0;
                                }
                            }
                            Event::Precipitated {
                                species: sid,
                                moles,
                                ..
                            } => {
                                if let Some(dh) =
                                    species::lookup(sid).and_then(|d| d.dissolution_enthalpy_kj)
                                {
                                    q_joules += dh * 1000.0 * moles.0;
                                }
                            }
                            _ => {}
                        }
                    }
                    // Nothing charged and nothing guessed.
                    //
                    // Deliberately SILENT, and this was measured rather
                    // than assumed. A `NotYetModeled` here reads "no route
                    // answered this step", and the curiosity classifier
                    // takes it at its word ahead of the route branches: it
                    // moved fifteen corpus rows from `computed` to
                    // `missing` — rows whose pH, speciation and products
                    // were all still there and correct. Only the HEAT was
                    // unpriced, and saying the step was not modelled
                    // because of that is a false statement about the whole
                    // step, of exactly the kind this module exists to stop
                    // making about temperature.
                    //
                    // Not charging is also the status quo: an unpriceable
                    // dissolution was silently uncharged before this
                    // existed too. The refusal is still real and still
                    // names its species — `heat_released_j` returns it,
                    // and `reaction_heat.rs` asserts on it — it just is
                    // not broadcast as a claim about anything but itself.
                }
            }
        }

        Ok((events, q_joules))
    }

    fn setup_problem(&self, vessel: &Vessel) -> Result<Option<SolveSetup>, SolveError> {
        let Some(mut problem) = partition(vessel) else {
            return Ok(None);
        };

        // Route by validity domain: minteq.v4 when its extended chemistry
        // is needed — organics (it alone has acetate) or phosphate (wateq4f
        // lacks the free H3PO4 species, so the first proton would come out
        // artificially strong); pitzer for concentrated major-ion brines
        // (the ion-interaction model is built for them: halite saturates at
        // the textbook 6.13 mol/kgw where wateq4f gives 6.50); wateq4f
        // otherwise.
        // "Extended": any element wateq4f does not know (derived — e.g.
        // Acetate), plus phosphate as a documented exception (wateq4f has
        // the P element but no free H3PO4 species).
        let needs_extended = problem
            .elements
            .iter()
            .any(|e| !derived::index_for("wateq4f").has_element(e) || e == "P");
        // Rough concentration estimate: dissolved totals plus what the
        // solid phases could dissolve (each formula unit ~2 ions).
        let potential_molality = (problem.totals.iter().map(|(_, n)| n).sum::<f64>()
            + 2.0 * problem.phases.iter().map(|(_, n, _)| n).sum::<f64>()
            + 2.0
                * problem
                    .solid_solutions
                    .iter()
                    .map(|solid_solution| solid_solution.total_moles().0)
                    .sum::<f64>())
            / problem.kgw;
        let pitzer_capable = problem
            .elements
            .iter()
            .all(|el| derived::index_for("pitzer").has_element(el));
        let (db_tag, routing) = if needs_extended {
            (
                "minteq.v4",
                "chosen because the problem needs chemistry the default dataset lacks (organic ligands or free phosphoric acid)".to_string(),
            )
        } else if potential_molality > 1.0
            && pitzer_capable
            && problem.surfaces.is_empty()
            && problem.exchanges.is_empty()
            && problem.solid_solutions.is_empty()
        {
            (
                "pitzer",
                format!(
                    "chosen because the solution is concentrated (~{potential_molality:.1} mol/kgw), where the ion-interaction model is the valid one"
                ),
            )
        } else {
            (
                "wateq4f",
                "the default for dilute inorganic aqueous chemistry".to_string(),
            )
        };
        if !problem.surfaces.is_empty() {
            if potential_molality > 1.0 && db_tag != "minteq.v4" {
                return Err(SolveError::NotConverged {
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
                    solver: self.name().to_string(),
                    detail: "typed aragonite-strontianite solid solution is currently validated only for dilute inorganic water through wateq4f.dat"
                        .to_string(),
                });
            }
            for component in SolidSolutionComponent::ALL {
                let phase = phreeqc_solid_solution_component(component);
                if !derived::index_for(db_tag).has_phase(phase) {
                    return Err(SolveError::NotConverged {
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
        let coupled_now = redox_coupling(&problem, db_tag).is_some();
        problem.phases.retain_mut(|(name, moles, _)| {
            if idx.has_phase(name) {
                return true;
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
        let key = format!("#{db_tag}\n{input}");

        Ok(Some(SolveSetup {
            problem,
            db_tag,
            routing,
            freed_phases,
            input,
            key,
        }))
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
        let cached = if let Some(hit) = self.cache.get(&key) {
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
            let out = match redox_coupling(problem, db_tag) {
                Some(coupling) => match self.solve_coupled(vessel, problem, db_tag, &coupling) {
                    Ok(out) => out,
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
                        self.run_raw(db_tag, input)?
                    }
                },
                None => match self.run_raw(db_tag, input) {
                    Ok(out) => out,
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
                            self.run_raw(db_tag, &recast_input).map_err(|_| e)?
                        }
                        None => return Err(e),
                    },
                },
            };
            let cached = Rc::new(CachedSolve {
                pe_determined: !out.pe_undetermined,
                rows: out.selected,
                speciation: parse_species_distribution(&out.report),
                saturation: parse_saturation_indices(&out.report),
                redox_adjusted: out.report.contains("Adjusted to redox equilibrium"),
            });
            if self.cache.len() >= 10_000 {
                self.cache.clear(); // simple bound; refine when profiling says so
            }
            self.cache.insert(key, Rc::clone(&cached));
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
        let kgw_out = value("mass_H2O").ok_or_else(|| missing("mass_H2O"))?;
        let mut solvent_kgw_out = kgw_out;
        let mut new_surfaces = problem.surfaces.clone();
        if !new_surfaces.is_empty() {
            let strong = value("m_Hfo_sOZn+").ok_or_else(|| missing("m_Hfo_sOZn+"))? * kgw_out;
            let weak = value("m_Hfo_wOZn+").ok_or_else(|| missing("m_Hfo_wOZn+"))? * kgw_out;
            // The reviewed HFO datasets also bind the sulfate
            // counterion used by the zinc-sulfate lesson. Both weak-site
            // complexes contain one sulfate and occupy one site; failing to
            // round-trip them made sulfur (and its mass) disappear from the
            // vessel after every equilibrium pass.
            let weak_sulfate_water =
                value("m_Hfo_wSO4-").ok_or_else(|| missing("m_Hfo_wSO4-"))? * kgw_out;
            let weak_sulfate_hydroxyl =
                value("m_Hfo_wOHSO4-2").ok_or_else(|| missing("m_Hfo_wOHSO4-2"))? * kgw_out;
            let weak_sulfate = weak_sulfate_water + weak_sulfate_hydroxyl;
            // MINTEQ additionally defines the equivalent strong-site pair.
            let (strong_sulfate, strong_sulfate_water) = if db_tag == "minteq.v4" {
                let water = value("m_Hfo_sSO4-").ok_or_else(|| missing("m_Hfo_sSO4-"))? * kgw_out;
                let hydroxyl =
                    value("m_Hfo_sOHSO4-2").ok_or_else(|| missing("m_Hfo_sOHSO4-2"))? * kgw_out;
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
                let moles = value(&column).ok_or_else(|| missing(&column))? * kgw_out;
                distribute_exchange_occupancy(&mut new_exchanges, ion, moles);
            }
            if let Some(exchange) = new_exchanges
                .iter()
                .find(|exchange| !exchange.has_valid_capacity())
            {
                return Err(SolveError::NotConverged {
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
                amount.moles = Moles(value(&column).ok_or_else(|| missing(&column))?.max(0.0));
            }
            if !solid_solution.has_valid_state() {
                return Err(SolveError::NotConverged {
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
            let molality = value(el).ok_or_else(|| missing(el))?;
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
                .filter_map(|c| value(c).map(|m| (c, m.max(0.0))))
                .collect();
            let sum: f64 = split.iter().map(|(_, m)| m).sum();
            let total = value(base).unwrap_or(sum) * kgw_out;
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
                    value(&format!("m_{species}")).map(|m| (*key, m.max(0.0)))
                })
                .collect();
            // All or nothing. A partial split would divide the total by a
            // denominator missing one of its terms, which is not a
            // rounding error — it is the whole of the missing species
            // silently reassigned to the one that was found.
            if weights.len() != split.len() {
                continue;
            }
            let sum: f64 = weights.iter().map(|(_, m)| m).sum();
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

    #[allow(clippy::type_complexity)]
    fn apply_balance_corrections(
        vessel: &Vessel,
        problem: &Problem,
        new_ions: &mut Vec<(String, f64)>,
        new_surfaces: &mut [SurfaceSites],
        new_exchanges: &[ExchangeSites],
        new_solid_solutions: &[SolidSolution],
        value: &dyn Fn(&str) -> Option<f64>,
    ) -> Result<(Vec<(String, f64)>, Vec<(String, String, f64)>, f64, f64), SolveError> {
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
        // The typed mixed crystal owns its end-member formula units. Keep
        // aqueous selected totals equal the analytical inventory remaining
        // after that ownership is removed, regardless of whether a database
        // reports SOLID_SOLUTIONS inside or outside its selected total. In a
        // closed headspace carbon also moves into CO2(g), so that owned gas
        // is part of both sides of the ledger. An external gas boundary is
        // deliberately open and therefore cannot be closed this way.
        if !problem.solid_solutions.is_empty() {
            for element in ["Ca", "Sr", "C"] {
                if element == "C" && !problem.external_gases.is_empty() {
                    continue;
                }
                let solution_inventory: f64 = problem
                    .totals
                    .iter()
                    .filter(|(candidate, _)| {
                        candidate.split('(').next().unwrap_or(candidate) == element
                    })
                    .map(|(_, moles)| moles)
                    .sum();
                let initial_solid_solution =
                    solid_solution_element_inventory(&problem.solid_solutions, element);
                let final_solid_solution =
                    solid_solution_element_inventory(new_solid_solutions, element);
                let (initial_gas, final_gas) = if element == "C" {
                    let initial = vessel
                        .contents
                        .iter()
                        .filter(|portion| portion.phase == Phase::Gas && portion.species.0 == "CO2")
                        .map(|portion| portion.moles.0)
                        .sum::<f64>();
                    let final_amount = problem
                        .gases
                        .iter()
                        .filter(|(_, species, _)| species == "CO2")
                        .filter_map(|(phase, _, _)| value(&format!("g_{phase}")))
                        .sum::<f64>();
                    (initial, final_amount)
                } else {
                    (0.0, 0.0)
                };
                let target = (solution_inventory + initial_solid_solution + initial_gas
                    - final_solid_solution
                    - final_gas)
                    .max(0.0);
                let aqueous: f64 = new_ions
                    .iter()
                    .filter(|(candidate, _)| {
                        candidate.split('(').next().unwrap_or(candidate) == element
                    })
                    .map(|(_, moles)| moles)
                    .sum();
                if aqueous > 0.0 {
                    let scale = target / aqueous;
                    for (candidate, moles) in new_ions.iter_mut() {
                        if candidate.split('(').next().unwrap_or(candidate) == element {
                            *moles *= scale;
                        }
                    }
                } else if target > TRACE {
                    new_ions.push((element.to_string(), target));
                }
            }
        }
        let mut new_phases: Vec<(String, f64)> = Vec::new();
        for (phase, ..) in &problem.phases {
            let moles = value(phase).ok_or_else(|| missing(phase))?;
            new_phases.push((phase.clone(), moles));
        }
        let mut new_gases: Vec<(String, String, f64)> = Vec::new();
        for (phase, species, _) in &problem.gases {
            let column = format!("g_{phase}");
            let moles = value(&column).ok_or_else(|| missing(&column))?;
            new_gases.push((phase.clone(), species.clone(), moles.max(0.0)));
        }
        let ph = value("pH").ok_or_else(|| missing("pH"))?;
        let mu = value("mu").ok_or_else(|| missing("mu"))?;
        Ok((new_phases, new_gases, ph, mu))
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
                    let dissolves =
                        species::lookup(&p.species).is_some_and(|d| d.dissolves_without_speciation);
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
                        if share <= TRACE {
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
        routing: String,
        speciation: Vec<SpeciesDetail>,
        saturation: &[(String, f64)],
        coupling_failed: Option<String>,
        pe_determined: bool,
        redox_constrained: bool,
        value: &dyn Fn(&str) -> Option<f64>,
        unnameable: &[(String, f64)],
        ph: f64,
        mu: f64,
        events: &mut Vec<Event>,
    ) {
        let idx = derived::index_for(db_tag);
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
        // Say what the input does to redox, because it is load-bearing and
        // otherwise invisible. Naming an oxidation state in a PHREEQC
        // solution *decouples* that element: it gets its own mass balance
        // and exchanges electrons with nothing. That is how this bench can
        // show permanganate and iron(II) sitting in one beaker — a solution
        // that cannot exist. Each state is right; their coexistence is not,
        // and the difference has to be visible rather than inferred.
        let redox_note = match (&coupling_failed, redox.len()) {
            (Some(why), _) => format!(
                "the redox elements here could not be coupled, so each is shown in the oxidation state it was added in and they have not reacted with each other — {why}"
            ),
            (None, n) if n > 1 && redox_coupling(problem, db_tag).is_none() => {
                // Coupled elements are settled by the electron balance;
                // this note is for the ones deliberately left pinned.
                "some elements here keep the oxidation state they were added in: only the couples that equilibrate on a bench timescale exchange electrons, and the slow ones — sulfate, nitrate, carbonate — are held as added".to_string()
            }
            _ => String::new(),
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
            events.push(Event::NotYetModeled {
                cause: kerotakis_core::ops::NotModelledCause::NoSolver,
                vessel: vessel.id,
                what: format!(
                    "these elements have not reacted with each other — they are shown in \
                     the oxidation states they were added in, which is not what the beaker \
                     would do: {why}"
                ),
            });
        }

        redox.sort_by(|a, b| {
            a.element
                .cmp(&b.element)
                .then(b.molality.total_cmp(&a.molality))
        });

        let info = SolutionInfo {
            redox,
            pe: (redox_constrained && pe_determined)
                .then(|| value("pe"))
                .flatten(),
            ph,
            ionic_strength: mu,
            species: speciation,
            provenance: Some(Provenance {
                engine: "PHREEQC (IPhreeqc, USGS)".to_string(),
                dataset: dataset_name(db_tag),
                model: idx.activity_model.describe().to_string(),
                dataset_sources: idx.citations.iter().take(3).cloned().collect(),
                routing: if redox_note.is_empty() {
                    routing
                } else {
                    format!("{routing}. {redox_note}")
                },
            }),
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
            events.push(Event::NotYetModeled { cause: kerotakis_core::ops::NotModelledCause::PhaseNotInRegistry,
                vessel: vessel.id,
                what: format!(
                    "{moles:.3e} mol settled as {column}, an oxidation state this lab has no name for — it is not in the vessel's inventory, so there is slightly less of that element in the glass than went in"
                ),
            });
        }

        // The honesty boundary, said out loud.
        //
        // Only phases this lab can *name* are offered to the solver, so that
        // an equilibrium can never contain a mineral we would have to drop
        // (losing mass) or display with no story attached. That filter is
        // right and stays. What was wrong is that it was silent: copper
        // sulfate and lye reported pH 9.9 holding 0.01 mol/L of Cu(2+), a
        // solution that cannot exist, because both Cu(OH)2 and tenorite are
        // in the database and neither is in our registry.
        //
        // A phase we *did* offer gets driven to SI 0 by the solver, so it
        // never appears here. Anything left is a phase the database says
        // would form and we declined to model.
        let offered: Vec<&str> = problem.phases.iter().map(|(p, ..)| p.as_str()).collect();
        let mut ignored: Vec<(&str, f64)> = saturation
            .iter()
            .filter(|(phase, si)| {
                *si >= SUPERSATURATION_REPORTING_SI && !offered.contains(&phase.as_str())
            })
            .map(|(p, si)| (p.as_str(), *si))
            .collect();
        ignored.sort_by(|a, b| b.1.total_cmp(&a.1));

        // Two different admissions, and conflating them would be its own
        // small dishonesty. A phase we cannot name at all is a gap in the
        // registry. A phase we *can* name but withheld is a deliberate
        // kinetic claim — tenorite is the stable copper solid and we are
        // asserting it does not form fast enough to see at this
        // temperature — and the user is entitled to know which they are
        // looking at.
        let (withheld, unnamed): (Vec<_>, Vec<_>) = ignored.iter().partition(|(phase, _)| {
            derived::phase_by_name(phase)
                .and_then(|p| species::lookup_key(p.species))
                .and_then(|d| d.forms_only_above_k)
                .is_some()
        });
        let describe = |list: &[&(&str, f64)]| {
            let named: Vec<String> = list
                .iter()
                .take(3)
                .map(|(p, si)| format!("{p} (SI {si:+.1})"))
                .collect();
            let rest = match list.len().saturating_sub(3) {
                0 => String::new(),
                n => format!(", and {n} more"),
            };
            format!("{}{rest}", named.join(", "))
        };
        if !unnamed.is_empty() {
            events.push(Event::NotYetModeled { cause: kerotakis_core::ops::NotModelledCause::PhaseNotInRegistry,
                vessel: vessel.id,
                what: format!(
                    "a real beaker would not stay like this: the solution is supersaturated against {}. Those phases are in {db_tag}.dat but not in this lab's registry, so nothing can precipitate out of it here",
                    describe(&unnamed)
                ),
            });
        }
        if !withheld.is_empty() {
            let t_c = vessel.temperature.to_celsius();
            events.push(Event::NotYetModeled { cause: kerotakis_core::ops::NotModelledCause::PhaseNotInRegistry,
                vessel: vessel.id,
                what: format!(
                    "the solution is supersaturated against {}, which this lab is deliberately holding back: it is the more stable solid, but at {t_c:.0} °C the metastable one forms first and stays. That is a claim about rates, not about equilibrium, and it is curated rather than computed",
                    describe(&withheld)
                ),
            });
        }
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
        && derived::UNSPECIATED_SOLUTES.iter().any(|(key, _)| {
            vessel
                .contents
                .iter()
                .any(|portion| portion.species.0 == *key && portion.moles.0 > TRACE)
        })
}

fn unspeciated_solute_notes(vessel: &Vessel) -> Vec<Event> {
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
    notes
        .into_iter()
        .map(|(key, why)| {
            let name = kerotakis_core::species::lookup_key(key)
                .map(|d| d.name)
                .unwrap_or(key);
            Event::NotYetModeled {
                // Not in our gift, and not in anybody's — which is the
                // distinction this cause exists to carry.
                cause: kerotakis_core::ops::NotModelledCause::NotInAnyDatabase,
                vessel: vessel.id,
                what: format!("{name} is dissolved and unspeciated: {why}"),
            }
        })
        .collect()
}

fn unspeciated_acid_notes(vessel: &Vessel) -> Vec<Event> {
    if !holds_unspeciated_acid(vessel) {
        return Vec::new();
    }
    let mut notes: Vec<&str> = derived::UNSPECIATED_ACIDS
        .iter()
        .filter(|(key, _)| {
            vessel
                .contents
                .iter()
                .any(|portion| portion.species.0 == *key && portion.moles.0 > TRACE)
        })
        .map(|(_, why)| *why)
        .collect();
    notes.sort_unstable();
    notes.dedup();
    notes
        .into_iter()
        .map(|why| Event::NotYetModeled {
            cause: kerotakis_core::ops::NotModelledCause::NotSpeciated,
            vessel: vessel.id,
            what: format!(
                "this solution holds an acid whose acidity is not modelled: {why}. \
                 Whatever pH is shown is the pH of everything else in the glass, and \
                 the real solution is more acidic than it says"
            ),
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
        .find(|a| a.contains("CASEIN IS NOT MODELLED"))
        .and_then(|a| a.split_once(", which").map(|(head, _)| head.to_string()))
    else {
        return Vec::new();
    };
    vec![Event::NotYetModeled {
        cause: kerotakis_core::ops::NotModelledCause::NoReviewedDatum,
        vessel: vessel.id,
        what: format!(
            "this milk has been acidified to pH {ph:.2} and the number is a \
             LOWER BOUND — the real beaker is milder. {sentence} \
             (the recipe's full assumption is in `explain material whole_milk`)"
        ),
    }]
}

fn missing(column: &str) -> SolveError {
    SolveError::NotConverged {
        solver: "phreeqc-aqueous".to_string(),
        detail: format!("selected output lacks column '{column}'"),
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
    let idx = derived::index_for(db_tag);
    let mut target = 0.0;
    let mut scale = 0.0;
    let mut coupled: Vec<&str> = Vec::new();
    for (key, moles) in &problem.totals {
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
    let header = rows.first()?;
    let last = rows.last()?;
    let mut sum = 0.0;
    for column in columns {
        let Some(i) = header.iter().position(|h| h == column) else {
            continue;
        };
        let molality: f64 = last.get(i)?.parse().ok()?;
        let state = tagged_state(column)? as f64;
        sum += state * molality * kgw;
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
) -> String {
    use std::fmt::Write;
    let mut input = String::new();
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
    writeln!(input, "    temp      {temp_a_c:.4}").unwrap();
    writeln!(input, "    pH        7  charge").unwrap();
    writeln!(input, "    water     {:.9}", problem_a.kgw).unwrap();
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
    writeln!(input, "    temp      {temp_b_c:.4}").unwrap();
    writeln!(input, "    pH        7  charge").unwrap();
    writeln!(input, "    water     {:.9}", problem_b.kgw).unwrap();
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
    let mut molalities: Vec<&str> = Vec::new();
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
        input.push_str(&fast_redox_pin_block(&problem.totals, db_tag));
    }
    let temp_c = vessel.temperature.to_celsius();
    writeln!(input, "SOLUTION 1").unwrap();
    writeln!(input, "    units     mol/kgw").unwrap();
    writeln!(input, "    temp      {temp_c:.4}").unwrap();
    writeln!(input, "    pH        7  charge").unwrap();
    writeln!(input, "    water     {:.9}", problem.kgw).unwrap();
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
        writeln!(input, "GAS_PHASE 1").unwrap();
        match vessel.headspace {
            Headspace::Sealed { volume } => {
                writeln!(input, "    -fixed_volume").unwrap();
                writeln!(input, "    -volume {:.12e}", volume.0).unwrap();
            }
            Headspace::PressureControlled { volume, .. } => {
                // The wrapper iterates this fixed-volume solve with V=nRT/P
                // until the requested controller pressure and gas/liquid
                // partition agree. This keeps inert carrier gas out of
                // PHREEQC's unrestricted redox equilibrium.
                writeln!(input, "    -fixed_volume").unwrap();
                writeln!(input, "    -volume {:.12e}", volume.0).unwrap();
            }
            _ => unreachable!("gas components require a material-closed headspace"),
        }
        for (phase, _, partial_pressure) in &problem.gases {
            writeln!(input, "    {phase} {:.12e}", partial_pressure).unwrap();
        }
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
    let mut molalities: Vec<&str> = Vec::new();
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
    writeln!(input, "END").unwrap();
    input
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
