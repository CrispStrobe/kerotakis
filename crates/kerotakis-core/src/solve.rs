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
    fn equilibrate(&self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError>;
}

/// L0. Runs before any chemistry; may veto.
pub trait SafetyScreen {
    /// `Some(reason)` vetoes the step.
    fn veto(&self, vessel: &Vessel) -> Option<String>;
}

/// v0 screen: permissive. Replaced in P1 by the reimplemented NOAA
/// reactive-group matrix — this type exists so the loop is wired for a veto
/// from day one.
pub struct PermissiveScreen;

impl SafetyScreen for PermissiveScreen {
    fn veto(&self, _vessel: &Vessel) -> Option<String> {
        None
    }
}

/// v0 equilibrator: honest physics, no chemistry.
///
/// - Thermostatted vessels relax to their bath temperature.
/// - Solids sitting in liquid are reported as `NotYetModeled` (dissolution
///   arrives with the aqueous solver in P2) rather than silently ignored or
///   faked.
///
/// Thermal mixing itself happens in the bench loop when matter enters at a
/// different temperature; by the time this runs, the vessel already has a
/// single well-defined T.
pub struct MixingEquilibrator;

impl Equilibrator for MixingEquilibrator {
    fn name(&self) -> &'static str {
        "mixing-v0"
    }

    fn equilibrate(&self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
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
                        "{name} in contact with liquid: dissolution/reaction needs the aqueous solver (not wired yet)"
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
