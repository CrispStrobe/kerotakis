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

use kerotakis_core::{Phase, Register};
use serde::{Deserialize, Serialize};

/// One curated reaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    /// Stable slug, used by lessons and the concept graph.
    pub id: String,
    /// Balanced equation, shown from the student register up.
    pub equation: String,
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
    /// Plausible answers to choose between; exactly one is right, and the
    /// wrong ones should be the mistakes learners actually make, not
    /// strawmen.
    pub options: Vec<String>,
    /// Index into `options`.
    pub answer: usize,
    /// Why the tempting wrong answer is tempting — the misconception this
    /// question exists to surface.
    #[serde(default)]
    pub misconception: Option<String>,
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
}

impl Codex {
    pub fn parse(text: &str) -> Result<Codex, CodexError> {
        Ok(toml::from_str(text)?)
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
            if r.equation.trim().is_empty() {
                problems.push(format!("{}: no equation", r.id));
            }
            if r.setup.script.trim().is_empty() {
                problems.push(format!("{}: no setup script — nothing to verify", r.id));
            }
            if let Some(p) = &r.expect.predict {
                if p.answer >= p.options.len() {
                    problems.push(format!("{}: prediction answer is out of range", r.id));
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
                if Register::parse(key).is_none() {
                    problems.push(format!(
                        "{}: register key '{key}' is not a level (use lv1, lv2, …)",
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
                if Register::parse(key).is_none() {
                    problems.push(format!(
                        "{}: register key '{key}' is not a level (use lv1, lv2, …)",
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
        E::Consumed { species, .. } => ("consumed", Some(species.0.as_str())),
        E::Added { species, .. } => ("added", Some(species.0.as_str())),
        E::FlameTest { species, .. } => ("flame_test", Some(species.0.as_str())),
        E::Ignited { .. } => ("ignited", None),
        E::DidNotIgnite { .. } => ("did_not_ignite", None),
        E::HazardWarning { .. } => ("hazard_warning", None),
        E::SafetyVeto { .. } => ("safety_veto", None),
        E::ReactionOccurred { .. } => ("reaction", None),
        E::SolutionCharacterized { .. } => ("solution", None),
        E::ThermalEquilibrium { .. } => ("thermal_equilibrium", None),
        E::TemperatureChanged { .. } => ("temperature_changed", None),
        E::Evaporated { .. } => ("evaporated", None),
        E::Filtered { .. } => ("filtered", None),
        E::Transferred { .. } => ("transferred", None),
        E::Measured { .. } => ("measured", None),
        E::Observed { .. } => ("observed", None),
        E::VesselCreated { .. } => ("vessel_created", None),
        E::NotYetModeled { .. } => ("not_yet_modelled", None),
        E::SolverFailed { .. } => ("solver_failed", None),
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
equation = "A + B → C"
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
}
