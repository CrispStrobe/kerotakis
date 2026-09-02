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
    /// WORLD-005 — PRODUCE: this species actually appeared, in a quantity
    /// worth seeing. Reads the step's own typed events (a precipitate, an
    /// evolved gas, a dissolution) rather than the bench, because producing
    /// a thing and having been handed it are different acts.
    Produce {
        produce: String,
        #[serde(default)]
        minimum_moles: f64,
    },
    /// WORLD-005 — SEPARATE: the sample came apart into this many
    /// components the instruments can tell apart.
    ///
    /// Two materially different solutions both count, which is the point:
    /// a column that baseline-resolves them, or a funnel that leaves them
    /// in different layers. Neither is the author's intended one.
    Separate { separate: u32 },
    /// WORLD-005 — COMPARE: two vessels differ, by enough to mean it.
    Compare {
        /// Same quantity vocabulary as a value claim.
        compare: String,
        /// Exactly two vessel words, `["v1", "v2"]`.
        between: Vec<String>,
        #[serde(default)]
        differ_by: f64,
    },
    /// WORLD-005 — DESIGN: the learner BUILT something, evidenced by a
    /// transport train of at least this many connected stages.
    Design { design: u32 },
    /// WORLD-005 — EXPLAIN: the learner stated the answer to a named
    /// question (`quest answer <topic> <answer>`), checked against the
    /// quest's own `explanations` table.
    Explain { explain: String },
}

/// WORLD-005: why a claim is not satisfied yet.
///
/// A stable tag with parameters, never prose — the same rule the catalog
/// follows, and for the same reason: the client says it in the learner's
/// language, and two clients say the same thing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "unmet", rename_all = "snake_case")]
pub enum Unmet {
    /// No qualifying evidence at all yet.
    NothingYet,
    /// Seen, but not enough of it.
    BelowThreshold { got: f64, wanted: f64 },
    /// The quantity was never measured on that vessel.
    NotMeasured { quantity: String },
    /// Measured, but not at the target.
    OutOfTolerance {
        got: f64,
        target: f64,
        tolerance: f64,
    },
    /// Separated, but not into enough distinguishable components.
    TooFewComponents { got: u32, wanted: u32 },
    /// Compared, but the two are closer than the claim asks.
    NoDifference { got: f64, wanted: f64 },
    /// A sealed unknown that has not been named.
    NotNamed { alias: String },
    /// A question that has not been answered.
    NotExplained { topic: String },
    /// Built, but not enough stages.
    TooFewStages { got: u32, wanted: u32 },
}

/// What one claim looks like right now.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaimStatus {
    pub id: String,
    pub satisfied: bool,
    /// Absent once satisfied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unmet: Option<Unmet>,
}

/// The chromatographic resolution below which two peaks are one component
/// on the trace. R = 2·Δt_R/(w₁+w₂).
const BASELINE_RESOLUTION: f64 = 1.0;

/// The share-of-the-lower-layer spread a funnel must achieve before it has
/// told two solutes apart rather than carrying both across. Measured
/// against the shipped separation sample: 0.024 at 10 mL of extracting
/// solvent, 0.107 at 50, 0.190 at 100 — so the bar sits between a token
/// splash and a real, sample-sized extraction.
const PARTITION_SPREAD: f64 = 0.15;

