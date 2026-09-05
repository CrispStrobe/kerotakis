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

/// Keep ordinary classroom quantities easy to scan while never displaying a
/// real non-zero amount as zero. Scientific notation is reserved for values
/// below half a unit in the requested final decimal place.
fn quantity(value: f64, decimals: usize) -> String {
    let rounds_to_zero = value != 0.0 && value.abs() < 0.5 * 10_f64.powi(-(decimals as i32));
    if rounds_to_zero {
        format!("{value:.3e}")
    } else {
        format!("{value:.decimals$}")
    }
}

// ── The tables a coverage gate can walk ──────────────────────────────
//
// Four vocabularies below were written inline as `match` arms, which is
// the clearest way to read them at the call site and the one shape a test
// cannot enumerate. They are functions now for exactly one reason: so
// `tests/i18n_coverage.rs` can ask the engine what words it is able to
// emit, instead of a person maintaining a second list beside them.
//
// That matters more here than the tidiness would justify. Every one of
// these falls back to English per term, silently, so a word added to one
// of these tables without German renders in English inside a German
// sentence and nothing anywhere reports it. Walking the table is the only
// defence that survives someone adding an instrument in a hurry.

/// The catalogue key naming a phase — `phase.aqueous`.
///
/// `{:?}` printed "Liquid" in every language, which is how this started.
pub fn phase_key(phase: Phase) -> &'static str {
    match phase {
        Phase::Aqueous => "phase.aqueous",
        Phase::Liquid => "phase.liquid",
        Phase::Solid => "phase.solid",
        Phase::Gas => "phase.gas",
    }
}

/// The English a phase falls back to when a language has not named it.
pub fn phase_english(phase: Phase) -> &'static str {
    match phase {
        Phase::Aqueous => "Aqueous",
        Phase::Liquid => "Liquid",
        Phase::Solid => "Solid",
        Phase::Gas => "Gas",
    }
}

/// What the journal calls an instrument, in English.
///
/// This English IS the catalogue key — `instrument.pH meter` — the same
/// lookup-by-value the species and glassware tables use, so an instrument
/// nobody has translated reads in English inside a German sentence rather
/// than disappearing from it.
pub fn instrument_name(instrument: Instrument) -> &'static str {
    match instrument {
        Instrument::Thermometer => "thermometer",
        Instrument::Balance => "balance",
        Instrument::PhMeter => "pH meter",
        Instrument::Eyes => "eyes",
        Instrument::PressureGauge => "pressure gauge",
        Instrument::VolumeMeter => "volume meter",
        Instrument::ConductivityMeter => "conductivity meter",
        // Not "hydrometer": that is the instrument for a liquid, and
        // this one also answers for a dry solid, where the real apparatus
        // is a balance and a measuring cylinder. `measure v1 hydrometer`
        // still reaches it, because that is what a learner will type.
        Instrument::Densitometer => "density meter",
        Instrument::Spectrophotometer => "spectrophotometer",
        Instrument::Calorimeter => "calorimeter",
        // The column never emits a scalar Measured — it reports a peak
        // table via Chromatographed — but the name must exist.
        Instrument::Chromatograph => "chromatograph",
        Instrument::GeigerCounter => "Geiger counter",
        Instrument::MeltingPointApparatus => "melting-point apparatus",
        Instrument::BoilingPointApparatus => "boiling-point apparatus",
    }
}

/// The verb for a phase change, in English.
///
/// English builds these from a table and drops them into a translated
/// sentence, so German needs its own — gefror, schmolz, siedete — and a
/// table is where they belong. A suffix rule cannot inflect a German verb.
pub fn phase_change_verb(from: Phase, to: Phase) -> &'static str {
    match (from, to) {
        (Phase::Liquid, Phase::Solid) => "froze",
        (Phase::Solid, Phase::Liquid) => "melted",
        (Phase::Liquid, Phase::Gas) => "boiled",
        _ => "changed state",
    }
}

