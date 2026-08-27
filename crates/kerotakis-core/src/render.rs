//! Register rendering: one event stream, three voices. Deterministic
//! templates over solver output — never a language model (PLAN.md).
//!
//! The solver has no idea who is asking; this module is the only place that
//! does.

use crate::i18n::Locale;
use serde::{Deserialize, Serialize};

use crate::ops::{Event, Instrument};
use crate::species::Phase;
use crate::species::{self, SpeciesId};
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
    render_vessel_in(v, register, Locale::EN)
}

/// `render_vessel`, in the reader's language.
pub fn render_vessel_in(v: &Vessel, register: Register, locale: Locale) -> Vec<String> {
    let mut out = Vec::new();
    let solution = v
        .solution
        .as_ref()
        .map(|s| {
            let redox = match (s.pe, s.eh_volts(v.temperature.0)) {
                (Some(pe), Some(eh)) => format!(", pe {pe:.2} ({eh:+.3} V)"),
                _ => String::new(),
            };
            locale.number(format!(
                ", {} {:.2}{redox}, I = {:.4} m",
                locale.t("vessel.ph", "pH"),
                s.ph,
                s.ionic_strength
            ))
        })
        .unwrap_or_default();
    let boundary = match v.headspace {
        Headspace::Open => locale.t("vessel.open", ", open to atmosphere").to_string(),
        Headspace::Sealed { volume } => locale.number(format!(
            "{}",
            format_args!(
                "{}{:.1}{}{:.3}{}",
                locale.t("vessel.sealed-before", ", sealed "),
                volume.0 * 1000.0,
                locale.t("vessel.sealed-after", " mL headspace at "),
                v.pressure.0 / 100_000.0,
                " bar"
            )
        )),
        Headspace::PressureControlled { pressure, volume } => locale.number(format!(
            "{}{:.1}{}{:.3}{}",
            locale.t(
                "vessel.pressure-controlled-before",
                ", pressure-controlled "
            ),
            volume.0 * 1000.0,
            locale.t("vessel.sealed-after", " mL headspace at "),
            pressure.0 / 100_000.0,
            " bar"
        )),
        Headspace::Swept { pressure } => locale.number(format!(
            "{}{:.3}{}",
            locale.t("vessel.swept-before", ", nitrogen-swept at "),
            pressure.0 / 100_000.0,
            " bar"
        )),
    };
    // The id keeps its point (v1.2 is a name, not a number), so the comma
    // swap is applied to the measured part only.
    out.push(format!(
        "{} ({}) — {}",
        v.id,
        // A vessel absent from the catalogue keeps its English name: better
        // a word we have than a word we do not.
        locale
            .lookup(&format!("glassware.{}", v.label))
            .unwrap_or(v.label.as_str()),
        locale.number(format!(
            "{:.2} °C, {:.1} g, {:.1} mL {}{boundary}{solution}",
            v.temperature.to_celsius(),
            v.mass().0 + 0.0,
            v.liquid_volume().0 * 1000.0 + 0.0,
            locale.t("vessel.liquid", "liquid")
        ))
    ));
    if let Some(redox) = redox_words(v.solution.as_ref()) {
        out.push(format!(
            "    {} — {redox}",
            locale.t("vessel.redox", "redox")
        ));
    }
    for p in &v.contents {
        let english = species::lookup(&p.species)
            .map(|d| d.name)
            .unwrap_or(p.species.0.as_str());
        // The species catalogue is the engine's, so its German belongs to
        // the engine too — but only where a language has actually named a
        // species. An unnamed one reads in English inside a German line
        // rather than falling back to its formula, which nobody asked for.
        let name = locale
            .lookup(&format!("species.{english}"))
            .unwrap_or(english);
        // `{:?}` on the phase printed "Liquid" in every language. It is a
        // closed set of four words, so it is worth naming properly.
        let phase = locale.t(
            match p.phase {
                Phase::Aqueous => "phase.aqueous",
                Phase::Liquid => "phase.liquid",
                Phase::Solid => "phase.solid",
                Phase::Gas => "phase.gas",
            },
            match p.phase {
                Phase::Aqueous => "Aqueous",
                Phase::Liquid => "Liquid",
                Phase::Solid => "Solid",
                Phase::Gas => "Gas",
            },
        );
        // The comma goes here as well: a header reading 25,00 above rows
        // reading 11.0686 is worse than either convention used throughout.
        out.push(
            locale.number(format!("    {:>10.4} mol  ", p.moles.0))
                + &format!("{name:<18} {phase}"),
        );
    }
    for solid_solution in &v.solid_solutions {
        out.push(format!(
            "    {:>10.4} mol  {} {}",
            solid_solution.total_moles().0,
            solid_solution.label,
            locale.t("vessel.mixed-crystal", "mixed crystal")
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
        out.push(locale.t("vessel.empty", "    (empty)").to_string());
    }
    if register >= Register::LV3 {
        if let Some(info) = &v.solution {
            if !info.species.is_empty() {
                out.push(
                    locale
                        .t(
                            "vessel.speciation",
                            "    speciation (mol/kgw · activity · γ):",
                        )
                        .to_string(),
                );
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
    render_events_in(events, register, Locale::EN)
}

/// `render_events`, in the reader's language.
pub fn render_events_in(events: &[Event], register: Register, locale: Locale) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for event in events.iter().filter(|e| e.is_observable()) {
        let line = render_event_in(event, register, locale);
        if register.level() == 1 && out.contains(&line) {
            continue;
        }
        out.push(line);
    }
    out
}

pub fn render_event(event: &Event, register: Register) -> String {
    render_event_in(event, register, Locale::EN)
}

/// `render_event`, in the reader's language.
///
/// The old signature keeps meaning English so the eight existing callers —
/// mostly tests asserting on English prose — need no change, the same
/// reasoning as `render_vessel_in`.
pub fn render_event_in(event: &Event, register: Register, locale: Locale) -> String {
    match event {
        Event::VesselCreated { vessel } => match register.level() {
            1 => locale.fill(
                "event.vessel-created.lv1",
                "A fresh beaker appears on the bench: {vessel}.",
                &[("vessel", &vessel.to_string())],
            ),
            _ => locale.fill(
                "event.vessel-created.lv2",
                "{vessel}: new vessel",
                &[("vessel", &vessel.to_string())],
            ),
        },
        Event::VesselRemoved { vessel } => match register.level() {
            1 => locale.fill(
                "event.vessel-removed.lv1",
                "The empty {vessel} goes back into storage.",
                &[("vessel", &vessel.to_string())],
            ),
            _ => locale.fill(
                "event.vessel-removed.lv2",
                "{vessel}: empty vessel removed",
                &[("vessel", &vessel.to_string())],
            ),
        },
        Event::Added {
            vessel,
            species: sid,
            moles,
            total_after,
        } => {
            let name = species::lookup(sid).map(|d| d.name).unwrap_or(sid.0.as_str());
            let named = locale.lookup(&format!("species.{name}")).unwrap_or(name);
            match register.level() {
                1 => locale.fill(
                    "event.added.lv1",
                    "You add {what} to {vessel}.",
                    &[("what", named), ("vessel", &vessel.to_string())],
                ),
                2 => match total_after {
                    Some(total) if (total.0 - moles.0).abs() > 1e-12 => locale.fill(
                        "event.added.lv2-total",
                        "{vessel}: +{amount} mol {what} — {total} mol now in vessel",
                        &[
                            ("vessel", &vessel.to_string()),
                            ("amount", &locale.number(format!("{:.4}", moles.0))),
                            ("what", named),
                            ("total", &locale.number(format!("{:.4}", total.0))),
                        ],
                    ),
                    _ => locale.fill(
                        "event.added.lv2",
                        "{vessel}: +{amount} mol {what}",
                        &[
                            ("vessel", &vessel.to_string()),
                            ("amount", &locale.number(format!("{:.4}", moles.0))),
                            ("what", named),
                        ],
                    ),
                },
                _ => {
                    let extra = species::lookup(sid)
                        .map(|d| format!(" ({}, M = {:.3} g/mol)", d.formula, d.molar_mass))
                        .unwrap_or_default();
                    format!("{vessel}: +{:.6} mol {name}{extra}", moles.0)
                }
            }
        }
        Event::MaterialAdded {
            vessel,
            material,
            total_amount,
            basis,
            components,
            unresolved_amount,
            ..
        } => {
            let unit = match basis {
                crate::material::MaterialBasis::MassFraction => "g",
                crate::material::MaterialBasis::MoleFraction => "mol",
                crate::material::MaterialBasis::VolumeFraction => "mL",
            };
            match register.level() {
                1 => locale.fill(
                    "event.material-added.lv1",
                    "You add {material} to {vessel}.",
                    &[("material", &material.to_string()), ("vessel", &vessel.to_string())],
                ),
                2 => format!(
                    "{vessel}: +{total_amount:.3} {unit} {material} ({} known ingredients)",
                    components.len()
                ),
                _ => format!(
                    "{vessel}: +{total_amount:.6} {unit} {material}; {} canonical components, {unresolved_amount:.6} {unit} unresolved",
                    components.len()
                ),
            }
        }
        Event::GasProduced {
            vessel,
            species,
            moles,
            rate_moles_per_second,
            ..
        } => match register.level() {
            1 => locale.fill(
                "event.gas-produced.lv1",
                "{vessel}: {species} bubbles are being made.",
                &[("vessel", &vessel.to_string()), ("species", &species.to_string())],
            ),
            _ => format!(
                "{vessel}: {:.6} mol {species} produced ({rate_moles_per_second:.3e} mol/s)",
                moles.0
            ),
        },
        Event::ReactionHeatReleased {
            vessel,
            reaction,
            energy_j,
        } => match register.level() {
            1 => locale.fill(
                "event.reaction-heat-released.lv1",
                "{vessel} grows warmer as the reaction runs.",
                &[("vessel", &vessel.to_string())],
            ),
            _ => format!("{vessel}: {reaction} released {energy_j:.2} J"),
        },
        Event::FoamChanged {
            vessel,
            volume_liters,
            height_cm,
            overflow_liters,
            ..
        } => match register.level() {
            1 if *overflow_liters > 0.0 => {
                format!("Foam climbs out of {vessel} and spills over the rim!")
            }
            1 => locale.fill(
                "event.foam-changed.lv1",
                "Foam rises in {vessel}.",
                &[("vessel", &vessel.to_string())],
            ),
            _ => format!(
                "{vessel}: foam {volume_liters:.3} L, {height_cm:.1} cm high, overflow {overflow_liters:.3} L"
            ),
        },
        Event::TemperatureChanged { vessel, from, to } => {
            let d = to.0 - from.0;
            match register.level() {
                1 => {
                    if d.abs() < 0.05 {
                        locale.fill(
                            "event.temperature-changed.lv1-stays-same-temperature",
                            "{vessel} stays about the same temperature.",
                            &[("vessel", &vessel.to_string())],
                        )
                    } else if d > 0.0 {
                        locale.fill(
                            "event.temperature-changed.lv1-gets-warmer",
                            "{vessel} gets warmer!",
                            &[("vessel", &vessel.to_string())],
                        )
                    } else {
                        locale.fill(
                            "event.temperature-changed.lv1-gets-colder",
                            "{vessel} gets colder!",
                            &[("vessel", &vessel.to_string())],
                        )
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
        Event::Stirred {
            vessel,
            rpm,
            seconds,
            bar_length_m,
            tip_speed_m_s,
            resuspended_fraction,
            rate_coupled,
        } => match register.level() {
            1 => locale.fill(
                "event.stirred.lv1",
                "The magnetic stirrer spins {vessel} for {seconds} seconds.",
                &[
                    ("vessel", &vessel.to_string()),
                    ("seconds", &locale.number(format!("{seconds:.0}"))),
                ],
            ),
            2 => locale.fill(
                "event.stirred.lv2",
                "{vessel}: magnetic stirrer {rpm} rpm for {seconds} s — bar tip {tip} m/s; {resuspended}% resuspension",
                &[
                    ("vessel", &vessel.to_string()),
                    ("rpm", &locale.number(format!("{rpm:.0}"))),
                    ("seconds", &locale.number(format!("{seconds:.0}"))),
                    ("tip", &locale.number(format!("{tip_speed_m_s:.3}"))),
                    ("resuspended", &locale.number(format!("{:.0}", resuspended_fraction * 100.0))),
                ],
            ),
            _ => format!(
                "{vessel}: stir {rpm:.1} rpm × {seconds:.1} s; bar {:.1} mm; tip {:.5} m/s; resuspended {:.2}%; rate coupling {}",
                bar_length_m * 1000.0,
                tip_speed_m_s,
                resuspended_fraction * 100.0,
                if *rate_coupled { "active" } else { "not yet modelled" }
            ),
        },
        Event::Ground {
            vessel,
            species: sid,
            diameter_um,
            solid_moles,
            surface_area_m2,
            rate_coupled,
        } => {
            let name = species::lookup(sid)
                .map(|data| data.name)
                .unwrap_or(sid.0.as_str());
            match register.level() {
                1 => locale.fill(
                    "event.ground.lv1",
                    "You grind the {what} in {vessel} into a finer powder.",
                    &[
                        ("what", locale.lookup(&format!("species.{name}")).unwrap_or(name)),
                        ("vessel", &vessel.to_string()),
                    ],
                ),
                2 => locale.fill(
                    "event.ground.lv2",
                    "{vessel}: {what} ground to {diameter} µm — about {area} m² surface area",
                    &[
                        ("vessel", &vessel.to_string()),
                        ("what", locale.lookup(&format!("species.{name}")).unwrap_or(name)),
                        ("diameter", &locale.number(format!("{diameter_um:.1}"))),
                        ("area", &locale.number(format!("{surface_area_m2:.3}"))),
                    ],
                ),
                _ => format!(
                    "{vessel}: grind {name}; {:.6} mol solid; mean diameter {:.3} µm; spherical-particle area {:.6} m²; rate coupling {}",
                    solid_moles.0,
                    diameter_um,
                    surface_area_m2,
                    if *rate_coupled { "active" } else { "not yet modelled" }
                ),
            }
        }
        Event::Centrifuged {
            vessel,
            rpm,
            seconds,
            rotor_radius_m,
            rcf,
            sample_mass_g,
            counterbalance_g,
            imbalance_g,
            fluid_density_kg_m3,
            dynamic_viscosity_pa_s,
            separations,
            state_coupled,
        } => {
            let strongest = separations
                .iter()
                .map(|separation| separation.separated_fraction)
                .fold(0.0_f64, f64::max);
            match register.level() {
                1 => format!(
                    "The mini centrifuge spins {vessel}; the particles travel {:.0}% of the tube path.",
                    strongest * 100.0
                ),
                2 => format!(
                    "{vessel}: {rpm:.0} rpm for {seconds:.0} s — {rcf:.0} × g; {:.0}% separation; balanced within {imbalance_g:.2} g",
                    strongest * 100.0
                ),
                _ => {
                    let detail = separations
                        .iter()
                        .map(|separation| {
                            let assumption = if separation.particle_size_assumed {
                                " (diameter assumed)"
                            } else {
                                ""
                            };
                            format!(
                                "{}: {:.1} µm{}, v={:.6} m/s, x={:.5} m, {:.1}% {:?}",
                                separation.species,
                                separation.particle_diameter_um,
                                assumption,
                                separation.terminal_speed_m_s,
                                separation.distance_m,
                                separation.separated_fraction * 100.0,
                                separation.direction,
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("; ");
                    format!(
                        "{vessel}: centrifuge {rpm:.1} rpm × {seconds:.1} s, r={:.3} m, RCF={rcf:.2}; sample={sample_mass_g:.3} g, counterbalance={counterbalance_g:.3} g, Δm={imbalance_g:.3} g; ρfluid={fluid_density_kg_m3:.1} kg/m³, μ={dynamic_viscosity_pa_s:.6} Pa·s; {detail}; state coupling {}",
                        rotor_radius_m,
                        if *state_coupled { "active" } else { "not yet modelled" }
                    )
                }
            }
        }
        Event::GravitySettled {
            vessel,
            seconds,
            separations,
        } => {
            let strongest = separations
                .iter()
                .map(|separation| separation.separated_fraction)
                .fold(0.0_f64, f64::max);
            match register.level() {
                1 => locale.fill(
                    "event.gravity-settled.lv1",
                    "While you wait, particles in {vessel} sink toward the bottom.",
                    &[("vessel", &vessel.to_string())],
                ),
                2 => format!(
                    "{vessel}: {:.0}% of the suspended particles settle in {seconds:.0} s",
                    strongest * 100.0
                ),
                _ => {
                    let detail = separations
                        .iter()
                        .map(|separation| {
                            format!(
                                "{} {:.3}% ({:.6} m)",
                                separation.species,
                                separation.separated_fraction * 100.0,
                                separation.distance_m
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("; ");
                    locale.fill(
                        "event.gravity-settled.lv3",
                        "{vessel}: gravity settling for {seconds} s (Stokes, 1 g, 0.04 m path): {detail}",
                        &[("vessel", &vessel.to_string()), ("seconds", &locale.number(format!("{seconds:.3}"))), ("detail", &detail.to_string())],
                    )
                }
            }
        }
        Event::Filtered { from, to } => match register.level() {
            1 => locale.fill(
                "event.filtered.lv1",
                "You pour {from} through the filter paper — the liquid runs into {to}, and the solid stays behind on the paper.",
                &[("from", &from.to_string()), ("to", &to.to_string())],
            ),
            _ => locale.fill(
                "event.filtered.lv3",
                "{from} → {to}: filtrate passed; residue retained",
                &[("from", &from.to_string()), ("to", &to.to_string())],
            ),
        },
        Event::MagnetSeparated { from, to, attracted, remained } => {
            let name = |s: &SpeciesId| species::lookup(s).map(|d| d.name).unwrap_or(s.0.as_str()).to_string();
            if attracted.is_empty() {
                match register.level() {
                    1 => locale.fill(
                        "event.magnet-separated.lv1",
                        "You hold a magnet over {from} — nothing jumps to it.",
                        &[("from", &from.to_string())],
                    ),
                    _ => locale.fill(
                        "event.magnet-separated.lv3",
                        "{from}: no magnetic species present",
                        &[("from", &from.to_string())],
                    ),
                }
            } else {
                let att: Vec<String> = attracted.iter().map(name).collect();
                let rem: Vec<String> = remained.iter().map(name).collect();
                match register.level() {
                    1 => {
                        // Singular and plural are separate KEYS, not one
                        // sentence with a letter appended. English adds an
                        // "s" to the verb; German changes the word
                        // (springt/springen), and other languages do other
                        // things again. A skeleton that fits English cannot
                        // be made to fit them by substitution.
                        let rem_part = if rem.is_empty() {
                            String::new()
                        } else if rem.len() == 1 {
                            locale.fill(
                                "event.magnet-separated.lv1-stays-one",
                                " The {what} stays behind.",
                                &[("what", &rem.join(", "))],
                            )
                        } else {
                            locale.fill(
                                "event.magnet-separated.lv1-stays-many",
                                " The {what} stay behind.",
                                &[("what", &rem.join(", "))],
                            )
                        };
                        let main = if att.len() == 1 {
                            locale.fill(
                                "event.magnet-separated.lv1-one",
                                "You hold a magnet over {from} — the {what} jumps to it. You drop it into {to}.",
                                &[
                                    ("from", &from.to_string()),
                                    ("what", &att.join(", ")),
                                    ("to", &to.to_string()),
                                ],
                            )
                        } else {
                            locale.fill(
                                "event.magnet-separated.lv1-many",
                                "You hold a magnet over {from} — the {what} jump to it. You drop them into {to}.",
                                &[
                                    ("from", &from.to_string()),
                                    ("what", &att.join(", ")),
                                    ("to", &to.to_string()),
                                ],
                            )
                        };
                        format!("{main}{rem_part}")
                    }
                    _ => format!(
                        "{from} → {to}: magnetic {} attracted; non-magnetic {} remained",
                        att.join(", "),
                        if rem.is_empty() { "none".to_string() } else { rem.join(", ") },
                    ),
                }
            }
        }
        Event::Partitioned { vessel, species, fraction_lower } => match register.level() {
            1 => format!("Some of the {} in {vessel} moves into each layer.",
                species::lookup(species).map(|d| d.name).unwrap_or(species.0.as_str())),
            2 => format!(
                "{vessel}: {} split between the layers — {:.0}% in the lower, the rest dissolved in the upper",
                species::lookup(species).map(|d| d.name).unwrap_or(species.0.as_str()),
                fraction_lower * 100.0,
            ),
            _ => format!(
                "{vessel}: {} partitioned at K from UNIFAC γ∞ ratio; fraction in lower layer {:.4} (equal-activity split over the layer sizes)",
                species.0, fraction_lower,
            ),
        },
        Event::DissolvedInSolvent { vessel, species, solvent, dissolved, undissolved } => {
            let name = species::lookup(species).map(|d| d.name).unwrap_or(species.0.as_str());
            let solv = species::lookup(solvent).map(|d| d.name).unwrap_or(solvent.0.as_str());
            let name = locale.lookup(&format!("species.{name}")).unwrap_or(name);
            let solv = locale.lookup(&format!("species.{solv}")).unwrap_or(solv);
            match register.level() {
                1 => {
                    // Three separate keys, not one template with a
                    // placeholder for the outcome: these are the whole
                    // teaching point of solubility, and German says them
                    // differently enough that a single sentence with a
                    // swapped word would read as machine output.
                    if dissolved.0 <= 0.0 {
                        locale.fill(
                            "event.dissolved-in-solvent.lv1-none",
                            "The {what} just sits at the bottom of the {solvent} — it will not dissolve.",
                            &[("what", name), ("solvent", solv)],
                        )
                    } else if undissolved.0 <= 0.0 {
                        locale.fill(
                            "event.dissolved-in-solvent.lv1-all",
                            "The {what} disappears into the {solvent}.",
                            &[("what", name), ("solvent", solv)],
                        )
                    } else {
                        locale.fill(
                            "event.dissolved-in-solvent.lv1-some",
                            "A little of the {what} dissolves in the {solvent}; the rest sits on the bottom.",
                            &[("what", name), ("solvent", solv)],
                        )
                    }
                }
                2 => locale.fill(
                    "event.dissolved-in-solvent.lv2",
                    "{vessel}: {what} in {solvent} — {dissolved} mol dissolved (handbook limit), {solid} mol left as solid",
                    &[
                        ("vessel", &vessel.to_string()),
                        ("what", name),
                        ("solvent", solv),
                        ("dissolved", &locale.number(format!("{:.4}", dissolved.0))),
                        ("solid", &locale.number(format!("{:.4}", undissolved.0))),
                    ],
                ),
                _ => format!(
                    "{vessel}: {} in {}: dissolved {:.6} mol to the curated solubility limit, {:.6} mol solid remains. Model boundary: undissociated solute, no speciation or activity model in an organic phase",
                    species.0, solvent.0, dissolved.0, undissolved.0
                ),
            }
        }
        Event::InertInSolvent { vessel, species, solvent, why } => {
            let name = species::lookup(species).map(|d| d.name).unwrap_or(species.0.as_str());
            let solv = species::lookup(solvent).map(|d| d.name).unwrap_or(solvent.0.as_str());
            match register.level() {
                1 => locale.fill(
                    "event.inert-in-solvent.lv1",
                    "The {name} just sits in the {solv} — nothing happens to it.",
                    &[("name", name), ("solv", solv)],
                ),
                2 => locale.fill(
                    "event.inert-in-solvent.lv2",
                    "{vessel}: {name} does not react with {solv} — computed no-reaction, not a gap",
                    &[("vessel", &vessel.to_string()), ("name", name), ("solv", solv)],
                ),
                _ => format!("{vessel}: {} inert in {}: {why}", species.0, solvent.0),
            }
        }
        Event::OrgReacted { vessel, name, equation, extent, boundary } => match register.level() {
            1 => locale.fill(
                "event.org-reacted.lv1",
                "Something new forms in {vessel} — the {name} reaction turns the mixture into different substances.",
                &[("vessel", &vessel.to_string()), ("name", &name.to_string())],
            ),
            2 => format!(
                "{vessel}: {name} ran — {equation} ({:.3} mol reacted)",
                extent.0
            ),
            _ => format!(
                "{vessel}: {name}, {equation}, extent {:.6} mol. Boundary: {boundary}",
                extent.0
            ),
        },
        Event::Smelled { vessel, notes } => {
            if notes.is_empty() {
                match register.level() {
                    1 => locale.fill(
                        "event.smelled.lv1",
                        "You waft the air from {vessel} toward your nose — nothing you can pick out.",
                        &[("vessel", &vessel.to_string())],
                    ),
                    2 => locale.fill(
                        "event.smelled.lv2",
                        "{vessel}: no odour a careful waft detects",
                        &[("vessel", &vessel.to_string())],
                    ),
                    _ => locale.fill(
                        "event.smelled.lv3",
                        "{vessel}: no curated odour among the volatile species — and 'odourless' is itself data: CO2 and CO teach why a nose is not a gas detector",
                        &[("vessel", &vessel.to_string())],
                    ),
                }
            } else {
                let list: Vec<String> = notes
                    .iter()
                    .map(|(sp, d)| {
                        let name = species::lookup(sp).map(|x| x.name).unwrap_or(sp.0.as_str());
                        locale.fill(
                            "event.smelled.lv3",
                            "{name}: {d}",
                            &[("name", name), ("d", d)],
                        )
                    })
                    .collect();
                match register.level() {
                    1 => format!("You waft the air from {vessel} toward your nose — {}.", list.join("; ")),
                    2 => format!("{vessel}: wafted — {}", list.join("; ")),
                    _ => format!("{vessel}: waft (taught technique — never a direct huff): {}. Odour words are editorial curation in the qualitative-analysis register", list.join("; ")),
                }
            }
        }
        Event::GasTested { vessel, test, positive, notes } => match register.level() {
            1 => {
                if *positive {
                    locale.fill(
                        "event.gas-tested.lv1-positive",
                        "The {test} on {vessel} is positive!",
                        &[("test", &test.to_string()), ("vessel", &vessel.to_string())],
                    )
                } else {
                    locale.fill(
                        "event.gas-tested.lv1-shows-nothing",
                        "The {test} on {vessel} shows nothing.",
                        &[("test", &test.to_string()), ("vessel", &vessel.to_string())],
                    )
                }
            }
            2 => format!("{vessel}: {test} — {}", if *positive { "positive" } else { "negative" }),
            _ => format!("{vessel}: {test}: {notes}"),
        },
        Event::Burst { vessel, at_pa, rating_pa } => match register.level() {
            1 => locale.fill(
                "event.burst.lv1",
                "BANG — the sealed {vessel} could not hold the pressure and let go!",
                &[("vessel", &vessel.to_string())],
            ),
            2 => format!(
                "{vessel}: BURST at {:.0} kPa (glass rating ~{:.0} kPa) — seal gone, gases vented",
                at_pa / 1000.0, rating_pa / 1000.0
            ),
            _ => format!(
                "{vessel}: sealed headspace exceeded the teaching burst constant ({:.3e} Pa > {:.3e} Pa); the seal failed, the headspace is open, every gas vented as events, and the ledger is exact through the failure. The constant is editorial — the model's claim is that sealed vessels HAVE limits, not a certification of any flask",
                at_pa, rating_pa
            ),
        },
        Event::HeatOfMixing { vessel, joules } => match register.level() {
            1 => {
                if *joules > 0.0 {
                    locale.fill(
                        "event.heat-of-mixing.lv1-liquids-mingle-glass-1",
                        "As the liquids mingle in {vessel}, the glass grows a little warm.",
                        &[("vessel", &vessel.to_string())],
                    )
                } else {
                    locale.fill(
                        "event.heat-of-mixing.lv1-liquids-mingle-glass-2",
                        "As the liquids mingle in {vessel}, the glass grows a little cool.",
                        &[("vessel", &vessel.to_string())],
                    )
                }
            }
            2 => {
                // "released" and "absorbed" were English words chosen
                // by a condition and dropped into an English sentence.
                // Two keys instead, so German writes both in full.
                let key = if *joules > 0.0 {
                    "event.heat-of-mixing.lv2-released"
                } else {
                    "event.heat-of-mixing.lv2-absorbed"
                };
                let en = if *joules > 0.0 {
                    "{vessel}: heat of mixing released {joules} J"
                } else {
                    "{vessel}: heat of mixing absorbed {joules} J"
                };
                locale.fill(
                    key,
                    en,
                    &[
                        ("vessel", &vessel.to_string()),
                        ("joules", &locale.number(format!("{:.1}", joules.abs()))),
                    ],
                )
            }
            _ => locale.fill(
                "event.heat-of-mixing.lv3",
                "{vessel}: q_mix = {joules} J from ΔHᴱ (UNIFAC Gibbs–Helmholtz, verified-pair allowlist; state-function bookkeeping, so the pour path cannot change the answer). Boundary: VLE-fitted parameters make hᴱ magnitude-class, and unverified pairs are withheld, not guessed",
                &[("vessel", &vessel.to_string()), ("joules", &locale.number(format!("{joules:+.3}")))],
            ),
        },
        Event::NuclideSpiked { vessel, nuclide, moles, activity_bq } => match register.level() {
            1 => locale.fill(
                "event.nuclide-spiked.lv1",
                "A tiny radioactive sample of {nuclide} goes into {vessel} — the counter near it starts clicking.",
                &[("nuclide", &nuclide.to_string()), ("vessel", &vessel.to_string())],
            ),
            2 => format!(
                "{vessel}: spiked with {:.3e} mol {nuclide} — initial activity {:.3e} Bq",
                moles.0, activity_bq
            ),
            _ => format!(
                "{vessel}: {nuclide} tracer, {:.6e} mol, A₀ = {:.6e} Bq (λN, NUBASE2020 half-life). Boundary: tracer-scale, chemically inert in this model; ionising-radiation practice is real-world, not simulated",
                moles.0, activity_bq
            ),
        },
        Event::Decayed { vessel, parent, daughter, mode, moles, half_life_s, equation } => match register.level() {
            1 => locale.fill(
                "event.decayed.lv1",
                "Inside {vessel}, some of the {parent} quietly turned into {daughter} while you waited.",
                &[("vessel", &vessel.to_string()), ("parent", &parent.to_string()), ("daughter", &daughter.to_string())],
            ),
            2 => format!(
                "{vessel}: {equation} — {:.3e} mol decayed ({mode}, t½ = {:.3e} s)",
                moles.0, half_life_s
            ),
            _ => format!(
                "{vessel}: {equation}; {:.6e} mol transmuted. Elements do NOT conserve across this event — nucleons do (α parcels keep their He-4 in the ledger), charge bookkeeping notes the departing β/ν, and the mass defect is a stated model boundary",
                moles.0
            ),
        },
        Event::Chromatographed { vessel, plates, void_time_s, peaks, outside_method } => match register.level() {
            1 => {
                let order = peaks
                    .iter()
                    .map(|p| species::lookup(&p.species).map(|d| d.name).unwrap_or(p.species.0.as_str()))
                    .collect::<Vec<_>>()
                    .join(", then ");
                locale.fill(
                    "event.chromatographed.lv1-mixture-from-runs",
                    "The mixture from {vessel} runs through the column and comes out one thing at a time: {order}.",
                    &[("vessel", &vessel.to_string()), ("order", &order.to_string())],
                )
            }
            2 => {
                let table = peaks
                    .iter()
                    .map(|p| {
                        format!(
                            "{} at {:.0} s ({:.0}% area)",
                            species::lookup(&p.species).map(|d| d.name).unwrap_or(p.species.0.as_str()),
                            p.retention_time_s,
                            p.relative_area * 100.0,
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                let unseen = if outside_method.is_empty() {
                    String::new()
                } else {
                    format!(
                        " — the dissolved ions ({}) pass with the water and are not separated",
                        outside_method.iter().map(|s| s.0.as_str()).collect::<Vec<_>>().join(", ")
                    )
                };
                locale.fill(
                    "event.chromatographed.lv1-chromatogram",
                    "{vessel}: chromatogram — {table}{unseen}",
                    &[("vessel", &vessel.to_string()), ("table", &table.to_string()), ("unseen", &unseen.to_string())],
                )
            }
            _ => {
                let table = peaks
                    .iter()
                    .map(|p| {
                        format!(
                            "{} K={:.3} tR={:.1}s w={:.1}s A={:.3}",
                            p.species.0, p.partition_k, p.retention_time_s, p.width_s, p.relative_area,
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                let unseen = if outside_method.is_empty() {
                    String::new()
                } else {
                    format!(
                        "; outside the method: {} (ion exchange not modeled)",
                        outside_method.iter().map(|s| s.0.as_str()).collect::<Vec<_>>().join(", ")
                    )
                };
                locale.fill(
                    "event.chromatographed.lv3",
                    "{vessel}: N={plates}, t0={void_time_s}s, β=0.5; K = γ∞(water)/γ∞(alkane) from the same UNIFAC the funnel partitions on; tR = t0·(1+K·β), w = 4·tR/√N — {table}{unseen}",
                    &[("vessel", &vessel.to_string()), ("plates", &plates.to_string()), ("void_time_s", &locale.number(format!("{void_time_s:.0}"))), ("table", &table.to_string()), ("unseen", &unseen.to_string())],
                )
            }
        },
        Event::Drained { from, to, solvent, moles } => match register.level() {
            1 => locale.fill(
                "event.drained.lv1",
                "You open the tap and the bottom layer runs from {from} into {to}.",
                &[("from", &from.to_string()), ("to", &to.to_string())],
            ),
            2 => format!(
                "{from} → {to}: the lower layer drained — {:.3} mol {} with everything dissolved in it; the upper layer stays behind",
                moles.0,
                species::lookup(solvent).map(|d| d.name).unwrap_or(solvent.0.as_str()),
            ),
            _ => format!(
                "{from} → {to}: lower layer ({}, {:.6} mol solvent) plus its aqueous solutes; solids left behind — a stopcock passes liquid, and a settled solid is a filtration question",
                solvent.0, moles.0,
            ),
        },
        Event::LayersFormed { vessel, upper, lower } => match register.level() {
            1 => locale.fill(
                "event.layers-formed.lv1",
                "The liquid in {vessel} separates into two layers.",
                &[("vessel", &vessel.to_string())],
            ),
            2 => format!(
                "{vessel}: two layers — {} floating on {}; mixing them would raise the Gibbs energy, so they split",
                species::lookup(upper).map(|d| d.name).unwrap_or(upper.0.as_str()),
                species::lookup(lower).map(|d| d.name).unwrap_or(lower.0.as_str()),
            ),
            _ => format!(
                "{vessel}: liquid–liquid split (UNIFAC LLE, common-tangent construction). The split and the layer order are robust; the trace mutual solubilities are upper bounds — VLE-fitted UNIFAC parameters underestimate alkane–water γ∞ — and are deliberately not reported",
            ),
        },
        Event::Evaporated { vessel, moles } => match register.level() {
            1 => locale.fill(
                "event.evaporated.lv1",
                "Steam rises from {vessel} — the water is boiling away!",
                &[("vessel", &vessel.to_string())],
            ),
            2 => format!("{vessel}: {:.3} mol water evaporated", moles.0),
            _ => format!("{vessel}: {:.6} mol H2O evaporated (vaporisation enthalpy not yet in the balance)", moles.0),
        },
        Event::Distilled { from, to, water, ethanol, at, ended, stages, energy_kj, azeotropic } => match register.level() {
            1 => locale.fill(
                "event.distilled.lv1",
                "Vapour rises from {from}, cools in the tube, and drips into {to}.",
                &[("from", &from.to_string()), ("to", &to.to_string())],
            ),
            2 => {
                let t0 = at.to_celsius();
                let t1 = ended.to_celsius();
                let column = if *stages > 1 {
                    locale.fill(
                        "event.distilled.lv2",
                        " through a {stages}-stage column",
                        &[("stages", &stages.to_string())],
                    )
                } else {
                    String::new()
                };
                if *azeotropic {
                    format!(
                        "{from} → {to}: {:.3} mol water + {:.3} mol ethanol over{column} — the vapour matches the liquid now (azeotrope), so more stages or harder boiling enrich nothing",
                        water.0, ethanol.0
                    )
                } else if (t1 - t0).abs() > 0.05 {
                    format!(
                        "{from} → {to}: {:.3} mol water + {:.3} mol ethanol over{column}; the pot boiled at {t0:.1} °C and climbed to {t1:.1} °C as the light component left",
                        water.0, ethanol.0
                    )
                } else {
                    format!(
                        "{from} → {to}: {:.3} mol water + {:.3} mol ethanol over{column}, boiling at {t0:.1} °C",
                        water.0, ethanol.0
                    )
                }
            }
            _ => format!(
                "{from} → {to}: Rayleigh batch cut, {stages} ideal stage(s) at total reflux (a real column at finite reflux separates less, never more); pot {:.2} K → {:.2} K; latent heat {energy_kj:.2} kJ paid by the burner and dumped by the condenser, deliberately outside the vessel ledger{}",
                at.0,
                ended.0,
                if *azeotropic { "; azeotrope reached: y = x" } else { "" }
            ),
        },
        Event::Transferred { from, to, fraction } => match register.level() {
            1 => locale.fill(
                "event.transferred.lv1",
                "You pour some of {from} into {to}.",
                &[("from", &from.to_string()), ("to", &to.to_string())],
            ),
            _ => format!("{from} → {to}: {:.0}% of the liquid", fraction * 100.0),
        },
        Event::Mixed {
            a,
            b,
            into,
            fraction_a,
            fraction_b,
            temperature_into,
            ..
        } => match register.level() {
            1 => format!(
                "You mix some of {a} and {b} together in {into}. It settles at {:.0} °C.",
                temperature_into.to_celsius()
            ),
            _ => format!(
                "{a} ({:.0}%) + {b} ({:.0}%) → {into} at {:.2} °C",
                fraction_a * 100.0,
                fraction_b * 100.0,
                temperature_into.to_celsius(),
            ),
        },
        Event::Dissolved {
            vessel,
            species: sid,
            moles,
        } => {
            let name = species::lookup(sid).map(|d| d.name).unwrap_or(sid.0.as_str());
            match register.level() {
                1 => locale.fill(
                    "event.dissolved.lv1",
                    "The {name} disappears into the water in {vessel}!",
                    &[("name", name), ("vessel", &vessel.to_string())],
                ),
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
                    locale.fill(
                        "event.precipitated.lv1",
                        "It went cloudy in {vessel}! A {colour} solid appears at the bottom — that's called a precipitate.",
                        &[("vessel", &vessel.to_string()), ("colour", colour)],
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
                    locale.fill(
                        "event.plated.lv1",
                        "A {colour} coating of {name} grows on the {host} in {vessel} — the {name} came out of the water onto it.",
                        &[("colour", colour), ("name", name), ("host", host), ("vessel", &vessel.to_string())],
                    )
                }
                2 => format!("{vessel}: {:.4} mol {name} plated out onto {host}", moles.0),
                _ => format!("{vessel}: {:.6} mol {name} plated out onto {host}", moles.0),
            }
        }
        Event::Inert { vessel, species: sid, why } => {
            let name = species::lookup(sid).map(|d| d.name).unwrap_or(sid.0.as_str());
            match register.level() {
                1 => locale.fill(
                    "event.inert.lv1",
                    "Nothing happens to the {name} in {vessel} — and that is the real answer, not a gap: it is too unreactive for this.",
                    &[("name", name), ("vessel", &vessel.to_string())],
                ),
                2 => locale.fill(
                    "event.inert.lv2",
                    "{vessel}: {name} does not react — {why}",
                    &[("vessel", &vessel.to_string()), ("name", name), ("why", why)],
                ),
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
                        locale.fill(
                            "event.consumed.lv1",
                            "The {name} in {vessel} is used up.",
                            &[("name", name), ("vessel", &vessel.to_string())],
                        )
                    }
                    Some(_) => locale.fill(
                        "event.consumed.lv1-some-used",
                        "Some of the {name} in {vessel} is used up.",
                        &[("name", name), ("vessel", &vessel.to_string())],
                    ),
                    None => locale.fill(
                        "event.consumed.lv1-being-used",
                        "The {name} in {vessel} is being used up.",
                        &[("name", name), ("vessel", &vessel.to_string())],
                    ),
                },
                2 => format!("{vessel}: {:.4} mol {name} consumed", moles.0),
                _ => format!("{vessel}: −{:.6} mol {name}", moles.0),
            }
        }
        Event::Ignited {
            vessel,
            flame,
            energy_j,
        } => match register.level() {
            1 => match flame {
                Some(colour) => {
                    locale.fill(
                        "event.ignited.lv1-catches-fire-burning",
                        "It catches fire in {vessel} — burning with {colour} light!",
                        &[("vessel", &vessel.to_string()), ("colour", &colour.to_string())],
                    )
                }
                None => locale.fill(
                    "event.ignited.lv1",
                    "It catches fire in {vessel}!",
                    &[("vessel", &vessel.to_string())],
                ),
            },
            2 => {
                let colour = flame
                    .as_ref()
                    .map(|colour| format!(" — {colour} flame"))
                    .unwrap_or_default();
                let energy = energy_j
                    .map(|joules| format!(" · {:.2} kJ released", joules / 1000.0))
                    .unwrap_or_default();
                locale.fill(
                    "event.ignited.lv1-ignited",
                    "{vessel}: ignited{colour}{energy}",
                    &[("vessel", &vessel.to_string()), ("colour", &colour.to_string()), ("energy", &energy.to_string())],
                )
            }
            _ => match energy_j {
                Some(joules) => format!(
                    "{vessel}: ignition source applied; computed reaction energy = {:.3} J",
                    joules
                ),
                None => locale.fill(
                    "event.ignited.lv3",
                    "{vessel}: ignition source applied; reaction energy unavailable",
                    &[("vessel", &vessel.to_string())],
                ),
            },
        },
        Event::FlameTest {
            vessel,
            species: sid,
            colour,
        } => {
            let name = species::lookup(sid).map(|d| d.name).unwrap_or(sid.0.as_str());
            match register.level() {
                1 => locale.fill(
                    "event.flame-test.lv1",
                    "It does not catch fire — but look: it turns the flame {colour}! Every metal has its own colour, which is how you can tell them apart.",
                    &[("colour", &colour.to_string())],
                ),
                2 => {
                    locale.fill(
                        "event.flame-test.lv2",
                        "{vessel}: flame test — {name} colours the flame {colour}",
                        &[("vessel", &vessel.to_string()), ("name", name), ("colour", colour)],
                    )
                }
                _ => locale.fill(
                    "event.flame-test.lv3",
                    "{vessel}: no combustion; characteristic emission of {name} ({colour})",
                    &[("vessel", &vessel.to_string()), ("name", name), ("colour", &colour.to_string())],
                ),
            }
        }
        Event::DidNotIgnite { vessel } => match register.level() {
            1 => {
                locale.fill(
                    "event.did-not-ignite.lv1",
                    "You hold the flame to {vessel} — and nothing happens. Not everything burns.",
                    &[("vessel", &vessel.to_string())],
                )
            }
            _ => locale.fill(
                "event.did-not-ignite.lv3",
                "{vessel}: nothing ignited",
                &[("vessel", &vessel.to_string())],
            ),
        },
        Event::ThermalEquilibrium {
            vessel,
            temperature,
            reaction_energy_j: _,
            provenance,
        } => match register.level() {
            1 => locale.fill(
                "event.thermal-equilibrium.lv1",
                "Everything in {vessel} settles into what it wants to be at this heat.",
                &[("vessel", &vessel.to_string())],
            ),
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
                    locale.fill(
                        "event.solution-characterized.lv1-liquid-acid",
                        "The liquid in {vessel} is an acid.",
                        &[("vessel", &vessel.to_string())],
                    )
                } else if *ph > 8.0 {
                    locale.fill(
                        "event.solution-characterized.lv1-liquid-base-opposite",
                        "The liquid in {vessel} is a base (the opposite of an acid).",
                        &[("vessel", &vessel.to_string())],
                    )
                } else {
                    locale.fill(
                        "event.solution-characterized.lv1-liquid-neutral-like",
                        "The liquid in {vessel} is neutral — like pure water.",
                        &[("vessel", &vessel.to_string())],
                    )
                }
            }
            2 => locale.fill(
                "event.solution-characterized.lv2",
                "{vessel}: pH {ph}",
                &[("vessel", &vessel.to_string()), ("ph", &locale.number(format!("{ph:.2}")))],
            ),
            _ => {
                locale.fill(
                    "event.solution-characterized.lv1-mol-kgw",
                    "{vessel}: pH {ph} · I = {ionic_strength} mol/kgw",
                    &[("vessel", &vessel.to_string()), ("ph", &locale.number(format!("{ph:.3}"))), ("ionic_strength", &locale.number(format!("{ionic_strength:.4}")))],
                )
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
                Instrument::Calorimeter => "calorimeter",
                // The column never emits a scalar Measured — it reports a
                // peak table via Chromatographed — but the name must exist.
                Instrument::Chromatograph => "chromatograph",
                Instrument::GeigerCounter => "Geiger counter",
            };
            match register.level() {
                1 => locale.fill(
                    "event.measured.lv1",
                    "The {device} on {vessel} reads {value} {unit}.",
                    &[("device", device), ("vessel", &vessel.to_string()), ("value", &locale.number(format!("{value:.0}"))), ("unit", unit)],
                ),
                2 => locale.fill(
                    "event.measured.lv2",
                    "{vessel} {device}: {value} {unit}",
                    &[("vessel", &vessel.to_string()), ("device", device), ("value", &locale.number(format!("{value:.2}"))), ("unit", unit)],
                ),
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
                1 => locale.fill(
                    "event.electrolysed.lv1",
                    "{grams} g of {name} builds up on the electrode in {vessel}.",
                    &[("grams", &locale.number(format!("{grams:.2}"))), ("name", name), ("vessel", &vessel.to_string())],
                ),
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
            1 => locale.fill(
                "event.cell-voltage.lv1",
                "The voltmeter between {anode} and {cathode} reads {volts} V — you have made a battery! The electrons want to flow from {anode} to {cathode}. (Nothing is using the current yet, so nothing in the beakers changes.)",
                &[("anode", &anode.to_string()), ("cathode", &cathode.to_string()), ("volts", &locale.number(format!("{volts:.2}")))],
            ),
            2 => locale.fill(
                "event.cell-voltage.lv2",
                "{notation}: E = {volts} V open-circuit (E° = {standard_volts} V); electrons would flow {anode} → {cathode}; closing the circuit would run {equation}. No current is drawn, so this is the voltage the cell *offers*, not what it delivers under load",
                &[("notation", &notation.to_string()), ("volts", &locale.number(format!("{volts:.3}"))), ("standard_volts", &locale.number(format!("{standard_volts:.3}"))), ("anode", &anode.to_string()), ("cathode", &cathode.to_string()), ("equation", &equation.to_string())],
            ),
            _ => locale.fill(
                "event.cell-voltage.lv3",
                "{notation}: E_cell = {volts} V open-circuit, no current, no internal resistance modelled (E°_cell = {standard_volts} V; the difference is the Nernst term over the computed ion activities; ideal salt bridge, no liquid-junction potential); anode {anode}, cathode {cathode}; {equation}",
                &[("notation", &notation.to_string()), ("volts", &locale.number(format!("{volts:.4}"))), ("standard_volts", &locale.number(format!("{standard_volts:.4}"))), ("anode", &anode.to_string()), ("cathode", &cathode.to_string()), ("equation", &equation.to_string())],
            ),
        },
        Event::NoCell { a, b, why } => match register.level() {
            1 => locale.fill(
                "event.no-cell.lv1",
                "The voltmeter between {a} and {b} reads nothing — one of them isn't a proper half-cell yet.",
                &[("a", &a.to_string()), ("b", &b.to_string())],
            ),
            2 => locale.fill(
                "event.no-cell.lv2",
                "{a}–{b}: no cell — {why}",
                &[("a", &a.to_string()), ("b", &b.to_string()), ("why", &why.to_string())],
            ),
            _ => format!("{a}–{b}: no cell: {why}"),
        },
        Event::HazardWarning {
            severity,
            hazard,
            real_world,
        } => match register.level() {
            1 => locale.fill(
                "event.hazard-warning.lv1",
                "⚠️  STOP AND READ: {hazard}. {real_world} NEVER try this outside the virtual lab — here, we can watch what happens safely.",
                &[("hazard", &hazard.to_string()), ("real_world", &real_world.to_string())],
            ),
            2 => locale.fill(
                "event.hazard-warning.lv2",
                "⚠ HAZARD ({severity}): {hazard} — {real_world} Safe only because this lab is virtual.",
                &[("severity", &locale.number(format!("{severity:?}"))), ("hazard", &hazard.to_string()), ("real_world", &real_world.to_string())],
            ),
            _ => format!("HAZARD [{severity:?}] (L0): {hazard}; {real_world}"),
        },
        Event::SafetyVeto { reason } => match register.level() {
            1 => locale.fill(
                "event.safety-veto.lv1",
                "The lab won't do that: {reason}",
                &[("reason", &reason.to_string())],
            ),
            _ => locale.fill(
                "event.safety-veto.lv3",
                "SAFETY VETO (L0): {reason}",
                &[("reason", &reason.to_string())],
            ),
        },
        Event::ReactionOccurred { vessel, equation } => match register.level() {
            1 => locale.fill(
                "event.reaction-occurred.lv1",
                "The mixture in {vessel} changes — something new is forming!",
                &[("vessel", &vessel.to_string())],
            ),
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
                        locale.fill(
                            "event.gas-evolved.lv1",
                            "A gas rises out of {vessel} — this one is poisonous. In a real room you would have to leave NOW.",
                            &[("vessel", &vessel.to_string())],
                        )
                    } else {
                        locale.fill(
                            "event.gas-evolved.lv1",
                            "Bubbles! A gas rises out of {vessel}.",
                            &[("vessel", &vessel.to_string())],
                        )
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
                1 => locale.fill(
                    "event.gas-absorbed.lv1",
                    "Gas bubbles into {vessel} and is taken up by the liquid.",
                    &[("vessel", &vessel.to_string())],
                ),
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
                1 => locale.fill(
                    "event.gas-contained.lv1",
                    "Bubbles form in {vessel}, but the gas stays inside.",
                    &[("vessel", &vessel.to_string())],
                ),
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
            1 => locale.fill(
                "event.vessel-sealed.lv1",
                "A lid seals {vessel}. Nothing gaseous can escape now.",
                &[("vessel", &vessel.to_string())],
            ),
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
            1 => locale.fill(
                "event.vessel-pressure-controlled.lv1",
                "A movable piston holds {vessel} at constant pressure.",
                &[("vessel", &vessel.to_string())],
            ),
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
            1 => locale.fill(
                "event.vessel-swept.lv1",
                "Nitrogen flows across {vessel} and carries gases away.",
                &[("vessel", &vessel.to_string())],
            ),
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
            1 => locale.fill(
                "event.vessel-opened.lv1",
                "The lid comes off {vessel}; its gas can escape into the room.",
                &[("vessel", &vessel.to_string())],
            ),
            _ => locale.fill(
                "event.vessel-opened.lv3",
                "{vessel}: sealed boundary opened to the atmospheric reservoir",
                &[("vessel", &vessel.to_string())],
            ),
        },
        Event::HeadspaceEquilibrated {
            vessel,
            pressure,
            total_moles,
        } => match register.level() {
            1 => locale.fill(
                "event.headspace-equilibrated.lv1",
                "The gas under the lid of {vessel} settles with the liquid.",
                &[("vessel", &vessel.to_string())],
            ),
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
                        locale.fill(
                            "event.state-changed.lv1-turned-ice",
                            "The {name} in {vessel} turned to ice!",
                            &[("name", name), ("vessel", &vessel.to_string())],
                        )
                    }
                    (Phase::Solid, Phase::Liquid) => {
                        locale.fill(
                            "event.state-changed.lv1-ice-melted-back",
                            "The ice in {vessel} melted back into {name}.",
                            &[("vessel", &vessel.to_string()), ("name", name)],
                        )
                    }
                    (Phase::Liquid, Phase::Gas) => {
                        locale.fill(
                            "event.state-changed.lv1-boiling-look-steam",
                            "The {name} in {vessel} is boiling — look at the steam!",
                            &[("name", name), ("vessel", &vessel.to_string())],
                        )
                    }
                    _ => format!("The {name} in {vessel} {verb}."),
                },
                2 => {
                    if shifted_by.abs() < 0.05 {
                        locale.fill(
                            "event.state-changed.lv1-x",
                            "{vessel}: {name} {verb} at {c} °C",
                            &[("vessel", &vessel.to_string()), ("name", name), ("verb", verb), ("c", &locale.number(format!("{c:.1}")))],
                        )
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
                Some(c) => locale.fill(
                    "event.reacted.lv1-making-happen-much",
                    "In {vessel}, the {c} is making it happen much faster — after {seconds} seconds a lot has changed!",
                    &[("vessel", &vessel.to_string()), ("c", &c.to_string()), ("seconds", &locale.number(format!("{seconds:.0}")))],
                ),
                None => locale.fill(
                    "event.reacted.lv1-after-seconds-something",
                    "After {seconds} seconds, something has been happening in {vessel}.",
                    &[("seconds", &locale.number(format!("{seconds:.0}"))), ("vessel", &vessel.to_string())],
                ),
            },
            2 => {
                let with = match catalyst {
                    Some(c) => locale.fill(
                        "event.reacted.lv2",
                        ", sped up by {c}",
                        &[("c", &c.to_string())],
                    ),
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
            1 => locale.fill(
                "event.not-yet-modeled.lv1",
                "Hmm — nothing visible happens in {vessel} (this part of the lab isn't awake yet).",
                &[("vessel", &vessel.to_string())],
            ),
            2 => locale.fill(
                "event.not-yet-modeled.lv2",
                "{vessel}: not yet modelled — {what}",
                &[("vessel", &vessel.to_string()), ("what", &what.to_string())],
            ),
            _ => locale.fill(
                "event.not-yet-modeled.lv3",
                "{vessel}: NOT MODELLED: {what}",
                &[("vessel", &vessel.to_string()), ("what", &what.to_string())],
            ),
        },
        Event::SolverFailed {
            vessel,
            solver,
            detail,
        } => match register.level() {
            1 => locale.fill(
                "event.solver-failed.lv1",
                "The lab couldn't work out what happens in {vessel}. That's honest — better than guessing!",
                &[("vessel", &vessel.to_string())],
            ),
            _ => format!("{vessel}: solver '{solver}' failed: {detail}"),
        },
        Event::Diluted {
            vessel,
            volume,
            moles,
        } => match register.level() {
            1 => locale.fill(
                "event.diluted.lv1",
                "You add water to {vessel} — the solution gets weaker.",
                &[("vessel", &vessel.to_string())],
            ),
            2 => format!(
                "{vessel}: diluted with {:.1} mL water ({:.4} mol)",
                volume.0 * 1000.0,
                moles.0
            ),
            _ => format!(
                "{vessel}: +{:.6} mol H₂O from {:.4} L dilution water",
                moles.0, volume.0
            ),
        },
        Event::Titrated {
            vessel,
            titrant,
            concentration,
            steps,
            total_volume,
            final_ph,
            ..
        } => {
            let name = species::lookup(titrant)
                .map(|d| d.name)
                .unwrap_or(titrant.0.as_str());
            match register.level() {
                1 => locale.fill(
                    "event.titrated.lv1",
                    "You titrate {vessel} with {name} — after {steps} additions the pH reaches {final_ph}.",
                    &[("vessel", &vessel.to_string()), ("name", name), ("steps", &steps.to_string()), ("final_ph", &locale.number(format!("{final_ph:.1}")))],
                ),
                2 => format!(
                    "{vessel}: titrated with {concentration} mol/L {name}; {steps} steps, {:.1} mL total, final pH {final_ph:.2}",
                    total_volume.0 * 1000.0
                ),
                _ => format!(
                    "{vessel}: auto-titration with {} standard solution ({concentration} mol/L; {steps} steps, {:.3} mL cumulative = {:.5} mol delivered with its carrier water); final pH {final_ph:.3}",
                    titrant.0,
                    total_volume.0 * 1000.0,
                    concentration * total_volume.0,
                ),
            }
        }
        Event::Transported {
            chain,
            receiver,
            steps,
            courant,
            effluent_moles,
        } => {
            let cells = chain.len();
            let total: f64 = effluent_moles.iter().map(|(_, m)| m.0).sum();
            match register.level() {
                1 => locale.fill(
                    "event.transported.lv1",
                    "Solution flows through {cells} column cells and collects in {receiver}.",
                    &[("cells", &cells.to_string()), ("receiver", &receiver.to_string())],
                ),
                2 => {
                    let species_list: Vec<String> = effluent_moles
                        .iter()
                        .filter(|(_, m)| m.0 > crate::OBSERVABLE_MOLES)
                        .map(|(s, m)| {
                            let name = species::lookup(s)
                                .map(|d| d.name)
                                .unwrap_or(s.0.as_str());
                            format!("{:.4} mol {name}", m.0)
                        })
                        .collect();
                    let what = if species_list.is_empty() {
                        "solvent only".to_string()
                    } else {
                        species_list.join(", ")
                    };
                    format!(
                        "{cells} cells × {steps} steps (Cf={courant:.2}); effluent → {receiver}: {what}"
                    )
                }
                _ => locale.fill(
                    "event.transported.lv3",
                    "1-D upwind transport: {cells} cells × {steps} steps @ Cf={courant}; \
                     effluent total {total} mol → {receiver}",
                    &[("cells", &cells.to_string()), ("steps", &steps.to_string()), ("courant", &locale.number(format!("{courant:.4}"))), ("total", &locale.number(format!("{total:.6}"))), ("receiver", &receiver.to_string())],
                ),
            }
        }
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