/// How many components a peak table actually shows. Peaks closer than
/// baseline resolution co-elute into one, exactly as they would on the
/// recorder trace, so a failed separation cannot pass on raw peak count.
pub fn resolved_components(peaks: &[kerotakis_core::ops::ElutedPeak]) -> u32 {
    let mut ordered: Vec<&kerotakis_core::ops::ElutedPeak> = peaks.iter().collect();
    ordered.sort_by(|a, b| {
        a.retention_time_s
            .partial_cmp(&b.retention_time_s)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut components = 0u32;
    let mut previous: Option<&kerotakis_core::ops::ElutedPeak> = None;
    for peak in ordered {
        match previous {
            None => components = 1,
            Some(prev) => {
                let spread = prev.width_s + peak.width_s;
                let resolution = if spread > 0.0 {
                    2.0 * (peak.retention_time_s - prev.retention_time_s) / spread
                } else {
                    0.0
                };
                if resolution >= BASELINE_RESOLUTION {
                    components += 1;
                }
            }
        }
        previous = Some(peak);
    }
    components
}

/// Components a funnel told apart: solutes whose shares of the lower layer
/// span at least [`PARTITION_SPREAD`]. A solvent that carried the whole
/// sample across separated nothing, however many solutes it moved.
fn partitioned_components(events: &[Event]) -> u32 {
    let mut fractions: Vec<f64> = Vec::new();
    for event in events {
        if let Event::Partitioned { fraction_lower, .. } = event {
            fractions.push(*fraction_lower);
        }
    }
    if fractions.len() < 2 {
        return 0;
    }
    let max = fractions.iter().cloned().fold(f64::MIN, f64::max);
    let min = fractions.iter().cloned().fold(f64::MAX, f64::min);
    if max - min >= PARTITION_SPREAD {
        fractions.len() as u32
    } else {
        0
    }
}

/// Moles of `key` this step actually PRODUCED, by the engine's own account.
fn produced_moles(events: &[Event], key: &str) -> Option<f64> {
    let mut total: Option<f64> = None;
    for event in events {
        let contribution = match event {
            Event::Precipitated { species, moles, .. }
            | Event::GasEvolved { species, moles, .. }
            | Event::Dissolved { species, moles, .. } => (species.0 == key).then_some(moles.0),
            _ => None,
        };
        if let Some(amount) = contribution {
            total = Some(total.unwrap_or(0.0) + amount);
        }
    }
    total
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

/// WORLD-004: where a mission sits in the world.
///
/// Placement is content, not code: the campus map reads it instead of a
/// hard-coded district table, so adding a mission to a district is a TOML
/// edit rather than a release.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Placement {
    /// Stable district id (`discovery-hall`, `matter-gardens`, …).
    pub district: String,
    /// Optional chapter within the district's story.
    #[serde(default)]
    pub chapter: Option<String>,
    /// Sort key inside the district; ties fall back to the quest id.
    #[serde(default)]
    pub order: Option<u32>,
}

/// WORLD-004: one materially different way to finish a mission.
///
/// A route is an AND of the claims it names; a mission with routes is an OR
/// of its routes. Deliberately the same combinator the web client's outcome
/// contracts use (GUI-080), because a learner separating a mixture on a
/// column and one separating it in a funnel have both separated the mixture,
/// and neither the engine nor the client may prefer the author's first idea.
///
/// A mission with NO routes is the v1 shape: every required claim, in any
/// order. That is what makes every shipped quest a valid v2 quest already.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub id: String,
    /// Names the approach, per register — shown only when a mission offers
    /// more than one, so a single-route mission never mentions a choice
    /// that does not exist.
    pub label: Registers,
    /// Claim ids that together satisfy this route.
    pub claims: Vec<String>,
}

/// WORLD-004: something the run must NOT do.
///
/// Constraints are how a mission says "not like that" without prescribing
/// what to do instead: a violation is recorded and spoken, never blocked,
/// because a lab that refuses the mistake never teaches it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub id: String,
    /// Codex claim syntax, same matcher as a nudge or an event claim.
    pub forbid: String,
    pub say: Registers,
}

/// WORLD-004: what completing a mission is worth.
///
/// Rewards are DECLARED here and granted by derivation from completion —
/// they are not a ledger the save mutates, because a granted-rewards list is
/// a second copy of the truth that a failed write drops and a retried one
/// grants twice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reward {
    /// `equipment` (a catalog verb or `measure:<token>`) or `reagent`
    /// (a registry species key).
    pub kind: String,
    /// The catalog id this reward grants — the same id space WORLD-003's
    /// catalog answers with.
    pub id: String,
}

