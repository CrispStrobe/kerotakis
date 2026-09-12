//! The solver router and the L0 safety screen traits, with the v0
//! implementations: a physical mixing equilibrator (mass + energy balance,
//! no chemistry) and a permissive safety screen.
//!
//! The real L2/L2g/L3 engines plug in behind `Equilibrator`; the real
//! reactive-group matrix plugs in behind `SafetyScreen` (PLAN.md, P1/P2).

use crate::ops::Event;
use crate::species::{self, Phase, SpeciesId};
use crate::units::{Kelvin, Moles};
use crate::vessel::{ThermalMode, Vessel};

// ── ARCH-010: structured capability/validity reports ───────────────

/// Why a solver does or does not apply to a given vessel state.
#[derive(Debug, Clone)]
pub enum Applicability {
    /// The solver can handle this vessel state.
    Applicable,
    /// The solver cannot handle this state and explains why.
    NotApplicable { reason: String },
    /// The solver can handle it partially (some species/phases covered).
    Partial {
        covered: Vec<String>,
        uncovered: Vec<String>,
    },
}

impl Applicability {
    pub fn is_applicable(&self) -> bool {
        matches!(
            self,
            Applicability::Applicable | Applicability::Partial { .. }
        )
    }

    pub fn is_fully_applicable(&self) -> bool {
        matches!(self, Applicability::Applicable)
    }
}

/// A structured report of what a solver can do with a given vessel.
#[derive(Debug, Clone)]
pub struct CapabilityReport {
    pub solver: &'static str,
    pub applicability: Applicability,
    /// Whether this solver claims to handle chemistry (not just physics).
    pub is_chemistry: bool,
    /// Optional validity bounds on the result.
    pub validity: Option<ValidityBounds>,
}

/// Bounds within which the solver's result is expected to be valid.
#[derive(Debug, Clone)]
pub struct ValidityBounds {
    pub temperature_range: Option<(f64, f64)>,
    pub pressure_range: Option<(f64, f64)>,
    pub ionic_strength_max: Option<f64>,
}

#[derive(Debug, thiserror::Error)]
pub enum SolveError {
    #[error("{solver} could not solve this state: {detail}")]
    NotConverged { solver: String, detail: String },
}

/// Scientific authority of a solver route, independent of its display name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SolverRouteKind {
    Computed,
    Curated,
    Qualitative,
}

/// Re-equilibrates one vessel after an operator touched it.
pub trait Equilibrator {
    fn name(&self) -> &'static str;
    fn route_kind(&self) -> SolverRouteKind {
        SolverRouteKind::Computed
    }
    /// Whether this solver has anything to say about this vessel's state.
    fn applies(&self, _vessel: &Vessel) -> bool {
        true
    }
    /// Whether a *chemistry* engine claims this state — one that decides
    /// what reacts, as opposed to the physical mixing pass that moves heat
    /// around and the honesty pass that only reports.
    ///
    /// The bench needs this to tell two very different situations apart:
    /// a solver that examined the vessel and found no reaction, and no
    /// solver having examined it at all. Reporting the second as the first
    /// turns a gap in our modelling into a claim about the world.
    fn chemistry_applies(&self, vessel: &Vessel) -> bool {
        self.applies(vessel)
    }
    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError>;

    /// Record a model-backed operation that runs directly on the bench rather
    /// than through an equilibrium pass. Most equilibrators ignore this;
    /// [`SolverStack`] retains it beside equilibrium routes so diagnostics can
    /// attribute a pressure reading, gas test, or explicitly requested
    /// curated reaction to the model that actually produced it.
    fn record_route(&mut self, _route: SolverRoute) {}

    /// Missing time models relevant to a wait, without claiming a rate from an
    /// equilibrium calculation. Ordinary additions need not repeat these notes.
    fn time_boundaries(&self, _vessel: &Vessel) -> Vec<Event> {
        Vec::new()
    }

    /// Mix two solutions by fraction into a target vessel using the solver's
    /// native mixing (PHREEQC MIX). Returns `None` if the solver does not
    /// support native mixing; the caller falls back to `equilibrate`.
    fn mix(
        &mut self,
        _vessel: &mut Vessel,
        _soln_a: &Vessel,
        _frac_a: f64,
        _soln_b: &Vessel,
        _frac_b: f64,
    ) -> Option<Result<Vec<Event>, SolveError>> {
        None
    }

    /// ARCH-012: produce a delta and events without mutating the vessel.
    ///
    /// The default clones the vessel, runs `equilibrate()` on the clone,
    /// and diffs the result into a `StateDelta`. Solvers that can produce
    /// deltas directly (without cloning) override this for efficiency.
    fn equilibrate_delta(
        &mut self,
        vessel: &Vessel,
    ) -> Result<(crate::delta::StateDelta, Vec<Event>), SolveError> {
        let mut copy = vessel.clone();
        let events = self.equilibrate(&mut copy)?;
        Ok((
            crate::orchestrator::diff_vessels(vessel, &copy, self.name()),
            events,
        ))
    }

    /// ARCH-010: structured capability report.
    /// Default adapter wraps the existing boolean `applies()`/`chemistry_applies()`.
    fn capability(&self, vessel: &Vessel) -> CapabilityReport {
        let applicability = if self.applies(vessel) {
            Applicability::Applicable
        } else {
            Applicability::NotApplicable {
                reason: format!("{} does not apply to this vessel state", self.name()),
            }
        };
        CapabilityReport {
            solver: self.name(),
            applicability,
            is_chemistry: self.chemistry_applies(vessel),
            validity: None,
        }
    }
}

/// Runs every applicable solver in order, concatenating their events. The
/// order is the routing: physics first, chemistry engines next, the honesty
/// pass last.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SolverRouteOutcome {
    NotApplicable,
    Succeeded { event_count: usize },
    Failed,
}

/// Machine-readable evidence for the most recent stack equilibrium pass.
/// This deliberately sits beside the stack rather than in rendered events:
/// observing routing must not alter a simulation's event stream.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SolverRoute {
    pub solver: String,
    pub kind: SolverRouteKind,
    pub chemistry: bool,
    pub outcome: SolverRouteOutcome,
    /// The vessel this pass examined. A step may equilibrate more than one
    /// vessel, and a routing record that cannot say which one it belongs to
    /// can be read against the wrong beaker.
    ///
    /// `serde(default)` so route records written before this field existed
    /// still load; absent rather than null on the wire.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vessel: Option<crate::vessel::VesselId>,
    /// The solver's own sentence for declining, where it gives one.
    ///
    /// `applies()` answers yes or no; `capability()` answers why. Only a
    /// decline pays for the second call, and only a solver that overrides
    /// `capability()` says anything a reader could not have guessed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// The sentence a solver gives for declining this vessel, if it gives one.
///
/// Kept beside `SolverRoute` rather than inside `applies()` because it costs
/// a second pass over the vessel: the stack asks for it only where a solver
/// has already declined, so an applicable route pays nothing.
pub fn decline_reason(solver: &dyn Equilibrator, vessel: &Vessel) -> Option<String> {
    match solver.capability(vessel).applicability {
        Applicability::NotApplicable { reason } => Some(reason),
        Applicability::Applicable | Applicability::Partial { .. } => None,
    }
}

pub struct SolverStack {
    pub solvers: Vec<Box<dyn Equilibrator>>,
    pub last_routes: Vec<SolverRoute>,
}

impl SolverStack {
    pub fn new(solvers: Vec<Box<dyn Equilibrator>>) -> Self {
        SolverStack {
            solvers,
            last_routes: Vec::new(),
        }
    }
}

impl Equilibrator for SolverStack {
    fn record_route(&mut self, route: SolverRoute) {
        self.last_routes.push(route);
    }

    fn time_boundaries(&self, vessel: &Vessel) -> Vec<Event> {
        self.solvers
            .iter()
            .flat_map(|s| s.time_boundaries(vessel))
            .collect()
    }

    fn name(&self) -> &'static str {
        "solver-stack"
    }

    fn chemistry_applies(&self, vessel: &Vessel) -> bool {
        self.solvers.iter().any(|s| s.chemistry_applies(vessel))
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.last_routes.clear();
        let mut events = Vec::new();
        for solver in &mut self.solvers {
            let solver_name = solver.name().to_string();
            let kind = solver.route_kind();
            let chemistry = solver.chemistry_applies(vessel);
            if !solver.applies(vessel) {
                self.last_routes.push(SolverRoute {
                    solver: solver_name,
                    kind,
                    chemistry,
                    outcome: SolverRouteOutcome::NotApplicable,
                    vessel: Some(vessel.id),
                    reason: decline_reason(&**solver, vessel),
                });
                continue;
            }
            match solver.equilibrate(vessel) {
                Ok(mut more) => {
                    self.last_routes.push(SolverRoute {
                        solver: solver_name,
                        kind,
                        chemistry,
                        outcome: SolverRouteOutcome::Succeeded {
                            event_count: more.len(),
                        },
                        vessel: Some(vessel.id),
                        reason: None,
                    });
                    // Gas this solver sent out of the vessel, booked on the
                    // step's snapshot before the next solver runs: the
                    // aqueous tail prices the whole step's heat, and the
                    // carbon dioxide a curated row evolved ahead of it must
                    // not cease to exist on the way. `GasContained` stays a
                    // portion and is not an outward transfer.
                    if let Some(start) = vessel.step_start.as_mut() {
                        for event in &more {
                            match event {
                                Event::GasEvolved { species, moles, .. } => {
                                    start.note_gas_out(species, moles.0);
                                }
                                Event::GasAbsorbed { species, moles, .. } => {
                                    start.note_gas_out(species, -moles.0);
                                }
                                _ => {}
                            }
                        }
                    }
                    events.append(&mut more);
                }
                // One solver failing must not silence the rest. The stack is
                // a sequence of independent questions — what dissolves, what
                // burns, what state the solvent is in — and an aqueous
                // engine that cannot answer the first has nothing to say
                // about the third. Aborting here left water liquid at
                // −24 °C, because the freezing pass never ran once PHREEQC
                // had declined the solution.
                Err(e) => {
                    self.last_routes.push(SolverRoute {
                        solver: solver_name.clone(),
                        kind,
                        chemistry,
                        outcome: SolverRouteOutcome::Failed,
                        vessel: Some(vessel.id),
                        reason: Some(e.to_string()),
                    });
                    events.push(Event::SolverFailed {
                        vessel: vessel.id,
                        solver: solver_name,
                        detail: e.to_string(),
                    });
                }
            }
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
        for solver in &mut self.solvers {
            if let Some(result) = solver.mix(vessel, soln_a, frac_a, soln_b, frac_b) {
                return Some(result);
            }
        }
        None
    }
}

/// How dangerous the real-world version of this state is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Caution,
    Danger,
}

/// L0's judgment of a prospective state.
///
/// This is a *pedagogical* tool: known hazards **warn strongly and then
/// proceed** — being precise about what would happen is the lesson, and the
/// virtual lab is the one place it can be watched safely. `Veto` exists for
/// the product-safety boundary (states the product must not compute at all,
/// e.g. anything shading into synthesis-oracle territory — see PLAN.md,
/// "What this will not do"), not for curriculum hazards.
#[derive(Debug, Clone, PartialEq)]
pub enum SafetyVerdict {
    Allow,
    /// Proceed, but emit a strong warning first: what the hazard is, and what
    /// it would mean outside the simulation.
    Warn {
        severity: Severity,
        /// Stable machine identity of the hazard (e.g.
        /// `bleach-ammonia-chloramine`). The prose fields are localized on
        /// their way to the reader, so anything that needs to RECOGNISE a
        /// hazard — a mission contract, a test — must key on this, never
        /// on the wording. Empty when a producer has no curated rule.
        rule: String,
        hazard: String,
        real_world: String,
    },
    /// Refuse entirely. Reserved for the product-safety boundary.
    Veto {
        reason: String,
    },
}

/// L0. Runs before any chemistry, on the prospective state.
pub trait SafetyScreen {
    fn assess(&self, vessel: &Vessel) -> SafetyVerdict;

    /// KID-3: assess a *pour* rather than a state.
    ///
    /// The reactivity screen warns about mixing, and mixing is something a
    /// learner does. A reviewed material recipe that ships an oxidiser and
    /// a reducing agent in one bottle — Lugol's iodine is iodine and
    /// potassium iodide, and has been sold that way for two centuries — is
    /// not a mixture anyone made at the bench, and warning about it teaches
    /// the learner to ignore the banner that matters. Screening the fully
    /// expanded prospective mixture is still right for everything the pour
    /// meets *in the vessel*; what a screen implementation may drop here is
    /// a pair that arrived together in one bottle and was in neither the
    /// vessel nor another bottle beforehand.
    ///
    /// The default keeps the old behaviour exactly, so a screen that does
    /// not care is unaffected.
    fn assess_pour(&self, _before: &Vessel, after: &Vessel) -> SafetyVerdict {
        self.assess(after)
    }
}

