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
                    });
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

        // Neutral molecular solids with an explicit reviewed room-temperature
        // limit dissolve only up to that finite capacity. This changes phase
        // bookkeeping but makes no pH, ionic-strength, or activity claim.
        for (solute, moles) in finite_aqueous_dissolutions(vessel) {
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
        for (solute, moles) in finite_aqueous_dissolutions(vessel) {
            delta = delta
                .with_moles(solute.clone(), Phase::Solid, -moles.0)
                .with_moles(solute.clone(), Phase::Aqueous, moles.0);
            events.push(Event::Dissolved {
                vessel: vessel.id,
                species: solute,
                moles,
            });
        }

        Ok((delta, events))
    }
}

fn finite_aqueous_dissolutions(vessel: &Vessel) -> Vec<(SpeciesId, Moles)> {
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
    for portion in vessel
        .contents
        .iter()
        .filter(|portion| portion.phase == Phase::Solid)
    {
        if !seen.insert(portion.species.clone()) {
            continue;
        }
        let Some(data) = species::lookup(&portion.species) else {
            continue;
        };
        let Some(limit_g_per_100_ml) = data.aqueous_solubility_g_per_100_ml else {
            continue;
        };
        let solid = vessel
            .contents
            .iter()
            .filter(|p| p.species == portion.species && p.phase == Phase::Solid)
            .map(|p| p.moles.0)
            .sum::<f64>();
        let aqueous = vessel
            .contents
            .iter()
            .filter(|p| p.species == portion.species && p.phase == Phase::Aqueous)
            .map(|p| p.moles.0)
            .sum::<f64>();
        let capacity_moles = limit_g_per_100_ml * water_ml / 100.0 / data.molar_mass;
        let dissolves = solid.min((capacity_moles - aqueous).max(0.0));
        if dissolves > 1e-15 {
            out.push((portion.species.clone(), Moles(dissolves)));
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
/// resolve. This tolerance is only a coupled-solver stop; an initially
/// supercooled single liquid phase still undergoes its physical transfer.
pub const PHASE_COUPLED_TEMPERATURE_TOLERANCE_K: f64 = 0.05;

fn dissolved_particle_molality(vessel: &Vessel) -> f64 {
    vessel.solution.as_ref().map_or(0.0, |info| {
        info.species
            .iter()
            .filter(|species| species.name != "H2O")
            .map(|species| species.molality)
            .sum()
    })
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
        let solute_molality = dissolved_particle_molality(vessel);
        let t = crate::states::transitions(solute_molality);
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

        if liquid_water && now < t.freezing_k {
            if solute_molality > 0.0
                && t.freezing_k <= crate::states::BRINE_MODEL_MIN_K
                && now <= crate::states::BRINE_MODEL_MIN_K
            {
                events.push(Event::NotYetModeled {
                    vessel: vessel.id,
                    what: format!(
                        "the partial-freezing model boundary at {:.1} °C: below this point salt crystallisation and a solute-specific eutectic phase diagram are required, so the bench will not extrapolate the dilute colligative relation",
                        Kelvin(crate::states::BRINE_MODEL_MIN_K).to_celsius()
                    ),
                });
                return Ok(events);
            }
            // Energy that would have to leave to get this cold, spent on
            // freezing instead.
            let excess_j = cp * (t.freezing_k - now);
            let latent_total = liquid_moles * crate::states::WATER_H_FUS;
            let requested_freezing = (excess_j / crate::states::WATER_H_FUS).min(liquid_moles);
            // Keep enough liquid water to stay inside the explicit brine
            // boundary. Solutes remain in the liquid compartment, so their
            // particle amount is current molality times current solvent kg.
            let liquid_kg = liquid_moles * 0.018_015;
            let particle_moles = solute_molality * liquid_kg;
            let minimum_liquid_moles = if particle_moles > 0.0 {
                particle_moles / crate::states::brine_model_max_particle_molality() / 0.018_015
            } else {
                0.0
            };
            let maximum_freezing = (liquid_moles - minimum_liquid_moles).max(0.0);
            let freezing = requested_freezing.min(maximum_freezing);
            let reached_boundary = requested_freezing > maximum_freezing + 1e-12;

            if freezing <= crate::OBSERVABLE_MOLES {
                if reached_boundary {
                    events.push(Event::NotYetModeled {
                        vessel: vessel.id,
                        what: format!(
                            "the partial-freezing model boundary at {:.1} °C: further cooling needs salt crystallisation and a solute-specific eutectic phase diagram",
                            Kelvin(crate::states::BRINE_MODEL_MIN_K).to_celsius()
                        ),
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

            vessel.temperature = if reached_boundary {
                Kelvin(crate::states::BRINE_MODEL_MIN_K)
            } else if excess_j < latent_total {
                Kelvin(t.freezing_k) // still freezing: the plateau
            } else {
                // All of it froze; what is left over chills the ice.
                let leftover = excess_j - latent_total;
                Kelvin(t.freezing_k - leftover / cp)
            };

            events.push(Event::StateChanged {
                vessel: vessel.id,
                species: solvent.clone(),
                from: Phase::Liquid,
                to: Phase::Solid,
                at: Kelvin(t.freezing_k),
                shifted_by: -t.freezing_depression(),
            });
            // Ice has no pH. Withdraw the aqueous answer rather than
            // leave a stale one beside a frozen vessel.
            vessel.solution = None;
            if reached_boundary {
                events.push(Event::NotYetModeled {
                    vessel: vessel.id,
                    what: format!(
                        "the partial-freezing model boundary at {:.1} °C: pure ice was removed and the residual brine retained, but further cooling needs salt crystallisation and a solute-specific eutectic phase diagram",
                        Kelvin(crate::states::BRINE_MODEL_MIN_K).to_celsius()
                    ),
                });
            }
        } else if frozen_water && now > t.freezing_k {
            // Melting, with the same plateau in reverse.
            let available_j = cp * (now - t.freezing_k);
            let melting = (available_j / crate::states::WATER_H_FUS).min(frozen_moles);
            let latent_total = frozen_moles * crate::states::WATER_H_FUS;

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

            vessel.temperature = if available_j < latent_total {
                Kelvin(t.freezing_k)
            } else {
                Kelvin(t.freezing_k + (available_j - latent_total) / cp)
            };

            events.push(Event::StateChanged {
                vessel: vessel.id,
                species: solvent.clone(),
                from: Phase::Solid,
                to: Phase::Liquid,
                at: Kelvin(t.freezing_k),
                shifted_by: -t.freezing_depression(),
            });
            // The solvent mass changed; molalities and activities describe
            // the old brine until the phase-coupled solver re-runs chemistry.
            vessel.solution = None;
        } else if liquid_water && now >= t.boiling_k {
            events.push(Event::StateChanged {
                vessel: vessel.id,
                species: solvent.clone(),
                from: Phase::Liquid,
                to: Phase::Gas,
                at: Kelvin(t.boiling_k),
                shifted_by: t.boiling_elevation(),
            });
            vessel.solution = None;
            events.push(Event::NotYetModeled {
                vessel: vessel.id,
                what: "a boiling vessel: the temperature should hold at the boiling point while water leaves as steam, and that latent-heat plateau is not modelled yet".to_string(),
            });
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
            let liquidus =
                crate::states::transitions(dissolved_particle_molality(vessel)).freezing_k;
            if (vessel.temperature.0 - liquidus).abs() <= PHASE_COUPLED_TEMPERATURE_TOLERANCE_K {
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
        if vessel.solution.is_some() {
            return Ok(events);
        }
        let has_liquid = vessel
            .contents
            .iter()
            .any(|p| matches!(p.phase, Phase::Liquid | Phase::Aqueous));
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
                let what = if species::lookup(&p.species)
                    .is_some_and(|d| d.dissolves_without_speciation)
                {
                    format!(
                        "{name} dissolves, but no wired engine speciates it: it contributes nothing to the pH or the ionic strength here, and those numbers are for everything else in the beaker"
                    )
                } else {
                    format!(
                        "{name} in contact with liquid: no wired solver models this dissolution/reaction"
                    )
                };
                events.push(Event::NotYetModeled {
                    vessel: vessel.id,
                    what,
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

        if vessel.solution.is_some() {
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
                let what = if species::lookup(&p.species)
                    .is_some_and(|d| d.dissolves_without_speciation)
                {
                    format!(
                        "{name} dissolves, but no wired engine speciates it: it contributes nothing to the pH or the ionic strength here, and those numbers are for everything else in the beaker"
                    )
                } else {
                    format!(
                        "{name} in contact with liquid: no wired solver models this dissolution/reaction"
                    )
                };
                events.push(Event::NotYetModeled {
                    vessel: vessel.id,
                    what,
                });
            }
        }

        Ok((delta, events))
    }
}

/// Mix incoming matter at `t_in` with heat capacity `cp_in` (J/K) into a
/// vessel currently at `t_vessel` with heat capacity `cp_vessel` (J/K),
/// adiabatically. Returns the common final temperature.
///
/// Energy balance: cp_v·(T_f − T_v) + cp_in·(T_f − T_in) = 0.
pub fn adiabatic_mix_temperature(
    t_vessel: Kelvin,
    cp_vessel: f64,
    t_in: Kelvin,
    cp_in: f64,
) -> Kelvin {
    let total = cp_vessel + cp_in;
    if total <= 0.0 {
        return t_in;
    }
    Kelvin((cp_vessel * t_vessel.0 + cp_in * t_in.0) / total)
}

#[cfg(test)]
mod route_trace_tests {
    use super::*;

    struct TestRoute {
        name: &'static str,
        applies: bool,
        kind: SolverRouteKind,
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
}