/// What a precipitate or a plating LOOKS like, in the reader's language.
///
/// The registry's `appearance` — "pale blue", "white crystalline powder" —
/// was dropped straight into a translated sentence, so a German reader got
/// "Ein white Feststoff erscheint am Boden". The colour is most of what
/// that sentence is FOR, so leaving it English loses the observation and
/// keeps the grammar.
///
/// German cannot inflect these as attributive adjectives — "weiß" before
/// "Feststoff" needs an ending, and "weißes kristallines Pulver" cannot
/// take one at all — so the German sentences carry the word as a free
/// apposition in parentheses and the table stays uninflected.
///
/// `None` is a species the registry does not describe; "new" is the word
/// English used for it, and it is looked up like any other.
fn appearance_word(appearance: Option<&'static str>, locale: Locale) -> &'static str {
    let english = appearance.unwrap_or("new");
    locale
        .lookup(&format!("appearance.{english}"))
        .unwrap_or(english)
}

/// What a species is called in the reader's language.
///
/// The species catalogue is the engine's, so its German belongs to the
/// engine too — but only where a language has actually named a species.
/// An unnamed one reads in English inside a German line rather than
/// falling back to its formula, which nobody asked for.
///
/// This is a FUNCTION rather than the two lines it replaces because the
/// two lines were the bug. Every arm that names a substance wrote
/// `species::lookup(id).map(|d| d.name)` — the catalogue's ENGLISH — and
/// roughly half of them then forgot the `locale.lookup` beside it. The
/// forgetting is invisible: the fallback is per term, so the English name
/// simply appears inside the German sentence and no gate anywhere
/// reports it. That is how "0,0108 mol carbon dioxide gebildet und
/// verbleibt im geschlossenen Gasraum" reached a German reader.
///
/// With one function there is nothing left to forget, and a grep for
/// `\.name` in this file is a real check rather than a hint.
fn species_name(locale: Locale, id: &SpeciesId) -> &str {
    let english = species::lookup(id).map(|d| d.name).unwrap_or(id.0.as_str());
    locale
        .lookup(&format!("species.{english}"))
        .unwrap_or(english)
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
        let name = species_name(locale, &p.species);
        // `{:?}` on the phase printed "Liquid" in every language. It is a
        // closed set of four words, so it is worth naming properly.
        let phase = locale.t(phase_key(p.phase), phase_english(p.phase));
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

/// The net ionic equation as a line for the feed (GUI-092).
///
/// `None` at lv1 on purpose: an equation is lv2's business, and a reader
/// being told "it went cloudy" is not helped by being handed a charge
/// balance in the same breath. lv3 adds the ions that stayed out of it,
/// because naming the spectators is the half of the lesson the equation
/// itself cannot show.
pub fn render_ionic(net: &crate::ionic::NetIonic, register: Register) -> Option<String> {
    render_ionic_in(net, register, Locale::EN)
}

/// `render_ionic`, in the reader's language. Only the label is translated:
/// the equation itself is chemical notation, which is the same in every
/// language and must not be "localised" into something a chemist would not
/// recognise.
pub fn render_ionic_in(
    net: &crate::ionic::NetIonic,
    register: Register,
    locale: Locale,
) -> Option<String> {
    if register.level() < 2 {
        return None;
    }
    let label = locale.t("ionic.net", "net ionic");
    let mut line = format!("{}: {label}: {}", net.vessel, net.equation);
    if register.level() >= 3 {
        if let Some(phrase) = net.spectator_phrase() {
            let spectators = locale.t("ionic.spectators", "spectator ions");
            line.push_str(&format!("  ({spectators}: {phrase})"));
        }
    }
    Some(line)
}

/// Every net ionic line a step's events earn, in order — the one call a
/// host needs beside `render_events_in`.
pub fn render_ionic_for(
    events: &[Event],
    vessels: &[Vessel],
    register: Register,
    locale: Locale,
) -> Vec<String> {
    crate::ionic::net_ionic_for(events, vessels)
        .iter()
        .filter_map(|net| render_ionic_in(net, register, locale))
        .collect()
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
        Event::SpillCreated {
            destination,
            source,
            fraction,
            ..
        } => match register.level() {
            1 => locale.fill(
                "event.spill-created.lv1",
                "Material from {source} spills onto {where}.",
                &[
                    ("source", &source.to_string()),
                    ("where", &format!("{destination:?}")),
                ],
            ),
            _ => format!(
                "{source}: {:.3}% transferred to spill {destination:?}",
                fraction * 100.0
            ),
        },
        Event::ContainerBroken {
            vessel,
            destination,
            impulse_ns,
            ..
        } => match register.level() {
            1 => locale.fill(
                "event.container-broken.lv1",
                "{vessel} breaks; its contents spill onto {where}.",
                &[
                    ("vessel", &vessel.to_string()),
                    ("where", &format!("{destination:?}")),
                ],
            ),
            _ => format!("{vessel}: container broke at {impulse_ns:.3} N·s; destination {destination:?}"),
        },
        Event::CollisionWithstood {
            vessel, impulse_ns, ..
        } => match register.level() {
            1 => locale.fill(
                "event.collision-withstood.lv1",
                "{vessel} is knocked but stays intact.",
                &[("vessel", &vessel.to_string())],
            ),
            _ => format!("{vessel}: collision withstood at {impulse_ns:.3} N·s"),
        },
        Event::SpillRecovered {
            destination,
            to,
            fraction,
        } => match register.level() {
            1 => locale.fill(
                "event.spill-recovered.lv1",
                "Material from {where} is recovered into {to}.",
                &[
                    ("where", &format!("{destination:?}")),
                    ("to", &to.to_string()),
                ],
            ),
            _ => format!(
                "spill {destination:?}: {:.3}% recovered into {to}",
                fraction * 100.0
            ),
        },
        Event::SpillHazard {
            destination,
            severity,
            hazard,
            real_world,
            ..
        } => match register.level() {
            1 => locale.fill(
                "event.spill-hazard.lv1",
                "Hazard at spill {where}: {hazard} {real_world}",
                &[
                    ("where", &format!("{destination:?}")),
                    ("hazard", hazard),
                    ("real_world", real_world),
                ],
            ),
            _ => format!("spill {destination:?}: {severity:?}: {hazard}; {real_world}"),
        },
        Event::Added {
            vessel,
            species: sid,
            moles,
            total_after,
        } => {
            let named = species_name(locale, sid);
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
                2 => locale.fill(
                    "event.material-added.lv2",
                    "{vessel}: +{total_amount} {unit} {material} ({components} known ingredients)",
                    &[("vessel", &vessel.to_string()), ("total_amount", &locale.number(format!("{total_amount:.3}"))), ("unit", unit), ("material", &material.to_string()), ("components", &format!("{}", components.len()))],
                ),
                _ => locale.fill(
                    "event.material-added.lv3",
                    "{vessel}: +{total_amount} {unit} {material}; {components} canonical components, {unresolved_amount} {unit} unresolved",
                    &[("vessel", &vessel.to_string()), ("total_amount", &locale.number(format!("{total_amount:.6}"))), ("unit", unit), ("material", &material.to_string()), ("components", &format!("{}", components.len())), ("unresolved_amount", &locale.number(format!("{unresolved_amount:.6}")))],
                ),
            }
        }
        Event::ObjectSpillBoundary { vessel, object_count } => locale.fill(
            "event.object-spill-boundary.lv1",
            "{vessel}: {count} coherent object(s) remain in the vessel; this spill model moves bulk phases only",
            &[("vessel", &vessel.to_string()), ("count", &object_count.to_string())],
        ),
        Event::OsmosisChanged { vessel, material, water_moles, mass_change_g } => locale.fill(
            "event.osmosis-changed.lv1",
            "{vessel}: {material} exchanged {water} mol water ({mass} g)",
            &[("vessel", &vessel.to_string()), ("material", material), ("water", &locale.number(format!("{water_moles:.6}"))), ("mass", &locale.number(format!("{mass_change_g:+.3}")))],
        ),
        Event::BrowningChanged { vessel, material, browned_fraction } => locale.fill(
            "event.browning-changed.lv1",
            "{vessel}: {material} surface is {percent}% browned",
            &[("vessel", &vessel.to_string()), ("material", material), ("percent", &locale.number(format!("{:.0}", browned_fraction * 100.0)))],
        ),
        Event::SoapScumFormed { vessel, aggregate_mass_g, divalent_ion_moles } => locale.fill(
            "event.soap-scum-formed.lv1",
            "{vessel}: {mass} g soap-scum aggregate formed from {moles} mol Ca/Mg",
            &[("vessel", &vessel.to_string()), ("mass", &locale.number(format!("{aggregate_mass_g:.3}"))), ("moles", &locale.number(format!("{divalent_ion_moles:.6}")))],
        ),
        Event::LemonPaperMarked { vessel, lemon_amount_g, paper_amount_g } => locale.fill(
            "event.lemon-paper-marked.lv1",
            "{vessel}: {lemon_amount_g} g lemon juice marks {paper_amount_g} g paper; the mark is still wet",
            &[("vessel", &vessel.to_string()), ("lemon_amount_g", &locale.number(format!("{lemon_amount_g:.2}"))), ("paper_amount_g", &locale.number(format!("{paper_amount_g:.2}")))],
        ),
        Event::LemonPaperDried { vessel } => locale.fill("event.lemon-paper-dried.lv1", "{vessel}: the lemon mark is dry and still faint", &[("vessel", &vessel.to_string())]),
        Event::LemonPaperBrowned { vessel, browned_fraction, temperature_k } => locale.fill(
            "event.lemon-paper-browned.lv1", "{vessel}: the dry lemon mark is {percent}% brown at {temperature_k} K",
            &[("vessel", &vessel.to_string()), ("percent", &locale.number(format!("{:.0}", browned_fraction * 100.0))), ("temperature_k", &locale.number(format!("{temperature_k:.1}")))],
        ),
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
        Event::Fermented {
            vessel,
            sucrose_moles,
            ethanol_moles,
            carbon_dioxide_moles,
            active_yeast_grams,
            seconds,
        } => match register.level() {
            1 => locale.fill(
                "event.fermented.lv1",
                "Yeast feeds on sugar in {vessel}, making alcohol and carbon-dioxide bubbles.",
                &[("vessel", &vessel.to_string())],
            ),
            _ => format!(
                "{vessel}: yeast fermented {:.6} mol sucrose in {seconds:.0} s → {:.6} mol ethanol + {:.6} mol CO2 ({active_yeast_grams:.3} g effective yeast)",
                sucrose_moles.0,
                ethanol_moles.0,
                carbon_dioxide_moles.0,
            ),
        },
        Event::EnzymeHydrolysed {
            vessel,
            family,
            material,
            substrate,
            hydrolysed_mass_g,
            converted_fraction,
            seconds,
        } => match register.level() {
            1 => locale.fill(
                "event.enzyme-hydrolysed.lv1",
                "The {family} in {vessel} is cutting up {substrate} from {material}.",
                &[
                    ("family", &format!("{family:?}").to_lowercase()),
                    ("vessel", &vessel.to_string()),
                    ("substrate", substrate),
                    ("material", material),
                ],
            ),
            2 => locale.fill(
                "event.hydrolysed.lv2",
                "{vessel}: {family} hydrolysed {mass} g of {substrate} in {seconds} s ({percent}% converted)",
                &[
                    ("vessel", &vessel.to_string()),
                    // lv1 lower-cases this; lv2 never did, and changing
                    // the English here would be a rewording rather than a
                    // translation fix.
                    ("family", &format!("{family:?}")),
                    ("mass", &locale.number(format!("{hydrolysed_mass_g:.4}"))),
                    ("substrate", substrate),
                    ("seconds", &locale.number(format!("{seconds:.0}"))),
                    ("percent", &locale.number(format!("{:.1}", converted_fraction * 100.0))),
                ],
            ),
            _ => format!(
                "{vessel}: {:?} bounded activity hydrolysed {hydrolysed_mass_g:.6} g of {substrate} in {material}; {:.3}% converted. Products remain in conserved unresolved material: no named product inventory is claimed.",
                family,
                converted_fraction * 100.0,
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
                locale.fill(
                    "event.foam-changed.lv1-overflow",
                    "Foam climbs out of {vessel} and spills over the rim!",
                    &[("vessel", &vessel.to_string())],
                )
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
        Event::SurfaceSpread {
            vessel,
            material,
            from_cleared_fraction,
            to_cleared_fraction,
            ..
        } => match register.level() {
            1 => locale.fill(
                "event.surface-spread.lv1",
                "The {material} darts away from the soap in {vessel}!",
                &[("material", material), ("vessel", &vessel.to_string())],
            ),
            _ => format!(
                "{vessel}: {material} central clearing increased from {:.0}% to {:.0}%",
                100.0 * from_cleared_fraction,
                100.0 * to_cleared_fraction
            ),
        },
        Event::SurfaceColourSpread {
            vessel,
            from_spread_fraction,
            to_spread_fraction,
            spot_count,
        } => match register.level() {
            1 => locale.fill(
                "event.surface-colour-spread.lv1",
                "The colours race and swirl across the milk in {vessel}!",
                &[("vessel", &vessel.to_string())],
            ),
            _ => format!(
                "{vessel}: {spot_count} surface colour spot(s) spread from {:.0}% to {:.0}%",
                100.0 * from_spread_fraction,
                100.0 * to_spread_fraction
            ),
        },
        Event::SurfaceColourMixed { vessel, spot_count } => match register.level() {
            1 => locale.fill(
                "event.surface-colour-mixed.lv1",
                "Stirring blends the surface colours through {vessel}.",
                &[("vessel", &vessel.to_string())],
            ),
            _ => format!("{vessel}: homogenized {spot_count} surface colour spot(s)"),
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
                // "25.0 °C → 25.0 °C" in a German journal: the numbers
                // were the only thing on the line and they were the one
                // thing not going through the catalogue.
                2 => locale.fill(
                    "event.temperature-changed.lv2",
                    "{vessel}: {from} °C → {to} °C",
                    &[
                        ("vessel", &vessel.to_string()),
                        ("from", &locale.number(format!("{:.1}", from.to_celsius()))),
                        ("to", &locale.number(format!("{:.1}", to.to_celsius()))),
                    ],
                ),
                _ => locale.fill(
                    "event.temperature-changed.lv3",
                    "{vessel}: T {from} K → {to} K (ΔT = {delta} K)",
                    &[
                        ("vessel", &vessel.to_string()),
                        ("from", &locale.number(format!("{:.3}", from.0))),
                        ("to", &locale.number(format!("{:.3}", to.0))),
                        ("delta", &locale.number(format!("{d:+.3}"))),
                    ],
                ),
            }
        }
        Event::EnergyTransferred {
            vessel,
            heating,
            requested_j,
            delivered_j,
            time_coupled,
        } => {
            let vessel = vessel.to_string();
            let delivered_kj = locale.number(format!("{:.2}", delivered_j / 1000.0));
            let requested_kj = locale.number(format!("{:.2}", requested_j / 1000.0));
            let transfer = if *heating {
                locale.t("event.energy-transferred.delivered", "delivered")
            } else {
                locale.t("event.energy-transferred.removed", "removed")
            };
            let coupling = if *time_coupled {
                locale.t("event.energy-transferred.coupled", "coupled")
            } else {
                locale.t("event.energy-transferred.not-yet-coupled", "not yet coupled")
            };
            let heating_value = if *heating {
                locale.t("event.value.true", "true")
            } else {
                locale.t("event.value.false", "false")
            };
            let time_coupled_value = if *time_coupled {
                locale.t("event.value.true", "true")
            } else {
                locale.t("event.value.false", "false")
            };
            match register.level() {
                1 if *heating => locale.fill(
                    "event.energy-transferred.lv1-heating",
                    "{vessel} receives {delivered} kJ of heat. This energy step has no elapsed-time model yet.",
                    &[("vessel", &vessel), ("delivered", &delivered_kj)],
                ),
                1 => locale.fill(
                    "event.energy-transferred.lv1-cooling",
                    "{vessel} releases {delivered} kJ of heat. This energy step has no elapsed-time model yet.",
                    &[("vessel", &vessel), ("delivered", &delivered_kj)],
                ),
                2 => locale.fill(
                    "event.energy-transferred.lv2",
                    "{vessel}: {requested} kJ requested; {delivered} kJ {transfer} — time model {coupling}",
                    &[
                        ("vessel", &vessel),
                        ("requested", &requested_kj),
                        ("delivered", &delivered_kj),
                        ("transfer", transfer),
                        ("coupling", coupling),
                    ],
                ),
                _ => locale.fill(
                    "event.energy-transferred.lv3",
                    "{vessel}: thermal energy requested={requested} J, delivered={delivered} J, heating={heating}, time_coupled={time_coupled}",
                    &[
                        ("vessel", &vessel),
                        ("requested", &locale.number(format!("{requested_j:.6}"))),
                        ("delivered", &locale.number(format!("{delivered_j:.6}"))),
                        ("heating", heating_value),
                        ("time_coupled", time_coupled_value),
                    ],
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
            _ => locale.fill(
                "event.stirred.lv3",
                "{vessel}: stir {rpm} rpm × {seconds} s; bar {bar_length_m} mm; tip {tip_speed_m_s} m/s; resuspended {resuspended_fraction}%; rate coupling {coupling}",
                &[("vessel", &vessel.to_string()), ("rpm", &locale.number(format!("{rpm:.1}"))), ("seconds", &locale.number(format!("{seconds:.1}"))), ("bar_length_m", &locale.number(format!("{:.1}", bar_length_m * 1000.0))), ("tip_speed_m_s", &locale.number(format!("{:.5}", tip_speed_m_s))), ("resuspended_fraction", &locale.number(format!("{:.2}", resuspended_fraction * 100.0))), ("coupling", locale.t(
                    if *rate_coupled { "model.coupling-active" } else { "model.coupling-absent" },
                    if *rate_coupled { "active" } else { "not yet modelled" },
                ))],
            ),
        },
        Event::EmulsionChanged {
            vessel,
            material,
            from_dispersed_fraction,
            to_dispersed_fraction,
            dispersed_volume_l,
            half_life_seconds,
        } => match register.level() {
            // Two directions, two whole sentences. German cannot reach
            // this by swapping a word inside one of them.
            1 if to_dispersed_fraction > from_dispersed_fraction => locale.fill(
                "event.emulsion-changed.lv1-dispersing",
                "Tiny {material} droplets spread through the water in {vessel}, making it cloudy.",
                &[("material", material), ("vessel", &vessel.to_string())],
            ),
            1 => locale.fill(
                "event.emulsion-changed.lv1-coalescing",
                "The droplets in {vessel} join back together, and the {material} layer starts returning.",
                &[("material", material), ("vessel", &vessel.to_string())],
            ),
            2 => locale.fill(
                "event.emulsion-changed.lv2",
                "{vessel}: {material} dispersed {from}% → {to}% ({volume} mL; {half_life} s coalescence half-life)",
                &[
                    ("vessel", &vessel.to_string()),
                    ("material", material),
                    ("from", &locale.number(format!("{:.0}", from_dispersed_fraction * 100.0))),
                    ("to", &locale.number(format!("{:.0}", to_dispersed_fraction * 100.0))),
                    ("volume", &locale.number(format!("{:.1}", dispersed_volume_l * 1000.0))),
                    ("half_life", &locale.number(format!("{half_life_seconds:.0}"))),
                ],
            ),
            _ => format!(
                "{vessel}: bounded recipe-level emulsion {:.6} → {:.6}; dispersed {:.9} L; half-life {:.3} s — no CMC, droplet distribution or CFD claim",
                from_dispersed_fraction,
                to_dispersed_fraction,
                dispersed_volume_l,
                half_life_seconds,
            ),
        },
        Event::Thickened {
            vessel,
            solid,
            strength,
            solid_mass_fraction,
            tip_speed_m_s,
            sheared_hard,
        } => {
            let name = species_name(locale, solid);
            match register.level() {
                1 => {
                    if *sheared_hard {
                        locale.fill(
                            "event.thickened.lv1-hard",
                            "Stir {vessel} fast and it fights back — it goes stiff under the stirrer, like a solid. Slow down and it runs like a liquid again.",
                            &[("vessel", &vessel.to_string())],
                        )
                    } else {
                        locale.fill(
                            "event.thickened.lv1-slow",
                            "Stirred gently, {vessel} flows like a thick liquid. Push it faster and it will not let you.",
                            &[("vessel", &vessel.to_string())],
                        )
                    }
                }
                2 => locale.fill(
                    "event.thickened.lv2",
                    "{vessel}: {percent}% {name} by mass — a shear-thickening suspension at {strength} of full strength, sheared at {speed} m/s",
                    &[("vessel", &vessel.to_string()), ("percent", &locale.number(format!("{:.0}", solid_mass_fraction * 100.0))), ("name", name), ("strength", &locale.number(format!("{strength:.2}"))), ("speed", &locale.number(format!("{tip_speed_m_s:.2}")))],
                ),
                _ => locale.fill(
                    "event.thickened.lv3",
                    "{vessel}: shear-thickening response {strength} at solid mass fraction {fraction}, bar tip speed {speed} m/s; a bounded statement that this mixture is one of the ones that does this and that this stir was fast enough to notice — no viscosity, yield stress or critical shear rate is computed, and the ledger does not move",
                    &[("vessel", &vessel.to_string()), ("strength", &locale.number(format!("{strength:.4}"))), ("fraction", &locale.number(format!("{solid_mass_fraction:.4}"))), ("speed", &locale.number(format!("{tip_speed_m_s:.4}")))],
                ),
            }
        }
        Event::GelFormed {
            vessel,
            polymer,
            crosslinker,
            to_gelled_fraction,
            polymer_grams,
            crosslinker_moles,
            ..
        } => {
            let poly = species_name(locale, polymer);
            let link = species_name(locale, crosslinker);
            match register.level() {
                1 => locale.fill(
                    "event.gel-formed.lv1",
                    "The glue in {vessel} stops running and starts stretching — it has turned into slime!",
                    &[("vessel", &vessel.to_string())],
                ),
                2 => locale.fill(
                    "event.gel-formed.lv2",
                    "{vessel}: {percent}% of the {poly} is bound into a gel by {moles} mol {link} across {grams} g of polymer",
                    &[("vessel", &vessel.to_string()), ("percent", &locale.number(format!("{:.0}", to_gelled_fraction * 100.0))), ("poly", poly), ("moles", &locale.number(format!("{:.5}", crosslinker_moles.0))), ("link", link), ("grams", &locale.number(format!("{polymer_grams:.2}")))],
                ),
                _ => locale.fill(
                    "event.gel-formed.lv3",
                    "{vessel}: gelled fraction {fraction} at {dose} mol {link} per gram {poly}; borate diesters bridge hydroxyls on neighbouring chains and exchange continuously, so nothing is consumed — a bounded teaching response, not measured rheology, and no modulus or relaxation time is claimed",
                    &[("vessel", &vessel.to_string()), ("fraction", &locale.number(format!("{to_gelled_fraction:.4}"))), ("dose", &locale.number(format!("{:.3e}", crosslinker_moles.0 / polymer_grams.max(1e-12)))), ("link", link), ("poly", poly)],
                ),
            }
        }
        Event::PolymerSwelled { vessel, dry_polymer_g, retained_water_g, swelling_ratio_g_per_g, capacity_g_per_g, saturated } => match register.level() {
            1 => locale.fill("event.polymer-swelled.lv1", "The powder in {vessel} drinks up the water and becomes a heap of soft, wet snow.", &[("vessel", &vessel.to_string())]),
            2 => locale.fill("event.polymer-swelled.lv2", "{vessel}: {polymer} g powder retains {water} g water — {ratio} times its dry mass", &[("vessel", &vessel.to_string()), ("polymer", &locale.number(format!("{dry_polymer_g:.2}"))), ("water", &locale.number(format!("{retained_water_g:.1}"))), ("ratio", &locale.number(format!("{swelling_ratio_g_per_g:.1}")))]),
            _ => locale.fill("event.polymer-swelled.lv3", "{vessel}: bounded equilibrium swelling {ratio} g/g (declared capacity {capacity} g/g; saturated={saturated}); water remains in the conserved ledger, and salinity, pH, particle size and swelling time are outside this teaching model", &[("vessel", &vessel.to_string()), ("ratio", &locale.number(format!("{swelling_ratio_g_per_g:.4}"))), ("capacity", &locale.number(format!("{capacity_g_per_g:.1}"))), ("saturated", &saturated.to_string())]),
        },
        Event::ChemiluminescenceObserved { vessel, relative_intensity, half_life_s, elapsed_s, temperature, oxidant_moles } => match register.level() {
            1 => locale.fill("event.chemiluminescence-observed.lv1", "The mixture in {vessel} glows blue. Warmer mixtures shine more brightly now, but fade sooner.", &[("vessel", &vessel.to_string())]),
            2 => locale.fill("event.chemiluminescence-observed.lv2", "{vessel}: relative blue-light intensity {intensity}; estimated half-life {half}s at {temp} K", &[("vessel", &vessel.to_string()), ("intensity", &locale.number(format!("{relative_intensity:.2}"))), ("half", &locale.number(format!("{half_life_s:.1}"))), ("temp", &locale.number(format!("{:.1}", temperature.0)))]),
            _ => locale.fill("event.chemiluminescence-observed.lv3", "{vessel}: bounded luminol-system relative intensity {intensity} after {elapsed} s at {temp} K with {oxidant} mol H2O2; Arrhenius-shaped teaching response, not photon yield or product speciation, and not the peroxyoxalate chemistry of a commercial glow stick", &[("vessel", &vessel.to_string()), ("intensity", &locale.number(format!("{relative_intensity:.4}"))), ("elapsed", &locale.number(format!("{elapsed_s:.1}"))), ("temp", &locale.number(format!("{:.2}", temperature.0))), ("oxidant", &locale.number(format!("{:.6}", oxidant_moles.0)))]),
        },
        Event::CurdlingChanged {
            vessel,
            material,
            from_formed_fraction,
            to_formed_fraction,
            separation_progress,
            curd_solids_mass_g,
            acid_species,
            acid_moles,
        } => match register.level() {
            1 => locale.fill(
                "event.curdling-changed.lv1",
                "The {material} in {vessel} separates into soft white curds and cloudy whey.",
                &[("material", material), ("vessel", &vessel.to_string())],
            ),
            2 => locale.fill(
                "event.curdling-changed.lv2",
                "{vessel}: {material} curd solids {from}% → {to}% ({mass} g aggregate solids in visible curds)",
                &[
                    ("vessel", &vessel.to_string()),
                    ("material", material),
                    ("from", &locale.number(format!("{:.0}", from_formed_fraction * 100.0))),
                    ("to", &locale.number(format!("{:.0}", to_formed_fraction * 100.0))),
                    ("mass", &locale.number(format!("{curd_solids_mass_g:.2}"))),
                ],
            ),
            _ => format!(
                "{vessel}: bounded acid-curdling response {:.6} → {:.6}; separation progress {:.6}; curd solids {:.6} g from {:.9} mol {}; conserved aggregate, no casein-speciation or wet-yield claim",
                from_formed_fraction,
                to_formed_fraction,
                separation_progress,
                curd_solids_mass_g,
                acid_moles.0,
                acid_species.0,
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
            let name = species_name(locale, sid);
            match register.level() {
                1 => locale.fill(
                    "event.ground.lv1",
                    "You grind the {what} in {vessel} into a finer powder.",
                    &[
                        ("what", name),
                        ("vessel", &vessel.to_string()),
                    ],
                ),
                2 => locale.fill(
                    "event.ground.lv2",
                    "{vessel}: {what} ground to {diameter} µm — about {area} m² surface area",
                    &[
                        ("vessel", &vessel.to_string()),
                        ("what", name),
                        ("diameter", &locale.number(format!("{diameter_um:.1}"))),
                        ("area", &locale.number(format!("{surface_area_m2:.3}"))),
                    ],
                ),
                _ => locale.fill(
                    "event.ground.lv3",
                    "{vessel}: grind {name}; {solid_moles} mol solid; mean diameter {diameter_um} µm; spherical-particle area {surface_area_m2} m²; rate coupling {coupling}",
                    &[("vessel", &vessel.to_string()), ("name", name), ("solid_moles", &locale.number(format!("{:.6}", solid_moles.0))), ("diameter_um", &locale.number(format!("{:.3}", diameter_um))), ("surface_area_m2", &locale.number(format!("{:.6}", surface_area_m2))), ("coupling", locale.t(
                    if *rate_coupled { "model.coupling-active" } else { "model.coupling-absent" },
                    if *rate_coupled { "active" } else { "not yet modelled" },
                ))],
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
                1 => locale.fill(
                    "event.centrifuged.lv1",
                    "The mini centrifuge spins {vessel}; the particles travel {travelled}% of the tube path.",
                    &[
                        ("vessel", &vessel.to_string()),
                        ("travelled", &locale.number(format!("{:.0}", strongest * 100.0))),
                    ],
                ),
                2 => locale.fill(
                    "event.centrifuged.lv2",
                    "{vessel}: {rpm} rpm for {seconds} s — {rcf} × g; {separation}% separation; balanced within {imbalance} g",
                    &[
                        ("vessel", &vessel.to_string()),
                        ("rpm", &locale.number(format!("{rpm:.0}"))),
                        ("seconds", &locale.number(format!("{seconds:.0}"))),
                        ("rcf", &locale.number(format!("{rcf:.0}"))),
                        ("separation", &locale.number(format!("{:.0}", strongest * 100.0))),
                        ("imbalance", &locale.number(format!("{imbalance_g:.2}"))),
                    ],
                ),
                _ => {
                    let detail = separations
                        .iter()
                        .map(|separation| {
                            let assumption = if separation.particle_size_assumed {
                                // The last bare literal in this file: a
                                // parenthetical appended to a diagnostic
                                // line, and a claim about the SIMULATION —
                                // it says the engine assumed a diameter
                                // rather than being told one.
                                locale.t("centrifuged.diameter-assumed", " (diameter assumed)")
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
                    locale.fill(
                        "event.centrifuged.lv3",
                        "{vessel}: centrifuge {rpm} rpm × {seconds} s, r={rotor_radius_m} m, RCF={rcf}; sample={sample_mass_g} g, counterbalance={counterbalance_g} g, Δm={imbalance_g} g; ρfluid={fluid_density_kg_m3} kg/m³, μ={dynamic_viscosity_pa_s} Pa·s; {detail}; state coupling {coupling}",
                        &[("vessel", &vessel.to_string()), ("rpm", &locale.number(format!("{rpm:.1}"))), ("seconds", &locale.number(format!("{seconds:.1}"))), ("rotor_radius_m", &locale.number(format!("{:.3}", rotor_radius_m))), ("rcf", &locale.number(format!("{rcf:.2}"))), ("sample_mass_g", &locale.number(format!("{sample_mass_g:.3}"))), ("counterbalance_g", &locale.number(format!("{counterbalance_g:.3}"))), ("imbalance_g", &locale.number(format!("{imbalance_g:.3}"))), ("fluid_density_kg_m3", &locale.number(format!("{fluid_density_kg_m3:.1}"))), ("dynamic_viscosity_pa_s", &locale.number(format!("{dynamic_viscosity_pa_s:.6}"))), ("detail", &detail.to_string()), ("coupling", locale.t(
                    if *state_coupled { "model.coupling-active" } else { "model.coupling-absent" },
                    if *state_coupled { "active" } else { "not yet modelled" },
                ))],
                    )
                }
            }
        }
        Event::UvAttenuated {
            vessel,
            material,
            wavelength_nm,
            band,
            transmitted_fraction,
            mechanism,
        } => {
            let percent = locale.number(format!("{:.1}", transmitted_fraction * 100.0));
            let nm = locale.number(format!("{wavelength_nm:.0}"));
            match register.level() {
                1 => locale.fill(
                    "event.uv-attenuated.lv1",
                    "The {name} in {vessel} lets only {percent}% of the {band} light through at {nm} nm — the rest is stopped by its filters.",
                    &[("name", material), ("vessel", &vessel.to_string()), ("percent", &percent), ("band", band), ("nm", &nm)],
                ),
                2 => locale.fill(
                    "event.uv-attenuated.lv2",
                    "{vessel}: {name} transmits {percent}% at {nm} nm ({band}) — {mechanism}",
                    &[("vessel", &vessel.to_string()), ("name", material), ("percent", &percent), ("nm", &nm), ("band", band), ("mechanism", mechanism)],
                ),
                _ => format!(
                    "{vessel}: {material} transmits {transmitted_fraction:.4} of {wavelength_nm:.1} nm ({band}); {mechanism}"
                ),
            }
        }
        Event::HeadspacePartitioned {
            vessel,
            species: sid,
            to_gas,
            moles,
            gas_fraction,
            partial_pressure_pa,
            henry_mol_per_l_atm,
            source,
        } => {
            let name = species_name(locale, sid);
            let percent = locale.number(format!("{:.1}", gas_fraction * 100.0));
            let kpa = locale.number(format!("{:.3}", partial_pressure_pa / 1000.0));
            let moles_s = locale.number(format!("{:.4}", moles.0));
            match (register.level(), *to_gas) {
                (1, true) => locale.fill(
                    "event.headspace-partitioned.lv1",
                    "Some of the {name} in {vessel} leaves the liquid for the air above it — {percent}% of it is now in the headspace.",
                    &[("name", name), ("vessel", &vessel.to_string()), ("percent", &percent)],
                ),
                (1, false) => locale.fill(
                    "event.headspace-partitioned.lv1-back",
                    "Some of the {name} in the air above {vessel} goes back into the liquid — {percent}% of it stays in the headspace.",
                    &[("name", name), ("vessel", &vessel.to_string()), ("percent", &percent)],
                ),
                (2, true) => locale.fill(
                    "event.headspace-partitioned.lv2",
                    "{vessel}: {moles} mol {name} liquid → headspace; {percent}% in the gas at {kpa} kPa partial pressure (Henry's law)",
                    &[("vessel", &vessel.to_string()), ("moles", &moles_s), ("name", name), ("percent", &percent), ("kpa", &kpa)],
                ),
                (2, false) => locale.fill(
                    "event.headspace-partitioned.lv2-back",
                    "{vessel}: {moles} mol {name} headspace → liquid; {percent}% in the gas at {kpa} kPa partial pressure (Henry's law)",
                    &[("vessel", &vessel.to_string()), ("moles", &moles_s), ("name", name), ("percent", &percent), ("kpa", &kpa)],
                ),
                _ => format!(
                    "{vessel}: {name} {} {:.6} mol; gas fraction {gas_fraction:.4}, p = {partial_pressure_pa:.1} Pa, H = {henry_mol_per_l_atm:.3} mol/(L·atm) — {source}",
                    if *to_gas { "liquid → headspace" } else { "headspace → liquid" },
                    moles.0
                ),
            }
        }
        Event::Irradiated {
            vessel,
            wavelength_nm,
            irradiance_w_m2,
            photolysis_coupled,
        } => {
            let vessel = vessel.to_string();
            let coupling = if *photolysis_coupled {
                locale.t("event.irradiated.coupled", "coupled")
            } else {
                locale.t("event.irradiated.not-yet-coupled", "not yet coupled")
            };
            let coupled_value = if *photolysis_coupled {
                locale.t("event.value.true", "true")
            } else {
                locale.t("event.value.false", "false")
            };
            match register.level() {
                1 => locale.fill(
                    "event.irradiated.lv1",
                    "The lamp shines on {vessel}. The light is applied, but photolysis is not connected yet.",
                    &[("vessel", &vessel)],
                ),
                2 => locale.fill(
                    "event.irradiated.lv2",
                    "{vessel}: lamp {wavelength} nm at {irradiance} W/m² — photolysis {coupling}",
                    &[
                        ("vessel", &vessel),
                        ("wavelength", &locale.number(format!("{wavelength_nm:.0}"))),
                        ("irradiance", &locale.number(format!("{irradiance_w_m2:.2}"))),
                        ("coupling", coupling),
                    ],
                ),
                _ => locale.fill(
                    "event.irradiated.lv3",
                    "{vessel}: irradiate λ={wavelength} nm, Ė/A={irradiance} W/m²; photolysis_coupled={coupled}",
                    &[
                        ("vessel", &vessel),
                        ("wavelength", &locale.number(format!("{wavelength_nm:.3}"))),
                        ("irradiance", &locale.number(format!("{irradiance_w_m2:.6}"))),
                        ("coupled", coupled_value),
                    ],
                ),
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
                2 => locale.fill(
                    "event.gravity-settled.lv2",
                    "{vessel}: {settled}% of the suspended particles settle in {seconds} s",
                    &[
                        ("vessel", &vessel.to_string()),
                        ("settled", &locale.number(format!("{:.0}", strongest * 100.0))),
                        ("seconds", &locale.number(format!("{seconds:.0}"))),
                    ],
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
            let name = |s: &SpeciesId| species_name(locale, s).to_string();
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
            1 => locale.fill(
                "event.partitioned.lv1",
                "Some of the {species} in {vessel} moves into each layer.",
                &[("species", species_name(locale, species)), ("vessel", &vessel.to_string())],
            ),
            2 => locale.fill(
                "event.partitioned.lv2",
                "{vessel}: {species} split between the layers — {fraction_lower}% in the lower, the rest dissolved in the upper",
                &[("vessel", &vessel.to_string()), ("species", species_name(locale, species)), ("fraction_lower", &locale.number(format!("{:.0}", fraction_lower * 100.0)))],
            ),
            _ => locale.fill(
                "event.partitioned.lv3",
                "{vessel}: {species} partitioned at K from UNIFAC γ∞ ratio; fraction in lower layer {fraction_lower} (equal-activity split over the layer sizes)",
                &[("vessel", &vessel.to_string()), ("species", &species.0.to_string()), ("fraction_lower", &locale.number(format!("{:.4}", fraction_lower)))],
            ),
        },
        Event::DissolvedInSolvent { vessel, species, solvent, dissolved, undissolved } => {
            let name = species_name(locale, species);
            let solv = species_name(locale, solvent);
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
                _ => locale.fill(
                    "event.dissolved-in-solvent.lv3",
                    "{vessel}: {species} in {solvent}: dissolved {dissolved} mol to the curated solubility limit, {undissolved} mol solid remains. Model boundary: undissociated solute, no speciation or activity model in an organic phase",
                    &[("vessel", &vessel.to_string()), ("species", &species.0.to_string()), ("solvent", &solvent.0.to_string()), ("dissolved", &locale.number(format!("{:.6}", dissolved.0))), ("undissolved", &locale.number(format!("{:.6}", undissolved.0)))],
                ),
            }
        }
        Event::InertInSolvent { vessel, species, solvent, why } => {
            let name = species_name(locale, species);
            let solv = species_name(locale, solvent);
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
                _ => locale.fill(
                    "event.inert-in-solvent.lv3",
                    "{vessel}: {species} inert in {solvent}: {why}",
                    &[("vessel", &vessel.to_string()), ("species", &species.0.to_string()), ("solvent", &solvent.0.to_string()), ("why", &why.to_string())],
                ),
            }
        }
        Event::OrgReacted { vessel, name, equation, extent, boundary } => match register.level() {
            1 => locale.fill(
                "event.org-reacted.lv1",
                "Something new forms in {vessel} — the {name} reaction turns the mixture into different substances.",
                &[("vessel", &vessel.to_string()), ("name", &name.to_string())],
            ),
            2 => locale.fill(
                "event.org-reacted.lv2",
                "{vessel}: {name} ran — {equation} ({extent} mol reacted)",
                &[("vessel", &vessel.to_string()), ("name", &name.to_string()), ("equation", &equation.to_string()), ("extent", &locale.number(format!("{:.3}", extent.0)))],
            ),
            _ => format!(
                "{vessel}: {name}, {equation}, extent {:.6} mol. Boundary: {boundary}",
                extent.0
            ),
        },
        // The census renders itself, and has since EXP-?? — the CLI was
        // calling `Census::render` directly because there was no event to
        // carry it. Now that there is, the picture reaches every host
        // through the ordinary event stream, and the sealed-unknown mask
        // applies to it like any other rendered line.
        Event::ParticlesCounted { vessel, census } => {
            // The drawing itself already answers to the register; only the
            // sentence introducing it changes. lv2 keeps the CLI's existing
            // wording verbatim, because that is the line the REPL prints
            // and `quest.rs` splits its output on.
            let drawing = census.render(register).to_string();
            let id = vessel.to_string();
            let args = [("vessel", id.as_str()), ("drawing", drawing.as_str())];
            match register.level() {
                1 => locale.fill(
                    "event.particles-counted.lv1",
                    "What the particles in {vessel} are doing:\n{drawing}",
                    &args,
                ),
                _ => locale.fill(
                    "event.particles-counted.lv2",
                    "{vessel} — what the particles are doing:\n{drawing}",
                    &args,
                ),
            }
        }
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
                        "event.smelled.lv3-none",
                        "{vessel}: no curated odour among the volatile species — and 'odourless' is itself data: CO2 and CO teach why a nose is not a gas detector",
                        &[("vessel", &vessel.to_string())],
                    ),
                }
            } else {
                let list: Vec<String> = notes
                    .iter()
                    .map(|(sp, d)| {
                        let name = species_name(locale, sp);
                        locale.fill(
                            "event.smelled.lv3-species",
                            "{name}: {d}",
                            &[("name", name), ("d", d)],
                        )
                    })
                    .collect();
                match register.level() {
                    1 => locale.fill(
                        "event.smelled.lv1-you-waft-air",
                        "You waft the air from {vessel} toward your nose — {list}.",
                        &[("vessel", &vessel.to_string()), ("list", &list.join("; ").to_string())],
                    ),
                    2 => format!("{vessel}: wafted — {}", list.join("; ")),
                    _ => locale.fill(
                        "event.smelled.lv3",
                        "{vessel}: waft (taught technique — never a direct huff): {list}. Odour words are editorial curation in the qualitative-analysis register",
                        &[("vessel", &vessel.to_string()), ("list", &list.join("; ").to_string())],
                    ),
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
            2 => locale.fill(
                "event.gas-tested.lv2",
                "{vessel}: {test} — {outcome}",
                &[
                    ("vessel", &vessel.to_string()),
                    ("test", &test.to_string()),
                    // The verdict word is chosen by a branch, so it goes
                    // through the catalogue rather than being an English
                    // literal dropped into a German sentence.
                    (
                        "outcome",
                        locale.t(
                            if *positive { "test.positive" } else { "test.negative" },
                            if *positive { "positive" } else { "negative" },
                        ),
                    ),
                ],
            ),
            _ => format!("{vessel}: {test}: {notes}"),
        },
        Event::Burst { vessel, at_pa, rating_pa } => match register.level() {
            1 => locale.fill(
                "event.burst.lv1",
                "BANG — the sealed {vessel} could not hold the pressure and let go!",
                &[("vessel", &vessel.to_string())],
            ),
            2 => locale.fill(
                "event.burst.lv2",
                "{vessel}: BURST at {at_pa} kPa (glass rating ~{rating_pa} kPa) — seal gone, gases vented",
                &[("vessel", &vessel.to_string()), ("at_pa", &locale.number(format!("{:.0}", at_pa / 1000.0))), ("rating_pa", &locale.number(format!("{:.0}", rating_pa / 1000.0)))],
            ),
            _ => locale.fill(
                "event.burst.lv3",
                "{vessel}: sealed headspace exceeded the teaching burst constant ({at_pa} Pa > {rating_pa} Pa); the seal failed, the headspace is open, every gas vented as events, and the ledger is exact through the failure. The constant is editorial — the model's claim is that sealed vessels HAVE limits, not a certification of any flask",
                &[("vessel", &vessel.to_string()), ("at_pa", &locale.number(format!("{:.3e}", at_pa))), ("rating_pa", &locale.number(format!("{:.3e}", rating_pa)))],
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
            2 => locale.fill(
                "event.nuclide-spiked.lv2",
                "{vessel}: spiked with {moles} mol {nuclide} — initial activity {activity_bq} Bq",
                &[("vessel", &vessel.to_string()), ("moles", &locale.number(format!("{:.3e}", moles.0))), ("nuclide", &nuclide.to_string()), ("activity_bq", &locale.number(format!("{:.3e}", activity_bq)))],
            ),
            _ => locale.fill(
                "event.nuclide-spiked.lv3",
                "{vessel}: {nuclide} tracer, {moles} mol, A₀ = {activity_bq} Bq (λN, NUBASE2020 half-life). Boundary: tracer-scale, chemically inert in this model; ionising-radiation practice is real-world, not simulated",
                &[("vessel", &vessel.to_string()), ("nuclide", &nuclide.to_string()), ("moles", &locale.number(format!("{:.6e}", moles.0))), ("activity_bq", &locale.number(format!("{:.6e}", activity_bq)))],
            ),
        },
        Event::Decayed { vessel, parent, daughter, mode, moles, half_life_s, equation } => match register.level() {
            1 => locale.fill(
                "event.decayed.lv1",
                "Inside {vessel}, some of the {parent} quietly turned into {daughter} while you waited.",
                &[("vessel", &vessel.to_string()), ("parent", &parent.to_string()), ("daughter", &daughter.to_string())],
            ),
            2 => locale.fill(
                "event.decayed.lv2",
                "{vessel}: {equation} — {moles} mol decayed ({mode}, t½ = {half_life_s} s)",
                &[("vessel", &vessel.to_string()), ("equation", &equation.to_string()), ("moles", &locale.number(format!("{:.3e}", moles.0))), ("mode", &mode.to_string()), ("half_life_s", &locale.number(format!("{:.3e}", half_life_s)))],
            ),
            _ => locale.fill(
                "event.decayed.lv3",
                "{vessel}: {equation}; {moles} mol transmuted. Elements do NOT conserve across this event — nucleons do (α parcels keep their He-4 in the ledger), charge bookkeeping notes the departing β/ν, and the mass defect is a stated model boundary",
                &[("vessel", &vessel.to_string()), ("equation", &equation.to_string()), ("moles", &locale.number(format!("{:.6e}", moles.0)))],
            ),
        },
        Event::Chromatographed { vessel, plates, void_time_s, peaks, outside_method } => match register.level() {
            // An empty chromatogram is a result, not a blank line: the run
            // happened and the detector saw nothing. Saying which species went
            // past unseparated is the whole of the method's scope, and without
            // this the sentence would read "comes out one thing at a time: ".
            1 if peaks.is_empty() => locale.fill(
                "event.chromatographed.lv1-nothing-separated",
                "Everything dissolved in {vessel} runs straight through the column with the water — this method separates none of it ({outside_method}).",
                &[
                    ("vessel", &vessel.to_string()),
                    ("outside_method", &outside_method.iter().map(|s| species_name(locale, s)).collect::<Vec<_>>().join(", ").to_string()),
                ],
            ),
            1 => {
                // KID-9: the same separation as a child sees it — spots at
                // different heights up a strip of paper. The distances are
                // the Rf values, which are the same partition coefficients
                // the column's times come from.
                let order = peaks
                    .iter()
                    .map(|p| species_name(locale, &p.species))
                    .collect::<Vec<_>>()
                    .join(", then ");
                let strip = peaks
                    .iter()
                    .rev()
                    .map(|p| {
                        locale.fill(
                            "event.chromatographed.lv1-spot",
                            "{name} {mm} mm up",
                            &[
                                ("name", species_name(locale, &p.species)),
                                ("mm", &locale.number(format!("{:.0}", p.rf * 100.0))),
                            ],
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                locale.fill(
                    "event.chromatographed.lv1-mixture-from-runs",
                    "The mixture from {vessel} separates: on the column it comes out one thing at a time — {order} — and on a paper strip the same colours stop at different heights: {strip}.",
                    &[("vessel", &vessel.to_string()), ("order", &order.to_string()), ("strip", &strip.to_string())],
                )
            }
            2 => {
                let table = peaks
                    .iter()
                    .map(|p| {
                        locale.fill(
                            "event.chromatographed.lv2-peak",
                            "{name} at {seconds} s, Rf {rf} ({area}% area)",
                            &[
                                ("name", species_name(locale, &p.species)),
                                ("seconds", &locale.number(format!("{:.0}", p.retention_time_s))),
                                ("rf", &locale.number(format!("{:.2}", p.rf))),
                                ("area", &locale.number(format!("{:.0}", p.relative_area * 100.0))),
                            ],
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                let unseen = if outside_method.is_empty() {
                    String::new()
                } else {
                    locale.fill(
                        "event.chromatographed.lv2",
                        " — the dissolved ions ({outside_method}) pass with the water and are not separated",
                        &[("outside_method", &outside_method.iter().map(|s| s.0.as_str()).collect::<Vec<_>>().join(", ").to_string())],
                    )
                };
                if peaks.is_empty() {
                    locale.fill(
                        "event.chromatographed.lv2-no-peaks",
                        "{vessel}: chromatogram — no peaks{unseen}",
                        &[("vessel", &vessel.to_string()), ("unseen", &unseen.to_string())],
                    )
                } else {
                    locale.fill(
                        "event.chromatographed.lv1-chromatogram",
                        "{vessel}: chromatogram — {table}{unseen}",
                        &[("vessel", &vessel.to_string()), ("table", &table.to_string()), ("unseen", &unseen.to_string())],
                    )
                }
            }
            _ => {
                let table = peaks
                    .iter()
                    .map(|p| {
                        format!(
                            "{} K={:.3} tR={:.1}s w={:.1}s Rf={:.3} A={:.3}",
                            p.species.0, p.partition_k, p.retention_time_s, p.width_s, p.rf, p.relative_area,
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                let unseen = if outside_method.is_empty() {
                    String::new()
                } else {
                    locale.fill(
                        "event.chromatographed.lv3-outside-method-ion",
                        "; outside the method: {outside_method} (ion exchange not modeled)",
                        &[("outside_method", &outside_method.iter().map(|s| s.0.as_str()).collect::<Vec<_>>().join(", ").to_string())],
                    )
                };
                // KID-9: which K's were computed and which were reviewed.
                // The header used to claim every one came from UNIFAC. That
                // was true while only three species had group
                // decompositions, and became false the moment a curated
                // coefficient let the dyes onto the column — a provenance
                // line that names the wrong method is worse than none.
                let curated = peaks
                    .iter()
                    .filter(|p| crate::instrument::curated_partition_k(&p.species.0).is_some())
                    .map(|p| p.species.0.as_str())
                    .collect::<Vec<_>>();
                let method = if curated.is_empty() {
                    locale
                        .lookup("event.chromatographed.lv3-method-unifac")
                        .unwrap_or(
                            "K = γ∞(water)/γ∞(alkane) from the same UNIFAC the funnel partitions on",
                        )
                        .to_string()
                } else {
                    locale.fill(
                        "event.chromatographed.lv3-method-mixed",
                        "K = γ∞(water)/γ∞(alkane) from the same UNIFAC the funnel partitions on, except for {curated}, whose K is a reviewed teaching value: a dye is a glycoside or a sulfonated aromatic and a UNIFAC decomposition of one would be a fiction dressed as a calculation",
                        &[("curated", &curated.join(", "))],
                    )
                };
                locale.fill(
                    "event.chromatographed.lv3",
                    "{vessel}: N={plates}, t0={void_time_s}s, β=0.5; {method}; tR = t0·(1+K·β), w = 4·tR/√N, Rf = Kβ_paper/(1+Kβ_paper) on a 100 mm strip — {table}{unseen}",
                    &[("vessel", &vessel.to_string()), ("plates", &plates.to_string()), ("void_time_s", &locale.number(format!("{void_time_s:.0}"))), ("method", &method.to_string()), ("table", &table.to_string()), ("unseen", &unseen.to_string())],
                )
            }
        },
        Event::Drained { from, to, solvent, moles } => match register.level() {
            1 => locale.fill(
                "event.drained.lv1",
                "You open the tap and the bottom layer runs from {from} into {to}.",
                &[("from", &from.to_string()), ("to", &to.to_string())],
            ),
            2 => locale.fill(
                "event.drained.lv2",
                "{from} → {to}: the lower layer drained — {moles} mol {species} with everything dissolved in it; the upper layer stays behind",
                &[("from", &from.to_string()), ("to", &to.to_string()), ("moles", &locale.number(format!("{:.3}", moles.0))), ("species", species_name(locale, solvent))],
            ),
            _ => locale.fill(
                "event.drained.lv3",
                "{from} → {to}: lower layer ({solvent}, {moles} mol solvent) plus its aqueous solutes; solids left behind — a stopcock passes liquid, and a settled solid is a filtration question",
                &[("from", &from.to_string()), ("to", &to.to_string()), ("solvent", &solvent.0.to_string()), ("moles", &locale.number(format!("{:.6}", moles.0)))],
            ),
        },
        Event::LayersFormed { vessel, upper, lower } => match register.level() {
            1 => locale.fill(
                "event.layers-formed.lv1",
                "The liquid in {vessel} separates into two layers.",
                &[("vessel", &vessel.to_string())],
            ),
            2 => locale.fill(
                "event.layers-formed.lv2",
                "{vessel}: two layers — {species} floating on {species2}; mixing them would raise the Gibbs energy, so they split",
                &[("vessel", &vessel.to_string()), ("species", species_name(locale, upper)), ("species2", species_name(locale, lower))],
            ),
            _ => locale.fill(
                "event.layers-formed.lv3",
                "{vessel}: liquid–liquid split (UNIFAC LLE, common-tangent construction). The split and the layer order are robust; the trace mutual solubilities are upper bounds — VLE-fitted UNIFAC parameters underestimate alkane–water γ∞ — and are deliberately not reported",
                &[("vessel", &vessel.to_string())],
            ),
        },
        Event::MaterialLayersFormed {
            vessel,
            upper_material,
            lower,
        } => match register.level() {
            1 => locale.fill(
                "event.material-layers-formed.lv1",
                "The {upper_material} floats in a separate layer above the water in {vessel}.",
                &[
                    ("upper_material", upper_material),
                    ("vessel", &vessel.to_string()),
                ],
            ),
            2 => locale.fill(
                "event.material-layers-formed.lv2",
                "{vessel}: {upper_material} forms the upper layer; {lower} is denser and remains below",
                &[
                    ("vessel", &vessel.to_string()),
                    ("upper_material", upper_material),
                    ("lower", species_name(locale, lower)),
                ],
            ),
            _ => format!(
                "{vessel}: reviewed material-layer role — unresolved {upper_material} is immiscible with the aqueous phase and lies above {}; this is not a molecular LLE calculation",
                lower.0,
            ),
        },
        Event::Evaporated { vessel, moles } => match register.level() {
            1 => locale.fill(
                "event.evaporated.lv1",
                "Steam rises from {vessel} — the water is boiling away!",
                &[("vessel", &vessel.to_string())],
            ),
            2 => locale.fill(
                "event.evaporated.lv2",
                "{vessel}: {moles} mol water evaporated",
                &[("vessel", &vessel.to_string()), ("moles", &locale.number(format!("{:.3}", moles.0)))],
            ),
            _ => locale.fill(
                "event.evaporated.lv3",
                "{vessel}: {moles} mol H2O evaporated (vaporisation enthalpy not yet in the balance)",
                &[("vessel", &vessel.to_string()), ("moles", &locale.number(format!("{:.6}", moles.0)))],
            ),
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
                    locale.fill(
                        "event.distilled.lv2-mol-water-mol",
                        "{from} → {to}: {water} mol water + {ethanol} mol ethanol over{column} — the vapour matches the liquid now (azeotrope), so more stages or harder boiling enrich nothing",
                        &[("from", &from.to_string()), ("to", &to.to_string()), ("water", &locale.number(format!("{:.3}", water.0))), ("ethanol", &locale.number(format!("{:.3}", ethanol.0))), ("column", &column.to_string())],
                    )
                } else if (t1 - t0).abs() > 0.05 {
                    locale.fill(
                        "event.distilled.lv2-mol-water-mol-2",
                        "{from} → {to}: {water} mol water + {ethanol} mol ethanol over{column}; the pot boiled at {t0} °C and climbed to {t1} °C as the light component left",
                        &[("from", &from.to_string()), ("to", &to.to_string()), ("water", &locale.number(format!("{:.3}", water.0))), ("ethanol", &locale.number(format!("{:.3}", ethanol.0))), ("column", &column.to_string()), ("t0", &locale.number(format!("{t0:.1}"))), ("t1", &locale.number(format!("{t1:.1}")))],
                    )
                } else {
                    locale.fill(
                        "event.distilled.lv2-mol-water-mol-3",
                        "{from} → {to}: {water} mol water + {ethanol} mol ethanol over{column}, boiling at {t0} °C",
                        &[("from", &from.to_string()), ("to", &to.to_string()), ("water", &locale.number(format!("{:.3}", water.0))), ("ethanol", &locale.number(format!("{:.3}", ethanol.0))), ("column", &column.to_string()), ("t0", &locale.number(format!("{t0:.1}")))],
                    )
                }
            }
            _ => locale.fill(
                "event.distilled.lv3",
                "{from} → {to}: Rayleigh batch cut, {stages} ideal stage(s) at total reflux (a real column at finite reflux separates less, never more); pot {at} K → {ended} K; latent heat {energy_kj} kJ paid by the burner and dumped by the condenser, deliberately outside the vessel ledger{azeotrope}",
                &[("from", &from.to_string()), ("to", &to.to_string()), ("stages", &stages.to_string()), ("at", &locale.number(format!("{:.2}", at.0))), ("ended", &locale.number(format!("{:.2}", ended.0))), ("energy_kj", &locale.number(format!("{energy_kj:.2}"))), ("azeotrope", (if *azeotropic { locale.t("distilled.azeotrope", "; azeotrope reached: y = x") } else { "" }))],
            ),
        },
        Event::Transferred { from, to, fraction } => match register.level() {
            1 => locale.fill(
                "event.transferred.lv1",
                "You pour some of {from} into {to}.",
                &[("from", &from.to_string()), ("to", &to.to_string())],
            ),
            _ => locale.fill(
                "event.transferred.lv3",
                "{from} → {to}: {fraction}% of the liquid",
                &[("from", &from.to_string()), ("to", &to.to_string()), ("fraction", &locale.number(format!("{:.0}", fraction * 100.0)))],
            ),
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
            1 => locale.fill(
                "event.mixed.lv1",
                "You mix some of {a} and {b} together in {into}. It settles at {temperature_into} °C.",
                &[("a", &a.to_string()), ("b", &b.to_string()), ("into", &into.to_string()), ("temperature_into", &locale.number(format!("{:.0}", temperature_into.to_celsius())))],
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
            // The substance is the subject of the sentence, so it is
            // named in the reader's language and not in the catalogue's.
            let name = species_name(locale, sid);
            match register.level() {
                1 => locale.fill(
                    "event.dissolved.lv1",
                    "The {name} disappears into the water in {vessel}!",
                    &[("name", name), ("vessel", &vessel.to_string())],
                ),
                2 => locale.fill(
                    "event.dissolved.lv2",
                    "{vessel}: {moles} mol {name} dissolved",
                    &[
                        ("vessel", &vessel.to_string()),
                        ("moles", &locale.number(quantity(moles.0, 4))),
                        ("name", name),
                    ],
                ),
                _ => format!("{vessel}: {} mol {name} dissolved", quantity(moles.0, 6)),
            }
        }
        Event::Neutralised { vessel, moles } => match register.level() {
            1 => locale.fill(
                "event.neutralised.lv1",
                "The acid and the alkali cancel each other out in {vessel} — what they make is water.",
                &[("vessel", &vessel.to_string())],
            ),
            2 => locale.fill(
                "event.neutralised.lv2",
                "{vessel}: {moles} mol neutralised",
                &[("vessel", &vessel.to_string()), ("moles", &locale.number(quantity(moles.0, 4)))],
            ),
            _ => locale.fill(
                "event.neutralised.lv3",
                "{vessel}: {moles} mol of acidity cancelled (from the change in the solutes' net charge; PHREEQC reports element totals and never this reaction)",
                &[("vessel", &vessel.to_string()), ("moles", &locale.number(quantity(moles.0, 6)))],
            ),
        },
        Event::Precipitated {
            vessel,
            species: sid,
            moles,
        } => {
            let data = species::lookup(sid);
            let name = species_name(locale, sid);
            match register.level() {
                1 => {
                    let colour = appearance_word(data.and_then(|d| d.appearance), locale);
                    locale.fill(
                        "event.precipitated.lv1",
                        "It went cloudy in {vessel}! A {colour} solid appears at the bottom — that's called a precipitate.",
                        &[("vessel", &vessel.to_string()), ("colour", colour)],
                    )
                }
                2 => locale.fill(
                    "event.precipitated.lv2",
                    "{vessel}: {moles} mol {name} precipitated ↓",
                    &[
                        ("vessel", &vessel.to_string()),
                        ("moles", &locale.number(quantity(moles.0, 4))),
                        ("name", name),
                    ],
                ),
                _ => {
                    format!("{vessel}: {} mol {name} precipitated", quantity(moles.0, 6))
                }
            }
        }
        Event::Supersaturated {
            vessel,
            species: sid,
            dissolved,
            capacity,
        } => {
            let name = species_name(locale, sid);
            let excess = (dissolved.0 - capacity.0).max(0.0);
            let times = if capacity.0 > 0.0 {
                dissolved.0 / capacity.0
            } else {
                f64::INFINITY
            };
            match register.level() {
                1 => locale.fill(
                    "event.supersaturated.lv1",
                    "There is more {name} in the water in {vessel} than it can really hold — and it is staying there. It needs something to start growing on.",
                    &[("name", name), ("vessel", &vessel.to_string())],
                ),
                2 => locale.fill(
                    "event.supersaturated.lv2",
                    "{vessel}: supersaturated — {dissolved} mol {name} dissolved against a limit of {capacity} mol at this temperature ({times}× saturation); it stays in solution until something seeds it",
                    &[("vessel", &vessel.to_string()), ("dissolved", &locale.number(format!("{:.4}", dissolved.0))), ("name", name), ("capacity", &locale.number(format!("{:.4}", capacity.0))), ("times", &locale.number(format!("{times:.2}")))],
                ),
                _ => locale.fill(
                    "event.supersaturated.lv3",
                    "{vessel}: {name} supersaturated by {excess} mol ({dissolved} dissolved against {capacity} held at this vessel's temperature, from the two-point limit); metastable — this bench crystallises it only onto a seed of the same solid, and models neither spontaneous nucleation nor the induction time before it",
                    &[("vessel", &vessel.to_string()), ("name", name), ("excess", &locale.number(format!("{excess:.6}"))), ("dissolved", &locale.number(format!("{:.6}", dissolved.0))), ("capacity", &locale.number(format!("{:.6}", capacity.0)))],
                ),
            }
        }
        Event::Plated {
            vessel,
            species: sid,
            onto,
            moles,
        } => {
            let data = species::lookup(sid);
            let name = species_name(locale, sid);
            let host = species_name(locale, onto);
            match register.level() {
                1 => {
                    let colour = appearance_word(data.and_then(|d| d.appearance), locale);
                    locale.fill(
                        "event.plated.lv1",
                        "A {colour} coating of {name} grows on the {host} in {vessel} — the {name} came out of the water onto it.",
                        &[("colour", colour), ("name", name), ("host", host), ("vessel", &vessel.to_string())],
                    )
                }
                2 => locale.fill(
                    "event.plated.lv2",
                    "{vessel}: {moles} mol {name} plated out onto {host}",
                    &[("vessel", &vessel.to_string()), ("moles", &locale.number(quantity(moles.0, 4))), ("name", name), ("host", host)],
                ),
                _ => locale.fill(
                    "event.plated.lv3",
                    "{vessel}: {moles} mol {name} plated out onto {host}",
                    &[("vessel", &vessel.to_string()), ("moles", &locale.number(quantity(moles.0, 6))), ("name", name), ("host", host)],
                ),
            }
        }
        Event::Inert { vessel, species: sid, why } => {
            let name = species_name(locale, sid);
            match register.level() {
                // KID-5: say which question this answers.
                //
                // Every `Inert` comes from the activity series asking one
                // thing — can anything dissolved here take this metal's
                // electrons? The lv2 and lv3 forms carry that scope in
                // `why`; the lv1 form dropped it, and read as a claim about
                // the metal in general. That became false the day iron
                // started rusting: the bench would report the nail going
                // orange and, in the same breath, that nothing happens to
                // the iron because it is too unreactive.
                1 => locale.fill(
                    "event.inert.lv1",
                    "The {name} in {vessel} does not swap places with anything dissolved here — that is the real answer, not a gap.",
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
        // BRD-023: the corrosion verdict. `why` already carries the
        // whole mechanism in the register the module wrote it in, so the
        // levels differ in how much of it they show rather than in what
        // they claim.
        Event::BelowAutoignition {
            vessel,
            fuel,
            autoignition,
            temperature,
        } => {
            let name = species_name(locale, fuel);
            let needs = locale.number(format!("{:.0}", autoignition.to_celsius()));
            let at = locale.number(format!("{:.0}", temperature.to_celsius()));
            match register.level() {
                1 => locale.fill(
                    "event.below-autoignition.lv1",
                    "The {name} in {vessel} is warm but not hot enough to catch — it needs {needs} °C, or a spark.",
                    &[("name", name), ("vessel", &vessel.to_string()), ("needs", &needs)],
                ),
                2 => locale.fill(
                    "event.below-autoignition.lv2",
                    "{vessel}: {name} below autoignition — at {at} °C, needs {needs} °C or a spark",
                    &[("vessel", &vessel.to_string()), ("name", name), ("at", &at), ("needs", &needs)],
                ),
                _ => format!(
                    "{vessel}: {name} with oxygen at {:.1} K, below its autoignition temperature of {:.1} K; no spark, so the thermal solver stands aside",
                    temperature.0, autoignition.0
                ),
            }
        }
        Event::SealedCell {
            vessel,
            material,
            open_circuit_volts,
            reaction,
            why,
        } => {
            let volts = locale.number(format!("{open_circuit_volts:.1}"));
            match register.level() {
                1 => locale.fill(
                    "event.sealed-cell.lv1",
                    "The {material} in {vessel} is a sealed cell: {why} It pushes about {volts} V, and because nothing gets out through the case it weighs exactly what it weighed before.",
                    &[("material", material), ("vessel", &vessel.to_string()), ("why", why), ("volts", &volts)],
                ),
                2 => locale.fill(
                    "event.sealed-cell.lv2",
                    "{vessel}: {material} — {reaction}, about {volts} V open-circuit; sealed, so the mass is conserved",
                    &[("vessel", &vessel.to_string()), ("material", material), ("reaction", reaction), ("volts", &volts)],
                ),
                _ => format!(
                    "{vessel}: {material}, curated discharge {reaction} at {open_circuit_volts:.2} V open-circuit. The reaction is NAMED and not run — its products have no species in this registry and no charge is tracked — so the ledger is untouched and the object's mass is conserved by construction rather than by arithmetic over products"
                ),
            }
        }
        Event::PolymerHeated {
            vessel,
            material,
            state,
            temperature,
            threshold,
            reversible: _,
            cross_linked,
        } => {
            // The recipe display name, untouched: material names are
            // translated by the shell against the registry rather than by
            // this catalogue, which is what `MaterialAdded` above does too.
            let name = material.as_str();
            let at = locale.number(format!("{:.0}", temperature.to_celsius()));
            let limit = locale.number(format!("{:.0}", threshold.to_celsius()));
            match (register.level(), state, cross_linked) {
                (1, crate::ops::PolymerState::Softened, _) => locale.fill(
                    "event.polymer-heated.lv1-softened",
                    "The {name} in {vessel} has gone soft — its chains are sliding past one another, so it can be pressed into a new shape, and it will set that way when it cools.",
                    &[("name", name), ("vessel", &vessel.to_string())],
                ),
                (1, crate::ops::PolymerState::Charred, _) => locale.fill(
                    "event.polymer-heated.lv1-charred",
                    "The {name} in {vessel} has charred. Past {limit} °C the bonds holding it together break, and nothing brings it back.",
                    &[("name", name), ("vessel", &vessel.to_string()), ("limit", &limit)],
                ),
                (1, crate::ops::PolymerState::Rigid, true) => locale.fill(
                    "event.polymer-heated.lv1-network",
                    "The {name} in {vessel} keeps its shape, and it will keep it all the way up: a cured thermoset has no melting point to reach. It is cross-linked into one molecule, so there are no separate chains to slide.",
                    &[("name", name), ("vessel", &vessel.to_string())],
                ),
                (1, crate::ops::PolymerState::Rigid, false) => locale.fill(
                    "event.polymer-heated.lv1-rigid",
                    "The {name} in {vessel} is still hard. Above {limit} °C its chains would start to slide and it would soften.",
                    &[("name", name), ("vessel", &vessel.to_string()), ("limit", &limit)],
                ),
                (2, crate::ops::PolymerState::Softened, _) => locale.fill(
                    "event.polymer-heated.lv2-softened",
                    "{vessel}: {name} softened — {at} °C, above its {limit} °C softening point; reversible on cooling",
                    &[("vessel", &vessel.to_string()), ("name", name), ("at", &at), ("limit", &limit)],
                ),
                (2, crate::ops::PolymerState::Charred, _) => locale.fill(
                    "event.polymer-heated.lv2-charred",
                    "{vessel}: {name} charred — {at} °C, past its {limit} °C decomposition temperature; not reversible",
                    &[("vessel", &vessel.to_string()), ("name", name), ("at", &at), ("limit", &limit)],
                ),
                (2, crate::ops::PolymerState::Rigid, true) => locale.fill(
                    "event.polymer-heated.lv2-network",
                    "{vessel}: {name} rigid at {at} °C — cross-linked, so no softening point exists; it decomposes at {limit} °C instead",
                    &[("vessel", &vessel.to_string()), ("name", name), ("at", &at), ("limit", &limit)],
                ),
                (2, crate::ops::PolymerState::Rigid, false) => locale.fill(
                    "event.polymer-heated.lv2-rigid",
                    "{vessel}: {name} rigid at {at} °C — below its {limit} °C softening point",
                    &[("vessel", &vessel.to_string()), ("name", name), ("at", &at), ("limit", &limit)],
                ),
                (_, crate::ops::PolymerState::Softened, _) => format!(
                    "{vessel}: {name} at {:.1} K, above the reviewed softening point {:.1} K — chain slip, reversible; no viscosity or rate of flow is claimed",
                    temperature.0, threshold.0
                ),
                (_, crate::ops::PolymerState::Charred, _) => format!(
                    "{vessel}: {name} at {:.1} K, past the reviewed decomposition temperature {:.1} K — irreversible; the products of the pyrolysis are not modelled and the ledger is untouched",
                    temperature.0, threshold.0
                ),
                (_, crate::ops::PolymerState::Rigid, true) => format!(
                    "{vessel}: {name} at {:.1} K, cross-linked — no softening point exists at any temperature; the nearest threshold is decomposition at {:.1} K",
                    temperature.0, threshold.0
                ),
                (_, crate::ops::PolymerState::Rigid, false) => format!(
                    "{vessel}: {name} at {:.1} K, below the reviewed softening point {:.1} K",
                    temperature.0, threshold.0
                ),
            }
        }
        Event::Corroded {
            vessel,
            species: sid,
            corroding,
            why,
        } => {
            let name = species_name(locale, sid);
            match (register.level(), *corroding) {
                (1, true) => locale.fill(
                    "event.corroded.lv1",
                    "The {name} in {vessel} is the one that corrodes here.",
                    &[("name", name), ("vessel", &vessel.to_string())],
                ),
                (1, false) => locale.fill(
                    "event.corroded.lv1-spared",
                    "The {name} in {vessel} is not corroding — and that is an answer, not a gap.",
                    &[("name", name), ("vessel", &vessel.to_string())],
                ),
                (2, true) => locale.fill(
                    "event.corroded.lv2",
                    "{vessel}: {name} corrodes — {why}",
                    &[("vessel", &vessel.to_string()), ("name", name), ("why", why)],
                ),
                (2, false) => locale.fill(
                    "event.corroded.lv2-spared",
                    "{vessel}: {name} does not corrode — {why}",
                    &[("vessel", &vessel.to_string()), ("name", name), ("why", why)],
                ),
                _ => {
                    let verdict = if *corroding { "corroding" } else { "not corroding" };
                    format!("{vessel}: {name} {verdict}: {why}")
                }
            }
        }
        Event::Adsorbed {
            vessel,
            sorbate,
            sorbent,
            held,
            loading_mg_per_g,
            still_dissolved,
            boundary,
        } => {
            let dye = species_name(locale, sorbate);
            let solid = species_name(locale, sorbent);
            // The fraction is what answers the question, so it is what
            // lv1 says: "most of it" and "some of it" are different
            // answers to "can charcoal take the colour out of water".
            let total = held.0 + still_dissolved.0;
            let taken = if total > 0.0 { held.0 / total } else { 0.0 };
            match register.level() {
                1 => locale.fill(
                    "event.adsorbed.lv1",
                    "The {solid} in {vessel} holds on to {percent} of the {dye}; the rest stays in the water.",
                    &[
                        ("solid", solid),
                        ("vessel", &vessel.to_string()),
                        ("percent", &locale.number(format!("{:.0}%", taken * 100.0))),
                        ("dye", dye),
                    ],
                ),
                2 => locale.fill(
                    "event.adsorbed.lv2",
                    "{vessel}: {solid} holds {held} mol of {dye} ({loading} mg/g); {left} mol is still dissolved",
                    &[
                        ("vessel", &vessel.to_string()),
                        ("solid", solid),
                        ("held", &locale.number(format!("{:.6}", held.0))),
                        ("dye", dye),
                        ("loading", &locale.number(format!("{loading_mg_per_g:.1}"))),
                        ("left", &locale.number(format!("{:.6}", still_dissolved.0))),
                    ],
                ),
                _ => format!(
                    "{vessel}: {dye} on {solid}, held {:.6} mol at {loading_mg_per_g:.2} mg/g, {:.6} mol still dissolved. Boundary: {boundary}",
                    held.0, still_dissolved.0
                ),
            }
        }
        Event::Consumed {
            vessel,
            species: sid,
            moles,
            remaining,
        } => {
            let name = species_name(locale, sid);
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
                2 => locale.fill(
                    "event.consumed.lv2",
                    "{vessel}: {moles} mol {name} consumed",
                    &[
                        ("vessel", &vessel.to_string()),
                        ("moles", &locale.number(quantity(moles.0, 4))),
                        ("name", name),
                    ],
                ),
                _ => format!("{vessel}: −{} mol {name}", quantity(moles.0, 6)),
            }
        }
        Event::Ignited {
            vessel,
            flame,
            energy_j,
        } => match register.level() {
            1 => match flame {
                Some(colour) => {
                    // The flame colour is the observation. It goes through
                    // the same table the flame test uses, or the German
                    // sentence ends in an English word.
                    let colour = locale
                        .lookup(&format!("flame.{colour}"))
                        .unwrap_or(colour.as_str());
                    locale.fill(
                        "event.ignited.lv1-catches-fire-burning",
                        "It catches fire in {vessel} — burning with {colour} light!",
                        &[("vessel", &vessel.to_string()), ("colour", colour)],
                    )
                }
                None => locale.fill(
                    "event.ignited.lv1",
                    "It catches fire in {vessel}!",
                    &[("vessel", &vessel.to_string())],
                ),
            },
            2 => {
                // Both halves of this line were composed in English and
                // then dropped into a German template, so the sentence
                // read "entzündet — orange-red flame". lv1 already routes
                // the colour through the flame table; every level does
                // now, and the noun beside it is a key of its own.
                let colour = flame
                    .as_ref()
                    .map(|colour| {
                        locale.fill(
                            "event.ignited.lv2-flame",
                            " — {colour} flame",
                            &[(
                                "colour",
                                locale
                                    .lookup(&format!("flame.{colour}"))
                                    .unwrap_or(colour.as_str()),
                            )],
                        )
                    })
                    .unwrap_or_default();
                let energy = energy_j
                    .map(|joules| {
                        locale.fill(
                            "event.ignited.lv2-energy",
                            " · {kilojoules} kJ released",
                            &[(
                                "kilojoules",
                                &locale.number(format!("{:.2}", joules / 1000.0)),
                            )],
                        )
                    })
                    .unwrap_or_default();
                locale.fill(
                    "event.ignited.lv1-ignited",
                    "{vessel}: ignited{colour}{energy}",
                    &[("vessel", &vessel.to_string()), ("colour", &colour.to_string()), ("energy", &energy.to_string())],
                )
            }
            _ => match energy_j {
                Some(joules) => locale.fill(
                    "event.ignited.lv3-ignition-source-applied",
                    "{vessel}: ignition source applied; computed reaction energy = {joules} J",
                    &[("vessel", &vessel.to_string()), ("joules", &locale.number(format!("{:.3}", joules)))],
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
            let name = species_name(locale, sid);
            // The colour is the result. A German frame around an English
            // colour loses the one word the reader needed.
            let colour = locale
                .lookup(&format!("flame.{colour}"))
                .unwrap_or(colour.as_str());
            match register.level() {
                1 => locale.fill(
                    "event.flame-test.lv1",
                    "It does not catch fire — but look: it turns the flame {colour}! Every metal has its own colour, which is how you can tell them apart.",
                    &[("colour", colour)],
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
                    &[("vessel", &vessel.to_string()), ("name", name), ("colour", colour)],
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
        Event::FlameStarved {
            vessel,
            fuel,
            burned,
            oxygen_fraction,
        } => {
            let name = species_name(locale, fuel);
            let percent = locale.number(format!("{:.0}", oxygen_fraction * 100.0));
            // Nothing burned at all is a different story from a flame
            // that ran itself out, and the learner is doing a different
            // experiment in each case: one smothered the flame before it
            // started, the other watched it use the jar up.
            let never_caught = burned.0 <= crate::OBSERVABLE_MOLES;
            match (register.level(), never_caught) {
                (1, true) => locale.fill(
                    "event.flame-starved.lv1-never-caught",
                    "The flame will not take hold in {vessel} at all. There IS oxygen in there — but not enough of it, because something else has taken up the room.",
                    &[("vessel", &vessel.to_string())],
                ),
                (1, false) => locale.fill(
                    "event.flame-starved.lv1",
                    "The flame in {vessel} shrinks and goes out — and there is still air in there, and still something left to burn. A flame needs air that is RICH in oxygen; a little is not enough.",
                    &[("vessel", &vessel.to_string())],
                ),
                (2, true) => locale.fill(
                    "event.flame-starved.lv2-never-caught",
                    "{vessel}: the {name} never caught — only {percent}% of the gas is oxygen, and a flame needs about 16%",
                    &[("vessel", &vessel.to_string()), ("name", name), ("percent", &percent)],
                ),
                (2, false) => locale.fill(
                    "event.flame-starved.lv2",
                    "{vessel}: the {name} stopped burning with {percent}% oxygen still in the gas — a flame goes out long before the last of it is used up",
                    &[("vessel", &vessel.to_string()), ("name", name), ("percent", &percent)],
                ),
                _ => locale.fill(
                    "event.flame-starved.lv3",
                    "{vessel}: flame starved — {burned} mol {name} burned, oxygen mole fraction {fraction}",
                    &[("vessel", &vessel.to_string()), ("name", name), ("burned", &locale.number(format!("{:.6}", burned.0))), ("fraction", &locale.number(format!("{oxygen_fraction:.3}")))],
                ),
            }
        }
        Event::BubbleRide {
            vessel,
            object,
            object_density_g_per_ml,
            liquid_density_g_per_ml,
            lift_gas_fraction,
        } => {
            let percent = locale.number(format!("{:.0}", lift_gas_fraction * 100.0));
            // A rider that needs no gas is not dancing, it is floating,
            // and saying "it rises when the bubbles gather" about
            // something already at the top would be a false observation.
            let floats = *lift_gas_fraction <= 0.0;
            match (register.level(), floats) {
                (1, true) => locale.fill(
                    "event.bubble-ride.lv1-floats",
                    "The {object} in {vessel} does not sink at all — this liquid is heavier than it is, so it simply floats, bubbles or no bubbles.",
                    &[("vessel", &vessel.to_string()), ("object", object)],
                ),
                (1, false) => locale.fill(
                    "event.bubble-ride.lv1",
                    "Bubbles gather on the {object} in {vessel}. It is heavier than the liquid, so it sits at the bottom — until the bubbles clinging to it are worth about {percent} of its own size, and up it goes. At the top they pop, and down it comes again.",
                    &[("vessel", &vessel.to_string()), ("object", object), ("percent", &format!("{percent}%"))],
                ),
                (2, true) => locale.fill(
                    "event.bubble-ride.lv2-floats",
                    "{vessel}: the {object} floats unaided — {density} g/mL against a liquid at {liquid} g/mL",
                    &[("vessel", &vessel.to_string()), ("object", object), ("density", &locale.number(format!("{object_density_g_per_ml:.2}"))), ("liquid", &locale.number(format!("{liquid_density_g_per_ml:.2}")))],
                ),
                (2, false) => locale.fill(
                    "event.bubble-ride.lv2",
                    "{vessel}: the {object} sinks at {density} g/mL in a liquid at {liquid} g/mL, and attached bubbles worth {percent}% of its own volume would lift it",
                    &[("vessel", &vessel.to_string()), ("object", object), ("density", &locale.number(format!("{object_density_g_per_ml:.2}"))), ("liquid", &locale.number(format!("{liquid_density_g_per_ml:.2}"))), ("percent", &percent)],
                ),
                _ => locale.fill(
                    "event.bubble-ride.lv3",
                    "{vessel}: bubble-riding — object {density} g/mL, liquid {liquid} g/mL, lift at attached gas fraction {fraction}; no period, bubble size or nucleation-site count is modelled",
                    &[("vessel", &vessel.to_string()), ("density", &locale.number(format!("{object_density_g_per_ml:.3}"))), ("liquid", &locale.number(format!("{liquid_density_g_per_ml:.3}"))), ("fraction", &locale.number(format!("{lift_gas_fraction:.3}")))],
                ),
            }
        }
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
            2 => locale.fill(
                "event.thermal-equilibrium.lv2",
                "{vessel}: thermal equilibrium at {temperature} °C",
                &[("vessel", &vessel.to_string()), ("temperature", &locale.number(format!("{:.0}", temperature.to_celsius())))],
            ),
            _ => locale.fill(
                "event.thermal-equilibrium.lv3",
                "{vessel}: Gibbs minimum at {temperature} K · {provenance} · {provenance2}",
                &[("vessel", &vessel.to_string()), ("temperature", &locale.number(format!("{:.2}", temperature.0))), ("provenance", &provenance.dataset.to_string()), ("provenance2", &provenance.model.to_string())],
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
            1 => locale.fill(
                "event.observed.lv1",
                "You look closely at {vessel}. {appearance}",
                &[("vessel", &vessel.to_string()), ("appearance", &appearance.words.to_string())],
            ),
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
        // EXP-33. The refusals are the lesson, so they get the same care as
        // the number: a mixture is told what is in the way, and a sample
        // that decomposes is told that it decomposes.
        Event::TransitionPointRead { vessel, reading } => {
            use crate::instrument::PurityVerdict as V;
            let what = locale
                .lookup(&format!("transition.{}", reading.kind.as_str()))
                .unwrap_or(reading.kind.as_str());
            let name = |s: &crate::SpeciesId| {
                species_name(locale, s).to_string()
            };
            let subject = reading.species.as_ref().map(name).unwrap_or_default();
            let listed = reading
                .components
                .iter()
                .map(name)
                .collect::<Vec<_>>()
                .join(", ");
            match (&reading.verdict, register.level()) {
                (V::Pure, level) => {
                    let value = reading.value_c.unwrap_or_default();
                    let outcome = reading
                        .outcome
                        .map(|o| o.as_str())
                        .unwrap_or("changes state");
                    let outcome = locale
                        .lookup(&format!("transition.outcome.{outcome}"))
                        .unwrap_or(outcome);
                    match level {
                        1 => locale.fill(
                            "event.transition-point-read.lv1",
                            "The sample from {vessel} {outcome} at {value} °C. Every pure substance has its own, always the same — which is how you tell one white powder from another.",
                            &[("vessel", &vessel.to_string()), ("outcome", outcome), ("value", &locale.number(format!("{value:.1}")))],
                        ),
                        2 => locale.fill(
                            "event.transition-point.lv2",
                            "{vessel} {what}: {subject} {outcome} at {value} °C (± 0.5), sharp — one substance",
                            &[("vessel", &vessel.to_string()), ("what", what), ("subject", &subject), ("outcome", outcome), ("value", &locale.number(format!("{value:.1}")))],
                        ),
                        _ => {
                            let source = reading.source.as_deref().unwrap_or("uncited");
                            let boundary = reading.boundary.as_deref().unwrap_or("");
                            locale.fill(
                                "event.transition-point.lv3",
                                "{vessel} {what}: {subject} {outcome} at {value} °C ± 0.5 (sharp: one substance above the {trace} mole-fraction purity threshold). Source: {source}. Boundary: {boundary}",
                                &[("vessel", &vessel.to_string()), ("what", what), ("subject", &subject), ("outcome", outcome), ("value", &locale.number(format!("{value:.2}"))), ("trace", &locale.number(format!("{:.0e}", crate::instrument::PURITY_TRACE_FRACTION))), ("source", source), ("boundary", boundary)],
                            )
                        }
                    }
                }
                (V::Mixture, level) => {
                    let bound = reading
                        .lowest_component_c
                        .map(|c| locale.number(format!("{c:.1}")))
                        .unwrap_or_else(|| "—".to_string());
                    match level {
                        1 => locale.fill(
                            "event.transition-point.mixture.lv1",
                            "It will not melt at one temperature — it goes soft and mushy over a whole range. That is what a mixture always does, and it is how you know this sample is not pure.",
                            &[],
                        ),
                        2 => locale.fill(
                            "event.transition-point.mixture.lv2",
                            "{vessel} {what}: no sharp point — the sample is a mixture ({listed}). It softens and melts over a range, beginning below {bound} °C",
                            &[("vessel", &vessel.to_string()), ("what", what), ("listed", &listed), ("bound", &bound)],
                        ),
                        _ => locale.fill(
                            "event.transition-point.mixture.lv3",
                            "{vessel} {what}: refused — {listed} are all present above the purity threshold, so no sharp transition exists to report. {boundary}",
                            &[("vessel", &vessel.to_string()), ("what", what), ("listed", &listed), ("boundary", reading.boundary.as_deref().unwrap_or(""))],
                        ),
                    }
                }
                (V::NotIsolated, level) => match level {
                    1 => locale.fill(
                        "event.transition-point.wet.lv1",
                        "The sample is still wet. Dry it first — a damp sample melts low and messily, and the reading would be a lie.",
                        &[],
                    ),
                    2 => locale.fill(
                        "event.transition-point.wet.lv2",
                        "{vessel} {what}: the sample is not isolated — something of another phase shares the vessel with the {subject}. Dry or separate it first",
                        &[("vessel", &vessel.to_string()), ("what", what), ("subject", &subject)],
                    ),
                    _ => locale.fill(
                        "event.transition-point.wet.lv3",
                        "{vessel} {what}: refused — {subject} is the only substance of the wanted phase, but matter of another phase is present, so this is a wet or dissolved sample rather than an isolated one. {boundary}",
                        &[("vessel", &vessel.to_string()), ("what", what), ("subject", &subject), ("boundary", reading.boundary.as_deref().unwrap_or(""))],
                    ),
                },
                (V::NoData, level) => match level {
                    1 => locale.fill(
                        "event.transition-point.nodata.lv1",
                        "Nobody has written this one down in the lab's book yet, so the apparatus has nothing to tell you.",
                        &[],
                    ),
                    _ => locale.fill(
                        "event.transition-point.nodata.lv2",
                        "{vessel} {what}: the sample is one substance ({subject}), but the registry carries no {what} for it — no value is invented",
                        &[("vessel", &vessel.to_string()), ("what", what), ("subject", &subject)],
                    ),
                },
                (V::NothingToTest, level) => match level {
                    1 => locale.fill(
                        "event.transition-point.empty.lv1",
                        "There is nothing in the tube to melt.",
                        &[],
                    ),
                    _ => locale.fill(
                        "event.transition-point.empty.lv2",
                        "{vessel} {what}: nothing of the required phase is in the vessel",
                        &[("vessel", &vessel.to_string()), ("what", what)],
                    ),
                },
            }
        }
        Event::Dehydrated {
            vessel,
            hydrate,
            anhydrous,
            formula_units,
            water,
            at,
        } => {
            let hname = species_name(locale, hydrate);
            let aname = species_name(locale, anhydrous);
            let water_g = species::lookup(&crate::SpeciesId::new("water"))
                .map(|d| water.0 * d.molar_mass)
                .unwrap_or(0.0);
            match register.level() {
                1 => locale.fill(
                    "event.dehydrated.lv1",
                    "The blue crystals turn white as the water is driven out of them. The water was never wet — it was locked inside the crystal, and now it has gone off as steam.",
                    &[],
                ),
                2 => locale.fill(
                    "event.dehydrated.lv2",
                    "{vessel}: {hydrate} → {anhydrous} + water, driven off at {at} °C — {water_g} g of water left the crucible",
                    &[("vessel", &vessel.to_string()), ("hydrate", hname), ("anhydrous", aname), ("at", &locale.number(format!("{:.0}", at.to_celsius()))), ("water_g", &locale.number(format!("{water_g:.4}")))],
                ),
                _ => locale.fill(
                    "event.dehydrated.lv3",
                    "{vessel}: dehydration at {at} K — {units} mol {hydrate} → {units} mol {anhydrous} + {water} mol H2O ({water_g} g). One step, to the anhydrous salt: the intermediate hydrates are real and are not modelled here",
                    &[("vessel", &vessel.to_string()), ("at", &locale.number(format!("{:.1}", at.0))), ("units", &locale.number(format!("{:.6}", formula_units.0))), ("hydrate", hname), ("anhydrous", aname), ("water", &locale.number(format!("{:.6}", water.0))), ("water_g", &locale.number(format!("{water_g:.4}")))],
                ),
            }
        }
        Event::Hydrated {
            vessel,
            anhydrous,
            hydrate,
            formula_units,
            water,
        } => {
            let hname = species_name(locale, hydrate);
            let aname = species_name(locale, anhydrous);
            match register.level() {
                1 => locale.fill(
                    "event.hydrated.lv1",
                    "A drop of water and the white powder goes blue again. The water has gone back into the crystal, exactly as much as came out.",
                    &[],
                ),
                2 => locale.fill(
                    "event.hydrated.lv2",
                    "{vessel}: {anhydrous} + water → {hydrate}; {water} mol of water taken back into the crystal",
                    &[("vessel", &vessel.to_string()), ("anhydrous", aname), ("hydrate", hname), ("water", &locale.number(format!("{:.6}", water.0)))],
                ),
                _ => locale.fill(
                    "event.hydrated.lv3",
                    "{vessel}: rehydration — {units} mol {anhydrous} + {water} mol H2O → {units} mol {hydrate}. Fired because the free water present is within the crystal's own stoichiometric demand; with more water than that, dissolution is the honest answer and the aqueous engine owns it",
                    &[("vessel", &vessel.to_string()), ("units", &locale.number(format!("{:.6}", formula_units.0))), ("anhydrous", aname), ("water", &locale.number(format!("{:.6}", water.0))), ("hydrate", hname)],
                ),
            }
        }
        Event::Measured {
            vessel,
            instrument,
            value,
            unit,
        } => {
            let device = instrument_name(*instrument);
            // The English name is the source text and the fallback; German
            // comes from the catalogue, keyed by that name. An instrument
            // nobody has translated reads in English inside a German
            // sentence rather than disappearing from it.
            let device = locale
                .lookup(&format!("instrument.{device}"))
                .unwrap_or(device);
            // KID-19a: a reading rounded past its own resolution is not a
            // simplification, it is a wrong number. Aluminium at 2.7 g/mL
            // and copper at 8.96 are the entire content of the density
            // row, and "3" and "9" are not — while 25 °C gains nothing
            // from a decimal. So the reading keeps one below ten and none
            // above, which is roughly how a person reads a real dial.
            let plain = if value.abs() < 10.0 {
                format!("{value:.1}")
            } else {
                format!("{value:.0}")
            };
            match register.level() {
                1 => locale.fill(
                    "event.measured.lv1",
                    "The {device} on {vessel} reads {value} {unit}.",
                    &[("device", device), ("vessel", &vessel.to_string()), ("value", &locale.number(plain)), ("unit", unit)],
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
            amps,
            seconds,
            coulombs,
            electrons,
            moles,
            grams,
            per_ion,
        } => {
            let name = species_name(locale, species);
            match register.level() {
                1 => locale.fill(
                    "event.electrolysed.lv1",
                    "{grams} g of {name} builds up on the electrode in {vessel}.",
                    &[("grams", &locale.number(format!("{grams:.2}"))), ("name", name), ("vessel", &vessel.to_string())],
                ),
                2 => locale.fill(
                    "event.electrolysed.lv2",
                    "{vessel}: {amps} A for {seconds} s = {coulombs} C → {electrons} mol e⁻ → {moles} mol {name} = {grams} g",
                    &[
                        ("vessel", &vessel.to_string()),
                        ("amps", &locale.number(format!("{amps:.3}"))),
                        ("seconds", &locale.number(format!("{seconds:.0}"))),
                        ("coulombs", &locale.number(format!("{coulombs:.0}"))),
                        ("electrons", &locale.number(format!("{:.4}", electrons.0))),
                        ("moles", &locale.number(format!("{:.4}", moles.0))),
                        ("name", name),
                        ("grams", &locale.number(format!("{grams:.3}"))),
                    ],
                ),
                // The chain, with the one step that is chemistry rather
                // than arithmetic marked: everything else is division.
                _ => locale.fill(
                    "event.electrolysed.lv3",
                    "{vessel}: I = {amps} A; t = {seconds} s; Q = It = {coulombs} C; n(e⁻) = Q/F = {electrons} mol; n({name}) = n(e⁻)/{per_ion} = {moles} mol; m = {grams} g — only the {per_ion} is chemistry. Inert anode assumed: the water is oxidised there, so the oxygen leaves and the acid stays",
                    &[
                        ("vessel", &vessel.to_string()),
                        ("amps", &locale.number(format!("{amps:.6}"))),
                        ("seconds", &locale.number(format!("{seconds:.3}"))),
                        ("coulombs", &locale.number(format!("{coulombs:.1}"))),
                        ("electrons", &locale.number(format!("{:.6}", electrons.0))),
                        ("name", name),
                        ("per_ion", &locale.number(format!("{per_ion:.0}"))),
                        ("moles", &locale.number(format!("{:.6}", moles.0))),
                        ("grams", &locale.number(format!("{grams:.4}"))),
                    ],
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
        Event::AcidMetalCellVoltage { anode, cathode, volts, ph } => match register.level() {
            1 => locale.fill(
                "event.acid-metal-cell-voltage.lv1",
                "The zinc and copper in acid offer about {volts} V — electrons would flow from zinc at {anode} to the copper surface at {cathode}. This is a voltmeter estimate, not a promise about how much current a lemon can deliver.",
                &[("volts", &locale.number(format!("{volts:.2}"))), ("anode", &anode.to_string()), ("cathode", &cathode.to_string())],
            ),
            _ => locale.fill(
                "event.acid-metal-cell-voltage.lv2",
                "Zn | Zn²⁺(unit-activity estimate) ‖ H⁺(pH {ph}) | H₂ on Cu: E ≈ {volts} V open-circuit; zinc is the anode at {anode}, copper is the inert hydrogen-evolution surface at {cathode}. The zinc-ion activity was not measured, so this is a bounded teaching estimate; internal resistance, current, power and lifetime are not modeled",
                &[("volts", &locale.number(format!("{volts:.3}"))), ("ph", &locale.number(format!("{ph:.2}"))), ("anode", &anode.to_string()), ("cathode", &cathode.to_string())],
            ),
        },
        Event::NoCell { a, b, why } => match register.level() {
            // KID-21: say what a half-cell is, since that is the thing the
            // learner is missing. A beaker of acid with a metal in it is not
            // one; a metal standing in a solution of its *own* ion is.
            1 => locale.fill(
                "event.no-cell.lv1",
                "The voltmeter between {a} and {b} reads nothing — one of them isn't a proper half-cell yet. Each side needs a metal standing in a solution of its own ion: zinc in zinc sulfate, copper in copper sulfate.",
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
            ..
        } => {
            let severity_en = format!("{severity:?}");
            let severity_text = locale
                .lookup(&format!("severity.{severity_en}"))
                .map(str::to_string)
                .unwrap_or(severity_en);
            match register.level() {
            1 => locale.fill(
                "event.hazard-warning.lv1",
                "⚠️  STOP AND READ: {hazard}. {real_world} NEVER try this outside the virtual lab — here, we can watch what happens safely.",
                &[("hazard", &hazard.to_string()), ("real_world", &real_world.to_string())],
            ),
            2 => locale.fill(
                "event.hazard-warning.lv2",
                "⚠ HAZARD ({severity}): {hazard} — {real_world} Safe only because this lab is virtual.",
                &[("severity", &severity_text), ("hazard", &hazard.to_string()), ("real_world", &real_world.to_string())],
            ),
            _ => format!("HAZARD [{severity:?}] (L0): {hazard}; {real_world}"),
            }
        }
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
        // BRD-002: the cabinet speaks. Stocking is quiet bookkeeping at
        // lv1 — a learner who just filled a bottle does not need the
        // number read back — and exact at lv3, where the ledger matters.
        Event::ShelfStocked { key, amount, unit } => match register.level() {
            1 => locale.fill(
                "event.shelf-stocked.lv1",
                "There is now {amount} {unit} of {key} on the shelf.",
                &[("amount", &locale.number(format!("{amount:.4}"))), ("unit", unit.label()), ("key", &key.to_string())],
            ),
            2 => locale.fill(
                "event.shelf-stocked.lv2",
                "shelf: {key} stocked at {amount} {unit}",
                &[("key", &key.to_string()), ("amount", &locale.number(format!("{amount:.4}"))), ("unit", unit.label())],
            ),
            _ => locale.fill(
                "event.shelf-stocked.lv3",
                "SHELF STOCK: {key} = {amount} {unit}",
                &[("key", &key.to_string()), ("amount", &locale.number(format!("{amount:.6}"))), ("unit", unit.label())],
            ),
        },
        // The refusal names both numbers at every level. "There isn't
        // enough" without saying how much is left is the kind of honesty
        // that still leaves the reader stuck.
        Event::StockExhausted {
            key,
            requested,
            remaining,
            unit,
        } => match register.level() {
            1 => locale.fill(
                "event.stock-exhausted.lv1",
                "There isn't enough {key} left — the bottle holds {remaining} {unit} and that needed {requested} {unit}. Nothing was poured.",
                &[("key", &key.to_string()), ("remaining", &locale.number(format!("{remaining:.4}"))), ("unit", unit.label()), ("requested", &locale.number(format!("{requested:.4}")))],
            ),
            2 => locale.fill(
                "event.stock-exhausted.lv2",
                "shelf: not enough {key} — {remaining} {unit} left, {requested} {unit} asked for; nothing was taken",
                &[("key", &key.to_string()), ("remaining", &locale.number(format!("{remaining:.4}"))), ("unit", unit.label()), ("requested", &locale.number(format!("{requested:.4}")))],
            ),
            _ => locale.fill(
                "event.stock-exhausted.lv3",
                "STOCK REFUSED: {key} remaining {remaining} {unit} < requested {requested} {unit}; bench unchanged",
                &[("key", &key.to_string()), ("remaining", &locale.number(format!("{remaining:.6}"))), ("unit", unit.label()), ("requested", &locale.number(format!("{requested:.6}")))],
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
            let name = species_name(locale, sid);
            let toxic = data
                .and_then(|d| d.appearance)
                .is_some_and(|a| a.contains("toxic"));
            match register.level() {
                1 => {
                    if toxic {
                        locale.fill(
                            "event.gas-evolved.lv1-toxic",
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
                    locale.fill(
                        "event.gas-evolved.lv2",
                        "{vessel}: {moles} mol {name} ↑ (gas escapes the open vessel)",
                        &[("vessel", &vessel.to_string()), ("moles", &locale.number(format!("{:.4}", moles.0))), ("name", name)],
                    )
                }
                _ => {
                    locale.fill(
                        "event.gas-evolved.lv3",
                        "{vessel}: {moles} mol {name} evolved (open system; mass leaves)",
                        &[("vessel", &vessel.to_string()), ("moles", &locale.number(format!("{:.6}", moles.0))), ("name", name)],
                    )
                }
            }
        }
        Event::GasAbsorbed {
            vessel,
            species: sid,
            moles,
        } => {
            let name = species_name(locale, sid);
            match register.level() {
                1 => locale.fill(
                    "event.gas-absorbed.lv1",
                    "Gas bubbles into {vessel} and is taken up by the liquid.",
                    &[("vessel", &vessel.to_string())],
                ),
                2 => locale.fill(
                    "event.gas-absorbed.lv2",
                    "{vessel}: {moles} mol {name} absorbed from the gas boundary",
                    &[("vessel", &vessel.to_string()), ("moles", &locale.number(format!("{:.4}", moles.0))), ("name", name)],
                ),
                _ => locale.fill(
                    "event.gas-absorbed.lv3",
                    "{vessel}: {moles} mol {name} transferred gas → condensed inventory",
                    &[("vessel", &vessel.to_string()), ("moles", &locale.number(format!("{:.6}", moles.0))), ("name", name)],
                ),
            }
        }
        Event::GasContained {
            vessel,
            species: sid,
            moles,
        } => {
            let name = species_name(locale, sid);
            match register.level() {
                1 => locale.fill(
                    "event.gas-contained.lv1",
                    "Bubbles form in {vessel}, but the gas stays inside.",
                    &[("vessel", &vessel.to_string())],
                ),
                2 => locale.fill(
                    "event.gas-contained.lv2",
                    "{vessel}: {moles} mol {name} formed and remains in the closed headspace",
                    &[("vessel", &vessel.to_string()), ("moles", &locale.number(format!("{:.4}", moles.0))), ("name", name)],
                ),
                _ => locale.fill(
                    "event.gas-contained.lv3",
                    "{vessel}: {moles} mol {name} transferred to the finite headspace (closed system; mass retained)",
                    &[("vessel", &vessel.to_string()), ("moles", &locale.number(format!("{:.6}", moles.0))), ("name", name)],
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
            2 => locale.fill(
                "event.vessel-sealed.lv2",
                "{vessel}: sealed over {headspace_volume} L of headspace, trapping {trapped_air} mol of room air",
                &[("vessel", &vessel.to_string()), ("headspace_volume", &locale.number(format!("{:.3}", headspace_volume.0))), ("trapped_air", &locale.number(format!("{:.4}", trapped_air.0)))],
            ),
            _ => locale.fill(
                "event.vessel-sealed.lv3",
                "{vessel}: boundary=open → sealed; V_gas={headspace_volume} L, trapped dry-air approximation={trapped_air} mol",
                &[("vessel", &vessel.to_string()), ("headspace_volume", &locale.number(format!("{:.6}", headspace_volume.0))), ("trapped_air", &locale.number(format!("{:.8}", trapped_air.0)))],
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
            2 => locale.fill(
                "event.vessel-pressure-controlled.lv2",
                "{vessel}: pressure controlled at {pressure} bar; initial headspace {initial_volume} L",
                &[("vessel", &vessel.to_string()), ("pressure", &locale.number(format!("{:.3}", pressure.0 / 100_000.0))), ("initial_volume", &locale.number(format!("{:.3}", initial_volume.0)))],
            ),
            _ => locale.fill(
                "event.vessel-pressure-controlled.lv3",
                "{vessel}: boundary=pressure_controlled; P={pressure} Pa, V_initial={initial_volume} L, trapped gas={trapped_gas} mol",
                &[("vessel", &vessel.to_string()), ("pressure", &locale.number(format!("{:.3}", pressure.0))), ("initial_volume", &locale.number(format!("{:.6}", initial_volume.0))), ("trapped_gas", &locale.number(format!("{:.8}", trapped_gas.0)))],
            ),
        },
        Event::VesselSwept { vessel, pressure } => match register.level() {
            1 => locale.fill(
                "event.vessel-swept.lv1",
                "Nitrogen flows across {vessel} and carries gases away.",
                &[("vessel", &vessel.to_string())],
            ),
            2 => locale.fill(
                "event.vessel-swept.lv2",
                "{vessel}: swept by nitrogen at {pressure} bar; volatile products are purged",
                &[("vessel", &vessel.to_string()), ("pressure", &locale.number(format!("{:.3}", pressure.0 / 100_000.0)))],
            ),
            _ => locale.fill(
                "event.vessel-swept.lv3",
                "{vessel}: boundary=swept; inert N2 purge at P={pressure} Pa, gas inventory external",
                &[("vessel", &vessel.to_string()), ("pressure", &locale.number(format!("{:.3}", pressure.0)))],
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
            2 => locale.fill(
                "event.headspace-equilibrated.lv2",
                "{vessel}: headspace settled at {pressure} bar with {total_moles} mol gas",
                &[("vessel", &vessel.to_string()), ("pressure", &locale.number(format!("{:.3}", pressure.0 / 100_000.0))), ("total_moles", &locale.number(format!("{:.4}", total_moles.0)))],
            ),
            _ => locale.fill(
                "event.headspace-equilibrated.lv3",
                "{vessel}: finite-volume gas/liquid equilibrium; P={pressure} Pa, n_gas={total_moles} mol",
                &[("vessel", &vessel.to_string()), ("pressure", &locale.number(format!("{:.3}", pressure.0))), ("total_moles", &locale.number(format!("{:.8}", total_moles.0)))],
            ),
        },
        Event::BoilingPointRouted {
            vessel,
            species,
            pressure_kpa,
            boiling,
            shifted_by,
            route,
            model,
        } => {
            let name = species_name(locale, species);
            let c = boiling.to_celsius();
            let pressure = locale.number(format!("{pressure_kpa:.1}"));
            let direction = if *shifted_by < 0.0 {
                locale.t("shifted.lower", "lower")
            } else {
                locale.t("shifted.higher", "higher")
            };
            let route_name = locale
                .lookup(&format!("boiling-route.{}", route.as_str()))
                .unwrap_or(route.as_str());
            match register.level() {
                1 if route.routed() => locale.fill(
                    "event.boiling-point-routed.lv1",
                    "The pressure over {vessel} is not ordinary air pressure, so the {name} boils at {c} °C.",
                    &[("vessel", &vessel.to_string()), ("name", name), ("c", &locale.number(format!("{c:.0}")))],
                ),
                1 => locale.fill(
                    "event.boiling-point-routed.lv1-unrouted",
                    "The pressure over {vessel} is outside what this lab has measurements for, so the {name} still boils at {c} °C.",
                    &[("vessel", &vessel.to_string()), ("name", name), ("c", &locale.number(format!("{c:.0}")))],
                ),
                2 if route.routed() => locale.fill(
                    "event.boiling-point-routed.lv2",
                    "{vessel}: at {pressure} kPa the {name} boils at {c} °C — {shifted_by} °C {direction} than at one atmosphere",
                    &[("vessel", &vessel.to_string()), ("pressure", &pressure), ("name", name), ("c", &locale.number(format!("{c:.1}"))), ("shifted_by", &locale.number(format!("{:.1}", shifted_by.abs()))), ("direction", direction)],
                ),
                2 => locale.fill(
                    "event.boiling-point-routed.lv2-unrouted",
                    "{vessel}: at {pressure} kPa no cleared measurements cover the {name}, so its one-atmosphere boiling point of {c} °C stands",
                    &[("vessel", &vessel.to_string()), ("pressure", &pressure), ("name", name), ("c", &locale.number(format!("{c:.1}")))],
                ),
                _ => locale.fill(
                    "event.boiling-point-routed.lv3",
                    "{vessel}: {name} boiling point {boiling} K at {pressure} kPa; route {route}; model {model}; pressure shift {shifted_by} K",
                    &[("vessel", &vessel.to_string()), ("name", name), ("boiling", &locale.number(format!("{:.3}", boiling.0))), ("pressure", &pressure), ("route", route_name), ("model", model.as_str()), ("shifted_by", &locale.number(format!("{shifted_by:+.3}")))],
                ),
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
            let name = species_name(locale, species);
            let verb_en = phase_change_verb(*from, *to);
            let verb = locale
                .lookup(&format!("verb.{verb_en}"))
                .unwrap_or(verb_en);
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
                        locale.fill(
                            "event.state-changed.lv2",
                            "{vessel}: {name} {verb} at {c} °C — {shifted_by} °C {direction} than pure {name}, because of what is dissolved in it",
                            &[("vessel", &vessel.to_string()), ("name", name), ("verb", verb), ("c", &locale.number(format!("{c:.1}"))), ("shifted_by", &locale.number(format!("{:.1}", shifted_by.abs()))), ("direction", (if *shifted_by < 0.0 { locale.t("shifted.lower", "lower") } else { locale.t("shifted.higher", "higher") }))],
                        )
                    }
                }
                _ => locale.fill(
                    "event.state-changed.lv3",
                    "{vessel}: {name} {from} → {to} at {at} K ({c} °C), shifted {shifted_by} K by dissolved particles",
                    &[("vessel", &vessel.to_string()), ("name", name), ("from", &locale.number(format!("{from:?}"))), ("to", &locale.number(format!("{to:?}"))), ("at", &locale.number(format!("{:.2}", at.0))), ("c", &locale.number(format!("{c:.2}"))), ("shifted_by", &locale.number(format!("{shifted_by:+.3}")))],
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
                locale.fill(
                    "event.reacted.lv2-mol-reacted",
                    "{vessel}: {moles} mol reacted in {seconds} s{with}  —  {equation}",
                    &[("vessel", &vessel.to_string()), ("moles", &locale.number(format!("{:.4}", moles.0))), ("seconds", &locale.number(format!("{seconds:.0}"))), ("with", &with.to_string()), ("equation", &equation.to_string())],
                )
            }
            _ => {
                let with = match catalyst {
                    Some(c) => format!(" (catalyst: {c})"),
                    None => String::new(),
                };
                locale.fill(
                    "event.reacted.lv3",
                    "{vessel}: extent {moles} mol over {seconds} s, Ea = {activation_energy} kJ/mol{with}  —  {equation}",
                    &[("vessel", &vessel.to_string()), ("moles", &locale.number(format!("{:.6}", moles.0))), ("seconds", &locale.number(format!("{seconds:.1}"))), ("activation_energy", &locale.number(format!("{:.1}", activation_energy / 1000.0))), ("with", &with.to_string()), ("equation", &equation.to_string())],
                )
            }
        },
        Event::NotYetModeled { vessel, what, .. } => {
            // `what` is English composed in bench.rs and carried in the
            // event, so a German frame was wrapping an English reason:
            // "v1: noch nicht modelliert — nothing to evaporate". Looked
            // up by value, the same way species and instrument names are.
            //
            // Only the reasons with no interpolated value can match this
            // way. The rest need the EVENT to carry a key and its
            // arguments rather than a finished sentence, which is a change
            // to the wire format and a separate decision.
            let what = &locale
                .lookup(&format!("refusal.{what}"))
                .map(str::to_string)
                .unwrap_or_else(|| what.clone());
            match register.level() {
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
            }
        }
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
            2 => locale.fill(
                "event.diluted.lv2",
                "{vessel}: diluted with {volume} mL water ({moles} mol)",
                &[("vessel", &vessel.to_string()), ("volume", &locale.number(format!("{:.1}", volume.0 * 1000.0))), ("moles", &locale.number(format!("{:.4}", moles.0)))],
            ),
            _ => locale.fill(
                "event.diluted.lv3",
                "{vessel}: +{moles} mol H₂O from {volume} L dilution water",
                &[("vessel", &vessel.to_string()), ("moles", &locale.number(format!("{:.6}", moles.0))), ("volume", &locale.number(format!("{:.4}", volume.0)))],
            ),
        },
        Event::Titrated {
            vessel,
            titrant,
            concentration,
            steps,
            total_volume,
            final_ph,
            pe_curve,
            endpoint_reached,
            endpoint,
            ..
        } if !endpoint.is_ph() => {
            // EXP-39. A redox titration is not narrated as "the pH
            // reaches 7": the number a reader wants is the potential or
            // the colour, and the pH is a bystander. It is still printed
            // at lv3, because the endpoint of a permanganate titration
            // depends on the flask staying acidic and a reader checking
            // the work needs to see that it did.
            let name = species_name(locale, titrant);
            // A noun phrase in both languages, so the three sentences
            // below can slot it in without either one fighting the other
            // language's word order.
            let arrival = match endpoint {
                crate::ops::Endpoint::Pe { compare, value } => locale.fill(
                    "event.titrated.endpoint.pe",
                    "pe {compare} {value}",
                    &[("compare", compare.symbol()), ("value", &locale.number(format!("{value}")))],
                ),
                _ => locale.fill(
                    "event.titrated.endpoint.colour",
                    "a {name} colour that stays",
                    &[("name", name)],
                ),
            };
            let final_pe = match pe_curve.last() {
                Some(&(_, pe)) => locale.number(format!("{pe:.2}")),
                None => locale
                    .t(
                        "event.titrated.no_potential",
                        "undefined — no potential was pinned",
                    )
                    .to_string(),
            };
            let outcome = if endpoint_reached.unwrap_or(false) {
                locale.t("event.titrated.reached", "endpoint reached")
            } else {
                locale.t("event.titrated.not_reached", "endpoint NOT reached")
            };
            match register.level() {
                1 => locale.fill(
                    "event.titrated.redox.lv1",
                    "You titrate {vessel} with {name} until you get {arrival} — that took {steps} additions.",
                    &[("vessel", &vessel.to_string()), ("name", name), ("steps", &steps.to_string()), ("arrival", &arrival)],
                ),
                2 => locale.fill(
                    "event.titrated.redox.lv2",
                    "{vessel}: titrated with {concentration} mol/L {name} to {arrival}; {steps} steps, {total_volume} mL total, {outcome}",
                    &[("vessel", &vessel.to_string()), ("concentration", &concentration.to_string()), ("name", name), ("arrival", &arrival), ("steps", &steps.to_string()), ("total_volume", &locale.number(format!("{:.1}", total_volume.0 * 1000.0))), ("outcome", outcome)],
                ),
                _ => locale.fill(
                    "event.titrated.redox.lv3",
                    "{vessel}: auto-titration with {titrant} standard solution ({concentration} mol/L; {steps} steps, {total_volume} mL cumulative = {delivered} mol delivered with its carrier water); endpoint = {arrival}, {outcome}; final pe {final_pe}, final pH {final_ph}",
                    &[("vessel", &vessel.to_string()), ("titrant", &titrant.0.to_string()), ("concentration", &concentration.to_string()), ("steps", &steps.to_string()), ("total_volume", &locale.number(format!("{:.3}", total_volume.0 * 1000.0))), ("delivered", &locale.number(format!("{:.5}", concentration * total_volume.0))), ("arrival", &arrival), ("outcome", outcome), ("final_pe", &final_pe), ("final_ph", &locale.number(format!("{final_ph:.3}")))],
                ),
            }
        }
        Event::Titrated {
            vessel,
            titrant,
            concentration,
            steps,
            total_volume,
            final_ph,
            ..
        } => {
            let name = species_name(locale, titrant);
            match register.level() {
                1 => locale.fill(
                    "event.titrated.lv1",
                    "You titrate {vessel} with {name} — after {steps} additions the pH reaches {final_ph}.",
                    &[("vessel", &vessel.to_string()), ("name", name), ("steps", &steps.to_string()), ("final_ph", &locale.number(format!("{final_ph:.1}")))],
                ),
                2 => locale.fill(
                    "event.titrated.lv2",
                    "{vessel}: titrated with {concentration} mol/L {name}; {steps} steps, {total_volume} mL total, final pH {final_ph}",
                    &[("vessel", &vessel.to_string()), ("concentration", &concentration.to_string()), ("name", name), ("steps", &steps.to_string()), ("total_volume", &locale.number(format!("{:.1}", total_volume.0 * 1000.0))), ("final_ph", &locale.number(format!("{final_ph:.2}")))],
                ),
                _ => locale.fill(
                    "event.titrated.lv3",
                    "{vessel}: auto-titration with {titrant} standard solution ({concentration} mol/L; {steps} steps, {total_volume} mL cumulative = {concentration2} mol delivered with its carrier water); final pH {final_ph}",
                    &[("vessel", &vessel.to_string()), ("titrant", &titrant.0.to_string()), ("concentration", &concentration.to_string()), ("steps", &steps.to_string()), ("total_volume", &locale.number(format!("{:.3}", total_volume.0 * 1000.0))), ("concentration2", &locale.number(format!("{:.5}", concentration * total_volume.0))), ("final_ph", &locale.number(format!("{final_ph:.3}")))],
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
                            locale.fill(
                                "event.transported.lv2-portion",
                                "{moles} mol {name}",
                                &[
                                    ("moles", &locale.number(format!("{:.4}", m.0))),
                                    ("name", species_name(locale, s)),
                                ],
                            )
                        })
                        .collect();
                    let what = if species_list.is_empty() {
                        locale.t("chromatographed.solvent-only", "solvent only").to_string()
                    } else {
                        species_list.join(", ")
                    };
                    locale.fill(
                        "event.transported.lv2",
                        "{cells} cells × {steps} steps (Cf={courant}); effluent → {receiver}: {what}",
                        &[
                            ("cells", &cells.to_string()),
                            ("steps", &steps.to_string()),
                            ("courant", &locale.number(format!("{courant:.2}"))),
                            ("receiver", &receiver.to_string()),
                            ("what", &what),
                        ],
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

/// Translate the prose a host renders directly off a serialised event.
///
/// Most engine text reaches the reader through `render_event_in`, which
/// takes a locale. `HazardWarning` does not: the shell reads `hazard` and
/// `real_world` off the event itself, so a German session showed a German
/// frame around English safety prose. These strings are built in
/// `bench.rs` and the safety screen, neither of which knows the locale —
/// it lives in the wasm wrapper — so the translation happens here, on the
/// way out.
///
/// Untranslated strings pass through unchanged. English is a working
/// fallback for a hazard note; a missing one is not.
pub fn localize_event(event: &Event, locale: Locale) -> Event {
    if locale.is_english() {
        return event.clone();
    }
    match event {
        Event::HazardWarning {
            severity,
            rule,
            hazard,
            real_world,
        } => Event::HazardWarning {
            severity: *severity,
            // The rule id is machine identity, not prose: it crosses the
            // locale boundary untranslated so contracts recognise it in
            // every language.
            rule: rule.clone(),
            hazard: localize_hazard(hazard, locale),
            real_world: locale
                .lookup(&format!("real_world.{real_world}"))
                .map(str::to_string)
                .unwrap_or_else(|| real_world.clone()),
        },
        // A spill warns with the same two sentences a vessel does, and the
        // shell reads them straight off the event — so they have to be
        // translated here or not at all. Missing this arm is how
        // `HazardWarning` shipped a German frame around English prose.
        Event::SpillHazard {
            destination,
            severity,
            rule,
            hazard,
            real_world,
            contributors,
        } => Event::SpillHazard {
            destination: destination.clone(),
            severity: *severity,
            rule: rule.clone(),
            hazard: localize_hazard(hazard, locale),
            real_world: locale
                .lookup(&format!("real_world.{real_world}"))
                .map(str::to_string)
                .unwrap_or_else(|| real_world.clone()),
            contributors: contributors.clone(),
        },
        // Translating the sentence must not lose the cause: the German
        // reader is entitled to the same grouping the English one has.
        Event::NotYetModeled {
            vessel,
            what,
            cause,
        } => Event::NotYetModeled {
            vessel: *vessel,
            what: localize_refusal(what, locale),
            cause: *cause,
        },
        other => other.clone(),
    }
}

fn localize_refusal(what: &str, locale: Locale) -> String {
    if let Some(translated) = locale.lookup(&format!("refusal.{what}")) {
        return translated.to_string();
    }

    const CONTACT: &str =
        " in contact with liquid: no wired solver models this dissolution/reaction";
    if let Some(name) = what.strip_suffix(CONTACT) {
        let translated_name = locale.lookup(&format!("species.{name}")).unwrap_or(name);
        return locale.fill(
            "refusal.solid-in-liquid",
            "{name} in contact with liquid: no wired solver models this dissolution/reaction",
            &[("name", translated_name)],
        );
    }

    const UNSPECIATED: &str = " dissolves, but no wired engine speciates it: it contributes nothing to the pH or the ionic strength here, and those numbers are for everything else in the beaker";
    if let Some(name) = what.strip_suffix(UNSPECIATED) {
        let translated_name = locale.lookup(&format!("species.{name}")).unwrap_or(name);
        return locale.fill(
            "refusal.dissolves-without-speciation",
            "{name} dissolves, but no wired engine speciates it: it contributes nothing to the pH or the ionic strength here, and those numbers are for everything else in the beaker",
            &[("name", translated_name)],
        );
    }

    what.to_string()
}

/// As `localize_event`, over a slice.
pub fn localize_events(events: &[Event], locale: Locale) -> Vec<Event> {
    events.iter().map(|e| localize_event(e, locale)).collect()
}

fn localize_hazard(hazard: &str, locale: Locale) -> String {
    // Two tables, tried in turn: the fixed hazards the safety screen and
    // bench raise, then the per-substance vapour sentences. Both are keyed
    // by the exact English, so an untranslated one renders in English
    // rather than vanishing.
    locale
        .lookup(&format!("hazard.{hazard}"))
        .or_else(|| locale.lookup(&format!("hazard_vapour.{hazard}")))
        .map(str::to_string)
        .unwrap_or_else(|| hazard.to_string())
}

#[cfg(test)]
mod dedupe_tests {
    use super::*;
    use crate::vessel::VesselId;

    fn notes() -> Vec<Event> {
        vec![
            Event::NotYetModeled {
                cause: crate::ops::NotModelledCause::NoSolver,
                vessel: VesselId(0),
                what: "one thing".to_string(),
            },
            Event::NotYetModeled {
                cause: crate::ops::NotModelledCause::NoSolver,
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
mod quantity_tests {
    use super::*;
    use crate::units::Moles;
    use crate::vessel::VesselId;

    #[test]
    fn a_real_trace_amount_never_renders_as_zero() {
        let event = Event::Dissolved {
            vessel: VesselId(0),
            species: SpeciesId::new("CaCO3"),
            moles: Moles(4.2e-7),
        };
        assert_eq!(
            render_event(&event, Register::LV2),
            "v1: 4.200e-7 mol chalk (calcium carbonate) dissolved"
        );
        assert_eq!(quantity(0.0, 4), "0.0000");
        assert_eq!(quantity(0.01234, 4), "0.0123");
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