/// v0 screen: permissive. The real screen lives in `kerotakis-safety`; this
/// type exists so the loop is wired for L0 from day one.
pub struct PermissiveScreen;

impl SafetyScreen for PermissiveScreen {
    fn assess(&self, _vessel: &Vessel) -> SafetyVerdict {
        SafetyVerdict::Allow
    }
}

/// Physics pass: thermostatted vessels relax to their bath temperature.
///
/// Thermal mixing itself happens in the bench loop when matter enters at a
/// different temperature; by the time this runs, the vessel already has a
/// single well-defined T.
pub struct MixingEquilibrator;

/// The curated liquid–liquid pairs and the computed verdict: which two
/// layers, if any, this vessel's liquids separate into. Two liquids in
/// one vessel are not automatically one solution — the computed activity
/// decides, and where mixing would raise the Gibbs energy the bench
/// shows what a beaker shows. One source of truth for the solver (which
/// reports the layers) and the bench (whose `drain` verb taps them).
/// Growing the table is data work: a pair enters when its UNIFAC groups
/// are curated and the split is oracle-checked.
pub fn layered_pair(vessel: &Vessel) -> Option<(&'static str, &'static str)> {
    const LLE_PAIRS: &[(&str, &str)] = &[
        // (upper by density, lower)
        ("hexane", "water"),
    ];
    for (upper, lower) in LLE_PAIRS {
        let moles_of = |key: &str| -> f64 {
            vessel
                .contents
                .iter()
                .filter(|p| p.species.0 == key && p.phase == Phase::Liquid)
                .map(|p| p.moles.0)
                .sum()
        };
        let a = moles_of(upper);
        let b = moles_of(lower);
        if a > crate::OBSERVABLE_MOLES && b > crate::OBSERVABLE_MOLES {
            let z = a / (a + b);
            if let kerotakis_thermo::lle::LleResult::TwoPhase { .. } =
                kerotakis_thermo::lle::water_hexane_lle(z, vessel.temperature.0)
            {
                return Some((upper, lower));
            }
        }
    }
    None
}

impl Equilibrator for MixingEquilibrator {
    fn name(&self) -> &'static str {
        "mixing-v0"
    }

    fn chemistry_applies(&self, _vessel: &Vessel) -> bool {
        false
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        let mut events = Vec::new();

        if let ThermalMode::Thermostatted(bath) = vessel.thermal_mode {
            if (vessel.temperature.0 - bath.0).abs() > 1e-9 {
                events.push(Event::TemperatureChanged {
                    vessel: vessel.id,
                    from: vessel.temperature,
                    to: bath,
                });
                vessel.temperature = bath;
            }
        }

        if let Some((upper, lower)) = layered_pair(vessel) {
            events.push(Event::LayersFormed {
                vessel: vessel.id,
                upper: SpeciesId::new(upper),
                lower: SpeciesId::new(lower),
            });
        }
        if vessel
            .contents
            .iter()
            .any(|portion| portion.species.0 == "water" && portion.phase == Phase::Liquid)
        {
            for layer in crate::material::immiscible_liquid_layers(vessel) {
                events.push(Event::MaterialLayersFormed {
                    vessel: vessel.id,
                    upper_material: layer.material,
                    lower: SpeciesId::new("water"),
                });
            }
        }

        // A homogeneous catalyst cannot simultaneously be the concentration
        // in a liquid rate law and a pile of sediment. Move declared
        // solution-catalyst inventory into the aqueous phase as soon as a
        // liquid medium exists. This is a phase bookkeeping statement, not a
        // claim that the reduced core engine resolves its individual ions.
        if vessel.liquid_volume().0 > 0.0 {
            let dissolved = vessel
                .contents
                .iter()
                .filter(|portion| {
                    portion.phase == Phase::Solid
                        && crate::kinetics::is_solution_catalyst(&portion.species)
                })
                .map(|portion| (portion.species.clone(), portion.moles))
                .collect::<Vec<_>>();
            for (species, moles) in dissolved {
                vessel.withdraw(&species, moles);
                vessel.deposit(species.clone(), moles, Phase::Aqueous);
                for lot in vessel
                    .lots
                    .iter_mut()
                    .filter(|lot| lot.species == species && lot.phase == Phase::Solid)
                {
                    lot.phase = Phase::Aqueous;
                    lot.suspended_fraction = None;
                }
                events.push(Event::Dissolved {
                    vessel: vessel.id,
                    species,
                    moles,
                });
            }
            if !events.is_empty() {
                vessel.resolved.invalidate();
            }
        }

        let iodine = crate::starch_iodine::iodine_to_dissolve(vessel);
        if iodine.0 > 0.0 {
            let species = SpeciesId::new("I2");
            vessel.withdraw(&species, iodine);
            vessel.deposit(species.clone(), iodine, Phase::Aqueous);
            rephase_lots(vessel, &species, iodine);
            events.push(Event::Dissolved {
                vessel: vessel.id,
                species,
                moles: iodine,
            });
            vessel.resolved.invalidate();
        }

        // Neutral molecular solids with an explicit reviewed room-temperature
        // limit dissolve only up to that finite capacity. This changes phase
        // bookkeeping but makes no pH, ionic-strength, or activity claim.
        for move_ in saturation_moves(vessel) {
            match move_ {
                SaturationMove::Dissolve(solute, moles) => {
                    vessel.withdraw(&solute, moles);
                    vessel.deposit(solute.clone(), moles, Phase::Aqueous);
                    rephase_lots(vessel, &solute, moles);
                    events.push(Event::Dissolved {
                        vessel: vessel.id,
                        species: solute,
                        moles,
                    });
                    vessel.resolved.invalidate();
                }
                SaturationMove::Crystallise(solute, moles) => {
                    vessel.withdraw_phase(&solute, moles, Phase::Aqueous);
                    vessel.deposit(solute.clone(), moles, Phase::Solid);
                    events.push(Event::Precipitated {
                        vessel: vessel.id,
                        species: solute,
                        moles,
                        // A solubility limit is a property of a solution,
                        // so this route is wet by construction.
                        dry: false,
                    });
                    vessel.resolved.invalidate();
                }
                SaturationMove::Supersaturated {
                    species,
                    dissolved,
                    capacity,
                } => events.push(Event::Supersaturated {
                    vessel: vessel.id,
                    species,
                    dissolved,
                    capacity,
                }),
            }
        }
        // K51: and the salts that are past saturation and cannot be made
        // to come out. Silence here is the answer a learner cannot use.
        for gap in unavailable_crystallisations(vessel) {
            events.push(Event::NotYetModeled {
                vessel: vessel.id,
                // Not a gap in our gift: no PHREEQC database vendored with
                // this project defines an acetate solid phase at all.
                cause: crate::ops::NotModelledCause::NotInAnyDatabase,
                what: format!(
                    "the crystallisation of {}: {:.3} mol is dissolved against a limit of {:.3} mol at this temperature, and {}",
                    gap.salt, gap.dissolved.0, gap.capacity.0, gap.reason
                ),
            });
        }

        Ok(events)
    }

    /// ARCH-012: native delta — no clone needed for thermostat check.
    fn equilibrate_delta(
        &mut self,
        vessel: &Vessel,
    ) -> Result<(crate::delta::StateDelta, Vec<Event>), SolveError> {
        use crate::delta::{StateDelta, ThermalDelta};

        let mut delta = StateDelta::new("mixing-v0");
        let mut events = Vec::new();

        if let ThermalMode::Thermostatted(bath) = vessel.thermal_mode {
            if (vessel.temperature.0 - bath.0).abs() > 1e-9 {
                events.push(Event::TemperatureChanged {
                    vessel: vessel.id,
                    from: vessel.temperature,
                    to: bath,
                });
                delta = delta.with_thermal(ThermalDelta::SetTemperature(bath));
            }
        }

        if vessel.liquid_volume().0 > 0.0 {
            for portion in vessel.contents.iter().filter(|portion| {
                portion.phase == Phase::Solid
                    && crate::kinetics::is_solution_catalyst(&portion.species)
            }) {
                delta = delta
                    .with_moles(portion.species.clone(), Phase::Solid, -portion.moles.0)
                    .with_moles(portion.species.clone(), Phase::Aqueous, portion.moles.0);
                events.push(Event::Dissolved {
                    vessel: vessel.id,
                    species: portion.species.clone(),
                    moles: portion.moles,
                });
            }
        }
        let iodine = crate::starch_iodine::iodine_to_dissolve(vessel);
        if iodine.0 > 0.0 {
            let species = SpeciesId::new("I2");
            delta = delta
                .with_moles(species.clone(), Phase::Solid, -iodine.0)
                .with_moles(species.clone(), Phase::Aqueous, iodine.0);
            events.push(Event::Dissolved {
                vessel: vessel.id,
                species,
                moles: iodine,
            });
        }
        for move_ in saturation_moves(vessel) {
            match move_ {
                SaturationMove::Dissolve(solute, moles) => {
                    delta = delta
                        .with_moles(solute.clone(), Phase::Solid, -moles.0)
                        .with_moles(solute.clone(), Phase::Aqueous, moles.0);
                    events.push(Event::Dissolved {
                        vessel: vessel.id,
                        species: solute,
                        moles,
                    });
                }
                SaturationMove::Crystallise(solute, moles) => {
                    delta = delta
                        .with_moles(solute.clone(), Phase::Aqueous, -moles.0)
                        .with_moles(solute.clone(), Phase::Solid, moles.0);
                    events.push(Event::Precipitated {
                        vessel: vessel.id,
                        species: solute,
                        moles,
                        // A solubility limit is a property of a solution,
                        // so this route is wet by construction.
                        dry: false,
                    });
                }
                SaturationMove::Supersaturated {
                    species,
                    dissolved,
                    capacity,
                } => events.push(Event::Supersaturated {
                    vessel: vessel.id,
                    species,
                    dissolved,
                    capacity,
                }),
            }
        }
        // ARCH-012: the delta path must say everything the direct path
        // says, or a host that computes deltas gets a quieter bench than
        // one that does not. K51's refusal is exactly the kind of line
        // that would go missing.
        for gap in unavailable_crystallisations(vessel) {
            events.push(Event::NotYetModeled {
                vessel: vessel.id,
                // Not a gap in our gift: no PHREEQC database vendored with
                // this project defines an acetate solid phase at all.
                cause: crate::ops::NotModelledCause::NotInAnyDatabase,
                what: format!(
                    "the crystallisation of {}: {:.3} mol is dissolved against a limit of {:.3} mol at this temperature, and {}",
                    gap.salt, gap.dissolved.0, gap.capacity.0, gap.reason
                ),
            });
        }

        Ok((delta, events))
    }
}

/// KID-7: which way a saturated solute is moving, and whether it is stuck.
#[derive(Debug, Clone, PartialEq)]
pub enum SaturationMove {
    /// Solid going into solution: there is room for it.
    Dissolve(SpeciesId, Moles),
    /// Solution coming back out onto a seed: the water can no longer hold it
    /// and there is already a crystal of the same solute for it to grow on.
    Crystallise(SpeciesId, Moles),
    /// More in solution than the water can hold, and nothing to build on.
    ///
    /// This is not an error and not a rounding artefact: it is the state a
    /// cooled sugar syrup is actually in, and the reason rock candy needs a
    /// string. Reported rather than silently precipitated, because
    /// precipitating it would erase the experiment.
    Supersaturated {
        species: SpeciesId,
        dissolved: Moles,
        capacity: Moles,
    },
}

/// What the saturation limit says about every solute with a reviewed one.
///
/// Before KID-7 this answered one question — how much more will dissolve —
/// against a single room-temperature number. That made hot water hold no
/// more sugar than cold water, so the one thing every crystal experiment is
/// run to show could not happen. It now reads the limit at the vessel's own
/// temperature and answers in both directions.
/// K51: a salt that is over its solubility and cannot be made to come out,
/// because no shipped database carries the solid it would come out as.
///
/// The reusable hand warmer is a sodium acetate solution held far past
/// saturation; you click the disc, the trihydrate crystallises on the
/// scratch, and the heat of crystallisation is the whole product. This
/// bench cools such a solution from 65 °C to 8 °C and **nothing happens
/// and nothing is said**, which is the worst of the three possible
/// answers.
///
/// It cannot be fixed by a datum or by choosing another database. Every
/// `.dat` vendored with iphreeqc — wateq4f, minteq.v4, minteq, pitzer,
/// sit, llnl — was searched for an acetate solid in its `PHASES` section
/// and there is not one, anywhere. That is not a shipping choice this
/// project made; nobody's PHREEQC database carries one. `saturation_moves`
/// cannot help either: it works on undissociated molecular solutes, and
/// the aqueous engine has already split this salt into sodium and acetate
/// ions, so there is no `NaOAc` portion for it to find.
///
/// So the refusal is the deliverable. The salt is reconstructed from its
/// ions, compared against a curated solubility, and the bench says what it
/// cannot do and why — which is what the learner needed from the moment
/// the beaker refused to freeze.
#[derive(Debug, Clone, PartialEq)]
pub struct UnavailableCrystallisation {
    pub salt: &'static str,
    pub dissolved: Moles,
    pub capacity: Moles,
    pub reason: &'static str,
}

