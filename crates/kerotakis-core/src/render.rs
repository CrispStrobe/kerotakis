//! Register rendering: one event stream, three voices. Deterministic
//! templates over solver output — never a language model (PLAN.md).
//!
//! The solver has no idea who is asking; this module is the only place that
//! does.

use serde::{Deserialize, Serialize};

use crate::ops::{Event, Instrument};
use crate::species;
use crate::species::Phase;
use crate::vessel::{Headspace, SolutionInfo, Vessel};

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

/// Render a vessel for a person. CLI, Wasm, and future clients share this
/// instead of teaching each interface how to turn the state contract back
/// into laboratory prose.
pub fn render_vessel(v: &Vessel, register: Register) -> Vec<String> {
    let mut out = Vec::new();
    let solution = v
        .solution
        .as_ref()
        .map(|s| {
            let redox = match (s.pe, s.eh_volts(v.temperature.0)) {
                (Some(pe), Some(eh)) => format!(", pe {pe:.2} ({eh:+.3} V)"),
                _ => String::new(),
            };
            format!(", pH {:.2}{redox}, I = {:.4} m", s.ph, s.ionic_strength)
        })
        .unwrap_or_default();
    let boundary = match v.headspace {
        Headspace::Open => ", open to atmosphere".to_string(),
        Headspace::Sealed { volume } => format!(
            ", sealed {:.1} mL headspace at {:.3} bar",
            volume.0 * 1000.0,
            v.pressure.0 / 100_000.0
        ),
        Headspace::PressureControlled { pressure, volume } => format!(
            ", pressure-controlled {:.1} mL headspace at {:.3} bar",
            volume.0 * 1000.0,
            pressure.0 / 100_000.0
        ),
        Headspace::Swept { pressure } => {
            format!(", nitrogen-swept at {:.3} bar", pressure.0 / 100_000.0)
        }
    };
    out.push(format!(
        "{} ({}) — {:.2} °C, {:.1} g, {:.1} mL liquid{boundary}{solution}",
        v.id,
        v.label,
        v.temperature.to_celsius(),
        v.mass().0 + 0.0,
        v.liquid_volume().0 * 1000.0 + 0.0
    ));
    if let Some(redox) = redox_words(v.solution.as_ref()) {
        out.push(format!("    redox — {redox}"));
    }
    for p in &v.contents {
        let name = species::lookup(&p.species)
            .map(|d| d.name)
            .unwrap_or(p.species.0.as_str());
        out.push(format!(
            "    {:>10.4} mol  {:<18} {:?}",
            p.moles.0, name, p.phase
        ));
    }
    for solid_solution in &v.solid_solutions {
        out.push(format!(
            "    {:>10.4} mol  {} mixed crystal",
            solid_solution.total_moles().0,
            solid_solution.label
        ));
        if register >= Register::LV2 {
            for component in &solid_solution.components {
                out.push(format!(
                    "      {:>10.4} mol  {}",
                    component.moles.0,
                    component.component.species()
                ));
            }
        }
    }
    if v.is_empty() {
        out.push("    (empty)".to_string());
    }
    if register >= Register::LV3 {
        if let Some(info) = &v.solution {
            if !info.species.is_empty() {
                out.push("    speciation (mol/kgw · activity · γ):".to_string());
                for sp in &info.species {
                    let gamma = if sp.molality > 0.0 {
                        sp.activity / sp.molality
                    } else {
                        0.0
                    };
                    out.push(format!(
                        "      {:<12} {:>12.4e} {:>12.4e}   γ={:.3}",
                        sp.name, sp.molality, sp.activity, gamma
                    ));
                }
            }
        }
    }
    out
}

fn redox_words(solution: Option<&SolutionInfo>) -> Option<String> {
    let s = solution?;
    let mut elements: Vec<&str> = s.redox.iter().map(|r| r.element.as_str()).collect();
    elements.sort_unstable();
    elements.dedup();
    let mut parts = Vec::new();
    for element in elements {
        let states: Vec<_> = s
            .redox
            .iter()
            .filter(|state| state.element == element)
            .collect();
        let total: f64 = states.iter().map(|state| state.molality).sum();
        if total <= 0.0 {
            continue;
        }
        let visible: Vec<_> = states
            .into_iter()
            .filter(|state| state.molality / total >= 0.005)
            .collect();
        if visible.len() == 1 {
            parts.push(format!("all {element} as {}", visible[0].label()));
        } else if !visible.is_empty() {
            let split: Vec<_> = visible
                .iter()
                .map(|state| format!("{:.0}% {}", 100.0 * state.molality / total, state.label()))
                .collect();
            parts.push(format!("{element}: {}", split.join(", ")));
        }
    }
    (!parts.is_empty()).then(|| parts.join("; "))
}

