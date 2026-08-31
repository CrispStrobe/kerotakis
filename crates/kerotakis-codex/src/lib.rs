//! # kerotakis-codex
//!
//! The curated part: curriculum reactions with the concepts that explain
//! them, written at every register, each carrying its provenance.
//!
//! The codex is TOML because a chemistry editor has to be able to write it
//! without a build step. Its schema is deliberately small — an entry is a
//! *setup script*, the observations it claims, the words for each register,
//! and where the claim comes from.
//!
//! The point of the format is the check it enables. Because a setup is a
//! `.lab` script, `lint` can **replay every entry through the real
//! solvers** and confirm the claimed observations actually occur. A codex
//! entry that stops being true fails the build. Nothing else in this
//! project would catch a curation error; this does.

pub mod curiosity;
pub mod prose;
pub mod quest;

/// Every event kind `event_matches` knows, for lint use. Kept beside
/// the matcher on purpose: a new Event arm extends both or the quest
/// lint names the gap.
pub const KNOWN_EVENT_KINDS: &[&str] = &[
    "added",
    "cell_voltage",
    "chromatographed",
    "collision_withstood",
    "container_broken",
    "consumed",
    "curdling_changed",
    "did_not_ignite",
    "diluted",
    "dissolved",
    "dissolved_in_solvent",
    "distilled",
    "drained",
    "emulsion_changed",
    "electrolysed",
    "evaporated",
    "filtered",
    "flame_test",
    "foam_changed",
    "gas_absorbed",
    "gas_tested",
    "gas_contained",
    "gas_evolved",
    "gas_produced",
    "hazard_warning",
    "headspace_equilibrated",
    "ignited",
    "inert",
    "inert_in_solvent",
    "layers_formed",
    "magnet_separated",
    "material_added",
    "measured",
    "mixed",
    "no_cell",
    "not_yet_modelled",
    "observed",
    "org_reacted",
    "partitioned",
    "plated",
    "precipitated",
    "reacted",
    "reaction",
    "reaction_heat_released",
    "safety_veto",
    "solution",
    "solver_failed",
    "spill_created",
    "spill_hazard",
    "spill_recovered",
    "surface_spread",
    "temperature_changed",
    "thermal_equilibrium",
    "titrated",
    "transferred",
    "transported",
    "vessel_created",
    "vessel_opened",
    "vessel_pressure_controlled",
    "vessel_sealed",
    "vessel_swept",
    "nuclide_spiked",
    "decayed",
    "smelled",
    "burst",
    "heat_of_mixing",
];

use kerotakis_core::{Phase, Register};
use serde::{Deserialize, Serialize};

/// One curated reaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    /// Stable slug, used by lessons and the concept graph.
    pub id: String,
    /// A **balanced chemical equation**, and nothing else.
    ///
    /// If this is present it is a claim, and the lint enforces it: the
    /// string must parse as an equation and must conserve both atoms and
    /// charge. Anything that is not an equation — a mass calculation, a
    /// phrase like "CH₃COOH / CH₃COO⁻ buffer", a description of an
    /// experiment where nothing reacts — belongs in `summary`.
    ///
    /// Splitting these apart was forced by evidence: with one field doing
    /// both jobs, 27 of 66 entries held prose here, and a checker cannot
    /// tell a deliberate summary from an equation someone got wrong. Now
    /// the schema says which is which, so silence is no longer ambiguous.
    #[serde(default)]
    pub equation: Option<String>,
    /// A plain-language or arithmetic characterisation, for entries whose
    /// point is not a reaction: a yield calculation, a null result, a
    /// measurement, a physical change. Never parsed, never checked.
    #[serde(default)]
    pub summary: Option<String>,
    /// German. The catalogue translates field by field into `_de`
    /// siblings so an untranslated string falls back to English on its
    /// own, rather than an entry needing a complete German twin before
    /// any of it can ship.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary_de: Option<String>,
    /// Concepts this reaction teaches; the difficulty ladder is built from
    /// these edges.
    #[serde(default)]
    pub concepts: Vec<String>,
    /// Concepts a learner needs first.
    #[serde(default)]
    pub requires: Vec<String>,
    /// Topics from the curriculum spine this entry covers. Our own
    /// `concepts` are the words we teach in; these are the anchors into
    /// somebody else's published taxonomy, which is what lets us measure
    /// coverage honestly rather than declaring ourselves complete.
    #[serde(default)]
    pub spine: Vec<String>,
    /// Where this sits in a curriculum. Several may apply: the same
    /// reaction is met at different depths in different systems, and the
    /// same system meets it more than once.
    #[serde(default)]
    pub curriculum: Vec<Placement>,
    /// Apparatus the experiment needs, by registry-independent name
    /// ("beaker", "burette", "crucible", "bunsen"). Drives what a UI puts
    /// on the bench and what a real teacher would have to fetch.
    #[serde(default)]
    pub apparatus: Vec<String>,
    /// Calculations a learner is expected to perform here
    /// ("moles-from-mass", "concentration", "titration-stoichiometry",
    /// "percentage-yield", "enthalpy-change").
    #[serde(default)]
    pub calculations: Vec<String>,
    /// Models a learner is expected to reason with ("particle-model",
    /// "ionic-bonding", "collision-theory", "equilibrium", "orbital").
    /// The same reaction is explained by different models at different
    /// levels, which is what makes a level more than a change of wording.
    #[serde(default)]
    pub models: Vec<String>,
    pub setup: Setup,
    pub expect: Expect,
    pub registers: Registers,
    pub provenance: Provenance,
    /// Translations into languages no field here names.
    ///
    /// `summary_de` is a field; `summary_fr` is not, and without somewhere
    /// to land it would be parsed and then silently discarded. Adding a
    /// language is meant to be a data change — one sidecar file — so the
    /// types must not have to learn its name first.
    #[serde(
        flatten,
        default,
        skip_serializing_if = "std::collections::BTreeMap::is_empty"
    )]
    pub other_locales: std::collections::BTreeMap<String, toml::Value>,
}