/// (cation key, anion key, salt name, g per 100 mL at 20 °C, g/mol, why).
///
/// Deliberately a short curated list rather than anything derived: a row
/// here is a claim that the solid phase is absent from every shipped
/// database, and that claim is only worth making where somebody has looked.
const UNAVAILABLE_SOLID_PHASES: &[(&str, &str, &str, f64, f64, &str)] = &[(
    "Na+",
    "CH3COO-",
    "sodium acetate",
    46.5,
    82.034,
    "the solid it would crystallise as is sodium acetate trihydrate, and no PHREEQC database vendored with this project defines any acetate solid phase at all — so the aqueous engine has nothing to precipitate and the crystallisation a hand warmer is built on cannot be computed here",
)];

/// Salts held past saturation whose solid the bench cannot form.
pub fn unavailable_crystallisations(vessel: &Vessel) -> Vec<UnavailableCrystallisation> {
    let water_moles = vessel
        .contents
        .iter()
        .filter(|portion| portion.species.0 == SOLVENT && portion.phase == Phase::Liquid)
        .map(|portion| portion.moles.0)
        .sum::<f64>();
    if water_moles <= 0.0 {
        return Vec::new();
    }
    let water_ml = species::lookup_key(SOLVENT)
        .map(|water| water.liters_from_moles(Moles(water_moles)).0 * 1000.0)
        .unwrap_or(0.0);
    let dissolved_ion = |key: &str| {
        vessel
            .contents
            .iter()
            .filter(|p| p.species.0 == key && p.phase == Phase::Aqueous)
            .map(|p| p.moles.0)
            .sum::<f64>()
    };
    UNAVAILABLE_SOLID_PHASES
        .iter()
        .filter_map(
            |(cation, anion, salt, grams_per_100ml, molar_mass, reason)| {
                // The salt is only as present as its scarcer ion: a beaker of
                // sodium chloride and a little acetate is not a concentrated
                // acetate solution.
                let paired = dissolved_ion(cation).min(dissolved_ion(anion));
                let capacity = grams_per_100ml * water_ml / 100.0 / molar_mass;
                (paired > capacity + 1e-12).then_some(UnavailableCrystallisation {
                    salt,
                    dissolved: Moles(paired),
                    capacity: Moles(capacity),
                    reason,
                })
            },
        )
        .collect()
}

pub fn saturation_moves(vessel: &Vessel) -> Vec<SaturationMove> {
    let water_moles = vessel
        .contents
        .iter()
        .filter(|portion| portion.species.0 == SOLVENT && portion.phase == Phase::Liquid)
        .map(|portion| portion.moles.0)
        .sum::<f64>();
    if water_moles <= 0.0 {
        return Vec::new();
    }
    let water_ml = species::lookup_key(SOLVENT)
        .map(|water| water.liters_from_moles(Moles(water_moles)).0 * 1000.0)
        .unwrap_or(0.0);
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for portion in &vessel.contents {
        if !matches!(portion.phase, Phase::Solid | Phase::Aqueous) {
            continue;
        }
        if !seen.insert(portion.species.clone()) {
            continue;
        }
        let Some(data) = species::lookup(&portion.species) else {
            continue;
        };
        let Some(limit) = data.aqueous_solubility_at(vessel.temperature.0) else {
            continue;
        };
        let amount = |phase: Phase| {
            vessel
                .contents
                .iter()
                .filter(|p| p.species == portion.species && p.phase == phase)
                .map(|p| p.moles.0)
                .sum::<f64>()
        };
        let solid = amount(Phase::Solid);
        let aqueous = amount(Phase::Aqueous);
        let capacity = limit * water_ml / 100.0 / data.molar_mass;
        if aqueous > capacity + 1e-12 {
            // A seed is a crystal of the same solute already in the vessel.
            // Without one the solution stays where it is and says so; with
            // one it grows, which is the whole of rock candy.
            if solid > 1e-12 {
                out.push(SaturationMove::Crystallise(
                    portion.species.clone(),
                    Moles(aqueous - capacity),
                ));
            } else {
                out.push(SaturationMove::Supersaturated {
                    species: portion.species.clone(),
                    dissolved: Moles(aqueous),
                    capacity: Moles(capacity),
                });
            }
            continue;
        }
        let dissolves = solid.min((capacity - aqueous).max(0.0));
        if dissolves > 1e-15 {
            out.push(SaturationMove::Dissolve(
                portion.species.clone(),
                Moles(dissolves),
            ));
        }
    }
    out
}

fn rephase_lots(vessel: &mut Vessel, species: &SpeciesId, moles: Moles) {
    let mut remaining = moles.0;
    let mut split = Vec::new();
    for lot in vessel
        .lots
        .iter_mut()
        .filter(|lot| lot.species == *species && lot.phase == Phase::Solid)
    {
        if remaining <= 1e-15 {
            break;
        }
        let moved = remaining.min(lot.moles.0);
        remaining -= moved;
        if moved >= lot.moles.0 - 1e-15 {
            lot.phase = Phase::Aqueous;
            lot.suspended_fraction = None;
        } else {
            lot.moles.0 -= moved;
            let mut aqueous = lot.clone();
            aqueous.moles = Moles(moved);
            aqueous.phase = Phase::Aqueous;
            aqueous.suspended_fraction = None;
            split.push(aqueous);
        }
    }
    vessel.lots.extend(split);
}

/// Freezing and boiling: the solvent is allowed to stop being a liquid.
///
/// Runs *after* the aqueous engine, because where a solution freezes
/// depends on what is dissolved in it, and only the speciation knows how
/// many particles that is. If the vessel turns out to be outside its
/// liquid range, the aqueous answer is **invalidated**. A block of ice does
/// not have a pH; when liquid remains, [`PhaseEquilibrator`] re-runs the
/// chemistry solver against that smaller solvent compartment before the
/// state is exposed. Pure ice removal stops at the explicit low-temperature
/// boundary where salt crystallisation and a solute-specific phase diagram
/// would be required.
pub struct StateEquilibrator;

/// The solvent. Every transition here is water's; a non-aqueous solvent is
/// a separate problem and says so rather than borrowing water's constants.
const SOLVENT: &str = "water";
/// Resolution of the aqueous/ice common-temperature fixed point, K.
///
/// PHREEQC's own temperature/enthalpy fixed point settles to 0.05 K. Once
/// liquid and ice coexist, asking the outer phase loop for a tighter common
/// temperature creates a two-point oscillation that additional passes cannot
/// resolve. A converged state is projected onto the computed liquidus before
/// it is exposed, so callers still receive a phase-consistent temperature.
/// This tolerance is only a coupled-solver stop; an initially supercooled
/// single liquid phase still undergoes its physical transfer.
pub const PHASE_COUPLED_TEMPERATURE_TOLERANCE_K: f64 = 0.05;

/// Where the vessel lands once a phase change has finished and there is
/// energy left over, K.
///
/// Spent AFTER the transfer, so it is spent over the phase the vessel is now
/// in. `before` is the fallback for a vessel that has nothing left to warm —
/// an open flask boiled dry — where dividing by what remains would turn a
/// rounding error into thousands of kelvin.
fn settle_from(vessel: &Vessel, threshold: f64, joules: f64, before: f64) -> f64 {
    if vessel.heat_capacity() > 1e-9 {
        // Integrated over the contents the vessel has now: the leftover is
        // spent against the real curve rather than divided by one value of
        // it.
        vessel.temperature_after_from(threshold, joules)
    } else {
        threshold + joules / before
    }
}

/// The liquidus a brine reaches once `freezing` moles of its solvent have
/// left as pure ice: the same solvent activity, re-evaluated at the molality
/// the smaller liquid compartment then has.
///
/// Pure ice is what leaves, so the particles all stay behind and their
/// molality rises by exactly the ratio of the water. φ is what carries
/// across that, which is why [`crate::states::SolventActivity`] is held as
/// an osmotic coefficient rather than as a water activity.
fn liquidus_after_freezing(
    freezing: f64,
    liquid_moles: f64,
    particle_moles: f64,
    activity: crate::states::SolventActivity,
    pressure_kpa: f64,
) -> f64 {
    let liquid_kg = (liquid_moles - freezing) * 0.018_015;
    if liquid_kg <= 0.0 {
        return f64::NEG_INFINITY;
    }
    crate::states::transitions_with(activity, particle_moles / liquid_kg, pressure_kpa)
        .0
        .freezing_k
}

