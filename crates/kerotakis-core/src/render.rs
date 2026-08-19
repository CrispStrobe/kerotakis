//! Register rendering: one event stream, three voices. Deterministic
//! templates over solver output — never a language model (PLAN.md).
//!
//! The solver has no idea who is asking; this module is the only place that
//! does.

use serde::{Deserialize, Serialize};

use crate::ops::{Event, Instrument};
use crate::species;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Register {
    /// ~age 9: observations in plain words.
    Child,
    /// ~age 15: equations and quantities.
    Student,
    /// Full numeric output.
    Expert,
}

pub fn render_event(event: &Event, register: Register) -> String {
    match event {
        Event::VesselCreated { vessel } => match register {
            Register::Child => format!("A fresh beaker appears on the bench: {vessel}."),
            _ => format!("{vessel}: new vessel"),
        },
        Event::Added {
            vessel,
            species: sid,
            moles,
        } => {
            let name = species::lookup(sid).map(|d| d.name).unwrap_or(sid.0.as_str());
            match register {
                Register::Child => format!("You add {name} to {vessel}."),
                Register::Student => format!("{vessel}: +{:.4} mol {name}", moles.0),
                Register::Expert => {
                    let extra = species::lookup(sid)
                        .map(|d| format!(" ({}, M = {:.3} g/mol)", d.formula, d.molar_mass))
                        .unwrap_or_default();
                    format!("{vessel}: +{:.6} mol {name}{extra}", moles.0)
                }
            }
        }
        Event::TemperatureChanged { vessel, from, to } => {
            let d = to.0 - from.0;
            match register {
                Register::Child => {
                    if d.abs() < 0.05 {
                        format!("{vessel} stays about the same temperature.")
                    } else if d > 0.0 {
                        format!("{vessel} gets warmer!")
                    } else {
                        format!("{vessel} gets colder!")
                    }
                }
                Register::Student => format!(
                    "{vessel}: {:.1} °C → {:.1} °C",
                    from.to_celsius(),
                    to.to_celsius()
                ),
                Register::Expert => format!(
                    "{vessel}: T {:.3} K → {:.3} K (ΔT = {d:+.3} K)",
                    from.0, to.0
                ),
            }
        }
        Event::Filtered { from, to } => match register {
            Register::Child => format!(
                "You pour {from} through the filter paper — the liquid runs into {to}, and the solid stays behind on the paper."
            ),
            _ => format!("{from} → {to}: filtrate passed; residue retained"),
        },
        Event::Evaporated { vessel, moles } => match register {
            Register::Child => format!("Steam rises from {vessel} — the water is boiling away!"),
            Register::Student => format!("{vessel}: {:.3} mol water evaporated", moles.0),
            Register::Expert => format!("{vessel}: {:.6} mol H2O evaporated (vaporisation enthalpy not yet in the balance)", moles.0),
        },
        Event::Transferred { from, to, fraction } => match register {
            Register::Child => format!("You pour some of {from} into {to}."),
            _ => format!("{from} → {to}: {:.0}% of the liquid", fraction * 100.0),
        },
        Event::Dissolved {
            vessel,
            species: sid,
            moles,
        } => {
            let name = species::lookup(sid).map(|d| d.name).unwrap_or(sid.0.as_str());
            match register {
                Register::Child => format!("The {name} disappears into the water in {vessel}!"),
                Register::Student => format!("{vessel}: {:.4} mol {name} dissolved", moles.0),
                Register::Expert => format!("{vessel}: {:.6} mol {name} dissolved", moles.0),
            }
        }
        Event::Precipitated {
            vessel,
            species: sid,
            moles,
        } => {
            let data = species::lookup(sid);
            let name = data.map(|d| d.name).unwrap_or(sid.0.as_str());
            match register {
                Register::Child => {
                    let colour = data.and_then(|d| d.appearance).unwrap_or("new");
                    format!(
                        "It went cloudy in {vessel}! A {colour} solid appears at the bottom — that's called a precipitate."
                    )
                }
                Register::Student => {
                    format!("{vessel}: {:.4} mol {name} precipitated ↓", moles.0)
                }
                Register::Expert => {
                    format!("{vessel}: {:.6} mol {name} precipitated", moles.0)
                }
            }
        }
        Event::Consumed {
            vessel,
            species: sid,
            moles,
        } => {
            let name = species::lookup(sid).map(|d| d.name).unwrap_or(sid.0.as_str());
            match register {
                Register::Child => format!("The {name} in {vessel} is used up."),
                Register::Student => format!("{vessel}: {:.4} mol {name} consumed", moles.0),
                Register::Expert => format!("{vessel}: −{:.6} mol {name}", moles.0),
            }
        }
        Event::Ignited { vessel, flame } => match register {
            Register::Child => match flame {
                Some(colour) => {
                    format!("It catches fire in {vessel} — burning with {colour} light!")
                }
                None => format!("It catches fire in {vessel}!"),
            },
            Register::Student => match flame {
                Some(colour) => format!("{vessel}: ignited — {colour} flame"),
                None => format!("{vessel}: ignited"),
            },
            Register::Expert => format!("{vessel}: ignition source applied"),
        },
        Event::FlameTest {
            vessel,
            species: sid,
            colour,
        } => {
            let name = species::lookup(sid).map(|d| d.name).unwrap_or(sid.0.as_str());
            match register {
                Register::Child => format!(
                    "It does not catch fire — but look: it turns the flame {colour}! Every metal has its own colour, which is how you can tell them apart."
                ),
                Register::Student => {
                    format!("{vessel}: flame test — {name} colours the flame {colour}")
                }
                Register::Expert => format!(
                    "{vessel}: no combustion; characteristic emission of {name} ({colour})"
                ),
            }
        }
        Event::DidNotIgnite { vessel } => match register {
            Register::Child => {
                format!("You hold the flame to {vessel} — and nothing happens. Not everything burns.")
            }
            _ => format!("{vessel}: nothing ignited"),
        },
        Event::ThermalEquilibrium {
            vessel,
            temperature,
            provenance,
        } => match register {
            Register::Child => format!("Everything in {vessel} settles into what it wants to be at this heat."),
            Register::Student => format!(
                "{vessel}: thermal equilibrium at {:.0} °C",
                temperature.to_celsius()
            ),
            Register::Expert => format!(
                "{vessel}: Gibbs minimum at {:.2} K · {} · {}",
                temperature.0, provenance.dataset, provenance.model
            ),
        },
        Event::SolutionCharacterized {
            vessel,
            ph,
            ionic_strength,
        } => match register {
            Register::Child => {
                if *ph < 6.0 {
                    format!("The liquid in {vessel} is an acid.")
                } else if *ph > 8.0 {
                    format!("The liquid in {vessel} is a base (the opposite of an acid).")
                } else {
                    format!("The liquid in {vessel} is neutral — like pure water.")
                }
            }
            Register::Student => format!("{vessel}: pH {ph:.2}"),
            Register::Expert => {
                format!("{vessel}: pH {ph:.3} · I = {ionic_strength:.4} mol/kgw")
            }
        },
        Event::Measured {
            vessel,
            instrument,
            value,
            unit,
        } => {
            let device = match instrument {
                Instrument::Thermometer => "thermometer",
                Instrument::Balance => "balance",
                Instrument::PhMeter => "pH meter",
            };
            match register {
                Register::Child => format!("The {device} on {vessel} reads {value:.0} {unit}."),
                Register::Student => format!("{vessel} {device}: {value:.2} {unit}"),
                Register::Expert => format!("{vessel} {device}: {value:.4} {unit}"),
            }
        }
        Event::HazardWarning {
            severity,
            hazard,
            real_world,
        } => match register {
            Register::Child => format!(
                "⚠️  STOP AND READ: {hazard}. {real_world} NEVER try this outside the virtual lab — here, we can watch what happens safely."
            ),
            Register::Student => format!(
                "⚠ HAZARD ({severity:?}): {hazard} — {real_world} Safe only because this lab is virtual."
            ),
            Register::Expert => format!("HAZARD [{severity:?}] (L0): {hazard}; {real_world}"),
        },
        Event::SafetyVeto { reason } => match register {
            Register::Child => format!("The lab won't do that: {reason}"),
            _ => format!("SAFETY VETO (L0): {reason}"),
        },
        Event::ReactionOccurred { vessel, equation } => match register {
            Register::Child => format!("The mixture in {vessel} changes — something new is forming!"),
            _ => format!("{vessel}: {equation}"),
        },
        Event::GasEvolved {
            vessel,
            species: sid,
            moles,
        } => {
            let data = species::lookup(sid);
            let name = data.map(|d| d.name).unwrap_or(sid.0.as_str());
            let toxic = data
                .and_then(|d| d.appearance)
                .is_some_and(|a| a.contains("toxic"));
            match register {
                Register::Child => {
                    if toxic {
                        format!(
                            "A gas rises out of {vessel} — this one is poisonous. In a real room you would have to leave NOW."
                        )
                    } else {
                        format!("Bubbles! A gas rises out of {vessel}.")
                    }
                }
                Register::Student => {
                    format!("{vessel}: {:.4} mol {name} ↑ (gas escapes the open vessel)", moles.0)
                }
                Register::Expert => {
                    format!("{vessel}: {:.6} mol {name} evolved (open system; mass leaves)", moles.0)
                }
            }
        }
        Event::NotYetModeled { vessel, what } => match register {
            Register::Child => format!("Hmm — nothing visible happens in {vessel} (this part of the lab isn't awake yet)."),
            Register::Student => format!("{vessel}: not yet modelled — {what}"),
            Register::Expert => format!("{vessel}: NOT MODELLED: {what}"),
        },
        Event::SolverFailed {
            vessel,
            solver,
            detail,
        } => match register {
            Register::Child => format!(
                "The lab couldn't work out what happens in {vessel}. That's honest — better than guessing!"
            ),
            _ => format!("{vessel}: solver '{solver}' failed: {detail}"),
        },
    }
}