/// Schema version. Absent means 1: every quest written before v2 existed.
fn schema_v1() -> u8 {
    1
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
    /// Everything else the mask must cover, alias → species keys. A
    /// dissolved unknown does not sit in the vessel under its own name:
    /// it dissociates, and a census line reading `Na+` answers the quest
    /// no less than one reading `NaCl`. The quest author, who knows what
    /// the sample becomes on the bench, lists those keys here; keys of
    /// species the learner is meant to identify by measurement, not by
    /// reading (`H+`, `OH-` — water's own) do not belong in this list.
    #[serde(default)]
    pub covers: BTreeMap<String, Vec<String>>,

    // ── WORLD-004: schema v2. Every field below defaults, so a v1 quest
    // parses unchanged and behaves exactly as it did. ──────────────────
    /// 1 for quests written before v2. Never asserted against — it is here
    /// so a future breaking change can be detected rather than guessed.
    #[serde(default = "schema_v1")]
    pub version: u8,
    /// Where this mission sits in the world.
    #[serde(default)]
    pub placement: Option<Placement>,
    /// Alternative valid solutions. Empty = every required claim (v1).
    #[serde(default)]
    pub routes: Vec<Route>,
    /// Claim ids that are optional discoveries rather than requirements.
    ///
    /// TOML trap worth knowing: this is a ROOT key, so it must be written
    /// above the first `[[claims]]` table. Written below one it silently
    /// becomes a field of that claim and is ignored — serde has no way to
    /// refuse it, because `Claim` flattens `ClaimKind` and `flatten` and
    /// `deny_unknown_fields` cannot be combined. Same for `version`.
    #[serde(default)]
    pub discoveries: BTreeSet<String>,
    /// What the run must not do.
    #[serde(default)]
    pub constraints: Vec<Constraint>,
    /// What finishing is worth.
    #[serde(default)]
    pub rewards: Vec<Reward>,
    /// WORLD-005: topic → the answer an EXPLAIN claim accepts. Answered
    /// through the same channel a sealed unknown is named, because from
    /// the learner's side both are "say what you worked out".
    #[serde(default)]
    pub explanations: BTreeMap<String, String>,
}

impl QuestSpec {
    /// Claims that must be satisfied, as opposed to discoveries a learner
    /// may find and is never required to.
    pub fn required_claims(&self) -> impl Iterator<Item = &Claim> {
        self.claims
            .iter()
            .filter(|claim| !self.discoveries.contains(&claim.id))
    }

    /// Is this quest finished?
    ///
    /// With no routes, the v1 rule: every required claim. With routes, any
    /// one route in full — which is what lets two materially different
    /// solutions both be right.
    pub fn is_complete(&self, state: &QuestState) -> bool {
        if self.routes.is_empty() {
            return self
                .required_claims()
                .all(|claim| state.satisfied.contains(&claim.id));
        }
        self.routes
            .iter()
            .any(|route| route.claims.iter().all(|id| state.satisfied.contains(id)))
    }

    /// The route the learner actually completed, if any.
    pub fn completed_route(&self, state: &QuestState) -> Option<&Route> {
        self.routes
            .iter()
            .find(|route| route.claims.iter().all(|id| state.satisfied.contains(id)))
    }
}

/// Live progress of one started quest.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuestState {
    pub satisfied: BTreeSet<String>,
    pub fired: BTreeSet<String>,
    /// WORLD-004 constraint ids this run has tripped. Recorded, never
    /// blocking: the mistake is the lesson, and a mission that refuses to
    /// let it happen cannot teach it.
    #[serde(default)]
    pub violated: BTreeSet<String>,
    pub complete: bool,
}