/// How much of a brine's water freezes in one pass, given that freezing it
/// moves the plateau.
///
/// Two curves in the amount frozen, and they cross once. The vessel's
/// temperature RISES with it, because the excess cooling is paid off by
/// latent heat; the liquidus FALLS with it, because the residual brine is
/// more concentrated and — through the solvent's activity — more than
/// proportionally so. The answer is where they meet: freeze less and the
/// vessel is still below its plateau, freeze more and it is above one it has
/// itself created and would have to melt back.
///
/// Bisection rather than a formula because the liquidus side is a logarithm
/// of an exponential of the molality, and fifty halvings of a bracket that
/// starts at most a few moles wide is exact to far beyond the 0.05 K the
/// coupled loop resolves. `allowed` is the bracket's top — whatever the
/// cooling asked for, capped by any model boundary — so a case where the
/// curves do not cross inside it simply freezes all of it, which is the old
/// behaviour and the right one.
#[allow(clippy::too_many_arguments)]
fn self_consistent_freezing(
    allowed: f64,
    excess_j: f64,
    cp: f64,
    liquidus_now: f64,
    liquid_moles: f64,
    particle_moles: f64,
    activity: crate::states::SolventActivity,
    pressure_kpa: f64,
) -> f64 {
    // `is_nan()` first, deliberately: this guard was written as
    // `!(allowed > 0.0)` so that a NaN allowance takes the early return
    // rather than falling into the bisection below. `allowed <= 0.0`
    // alone is false for NaN, so the two are not interchangeable, and
    // `allowed.max(0.0)` then yields 0.0 for it.
    if allowed.is_nan() || allowed <= 0.0 || particle_moles <= 0.0 || cp <= 0.0 {
        return allowed.max(0.0);
    }
    // Temperature the vessel reaches having spent `x` of the excess cooling
    // on latent heat. Linear in `x` and that is enough: it is only used to
    // locate the crossing, and the settle that follows takes the liquidus.
    let temperature_after =
        |x: f64| liquidus_now - (excess_j - x * crate::states::WATER_H_FUS) / cp;
    let gap = |x: f64| {
        temperature_after(x)
            - liquidus_after_freezing(x, liquid_moles, particle_moles, activity, pressure_kpa)
    };
    if gap(allowed) <= 0.0 {
        return allowed;
    }
    let (mut lo, mut hi) = (0.0, allowed);
    if gap(lo) > 0.0 {
        return 0.0;
    }
    for _ in 0..50 {
        let mid = 0.5 * (lo + hi);
        if gap(mid) < 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// [`self_consistent_freezing`] with the sign turned round: how much ice
/// melts in one pass, given that melting it moves the plateau up.
///
/// The two curves still cross once and still cross the other way. The vessel
/// COOLS as latent heat is absorbed and the liquidus RISES as the brine is
/// diluted, so the gap that was rising in the amount frozen is falling in
/// the amount melted, and the bracket is walked the other direction.
#[allow(clippy::too_many_arguments)]
fn self_consistent_melting(
    allowed: f64,
    available_j: f64,
    cp: f64,
    liquidus_now: f64,
    liquid_moles: f64,
    particle_moles: f64,
    activity: crate::states::SolventActivity,
    pressure_kpa: f64,
) -> f64 {
    // `is_nan()` first, deliberately: this guard was written as
    // `!(allowed > 0.0)` so that a NaN allowance takes the early return
    // rather than falling into the bisection below. `allowed <= 0.0`
    // alone is false for NaN, so the two are not interchangeable, and
    // `allowed.max(0.0)` then yields 0.0 for it.
    if allowed.is_nan() || allowed <= 0.0 || particle_moles <= 0.0 || cp <= 0.0 {
        return allowed.max(0.0);
    }
    let temperature_after =
        |x: f64| liquidus_now + (available_j - x * crate::states::WATER_H_FUS) / cp;
    let gap = |x: f64| {
        temperature_after(x)
            - liquidus_after_freezing(-x, liquid_moles, particle_moles, activity, pressure_kpa)
    };
    if gap(allowed) >= 0.0 {
        return allowed;
    }
    if gap(0.0) < 0.0 {
        return 0.0;
    }
    let (mut lo, mut hi) = (0.0, allowed);
    for _ in 0..50 {
        let mid = 0.5 * (lo + hi);
        if gap(mid) > 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// Where this vessel's water freezes and boils, and which model set the
/// boiling point.
///
/// Exactly what [`StateEquilibrator`] asks itself before it decides whether
/// anything has changed state, and therefore exactly the plateau that
/// arrives in `state_changed.at` when something does. Public because the
/// scene has to carry the same number as a STANDING value: a vessel sitting
/// at 350 K under a partial vacuum is boiling, and a renderer that only
/// hears about it at the moment of transition has to fall back on pure
/// water at one atmosphere in between.
pub fn vessel_transitions(
    vessel: &Vessel,
) -> (crate::states::Transitions, crate::states::BoilingRoute) {
    let (speciated, unspeciated) = dissolved_particles(vessel);
    crate::states::transitions_with(
        solvent_activity_of(vessel, speciated, unspeciated),
        speciated + unspeciated,
        vessel.pressure.0 / 1000.0,
    )
}

/// The solvent's activity as the speciation that answered reports it.
///
/// **P3s, 2026-09-11.** The colligative relation used to be ΔT = K·m, and
/// the particle count was never the part that was wrong: at one molal the
/// dilute law read −3.72 °C against a measured −3.4 while counting exactly
/// the two particles a mole of sodium chloride really makes. What was
/// missing is this — the solvent's own activity — and PLAN's P3s item said
/// so from the beginning ("PHREEQC gives us the osmotic coefficient
/// already").
///
/// It is taken **only from an ion-interaction speciation**, and the reason
/// is written out in [`crate::states::SolventActivity`]: `pitzer.dat`
/// computes a solvent activity from fitted virial coefficients, while the
/// Debye–Hückel datasets report PHREEQC's hard-coded `1 − 0.017·Σm`
/// placeholder, which contains no information about what is dissolved. So
/// the activity is believed when the model that produced it is one, and
/// Raoult's law stands in otherwise — declared as ideal rather than dressed
/// up as measured.
///
/// The water activity arrives in the ordinary species distribution, as the
/// `H2O` row's activity: it has been on the wire and in the replay cache
/// all along, so native and wasm read one number and no new plumbing was
/// needed to reach it.
pub fn vessel_solvent_activity(vessel: &Vessel) -> crate::states::SolventActivity {
    let (speciated, unspeciated) = dissolved_particles(vessel);
    solvent_activity_of(vessel, speciated, unspeciated)
}

/// [`vessel_solvent_activity`] for a caller that has already counted the
/// particles — the whole of the hot path has, and counting them twice per
/// step to ask two questions about the same solution is the kind of waste
/// `OPT-5` exists to keep out.
fn solvent_activity_of(
    vessel: &Vessel,
    speciated: f64,
    unspeciated: f64,
) -> crate::states::SolventActivity {
    let ideal = crate::states::SolventActivity::ideal();
    let Some(info) = vessel.solution.as_ref() else {
        return ideal;
    };
    let ion_interaction = info.provenance.as_ref().is_some_and(|provenance| {
        provenance
            .model
            .starts_with(crate::states::ION_INTERACTION_MODEL_PREFIX)
    });
    if !ion_interaction {
        return ideal;
    }
    let Some(water) = info.species.iter().find(|species| species.name == "H2O") else {
        return ideal;
    };
    // φ belongs to the molality the speciation itself solved at, which is
    // the speciated particles alone — a sucrose no database carries was
    // never in the solve that produced this activity and must not be
    // divided into it.
    crate::states::SolventActivity::from_speciation(water.activity, speciated, info.ionic_strength)
        .with_unspeciated(speciated, unspeciated)
}

/// Dissolved particles, split by whether an aqueous engine counted them:
/// `(speciated, unspeciated)`, both mol per kg of the vessel's liquid water.
fn dissolved_particles(vessel: &Vessel) -> (f64, f64) {
    let speciated: f64 = vessel.solution.as_ref().map_or(0.0, |info| {
        info.species
            .iter()
            .filter(|species| species.name != "H2O")
            .map(|species| species.molality)
            .sum()
    });
    // A colligative property counts particles, and the commonest particle
    // a kitchen dissolves is one no aqueous engine lists: sucrose is a
    // non-electrolyte, so there is no PHREEQC species for it and the
    // speciation that reports the ionic strength cannot report it either.
    // Reading that silence as "no solute" made 20 g of sugar in 100 mL boil
    // at exactly 100.0 °C — the one temperature a sugar solution does not
    // boil at — while the engine already held every constant needed to say
    // 100.3 °C.
    //
    // Only species the registry itself marks as dissolving without
    // speciation are added, which is the registry's own statement that no
    // engine has already counted them; a flag left set after an engine
    // gains the species would double-count it, and belongs fixed in the
    // data rather than guessed at here.
    let solvent = SpeciesId::new(SOLVENT);
    let water_kg = vessel
        .contents
        .iter()
        .filter(|p| p.species == solvent && p.phase == Phase::Liquid)
        .map(|p| p.moles.0 * 0.018_015)
        .sum::<f64>();
    if water_kg <= 0.0 {
        return (speciated, 0.0);
    }
    let water_ml = species::lookup(&solvent)
        .map(|water| water.liters_from_moles(Moles(water_kg / 0.018_015)).0 * 1000.0)
        .unwrap_or(0.0);
    let unspeciated: f64 = vessel
        .contents
        .iter()
        .filter(|p| p.species != solvent)
        .filter_map(|p| {
            let data = species::lookup(&p.species)?;
            if !data.dissolves_without_speciation {
                return None;
            }
            // Only what actually went into solution counts. Sugar past its
            // solubility is sitting on the bottom of the beaker, and a
            // crystal on the bottom raises nothing.
            let dissolved = match data.aqueous_solubility_at(vessel.temperature.0) {
                Some(limit) => p.moles.0.min(limit * water_ml / 100.0 / data.molar_mass),
                None => p.moles.0,
            };
            Some(dissolved / water_kg)
        })
        .sum();
    (speciated, unspeciated)
}

impl Equilibrator for StateEquilibrator {
    fn name(&self) -> &'static str {
        "states"
    }

    fn chemistry_applies(&self, _vessel: &Vessel) -> bool {
        false
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        let mut events = Vec::new();
        let solvent = SpeciesId::new(SOLVENT);

        // Particle count, not solute identity: colligative properties do
        // not care what is dissolved. Taken from the solved speciation
        // where there is one, so ion pairs are counted as the single
        // particles they are rather than as the ions they came from.
        let (speciated, unspeciated) = dissolved_particles(vessel);
        let solute_molality = speciated + unspeciated;
        // …and the count is no longer the whole of it. The relation the
        // temperatures come out of runs on the SOLVENT's activity, which
        // the same speciation reports (P3s, 2026-09-11); the count is what
        // that activity is evaluated at.
        let activity = solvent_activity_of(vessel, speciated, unspeciated);
        // BRD-032: the vessel's own pressure, routed through the BRD-031
        // pack by the solvent's InChIKey rather than by its name. An open
        // beaker reports exactly one atmosphere, takes the curated normal
        // boiling point, and is bit-for-bit what it was before this line.
        let pressure_kpa = vessel.pressure.0 / 1000.0;
        let (t, boiling_route) =
            crate::states::transitions_with(activity, solute_molality, pressure_kpa);
        let now = vessel.temperature.0;

        let liquid_water = vessel
            .contents
            .iter()
            .any(|p| p.species == solvent && p.phase == Phase::Liquid);
        let frozen_water = vessel
            .contents
            .iter()
            .any(|p| p.species == solvent && p.phase == Phase::Solid);

        // Latent heat. A phase change absorbs or releases energy at a
        // constant temperature, which is why a glass of ice water sits at
        // 0 °C until the last ice has gone. Without this the bench simply
        // kept subtracting sensible heat: taking 40 kJ out of 100 mL of
        // water reported −71 °C, when in reality the water reaches 0 °C and
        // then *stays there*, converting the rest of that energy into ice.
        // The plateau is the observation the heating curve is built on.
        //
        // Cp is taken per species and does not vary with phase yet, so ice
        // is warmed and cooled with water's heat capacity. That is a stated
        // approximation, worth about a factor of two on the ice branch.
        let liquid_moles: f64 = vessel
            .contents
            .iter()
            .filter(|p| p.species == solvent && p.phase == Phase::Liquid)
            .map(|p| p.moles.0)
            .sum();
        let frozen_moles: f64 = vessel
            .contents
            .iter()
            .filter(|p| p.species == solvent && p.phase == Phase::Solid)
            .map(|p| p.moles.0)
            .sum();
        let cp = vessel.heat_capacity().max(1e-9);

        // Past the stated range of the model that supplied the activity,
        // no transition is claimed at all. A solution this concentrated
        // still HAS a freezing point and a boiling point; what it does not
        // have is one this bench can compute, and an extrapolated
        // ion-interaction fit reads as authoritatively as a fitted one,
        // which is what makes the silence worth more than the number.
        //
        // Only where a transition would otherwise fire, so a syrup standing
        // at room temperature is not lectured about a boundary it is
        // nowhere near.
        //
        // MELTING is deliberately not gated. Refusing to freeze leaves
        // liquid water below an uncertain freezing point, which is a
        // supercooled state and at least a physical one; refusing to melt
        // would leave ice sitting at room temperature, which is not. The
        // liquidus it melts at is the same uncertain number either way, so
        // the asymmetry is about which wrong answer is recoverable.
        let would_change_state = liquid_water && (now < t.freezing_k || now >= t.boiling_k);
        if would_change_state && !t.within_model_range() {
            events.push(Event::NotYetModeled {
                cause: crate::ops::NotModelledCause::ModelBoundary,
                vessel: vessel.id,
                what: t.solvent.out_of_range_reason(solute_molality),
            });
            return Ok(events);
        }

        if liquid_water && now < t.freezing_k {
            if solute_molality > 0.0
                && t.freezing_k <= crate::states::BRINE_MODEL_MIN_K
                && now <= crate::states::BRINE_MODEL_MIN_K
            {
                events.push(Event::NotYetModeled { cause: crate::ops::NotModelledCause::ModelBoundary,
                    vessel: vessel.id,
                    what: format!(
                        "the partial-freezing model boundary at {:.1} °C: below this point salt crystallisation and a solute-specific eutectic phase diagram are required, so the bench will not extrapolate a colligative relation whose solvent activity it can no longer place",
                        Kelvin(crate::states::BRINE_MODEL_MIN_K).to_celsius()
                    ),
                });
                return Ok(events);
            }
            // Energy that would have to leave to get this cold, spent on
            // freezing instead.
            let excess_j = vessel.energy_between(now, t.freezing_k);
            let requested_freezing = (excess_j / crate::states::WATER_H_FUS).min(liquid_moles);
            // Keep enough liquid water to stay inside the explicit brine
            // boundary. Solutes remain in the liquid compartment, so their
            // particle amount is current molality times current solvent kg.
            let liquid_kg = liquid_moles * 0.018_015;
            let particle_moles = solute_molality * liquid_kg;
            // Two caps, and which one bites decides which sentence the
            // refusal carries. The eutectic one is about the PHASE DIAGRAM
            // below 252 K; the activity one is about the solvent model
            // running out of fitted range. Telling a freeze-concentrated
            // syrup about salt crystallisation would be the wrong reason
            // confidently given.
            let eutectic_cap = t
                .solvent
                .particle_molality_freezing_at(crate::states::BRINE_MODEL_MIN_K);
            let activity_cap = t.solvent.ceiling_molality;
            let ceiling = eutectic_cap.min(activity_cap);
            let boundary_reason = if activity_cap < eutectic_cap {
                t.solvent.out_of_range_reason(activity_cap)
            } else {
                format!(
                    "the partial-freezing model boundary at {:.1} °C: further cooling needs salt crystallisation and a solute-specific eutectic phase diagram",
                    Kelvin(crate::states::BRINE_MODEL_MIN_K).to_celsius()
                )
            };
            let minimum_liquid_moles = if particle_moles > 0.0 {
                particle_moles / ceiling / 0.018_015
            } else {
                0.0
            };
            let maximum_freezing = (liquid_moles - minimum_liquid_moles).max(0.0);
            let over_the_cap = requested_freezing > maximum_freezing + 1e-12;
            let allowed = requested_freezing.min(maximum_freezing);

            // The liquidus this transfer is sized against is the one the
            // transfer MOVES, and since 2026-09-11 it moves further than it
            // used to.
            //
            // `requested_freezing` is `excess_j / ΔH_fus`: the ice the
            // cooling would make if the plateau stood still. Under ΔT =
            // K_f·m it very nearly did — the liquidus fell 1.86 K per molal,
            // linearly, and freezing the requested amount left the vessel
            // close enough to the new plateau that the coupled loop closed
            // the rest in a pass or two. With the solvent's own activity in
            // it the liquidus falls faster and faster: concentrating a brine
            // raises its osmotic coefficient as well as its molality, and
            // for a 2:1 chloride the two together move the liquidus several
            // kelvin in one transfer. Freeze the requested amount there and
            // the latent heat leaves the vessel WARMER than the plateau it
            // has just created, so the next pass melts it back, and the one
            // after re-freezes it: `th-005` ("why does calcium chloride help
            // melt road ice?") rang between −11.6 and −14.4 °C for all 32
            // passes and came out a solver failure.
            //
            // So solve the pass instead of stepping it. Both sides of
            // coexistence are closed-form in the amount frozen — the
            // temperature rises with it as latent heat is released, the
            // liquidus falls with it as the brine concentrates — so their
            // difference is monotone and one bisection lands on the crossing
            // exactly. Where the liquidus really does stand still (pure
            // water, or anything dilute) the crossing IS `requested_freezing`
            // and nothing changes.
            let freezing = self_consistent_freezing(
                allowed,
                excess_j,
                cp,
                t.freezing_k,
                liquid_moles,
                particle_moles,
                activity,
                pressure_kpa,
            );
            // Asking for more ice than the cap allows is not the same as
            // HITTING the cap, and it stopped being the same when the pass
            // started solving for coexistence. A brine cooled just past its
            // cap's worth of latent heat now meets its own falling liquidus
            // well before the cap, and freezes there; announcing a boundary
            // it never reached — and settling it at that boundary's
            // temperature, which is colder than where it actually is — would
            // be a refusal invented out of arithmetic. So the boundary is
            // reached only when the solve itself ran into the cap.
            let reached_boundary = over_the_cap && freezing >= maximum_freezing - 1e-12;

            if freezing <= crate::OBSERVABLE_MOLES {
                if reached_boundary {
                    events.push(Event::NotYetModeled {
                        cause: crate::ops::NotModelledCause::ModelBoundary,
                        vessel: vessel.id,
                        what: boundary_reason.clone(),
                    });
                }
                return Ok(events);
            }

            for p in vessel.contents.iter_mut() {
                if p.species == solvent && p.phase == Phase::Liquid {
                    p.moles = Moles((p.moles.0 - freezing).max(0.0));
                }
            }
            vessel.contents.retain(|p| p.moles.0 > 1e-12);
            vessel.deposit(solvent.clone(), Moles(freezing), Phase::Solid);

            let settled = if reached_boundary {
                // The liquidus of the brine the cap stopped at, not the
                // declared temperature itself: the two are the same number
                // when the eutectic cap is what bit (it is defined by
                // inverting this relation at 252 K), and they are not when
                // the solvent model's own range ran out first, which is a
                // warmer place to stop.
                Kelvin(
                    crate::states::transitions_with(activity, ceiling, pressure_kpa)
                        .0
                        .freezing_k,
                )
            } else if freezing < liquid_moles - 1e-12 {
                // Coexistence defines the temperature, and the bisection
                // above chose `freezing` so that this is also where the
                // energy balance puts it. Under the dilute law this was
                // `t.freezing_k`, the plateau the pass started from; it is
                // now the plateau the pass arrives at, which is the same
                // number whenever the liquidus did not move.
                Kelvin(liquidus_after_freezing(
                    freezing,
                    liquid_moles,
                    particle_moles,
                    activity,
                    pressure_kpa,
                ))
            } else {
                // All of it froze; what is left over chills the ICE, at
                // ice's own heat capacity. The deposit above has already
                // happened, so re-reading the vessel is re-reading the
                // phase: 37.7 J/(mol·K) rather than liquid water's 75.3,
                // which is the difference between −39 °C and −78 °C for a
                // beaker cooled with 60 kJ.
                let leftover = excess_j - freezing * crate::states::WATER_H_FUS;
                // Absolute zero is still the floor. `cool` clamps there and
                // says how much it could not remove; re-deriving the deficit
                // from the clamped temperature can ask for more than the
                // ice has, and a negative kelvin is not an answer.
                Kelvin(settle_from(vessel, t.freezing_k, -leftover, cp).max(0.0))
            };
            vessel.temperature = settled;

            events.push(Event::state_changed(
                vessel.id,
                solvent.clone(),
                Phase::Liquid,
                Phase::Solid,
                Kelvin(t.freezing_k),
                -t.freezing_depression(),
                Moles(freezing),
            ));
            // Ice has no pH. Withdraw the aqueous answer rather than
            // leave a stale one beside a frozen vessel.
            vessel.solution = None;
            if reached_boundary {
                events.push(Event::NotYetModeled {
                    cause: crate::ops::NotModelledCause::ModelBoundary,
                    vessel: vessel.id,
                    what: format!(
                        "pure ice was removed and the residual brine retained, but further cooling meets {boundary_reason}"
                    ),
                });
            }
        } else if frozen_water && now > t.freezing_k {
            // Melting, with the same plateau in reverse — and the same
            // solve, for the same reason and with the same asymmetry if it
            // is left out.
            //
            // Melting DILUTES: the liquidus rises as the ice returns, while
            // the vessel cools as latent heat is absorbed. Sized against the
            // plateau it started from, a melt overshoots past the plateau it
            // creates, and the next pass freezes the water back — which is
            // the freezing branch's failure wearing the other sign. Nothing
            // in the corpus rang this way, because the coupled loop reaches
            // coexistence from the freezing side; leaving one direction
            // solved and the other stepped would be a bug waiting for the
            // first vessel that arrives from above.
            let available_j = vessel.energy_between(t.freezing_k, now);
            let liquid_kg = liquid_moles * 0.018_015;
            let particle_moles = solute_molality * liquid_kg;
            let melting = self_consistent_melting(
                (available_j / crate::states::WATER_H_FUS).min(frozen_moles),
                available_j,
                cp,
                t.freezing_k,
                liquid_moles,
                particle_moles,
                activity,
                pressure_kpa,
            );

            if melting <= crate::OBSERVABLE_MOLES {
                return Ok(events);
            }
            for p in vessel.contents.iter_mut() {
                if p.species == solvent && p.phase == Phase::Solid {
                    p.moles = Moles((p.moles.0 - melting).max(0.0));
                }
            }
            vessel.contents.retain(|p| p.moles.0 > 1e-12);
            vessel.deposit(solvent.clone(), Moles(melting), Phase::Liquid);

            let settled = if melting < frozen_moles - 1e-12 {
                // Coexistence again, and again the solve above chose
                // `melting` so that the energy balance agrees with it.
                Kelvin(liquidus_after_freezing(
                    -melting,
                    liquid_moles,
                    particle_moles,
                    activity,
                    pressure_kpa,
                ))
            } else {
                // The same correction as the freezing branch: what is left
                // once the last of the ice has gone warms LIQUID water.
                Kelvin(settle_from(
                    vessel,
                    t.freezing_k,
                    available_j - melting * crate::states::WATER_H_FUS,
                    cp,
                ))
            };
            vessel.temperature = settled;

            events.push(Event::state_changed(
                vessel.id,
                solvent.clone(),
                Phase::Solid,
                Phase::Liquid,
                Kelvin(t.freezing_k),
                -t.freezing_depression(),
                Moles(melting),
            ));
            // The solvent mass changed; molalities and activities describe
            // the old brine until the phase-coupled solver re-runs chemistry.
            vessel.solution = None;
        } else if liquid_water && now >= t.boiling_k {
            // KID-6: the plateau at the top of the heating curve.
            //
            // Freezing and melting above have paid latent heat since they
            // were written; boiling announced the transition, left the water
            // liquid, and let the temperature run wherever the energy put
            // it. Heating juice on paper reached **670 °C with liquid water
            // still in the ledger** — a state the lv3 register named
            // honestly and the lv1 register reported as "the water is
            // boiling — look at the steam!" beside a mass that had not
            // moved. Pure water escaped this because it routes to the CEA
            // minimiser above 250 °C and gets vaporised there; anything with
            // a solute in it stayed on the aqueous path and simply cooked.
            //
            // Same arithmetic as the melting branch, in the other direction:
            // the energy above the boiling point buys vapour, and the
            // temperature holds at the boiling point until it has bought all
            // of it.
            let available_j = vessel.energy_between(t.boiling_k, now);
            let boiling = (available_j / crate::states::WATER_H_VAP).min(liquid_moles);
            let latent_total = liquid_moles * crate::states::WATER_H_VAP;

            if boiling <= crate::OBSERVABLE_MOLES {
                return Ok(events);
            }
            for p in vessel.contents.iter_mut() {
                if p.species == solvent && p.phase == Phase::Liquid {
                    p.moles = Moles((p.moles.0 - boiling).max(0.0));
                }
            }
            vessel.contents.retain(|p| p.moles.0 > 1e-12);
            // Sealed, the steam is headspace and the pressure says so; open,
            // it leaves the room and the balance notices. Either way the
            // matter is accounted for rather than left behind as a liquid
            // that is somehow above its boiling point.
            if vessel.retain_gas(solvent.clone(), Moles(boiling)) {
                events.push(Event::GasContained {
                    vessel: vessel.id,
                    species: solvent.clone(),
                    moles: Moles(boiling),
                });
            } else {
                events.push(Event::GasEvolved {
                    vessel: vessel.id,
                    species: solvent.clone(),
                    moles: Moles(boiling),
                });
            }

            // What is left once the last of the water has gone is spread
            // over whatever the vessel still holds — the steam, if the
            // flask is sealed, or the solute left behind. A vessel boiled
            // dry and open holds nothing, and `settle_from` then falls back
            // to the pre-transition figure: that under-reports the final
            // temperature, and never the plateau itself, which is the
            // observation the curve is for.
            let settled = if available_j < latent_total {
                Kelvin(t.boiling_k)
            } else {
                Kelvin(settle_from(
                    vessel,
                    t.boiling_k,
                    available_j - latent_total,
                    cp,
                ))
            };
            vessel.temperature = settled;

            if boiling_route != crate::states::BoilingRoute::NormalBoilingPoint {
                events.push(Event::BoilingPointRouted {
                    vessel: vessel.id,
                    species: solvent.clone(),
                    pressure_kpa,
                    boiling: Kelvin(t.boiling_k),
                    shifted_by: t.boiling_pressure_shift(),
                    route: boiling_route,
                    model: crate::states::solvent_row()
                        .and_then(|row| row.saturation_model())
                        .unwrap_or("no cleared correlation")
                        .to_owned(),
                });
            }
            events.push(Event::state_changed(
                vessel.id,
                solvent.clone(),
                Phase::Liquid,
                Phase::Gas,
                Kelvin(t.boiling_k),
                t.boiling_elevation(),
                Moles(boiling),
            ));
            // The solvent mass changed, so every molality and activity the
            // aqueous engine solved for describes water that has left.
            vessel.solution = None;

            // Boiled DRY, not merely boiling. Said here and not in the
            // honesty pass because here there is evidence: this branch
            // just removed the water, so "no water, and something is
            // still filed as dissolved" cannot be confused with a vessel
            // that never had any. `evaporate` says the same sentence for
            // the same reason, and they share it (`stranded_solutes`)
            // rather than each composing their own.
            let dry = !vessel
                .contents
                .iter()
                .any(|p| p.species == solvent && p.phase == Phase::Liquid);
            if dry {
                let stranded: Vec<&str> = vessel
                    .contents
                    .iter()
                    .filter(|p| p.phase == Phase::Aqueous)
                    .filter_map(|p| species::lookup(&p.species).map(|d| d.name))
                    .collect();
                if !stranded.is_empty() {
                    events.push(Event::NotYetModeled {
                        cause: crate::ops::NotModelledCause::NoSolver,
                        vessel: vessel.id,
                        what: stranded_solutes(&stranded),
                    });
                }
            }
        }

        Ok(events)
    }
}

/// Couples an aqueous/speciation solver to the solvent phase pass until the
/// liquid composition and ice fraction describe the same state.
///
/// Freezing removes pure solvent while leaving solutes in the liquid phase.
/// That invalidates molality and therefore the freezing point, so a one-pass
/// stack cannot be self-consistent. This bounded loop is intentionally narrow:
/// it revisits only chemistry and solvent phase state, not every solver in the
/// application stack.
pub fn equilibrate_phase_coupled(
    chemistry: &mut dyn Equilibrator,
    vessel: &mut Vessel,
) -> Result<Vec<Event>, SolveError> {
    const MAX_PASSES: usize = 32;
    let mut events = Vec::new();
    let mut states = StateEquilibrator;

    // A phase transition of independent pure water is a solvent-state
    // question, not an aqueous-speciation question. Probe that transition on
    // a clone before asking chemistry. This keeps ordinary room-temperature
    // water eligible for characterization, while water that is freezing,
    // melting, or boiling never reaches a solver whose solution model is
    // undefined at the phase boundary. Mixtures remain on the coupled path:
    // their activities move the boundary and cannot be replaced by pure-water
    // physics.
    if independent_water_phase_inventory(vessel) {
        let mut phase_trial = vessel.clone();
        phase_trial.solution = None;
        phase_trial.resolved.invalidate();
        phase_trial.free_proton = 0.0;
        phase_trial.free_hydroxide = 0.0;
        let phase_events = states.equilibrate(&mut phase_trial)?;
        if phase_events.iter().any(|event| {
            matches!(
                event,
                Event::StateChanged { species, .. } if species.0 == SOLVENT
            )
        }) {
            *vessel = phase_trial;
            return Ok(phase_events);
        }
    }

    for pass in 0..MAX_PASSES {
        if chemistry.applies(vessel) {
            match chemistry.equilibrate(vessel) {
                Ok(mut more) => events.append(&mut more),
                Err(error) => {
                    events.push(Event::SolverFailed {
                        vessel: vessel.id,
                        solver: chemistry.name().to_string(),
                        detail: error.to_string(),
                    });
                    // Pure solvent phase physics does not require an aqueous
                    // engine or a pre-warmed speciation result. A failed
                    // chemistry calculation must not leave pure liquid water
                    // below freezing solely because that engine is absent.
                    // Mixtures and unresolved material are deliberately NOT
                    // eligible: their unknown activities can move the phase
                    // boundary, so a pure-water answer would be fabricated.
                    if independent_water_phase_inventory(vessel) {
                        vessel.solution = None;
                        vessel.resolved.invalidate();
                        vessel.free_proton = 0.0;
                        vessel.free_hydroxide = 0.0;
                        events.extend(states.equilibrate(vessel)?);
                    }
                    return Ok(events);
                }
            }
        }

        let has_liquid_water = vessel
            .contents
            .iter()
            .any(|portion| portion.species.0 == SOLVENT && portion.phase == Phase::Liquid);
        let has_ice = vessel
            .contents
            .iter()
            .any(|portion| portion.species.0 == SOLVENT && portion.phase == Phase::Solid);
        if has_liquid_water && has_ice {
            // The vessel's OWN liquidus, activity and all — the same call
            // `StateEquilibrator` makes. Recomputing it from the molality
            // alone would compare the coupled solve's answer against a
            // different model's and call the difference numerical residue.
            let liquidus = vessel_transitions(vessel).0.freezing_k;
            if (vessel.temperature.0 - liquidus).abs() <= PHASE_COUPLED_TEMPERATURE_TOLERANCE_K {
                // The chemistry/enthalpy fixed point is only resolvable to
                // the tolerance above.  Do not leak that numerical residue
                // as physically impossible supercooled liquid beside ice:
                // coexistence defines the final temperature exactly.
                vessel.temperature = Kelvin(liquidus);
                // Earlier passes report the provisional liquidus that
                // triggered their water transfer.  Once the coupled solve
                // settles, make the last phase event describe the final
                // coexistence boundary rather than leaking an obsolete
                // intermediate value to renderers and invariant checks.
                if let Some(Event::StateChanged { at, shifted_by, .. }) =
                    events.iter_mut().rev().find(|event| {
                        matches!(
                            event,
                            Event::StateChanged {
                                species,
                                from: Phase::Liquid,
                                to: Phase::Solid,
                                ..
                            } | Event::StateChanged {
                                species,
                                from: Phase::Solid,
                                to: Phase::Liquid,
                                ..
                            } if species.0 == SOLVENT
                        )
                    })
                {
                    *at = Kelvin(liquidus);
                    *shifted_by = liquidus - crate::states::WATER_FREEZING_K;
                }
                return Ok(events);
            }
        }

        let liquid_before: f64 = vessel
            .contents
            .iter()
            .filter(|portion| portion.species.0 == SOLVENT && portion.phase == Phase::Liquid)
            .map(|portion| portion.moles.0)
            .sum();
        let mut phase_events = states.equilibrate(vessel)?;
        let liquid_after: f64 = vessel
            .contents
            .iter()
            .filter(|portion| portion.species.0 == SOLVENT && portion.phase == Phase::Liquid)
            .map(|portion| portion.moles.0)
            .sum();
        let last_transfer_moles = (liquid_after - liquid_before).abs();
        let transferred_water = phase_events.iter().any(|event| {
            matches!(
                event,
                Event::StateChanged {
                    species,
                    from: Phase::Liquid,
                    to: Phase::Solid,
                    ..
                } | Event::StateChanged {
                    species,
                    from: Phase::Solid,
                    to: Phase::Liquid,
                    ..
                } if species.0 == SOLVENT
            )
        });
        events.append(&mut phase_events);
        if !transferred_water {
            return Ok(events);
        }
        if pass + 1 == MAX_PASSES {
            events.push(Event::SolverFailed {
                vessel: vessel.id,
                solver: "phase-coupled".to_string(),
                detail: format!(
                    "aqueous/ice state did not settle within {MAX_PASSES} bounded passes (last water transfer {last_transfer_moles:.3e} mol at {:.6} K)",
                    vessel.temperature.0
                ),
            });
        }
    }
    Ok(events)
}

fn independent_water_phase_inventory(vessel: &Vessel) -> bool {
    vessel.solute_charge == 0.0
        && vessel.unresolved_materials.is_empty()
        && vessel.material_objects.is_empty()
        && vessel.surfaces.is_empty()
        && vessel.exchanges.is_empty()
        && vessel.adsorbed.is_empty()
        && vessel.solid_solutions.is_empty()
        && vessel
            .contents
            .iter()
            .any(|p| p.species.0 == SOLVENT && p.moles.0 > 0.0)
        && vessel.contents.iter().all(|p| {
            p.moles.0.is_finite()
                && p.moles.0 >= 0.0
                && (p.moles.0 == 0.0 || p.species.0 == SOLVENT)
        })
}

/// An application-stack adapter for one chemistry solver coupled to solvent
/// freezing/melting.
pub struct PhaseEquilibrator {
    chemistry: Box<dyn Equilibrator>,
}

impl PhaseEquilibrator {
    pub fn wrapping(chemistry: Box<dyn Equilibrator>) -> Self {
        Self { chemistry }
    }
}

impl Equilibrator for PhaseEquilibrator {
    fn name(&self) -> &'static str {
        "phase-coupled"
    }

    fn chemistry_applies(&self, vessel: &Vessel) -> bool {
        self.chemistry.chemistry_applies(vessel)
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        equilibrate_phase_coupled(self.chemistry.as_mut(), vessel)
    }

    fn mix(
        &mut self,
        vessel: &mut Vessel,
        soln_a: &Vessel,
        frac_a: f64,
        soln_b: &Vessel,
        frac_b: f64,
    ) -> Option<Result<Vec<Event>, SolveError>> {
        self.chemistry.mix(vessel, soln_a, frac_a, soln_b, frac_b)
    }
}

fn curated_solid_product(species: &SpeciesId) -> bool {
    crate::curated::REACTIONS.iter().any(|r| {
        r.products
            .iter()
            .any(|(key, _, phase)| *key == species.0 && *phase == Phase::Solid)
    })
}

/// The honesty pass, last in every stack: any state no chemistry solver has
/// characterised is said out loud rather than silently ignored or faked.
///
/// A vessel whose `solution` is set was handled by an aqueous solver, so a
/// solid coexisting with liquid there is a real computed state (a
/// precipitate), not a gap.
/// Pairing requirement (CAP-23): this pass stands aside for solids the
/// non-aqueous rung has a curated verdict for, so any stack that
/// carries it must carry `nonaqueous::NonAqueousEquilibrator` earlier —
/// otherwise a covered pair gets neither the verdict nor the apology.
/// All three production stacks and the engine test stack do.
/// The temperature above which the aqueous model is not asked at all.
///
/// The shipped USGS databases express their equilibrium constants as
/// analytic functions of temperature whose fitted ranges end, at the
/// most generous, around 300 °C (PHREEQC v3 manual, description of the
/// -analytic ranges; phreeqc.dat and wateq4f.dat are mostly fitted to
/// 100 °C and extended by those expressions). Invoking the engine
/// beyond that produced raw convergence errors on superheated water
/// (curiosity th-022) — an absence of a model surfacing as a crash.
/// Above this ceiling the aqueous engine stands aside and the honesty
/// pass names the boundary instead.
pub const AQUEOUS_MODEL_CEILING_K: f64 = 573.15;

/// What the SOLVENT'S OWN STATE says about whether this vessel has an
/// aqueous solution to characterise at all.
///
/// PLAN P3s, the last of the four: *"a frozen or boiling vessel is a state
/// the aqueous solver does not model, and must say so rather than keep
/// answering."* The first three items of that cluster gave the bench a
/// state model; this one makes the state model's verdict reach the
/// readouts. Without it the withdrawal is silent, and silence is what the
/// −7.95 °C bug was made of: `StateEquilibrator` clears `vessel.solution`
/// on every transition, `partition` counts only liquid water toward `kgw`,
/// and between them a frozen beaker simply stops having a pH — with no
/// sentence anywhere saying that ice is why.
///
/// Boiling is the case that survived all of that, and it needs the
/// argument spelled out because a boiling solution plainly *has* a pH.
/// What it does not have is a **settled** one. Solvent is leaving the
/// vessel while the reading is taken, so every molality the engine solved
/// for is the molality of a solution that is concentrating as you look at
/// it; PHREEQC was handed a composition that had already changed by the
/// time it answered. The honest report is the transition — the plateau,
/// the elevation, the steam — and not a pH quoted to two decimals off a
/// composition with a stopwatch running on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolventState {
    /// Liquid water below its boiling point: the aqueous engines' home
    /// ground, and the only state that reports a solution.
    Settled,
    /// The solvent is in a state that has no solution to report — ice, or
    /// on the boil — but nothing is dissolved in it anyway, so there is
    /// nothing being withheld and nothing to apologise for.
    ///
    /// This variant is the difference between an honesty pass and a
    /// nagging one. A kettle of pure water at 100 °C is in exactly the
    /// state [`SolventState::Boiling`] describes, and telling its user
    /// that no settled pH is reported for the nothing dissolved in it
    /// would be the "noise dressed as honesty" this module keeps warning
    /// itself about. The `StateChanged` event the boil already emits is
    /// the whole story there.
    Pure,
    /// Every drop of the solvent this vessel holds is ice.
    Frozen,
    /// Liquid solvent standing at or above its own (colligatively shifted,
    /// pressure-shifted) boiling point.
    Boiling,
    /// No solvent here.
    ///
    /// Deliberately **not** "the solvent boiled away and left its ions
    /// stranded", though that state exists and is worth naming. A vessel
    /// holding aqueous-filed matter and no water looks identical to one
    /// whose solvent left, and is not always the same thing: a kneaded
    /// dough holds its water in the flour matrix and pours none into the
    /// beaker, so a fermentation product filed aqueous beside it would
    /// trip that test with nothing wrong. Naming it safely needs the one
    /// fact a snapshot cannot carry — that the water *left* — so the
    /// claim is made by the two passes that removed it (`evaporate`, and
    /// the boiling branch of [`StateEquilibrator`]) and the sentence they
    /// share is [`stranded_solutes`].
    Absent,
}

impl SolventState {
    /// The sentence the bench owes a reader, and the cause to file it
    /// under. `None` where there is nothing to apologise for.
    ///
    pub fn boundary(self) -> Option<(String, crate::ops::NotModelledCause)> {
        match self {
            Self::Settled | Self::Pure | Self::Absent => None,
            Self::Frozen => Some((
                "the water in this vessel is ice, and ice is not a solution: pH, ionic strength and speciation all describe particles dissolved in a liquid, so none of them is reported while the solvent is frozen".to_string(),
                crate::ops::NotModelledCause::NoSolution,
            )),
            Self::Boiling => Some((
                "the water is at the boil and leaving as steam, so what is dissolved in the rest is concentrating while you look at it: this bench reports the transition rather than a settled pH for a composition that is still changing".to_string(),
                crate::ops::NotModelledCause::ModelBoundary,
            )),
        }
    }
}

/// Read the solvent's state off a vessel.
///
/// The boiling test uses the vessel's OWN transition — [`vessel_transitions`],
/// the same call `StateEquilibrator` makes — rather than 373.15 K, so a
/// pressure cooker and a salted pan are judged on their own plateau and a
/// flask under vacuum is called boiling at 60 °C when it really is.
pub fn solvent_state(vessel: &Vessel) -> SolventState {
    let solvent = SpeciesId::new(SOLVENT);
    let mut liquid = 0.0;
    let mut ice = 0.0;
    let mut dissolved = false;
    for p in &vessel.contents {
        if p.species == solvent {
            match p.phase {
                Phase::Liquid | Phase::Aqueous => liquid += p.moles.0,
                Phase::Solid => ice += p.moles.0,
                Phase::Gas => {}
            }
        } else if p.phase == Phase::Aqueous {
            dissolved = true;
        }
    }
    // Is there anything here whose pH, ionic strength or speciation a
    // reader could be waiting for? A standing `solution` counts even when
    // no portion is filed aqueous, because that is the answer being
    // withheld.
    let characterisable = dissolved || vessel.solution.is_some();
    if liquid > crate::OBSERVABLE_MOLES {
        // At or above the plateau. The settle in `StateEquilibrator` puts
        // a boiling vessel EXACTLY on `boiling_k`, so this has to admit
        // equality or the state it just computed would not be readable.
        let (t, _) = vessel_transitions(vessel);
        // A SEALED vessel at its own raised boiling point is not this
        // case, and the distinction is the whole argument rather than a
        // detail. What disqualifies an open beaker on the boil is that it
        // is losing mass to the room while the reading is taken, so the
        // aqueous engine — which solves a closed system — was handed a
        // composition that had already changed. Under a lid the steam
        // stays, the pressure it raises lifts the boiling point until the
        // two agree, and liquid and vapour at coexistence in a closed
        // vessel is an equilibrium the engine is entitled to solve. It
        // keeps its pH.
        if vessel.temperature.0 >= t.boiling_k - 1e-9 && !vessel.owns_headspace_gas() {
            return if characterisable {
                SolventState::Boiling
            } else {
                SolventState::Pure
            };
        }
        return SolventState::Settled;
    }
    if ice > crate::OBSERVABLE_MOLES {
        return if characterisable {
            SolventState::Frozen
        } else {
            SolventState::Pure
        };
    }
    SolventState::Absent
}

/// The sentence for dissolved matter whose solvent has left, by name.
///
/// Shared by the two passes entitled to say it — `evaporate`, and the
/// boiling branch of [`StateEquilibrator`] — so that a beaker taken to
/// dryness by a burner gets the same words as one dried on a hotplate.
/// Which verb reached the state is not the reader's problem; that it is a
/// state no beaker can be in is.
pub fn stranded_solutes(names: &[&str]) -> String {
    format!(
        "the last of the water is gone and {} are still shown as dissolved, \
         which is not a state a beaker can be in. What they crystallise into \
         is not decidable from the ions alone, so the bench will not guess at \
         the solids",
        names.join(", ")
    )
}

/// A solid portion of a substance that is a liquid at standard conditions,
/// standing below its curated melting point: frozen, not unreacted.
fn frozen_liquid(vessel: &Vessel, species: &SpeciesId) -> bool {
    species::lookup(species).is_some_and(|data| {
        data.standard_phase == Phase::Liquid
            && data
                .transitions
                .and_then(|t| t.melting_reading())
                .is_some_and(|(melting_k, _)| vessel.temperature.0 < melting_k)
    })
}

pub struct HonestyEquilibrator;

impl Equilibrator for HonestyEquilibrator {
    fn name(&self) -> &'static str {
        "honesty"
    }

    fn route_kind(&self) -> SolverRouteKind {
        SolverRouteKind::Qualitative
    }

    fn chemistry_applies(&self, _vessel: &Vessel) -> bool {
        false
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        let mut events = Vec::new();
        // The solvent's own state is asked FIRST, before the "a solution
        // was characterised, so there is no gap" early return below.
        //
        // That order is the whole point. `StateEquilibrator` withdraws
        // `vessel.solution` the moment it freezes or boils anything, but
        // the phase-coupled loop then runs the chemistry again over
        // whatever liquid is left and hands a fresh pH straight back — so
        // a beaker on the boil arrives here with an answer standing, and
        // an early return would let it stand. The state pass has already
        // decided this vessel is not a settled solution; this is where
        // that decision reaches the reader and the meter.
        let state = solvent_state(vessel);
        if let Some((what, cause)) = state.boundary() {
            events.push(Event::NotYetModeled {
                cause,
                vessel: vessel.id,
                what,
            });
            // And withdraw the reading itself, so `PhMeter::applies` is
            // false and the conductivity meter and the pH badge go with
            // it. Reporting the boundary in prose while the instrument
            // still answers would be the honesty pass contradicting the
            // bench in the same breath.
            vessel.solution = None;
            return Ok(events);
        }
        if vessel
            .solution
            .as_ref()
            .is_some_and(|solution| solution.scope == crate::vessel::SolutionScope::Complete)
        {
            return Ok(events);
        }
        let has_liquid = vessel
            .contents
            .iter()
            .any(|p| matches!(p.phase, Phase::Liquid | Phase::Aqueous));
        // Water above the aqueous model's temperature ceiling: the engine
        // stood aside on purpose, and the reason has to be spoken —
        // a silent stand-aside reads as "nothing dissolved here".
        if vessel.temperature.0 > AQUEOUS_MODEL_CEILING_K
            && vessel
                .contents
                .iter()
                .any(|p| p.species == SpeciesId::new(SOLVENT) && p.phase != Phase::Solid)
        {
            events.push(Event::NotYetModeled { cause: crate::ops::NotModelledCause::ModelBoundary,
                vessel: vessel.id,
                what: format!(
                    "the aqueous model's temperature ceiling at {:.0} °C: the shipped thermodynamic databases' temperature expressions end there, so this solution is reported uncharacterised rather than extrapolated",
                    Kelvin(AQUEOUS_MODEL_CEILING_K).to_celsius()
                ),
            });
            return Ok(events);
        }
        // Water in the minority beside an organic solvent: the aqueous
        // engine stood aside (CAP-23 rung 3) and the reason is the
        // dielectric environment, which is worth a sentence of its own —
        // but only when something is actually dissolved there. A clean
        // water–ethanol distillate has no ions to speciate, and an
        // apology about ionic speciation over pure solvents is noise
        // dressed as honesty.
        let has_solute = vessel.contents.iter().any(|p| {
            p.species.0 != SOLVENT
                && (p.species.0 == "CH3COOH"
                    || !crate::nonaqueous::KNOWN_SOLVENTS.contains(&p.species.0.as_str()))
                && p.phase != Phase::Gas
        });
        // ...and not when the curated chemistry already answered in this
        // medium: permanganate meeting ethanol reacts by the curated
        // route, whose own water byproduct would otherwise trip this
        // apology right after the answer. The curated product in the
        // vessel is the evidence the medium was handled.
        let curated_answered = vessel
            .contents
            .iter()
            .any(|p| curated_solid_product(&p.species));
        if let Some(x) = crate::nonaqueous::water_fraction_among_solvents(vessel) {
            if has_solute
                && !curated_answered
                && x < crate::nonaqueous::AQUEOUS_WATER_FRACTION_FLOOR
            {
                events.push(Event::NotYetModeled { cause: crate::ops::NotModelledCause::ModelBoundary,
                    vessel: vessel.id,
                    what: format!(
                        "a mixed solvent that is mostly organic (water is {:.0}% of the liquid): the shipped activity models assume water as the solvent, and in this dielectric environment their equilibrium constants do not apply, so ionic speciation here is reported uncharacterised",
                        x * 100.0
                    ),
                });
                return Ok(events);
            }
        }
        for p in &vessel.contents {
            // Frozen solvent is not an unmodelled dissolution — the state
            // pass just explained it, and saying "ice in contact with
            // liquid: no solver models this" would be noise dressed as
            // honesty.
            if p.species == SpeciesId::new(SOLVENT) {
                continue;
            }
            if p.phase == Phase::Solid && has_liquid {
                // A pair the non-aqueous rung has a computed verdict for
                // was already answered; an apology after an answer is
                // noise dressed as honesty.
                if let Some(solvent) = crate::nonaqueous::single_organic_solvent(vessel) {
                    if crate::nonaqueous::verdict_exists(&p.species, solvent) {
                        continue;
                    }
                }
                // A solid that a curated reaction produces is not an
                // unmodelled mystery — the reaction just put it there.
                // The water byproduct of the reaction can break
                // single_organic_solvent, so this check is independent.
                if curated_solid_product(&p.species) {
                    continue;
                }
                // A liquid the cold has frozen is not an unmodelled reagent
                // either: a phase route just put it in that state, on a
                // curated melting point, and the apology would read as
                // "nobody has modelled frozen ethanol" in the same breath
                // as the route that froze it.
                if frozen_liquid(vessel, &p.species) {
                    continue;
                }
                if crate::starch_iodine::covers_solid(vessel, &p.species) {
                    continue;
                }
                // A declared kinetic catalyst is already wired even when
                // this equilibrium rung cannot speciate the salt. The slow
                // clock consumes its catalytic effect and deliberately leaves
                // its inventory unchanged, so an "unmodelled reaction"
                // apology here would contradict the computed result.
                if crate::kinetics::applicable(vessel).iter().any(|reaction| {
                    reaction
                        .catalysts
                        .iter()
                        .any(|catalyst| catalyst.species == p.species.0)
                }) {
                    continue;
                }
                let name = species::lookup(&p.species)
                    .map(|d| d.name)
                    .unwrap_or(p.species.0.as_str());
                // Two different gaps wearing one event. It dissolves and
                // nothing speciates it, or nothing models it at all — and
                // which one it is decides whether a database could ever fix
                // it, so the cause is computed beside the sentence.
                // A reviewed solubility of essentially zero is not a gap,
                // it is the answer. "Does sand dissolve overnight?" was
                // reaching `not yet modelled` — the bench declining to say
                // what the registry already knew — and a learner cannot
                // tell that from "nobody has modelled this yet". Where a
                // solubility has been reviewed and is below what a beaker
                // could show, the bench says so and stops refusing.
                // ...but only where nothing else in the bench is about to
                // consume it. Starch is insoluble in cold water AND is what
                // amylase digests, and the first draft of this branch had
                // the bench print "starch does not react" in the same run
                // as `2 (C6H10O5) + H2O ->[amylase] C12H22O11`. A true
                // sentence about dissolution, said where it reads as a
                // claim about reactivity, is a false one.
                if !crate::curated::consumes(vessel, &p.species)
                    && !crate::kinetics::consumes(vessel, &p.species)
                {
                    if let Some(limit) = species::lookup(&p.species)
                        .and_then(|d| d.aqueous_solubility_at(vessel.temperature.0))
                        .filter(|limit| *limit < 0.01)
                    {
                        events.push(Event::Inert {
                        vessel: vessel.id,
                        species: p.species.clone(),
                        why: format!(
                            "{name} does not dissolve in water: its reviewed solubility is {limit:.4} g per 100 mL, which is below anything a beaker would show. It is still all there"
                        ),
                        computed: false,
                        spent: None,
                    });
                        continue;
                    }
                }
                let (what, cause) = if species::lookup(&p.species)
                    .is_some_and(|d| d.dissolves_without_speciation)
                {
                    (
                        format!(
                            "{name} dissolves, but no wired engine speciates it: it contributes nothing to the pH or the ionic strength here, and those numbers are for everything else in the beaker"
                        ),
                        crate::ops::NotModelledCause::NotSpeciated,
                    )
                } else {
                    (
                        format!(
                            "{name} in contact with liquid: no wired solver models this dissolution/reaction"
                        ),
                        crate::ops::NotModelledCause::NoSolver,
                    )
                };
                events.push(Event::NotYetModeled {
                    vessel: vessel.id,
                    what,
                    cause,
                });
            }
        }
        Ok(events)
    }

    /// ARCH-012: native delta — no mutations, only diagnostic events.
    fn equilibrate_delta(
        &mut self,
        vessel: &Vessel,
    ) -> Result<(crate::delta::StateDelta, Vec<Event>), SolveError> {
        let delta = crate::delta::StateDelta::new("honesty");
        let mut events = Vec::new();

        // Same order and same reason as `equilibrate` above: the solvent's
        // state is asked before a standing answer can short-circuit it.
        // The preview cannot withdraw the reading (it holds the vessel by
        // reference), so it says the sentence and leaves the withdrawal to
        // the pass that owns the mutation.
        if let Some((what, cause)) = solvent_state(vessel).boundary() {
            events.push(Event::NotYetModeled {
                cause,
                vessel: vessel.id,
                what,
            });
            return Ok((delta, events));
        }
        if vessel
            .solution
            .as_ref()
            .is_some_and(|solution| solution.scope == crate::vessel::SolutionScope::Complete)
        {
            return Ok((delta, events));
        }

        let has_liquid = vessel
            .contents
            .iter()
            .any(|p| matches!(p.phase, Phase::Liquid | Phase::Aqueous));

        for p in &vessel.contents {
            if p.species == SpeciesId::new(SOLVENT) {
                continue;
            }
            if p.phase == Phase::Solid && has_liquid {
                if let Some(solvent) = crate::nonaqueous::single_organic_solvent(vessel) {
                    if crate::nonaqueous::verdict_exists(&p.species, solvent) {
                        continue;
                    }
                }
                if curated_solid_product(&p.species) {
                    continue;
                }
                // A liquid the cold has frozen is not an unmodelled reagent
                // either: a phase route just put it in that state, on a
                // curated melting point, and the apology would read as
                // "nobody has modelled frozen ethanol" in the same breath
                // as the route that froze it.
                if frozen_liquid(vessel, &p.species) {
                    continue;
                }
                if crate::starch_iodine::covers_solid(vessel, &p.species) {
                    continue;
                }
                if crate::kinetics::applicable(vessel).iter().any(|reaction| {
                    reaction
                        .catalysts
                        .iter()
                        .any(|catalyst| catalyst.species == p.species.0)
                }) {
                    continue;
                }
                let name = species::lookup(&p.species)
                    .map(|d| d.name)
                    .unwrap_or(p.species.0.as_str());
                // Two different gaps wearing one event. It dissolves and
                // nothing speciates it, or nothing models it at all — and
                // which one it is decides whether a database could ever fix
                // it, so the cause is computed beside the sentence.
                // A reviewed solubility of essentially zero is not a gap,
                // it is the answer. "Does sand dissolve overnight?" was
                // reaching `not yet modelled` — the bench declining to say
                // what the registry already knew — and a learner cannot
                // tell that from "nobody has modelled this yet". Where a
                // solubility has been reviewed and is below what a beaker
                // could show, the bench says so and stops refusing.
                // ...but only where nothing else in the bench is about to
                // consume it. Starch is insoluble in cold water AND is what
                // amylase digests, and the first draft of this branch had
                // the bench print "starch does not react" in the same run
                // as `2 (C6H10O5) + H2O ->[amylase] C12H22O11`. A true
                // sentence about dissolution, said where it reads as a
                // claim about reactivity, is a false one.
                if !crate::curated::consumes(vessel, &p.species)
                    && !crate::kinetics::consumes(vessel, &p.species)
                {
                    if let Some(limit) = species::lookup(&p.species)
                        .and_then(|d| d.aqueous_solubility_at(vessel.temperature.0))
                        .filter(|limit| *limit < 0.01)
                    {
                        events.push(Event::Inert {
                        vessel: vessel.id,
                        species: p.species.clone(),
                        why: format!(
                            "{name} does not dissolve in water: its reviewed solubility is {limit:.4} g per 100 mL, which is below anything a beaker would show. It is still all there"
                        ),
                        computed: false,
                        spent: None,
                    });
                        continue;
                    }
                }
                let (what, cause) = if species::lookup(&p.species)
                    .is_some_and(|d| d.dissolves_without_speciation)
                {
                    (
                        format!(
                            "{name} dissolves, but no wired engine speciates it: it contributes nothing to the pH or the ionic strength here, and those numbers are for everything else in the beaker"
                        ),
                        crate::ops::NotModelledCause::NotSpeciated,
                    )
                } else {
                    (
                        format!(
                            "{name} in contact with liquid: no wired solver models this dissolution/reaction"
                        ),
                        crate::ops::NotModelledCause::NoSolver,
                    )
                };
                events.push(Event::NotYetModeled {
                    vessel: vessel.id,
                    what,
                    cause,
                });
            }
        }

        Ok((delta, events))
    }
}

