//! Register rendering: one event stream, three voices. Deterministic
//! templates over solver output — never a language model (PLAN.md).
//!
//! The solver has no idea who is asking; this module is the only place that
//! does.

use serde::{Deserialize, Serialize};

use crate::ops::{Event, Instrument};
use crate::species;
use crate::species::Phase;

/// How much detail an answer is rendered with.
///
/// Deliberately a *number*, not a set of named audiences: ages and labels
/// like "child" bake in an assumption about who a level is for, and the
/// levels want to multiply later (a level between equations and full
/// numerics, say). Adding one means adding a match arm where the wording
/// genuinely differs — everything unspecified inherits the nearest level
/// below, so nothing has to be rewritten to make room.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Register(pub u8);

impl Register {
    /// Plain observation: what you would see and say.
    pub const LV1: Register = Register(1);
    /// Equations, names and quantities.
    pub const LV2: Register = Register(2);
    /// Full numeric detail, models and provenance.
    pub const LV3: Register = Register(3);

    pub fn level(self) -> u8 {
        self.0
    }

    /// Parse `lv2`, `2`, or `level2`. Unknown input is an error rather
    /// than a silent default: rendering at the wrong level is the kind of
    /// mistake nobody notices.
    pub fn parse(text: &str) -> Option<Register> {
        let t = text.trim().to_ascii_lowercase();
        let digits = t
            .trim_start_matches("level")
            .trim_start_matches("lv")
            .trim_start_matches('l');
        digits.parse::<u8>().ok().filter(|n| *n >= 1).map(Register)
    }
}

impl Default for Register {
    fn default() -> Self {
        Register::LV2
    }
}

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "lv{}", self.0)
    }
}