/// What one observation pass produced, in the order it should be told.
#[derive(Debug, Clone)]
pub enum QuestOutput {
    Nudge {
        quest: String,
        say: Registers,
    },
    /// A constraint was tripped: said once, never blocking.
    ConstraintViolated {
        quest: String,
        say: Registers,
    },
    /// A claim was satisfied. Carries the claim's stable id as well as its
    /// prose: a client that already knows its claims by id should not have
    /// to recognise one by comparing a sentence, which two claims sharing a
    /// title would defeat.
    ClaimSatisfied {
        claim: String,
        quest: String,
        title: Registers,
    },
    Completed {
        quest: String,
        title: Registers,
    },
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

/// WORLD-005: evaluate one claim against this step's evidence.
///
/// Returns the reason it is not satisfied as a stable tag with parameters,
/// so a client can say "you have 0.3 mL, you need 1" in its own language
/// rather than being handed a sentence in English or a bare false.
///
/// Three of the nine objective kinds are not variants here, deliberately:
/// OBSERVE and MEASURE are event claims (`matches = "observed"`,
/// `matches = "measured"`), and AVOID is a WORLD-004 constraint, which is
/// recorded rather than required because it must never block.
pub fn evaluate_claim(
    spec: &QuestSpec,
    claim: &Claim,
    state: &QuestState,
    events: &[Event],
    bench: &Bench,
) -> ClaimStatus {
    // Already banked: evidence does not un-happen.
    if state.satisfied.contains(&claim.id) {
        return ClaimStatus {
            id: claim.id.clone(),
            satisfied: true,
            unmet: None,
        };
    }
    let unmet = match &claim.kind {
        ClaimKind::Event { matches } => {
            if events.iter().any(|e| crate::event_matches(e, matches)) {
                None
            } else {
                Some(Unmet::NothingYet)
            }
        }
        ClaimKind::Value {
            vessel,
            quantity,
            target,
            tolerance,
        } => match parse_quantity(quantity)
            .ok()
            .and_then(|q| value_now(bench, vessel, &q))
        {
            None => Some(Unmet::NotMeasured {
                quantity: quantity.clone(),
            }),
            Some(now) if (now - target).abs() <= *tolerance => None,
            Some(now) => Some(Unmet::OutOfTolerance {
                got: now,
                target: *target,
                tolerance: *tolerance,
            }),
        },
        // Identify and Explain are satisfied through `answer`, never by
        // watching the bench: naming the thing IS the act.
        ClaimKind::Identify { alias } => Some(Unmet::NotNamed {
            alias: alias.clone(),
        }),
        ClaimKind::Explain { explain } => Some(Unmet::NotExplained {
            topic: explain.clone(),
        }),
        ClaimKind::Produce {
            produce,
            minimum_moles,
        } => match produced_moles(events, produce) {
            None => Some(Unmet::NothingYet),
            Some(got) if got >= *minimum_moles => None,
            Some(got) => Some(Unmet::BelowThreshold {
                got,
                wanted: *minimum_moles,
            }),
        },
        ClaimKind::Separate { separate } => {
            // Either route counts, and the better reading is the one
            // reported: a learner who ran both should be told the truth.
            let by_column = events
                .iter()
                .filter_map(|e| match e {
                    Event::Chromatographed { peaks, .. } => Some(resolved_components(peaks)),
                    _ => None,
                })
                .max()
                .unwrap_or(0);
            let by_funnel = if events.iter().any(|e| matches!(e, Event::Drained { .. })) {
                partitioned_components(events)
            } else {
                0
            };
            let got = by_column.max(by_funnel);
            if got >= *separate {
                None
            } else if got == 0 {
                Some(Unmet::NothingYet)
            } else {
                Some(Unmet::TooFewComponents {
                    got,
                    wanted: *separate,
                })
            }
        }
        ClaimKind::Compare {
            compare,
            between,
            differ_by,
        } => {
            let quantity = parse_quantity(compare).ok();
            let read = |word: &String| quantity.as_ref().and_then(|q| value_now(bench, word, q));
            match (
                between.first().and_then(read),
                between.get(1).and_then(read),
            ) {
                (Some(a), Some(b)) => {
                    let difference = (a - b).abs();
                    if difference >= *differ_by {
                        None
                    } else {
                        Some(Unmet::NoDifference {
                            got: difference,
                            wanted: *differ_by,
                        })
                    }
                }
                _ => Some(Unmet::NotMeasured {
                    quantity: compare.clone(),
                }),
            }
        }
        ClaimKind::Design { design } => {
            let stages = events
                .iter()
                .filter_map(|e| match e {
                    Event::Transported { chain, .. } => Some(chain.len() as u32),
                    _ => None,
                })
                .max()
                .unwrap_or(0);
            if stages >= *design {
                None
            } else if stages == 0 {
                Some(Unmet::NothingYet)
            } else {
                Some(Unmet::TooFewStages {
                    got: stages,
                    wanted: *design,
                })
            }
        }
    };
    let _ = spec;
    ClaimStatus {
        id: claim.id.clone(),
        satisfied: unmet.is_none(),
        unmet,
    }
}

/// Every claim's status, for a client that wants to show the whole board.
pub fn status(
    spec: &QuestSpec,
    state: &QuestState,
    events: &[Event],
    bench: &Bench,
) -> Vec<ClaimStatus> {
    spec.claims
        .iter()
        .map(|claim| evaluate_claim(spec, claim, state, events, bench))
        .collect()
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
        for constraint in &spec.constraints {
            if state.violated.contains(&constraint.id) {
                continue;
            }
            if events
                .iter()
                .any(|e| crate::event_matches(e, &constraint.forbid))
            {
                state.violated.insert(constraint.id.clone());
                out.push(QuestOutput::ConstraintViolated {
                    quest: spec.id.clone(),
                    say: constraint.say.clone(),
                });
            }
        }
        for claim in &spec.claims {
            if state.satisfied.contains(&claim.id) {
                continue;
            }
            // One evaluator, so what `status` reports and what `observe`
            // banks can never disagree.
            let hit = evaluate_claim(spec, claim, state, events, bench).satisfied;
            if hit {
                state.satisfied.insert(claim.id.clone());
                out.push(QuestOutput::ClaimSatisfied {
                    claim: claim.id.clone(),
                    quest: spec.id.clone(),
                    title: claim.title.clone(),
                });
            }
        }
        if !state.complete && spec.is_complete(state) {
            state.complete = true;
            out.push(QuestOutput::Completed {
                quest: spec.id.clone(),
                title: spec.title.clone(),
            });
        }
    }
    out
}