/// The temperature at which every part of an adiabatic mix comes to rest:
/// the `T` where the heat they all absorb on the way to it sums to zero.
///
/// `total(t)` is that sum, each part measured from where it is now, so it is
/// negative below the answer and positive above it and the root is bracketed
/// by the coldest and warmest parts. This replaced a weighted mean of
/// temperatures, which is the same answer only while every heat capacity is
/// a constant. With curves it is not merely less accurate, it is not
/// CONSERVATIVE: pouring a millilitre of room-temperature water into a
/// beaker a kelvin warm changed the bench's total enthalpy, and
/// `conservation::energy_is_conserved` said so to eight figures.
///
/// Bisection rather than anything cleverer because the function is monotone
/// by construction - every heat capacity is positive - and the bracket is
/// exact. It stops at a nanokelvin or a microjoule, both far below what any
/// instrument on this bench reads.
pub fn adiabatic_rest_temperature(lo: f64, hi: f64, total: impl Fn(f64) -> f64) -> Kelvin {
    if hi - lo <= 1e-12 {
        return Kelvin(0.5 * (lo + hi));
    }
    if total(lo) >= 0.0 {
        // Nothing colder to warm: the answer is where the cold end already is.
        return Kelvin(lo);
    }
    if total(hi) <= 0.0 {
        return Kelvin(hi);
    }
    let (mut a, mut b) = (lo, hi);
    for _ in 0..200 {
        let m = 0.5 * (a + b);
        // Run the bracket down to adjacent floats rather than to a chosen
        // tolerance. The ledger is solved numerically now, and a stopping
        // rule of a nanokelvin leaves a residual of Cp nanojoules per mix
        // that accumulates across a script; `conservation::
        // energy_is_conserved` measures exactly that sum.
        if m <= a || m >= b {
            break;
        }
        if total(m) > 0.0 {
            b = m;
        } else {
            a = m;
        }
    }
    Kelvin(0.5 * (a + b))
}

