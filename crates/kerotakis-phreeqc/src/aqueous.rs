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
    species, Equilibrator, Event, Headspace, Kelvin, Moles, Phase, Portion, Provenance,
    SolutionInfo, SolveError, SpeciesDetail, SpeciesId, SurfaceOccupancy, SurfaceSiteKind,
    SurfaceSites, SurfaceSorbate, ThermalMode, Vessel,
};

use crate::PhreeqcError;
#[cfg(feature = "engine")]
use crate::{databases, Phreeqc};

use crate::derived::{self, DerivedRole, ATMOSPHERIC};

const WATER_MOLAR_MASS: f64 = 18.015;
const TRACE: f64 = 1e-12;

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
    /// concentrated brines — but it only knows the major-ion elements.
    #[cfg(feature = "engine")]
    brine: Phreeqc,
    /// Content-addressed result cache: same species set, T and P is the
    /// same answer (PLAN.md, P2). Keyed by database + canonical input.
    #[allow(clippy::type_complexity)]
    cache: std::collections::HashMap<
        String,
        (
            Vec<Vec<String>>,
            Vec<SpeciesDetail>,
            Vec<(String, f64)>,
            bool,
            bool,
        ),
    >,
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
    fn run_raw(&mut self, db_tag: &str, input: &str) -> Result<SolveOutput, SolveError> {
        #[cfg(feature = "engine")]
        {
            let engine = match db_tag {
                "minteq.v4" => &mut self.organic,
                "pitzer" => &mut self.brine,
                _ => &mut self.inorganic,
            };
            engine.run(input).map_err(|e| {
                // The input is the whole question; when the engine refuses
                // it, being able to see it is the difference between a
                // diagnosis and a guess.
                if std::env::var("KERO_DUMP_INPUT").is_ok() {
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
        let (mut lo, mut hi) = (-10.0f64, 17.0f64);
        let mut best: Option<SolveOutput> = None;
        let mut last_sum: Option<f64> = None;
        // Whether the residual was ever seen on both sides of zero. A
        // bisection that only ever approaches from one side never bracketed
        // a root: it walked to an edge, and the edge is not a measurement.
        let (mut saw_below, mut saw_above) = (false, false);
        for _ in 0..34 {
            let mid = 0.5 * (lo + hi);
            let input = build_input_at(vessel, problem, db_tag, Some((mid, coupling)));
            // A single awkward trial must not end the search. PHREEQC will
            // refuse some electron activities outright — a residual of one
            // part in a hundred thousand on chloride is enough — and those
            // are scattered through the range rather than at its edges.
            // Aborting on the first one threw away a titration the bisection
            // had very nearly solved, and reported the reagents as unreacted.
            let Ok(out) = self.run_raw(db_tag, &input) else {
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
            if std::env::var("KERO_REDOX").is_ok() {
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
            if hi - lo < 1e-6 {
                break;
            }
        }
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
        if residual > 1e-9_f64.max(1e-4 * coupling.scale) {
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
        let mut out = best.ok_or_else(|| SolveError::NotConverged {
            solver: "phreeqc-aqueous (redox)".to_string(),
            detail: "the electron balance did not converge".to_string(),
        })?;
        out.pe_undetermined = !(saw_below && saw_above);
        Ok(out)
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
        let mut organic = Phreeqc::with_database(databases::MINTEQ_V4)?;
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
            cache: std::collections::HashMap::new(),
            cache_hits: 0,
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
            hook: None,
            neutralisation: std::collections::HashMap::new(),
        })
    }

    /// Cache hits so far (content-addressed on the canonical PHREEQC input,
    /// which is a deterministic function of the vessel state).
    pub fn cache_hits(&self) -> usize {
        self.cache_hits
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
                .map(
                    |(k, (rows, species, saturation, redox, pe_ok))| CacheEntry {
                        key: k.clone(),
                        rows: rows.clone(),
                        species: species.clone(),
                        saturation: saturation.clone(),
                        redox_adjusted: *redox,
                        pe_undetermined: !*pe_ok,
                    },
                )
                .collect(),
            neutralisation_kj_per_mol,
        }
    }

    /// Load a pre-warmed cache. Entries already present are kept.
    pub fn import_cache(&mut self, data: CacheData) -> usize {
        let before = self.cache.len();
        for e in data.entries {
            self.cache.entry(e.key).or_insert((
                e.rows,
                e.species,
                e.saturation,
                e.redox_adjusted,
                !e.pe_undetermined,
            ));
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
                    dataset: format!("{db_tag}.dat"),
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
            let engine = match db_tag {
                "minteq.v4" => &mut self.organic,
                "pitzer" => &mut self.brine,
                _ => &mut self.inorganic,
            };
            let outcome = match engine.run(&input) {
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
                dataset: format!("{db_tag}.dat"),
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
            DerivedRole::Solvent => kgw += p.moles.0 * WATER_MOLAR_MASS / 1000.0,
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

    if kgw <= 0.0 || solutes == 0 {
        return None;
    }
    // Every derived candidate phase whose elements can reach solution can
    // precipitate, amount 0 if no solid exists yet.
    for cand in derived::candidate_phases() {
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
        if all_present && !listed && reachable {
            phases.push((cand.name.clone(), 0.0, 0.0));
        }
    }
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

impl Equilibrator for PhreeqcEquilibrator {
    fn name(&self) -> &'static str {
        "phreeqc-aqueous"
    }

    fn applies(&self, vessel: &Vessel) -> bool {
        partition(vessel).is_some()
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
            return Ok(Vec::new());
        };
        *vessel = solved;
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
}

impl PhreeqcEquilibrator {
    /// One pass at the vessel's current temperature. Returns the reaction
    /// heat rather than applying it, so the caller can iterate temperature
    /// and composition to a common answer instead of reporting one solved
    /// before the other.
    fn solve_once(&mut self, vessel: &mut Vessel) -> Result<(Vec<Event>, f64), SolveError> {
        let Some(mut problem) = partition(vessel) else {
            return Ok((Vec::new(), 0.0));
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
            + 2.0 * problem.phases.iter().map(|(_, n, _)| n).sum::<f64>())
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
        } else if potential_molality > 1.0 && pitzer_capable && problem.surfaces.is_empty() {
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
        problem.phases.retain(|(name, moles, _)| {
            if idx.has_phase(name) {
                return true;
            }
            if *moles > 0.0 {
                if let Some(p) = derived::phase_by_name(name) {
                    if p.waters == 0.0 {
                        for (el, c) in &p.elements {
                            freed.push((el.clone(), c * moles));
                        }
                        freed_phases.push((name.clone(), *moles));
                        return false;
                    }
                }
                return true;
            }
            false
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

        // Content-addressed cache: database + input string is a
        // deterministic canonicalisation of (species set, amounts, T) — same
        // state, same answer, no engine call.
        let mut coupling_failed: Option<String> = None;
        let (rows, speciation, saturation, redox_adjusted, pe_determined) = if let Some(hit) =
            self.cache.get(&key)
        {
            self.cache_hits += 1;
            hit.clone()
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
            if std::env::var("KERO_DUMP_INPUT").as_deref() == Ok("all") {
                eprintln!("--- PHREEQC input ---\n{input}---");
            }
            let out = match redox_coupling(&problem, db_tag) {
                Some(coupling) => match self.solve_coupled(vessel, &problem, db_tag, &coupling) {
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
                        self.run_raw(db_tag, &input)?
                    }
                },
                None => match self.run_raw(db_tag, &input) {
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
                    Err(e) => match condense_supersaturated(&problem) {
                        Some(recast) => {
                            let recast_input = build_input(vessel, &recast, db_tag);
                            self.run_raw(db_tag, &recast_input).map_err(|_| e)?
                        }
                        None => return Err(e),
                    },
                },
            };
            let pe_determined = !out.pe_undetermined;
            let rows = out.selected;
            let speciation = parse_species_distribution(&out.report);
            let saturation = parse_saturation_indices(&out.report);
            let redox_adjusted = out.report.contains("Adjusted to redox equilibrium");
            if self.cache.len() >= 10_000 {
                self.cache.clear(); // simple bound; refine when profiling says so
            }
            self.cache.insert(
                key,
                (
                    rows.clone(),
                    speciation.clone(),
                    saturation.clone(),
                    redox_adjusted,
                    pe_determined,
                ),
            );
            (rows, speciation, saturation, redox_adjusted, pe_determined)
        };
        let value = |column: &str| -> Option<f64> {
            let idx = rows.first()?.iter().position(|h| h == column)?;
            rows.last()?.get(idx)?.parse().ok()
        };

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
        let mut new_ions: Vec<(String, f64)> = Vec::new();
        let mut unnameable: Vec<(String, f64)> = Vec::new();
        if std::env::var("KERO_READBACK").is_ok() {
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
        let coupled: Vec<String> = valence_totals(&problem, db_tag);
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
            let bound: f64 = new_surfaces
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
                    for (candidate, moles) in &mut new_ions {
                        if candidate.split('(').next().unwrap_or(candidate) == base {
                            *moles *= scale;
                        }
                    }
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
                Some(DerivedRole::Solvent) => contents.push(Portion {
                    species: p.species.clone(),
                    moles: Moles(solvent_kgw_out * 1000.0 / WATER_MOLAR_MASS),
                    phase: p.phase,
                }),
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
                Some(_) => {}
            }
        }
        for (el, moles) in &new_ions {
            if *moles > TRACE {
                let ion = derived::booking_ion(el).expect("booking ion covered by tests");
                // Moles of an *element* are not moles of the ion that
                // carries it unless the ion holds exactly one atom of it.
                // Nitrogen booked as N2 counted twice, so a beaker of
                // silver nitrate gained a quarter of a percent of nitrogen
                // every time it was touched — the element total went in as
                // a molecule count and came back out as an atom count.
                let base = el.split('(').next().unwrap_or(el);
                let per_ion = species::lookup_key(ion)
                    .and_then(|d| kerotakis_core::stoich::parse_formula(d.formula).ok())
                    .and_then(|f| f.counts.get(base).copied())
                    .filter(|n| *n > 0.0)
                    .unwrap_or(1.0);
                contents.push(Portion {
                    species: SpeciesId::new(ion),
                    moles: Moles(*moles / per_ion),
                    phase: Phase::Aqueous,
                });
            }
        }
        for (_, species, moles) in &new_gases {
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
        for (phase, moles) in &new_phases {
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
        for (phase, moles) in &freed_phases {
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
        let neutralised =
            0.5 * (a_before.abs() + (a_after - a_before).abs() - a_after.abs()).max(0.0);
        vessel.solute_charge = a_after;

        vessel.contents = contents;
        vessel.surfaces = new_surfaces;
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
        let mut q_joules = 0.0; // heat released into the vessel
        if matches!(vessel.thermal_mode, ThermalMode::Adiabatic)
            && neutralised > kerotakis_core::OBSERVABLE_MOLES
        {
            if let Some(dh) = self.neutralisation.get(db_tag) {
                q_joules -= dh * 1000.0 * neutralised;
            }
        }
        if matches!(vessel.thermal_mode, ThermalMode::Adiabatic) {
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
        }

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
        let _ = redox_adjusted;
        // The redox split, read back from the per-valence totals asked for
        // in `build_input`. A state at zero is kept out: "0 mol of Mn(VII)"
        // is true and is not what anyone means by a distribution.
        let mut redox: Vec<kerotakis_core::RedoxState> = Vec::new();
        for column in valence_totals(&problem, db_tag) {
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
            (None, n) if n > 1 && redox_coupling(&problem, db_tag).is_none() => {
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
                dataset: format!("{db_tag}.dat"),
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
        for (column, moles) in &unnameable {
            events.push(Event::NotYetModeled {
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
            events.push(Event::NotYetModeled {
                vessel: vessel.id,
                what: format!(
                    "a real beaker would not stay like this: the solution is supersaturated against {}. Those phases are in {db_tag}.dat but not in this lab's registry, so nothing can precipitate out of it here",
                    describe(&unnamed)
                ),
            });
        }
        if !withheld.is_empty() {
            let t_c = vessel.temperature.to_celsius();
            events.push(Event::NotYetModeled {
                vessel: vessel.id,
                what: format!(
                    "the solution is supersaturated against {}, which this lab is deliberately holding back: it is the more stable solid, but at {t_c:.0} °C the metastable one forms first and stays. That is a claim about rates, not about equilibrium, and it is curated rather than computed",
                    describe(&withheld)
                ),
            });
        }

        Ok((events, q_joules))
    }
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
            writeln!(input, "    {phase} {target_si} {moles:.12e}").unwrap();
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
    if !problem.surfaces.is_empty() {
        let molalities = if db_tag == "minteq.v4" {
            "Hfo_sOZn+ Hfo_wOZn+ Hfo_sSO4- Hfo_sOHSO4-2 Hfo_wSO4- Hfo_wOHSO4-2"
        } else {
            "Hfo_sOZn+ Hfo_wOZn+ Hfo_wSO4- Hfo_wOHSO4-2"
        };
        writeln!(input, "    -molalities {molalities}").unwrap();
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
fn parse_species_distribution(output: &str) -> Vec<SpeciesDetail> {
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