/// WORLD-007: why an answer was not accepted.
///
/// The last user-facing string in this lane that was English prose crossing
/// the host boundary. It is the same class as a balancing drill shipping its
/// coefficients: something went over the wire that should have been an id.
/// A stable tag with parameters lets a German client say it in German, and
/// lets a contract recognise it without matching on a sentence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "refused", rename_all = "snake_case")]
pub enum AnswerRefusal {
    /// Named, but not what the alias hides. The measurements already taken
    /// settle it, which is what the rendering says and the id implies.
    WrongGuess { alias: String, guess: String },
    /// Nothing active seals or asks about this alias.
    UnknownAlias { alias: String },
}

impl AnswerRefusal {
    /// The English rendering, kept so hosts that have not yet moved to the
    /// id keep printing exactly what they printed before. Every locale's
    /// version of this belongs in that locale's catalogue, keyed by the tag.
    pub fn said(&self) -> String {
        match self {
            AnswerRefusal::WrongGuess { alias, guess } => format!(
                "{alias} is not {guess} — the measurements you have already \
                 made rule this in or out; look at what they say"
            ),
            AnswerRefusal::UnknownAlias { alias } => {
                format!("no active quest seals an unknown called '{alias}'")
            }
        }
    }
}

impl std::fmt::Display for AnswerRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.said())
    }
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
    answer_typed(specs, states, alias, guess).map_err(|refusal| refusal.said())
}

