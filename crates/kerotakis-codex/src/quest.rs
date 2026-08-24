//! EXP-0: the quest engine. A lesson is a corridor — a script replayed.
//! A quest is the opposite: a stated goal, a fully open bench, nudges
//! that fire on what the learner actually does, and completion claims
//! satisfiable in any order. Nine audit parts of EXPERIMENTS.md wait
//! behind this module.
//!
//! Three claim kinds carry all of it: EVENT claims reuse the codex's
//! own `kind:detail` syntax and matcher; VALUE claims read the solved
//! state (pH, temperature, mass, moles, molarity) against a target ±
//! tolerance — the numeric-goal essence of the quantitative corpora;
//! IDENTIFY claims close the sealed-unknown loop — the quest hides a
//! species behind an alias, the engine computes it truthfully
//! underneath, and naming it from measurements IS the game.
//!
//! Nudges fire at most once, never block, and never prescribe the only
//! next step. Multiple quests run concurrently by construction; the
//! engine is a matcher, not a narrator — the codex remains the
//! pedagogical voice.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use kerotakis_core::{species, Bench, Event, SpeciesId};

/// One line of prose per register, like every codex surface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registers {
    pub lv1: String,
    pub lv2: String,
    pub lv3: String,
}

impl Registers {
    pub fn at(&self, level: u8) -> &str {
        match level {
            1 => &self.lv1,
            2 => &self.lv2,
            _ => &self.lv3,
        }
    }
}

/// A hint that fires once when its event pattern first matches.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nudge {
    pub id: String,
    /// Codex claim syntax: `kind` or `kind:detail` (see
    /// [`crate::event_matches`]).
    pub when: String,
    pub say: Registers,
}

/// Shape-distinguished (untagged): a claim with `matches` is an event
/// claim, with `alias` an identify claim, with `quantity`+`target`+
/// `tolerance` a value claim. TOML stays flat and human-writable.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ClaimKind {
    /// An event of this kind (codex `kind:detail` syntax) occurred.
    Event { matches: String },
    /// A quantity of the solved state reached `target ± tolerance`.
    /// `quantity` is one of `ph`, `temperature_c`, `mass_g`,
    /// `moles:<species>`, `molarity:<species>`.
    Value {
        vessel: String,
        quantity: String,
        target: f64,
        tolerance: f64,
    },
    /// The learner correctly named a sealed unknown
    /// (`quest answer <alias> <species>`).
    Identify { alias: String },
}

#[derive(Debug, Clone)]
pub enum Quantity {
    Ph,
    TemperatureC,
    MassG,
    Moles(String),
    /// mol/L over the computed liquid volume (Σ n·M/ρ of liquid
    /// portions). A model, stated: solution volume ≈ solvent volume.
    Molarity(String),
}

pub fn parse_quantity(s: &str) -> Result<Quantity, String> {
    match s.split_once(':') {
        None => match s {
            "ph" => Ok(Quantity::Ph),
            "temperature_c" => Ok(Quantity::TemperatureC),
            "mass_g" => Ok(Quantity::MassG),
            other => Err(format!(
                "unknown quantity '{other}' (ph, temperature_c, mass_g,                  moles:<species>, molarity:<species>)"
            )),
        },
        Some(("moles", key)) => Ok(Quantity::Moles(key.to_string())),
        Some(("molarity", key)) => Ok(Quantity::Molarity(key.to_string())),
        Some((other, _)) => Err(format!("unknown quantity '{other}:…'")),
    }
}

/// One completion condition, with its own prose so status can speak
/// without leaking what a sealed claim is really checking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub id: String,
    pub title: Registers,
    #[serde(flatten)]
    pub kind: ClaimKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestSpec {
    pub id: String,
    pub title: Registers,
    pub goal: Registers,
    #[serde(default)]
    pub nudges: Vec<Nudge>,
    pub claims: Vec<Claim>,
    /// Sealed reagents: alias → real registry species. The alias is
    /// what the learner sees and types; the chemistry underneath is
    /// never altered — only the display layer wears the mask.
    #[serde(default)]
    pub unknowns: BTreeMap<String, String>,
}

/// Live progress of one started quest.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuestState {
    pub satisfied: BTreeSet<String>,
    pub fired: BTreeSet<String>,
    pub complete: bool,
}