/// One curriculum's placement of an entry.
///
/// Kept deliberately loose: a `system` string, a `stage` as that system
/// writes it, and an approximate age band so entries can be compared
/// across systems that number their years differently. The citation is
/// required — a placement claim is a claim about someone's document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Placement {
    /// e.g. "england-national-curriculum", "australian-curriculum-v9",
    /// "bayern-lehrplanplus", "openstax-chemistry-2e".
    pub system: String,
    /// The stage as that system names it: "KS3", "Year 10", "Jgst. 9",
    /// "Chapter 4.2".
    pub stage: String,
    /// Approximate age band, for cross-system comparison.
    #[serde(default)]
    pub ages: Option<Range>,
    /// Where the placement was read from, and under what licence.
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setup {
    /// A `.lab` script: the same grammar the CLI and the browser run.
    pub script: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Expect {
    /// Events that must occur, as `kind` or `kind:species`, e.g.
    /// `precipitated:AgCl`, `gas_evolved:CO2`, `hazard_warning`.
    #[serde(default)]
    pub events: Vec<String>,
    /// Events that must *not* occur — the guard against a lesson quietly
    /// starting to lie in the other direction.
    #[serde(default)]
    pub absent: Vec<String>,
    /// A question put to the learner *before* the chemistry runs, with the
    /// answer the engine will give. Predict-observe-explain is the
    /// best-evidenced sequence in science education, and it only works if
    /// the prediction is committed before the reveal.
    #[serde(default)]
    pub predict: Option<Prediction>,
    /// Optional numeric checks on the final state of a vessel.
    #[serde(default)]
    pub ph: Option<Range>,
    #[serde(default)]
    pub temperature_c: Option<Range>,
}

/// A prediction the learner commits to before running the experiment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    pub question: String,
    /// German. The catalogue translates field by field into `_de`
    /// siblings so an untranslated string falls back to English on its
    /// own, rather than an entry needing a complete German twin before
    /// any of it can ship.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub question_de: Option<String>,
    /// Plausible answers to choose between; exactly one is right, and the
    /// wrong ones should be the mistakes learners actually make, not
    /// strawmen.
    pub options: Vec<String>,
    /// Positional twin of `options`. Same length or ignored: the answer
    /// is an index into `options` and each diagnosis attaches by index,
    /// so a shorter array would mark the wrong answer correct.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options_de: Option<Vec<String>>,
    /// Index into `options`.
    pub answer: usize,
    /// Why the tempting wrong answer is tempting — the misconception this
    /// question exists to surface. A single note covering the whole
    /// question; `diagnosis` is the finer-grained form.
    #[serde(default)]
    pub misconception: Option<String>,
    /// German. The catalogue translates field by field into `_de`
    /// siblings so an untranslated string falls back to English on its
    /// own, rather than an entry needing a complete German twin before
    /// any of it can ship.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub misconception_de: Option<String>,
    /// What each individual wrong answer reveals, and what to do about it.
    ///
    /// One blanket note per question is not enough to *teach* with. A
    /// learner who picks option 2 has a specific idea in their head, and it
    /// is rarely the same idea as the one that leads to option 3 — the
    /// evidence on conceptual change is that instruction works by eliciting
    /// the learner's own model and confronting it, which cannot be done
    /// with a single averaged explanation.
    ///
    /// These are cheap to source honestly: misconception *prevalence
    /// findings* are research facts rather than copyrightable expression,
    /// so the finding is cited and the option is written by us. That is
    /// also the better path, because a distractor has to match what this
    /// engine actually computes rather than what a textbook rounds to.
    #[serde(default)]
    pub diagnosis: Vec<Diagnosis>,
    /// Translations into languages no field here names.
    ///
    /// `summary_de` is a field; `summary_fr` is not, and without somewhere
    /// to land it would be parsed and then silently discarded. Adding a
    /// language is meant to be a data change — one sidecar file — so the
    /// types must not have to learn its name first.
    #[serde(
        flatten,
        default,
        skip_serializing_if = "std::collections::BTreeMap::is_empty"
    )]
    pub other_locales: std::collections::BTreeMap<String, toml::Value>,
}

/// Write `value` at `<path>_<code>`, following the path through tables and
/// arrays. Numeric segments index arrays.
/// Why a translation did not fit its catalogue.
#[derive(Debug)]
pub enum Unfit {
    /// No entry with that id in this file. Expected, not a fault, when one
    /// sidecar covers a whole directory: the entry is in a sibling file.
    NoSuchEntry,
    /// The entry is here but the field it names is not. Always a fault —
    /// the English moved and this translation was left behind, which
    /// renders confidently and wrongly.
    NoSuchField(String),
}

impl std::fmt::Display for Unfit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Unfit::NoSuchEntry => write!(f, "no entry with that id"),
            Unfit::NoSuchField(m) => write!(f, "{m}"),
        }
    }
}

fn inject(doc: &mut toml::Value, path: &str, code: &str, value: toml::Value) -> Result<(), Unfit> {
    let parts: Vec<&str> = path.split('.').collect();
    let (last, rest) = parts
        .split_last()
        .ok_or_else(|| Unfit::NoSuchField(format!("empty path {path:?}")))?;

    // The first segment is an entry id, not a key: find the reaction or
    // model it names.
    let mut node =
        find_entry(doc, rest.first().copied().unwrap_or(last)).ok_or(Unfit::NoSuchEntry)?;

    for seg in rest.iter().skip(1) {
        // Reborrowing in place rather than reassigning: `node = node.get_mut()`
        // keeps the previous borrow alive for the whole loop.
        node = match node {
            toml::Value::Table(t) => t
                .get_mut(seg.to_string().as_str())
                .ok_or_else(|| Unfit::NoSuchField(format!("{path}: no field {seg:?}")))?,
            toml::Value::Array(a) => {
                let i: usize = seg
                    .parse()
                    .map_err(|_| Unfit::NoSuchField(format!("{path}: {seg:?} is not an index")))?;
                a.get_mut(i).ok_or_else(|| {
                    Unfit::NoSuchField(format!("{path}: index {i} is past the end"))
                })?
            }
            _ => {
                return Err(Unfit::NoSuchField(format!(
                    "{path}: {seg:?} is not a table or array"
                )))
            }
        };
    }

    let table = node.as_table_mut().ok_or_else(|| {
        Unfit::NoSuchField(format!("{path}: the parent of {last:?} is not a table"))
    })?;
    if !table.contains_key(*last) {
        return Err(Unfit::NoSuchField(format!(
            "{path}: there is no {last:?} to translate"
        )));
    }
    table.insert(format!("{last}_{code}"), value);
    Ok(())
}

/// The reaction or model with this id.
fn find_entry<'a>(doc: &'a mut toml::Value, id: &str) -> Option<&'a mut toml::Value> {
    let table = doc.as_table_mut()?;
    // Which section holds it is decided by an IMMUTABLE pass first, so the
    // mutable borrow below is taken exactly once. Looping with `get_mut`
    // keeps every iteration's borrow alive, because the value returned
    // from inside the loop escapes it.
    let section = ["reaction", "model"].into_iter().find(|s| {
        table.get(*s).and_then(|v| v.as_array()).is_some_and(|a| {
            a.iter()
                .any(|e| e.get("id").and_then(|v| v.as_str()) == Some(id))
        })
    })?;
    table
        .get_mut(section)?
        .as_array_mut()?
        .iter_mut()
        .find(|e| e.get("id").and_then(|v| v.as_str()) == Some(id))
}

/// What one particular wrong answer tells you about the learner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnosis {
    /// Index into `Prediction::options`.
    pub option: usize,
    /// The idea that leads here, stated as the learner would hold it.
    pub reveals: String,
    /// German. The catalogue translates field by field into `_de`
    /// siblings so an untranslated string falls back to English on its
    /// own, rather than an entry needing a complete German twin before
    /// any of it can ship.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reveals_de: Option<String>,
    /// What to do next: the experiment, comparison or question that puts
    /// pressure on exactly this idea. A diagnosis without a next move is
    /// a label, not teaching.
    #[serde(default)]
    pub next: Option<String>,
    /// German. The catalogue translates field by field into `_de`
    /// siblings so an untranslated string falls back to English on its
    /// own, rather than an entry needing a complete German twin before
    /// any of it can ship.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_de: Option<String>,
    /// Where the misconception is documented, if it is. Left absent rather
    /// than invented.
    #[serde(default)]
    pub source: Option<String>,
    /// Translations into languages no field here names.
    ///
    /// `summary_de` is a field; `summary_fr` is not, and without somewhere
    /// to land it would be parsed and then silently discarded. Adding a
    /// language is meant to be a data change — one sidecar file — so the
    /// types must not have to learn its name first.
    #[serde(
        flatten,
        default,
        skip_serializing_if = "std::collections::BTreeMap::is_empty"
    )]
    pub other_locales: std::collections::BTreeMap<String, toml::Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Range {
    pub min: f64,
    pub max: f64,
}