/// The same decision, with the refusal as a stable id rather than a
/// sentence. `answer` is this function plus a rendering, so there is one
/// place a wrong answer is judged and the prose cannot drift from the id.
pub fn answer_typed(
    specs: &[QuestSpec],
    states: &mut BTreeMap<String, QuestState>,
    alias: &str,
    guess: &str,
) -> Result<Vec<QuestOutput>, AnswerRefusal> {
    let mut out = Vec::new();
    let mut seen_alias = false;
    for spec in specs {
        // WORLD-005: an EXPLAIN topic answers through this same channel.
        // Checked first so a quest may name a topic and a sealed sample the
        // same way without the sample's species matcher swallowing it.
        if let Some(expected) = spec.explanations.get(alias) {
            seen_alias = true;
            let Some(state) = states.get_mut(&spec.id) else {
                continue;
            };
            let claim = spec
                .claims
                .iter()
                .find(|c| matches!(&c.kind, ClaimKind::Explain { explain } if explain == alias));
            let Some(claim) = claim else { continue };
            if state.satisfied.contains(&claim.id) {
                continue;
            }
            if guess.trim().eq_ignore_ascii_case(expected.trim()) {
                state.satisfied.insert(claim.id.clone());
                out.push(QuestOutput::ClaimSatisfied {
                    claim: claim.id.clone(),
                    quest: spec.id.clone(),
                    title: claim.title.clone(),
                });
                if spec.is_complete(state) {
                    state.complete = true;
                    out.push(QuestOutput::Completed {
                        quest: spec.id.clone(),
                        title: spec.title.clone(),
                    });
                }
            }
            continue;
        }
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
                claim: claim.id.clone(),
                quest: spec.id.clone(),
                title: claim.title.clone(),
            });
            if spec.is_complete(state) {
                state.complete = true;
                out.push(QuestOutput::Completed {
                    quest: spec.id.clone(),
                    title: spec.title.clone(),
                });
            }
        } else {
            return Err(AnswerRefusal::WrongGuess {
                alias: alias.to_string(),
                guess: guess.to_string(),
            });
        }
    }
    if seen_alias {
        Ok(out)
    } else {
        Err(AnswerRefusal::UnknownAlias {
            alias: alias.to_string(),
        })
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
                // ── WORLD-005: the new objective kinds ──────────────────
                ClaimKind::Produce { produce, .. } => {
                    if species::lookup(&SpeciesId::new(produce)).is_none() {
                        problems.push(format!(
                            "{q}: claim '{}' asks for '{produce}', not in the registry",
                            claim.id
                        ));
                    }
                }
                ClaimKind::Separate { separate } => {
                    if *separate < 2 {
                        problems.push(format!(
                            "{q}: claim '{}' separates into {separate} — a \
                             separation of fewer than two components is not one",
                            claim.id
                        ));
                    }
                }
                ClaimKind::Compare {
                    compare, between, ..
                } => {
                    if between.len() != 2 {
                        problems.push(format!(
                            "{q}: claim '{}' compares {} vessels, not two",
                            claim.id,
                            between.len()
                        ));
                    }
                    for word in between {
                        if kerotakis_core::script::parse_vessel(word).is_err() {
                            problems.push(format!(
                                "{q}: claim '{}' names bad vessel '{word}'",
                                claim.id
                            ));
                        }
                    }
                    if let Err(e) = parse_quantity(compare) {
                        problems.push(format!("{q}: claim '{}': {e}", claim.id));
                    }
                }
                ClaimKind::Design { design } => {
                    if *design < 2 {
                        problems.push(format!(
                            "{q}: claim '{}' designs {design} stages — a train \
                             of fewer than two is not connected apparatus",
                            claim.id
                        ));
                    }
                }
                ClaimKind::Explain { explain } => {
                    if !spec.explanations.contains_key(explain) {
                        problems.push(format!(
                            "{q}: claim '{}' explains '{explain}', which has no \
                             entry in [explanations] — nothing could ever answer it",
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
        for topic in spec.explanations.keys() {
            if !spec
                .claims
                .iter()
                .any(|c| matches!(&c.kind, ClaimKind::Explain { explain } if explain == topic))
            {
                problems.push(format!(
                    "{q}: [explanations] answers '{topic}', which no claim asks"
                ));
            }
        }

        // ── WORLD-004: schema v2. A route or discovery naming a claim
        // that does not exist is a mission that can never be finished, and
        // it must fail here rather than in front of a learner. ──────────
        let mut route_ids = BTreeSet::new();
        for route in &spec.routes {
            if !route_ids.insert(route.id.clone()) {
                problems.push(format!("{q}: duplicate route id '{}'", route.id));
            }
            if route.claims.is_empty() {
                problems.push(format!(
                    "{q}: route '{}' names no claims — an empty route is \
                     satisfied by doing nothing",
                    route.id
                ));
            }
            for id in &route.claims {
                if !claim_ids.contains(id) {
                    problems.push(format!(
                        "{q}: route '{}' names claim '{id}', which does not exist",
                        route.id
                    ));
                }
            }
        }
        // Every claim should be reachable through some route, or the quest
        // asks for work that can never count.
        if !spec.routes.is_empty() {
            let routed: BTreeSet<&String> =
                spec.routes.iter().flat_map(|r| r.claims.iter()).collect();
            for claim in &spec.claims {
                if !routed.contains(&claim.id) && !spec.discoveries.contains(&claim.id) {
                    problems.push(format!(
                        "{q}: claim '{}' belongs to no route and is not a \
                         discovery — it can never count toward completion",
                        claim.id
                    ));
                }
            }
        }
        for id in &spec.discoveries {
            if !claim_ids.contains(id) {
                problems.push(format!("{q}: discovery '{id}' names no such claim"));
            }
        }
        if spec.discoveries.len() == spec.claims.len() && !spec.claims.is_empty() {
            problems.push(format!(
                "{q}: every claim is a discovery — nothing is required, so the \
                 quest completes before it starts"
            ));
        }
        let mut constraint_ids = BTreeSet::new();
        for constraint in &spec.constraints {
            if !constraint_ids.insert(constraint.id.clone()) {
                problems.push(format!("{q}: duplicate constraint id '{}'", constraint.id));
            }
            let kind = constraint.forbid.split(':').next().unwrap_or("");
            if !crate::KNOWN_EVENT_KINDS.contains(&kind) {
                problems.push(format!(
                    "{q}: constraint '{}' forbids unknown event kind '{kind}'",
                    constraint.id
                ));
            }
        }
        for reward in &spec.rewards {
            if !matches!(reward.kind.as_str(), "equipment" | "reagent") {
                problems.push(format!(
                    "{q}: reward '{}' has unknown kind '{}' (equipment, reagent)",
                    reward.id, reward.kind
                ));
            }
            if reward.kind == "reagent" && species::lookup(&SpeciesId::new(&reward.id)).is_none() {
                problems.push(format!(
                    "{q}: reward reagent '{}' is not in the registry",
                    reward.id
                ));
            }
        }
    }
    problems
}