/// What one observation pass produced, in the order it should be told.
#[derive(Debug, Clone)]
pub enum QuestOutput {
    Nudge { quest: String, say: Registers },
    ClaimSatisfied { quest: String, title: Registers },
    Completed { quest: String, title: Registers },
}

/// The computed liquid volume of a vessel in litres — Σ n·M/ρ over
/// liquid portions. The stated model boundary for molarity claims.
fn liquid_volume_l(bench: &Bench, vessel: kerotakis_core::VesselId) -> f64 {
    bench
        .vessel(vessel)
        .map(|v| {
            v.contents
                .iter()
                .filter(|p| p.phase == species::Phase::Liquid)
                .filter_map(|p| {
                    species::lookup(&p.species)
                        .map(|d| p.moles.0 * d.molar_mass / d.density / 1000.0)
                })
                .sum()
        })
        .unwrap_or(0.0)
}

fn value_now(bench: &Bench, vessel_word: &str, q: &Quantity) -> Option<f64> {
    let vessel = kerotakis_core::script::parse_vessel(vessel_word).ok()?;
    let v = bench.vessel(vessel).ok()?;
    match q {
        Quantity::Ph => v.solution.as_ref().map(|s| s.ph),
        Quantity::TemperatureC => Some(v.temperature.to_celsius()),
        Quantity::MassG => Some(
            v.contents
                .iter()
                .filter_map(|p| species::lookup(&p.species).map(|d| p.moles.0 * d.molar_mass))
                .sum(),
        ),
        Quantity::Moles(key) => Some(v.moles_of(&SpeciesId::new(key)).0),
        Quantity::Molarity(key) => {
            let vol = liquid_volume_l(bench, vessel);
            if vol <= 0.0 {
                None
            } else {
                Some(v.moles_of(&SpeciesId::new(key)).0 / vol)
            }
        }
    }
}

/// Feed one step's events plus the settled bench through every active
/// quest. Answers to sealed unknowns arrive separately via [`answer`].
pub fn observe(
    specs: &[QuestSpec],
    states: &mut BTreeMap<String, QuestState>,
    events: &[Event],
    bench: &Bench,
) -> Vec<QuestOutput> {
    let mut out = Vec::new();
    for spec in specs {
        let Some(state) = states.get_mut(&spec.id) else {
            continue;
        };
        if state.complete {
            continue;
        }
        for nudge in &spec.nudges {
            if state.fired.contains(&nudge.id) {
                continue;
            }
            if events.iter().any(|e| crate::event_matches(e, &nudge.when)) {
                state.fired.insert(nudge.id.clone());
                out.push(QuestOutput::Nudge {
                    quest: spec.id.clone(),
                    say: nudge.say.clone(),
                });
            }
        }
        for claim in &spec.claims {
            if state.satisfied.contains(&claim.id) {
                continue;
            }
            let hit = match &claim.kind {
                ClaimKind::Event { matches } => {
                    events.iter().any(|e| crate::event_matches(e, matches))
                }
                ClaimKind::Value {
                    vessel,
                    quantity,
                    target,
                    tolerance,
                } => parse_quantity(quantity)
                    .ok()
                    .and_then(|q| value_now(bench, vessel, &q))
                    .is_some_and(|now| (now - target).abs() <= *tolerance),
                ClaimKind::Identify { .. } => false,
            };
            if hit {
                state.satisfied.insert(claim.id.clone());
                out.push(QuestOutput::ClaimSatisfied {
                    quest: spec.id.clone(),
                    title: claim.title.clone(),
                });
            }
        }
        if !state.complete && spec.claims.iter().all(|c| state.satisfied.contains(&c.id)) {
            state.complete = true;
            out.push(QuestOutput::Completed {
                quest: spec.id.clone(),
                title: spec.title.clone(),
            });
        }
    }
    out
}

