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
    species, Equilibrator, Event, Kelvin, Moles, Phase, Portion, Provenance, SolutionInfo,
    SolveError, SpeciesDetail, SpeciesId, ThermalMode, Vessel,
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
        ),
    >,
    cache_hits: usize,
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
                lo = mid;
            } else {
                hi = mid;
            }
            if hi - lo < 1e-6 {
                break;
            }
        }
        best.ok_or_else(|| SolveError::NotConverged {
            solver: "phreeqc-aqueous (redox)".to_string(),
            detail: "the electron balance did not converge".to_string(),
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
        Ok(PhreeqcEquilibrator {
            inorganic: Phreeqc::with_database(databases::WATEQ4F)?,
            organic: Phreeqc::with_database(databases::MINTEQ_V4)?,
            brine: Phreeqc::with_database(databases::PITZER)?,
            cache: std::collections::HashMap::new(),
            cache_hits: 0,
            hook: None,
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
        CacheData {
            entries: self
                .cache
                .iter()
                .map(|(k, (rows, species, saturation, redox))| CacheEntry {
                    key: k.clone(),
                    rows: rows.clone(),
                    species: species.clone(),
                    saturation: saturation.clone(),
                    redox_adjusted: *redox,
                })
                .collect(),
        }
    }

    /// Load a pre-warmed cache. Entries already present are kept.
    pub fn import_cache(&mut self, data: CacheData) -> usize {
        let before = self.cache.len();
        for e in data.entries {
            self.cache
                .entry(e.key)
                .or_insert((e.rows, e.species, e.saturation, e.redox_adjusted));
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
        for db_tag in ["wateq4f", "minteq.v4", "pitzer"] {
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
    /// Every element to read back: dissolved totals plus the elements of all
    /// involved phases (a dissolving solid puts its elements into solution
    /// even when none started there).
    elements: Vec<String>,
}

/// Partition the vessel into a PHREEQC problem, or None if this vessel is
/// not an aqueous problem this mapper fully understands.
fn partition(vessel: &Vessel) -> Option<Problem> {
    let mut kgw = 0.0;
    let mut totals: Vec<(String, f64)> = Vec::new();
    let mut phases: Vec<(String, f64, f64)> = Vec::new();
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
    // Atmospheric venting: a gas phase joins when its non-water elements
    // are present (derived from the gas formula itself).
    for (phase, _, target_si, _) in ATMOSPHERIC {
        let gas_formula = phase.trim_end_matches("(g)");
        let required = crate::dbindex::parse_formula(gas_formula).unwrap_or_default();
        let all_present = required
            .keys()
            .filter(|el| *el != "O" && *el != "H")
            .all(|el| elements.iter().any(|e| e == el));
        let listed = phases.iter().any(|(name, ..)| name == phase);
        if all_present && !listed {
            phases.push((phase.to_string(), 0.0, *target_si));
        }
    }
    Some(Problem {
        kgw,
        totals,
        phases,
        elements,
    })
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
        let start = vessel.clone();
        let t0 = start.temperature.0;
        let mut guess = t0;
        let mut settled: Option<(Vessel, Vec<Event>, f64)> = None;

        for _ in 0..8 {
            let mut trial = start.clone();
            trial.temperature = Kelvin(guess);
            let (events, q_joules) = self.solve_once(&mut trial)?;
            let cp = trial.heat_capacity();
            let delta = if cp > 0.0 { q_joules / cp } else { 0.0 };
            let next = t0 + delta;
            let converged = (next - guess).abs() < 0.05;
            settled = Some((trial, events, next));
            guess = next;
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
        } else if potential_molality > 1.0 && pitzer_capable {
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
        // Phases the routed database does not define must not reach the
        // input. Zero-amount candidates are dropped; a solid-backed
        // anhydrous phase (e.g. solid KCl on the wateq4f route — Sylvite is
        // pitzer-only) dissolves into element totals instead, i.e. is
        // treated as freely soluble in that database's domain. Hydrate
        // phases are kept so the engine errors honestly rather than the
        // ledger losing their crystal water silently.
        let idx = derived::index_for(db_tag);
        let mut freed: Vec<(String, f64)> = Vec::new();
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
                        return false;
                    }
                }
                return true;
            }
            false
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
        let (rows, speciation, saturation, redox_adjusted) = if let Some(hit) = self.cache.get(&key)
        {
            self.cache_hits += 1;
            hit.clone()
        } else {
            // Redox elements that equilibrate on a bench timescale are
            // coupled and pe is solved for; everything else keeps the
            // oxidation state it was added in. Which is which is curated —
            // see FAST_REDOX — because it is a claim about rates.
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
                None => self.run_raw(db_tag, &input)?,
            };
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
                ),
            );
            (rows, speciation, saturation, redox_adjusted)
        };
        let value = |column: &str| -> Option<f64> {
            let idx = rows.first()?.iter().position(|h| h == column)?;
            rows.last()?.get(idx)?.parse().ok()
        };

        // Read back: element totals (mol/kgw) and phase amounts (mol).
        // Molalities are per kg of *equilibrated* water (mass_H2O), which
        // differs slightly from the input water mass through speciation.
        let kgw_out = value("mass_H2O").ok_or_else(|| missing("mass_H2O"))?;
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
        let mut new_phases: Vec<(String, f64)> = Vec::new();
        for (phase, ..) in &problem.phases {
            let moles = value(phase).ok_or_else(|| missing(phase))?;
            new_phases.push((phase.clone(), moles));
        }
        let ph = value("pH").ok_or_else(|| missing("pH"))?;
        let mu = value("mu").ok_or_else(|| missing("mu"))?;

        // Rebuild the vessel inventory: water stays; solutes are replaced by
        // the computed state.

        let mut events = Vec::new();
        let mut contents = Vec::new();
        for p in &vessel.contents {
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
                    moles: Moles(kgw_out * 1000.0 / WATER_MOLAR_MASS),
                    phase: p.phase,
                }),
                // Matter this engine does not model passes through
                // untouched. The rebuild replaces the vessel's contents
                // with the computed state, so anything without a role used
                // to be *destroyed* here — not ignored, deleted. It was
                // invisible only because an unmodelled species also made
                // `partition` decline, so the solver never ran at all.
                None => {
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
            } else if let Some((_, gas, _, water_coproduct)) =
                ATMOSPHERIC.iter().find(|(name, ..)| name == phase)
            {
                // Escaped the open vessel: reported, not booked — the
                // balance notices the loss. The water co-product of the
                // gas-forming reaction stays behind.
                // Recorded whatever the size. Gating this on
                // observability meant a third of a micromole of CO2 could
                // leave an open beaker with no entry against it — and the
                // water co-product below was skipped with it, so the vessel
                // lost matter twice over. `Event::is_observable` decides
                // what gets *shown*.
                if *moles > TRACE {
                    events.push(Event::GasEvolved {
                        vessel: vessel.id,
                        species: SpeciesId::new(gas),
                        moles: Moles(*moles),
                    });
                    // The water this reaction makes is *not* added here.
                    // It used to be, and it had to be while the solvent was
                    // carried over unchanged from the previous step. Now
                    // the solvent is rebuilt from PHREEQC's own `mass_H2O`,
                    // which already counts water produced and consumed by
                    // the reaction — adding it again put the mass in twice,
                    // and a fizzing beaker lost 1.02 g where it should have
                    // lost the full 1.69 g of carbon dioxide.
                    let _ = water_coproduct;
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

        vessel.contents = contents;

        // Reaction heat: curated dissolution enthalpies feed the energy
        // balance (PLAN.md). Dissolution of an endothermic salt cools the
        // vessel; precipitation releases the corresponding heat. v1 applies
        // the temperature change once rather than iterating solver ↔ T; the
        // shifts at teaching concentrations are small against the ~25–100 °C
        // range of the database.
        let mut q_joules = 0.0; // heat released into the vessel
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
        redox.sort_by(|a, b| {
            a.element
                .cmp(&b.element)
                .then(b.molality.total_cmp(&a.molality))
        });

        let info = SolutionInfo {
            redox,
            pe: redox_constrained.then(|| value("pe")).flatten(),
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

/// The gas phase an open vessel's redox state is set by, and its share of
/// the atmosphere. log10(0.21) = −0.68.
const ATMOSPHERIC_OXYGEN: &str = "O2(g)";
const ATMOSPHERIC_LOG_PO2: &str = "-0.68";

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
    Some(RedoxCoupling { target, columns })
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
        None if derived::index_for(db_tag).has_phase(ATMOSPHERIC_OXYGEN) => writeln!(
            input,
            "    pe        4  {ATMOSPHERIC_OXYGEN}  {ATMOSPHERIC_LOG_PO2}"
        )
        .unwrap(),
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
    writeln!(input, "    -totals   {}", totals.join(" ")).unwrap();
    if !problem.phases.is_empty() {
        let phases: Vec<&str> = problem.phases.iter().map(|(p, ..)| p.as_str()).collect();
        writeln!(input, "    -equilibrium_phases {}", phases.join(" ")).unwrap();
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