/// Mix incoming matter arriving at `t_in` into `vessel`, adiabatically.
///
/// `incoming(t)` is the heat that matter absorbs going from `t_in` to `t`,
/// J, signed: negative when the incoming matter is the warmer side.
/// Build it from a list of portions with [`portions_enthalpy`].
pub fn adiabatic_mix_into(vessel: &Vessel, t_in: Kelvin, incoming: impl Fn(f64) -> f64) -> Kelvin {
    let held = vessel.temperature.0;
    adiabatic_rest_temperature(held.min(t_in.0), held.max(t_in.0), |t| {
        vessel.energy_between(held, t) + incoming(t)
    })
}

/// The heat a set of incoming portions absorbs going from `t0` to `t1`, J.
///
/// Per phase and along each species' own curve, exactly as the vessel they
/// are poured into charges its own contents - which is the whole point: a
/// mix whose two sides read different tables cannot conserve energy, however
/// small the difference between the tables.
pub fn portions_enthalpy<'a, I>(portions: I, t0: f64, t1: f64) -> f64
where
    I: IntoIterator<Item = (&'a SpeciesId, f64, Phase)>,
{
    portions
        .into_iter()
        .filter_map(|(species, moles, phase)| {
            let data = crate::species::lookup(species)?;
            Some(moles * crate::states::enthalpy_between(data, phase, t0, t1))
        })
        .sum()
}

#[cfg(test)]
mod route_trace_tests {
    use super::*;

    struct TestRoute {
        name: &'static str,
        applies: bool,
        kind: SolverRouteKind,
    }

    /// A solver that gives off one gas, for the step ledger below.
    struct GasRoute(&'static str, f64);

    impl Equilibrator for GasRoute {
        fn name(&self) -> &'static str {
            "gas-route"
        }

        fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
            Ok(vec![Event::GasEvolved {
                vessel: vessel.id,
                species: SpeciesId::new(self.0),
                moles: Moles(self.1),
            }])
        }
    }

    #[test]
    fn the_stack_books_each_solvers_gas_on_the_step_before_the_next_runs() {
        let mut vessel = Vessel::new(crate::vessel::VesselId(0), "v1");
        vessel.step_start = Some(crate::vessel::StepStart::capture(&vessel));
        let mut stack = SolverStack::new(vec![
            Box::new(GasRoute("CO2", 0.01)),
            Box::new(GasRoute("CO2", 0.02)),
            Box::new(GasRoute("O2", 0.005)),
        ]);
        let events = stack.equilibrate(&mut vessel).expect("equilibrates");
        assert_eq!(events.len(), 3);
        let start = vessel
            .step_start
            .as_ref()
            .expect("the stack does not clear the snapshot");
        assert_eq!(start.gas_out.len(), 2);
        let co2 = start
            .gas_out
            .iter()
            .find(|(s, _)| s.0 == "CO2")
            .unwrap()
            .1
             .0;
        assert!((co2 - 0.03).abs() < 1e-12, "{co2}");
        // Without a snapshot nothing is booked and nothing breaks.
        let mut bare = Vessel::new(crate::vessel::VesselId(1), "v2");
        stack.equilibrate(&mut bare).expect("equilibrates");
        assert!(bare.step_start.is_none());
    }

    /// A solver whose gas stays in a sealed headspace.
    struct ContainedRoute;

    impl Equilibrator for ContainedRoute {
        fn name(&self) -> &'static str {
            "contained-route"
        }

        fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
            let species = SpeciesId::new("CO2");
            let kept = vessel.retain_gas(species.clone(), Moles(0.01));
            assert!(kept, "a sealed headspace keeps its gas");
            Ok(vec![Event::GasContained {
                vessel: vessel.id,
                species,
                moles: Moles(0.01),
            }])
        }
    }

    #[test]
    fn gas_kept_in_a_headspace_is_not_booked_as_gas_that_left() {
        // Booking it would count it twice: once as the `Phase::Gas` portion
        // the headspace holds, once as an outward transfer.
        let mut vessel = Vessel::new(crate::vessel::VesselId(0), "sealed");
        vessel.headspace = crate::vessel::Headspace::Sealed {
            volume: crate::units::Liters(1.0),
        };
        vessel.step_start = Some(crate::vessel::StepStart::capture(&vessel));
        let mut stack = SolverStack::new(vec![Box::new(ContainedRoute)]);
        stack.equilibrate(&mut vessel).expect("equilibrates");
        assert!(vessel.step_start.as_ref().unwrap().gas_out.is_empty());
        assert!((vessel.moles_of(&SpeciesId::new("CO2")).0 - 0.01).abs() < 1e-12);
    }

    impl Equilibrator for TestRoute {
        fn name(&self) -> &'static str {
            self.name
        }

        fn route_kind(&self) -> SolverRouteKind {
            self.kind
        }

        fn applies(&self, _vessel: &Vessel) -> bool {
            self.applies
        }

        fn equilibrate(&mut self, _vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
            Ok(Vec::new())
        }
    }

    #[test]
    fn stack_records_typed_routes_without_changing_events() {
        let mut stack = SolverStack::new(vec![
            Box::new(TestRoute {
                name: "computed-test",
                applies: false,
                kind: SolverRouteKind::Computed,
            }),
            Box::new(TestRoute {
                name: "curated-test",
                applies: true,
                kind: SolverRouteKind::Curated,
            }),
        ]);
        let events = stack.equilibrate(&mut Vessel::new(crate::vessel::VesselId(0), "beaker"));
        assert!(events.expect("stack succeeds").is_empty());
        assert_eq!(stack.last_routes.len(), 2);
        assert_eq!(
            stack.last_routes[0].outcome,
            SolverRouteOutcome::NotApplicable
        );
        assert_eq!(stack.last_routes[1].kind, SolverRouteKind::Curated);
        assert_eq!(
            stack.last_routes[1].outcome,
            SolverRouteOutcome::Succeeded { event_count: 0 }
        );
    }

    /// GUI-052: a declining solver says which vessel it looked at and, if
    /// it overrides `capability()`, why it declined. The default adapter
    /// still gives a sentence, so a reader is never left with a bare "no".
    #[test]
    fn declined_routes_name_the_vessel_and_the_reason() {
        let mut stack = SolverStack::new(vec![Box::new(TestRoute {
            name: "computed-test",
            applies: false,
            kind: SolverRouteKind::Computed,
        })]);
        let mut vessel = Vessel::new(crate::vessel::VesselId(3), "beaker");
        stack.equilibrate(&mut vessel).expect("stack succeeds");
        assert_eq!(
            stack.last_routes[0].vessel,
            Some(crate::vessel::VesselId(3))
        );
        let reason = stack.last_routes[0]
            .reason
            .as_deref()
            .expect("a decline carries the solver's own sentence");
        assert!(reason.contains("computed-test"), "{reason}");
    }

    /// A route that answered has nothing to explain, and the wire says so
    /// by omission rather than by a null.
    #[test]
    fn answered_routes_carry_no_reason() {
        let mut stack = SolverStack::new(vec![Box::new(TestRoute {
            name: "curated-test",
            applies: true,
            kind: SolverRouteKind::Curated,
        })]);
        stack
            .equilibrate(&mut Vessel::new(crate::vessel::VesselId(0), "beaker"))
            .expect("stack succeeds");
        assert_eq!(stack.last_routes[0].reason, None);
        let wire = serde_json::to_value(&stack.last_routes[0]).expect("route serialises");
        assert!(wire.get("reason").is_none(), "{wire}");
        assert_eq!(wire["vessel"], serde_json::json!(0));
    }
}