/// Render a step's events for a person: the observable ones, in order.
///
/// At lv1 identical lines collapse to one. Every unmodelled note renders
/// there as the same sentence — "this part of the lab isn't awake yet" —
/// and a step with two different notes read as a stutter; a young reader
/// cannot use the distinction between them anyway. lv2 and above keep
/// every line, because there the notes render distinctly.
pub fn render_events(events: &[Event], register: Register) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for event in events.iter().filter(|e| e.is_observable()) {
        let line = render_event(event, register);
        if register.level() == 1 && out.contains(&line) {
            continue;
        }
        out.push(line);
    }
    out
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
        Event::Plated {
            vessel,
            species: sid,
            onto,
            moles,
        } => {
            let data = species::lookup(sid);
            let name = data.map(|d| d.name).unwrap_or(sid.0.as_str());
            let host = species::lookup(onto).map(|d| d.name).unwrap_or(onto.0.as_str());
            match register.level() {
                1 => {
                    let colour = data.and_then(|d| d.appearance).unwrap_or("new");
                    format!(
                        "A {colour} coating of {name} grows on the {host} in {vessel} — the {name} came out of the water onto it."
                    )
                }
                2 => format!("{vessel}: {:.4} mol {name} plated out onto {host}", moles.0),
                _ => format!("{vessel}: {:.6} mol {name} plated out onto {host}", moles.0),
            }
        }
        Event::Inert { vessel, species: sid, why } => {
            let name = species::lookup(sid).map(|d| d.name).unwrap_or(sid.0.as_str());
            match register.level() {
                1 => format!(
                    "Nothing happens to the {name} in {vessel} — and that is the real answer, not a gap: it is too unreactive for this."
                ),
                2 => format!("{vessel}: {name} does not react — {why}"),
                _ => format!("{vessel}: {name} inert: {why}"),
            }
        }
        Event::Consumed {
            vessel,
            species: sid,
            moles,
            remaining,
        } => {
            let name = species::lookup(sid).map(|d| d.name).unwrap_or(sid.0.as_str());
            match register.level() {
                // Says exactly as much as the event knows. With the
                // remainder on the event: gone, or some of it gone. Without
                // it, only that it is being used up — never a completeness
                // the emitter did not report.
                1 => match remaining {
                    Some(left) if left.0 < crate::OBSERVABLE_MOLES => {
                        format!("The {name} in {vessel} is used up.")
                    }
                    Some(_) => format!("Some of the {name} in {vessel} is used up."),
                    None => format!("The {name} in {vessel} is being used up."),
                },
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
                Instrument::PressureGauge => "pressure gauge",
                Instrument::VolumeMeter => "volume meter",
                Instrument::ConductivityMeter => "conductivity meter",
                Instrument::Spectrophotometer => "spectrophotometer",
            };
            match register.level() {
                1 => format!("The {device} on {vessel} reads {value:.0} {unit}."),
                2 => format!("{vessel} {device}: {value:.2} {unit}"),
                _ => format!("{vessel} {device}: {value:.4} {unit}"),
            }
        }
        Event::Electrolysed {
            vessel,
            species,
            coulombs,
            electrons,
            moles,
            grams,
            per_ion,
        } => {
            let name = species::lookup(species).map(|d| d.name).unwrap_or(&species.0);
            match register.level() {
                1 => format!("{grams:.2} g of {name} builds up on the electrode in {vessel}."),
                2 => format!(
                    "{vessel}: {coulombs:.0} C → {:.4} mol e⁻ → {:.4} mol {name} = {grams:.3} g",
                    electrons.0, moles.0
                ),
                // The chain, with the one step that is chemistry rather
                // than arithmetic marked: everything else is division.
                _ => format!(
                    "{vessel}: Q = {coulombs:.1} C; n(e⁻) = Q/F = {:.6} mol; \
                     n({name}) = n(e⁻)/{per_ion:.0} = {:.6} mol; \
                     m = {grams:.4} g — only the {per_ion:.0} is chemistry. \
                     Inert anode assumed: the water is oxidised there, so the \
                     oxygen leaves and the acid stays",
                    electrons.0, moles.0
                ),
            }
        }
        Event::CellVoltage {
            anode,
            cathode,
            volts,
            standard_volts,
            notation,
            equation,
        } => match register.level() {
            1 => format!(
                "The voltmeter between {anode} and {cathode} reads {volts:.2} V — you have made a battery! The electrons want to flow from {anode} to {cathode}. (Nothing is using the current yet, so nothing in the beakers changes.)"
            ),
            2 => format!(
                "{notation}: E = {volts:.3} V open-circuit (E° = {standard_volts:.3} V); electrons would flow {anode} → {cathode}; closing the circuit would run {equation}. No current is drawn, so this is the voltage the cell *offers*, not what it delivers under load"
            ),
            _ => format!(
                "{notation}: E_cell = {volts:.4} V open-circuit, no current, no internal resistance modelled (E°_cell = {standard_volts:.4} V; the difference is the Nernst term over the computed ion activities; ideal salt bridge, no liquid-junction potential); anode {anode}, cathode {cathode}; {equation}"
            ),
        },
        Event::NoCell { a, b, why } => match register.level() {
            1 => format!("The voltmeter between {a} and {b} reads nothing — one of them isn't a proper half-cell yet."),
            2 => format!("{a}–{b}: no cell — {why}"),
            _ => format!("{a}–{b}: no cell: {why}"),
        },
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
        Event::GasAbsorbed {
            vessel,
            species: sid,
            moles,
        } => {
            let name = species::lookup(sid)
                .map(|data| data.name)
                .unwrap_or(sid.0.as_str());
            match register.level() {
                1 => format!("Gas bubbles into {vessel} and is taken up by the liquid."),
                2 => format!(
                    "{vessel}: {:.4} mol {name} absorbed from the gas boundary",
                    moles.0
                ),
                _ => format!(
                    "{vessel}: {:.6} mol {name} transferred gas → condensed inventory",
                    moles.0
                ),
            }
        }
        Event::GasContained {
            vessel,
            species: sid,
            moles,
        } => {
            let name = species::lookup(sid)
                .map(|data| data.name)
                .unwrap_or(sid.0.as_str());
            match register.level() {
                1 => format!("Bubbles form in {vessel}, but the gas stays inside."),
                2 => format!(
                    "{vessel}: {:.4} mol {name} formed and remains in the closed headspace",
                    moles.0
                ),
                _ => format!(
                    "{vessel}: {:.6} mol {name} transferred to the finite headspace (closed system; mass retained)",
                    moles.0
                ),
            }
        }
        Event::VesselSealed {
            vessel,
            headspace_volume,
            trapped_air,
        } => match register.level() {
            1 => format!("A lid seals {vessel}. Nothing gaseous can escape now."),
            2 => format!(
                "{vessel}: sealed over {:.3} L of headspace, trapping {:.4} mol of room air",
                headspace_volume.0, trapped_air.0
            ),
            _ => format!(
                "{vessel}: boundary=open → sealed; V_gas={:.6} L, trapped dry-air approximation={:.8} mol",
                headspace_volume.0, trapped_air.0
            ),
        },
        Event::VesselPressureControlled {
            vessel,
            pressure,
            initial_volume,
            trapped_gas,
        } => match register.level() {
            1 => format!("A movable piston holds {vessel} at constant pressure."),
            2 => format!(
                "{vessel}: pressure controlled at {:.3} bar; initial headspace {:.3} L",
                pressure.0 / 100_000.0,
                initial_volume.0
            ),
            _ => format!(
                "{vessel}: boundary=pressure_controlled; P={:.3} Pa, V_initial={:.6} L, trapped gas={:.8} mol",
                pressure.0, initial_volume.0, trapped_gas.0
            ),
        },
        Event::VesselSwept { vessel, pressure } => match register.level() {
            1 => format!("Nitrogen flows across {vessel} and carries gases away."),
            2 => format!(
                "{vessel}: swept by nitrogen at {:.3} bar; volatile products are purged",
                pressure.0 / 100_000.0
            ),
            _ => format!(
                "{vessel}: boundary=swept; inert N2 purge at P={:.3} Pa, gas inventory external",
                pressure.0
            ),
        },
        Event::VesselOpened { vessel } => match register.level() {
            1 => format!("The lid comes off {vessel}; its gas can escape into the room."),
            _ => format!("{vessel}: sealed boundary opened to the atmospheric reservoir"),
        },
        Event::HeadspaceEquilibrated {
            vessel,
            pressure,
            total_moles,
        } => match register.level() {
            1 => format!("The gas under the lid of {vessel} settles with the liquid."),
            2 => format!(
                "{vessel}: headspace settled at {:.3} bar with {:.4} mol gas",
                pressure.0 / 100_000.0,
                total_moles.0
            ),
            _ => format!(
                "{vessel}: finite-volume gas/liquid equilibrium; P={:.3} Pa, n_gas={:.8} mol",
                pressure.0, total_moles.0
            ),
        },
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

