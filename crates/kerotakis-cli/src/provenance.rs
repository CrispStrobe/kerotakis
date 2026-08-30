//! Distribution provenance for code, data, and generated artifacts.
//!
//! This is intentionally a release-policy manifest, not scientific
//! provenance attached to individual computed values. It answers a different
//! question: may this external material enter a Kerotakis artifact, and what
//! must travel with it when it does?

use std::collections::BTreeSet;
use std::path::{Component, Path};

use serde::Deserialize;
use sha2::{Digest, Sha256};

const SCHEMA_VERSION: u32 = 1;
const POLICY: &str = "store-permissive-v1";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Manifest {
    schema: u32,
    policy: String,
    scope: String,
    #[serde(default, rename = "source")]
    sources: Vec<Source>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    id: String,
    name: String,
    kind: Kind,
    lane: Lane,
    decision: Decision,
    licence: String,
    origin: String,
    terms: String,
    copyright: String,
    retrieved: String,
    #[serde(default, rename = "checksum")]
    checksums: Vec<FileChecksum>,
    #[serde(default)]
    revision: Option<String>,
    #[serde(default)]
    revision_path: Option<String>,
    attribution: String,
    paths: Vec<String>,
    #[serde(default)]
    upstream_inputs: Vec<String>,
    allowed_outputs: Vec<String>,
    #[serde(default)]
    targets: Vec<String>,
    reviewer: String,
    #[serde(default)]
    notes: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileChecksum {
    path: String,
    sha256: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum Kind {
    Code,
    Data,
    Media,
    Documentation,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum Lane {
    RuntimeCode,
    RuntimeData,
    OptionalPack,
    GeneratedShippingArtifact,
    TestFixture,
    OracleOnly,
    DevelopmentTool,
    Blocked,
    /// A pinned upstream snapshot held by the BRD-003 quarantine
    /// framework: committed for reproducibility, checksummed, and by
    /// construction outside every runtime path and release payload.
    /// Cleared for nothing — promotion into a shipping lane is its own
    /// reviewed record. This is the lane the importer adapters
    /// (BRD-010/011/013/060) put raw source bytes in.
    Quarantine,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum Decision {
    Approved,
    ReviewRequired,
    Blocked,
}

impl Manifest {
    fn parse(text: &str) -> Result<Self, String> {
        toml::from_str(text).map_err(|error| format!("could not parse manifest: {error}"))
    }

    fn problems(&self, root: &Path) -> Vec<String> {
        let mut problems = Vec::new();
        if self.schema != SCHEMA_VERSION {
            problems.push(format!(
                "schema: expected {SCHEMA_VERSION}, found {}",
                self.schema
            ));
        }
        if self.policy != POLICY {
            problems.push(format!(
                "policy: expected '{POLICY}', found '{}'",
                self.policy
            ));
        }
        required("manifest", "scope", &self.scope, &mut problems);
        if self.sources.is_empty() {
            problems.push("manifest: contains no source records".to_string());
        }

        let ids: BTreeSet<&str> = self
            .sources
            .iter()
            .map(|source| source.id.as_str())
            .collect();
        if ids.len() != self.sources.len() {
            let mut seen = BTreeSet::new();
            for source in &self.sources {
                if !seen.insert(source.id.as_str()) {
                    problems.push(format!("{}: duplicate source id", source.id));
                }
            }
        }

        for source in &self.sources {
            source.check(root, &ids, &mut problems);
        }
        problems
    }

    fn counts(&self) -> (usize, usize, usize) {
        let distributed = self
            .sources
            .iter()
            .filter(|source| source.lane.distributed())
            .count();
        let oracle = self
            .sources
            .iter()
            .filter(|source| source.lane == Lane::OracleOnly)
            .count();
        let blocked = self
            .sources
            .iter()
            .filter(|source| source.lane == Lane::Blocked)
            .count();
        (distributed, oracle, blocked)
    }
}

impl Source {
    fn check(&self, root: &Path, ids: &BTreeSet<&str>, problems: &mut Vec<String>) {
        let label = if self.id.trim().is_empty() {
            "<missing-id>"
        } else {
            self.id.as_str()
        };

        if !valid_id(&self.id) {
            problems.push(format!(
                "{label}: id must contain only lowercase ASCII letters, digits, and hyphens"
            ));
        }
        required(label, "name", &self.name, problems);
        required(label, "licence", &self.licence, problems);
        required(label, "origin", &self.origin, problems);
        required(label, "terms", &self.terms, problems);
        required(label, "copyright", &self.copyright, problems);
        required(label, "attribution", &self.attribution, problems);
        required(label, "reviewer", &self.reviewer, problems);
        let _ = &self.notes;
        if !looks_like_url(&self.origin) {
            problems.push(format!("{label}: origin must be an HTTP(S) URL"));
        }

        if !valid_date(&self.retrieved) {
            problems.push(format!(
                "{label}: retrieved must be a real-looking YYYY-MM-DD date"
            ));
        }
        if self.checksums.is_empty() && self.revision.is_none() {
            problems.push(format!(
                "{label}: record a sha256 checksum or immutable revision"
            ));
        }
        match (&self.revision, &self.revision_path) {
            (Some(revision), Some(path)) if valid_revision(revision) => {
                check_revision(label, revision, path, root, problems);
            }
            (Some(_), Some(_)) => problems.push(format!(
                "{label}: revision must be a 40- or 64-character hexadecimal object id"
            )),
            (Some(_), None) => problems.push(format!(
                "{label}: revision_path is required when revision is present"
            )),
            (None, Some(_)) => problems.push(format!(
                "{label}: revision is required when revision_path is present"
            )),
            (None, None) => {}
        }

        if self.paths.is_empty() {
            problems.push(format!("{label}: paths is empty"));
        }
        let mut seen_paths = BTreeSet::new();
        for path in &self.paths {
            if !seen_paths.insert(path) {
                problems.push(format!("{label}: duplicate path '{path}'"));
            }
            check_local_path(label, "path", path, root, problems);
        }
        let mut seen_checksums = BTreeSet::new();
        for checksum in &self.checksums {
            if !seen_checksums.insert(checksum.path.as_str()) {
                problems.push(format!(
                    "{label}: duplicate checksum path '{}'",
                    checksum.path
                ));
            }
            if !valid_sha256(&checksum.sha256) {
                problems.push(format!(
                    "{label}: sha256 for '{}' must contain exactly 64 hexadecimal characters",
                    checksum.path
                ));
            }
            let checksum_path = Path::new(&checksum.path);
            if !self
                .paths
                .iter()
                .any(|source_path| checksum_path.starts_with(Path::new(source_path)))
            {
                problems.push(format!(
                    "{label}: checksum path '{}' is outside the source paths",
                    checksum.path
                ));
            }
            check_checksum(label, checksum, root, problems);
        }
        if !looks_like_url(&self.terms) {
            check_local_path(label, "terms", &self.terms, root, problems);
        }

        if self.allowed_outputs.is_empty() {
            problems.push(format!("{label}: allowed_outputs is empty"));
        }
        if self.lane.distributed() && self.targets.is_empty() {
            problems.push(format!(
                "{label}: a distributed source needs at least one target"
            ));
        }
        if self.lane.distributed() && self.decision != Decision::Approved {
            problems.push(format!(
                "{label}: lane {:?} is distributed but decision is {:?}",
                self.lane, self.decision
            ));
        }
        if self.lane == Lane::Blocked && self.decision != Decision::Blocked {
            problems.push(format!(
                "{label}: the blocked lane requires decision = 'blocked'"
            ));
        }
        if self.lane != Lane::Blocked && self.decision == Decision::Blocked {
            problems.push(format!(
                "{label}: a blocked decision requires lane = 'blocked'"
            ));
        }
        if self.lane.distributed() && !direct_licence_allowed(self.kind, &self.licence) {
            problems.push(format!(
                "{label}: licence '{}' is not directly includable for {:?}",
                self.licence, self.kind
            ));
        }
        if self.licence == "NOASSERTION" || self.licence == "UNKNOWN" {
            problems.push(format!("{label}: ambiguous licence '{}'", self.licence));
        }

        for upstream in &self.upstream_inputs {
            if upstream == &self.id {
                problems.push(format!("{label}: source cannot be its own upstream input"));
            } else if !ids.contains(upstream.as_str()) {
                problems.push(format!("{label}: unknown upstream input '{upstream}'"));
            }
        }
    }
}

impl Lane {
    fn distributed(self) -> bool {
        matches!(
            self,
            Self::RuntimeCode
                | Self::RuntimeData
                | Self::OptionalPack
                | Self::GeneratedShippingArtifact
                | Self::TestFixture
        )
    }
}

fn direct_licence_allowed(kind: Kind, licence: &str) -> bool {
    const CODE: &[&str] = &[
        "MIT",
        "Apache-2.0",
        "BSD-2-Clause",
        "BSD-3-Clause",
        "ISC",
        "Zlib",
        "0BSD",
        "Unlicense",
        "CC0-1.0",
        "LicenseRef-USGS-User-Rights-Notice",
    ];
    const DATA: &[&str] = &[
        "CC0-1.0",
        "CC-BY-4.0",
        "CC-BY-3.0",
        "Apache-2.0",
        "LicenseRef-USGS-User-Rights-Notice",
        "LicenseRef-Public-Domain",
    ];
    match kind {
        Kind::Code => CODE.contains(&licence),
        Kind::Data | Kind::Media | Kind::Documentation => DATA.contains(&licence),
    }
}

fn required(label: &str, field: &str, value: &str, problems: &mut Vec<String>) {
    if value.trim().is_empty() {
        problems.push(format!("{label}: {field} is empty"));
    }
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && !id.starts_with('-')
        && !id.ends_with('-')
        && id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn valid_date(date: &str) -> bool {
    let parts: Vec<_> = date.split('-').collect();
    if parts.len() != 3 || parts[0].len() != 4 || parts[1].len() != 2 || parts[2].len() != 2 {
        return false;
    }
    let Ok(year) = parts[0].parse::<u16>() else {
        return false;
    };
    let Ok(month) = parts[1].parse::<u8>() else {
        return false;
    };
    let Ok(day) = parts[2].parse::<u8>() else {
        return false;
    };
    year >= 2000 && (1..=12).contains(&month) && (1..=31).contains(&day)
}

fn valid_sha256(checksum: &str) -> bool {
    checksum.len() == 64 && checksum.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_revision(revision: &str) -> bool {
    matches!(revision.len(), 40 | 64) && revision.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn looks_like_url(value: &str) -> bool {
    value.starts_with("https://") || value.starts_with("http://")
}

fn check_local_path(
    label: &str,
    field: &str,
    value: &str,
    root: &Path,
    problems: &mut Vec<String>,
) {
    let path = Path::new(value);
    let safe = !value.trim().is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)));
    if !safe {
        problems.push(format!(
            "{label}: {field} '{value}' must be a safe repository-relative path"
        ));
    } else if !root.join(path).exists() {
        problems.push(format!(
            "{label}: {field} '{value}' does not exist under root"
        ));
    }
}

fn check_checksum(label: &str, checksum: &FileChecksum, root: &Path, problems: &mut Vec<String>) {
    let path = Path::new(&checksum.path);
    let safe = !checksum.path.trim().is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)));
    if !safe {
        problems.push(format!(
            "{label}: checksum path '{}' must be a safe repository-relative path",
            checksum.path
        ));
        return;
    }
    let absolute = root.join(path);
    let bytes = match std::fs::read(&absolute) {
        Ok(bytes) => bytes,
        Err(error) => {
            problems.push(format!(
                "{label}: cannot read checksum path '{}': {error}",
                checksum.path
            ));
            return;
        }
    };
    let actual = format!("{:x}", Sha256::digest(bytes));
    if !actual.eq_ignore_ascii_case(&checksum.sha256) {
        problems.push(format!(
            "{label}: checksum drift for '{}': expected {}, found {actual}",
            checksum.path, checksum.sha256
        ));
    }
}

fn check_revision(
    label: &str,
    expected: &str,
    repository: &str,
    root: &Path,
    problems: &mut Vec<String>,
) {
    let path = Path::new(repository);
    let safe = !repository.trim().is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)));
    if !safe {
        problems.push(format!(
            "{label}: revision_path '{repository}' must be a safe repository-relative path"
        ));
        return;
    }
    let repository_path = root.join(path);
    let Some(repository_path) = repository_path.to_str() else {
        problems.push(format!(
            "{label}: revision_path '{repository}' is not valid UTF-8"
        ));
        return;
    };
    let output = match std::process::Command::new("git")
        .args(["-C", repository_path, "rev-parse", "HEAD"])
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            problems.push(format!(
                "{label}: cannot inspect revision at '{repository}': {error}"
            ));
            return;
        }
    };
    if !output.status.success() {
        problems.push(format!(
            "{label}: git could not inspect revision at '{repository}': {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
        return;
    }
    let actual = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !actual.eq_ignore_ascii_case(expected) {
        problems.push(format!(
            "{label}: revision drift at '{repository}': expected {expected}, found {actual}"
        ));
    }
}

pub(crate) fn lint_command(manifest_path: &str, root: &str) -> ! {
    let text = std::fs::read_to_string(manifest_path).unwrap_or_else(|error| {
        eprintln!("kero provenance: cannot read {manifest_path}: {error}");
        std::process::exit(1);
    });
    let manifest = Manifest::parse(&text).unwrap_or_else(|error| {
        eprintln!("kero provenance: {manifest_path}: {error}");
        std::process::exit(1);
    });
    let problems = manifest.problems(Path::new(root));
    if !problems.is_empty() {
        for problem in &problems {
            eprintln!("kero provenance: {problem}");
        }
        eprintln!(
            "kero provenance: {} problem{}",
            problems.len(),
            if problems.len() == 1 { "" } else { "s" }
        );
        std::process::exit(1);
    }
    let (distributed, oracle, blocked) = manifest.counts();
    println!(
        "provenance: {} sources valid ({distributed} distributed, {oracle} oracle-only, {blocked} blocked)",
        manifest.sources.len()
    );
    std::process::exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_manifest() -> Manifest {
        let mut manifest = Manifest::parse(
            r#"
schema = 1
policy = "store-permissive-v1"
scope = "unit test"

[[source]]
id = "example-data"
name = "Example data"
kind = "data"
lane = "runtime-data"
decision = "approved"
licence = "CC0-1.0"
origin = "https://example.invalid/data"
terms = "https://example.invalid/terms"
copyright = "Example"
retrieved = "2026-08-20"
attribution = "Example data"
paths = ["Cargo.toml"]
allowed_outputs = ["runtime-data"]
targets = ["native"]
reviewer = "test"

[[source.checksum]]
path = "Cargo.toml"
sha256 = "0000000000000000000000000000000000000000000000000000000000000000"
"#,
        )
        .unwrap();
        let bytes = std::fs::read(Path::new("../..").join("Cargo.toml")).unwrap();
        manifest.sources[0].checksums[0].sha256 = format!("{:x}", Sha256::digest(bytes));
        manifest
    }

    #[test]
    fn accepts_a_complete_allowlisted_record() {
        assert_eq!(
            valid_manifest().problems(Path::new("../..")),
            Vec::<String>::new()
        );
    }

    #[test]
    fn rejects_copyleft_in_a_distributed_lane() {
        let mut manifest = valid_manifest();
        manifest.sources[0].licence = "CC-BY-SA-4.0".to_string();
        let problems = manifest.problems(Path::new("../.."));
        assert!(
            problems
                .iter()
                .any(|problem| problem.contains("not directly includable")),
            "{problems:?}"
        );
    }

    #[test]
    fn permits_copyleft_only_outside_distribution() {
        let mut manifest = valid_manifest();
        manifest.sources[0].licence = "LGPL-3.0-only".to_string();
        manifest.sources[0].lane = Lane::OracleOnly;
        manifest.sources[0].targets.clear();
        assert_eq!(manifest.problems(Path::new("../..")), Vec::<String>::new());
    }

    #[test]
    fn rejects_unsafe_or_missing_paths() {
        let mut manifest = valid_manifest();
        manifest.sources[0].paths = vec!["../outside".to_string(), "missing.file".to_string()];
        let problems = manifest.problems(Path::new("../.."));
        assert!(
            problems
                .iter()
                .any(|problem| problem.contains("'../outside' must be a safe")),
            "{problems:?}"
        );
        assert!(
            problems
                .iter()
                .any(|problem| problem.contains("'missing.file' does not exist")),
            "{problems:?}"
        );
        assert!(
            problems
                .iter()
                .any(|problem| problem.contains("checksum path 'Cargo.toml' is outside")),
            "{problems:?}"
        );
    }

    #[test]
    fn rejects_unknown_upstream_inputs() {
        let mut manifest = valid_manifest();
        manifest.sources[0].upstream_inputs = vec!["missing-source".to_string()];
        let problems = manifest.problems(Path::new("../.."));
        assert!(
            problems
                .iter()
                .any(|problem| problem.contains("unknown upstream input")),
            "{problems:?}"
        );
    }

    #[test]
    fn rejects_checksum_drift() {
        let mut manifest = valid_manifest();
        manifest.sources[0].checksums[0].sha256 = "a".repeat(64);
        let problems = manifest.problems(Path::new("../.."));
        assert!(
            problems
                .iter()
                .any(|problem| problem.contains("checksum drift")),
            "{problems:?}"
        );
    }

    #[test]
    fn rejects_an_unpinned_revision() {
        let mut manifest = valid_manifest();
        manifest.sources[0].checksums.clear();
        manifest.sources[0].revision = Some("main".to_string());
        manifest.sources[0].revision_path = Some(".".to_string());
        let problems = manifest.problems(Path::new("../.."));
        assert!(
            problems
                .iter()
                .any(|problem| problem.contains("hexadecimal object id")),
            "{problems:?}"
        );
    }

    #[test]
    fn a_quarantine_snapshot_may_await_its_review() {
        // The importer adapters commit pinned upstream bytes into the
        // BRD-003 quarantine, cleared for nothing. That lane ships in no
        // payload, so the distributed-means-approved rule must not force
        // a verdict that has not been reached — review-required is the
        // truthful state while a licence question stands.
        let mut manifest = valid_manifest();
        let source = &mut manifest.sources[0];
        source.lane = Lane::Quarantine;
        source.decision = Decision::ReviewRequired;
        let problems = manifest.problems(Path::new("."));
        assert!(
            !problems
                .iter()
                .any(|p| p.contains("is distributed but decision")),
            "quarantine is not a distributed lane: {problems:?}"
        );
    }

    #[test]
    fn a_distributed_lane_still_demands_a_verdict() {
        // The teeth stay in: anything that actually ships remains
        // approved-or-refused, never pending.
        let mut manifest = valid_manifest();
        manifest.sources[0].decision = Decision::ReviewRequired;
        let problems = manifest.problems(Path::new("."));
        assert!(
            problems
                .iter()
                .any(|p| p.contains("is distributed but decision")),
            "runtime-data must refuse a pending review: {problems:?}"
        );
    }
}