impl Range {
    pub fn contains(&self, v: f64) -> bool {
        v >= self.min && v <= self.max
    }
}

/// Split an `equation` string into the clauses that might be equations.
///
/// The field carries more than equations in practice: annotations after a
/// spaced middot, a parenthesised aside, a ΔH, or two reactions separated
/// by a semicolon. Splitting is deliberately conservative — anything that
/// does not come out as a chemical equation is reported as unverified
/// rather than silently passed.
pub fn equation_clauses(equation: &str) -> Vec<String> {
    const ARROWS: [&str; 8] = ["⇌", "⟶", "→", "->", "<=>", "=>", "⇄", "↔"];
    let mut out = Vec::new();
    for part in equation.split(';') {
        // A spaced middot introduces prose; a flush one is a hydrate.
        let mut head = part.split("  ·  ").next().unwrap_or(part);
        head = head.split(" · ").next().unwrap_or(head);
        head = match head.find("   Δ") {
            Some(i) => &head[..i],
            None => head,
        };
        let mut clause = head.trim().to_string();
        // Trailing asides: "(saturated at ≈6.1 mol/kgw)", "(K_sp ≈ …)".
        // A parenthetical containing a space is prose; state labels like
        // "(aq)" never do, and are handled by the formula parser. This runs
        // *before* the comma split below, because an aside may itself
        // contain a comma — "(open to the atmosphere, log pCO₂ = −3.408)"
        // was being cut in half and taking a real equation with it.
        while clause.ends_with(')') {
            let Some(open) = clause.rfind('(') else { break };
            if !clause[open..].contains(' ') {
                break;
            }
            clause = clause[..open].trim_end().to_string();
        }
        // A comma never appears inside a formula, so what follows one is
        // prose: "AgCl ⇌ Ag⁺ + Cl⁻, suppressed by added Cl⁻".
        clause = clause
            .split(", ")
            .next()
            .unwrap_or(&clause)
            .trim()
            .to_string();
        if clause.is_empty() {
            continue;
        }
        // A chain of equilibria — "H₃PO₄ ⇌ H⁺ + H₂PO₄⁻ ⇌ 2 H⁺ + HPO₄²⁻" —
        // is several equations sharing their intermediate terms. Check each
        // consecutive pair rather than reading the first arrow and treating
        // the rest of the chain as one enormous right-hand side.
        let mut segments: Vec<String> = vec![clause.clone()];
        for arrow in ARROWS {
            if clause.contains(arrow) {
                segments = clause.split(arrow).map(|s| s.trim().to_string()).collect();
                if segments.len() > 2 {
                    for pair in segments.windows(2) {
                        out.push(format!("{} {arrow} {}", pair[0], pair[1]));
                    }
                    break;
                }
                segments = vec![clause.clone()];
                break;
            }
        }
        if segments.len() == 1 {
            out.push(clause);
        }
    }
    out
}

/// How much of the codex's equation field is actually verifiable.
///
/// Returned rather than logged so the caller can print it: a checker that
/// silently ignores what it cannot parse claims a clean bill of health it
/// has not earned.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PredictionAudit {
    pub predictions: usize,
    /// Wrong options across all predictions.
    pub distractors: usize,
    /// Wrong options that say what believing them reveals.
    pub diagnosed: usize,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EquationAudit {
    /// Equation clauses that parsed and balanced, atoms and charge.
    pub balanced: usize,
    /// Entries that declare a `summary` instead of an equation — a
    /// deliberate statement that this entry is not about a reaction, not a
    /// gap in checking.
    pub summary_only: usize,
}

impl Codex {
    /// How much of the prediction layer actually diagnoses.
    ///
    /// Reported rather than enforced: requiring a diagnosis on every
    /// distractor would be right and would also fail the build on entries
    /// written before the field existed. A visible count is the honest
    /// middle — it is a work list, not a pass mark.
    pub fn prediction_audit(&self) -> PredictionAudit {
        let mut audit = PredictionAudit::default();
        for r in &self.reactions {
            let Some(p) = &r.expect.predict else { continue };
            audit.predictions += 1;
            audit.distractors += p.options.len().saturating_sub(1);
            audit.diagnosed += p
                .diagnosis
                .iter()
                .filter(|d| d.option != p.answer && d.option < p.options.len())
                .count();
        }
        audit
    }

    pub fn equation_audit(&self) -> EquationAudit {
        let mut audit = EquationAudit::default();
        for r in &self.reactions {
            match &r.equation {
                Some(e) => {
                    for clause in equation_clauses(e) {
                        if let Ok(eq) = kerotakis_core::stoich::parse_equation(&clause) {
                            if eq.is_balanced() {
                                audit.balanced += 1;
                            }
                        }
                    }
                }
                None => audit.summary_only += 1,
            }
        }
        audit
    }
}

/// The same chemistry at each level of detail, keyed by level number
/// (`lv1`, `lv2`, `lv3`, …). A map rather than named fields so that adding
/// granularity later is a data change, not a schema change. Never
/// generated by a language model: written by a person, checked against the
/// solvers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Registers(pub std::collections::BTreeMap<String, String>);

impl Registers {
    pub fn get(&self, level: u8) -> Option<&str> {
        self.0.get(&format!("lv{level}")).map(String::as_str)
    }

    /// Split a register key into its level and optional locale:
    /// `lv2` -> (`lv2`, None), `lv2_de` -> (`lv2`, Some("de")).
    ///
    /// German prose sits beside the English in the same map, following the
    /// `_de` sibling convention the rest of the catalogue uses, because it
    /// degrades one string at a time: a level nobody has translated yet
    /// falls back to English on its own, rather than the entry needing a
    /// complete German twin before any of it can ship.
    pub fn split_locale(key: &str) -> (&str, Option<&str>) {
        match key.rsplit_once('_') {
            Some((base, loc))
                if !base.is_empty()
                    && !loc.is_empty()
                    && loc.chars().all(|c| c.is_ascii_lowercase()) =>
            {
                (base, Some(loc))
            }
            _ => (key, None),
        }
    }

    /// The prose for a level in the reader's language, falling back to the
    /// English when that level has no translation yet.
    pub fn get_in(&self, level: u8, locale: &str) -> Option<&str> {
        self.0
            .get(&format!("lv{level}_{locale}"))
            .or_else(|| self.0.get(&format!("lv{level}")))
            .map(String::as_str)
    }