#[cfg(test)]
mod dedupe_tests {
    use super::*;
    use crate::vessel::VesselId;

    fn notes() -> Vec<Event> {
        vec![
            Event::NotYetModeled {
                vessel: VesselId(0),
                what: "one thing".to_string(),
            },
            Event::NotYetModeled {
                vessel: VesselId(0),
                what: "another thing".to_string(),
            },
        ]
    }

    #[test]
    fn identical_lv1_lines_collapse_and_lv2_lines_do_not() {
        assert_eq!(render_events(&notes(), Register::LV1).len(), 1);
        assert_eq!(render_events(&notes(), Register::LV2).len(), 2);
    }
}

#[cfg(test)]
mod consumed_tests {
    use super::*;
    use crate::units::Moles;
    use crate::vessel::VesselId;

    fn consumed(remaining: Option<f64>) -> Event {
        Event::Consumed {
            vessel: VesselId(0),
            species: crate::species::SpeciesId::new("Mg"),
            moles: Moles(0.01),
            remaining: remaining.map(Moles),
        }
    }

    #[test]
    fn lv1_says_only_what_the_event_knows() {
        assert_eq!(
            render_event(&consumed(Some(0.0)), Register::LV1),
            "The magnesium in v1 is used up."
        );
        assert_eq!(
            render_event(&consumed(Some(0.01)), Register::LV1),
            "Some of the magnesium in v1 is used up."
        );
        assert_eq!(
            render_event(&consumed(None), Register::LV1),
            "The magnesium in v1 is being used up."
        );
    }