/// The learner names a sealed unknown. A correct answer satisfies the
/// matching Identify claim; a wrong one is answered with the goal
/// register's diagnosis style — spoken, never punished, never blocking.
pub fn answer(
    specs: &[QuestSpec],
    states: &mut BTreeMap<String, QuestState>,
    alias: &str,
    guess: &str,
) -> Result<Vec<QuestOutput>, String> {
    let mut out = Vec::new();
    let mut seen_alias = false;
    for spec in specs {
        let Some(real) = spec.unknowns.get(alias) else {
            continue;
        };
        seen_alias = true;
        let Some(state) = states.get_mut(&spec.id) else {
            continue;
        };
        let claim = spec
            .claims
            .iter()
            .find(|c| matches!(&c.kind, ClaimKind::Identify { alias: a } if a == alias));
        let Some(claim) = claim else { continue };
        if state.satisfied.contains(&claim.id) {
            continue;
        }
        if guess == real
            || species::lookup(&SpeciesId::new(guess))
                .zip(species::lookup(&SpeciesId::new(real)))
                .is_some_and(|(g, r)| g.key == r.key)
        {
            state.satisfied.insert(claim.id.clone());
            out.push(QuestOutput::ClaimSatisfied {
                quest: spec.id.clone(),
                title: claim.title.clone(),
            });
            if spec.claims.iter().all(|c| state.satisfied.contains(&c.id)) {
                state.complete = true;
                out.push(QuestOutput::Completed {
                    quest: spec.id.clone(),
                    title: spec.title.clone(),
                });
            }
        } else {
            return Err(format!(
                "{alias} is not {guess} — the measurements you have already \
                 made rule this in or out; look at what they say"
            ));
        }
    }
    if seen_alias {
        Ok(out)
    } else {
        Err(format!("no active quest seals an unknown called '{alias}'"))
    }
}

/// Load every `*.toml` quest in a directory, sorted by id.
pub fn load_dir(dir: &std::path::Path) -> Result<Vec<QuestSpec>, String> {
    let mut specs = Vec::new();
    let entries = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for entry in entries {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let spec: QuestSpec =
            toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        specs.push(spec);
    }
    specs.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(specs)
}

/// The lint: a quest that could lie, block, or corridor fails here.
pub fn lint(specs: &[QuestSpec]) -> Vec<String> {
    let mut problems = Vec::new();
    let mut ids = BTreeSet::new();
    for spec in specs {
        let q = &spec.id;
        if !ids.insert(q.clone()) {
            problems.push(format!("{q}: duplicate quest id"));
        }
        if spec.claims.len() < 2 {
            problems.push(format!(
                "{q}: fewer than two claims — a single-claim quest is a \
                 corridor with a door at the end"
            ));
        }
        for (alias, real) in &spec.unknowns {
            if species::lookup(&SpeciesId::new(real)).is_none() {
                problems.push(format!(
                    "{q}: unknown '{alias}' maps to '{real}', not in the registry"
                ));
            }
            if species::lookup(&SpeciesId::new(alias)).is_some() {
                problems.push(format!(
                    "{q}: alias '{alias}' collides with a real registry species"
                ));
            }
        }
        let mut claim_ids = BTreeSet::new();
        for claim in &spec.claims {
            if !claim_ids.insert(claim.id.clone()) {
                problems.push(format!("{q}: duplicate claim id '{}'", claim.id));
            }
            match &claim.kind {
                ClaimKind::Event { matches } => {
                    let kind = matches.split(':').next().unwrap_or("");
                    if !crate::KNOWN_EVENT_KINDS.contains(&kind) {
                        problems.push(format!(
                            "{q}: claim '{}' matches unknown event kind '{kind}'",
                            claim.id
                        ));
                    }
                }
                ClaimKind::Value {
                    tolerance,
                    quantity,
                    vessel,
                    ..
                } => {
                    if *tolerance <= 0.0 {
                        problems.push(format!(
                            "{q}: claim '{}' has a non-positive tolerance — a \
                             value claim without slack is a lie about precision",
                            claim.id
                        ));
                    }
                    if let Err(e) = parse_quantity(quantity) {
                        problems.push(format!("{q}: claim '{}': {e}", claim.id));
                    }
                    if kerotakis_core::script::parse_vessel(vessel).is_err() {
                        problems.push(format!(
                            "{q}: claim '{}' names bad vessel '{vessel}'",
                            claim.id
                        ));
                    }
                }
                ClaimKind::Identify { alias } => {
                    if !spec.unknowns.contains_key(alias) {
                        problems.push(format!(
                            "{q}: claim '{}' identifies '{alias}' but the quest \
                             seals no such unknown",
                            claim.id
                        ));
                    }
                }
            }
        }
        for nudge in &spec.nudges {
            let kind = nudge.when.split(':').next().unwrap_or("");
            if !crate::KNOWN_EVENT_KINDS.contains(&kind) {
                problems.push(format!(
                    "{q}: nudge '{}' fires on unknown event kind '{kind}'",
                    nudge.id
                ));
            }
        }
    }
    problems
}
