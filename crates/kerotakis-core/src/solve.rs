//! The solver router and the L0 safety screen traits, with the v0
//! implementations: a physical mixing equilibrator (mass + energy balance,
//! no chemistry) and a permissive safety screen.
//!
//! The real L2/L2g/L3 engines plug in behind `Equilibrator`; the real
//! reactive-group matrix plugs in behind `SafetyScreen` (PLAN.md, P1/P2).

use crate::ops::Event;
use crate::species::{self, Phase};
use crate::units::Kelvin;
use crate::vessel::{ThermalMode, Vessel};

#[derive(Debug, thiserror::Error)]
pub enum SolveError {
    #[error("{solver} could not solve this state: {detail}")]
    NotConverged { solver: String, detail: String },
}

/// Re-equilibrates one vessel after an operator touched it.
pub trait Equilibrator {
    fn name(&self) -> &'static str;
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
}

/// Runs every applicable solver in order, concatenating their events. The
/// order is the routing: physics first, chemistry engines next, the honesty
/// pass last.
pub struct SolverStack {
    pub solvers: Vec<Box<dyn Equilibrator>>,
}

impl SolverStack {
    pub fn new(solvers: Vec<Box<dyn Equilibrator>>) -> Self {
        SolverStack { solvers }
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
        let mut events = Vec::new();
        for solver in &mut self.solvers {
            if solver.applies(vessel) {
                events.append(&mut solver.equilibrate(vessel)?);
            }
        }
        Ok(events)
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

        Ok(events)
    }
}

/// The honesty pass, last in every stack: any state no chemistry solver has
/// characterised is said out loud rather than silently ignored or faked.
///
/// A vessel whose `solution` is set was handled by an aqueous solver, so a
/// solid coexisting with liquid there is a real computed state (a
/// precipitate), not a gap.
pub struct HonestyEquilibrator;

impl Equilibrator for HonestyEquilibrator {
    fn name(&self) -> &'static str {
        "honesty"
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
            if p.phase == Phase::Solid && has_liquid {
                let name = species::lookup(&p.species)
                    .map(|d| d.name)
                    .unwrap_or(p.species.0.as_str());
                events.push(Event::NotYetModeled {
                    vessel: vessel.id,
                    what: format!(
                        "{name} in contact with liquid: no wired solver models this dissolution/reaction"
                    ),
                });
            }
        }
        Ok(events)
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