    /// Levels present, ascending.
    pub fn levels(&self) -> Vec<u8> {
        let mut out: Vec<u8> = self
            .0
            .keys()
            .filter_map(|k| k.trim_start_matches("lv").parse().ok())
            .collect();
        out.sort_unstable();
        out
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    /// Where the chemistry claim comes from.
    pub source: String,
    /// Licence of that source, where it is a dataset rather than common
    /// textbook knowledge.
    #[serde(default)]
    pub licence: Option<String>,
    /// What computed the numbers, if any were computed.
    #[serde(default)]
    pub computed_by: Option<String>,
}

/// One topic from the curriculum spine.
///
/// The spine is not ours: it is the openeduhub / WirLernenOnline topic
/// taxonomy, published **CC0**, extracted by
/// `tools/extract-oeh-topics.py`. Using someone else's vocabulary is the
/// point — it says what a German chemistry curriculum actually contains,
/// rather than what we happened to think of, and it turns "extend the
/// codex" from a guess into a checklist.
///
/// It carries no year mapping. Placing a topic in a school year is a
/// separate claim, made per curriculum on the entry itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concept {
    pub id: String,
    pub label_de: String,
    #[serde(default)]
    pub definition_de: Option<String>,
    #[serde(default)]
    pub broader: Option<String>,
    /// The concept's id in the source vocabulary.
    pub oeh: String,
    pub source: String,
    /// Translations into languages no field here names.
    ///
    /// `summary_de` is a field; `summary_fr` is not, and without somewhere
    /// to land it would be parsed and then silently discarded. Adding a
    /// language is meant to be a data change — one sidecar file — so the
    /// types must not have to learn its name first.
    #[serde(
        flatten,
        default,
        skip_serializing_if = "std::collections::BTreeMap::is_empty"
    )]
    pub other_locales: std::collections::BTreeMap<String, toml::Value>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Vocabulary {
    #[serde(default, rename = "concept")]
    pub concepts: Vec<Concept>,
}

impl Vocabulary {
    pub fn parse(text: &str) -> Result<Vocabulary, CodexError> {
        Ok(toml::from_str(text)?)
    }

    pub fn get(&self, id: &str) -> Option<&Concept> {
        self.concepts.iter().find(|c| c.id == id)
    }

    /// Spine topics no entry claims yet — the work list.
    pub fn gaps<'a>(&'a self, codex: &Codex) -> Vec<&'a Concept> {
        let claimed: Vec<&str> = codex
            .reactions
            .iter()
            .flat_map(|r| r.spine.iter().map(String::as_str))
            .collect();
        self.concepts
            .iter()
            .filter(|c| !claimed.contains(&c.id.as_str()))
            .collect()
    }
}

/// A model: the thing that explains, orders and systematises.
///
/// The codex treats models as first-class content rather than as
/// background to reactions, because that is the whole argument — school
/// chemistry overfeeds facts and underteaches the models that make facts
/// predictable. A fact learned inside a model is retained and can be
/// re-derived; a fact learned alone is a card in a deck.
///
/// The field that matters most is `fails_at`. A model without a stated
/// boundary is presented as truth, which is both false and the reason
/// learners are blindsided when the next model arrives. Knowing that the
/// particle model cannot explain *why* sodium and chlorine react — and
/// that this is exactly what the next model is for — is what makes the
/// sequence feel like inquiry rather than an arbitrary series of
/// replacements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    pub name: String,
    /// What this model lets a learner *predict or order* that they could
    /// not before. Not "what it says" — what it buys.
    pub power: String,
    /// Phenomena it accounts for.
    #[serde(default)]
    pub explains: Vec<String>,
    /// Where it stops working, in plain terms. The honest edge.
    pub fails_at: Vec<String>,
    /// The model that takes over, and why it is needed.
    #[serde(default)]
    pub superseded_by: Option<String>,
    /// Models a learner needs first.
    #[serde(default)]
    pub requires: Vec<String>,
    /// Where the *engine* stands on this model: which solver embodies it,
    /// so a learner can be shown the boundary rather than told about it.
    #[serde(default)]
    pub embodied_by: Option<String>,
    pub registers: Registers,
    pub provenance: Provenance,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Codex {
    #[serde(default, rename = "reaction")]
    pub reactions: Vec<Entry>,
    #[serde(default, rename = "model")]
    pub models: Vec<Model>,
}

#[derive(Debug, thiserror::Error)]
pub enum CodexError {
    #[error("could not parse the codex: {0}")]
    Parse(#[from] toml::de::Error),
    /// A sidecar key names a path the catalogue does not have. Its own
    /// error, not a parse error: the file is valid TOML and the problem is
    /// that the English it translates has moved or gone.
    #[error("translation does not fit the catalogue: {0}")]
    Translation(String),
}

impl Codex {
    pub fn parse(text: &str) -> Result<Codex, CodexError> {
        Ok(toml::from_str(text)?)
    }

    /// Every `*.toml` in a codex directory, with `<dir>/i18n/*.toml`
    /// folded in as translations.
    ///
    /// Here rather than in each caller because the layout is the codex's
    /// business. The CLI and the export snapshot each walked the directory
    /// themselves, and each would have had to learn where translations
    /// live — which is how two loaders drift until one of them quietly
    /// renders English.
    pub fn load_dir(dir: &std::path::Path) -> Result<Codex, CodexError> {
        let mut sidecars: Vec<(String, String)> = Vec::new();
        let i18n = dir.join("i18n");
        if i18n.is_dir() {
            let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(&i18n)
                .map_err(|e| CodexError::Translation(format!("{}: {e}", i18n.display())))?
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|x| x == "toml"))
                .collect();
            files.sort();
            for file in files {
                let code = file
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .to_string();
                let text = std::fs::read_to_string(&file)
                    .map_err(|e| CodexError::Translation(format!("{}: {e}", file.display())))?;
                sidecars.push((code, text));
            }
        }
        let borrowed: Vec<(&str, &str)> = sidecars
            .iter()
            .map(|(c, t)| (c.as_str(), t.as_str()))
            .collect();

