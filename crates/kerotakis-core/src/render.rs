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
        Event::Transferred { from, to, fraction } => match register {
            Register::Child => format!("You pour some of {from} into {to}."),
            _ => format!("{from} → {to}: {:.0}% of the liquid", fraction * 100.0),
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
            };
            match register {
                Register::Child => format!("The {device} on {vessel} reads {value:.0} {unit}."),
                Register::Student => format!("{vessel} {device}: {value:.2} {unit}"),
                Register::Expert => format!("{vessel} {device}: {value:.4} {unit}"),
            }
        }
        Event::SafetyVeto { reason } => match register {
            Register::Child => format!("Stop! That would be dangerous: {reason}"),
            _ => format!("SAFETY VETO (L0): {reason}"),
        },
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
