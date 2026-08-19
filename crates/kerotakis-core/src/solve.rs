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
            if !solver.applies(vessel) {
                continue;
            }
            match solver.equilibrate(vessel) {
                Ok(mut more) => events.append(&mut more),
                // One solver failing must not silence the rest. The stack is
                // a sequence of independent questions — what dissolves, what
                // burns, what state the solvent is in — and an aqueous
                // engine that cannot answer the first has nothing to say
                // about the third. Aborting here left water liquid at
                // −24 °C, because the freezing pass never ran once PHREEQC
                // had declined the solution.
                Err(e) => events.push(Event::SolverFailed {
                    vessel: vessel.id,
                    solver: solver.name().to_string(),
                    detail: e.to_string(),
                }),
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

/// Freezing and boiling: the solvent is allowed to stop being a liquid.
///
/// Runs *after* the aqueous engine, because where a solution freezes
/// depends on what is dissolved in it, and only the speciation knows how
/// many particles that is. If the vessel turns out to be outside its
/// liquid range, the aqueous answer is **withdrawn** — a block of ice does
/// not have a pH, and continuing to report one was the original bug.
///
/// Partial freezing is deliberately not modelled. A real solution freezing
/// gives ice plus an ever more concentrated brine, down to a eutectic, and
/// pretending otherwise would be a worse lie than admitting the gap.
pub struct StateEquilibrator;

/// The solvent. Every transition here is water's; a non-aqueous solvent is
/// a separate problem and says so rather than borrowing water's constants.
const SOLVENT: &str = "water";

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
        let solute_molality = match &vessel.solution {
            Some(info) => info
                .species
                .iter()
                .filter(|s| s.name != "H2O")
                .map(|s| s.molality)
                .sum::<f64>(),
            None => 0.0,
        };
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
            // Energy that would have to leave to get this cold, spent on
            // freezing instead.
            let excess_j = cp * (t.freezing_k - now);
            let latent_total = liquid_moles * crate::states::WATER_H_FUS;
            let freezing = (excess_j / crate::states::WATER_H_FUS).min(liquid_moles);

            for p in vessel.contents.iter_mut() {
                if p.species == solvent && p.phase == Phase::Liquid {
                    p.moles = Moles((p.moles.0 - freezing).max(0.0));
                }
            }
            vessel.contents.retain(|p| p.moles.0 > 1e-12);
            vessel.deposit(solvent.clone(), Moles(freezing), Phase::Solid);

            vessel.temperature = if excess_j < latent_total {
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
            if solute_molality > 1e-6 {
                events.push(Event::NotYetModeled {
                    vessel: vessel.id,
                    what: "what the dissolved substances do as the water freezes: real freezing concentrates them into a brine and ends at a eutectic, and this lab does not model that yet".to_string(),
                });
            }
        } else if frozen_water && now > t.freezing_k {
            // Melting, with the same plateau in reverse.
            let available_j = cp * (now - t.freezing_k);
            let melting = (available_j / crate::states::WATER_H_FUS).min(frozen_moles);
            let latent_total = frozen_moles * crate::states::WATER_H_FUS;

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
            // Frozen solvent is not an unmodelled dissolution — the state
            // pass just explained it, and saying "ice in contact with
            // liquid: no solver models this" would be noise dressed as
            // honesty.
            if p.species == SpeciesId::new(SOLVENT) {
                continue;
            }
            if p.phase == Phase::Solid && has_liquid {
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