        let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
            .map_err(|e| CodexError::Translation(format!("{}: {e}", dir.display())))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "toml"))
            .collect();
        files.sort();

        let mut all = Codex::default();
        for file in files {
            let text = std::fs::read_to_string(&file)
                .map_err(|e| CodexError::Translation(format!("{}: {e}", file.display())))?;
            // A sidecar covers the whole catalogue, so a key for another
            // file's entry is not an error HERE — it belongs to a sibling.
            // Only a key matching no entry in ANY file is stale, and that
            // is checked once at the end.
            let mut c = Codex::parse_with_translations_for_file(&text, &borrowed)?;
            all.reactions.append(&mut c.reactions);
            all.models.append(&mut c.models);
        }
        Ok(all)
    }

    /// Parse the English source and fold in per-language sidecars.
    ///
    /// Each sidecar is a flat table of `"<entry-id>.<path>" = "…"`, where
    /// the path names the ENGLISH field being translated. The value is
    /// written back as that field's `_<code>` sibling, which is the shape
    /// the structs and the export already use — so nothing downstream
    /// needs to know translations arrived from a different file.
    ///
    /// One file per language is the point. It is what lets a French
    /// translation and a Japanese one be worked on at the same time
    /// without their authors colliding, and it keeps the English source
    /// from growing a full copy per language.
    ///
    /// A key naming a path that does not exist is an error. That means the
    /// English moved and the translation is describing something that is
    /// no longer there — worse than a missing translation, because it
    /// renders confidently.
    pub fn parse_with_translations(
        text: &str,
        sidecars: &[(&str, &str)],
    ) -> Result<Codex, CodexError> {
        Self::merge(text, sidecars, true)
    }

    /// As `parse_with_translations`, but a key whose entry is not in THIS
    /// file is skipped rather than refused: one sidecar covers the whole
    /// directory, so most of its keys belong to sibling files.
    pub fn parse_with_translations_for_file(
        text: &str,
        sidecars: &[(&str, &str)],
    ) -> Result<Codex, CodexError> {
        Self::merge(text, sidecars, false)
    }

    fn merge(
        text: &str,
        sidecars: &[(&str, &str)],
        strict_entries: bool,
    ) -> Result<Codex, CodexError> {
        let mut doc: toml::Value = toml::from_str(text)?;
        for (code, sidecar) in sidecars {
            let table: toml::Value = toml::from_str(sidecar)?;
            let Some(map) = table.as_table() else {
                continue;
            };
            for (path, value) in map {
                match inject(&mut doc, path, code, value.clone()) {
                    Ok(()) => {}
                    Err(Unfit::NoSuchEntry) if !strict_entries => {}
                    Err(e) => {
                        // The PATH belongs in the message: "no entry with that
                        // id" without saying which id is a worse error
                        // than the String one this replaced.
                        return Err(CodexError::Translation(format!("{code}.toml: {path}: {e}")));
                    }
                }
            }
        }
        Ok(doc.try_into()?)
    }

    /// Structural problems that need no solver: duplicate ids, empty
    /// registers, missing provenance, dangling prerequisites.
    pub fn structural_problems(&self) -> Vec<String> {
        let mut problems = Vec::new();
        let mut seen: Vec<&str> = Vec::new();
        let taught: Vec<&str> = self
            .reactions
            .iter()
            .flat_map(|r| r.concepts.iter().map(String::as_str))
            .collect();
        for r in &self.reactions {
            if seen.contains(&r.id.as_str()) {
                problems.push(format!("{}: duplicate id", r.id));
            }
            seen.push(&r.id);
            match (&r.equation, &r.summary) {
                (None, None) => problems.push(format!(
                    "{}: neither an equation nor a summary — say what happens",
                    r.id
                )),
                (Some(e), _) if e.trim().is_empty() => {
                    problems.push(format!("{}: empty equation", r.id))
                }
                _ => {}
            }
            // `equation` is a claim, so it is enforced: it must parse as
            // chemistry and it must balance. Prose belongs in `summary`,
            // and the schema now lets an entry say so — which means an
            // unparseable equation is an error rather than a shrug.
            if let Some(equation) = &r.equation {
                let clauses = equation_clauses(equation);
                let mut parsed_any = false;
                for clause in &clauses {
                    match kerotakis_core::stoich::parse_equation(clause) {
                        Ok(eq) => {
                            parsed_any = true;
                            let bad = eq.element_imbalance();
                            if !bad.is_empty() {
                                let detail: Vec<String> =
                                    bad.iter().map(|(el, d)| format!("{el} {d:+.0}")).collect();
                                problems.push(format!(
                                    "{}: equation does not balance ({}): {clause}",
                                    r.id,
                                    detail.join(", ")
                                ));
                            } else if eq.charge_imbalance().abs() > 1e-6 {
                                problems.push(format!(
                                    "{}: equation conserves atoms but not charge ({:+.0}): {clause}",
                                    r.id,
                                    eq.charge_imbalance()
                                ));
                            }
                        }
                        Err(_) => continue,
                    }
                }
                if !parsed_any && !equation.trim().is_empty() {
                    problems.push(format!(
                        "{}: `equation` does not parse as chemistry — put prose in `summary` instead: {equation}",
                        r.id
                    ));
                }
            }
            if r.setup.script.trim().is_empty() {
                problems.push(format!("{}: no setup script — nothing to verify", r.id));
            }
            if let Some(p) = &r.expect.predict {
                if p.answer >= p.options.len() {
                    problems.push(format!("{}: prediction answer is out of range", r.id));
                }
                for d in &p.diagnosis {
                    if d.option >= p.options.len() {
                        problems.push(format!(
                            "{}: diagnosis points at option {} of {}",
                            r.id,
                            d.option,
                            p.options.len()
                        ));
                    }
                    if d.option == p.answer {
                        problems.push(format!(
                            "{}: a diagnosis is for a wrong answer; option {} is the right one",
                            r.id, d.option
                        ));
                    }
                }
                let mut seen_options: Vec<usize> = Vec::new();
                for d in &p.diagnosis {
                    if seen_options.contains(&d.option) {
                        problems.push(format!("{}: two diagnoses for option {}", r.id, d.option));
                    }
                    seen_options.push(d.option);
                }
                if p.options.len() < 2 {
                    problems.push(format!(
                        "{}: a prediction with one option is not a prediction",
                        r.id
                    ));
                }
            }
            if r.expect.events.is_empty() && r.expect.ph.is_none() {
                problems.push(format!(
                    "{}: claims nothing checkable; an entry the solvers cannot verify is a story, not chemistry",
                    r.id
                ));
            }
            // Every entry must speak at the three established levels; more
            // are welcome and need no code change.
            for level in [1u8, 2, 3] {
                match r.registers.get(level) {
                    Some(t) if !t.trim().is_empty() => {}
                    _ => problems.push(format!("{}: nothing written at lv{level}", r.id)),
                }
            }
            for key in r.registers.0.keys() {
                let (base, locale) = Registers::split_locale(key);
                if Register::parse(base).is_none() {
                    problems.push(format!(
                        "{}: register key '{key}' is not a level (use lv1, lv2, … or lv1_de)",
                        r.id
                    ));
                } else if locale.is_some() && !r.registers.0.contains_key(base) {
                    // A translation of a level that does not exist would
                    // fall back to English forever, silently.
                    problems.push(format!(
                        "{}: register key '{key}' translates '{base}', which is not written",
                        r.id
                    ));
                }
            }
            if r.provenance.source.trim().is_empty() {
                problems.push(format!("{}: no provenance", r.id));
            }
            if r.concepts.is_empty() {
                problems.push(format!("{}: teaches no named concept", r.id));
            }
            for p in &r.curriculum {
                if p.source.trim().is_empty() {
                    problems.push(format!(
                        "{}: placement in '{}' cites no source — a placement is a claim about someone's document",
                        r.id, p.system
                    ));
                }
            }
            for need in &r.requires {
                if !taught.contains(&need.as_str()) {
                    problems.push(format!(
                        "{}: requires '{need}', which no entry teaches",
                        r.id
                    ));
                }
            }
        }
        problems.extend(self.model_problems());
        problems
    }

    /// Structural checks on `[[model]]` entries.
    ///
    /// A model entry cannot be replayed through a solver the way a reaction
    /// can — there is no beaker to look into. What *can* be enforced is that
    /// it states the two things a model entry exists to state: what it buys
    /// (`power`) and where it stops (`fails_at`). An entry missing either is
    /// a definition dressed as a model, which is the failure mode this whole
    /// file is against.
    pub fn model_problems(&self) -> Vec<String> {
        let mut problems = Vec::new();
        let known: Vec<&str> = self.models.iter().map(|m| m.id.as_str()).collect();
        let mut seen: Vec<&str> = Vec::new();
        for m in &self.models {
            if seen.contains(&m.id.as_str()) {
                problems.push(format!("{}: duplicate model id", m.id));
            }
            seen.push(&m.id);
            if m.name.trim().is_empty() {
                problems.push(format!("{}: no name", m.id));
            }
            if m.power.trim().is_empty() {
                problems.push(format!(
                    "{}: states no power — a model that does not say what it lets you predict is a definition, not a model",
                    m.id
                ));
            }
            if m.fails_at.iter().all(|f| f.trim().is_empty()) {
                problems.push(format!(
                    "{}: states no boundary — a model presented without `fails_at` is presented as truth",
                    m.id
                ));
            }
            for level in [1u8, 2, 3] {
                match m.registers.get(level) {
                    Some(t) if !t.trim().is_empty() => {}
                    _ => problems.push(format!("{}: nothing written at lv{level}", m.id)),
                }
            }
            for key in m.registers.0.keys() {
                let (base, locale) = Registers::split_locale(key);
                if Register::parse(base).is_none() {
                    problems.push(format!(
                        "{}: register key '{key}' is not a level (use lv1, lv2, … or lv1_de)",
                        m.id
                    ));
                } else if locale.is_some() && !m.registers.0.contains_key(base) {
                    // A translation of a level that does not exist would
                    // fall back to English forever, silently.
                    problems.push(format!(
                        "{}: register key '{key}' translates '{base}', which is not written",
                        m.id
                    ));
                }
            }
            if m.provenance.source.trim().is_empty() {
                problems.push(format!("{}: no provenance", m.id));
            }
            for need in &m.requires {
                if !known.contains(&need.as_str()) {
                    problems.push(format!(
                        "{}: requires '{need}', which no model entry defines",
                        m.id
                    ));
                }
            }
            if let Some(next) = &m.superseded_by {
                if !known.contains(&next.as_str()) {
                    problems.push(format!(
                        "{}: superseded by '{next}', which no model entry defines",
                        m.id
                    ));
                }
            }
        }
        problems
    }

    /// Model progressions, as chains of `superseded_by`.
    ///
    /// The chain is the pedagogy: each link is a model that failed at
    /// something specific and the model that was built to survive it.
    pub fn model_chains(&self) -> Vec<Vec<&str>> {
        let succeeds: Vec<&str> = self
            .models
            .iter()
            .filter_map(|m| m.superseded_by.as_deref())
            .collect();
        let mut chains = Vec::new();
        for m in self
            .models
            .iter()
            .filter(|m| !succeeds.contains(&m.id.as_str()))
        {
            let mut chain = vec![m.id.as_str()];
            let mut cursor = m;
            while let Some(next) = cursor.superseded_by.as_deref() {
                match self.models.iter().find(|c| c.id == next) {
                    Some(c) if !chain.contains(&next) => {
                        chain.push(next);
                        cursor = c;
                    }
                    _ => break,
                }
            }
            if chain.len() > 1 {
                chains.push(chain);
            }
        }
        chains
    }

    /// Entries placed in a curriculum system, ordered by stage as that
    /// system writes it.
    pub fn by_system(&self, system: &str) -> Vec<(&str, &Entry)> {
        let mut out: Vec<(&str, &Entry)> = self
            .reactions
            .iter()
            .filter_map(|r| {
                r.curriculum
                    .iter()
                    .find(|p| p.system == system)
                    .map(|p| (p.stage.as_str(), r))
            })
            .collect();
        out.sort_by_key(|(stage, _)| stage.to_string());
        out
    }

    /// Coverage: what the codex teaches, grouped so gaps are visible.
    pub fn coverage(&self) -> Coverage {
        let mut c = Coverage::default();
        for r in &self.reactions {
            for x in &r.concepts {
                push_unique(&mut c.concepts, x);
            }
            for x in &r.apparatus {
                push_unique(&mut c.apparatus, x);
            }
            for x in &r.calculations {
                push_unique(&mut c.calculations, x);
            }
            for x in &r.models {
                push_unique(&mut c.models, x);
            }
            for p in &r.curriculum {
                push_unique(&mut c.systems, &p.system);
            }
        }
        c
    }

    /// A teaching order derived from prerequisites, not from school years.
    ///
    /// School years are an artefact of national administration; the
    /// dependency structure of the ideas is not. Topological order over
    /// `requires` gives a sequence that is defensible from the subject
    /// itself, and curriculum placements remain on each entry so a learner
    /// who *does* need to find their syllabus topic still can.
    ///
    /// Entries whose prerequisites are missing come last rather than
    /// vanishing: a broken chain should be visible, not silently dropped.
    pub fn teaching_order(&self) -> Vec<&Entry> {
        let mut placed: Vec<&str> = Vec::new();
        let mut out: Vec<&Entry> = Vec::new();
        let mut pending: Vec<&Entry> = self.reactions.iter().collect();
        loop {
            let (ready, rest): (Vec<&Entry>, Vec<&Entry>) = pending
                .into_iter()
                .partition(|e| e.requires.iter().all(|r| placed.contains(&r.as_str())));
            if ready.is_empty() {
                out.extend(rest);
                return out;
            }
            for e in &ready {
                for c in &e.concepts {
                    if !placed.contains(&c.as_str()) {
                        placed.push(c);
                    }
                }
            }
            out.extend(ready);
            pending = rest;
            if pending.is_empty() {
                return out;
            }
        }
    }

    /// Every concept the codex teaches, with the entries that teach it.
    pub fn concept_index(&self) -> Vec<(String, Vec<String>)> {
        let mut index: std::collections::BTreeMap<String, Vec<String>> = Default::default();
        for r in &self.reactions {
            for c in &r.concepts {
                index.entry(c.clone()).or_default().push(r.id.clone());
            }
        }
        index.into_iter().collect()
    }
}