    /// A log written before the field existed — or any client that never
    /// sends it — must still read, and must read as "not reported".
    #[test]
    fn a_consumed_event_without_the_remainder_still_deserialises() {
        let old = r#"{"event":"consumed","vessel":0,"species":"Mg","moles":0.01}"#;
        let e: Event = serde_json::from_str(old).expect("old shape parses");
        assert!(matches!(
            e,
            Event::Consumed {
                remaining: None,
                ..
            }
        ));
        assert_eq!(
            render_event(&e, Register::LV1),
            "The magnesium in v1 is being used up."
        );
    }
}

#[cfg(test)]
mod vessel_tests {
    use super::*;
    use crate::{Kelvin, Moles, SpeciesId, VesselId};

    #[test]
    fn a_frozen_vessel_is_prose_not_the_state_contract() {
        let mut vessel = Vessel::new(VesselId(0), "beaker");
        vessel.temperature = Kelvin::from_celsius(0.0);
        vessel.deposit(SpeciesId::new("water"), Moles(0.6122), Phase::Liquid);
        vessel.deposit(SpeciesId::new("water"), Moles(4.9221), Phase::Solid);

        let rendered = render_vessel(&vessel, Register::LV2).join("\n");
        assert!(rendered.contains("v1 (beaker) — 0.00 °C"), "{rendered}");
        assert!(rendered.contains("0.6122 mol  water"), "{rendered}");
        assert!(rendered.contains("Liquid"), "{rendered}");
        assert!(rendered.contains("4.9221 mol  water"), "{rendered}");
        assert!(rendered.contains("Solid"), "{rendered}");
        assert!(!rendered.contains("\"contents\""), "{rendered}");
        assert!(!rendered.contains('{'), "{rendered}");
    }
}
