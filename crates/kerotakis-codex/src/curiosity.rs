//! BRD-000: the versioned curiosity corpus.
//!
//! This module owns authored TOML and structural lint only. Executing prompts
//! through the real solver stack and classifying their typed outcomes belongs
//! to the CLI coverage runner (BRD-001). Keeping the corpus in the codex crate
//! follows the same boundary as quests: authored scientific content is parsed
//! and linted here; hosts decide how and when to execute it.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// The only on-disk manifest version understood by this release.
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CuriosityManifest {
    pub schema_version: u32,
    pub id: String,
    pub description: String,
    pub target_prompts: usize,
    pub smoke_prompts: Vec<String>,
    pub shards: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CuriosityShard {
    #[serde(default)]
    pub prompt: Vec<CuriosityPrompt>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CuriosityPrompt {
    /// Stable slug. Baselines and ownership reports join on this field.
    pub id: String,
    /// The learner's wording, retained even though execution uses `script`.
    pub question: String,
    pub age_band: AgeBand,
    pub action: ActionFamily,
    pub material_class: String,
    pub expected: Disposition,
    /// CAP/EXP/BRD identifier that owns the current expected route or gap.
    pub owning_task: String,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Canonical `.lab` commands. Empty only for an explicit boundary.
    #[serde(default)]
    pub script: Vec<String>,
    /// Stable boundary code, not prose. Required exactly when expected is
    /// `boundary`; the UI/codex owns localized explanation separately.
    #[serde(default)]
    pub boundary: Option<String>,
    /// Expected typed parser failure for an intentionally unsupported input.
    /// This is separate from `boundary`, which describes a deliberate product
    /// boundary that is never submitted as a bench command.
    #[serde(default)]
    pub parse_boundary: Option<kerotakis_core::script::ParseErrorKind>,
    /// Small deterministic subset run by ordinary CI.
    #[serde(default)]
    pub smoke: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Disposition {
    Computed,
    Curated,
    Qualitative,
    Boundary,
    Missing,
}

impl Disposition {
    pub const ALL: [Self; 5] = [
        Self::Computed,
        Self::Curated,
        Self::Qualitative,
        Self::Boundary,
        Self::Missing,
    ];
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum AgeBand {
    Age9To12,
    Age13To15,
    Age16To18,
    All,
}

impl AgeBand {
    pub const LEARNER_BANDS: [Self; 3] = [Self::Age9To12, Self::Age13To15, Self::Age16To18];
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ActionFamily {
    MixAndDissolve,
    HeatAndCool,
    BurnAndOxidise,
    AcidsBasesAndGases,
    Separate,
    Materials,
    FoodAndLife,
    HandleAndInspect,
}

impl ActionFamily {
    pub const ALL: [Self; 8] = [
        Self::MixAndDissolve,
        Self::HeatAndCool,
        Self::BurnAndOxidise,
        Self::AcidsBasesAndGases,
        Self::Separate,
        Self::Materials,
        Self::FoodAndLife,
        Self::HandleAndInspect,
    ];
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CuriosityCorpus {
    pub manifest: CuriosityManifest,
    pub prompts: Vec<CuriosityPrompt>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CuriosityInventory {
    pub corpus: String,
    pub target_prompts: usize,
    pub prompts: usize,
    pub smoke_prompts: usize,
    pub complete: bool,
    pub by_expected: BTreeMap<Disposition, usize>,
    pub by_action: BTreeMap<ActionFamily, usize>,
    pub by_age_band: BTreeMap<AgeBand, usize>,
    pub by_owning_task: BTreeMap<String, usize>,
}

impl CuriosityCorpus {
    pub fn inventory(&self) -> CuriosityInventory {
        let mut by_expected = Disposition::ALL
            .into_iter()
            .map(|kind| (kind, 0))
            .collect::<BTreeMap<_, _>>();
        let mut by_action = BTreeMap::new();
        let mut by_age_band = BTreeMap::new();
        let mut by_owning_task = BTreeMap::new();
        for prompt in &self.prompts {
            *by_expected.entry(prompt.expected).or_default() += 1;
            *by_action.entry(prompt.action).or_default() += 1;
            *by_age_band.entry(prompt.age_band).or_default() += 1;
            *by_owning_task
                .entry(prompt.owning_task.clone())
                .or_default() += 1;
        }
        CuriosityInventory {
            corpus: self.manifest.id.clone(),
            target_prompts: self.manifest.target_prompts,
            prompts: self.prompts.len(),
            smoke_prompts: self.manifest.smoke_prompts.len(),
            complete: self.prompts.len() >= self.manifest.target_prompts,
            by_expected,
            by_action,
            by_age_band,
            by_owning_task,
        }
    }

    /// Structural problems only. Solver truth is BRD-001's separate pass.
    pub fn lint(&self) -> Vec<String> {
        let mut problems = Vec::new();
        if self.manifest.schema_version != SCHEMA_VERSION {
            problems.push(format!(
                "manifest schema_version {} is unsupported (expected {SCHEMA_VERSION})",
                self.manifest.schema_version
            ));
        }
        if !valid_slug(&self.manifest.id) {
            problems.push(format!(
                "manifest id '{}' is not a stable slug",
                self.manifest.id
            ));
        }
        if self.manifest.description.trim().is_empty() {
            problems.push("manifest description is empty".to_string());
        }
        if self.manifest.target_prompts == 0 {
            problems.push("manifest target_prompts must be positive".to_string());
        }
        if self.manifest.shards.is_empty() {
            problems.push("manifest has no shards".to_string());
        }
        if self.manifest.target_prompts >= 16 && self.manifest.smoke_prompts.len() < 16 {
            problems.push("manifest smoke_prompts must contain at least 16 ids".to_string());
        }
        if self.prompts.len() != self.manifest.target_prompts {
            problems.push(format!(
                "corpus has {} prompts; manifest requires exactly {}",
                self.prompts.len(),
                self.manifest.target_prompts
            ));
        }

        let mut ids = BTreeSet::new();
        let mut normalized_questions = BTreeMap::<String, String>::new();
        for prompt in &self.prompts {
            let at = format!("prompt {}", prompt.id);
            if !valid_slug(&prompt.id) {
                problems.push(format!("{at}: id is not a stable slug"));
            }
            if !ids.insert(prompt.id.clone()) {
                problems.push(format!("{at}: duplicate id"));
            }
            if prompt.question.trim().is_empty() {
                problems.push(format!("{at}: question is empty"));
            } else {
                let normalized = normalize_question(&prompt.question);
                if let Some(first) = normalized_questions.insert(normalized, prompt.id.clone()) {
                    problems.push(format!(
                        "{at}: duplicates normalized question from prompt {first}"
                    ));
                }
            }
            if prompt.material_class.trim().is_empty() {
                problems.push(format!("{at}: material_class is empty"));
            }
            if !valid_owner(&prompt.owning_task) {
                problems.push(format!(
                    "{at}: owning_task '{}' is not CAP-*, EXP-* or BRD-*",
                    prompt.owning_task
                ));
            }
            if prompt.tags.is_empty() {
                problems.push(format!("{at}: tags are empty"));
            }
            let unique_tags = prompt.tags.iter().collect::<BTreeSet<_>>();
            if unique_tags.len() != prompt.tags.len() {
                problems.push(format!("{at}: tags contain duplicates"));
            }
            if !prompt.tags.windows(2).all(|pair| pair[0] < pair[1]) {
                problems.push(format!("{at}: tags must be sorted and unique"));
            }

            match prompt.expected {
                Disposition::Boundary => {
                    if !prompt.script.is_empty() {
                        problems.push(format!("{at}: boundary prompt must not carry a script"));
                    }
                    if !prompt.boundary.as_deref().is_some_and(valid_slug) {
                        problems.push(format!(
                            "{at}: boundary prompt needs a stable boundary code"
                        ));
                    }
                    if prompt.parse_boundary.is_some() {
                        problems.push(format!("{at}: product boundary carries parse_boundary"));
                    }
                }
                _ => {
                    if prompt.script.is_empty() {
                        problems.push(format!("{at}: runnable prompt has no script"));
                    }
                    if prompt.boundary.is_some() {
                        problems.push(format!("{at}: non-boundary prompt carries boundary code"));
                    }
                    if !matches!(prompt.expected, Disposition::Missing)
                        && prompt.parse_boundary.is_some()
                    {
                        problems.push(format!(
                            "{at}: only a missing prompt may expect a parser boundary"
                        ));
                    }
                }
            }
            let mut observed_parse_boundary = None;
            for (index, line) in prompt.script.iter().enumerate() {
                if line.trim().is_empty() || line.trim_start().starts_with('#') {
                    problems.push(format!(
                        "{at}: script line {} is empty/comment-only",
                        index + 1
                    ));
                }
                // Execution stops at the first parser failure. Commands after
                // an explicitly expected boundary are documentary context,
                // not reachable input, so do not invent additional failures.
                if observed_parse_boundary.is_some() {
                    continue;
                }
                match kerotakis_core::script::parse_op_typed(line) {
                    Ok(Some(_)) => {}
                    Ok(None) => problems.push(format!(
                        "{at}: script line {} is a session command, not an operator",
                        index + 1
                    )),
                    Err(error) if matches!(prompt.expected, Disposition::Missing) => {
                        observed_parse_boundary = Some(error.kind);
                    }
                    Err(error) => problems.push(format!(
                        "{at}: script line {} does not parse ({}): {}",
                        index + 1,
                        parse_error_code(error.kind),
                        error
                    )),
                }
            }
            if prompt.parse_boundary != observed_parse_boundary {
                problems.push(format!(
                    "{at}: declared parse_boundary {:?}, observed {:?}",
                    prompt.parse_boundary, observed_parse_boundary
                ));
            }
        }
        let smoke_ids = self.manifest.smoke_prompts.iter().collect::<BTreeSet<_>>();
        if smoke_ids.len() != self.manifest.smoke_prompts.len() {
            problems.push("manifest smoke_prompts contains duplicate ids".to_string());
        }
        let prompt_ids = self
            .prompts
            .iter()
            .map(|prompt| &prompt.id)
            .collect::<BTreeSet<_>>();
        for smoke_id in smoke_ids {
            if !prompt_ids.contains(smoke_id) {
                problems.push(format!("manifest smoke prompt '{smoke_id}' does not exist"));
            }
        }
        if self.manifest.target_prompts >= 16 {
            let smoke = self
                .prompts
                .iter()
                .filter(|prompt| self.is_smoke(prompt))
                .collect::<Vec<_>>();
            for action in ActionFamily::ALL {
                if !smoke.iter().any(|prompt| prompt.action == action) {
                    problems.push(format!("smoke set does not cover action {action:?}"));
                }
            }
            for age_band in AgeBand::LEARNER_BANDS {
                if !smoke.iter().any(|prompt| prompt.age_band == age_band) {
                    problems.push(format!("smoke set does not cover age band {age_band:?}"));
                }
            }
        }
        problems
    }

    pub fn is_smoke(&self, prompt: &CuriosityPrompt) -> bool {
        self.manifest
            .smoke_prompts
            .iter()
            .any(|id| id == &prompt.id)
    }
}

pub fn parse_manifest(text: &str) -> Result<CuriosityManifest, String> {
    toml::from_str(text).map_err(|error| error.to_string())
}

pub fn parse_shard(text: &str) -> Result<CuriosityShard, String> {
    toml::from_str(text).map_err(|error| error.to_string())
}

/// Load a manifest and its explicitly ordered shards. Paths may not escape the
/// manifest directory; this is authored content, not a general file loader.
pub fn load_manifest(path: &Path) -> Result<CuriosityCorpus, String> {
    let manifest_text = std::fs::read_to_string(path)
        .map_err(|error| format!("reading {}: {error}", path.display()))?;
    let manifest = parse_manifest(&manifest_text)
        .map_err(|error| format!("parsing {}: {error}", path.display()))?;
    let directory = path.parent().unwrap_or_else(|| Path::new("."));
    let mut prompts = Vec::new();
    let mut seen_shards = BTreeSet::new();
    for shard_name in &manifest.shards {
        let shard_path = safe_shard_path(directory, shard_name)?;
        if !seen_shards.insert(shard_name.clone()) {
            return Err(format!("manifest repeats shard '{shard_name}'"));
        }
        let text = std::fs::read_to_string(&shard_path)
            .map_err(|error| format!("reading {}: {error}", shard_path.display()))?;
        let mut shard = parse_shard(&text)
            .map_err(|error| format!("parsing {}: {error}", shard_path.display()))?;
        prompts.append(&mut shard.prompt);
    }
    Ok(CuriosityCorpus { manifest, prompts })
}

fn safe_shard_path(directory: &Path, name: &str) -> Result<PathBuf, String> {
    let relative = Path::new(name);
    if relative.is_absolute()
        || relative
            .components()
            .any(|part| matches!(part, std::path::Component::ParentDir))
        || relative.extension().and_then(|value| value.to_str()) != Some("toml")
    {
        return Err(format!("unsafe shard path '{name}'"));
    }
    Ok(directory.join(relative))
}

fn valid_slug(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && !value.starts_with('-')
        && !value.ends_with('-')
}

fn valid_owner(value: &str) -> bool {
    ["CAP-", "EXP-", "BRD-"].iter().any(|prefix| {
        value
            .strip_prefix(prefix)
            .is_some_and(|tail| !tail.is_empty())
    })
}

fn normalize_question(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_alphanumeric() || character.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_error_code(kind: kerotakis_core::script::ParseErrorKind) -> &'static str {
    use kerotakis_core::script::ParseErrorKind;
    match kind {
        ParseErrorKind::UnknownSpecies => "unknown_species",
        ParseErrorKind::UnknownReaction => "unknown_reaction",
        ParseErrorKind::InvalidSyntax => "invalid_syntax",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prompt(id: &str) -> CuriosityPrompt {
        CuriosityPrompt {
            id: id.to_string(),
            question: "What happens if salt meets water?".to_string(),
            age_band: AgeBand::Age9To12,
            action: ActionFamily::MixAndDissolve,
            material_class: "household-salt".to_string(),
            expected: Disposition::Computed,
            owning_task: "CAP-10".to_string(),
            tags: vec!["aqueous".to_string(), "salt".to_string()],
            script: vec!["add v1 water 100mL".to_string()],
            boundary: None,
            parse_boundary: None,
            smoke: true,
        }
    }

    fn corpus(prompts: Vec<CuriosityPrompt>) -> CuriosityCorpus {
        CuriosityCorpus {
            manifest: CuriosityManifest {
                schema_version: SCHEMA_VERSION,
                id: "curiosity-test".to_string(),
                description: "test".to_string(),
                target_prompts: prompts.len(),
                smoke_prompts: prompts.iter().map(|prompt| prompt.id.clone()).collect(),
                shards: vec!["core.toml".to_string()],
            },
            prompts,
        }
    }

    #[test]
    fn a_minimal_corpus_is_sound_and_inventoried() {
        let corpus = corpus(vec![prompt("salt-water")]);
        assert!(corpus.lint().is_empty());
        let inventory = corpus.inventory();
        assert_eq!(inventory.prompts, 1);
        assert_eq!(inventory.smoke_prompts, 1);
        assert_eq!(inventory.by_expected[&Disposition::Computed], 1);
        assert_eq!(inventory.by_expected[&Disposition::Missing], 0);
        assert!(inventory.complete);
    }

    #[test]
    fn duplicate_normalized_questions_and_ids_are_rejected() {
        let first = prompt("same");
        let mut second = prompt("same");
        second.question = "WHAT happens if salt meets water?!".to_string();
        let problems = corpus(vec![first, second]).lint().join("\n");
        assert!(problems.contains("duplicate id"));
        assert!(problems.contains("duplicates normalized question"));
    }

    #[test]
    fn boundaries_are_data_not_fake_scripts() {
        let mut boundary = prompt("sound-wave");
        boundary.expected = Disposition::Boundary;
        boundary.script.clear();
        boundary.boundary = Some("off-mission-acoustics".to_string());
        assert!(corpus(vec![boundary.clone()]).lint().is_empty());

        boundary.script.push("add v1 water 1mL".to_string());
        assert!(corpus(vec![boundary])
            .lint()
            .iter()
            .any(|problem| problem.contains("must not carry a script")));
    }

    #[test]
    fn shard_paths_cannot_escape_the_corpus() {
        assert!(safe_shard_path(Path::new("corpus"), "../secret.toml").is_err());
        assert!(safe_shard_path(Path::new("corpus"), "/tmp/secret.toml").is_err());
        assert!(safe_shard_path(Path::new("corpus"), "core.json").is_err());
        assert_eq!(
            safe_shard_path(Path::new("corpus"), "aqueous.toml").unwrap(),
            Path::new("corpus/aqueous.toml")
        );
    }
}