/// The full codex payload for the web app: entries, models, and the
/// concepts graph in a single JSON file. The shape is the serde of
/// the existing structs — Rust is the source of truth.
#[derive(Debug, Serialize, Deserialize)]
pub struct CodexExport {
    pub reactions: Vec<Entry>,
    pub models: Vec<Model>,
    pub concepts: Vec<Concept>,
}

impl CodexExport {
    pub fn build(codex: &Codex, vocabulary: &Vocabulary) -> Self {
        Self {
            reactions: codex.reactions.clone(),
            models: codex.models.clone(),
            concepts: vocabulary.concepts.clone(),
        }
    }
}

/// What the codex covers, for seeing what it does not.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Coverage {
    pub concepts: Vec<String>,
    pub apparatus: Vec<String>,
    pub calculations: Vec<String>,
    pub models: Vec<String>,
    pub systems: Vec<String>,
}

fn push_unique(v: &mut Vec<String>, item: &str) {
    if !v.iter().any(|e| e == item) {
        v.push(item.to_string());
    }
}

/// Does an observed event match a claim like `precipitated:AgCl`?
pub fn event_matches(event: &kerotakis_core::Event, claim: &str) -> bool {
    use kerotakis_core::Event as E;
    let (kind, want) = match claim.split_once(':') {
        Some((k, s)) => (k, Some(s)),
        None => (claim, None),
    };
    let (actual_kind, actual_species): (&str, Option<&str>) = match event {
        E::Precipitated { species, .. } => ("precipitated", Some(species.0.as_str())),
        E::Dissolved { species, .. } => ("dissolved", Some(species.0.as_str())),
        E::GasEvolved { species, .. } => ("gas_evolved", Some(species.0.as_str())),
        E::GasAbsorbed { species, .. } => ("gas_absorbed", Some(species.0.as_str())),
        E::GasContained { species, .. } => ("gas_contained", Some(species.0.as_str())),
        E::VesselSealed { .. } => ("vessel_sealed", None),
        E::VesselPressureControlled { .. } => ("vessel_pressure_controlled", None),
        E::VesselSwept { .. } => ("vessel_swept", None),
        E::VesselOpened { .. } => ("vessel_opened", None),
        E::HeadspaceEquilibrated { .. } => ("headspace_equilibrated", None),
        E::Consumed { species, .. } => ("consumed", Some(species.0.as_str())),
        E::Plated { species, .. } => ("plated", Some(species.0.as_str())),
        E::Inert { species, .. } => ("inert", Some(species.0.as_str())),
        E::Electrolysed { species, .. } => ("electrolysed", Some(species.0.as_str())),
        E::CellVoltage { .. } => ("cell_voltage", None),
        E::NoCell { .. } => ("no_cell", None),
        E::Added { species, .. } => ("added", Some(species.0.as_str())),
        E::MaterialAdded { .. } => ("material_added", None),
        E::GasProduced { species, .. } => ("gas_produced", Some(species.0.as_str())),
        E::Fermented { .. } => ("fermented", None),
        E::ReactionHeatReleased { .. } => ("reaction_heat_released", None),
        E::FoamChanged { .. } => ("foam_changed", None),
        E::SurfaceSpread { .. } => ("surface_spread", None),
        E::SurfaceColourSpread { .. } => ("surface_colour_spread", None),
        E::SurfaceColourMixed { .. } => ("surface_colour_mixed", None),
        E::FlameTest { species, .. } => ("flame_test", Some(species.0.as_str())),
        E::Ignited { .. } => ("ignited", None),
        E::DidNotIgnite { .. } => ("did_not_ignite", None),
        E::HazardWarning { .. } => ("hazard_warning", None),
        E::SpillCreated { .. } => ("spill_created", None),
        E::ContainerBroken { .. } => ("container_broken", None),
        E::CollisionWithstood { .. } => ("collision_withstood", None),
        E::SpillRecovered { .. } => ("spill_recovered", None),
        E::SpillHazard { .. } => ("spill_hazard", None),
        E::SafetyVeto { .. } => ("safety_veto", None),
        // BRD-002: the shelf's own two events. `key` is a shelf key rather
        // than a species id — a material recipe has no single species — so
        // a claim names it the same way `add` does: `stock_exhausted:NaCl`.
        E::ShelfStocked { key, .. } => ("shelf_stocked", Some(key.as_str())),
        E::StockExhausted { key, .. } => ("stock_exhausted", Some(key.as_str())),
        E::ReactionOccurred { .. } => ("reaction", None),
        E::SolutionCharacterized { .. } => ("solution", None),
        E::ThermalEquilibrium { .. } => ("thermal_equilibrium", None),
        E::TemperatureChanged { .. } => ("temperature_changed", None),
        E::EnergyTransferred { .. } => ("energy_transferred", None),
        E::Stirred { .. } => ("stirred", None),
        E::EmulsionChanged { .. } => ("emulsion_changed", None),
        E::CurdlingChanged { .. } => ("curdling_changed", None),
        E::Ground { species, .. } => ("ground", Some(species.0.as_str())),
        E::Centrifuged { .. } => ("centrifuged", None),
        E::Irradiated { .. } => ("irradiated", None),
        E::GravitySettled { .. } => ("gravity_settled", None),
        E::Evaporated { .. } => ("evaporated", None),
        E::Distilled { .. } => ("distilled", None),
        E::LayersFormed { .. } => ("layers_formed", None),
        E::MaterialLayersFormed { .. } => ("material_layers_formed", None),
        E::Drained { .. } => ("drained", None),
        E::Partitioned { species, .. } => ("partitioned", Some(species.0.as_str())),
        E::Chromatographed { .. } => ("chromatographed", None),
        E::OrgReacted { name, .. } => ("org_reacted", Some(name.as_str())),
        E::Smelled { .. } => ("smelled", None),
        E::GasTested { .. } => ("gas_tested", None),
        E::Burst { .. } => ("burst", None),
        E::HeatOfMixing { .. } => ("heat_of_mixing", None),
        E::NuclideSpiked { nuclide, .. } => ("nuclide_spiked", Some(nuclide.as_str())),
        E::Decayed { parent, .. } => ("decayed", Some(parent.as_str())),
        E::DissolvedInSolvent { species, .. } => ("dissolved_in_solvent", Some(species.0.as_str())),
        E::InertInSolvent { species, .. } => ("inert_in_solvent", Some(species.0.as_str())),
        E::Filtered { .. } => ("filtered", None),
        E::MagnetSeparated { .. } => ("magnet_separated", None),
        E::Transferred { .. } => ("transferred", None),
        E::Measured { .. } => ("measured", None),
        E::Observed { .. } => ("observed", None),
        E::VesselCreated { .. } => ("vessel_created", None),
        E::VesselRemoved { .. } => ("vessel_removed", None),
        E::NotYetModeled { .. } => ("not_yet_modelled", None),
        E::SolverFailed { .. } => ("solver_failed", None),
        // Keyed by reaction id, so an entry asserts `reacted:thiosulfate-acid`
        // rather than the weaker "something reacted".
        E::Diluted { .. } => ("diluted", None),
        E::Titrated { titrant, .. } => ("titrated", Some(titrant.0.as_str())),
        E::Mixed { .. } => ("mixed", None),
        E::Transported { .. } => ("transported", None),
        E::Reacted { reaction, .. } => ("reacted", Some(reaction.as_str())),
        // Named per direction so an entry can assert `froze:water` rather
        // than the weaker "some state changed".
        E::StateChanged {
            species, from, to, ..
        } => (
            match (from, to) {
                (Phase::Liquid, Phase::Solid) => "froze",
                (Phase::Solid, Phase::Liquid) => "melted",
                (Phase::Liquid, Phase::Gas) => "boiled",
                _ => "state_changed",
            },
            Some(species.0.as_str()),
        ),
    };
    if actual_kind != kind {
        return false;
    }
    match (want, actual_species) {
        (None, _) => true,
        (Some(w), Some(a)) => w == a,
        (Some(_), None) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
[[reaction]]
id = "test"
summary = "a placeholder, deliberately not a chemical equation"
concepts = ["thing"]

[reaction.setup]
script = "add v1 water 100mL"

[reaction.expect]
events = ["added:water"]

[reaction.registers]
lv1 = "c"
lv2 = "s"
lv3 = "e"

[reaction.provenance]
source = "textbook"
"#;

    #[test]
    fn parses_and_validates() {
        let codex = Codex::parse(SAMPLE).expect("parses");
        assert_eq!(codex.reactions.len(), 1);
        assert!(codex.structural_problems().is_empty());
        assert_eq!(codex.concept_index()[0].0, "thing");
    }

    #[test]
    fn catches_an_entry_that_claims_nothing() {
        let text = SAMPLE.replace("events = [\"added:water\"]", "events = []");
        let codex = Codex::parse(&text).expect("parses");
        let problems = codex.structural_problems();
        assert!(
            problems
                .iter()
                .any(|p| p.contains("claims nothing checkable")),
            "{problems:?}"
        );
    }

    const MODEL: &str = r#"
[[model]]
id = "teilchenmodell"
name = "Particle model (Teilchenmodell)"
power = "lets you predict that a dissolved solid is still there"
fails_at = ["cannot say why two substances react"]
superseded_by = "dalton"

[model.registers]
lv1 = "c"
lv2 = "s"
lv3 = "e"

[model.provenance]
source = "standard curriculum"

[[model]]
id = "dalton"
name = "Dalton's atomic model"
power = "lets you balance an equation"
fails_at = ["has no electrons, so it cannot explain ions"]
requires = ["teilchenmodell"]

[model.registers]
lv1 = "c"
lv2 = "s"
lv3 = "e"

[model.provenance]
source = "Dalton, A New System of Chemical Philosophy (1808)"
"#;

    #[test]
    fn models_parse_and_chain() {
        let codex = Codex::parse(MODEL).expect("parses");
        assert_eq!(codex.models.len(), 2);
        assert!(codex.structural_problems().is_empty());
        assert_eq!(codex.model_chains(), vec![vec!["teilchenmodell", "dalton"]]);
    }

    #[test]
    fn catches_a_model_with_no_boundary() {
        let text = MODEL.replace(
            "fails_at = [\"cannot say why two substances react\"]",
            "fails_at = []",
        );
        let codex = Codex::parse(&text).expect("parses");
        assert!(
            codex
                .structural_problems()
                .iter()
                .any(|p| p.contains("states no boundary")),
            "a model without fails_at must not pass"
        );
    }

    #[test]
    fn catches_a_dangling_prerequisite() {
        let text = SAMPLE.replace(
            "concepts = [\"thing\"]",
            "concepts = [\"thing\"]\nrequires = [\"nowhere\"]",
        );
        let codex = Codex::parse(&text).expect("parses");
        assert!(codex
            .structural_problems()
            .iter()
            .any(|p| p.contains("which no entry teaches")));
    }

    #[test]
    fn an_unbalanced_equation_fails_the_build() {
        let toml = r#"
[[reaction]]
id = "wrong"
equation = "Mg + O₂ → MgO"
concepts = ["thing"]

[reaction.setup]
script = "add v1 water 100mL"

[reaction.expect]
events = []

[reaction.registers]
lv1 = "a"
lv2 = "b"
lv3 = "c"

[reaction.provenance]
source = "test"
"#;
        let codex: Codex = toml::from_str(toml).expect("parse");
        let problems = codex.structural_problems();
        assert!(
            problems.iter().any(|p| p.contains("does not balance")),
            "an equation claiming to be balanced must be checked: {problems:?}"
        );
    }

    #[test]
    fn atoms_balancing_is_not_enough_charge_is_checked_too() {
        let toml = r#"
[[reaction]]
id = "charged"
equation = "Fe²⁺ → Fe³⁺"
concepts = ["thing"]

[reaction.setup]
script = "add v1 water 100mL"

[reaction.expect]
events = []

[reaction.registers]
lv1 = "a"
lv2 = "b"
lv3 = "c"

[reaction.provenance]
source = "test"
"#;
        let codex: Codex = toml::from_str(toml).expect("parse");
        let problems = codex.structural_problems();
        assert!(
            problems.iter().any(|p| p.contains("not charge")),
            "{problems:?}"
        );
    }

    #[test]
    fn prose_in_the_equation_field_is_counted_not_ignored() {
        let toml = r#"
[[reaction]]
id = "prose"
summary = "CH₃COOH / CH₃COO⁻ buffer"
concepts = ["thing"]

[reaction.setup]
script = "add v1 water 100mL"

[reaction.expect]
events = []

[reaction.registers]
lv1 = "a"
lv2 = "b"
lv3 = "c"

[reaction.provenance]
source = "test"
"#;
        let codex: Codex = toml::from_str(toml).expect("parse");
        // Not an equation error — the field is used for summaries too —
        // but it must show up in the audit rather than passing as verified.
        assert!(
            !codex
                .structural_problems()
                .iter()
                .any(|p| p.contains("balance")),
            "prose is not an unbalanced equation: {:?}",
            codex.structural_problems()
        );
        let audit = codex.equation_audit();
        assert_eq!(audit.balanced, 0);
        assert_eq!(audit.summary_only, 1);
    }

    #[test]
    fn a_chain_of_equilibria_is_checked_pairwise() {
        let clauses = equation_clauses("H₃PO₄ ⇌ H⁺ + H₂PO₄⁻ ⇌ 2 H⁺ + HPO₄²⁻");
        assert_eq!(clauses.len(), 2, "{clauses:?}");
        for c in clauses {
            let eq = kerotakis_core::stoich::parse_equation(&c).expect("parses");
            assert!(eq.is_balanced(), "{c}: {:?}", eq.element_imbalance());
        }
    }

    #[test]
    fn prose_in_the_equation_field_is_now_an_error() {
        // The point of splitting the fields: declaring something an
        // equation is a claim, so an entry that puts prose there is told to
        // use `summary` rather than being quietly waved through.
        let toml = r#"
[[reaction]]
id = "confused"
equation = "CH₃COOH / CH₃COO⁻ buffer"
concepts = ["thing"]

[reaction.setup]
script = "add v1 water 100mL"

[reaction.expect]
events = []

[reaction.registers]
lv1 = "a"
lv2 = "b"
lv3 = "c"

[reaction.provenance]
source = "test"
"#;
        let codex: Codex = toml::from_str(toml).expect("parse");
        let problems = codex.structural_problems();
        assert!(
            problems.iter().any(|p| p.contains("does not parse")),
            "{problems:?}"
        );
    }

    #[test]
    fn an_entry_must_say_something() {
        let toml = r#"
[[reaction]]
id = "silent"
concepts = ["thing"]

[reaction.setup]
script = "add v1 water 100mL"

[reaction.expect]
events = []

[reaction.registers]
lv1 = "a"
lv2 = "b"
lv3 = "c"

[reaction.provenance]
source = "test"
"#;
        let codex: Codex = toml::from_str(toml).expect("parse");
        assert!(codex
            .structural_problems()
            .iter()
            .any(|p| p.contains("neither an equation nor a summary")));
    }

    fn with_predict(extra: &str) -> Codex {
        let toml = format!(
            r#"
[[reaction]]
id = "p"
summary = "a test"
concepts = ["thing"]

[reaction.setup]
script = "add v1 water 100mL"

[reaction.expect]
events = []

[reaction.expect.predict]
question = "what happens?"
options = ["right", "wrong one", "wrong two"]
answer = 0
{extra}

[reaction.registers]
lv1 = "a"
lv2 = "b"
lv3 = "c"

[reaction.provenance]
source = "test"
"#
        );
        toml::from_str(&toml).expect("parse")
    }

    #[test]
    fn a_diagnosis_must_point_at_a_real_option() {
        let codex =
            with_predict("[[reaction.expect.predict.diagnosis]]\noption = 7\nreveals = \"nope\"");
        assert!(codex
            .structural_problems()
            .iter()
            .any(|p| p.contains("points at option 7")));
    }

    #[test]
    fn a_diagnosis_may_not_be_attached_to_the_right_answer() {
        // A diagnosis explains what believing a *wrong* answer reveals.
        // Attaching one to the correct option means the entry has its
        // answer index wrong, or has misunderstood the field.
        let codex =
            with_predict("[[reaction.expect.predict.diagnosis]]\noption = 0\nreveals = \"nope\"");
        assert!(codex
            .structural_problems()
            .iter()
            .any(|p| p.contains("is the right one")));
    }

    #[test]
    fn two_diagnoses_for_one_option_is_an_error() {
        let codex = with_predict(
            "[[reaction.expect.predict.diagnosis]]\noption = 1\nreveals = \"a\"\n\n[[reaction.expect.predict.diagnosis]]\noption = 1\nreveals = \"b\"",
        );
        assert!(codex
            .structural_problems()
            .iter()
            .any(|p| p.contains("two diagnoses")));
    }

    #[test]
    fn the_audit_counts_distractors_not_options() {
        let codex =
            with_predict("[[reaction.expect.predict.diagnosis]]\noption = 1\nreveals = \"a\"");
        let a = codex.prediction_audit();
        assert_eq!(a.predictions, 1);
        assert_eq!(a.distractors, 2, "three options, one of them right");
        assert_eq!(a.diagnosed, 1);
    }
}