pub fn render_event(event: &Event, register: Register) -> String {
    match event {
        Event::VesselCreated { vessel } => match register.level() {
            1 => format!("A fresh beaker appears on the bench: {vessel}."),
            _ => format!("{vessel}: new vessel"),
        },
        Event::Added {
            vessel,
            species: sid,
            moles,
        } => {
            let name = species::lookup(sid).map(|d| d.name).unwrap_or(sid.0.as_str());
            match register.level() {
                1 => format!("You add {name} to {vessel}."),
                2 => format!("{vessel}: +{:.4} mol {name}", moles.0),
                _ => {
                    let extra = species::lookup(sid)
                        .map(|d| format!(" ({}, M = {:.3} g/mol)", d.formula, d.molar_mass))
                        .unwrap_or_default();
                    format!("{vessel}: +{:.6} mol {name}{extra}", moles.0)
                }
            }
        }
        Event::TemperatureChanged { vessel, from, to } => {
            let d = to.0 - from.0;
            match register.level() {
                1 => {
                    if d.abs() < 0.05 {
                        format!("{vessel} stays about the same temperature.")
                    } else if d > 0.0 {
                        format!("{vessel} gets warmer!")
                    } else {
                        format!("{vessel} gets colder!")
                    }
                }
                2 => format!(
                    "{vessel}: {:.1} °C → {:.1} °C",
                    from.to_celsius(),
                    to.to_celsius()
                ),
                _ => format!(
                    "{vessel}: T {:.3} K → {:.3} K (ΔT = {d:+.3} K)",
                    from.0, to.0
                ),
            }
        }
        Event::Filtered { from, to } => match register.level() {
            1 => format!(
                "You pour {from} through the filter paper — the liquid runs into {to}, and the solid stays behind on the paper."
            ),
            _ => format!("{from} → {to}: filtrate passed; residue retained"),
        },
        Event::Evaporated { vessel, moles } => match register.level() {
            1 => format!("Steam rises from {vessel} — the water is boiling away!"),
            2 => format!("{vessel}: {:.3} mol water evaporated", moles.0),
            _ => format!("{vessel}: {:.6} mol H2O evaporated (vaporisation enthalpy not yet in the balance)", moles.0),
        },
        Event::Transferred { from, to, fraction } => match register.level() {
            1 => format!("You pour some of {from} into {to}."),
            _ => format!("{from} → {to}: {:.0}% of the liquid", fraction * 100.0),
        },
        Event::Dissolved {
            vessel,
            species: sid,
            moles,
        } => {
            let name = species::lookup(sid).map(|d| d.name).unwrap_or(sid.0.as_str());
            match register.level() {
                1 => format!("The {name} disappears into the water in {vessel}!"),
                2 => format!("{vessel}: {:.4} mol {name} dissolved", moles.0),
                _ => format!("{vessel}: {:.6} mol {name} dissolved", moles.0),
            }
        }
        Event::Precipitated {
            vessel,
            species: sid,
            moles,
        } => {
            let data = species::lookup(sid);
            let name = data.map(|d| d.name).unwrap_or(sid.0.as_str());
            match register.level() {
                1 => {
                    let colour = data.and_then(|d| d.appearance).unwrap_or("new");
                    format!(
                        "It went cloudy in {vessel}! A {colour} solid appears at the bottom — that's called a precipitate."
                    )
                }
                2 => {
                    format!("{vessel}: {:.4} mol {name} precipitated ↓", moles.0)
                }
                _ => {
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
            match register.level() {
                1 => format!("The {name} in {vessel} is used up."),
                2 => format!("{vessel}: {:.4} mol {name} consumed", moles.0),
                _ => format!("{vessel}: −{:.6} mol {name}", moles.0),
            }
        }
        Event::Ignited { vessel, flame } => match register.level() {
            1 => match flame {
                Some(colour) => {
                    format!("It catches fire in {vessel} — burning with {colour} light!")
                }
                None => format!("It catches fire in {vessel}!"),
            },
            2 => match flame {
                Some(colour) => format!("{vessel}: ignited — {colour} flame"),
                None => format!("{vessel}: ignited"),
            },
            _ => format!("{vessel}: ignition source applied"),
        },
        Event::FlameTest {
            vessel,
            species: sid,
            colour,
        } => {
            let name = species::lookup(sid).map(|d| d.name).unwrap_or(sid.0.as_str());
            match register.level() {
                1 => format!(
                    "It does not catch fire — but look: it turns the flame {colour}! Every metal has its own colour, which is how you can tell them apart."
                ),
                2 => {
                    format!("{vessel}: flame test — {name} colours the flame {colour}")
                }
                _ => format!(
                    "{vessel}: no combustion; characteristic emission of {name} ({colour})"
                ),
            }
        }
        Event::DidNotIgnite { vessel } => match register.level() {
            1 => {
                format!("You hold the flame to {vessel} — and nothing happens. Not everything burns.")
            }
            _ => format!("{vessel}: nothing ignited"),
        },
        Event::ThermalEquilibrium {
            vessel,
            temperature,
            provenance,
        } => match register.level() {
            1 => format!("Everything in {vessel} settles into what it wants to be at this heat."),
            2 => format!(
                "{vessel}: thermal equilibrium at {:.0} °C",
                temperature.to_celsius()
            ),
            _ => format!(
                "{vessel}: Gibbs minimum at {:.2} K · {} · {}",
                temperature.0, provenance.dataset, provenance.model
            ),
        },
        Event::SolutionCharacterized {
            vessel,
            ph,
            ionic_strength,
        } => match register.level() {
            1 => {
                if *ph < 6.0 {
                    format!("The liquid in {vessel} is an acid.")
                } else if *ph > 8.0 {
                    format!("The liquid in {vessel} is a base (the opposite of an acid).")
                } else {
                    format!("The liquid in {vessel} is neutral — like pure water.")
                }
            }
            2 => format!("{vessel}: pH {ph:.2}"),
            _ => {
                format!("{vessel}: pH {ph:.3} · I = {ionic_strength:.4} mol/kgw")
            }
        },
        Event::Observed { vessel, appearance } => match register.level() {
            1 => format!("You look closely at {vessel}. {}", appearance.words),
            2 => format!("{vessel}: {}", appearance.words),
            _ => {
                let colour = appearance
                    .liquid
                    .map(|c| format!("#{:02X}{:02X}{:02X}", c.r, c.g, c.b))
                    .unwrap_or_else(|| "—".to_string());
                format!(
                    "{vessel}: {} (liquid {colour}, turbidity {:.2})",
                    appearance.words, appearance.cloudiness
                )
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
                Instrument::Eyes => "eyes",
            };
            match register.level() {
                1 => format!("The {device} on {vessel} reads {value:.0} {unit}."),
                2 => format!("{vessel} {device}: {value:.2} {unit}"),
                _ => format!("{vessel} {device}: {value:.4} {unit}"),
            }
        }
        Event::HazardWarning {
            severity,
            hazard,
            real_world,
        } => match register.level() {
            1 => format!(
                "⚠️  STOP AND READ: {hazard}. {real_world} NEVER try this outside the virtual lab — here, we can watch what happens safely."
            ),
            2 => format!(
                "⚠ HAZARD ({severity:?}): {hazard} — {real_world} Safe only because this lab is virtual."
            ),
            _ => format!("HAZARD [{severity:?}] (L0): {hazard}; {real_world}"),
        },
        Event::SafetyVeto { reason } => match register.level() {
            1 => format!("The lab won't do that: {reason}"),
            _ => format!("SAFETY VETO (L0): {reason}"),
        },
        Event::ReactionOccurred { vessel, equation } => match register.level() {
            1 => format!("The mixture in {vessel} changes — something new is forming!"),
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
            match register.level() {
                1 => {
                    if toxic {
                        format!(
                            "A gas rises out of {vessel} — this one is poisonous. In a real room you would have to leave NOW."
                        )
                    } else {
                        format!("Bubbles! A gas rises out of {vessel}.")
                    }
                }
                2 => {
                    format!("{vessel}: {:.4} mol {name} ↑ (gas escapes the open vessel)", moles.0)
                }
                _ => {
                    format!("{vessel}: {:.6} mol {name} evolved (open system; mass leaves)", moles.0)
                }
            }
        }
        Event::StateChanged {
            vessel,
            species,
            from,
            to,
            at,
            shifted_by,
        } => {
            let name = crate::species::lookup(species)
                .map(|d| d.name)
                .unwrap_or(species.0.as_str());
            let verb = match (from, to) {
                (Phase::Liquid, Phase::Solid) => "froze",
                (Phase::Solid, Phase::Liquid) => "melted",
                (Phase::Liquid, Phase::Gas) => "boiled",
                _ => "changed state",
            };
            let c = at.to_celsius();
            match register.level() {
                1 => match (from, to) {
                    (Phase::Liquid, Phase::Solid) => {
                        format!("The {name} in {vessel} turned to ice!")
                    }
                    (Phase::Solid, Phase::Liquid) => {
                        format!("The ice in {vessel} melted back into {name}.")
                    }
                    (Phase::Liquid, Phase::Gas) => {
                        format!("The {name} in {vessel} is boiling — look at the steam!")
                    }
                    _ => format!("The {name} in {vessel} {verb}."),
                },
                2 => {
                    if shifted_by.abs() < 0.05 {
                        format!("{vessel}: {name} {verb} at {c:.1} °C")
                    } else {
                        format!(
                            "{vessel}: {name} {verb} at {c:.1} °C — {:.1} °C {} than pure {name}, because of what is dissolved in it",
                            shifted_by.abs(),
                            if *shifted_by < 0.0 { "lower" } else { "higher" }
                        )
                    }
                }
                _ => format!(
                    "{vessel}: {name} {from:?} → {to:?} at {:.2} K ({c:.2} °C), shifted {shifted_by:+.3} K by dissolved particles",
                    at.0
                ),
            }
        }
        Event::Reacted {
            vessel,
            equation,
            moles,
            seconds,
            catalyst,
            activation_energy,
            ..
        } => match register.level() {
            1 => match catalyst {
                Some(c) => format!(
                    "In {vessel}, the {c} is making it happen much faster — after {seconds:.0} seconds a lot has changed!"
                ),
                None => format!("After {seconds:.0} seconds, something has been happening in {vessel}."),
            },
            2 => {
                let with = match catalyst {
                    Some(c) => format!(", sped up by {c}"),
                    None => String::new(),
                };
                format!(
                    "{vessel}: {:.4} mol reacted in {seconds:.0} s{with}  —  {equation}",
                    moles.0
                )
            }
            _ => {
                let with = match catalyst {
                    Some(c) => format!(" (catalyst: {c})"),
                    None => String::new(),
                };
                format!(
                    "{vessel}: extent {:.6} mol over {seconds:.1} s, Ea = {:.1} kJ/mol{with}  —  {equation}",
                    moles.0,
                    activation_energy / 1000.0
                )
            }
        },
        Event::NotYetModeled { vessel, what } => match register.level() {
            1 => format!("Hmm — nothing visible happens in {vessel} (this part of the lab isn't awake yet)."),
            2 => format!("{vessel}: not yet modelled — {what}"),
            _ => format!("{vessel}: NOT MODELLED: {what}"),
        },
        Event::SolverFailed {
            vessel,
            solver,
            detail,
        } => match register.level() {
            1 => format!(
                "The lab couldn't work out what happens in {vessel}. That's honest — better than guessing!"
            ),
            _ => format!("{vessel}: solver '{solver}' failed: {detail}"),
        },
    }
}
