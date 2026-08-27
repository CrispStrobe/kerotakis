//! Register rendering: one event stream, three voices. Deterministic
//! templates over solver output — never a language model (PLAN.md).
//!
//! The solver has no idea who is asking; this module is the only place that
//! does.

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
        Event::VesselRemoved { vessel } => match register.level() {
            1 => format!("The empty {vessel} goes back into storage."),
            _ => format!("{vessel}: empty vessel removed"),
        },
        Event::Added {
            vessel,
            species: sid,
            moles,
            total_after,
        } => {
            let name = species::lookup(sid).map(|d| d.name).unwrap_or(sid.0.as_str());
            match register.level() {
                1 => format!("You add {name} to {vessel}."),
                2 => match total_after {
                    Some(total) if (total.0 - moles.0).abs() > 1e-12 => format!(
                        "{vessel}: +{:.4} mol {name} — {:.4} mol now in vessel",
                        moles.0, total.0
                    ),
                    _ => format!("{vessel}: +{:.4} mol {name}", moles.0),
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
                1 => format!("You add {material} to {vessel}."),
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
            1 => format!("{vessel}: {species} bubbles are being made."),
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
            1 => format!("{vessel} grows warmer as the reaction runs."),
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
            1 => format!("Foam rises in {vessel}."),
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
            1 => format!("The {material} darts away from the soap in {vessel}!"),
            _ => format!(
                "{vessel}: {material} central clearing increased from {:.0}% to {:.0}%",
                100.0 * from_cleared_fraction,
                100.0 * to_cleared_fraction
            ),
        },
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
        Event::Stirred {
            vessel,
            rpm,
            seconds,
            bar_length_m,
            tip_speed_m_s,
            resuspended_fraction,
            rate_coupled,
        } => match register.level() {
            1 => format!("The magnetic stirrer spins {vessel} for {seconds:.0} seconds."),
            2 => format!(
                "{vessel}: magnetic stirrer {rpm:.0} rpm for {seconds:.0} s — bar tip {:.3} m/s; {:.0}% resuspension",
                tip_speed_m_s,
                resuspended_fraction * 100.0,
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
                1 => format!("You grind the {name} in {vessel} into a finer powder."),
                2 => format!(
                    "{vessel}: {name} ground to {diameter_um:.1} µm — about {surface_area_m2:.3} m² surface area"
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
                1 => format!(
                    "While you wait, particles in {vessel} sink toward the bottom."
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
                    format!(
                        "{vessel}: gravity settling for {seconds:.3} s (Stokes, 1 g, 0.04 m path): {detail}"
                    )
                }
            }
        }
        Event::Filtered { from, to } => match register.level() {
            1 => format!(
                "You pour {from} through the filter paper — the liquid runs into {to}, and the solid stays behind on the paper."
            ),
            _ => format!("{from} → {to}: filtrate passed; residue retained"),
        },
        Event::MagnetSeparated { from, to, attracted, remained } => {
            let name = |s: &SpeciesId| species::lookup(s).map(|d| d.name).unwrap_or(s.0.as_str()).to_string();
            if attracted.is_empty() {
                match register.level() {
                    1 => format!("You hold a magnet over {from} — nothing jumps to it."),
                    _ => format!("{from}: no magnetic species present"),
                }
            } else {
                let att: Vec<String> = attracted.iter().map(name).collect();
                let rem: Vec<String> = remained.iter().map(name).collect();
                match register.level() {
                    1 => {
                        let rem_part = if rem.is_empty() {
                            String::new()
                        } else {
                            format!(" The {} stay{} behind.", rem.join(", "),
                                if rem.len() == 1 { "s" } else { "" })
                        };
                        format!(
                            "You hold a magnet over {from} — the {} jump{} to it. You drop {} into {to}.{rem_part}",
                            att.join(", "),
                            if att.len() == 1 { "s" } else { "" },
                            if att.len() == 1 { "it" } else { "them" },
                        )
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
            match register.level() {
                1 => {
                    if dissolved.0 <= 0.0 {
                        format!("The {name} just sits at the bottom of the {solv} — it will not dissolve.")
                    } else if undissolved.0 <= 0.0 {
                        format!("The {name} disappears into the {solv}.")
                    } else {
                        format!("A little of the {name} dissolves in the {solv}; the rest sits on the bottom.")
                    }
                }
                2 => format!(
                    "{vessel}: {name} in {solv} — {:.4} mol dissolved (handbook limit), {:.4} mol left as solid",
                    dissolved.0, undissolved.0
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
                1 => format!("The {name} just sits in the {solv} — nothing happens to it."),
                2 => format!("{vessel}: {name} does not react with {solv} — computed no-reaction, not a gap"),
                _ => format!("{vessel}: {} inert in {}: {why}", species.0, solvent.0),
            }
        }
        Event::OrgReacted { vessel, name, equation, extent, boundary } => match register.level() {
            1 => format!(
                "Something new forms in {vessel} — the {name} reaction turns the mixture into different substances."
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
                    1 => format!("You waft the air from {vessel} toward your nose — nothing you can pick out."),
                    2 => format!("{vessel}: no odour a careful waft detects"),
                    _ => format!("{vessel}: no curated odour among the volatile species — and 'odourless' is itself data: CO2 and CO teach why a nose is not a gas detector"),
                }
            } else {
                let list: Vec<String> = notes
                    .iter()
                    .map(|(sp, d)| {
                        let name = species::lookup(sp).map(|x| x.name).unwrap_or(sp.0.as_str());
                        format!("{name}: {d}")
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
                    format!("The {test} on {vessel} is positive!")
                } else {
                    format!("The {test} on {vessel} shows nothing.")
                }
            }
            2 => format!("{vessel}: {test} — {}", if *positive { "positive" } else { "negative" }),
            _ => format!("{vessel}: {test}: {notes}"),
        },
        Event::Burst { vessel, at_pa, rating_pa } => match register.level() {
            1 => format!("BANG — the sealed {vessel} could not hold the pressure and let go!"),
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
                    format!("As the liquids mingle in {vessel}, the glass grows a little warm.")
                } else {
                    format!("As the liquids mingle in {vessel}, the glass grows a little cool.")
                }
            }
            2 => format!(
                "{vessel}: heat of mixing {} {:.1} J",
                if *joules > 0.0 { "released" } else { "absorbed" },
                joules.abs()
            ),
            _ => format!(
                "{vessel}: q_mix = {joules:+.3} J from ΔHᴱ (UNIFAC Gibbs–Helmholtz, verified-pair allowlist; state-function bookkeeping, so the pour path cannot change the answer). Boundary: VLE-fitted parameters make hᴱ magnitude-class, and unverified pairs are withheld, not guessed"
            ),
        },
        Event::NuclideSpiked { vessel, nuclide, moles, activity_bq } => match register.level() {
            1 => format!("A tiny radioactive sample of {nuclide} goes into {vessel} — the counter near it starts clicking."),
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
            1 => format!("Inside {vessel}, some of the {parent} quietly turned into {daughter} while you waited."),
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
                format!("The mixture from {vessel} runs through the column and comes out one thing at a time: {order}.")
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
                format!("{vessel}: chromatogram — {table}{unseen}")
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
                format!(
                    "{vessel}: N={plates}, t0={void_time_s:.0}s, β=0.5; K = γ∞(water)/γ∞(alkane) from the same UNIFAC the funnel partitions on; tR = t0·(1+K·β), w = 4·tR/√N — {table}{unseen}"
                )
            }
        },
        Event::Drained { from, to, solvent, moles } => match register.level() {
            1 => format!("You open the tap and the bottom layer runs from {from} into {to}."),
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
            1 => format!("The liquid in {vessel} separates into two layers."),
            2 => format!(
                "{vessel}: two layers — {} floating on {}; mixing them would raise the Gibbs energy, so they split",
                species::lookup(upper).map(|d| d.name).unwrap_or(upper.0.as_str()),
                species::lookup(lower).map(|d| d.name).unwrap_or(lower.0.as_str()),
            ),
            _ => format!(
                "{vessel}: liquid–liquid split (UNIFAC LLE, common-tangent construction). The split and the layer order are robust; the trace mutual solubilities are upper bounds — VLE-fitted UNIFAC parameters underestimate alkane–water γ∞ — and are deliberately not reported",
            ),
        },
        Event::MaterialLayersFormed {
            vessel,
            upper_material,
            lower,
        } => match register.level() {
            1 => format!("The {upper_material} floats in a separate layer above the water in {vessel}."),
            2 => format!(
                "{vessel}: {upper_material} forms the upper layer; {} is denser and remains below",
                species::lookup(lower).map(|d| d.name).unwrap_or(lower.0.as_str()),
            ),
            _ => format!(
                "{vessel}: reviewed material-layer role — unresolved {upper_material} is immiscible with the aqueous phase and lies above {}; this is not a molecular LLE calculation",
                lower.0,
            ),
        },
        Event::Evaporated { vessel, moles } => match register.level() {
            1 => format!("Steam rises from {vessel} — the water is boiling away!"),
            2 => format!("{vessel}: {:.3} mol water evaporated", moles.0),
            _ => format!("{vessel}: {:.6} mol H2O evaporated (vaporisation enthalpy not yet in the balance)", moles.0),
        },
        Event::Distilled { from, to, water, ethanol, at, ended, stages, energy_kj, azeotropic } => match register.level() {
            1 => format!("Vapour rises from {from}, cools in the tube, and drips into {to}."),
            2 => {
                let t0 = at.to_celsius();
                let t1 = ended.to_celsius();
                let column = if *stages > 1 {
                    format!(" through a {stages}-stage column")
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
            1 => format!("You pour some of {from} into {to}."),
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
        Event::Ignited {
            vessel,
            flame,
            energy_j,
        } => match register.level() {
            1 => match flame {
                Some(colour) => {
                    format!("It catches fire in {vessel} — burning with {colour} light!")
                }
                None => format!("It catches fire in {vessel}!"),
            },
            2 => {
                let colour = flame
                    .as_ref()
                    .map(|colour| format!(" — {colour} flame"))
                    .unwrap_or_default();
                let energy = energy_j
                    .map(|joules| format!(" · {:.2} kJ released", joules / 1000.0))
                    .unwrap_or_default();
                format!("{vessel}: ignited{colour}{energy}")
            }
            _ => match energy_j {
                Some(joules) => format!(
                    "{vessel}: ignition source applied; computed reaction energy = {:.3} J",
                    joules
                ),
                None => format!("{vessel}: ignition source applied; reaction energy unavailable"),
            },
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
            reaction_energy_j: _,
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
                Instrument::Calorimeter => "calorimeter",
                // The column never emits a scalar Measured — it reports a
                // peak table via Chromatographed — but the name must exist.
                Instrument::Chromatograph => "chromatograph",
                Instrument::GeigerCounter => "Geiger counter",
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
        Event::Diluted {
            vessel,
            volume,
            moles,
        } => match register.level() {
            1 => format!("You add water to {vessel} — the solution gets weaker."),
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
                1 => format!(
                    "You titrate {vessel} with {name} — after {steps} additions the pH reaches {final_ph:.1}."
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
                1 => format!(
                    "Solution flows through {cells} column cells and collects in {receiver}."
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
                _ => format!(
                    "1-D upwind transport: {cells} cells × {steps} steps @ Cf={courant:.4}; \
                     effluent total {total:.6} mol → {receiver}"
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
