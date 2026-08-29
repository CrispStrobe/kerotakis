//! BRD-003: inspect pinned external-data snapshots without promoting them.

use std::fs;
use std::path::Path;

use kerotakis_data::{
    canonical_quarantine_bytes, default_runtime_data_licences, diff_quarantine, lint_promotion,
    review_candidates, EligibleFieldList, PromotionLintInput, PromotionPolicy,
    QuarantinedCandidate, SnapshotManifest,
};
use serde::de::DeserializeOwned;
use serde::Serialize;

fn main() {
    if let Err(error) = run(std::env::args().skip(1).collect()) {
        eprintln!("quarantine-review: {error}");
        std::process::exit(1);
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    match args.as_slice() {
        [command, manifest, raw] if command == "verify" => verify(manifest, raw),
        [command, candidates] if command == "canonicalize" => canonicalize(candidates),
        [command, candidates, policy] if command == "review" => review(candidates, policy),
        [command, old, new] if command == "diff" => diff(old, new),
        [command, manifest, raw, candidates, policy] if command == "lint" => {
            lint(manifest, raw, candidates, policy, None)
        }
        [command, manifest, raw, candidates, policy, eligible] if command == "lint" => {
            lint(manifest, raw, candidates, policy, Some(eligible))
        }
        _ => Err(usage().to_owned()),
    }
}

/// BRD-003's promotion gate: the same check importers call as a library
/// function, exiting non-zero when the flow must not proceed.
fn lint(
    manifest_path: &str,
    raw_path: &str,
    candidates_path: &str,
    policy_path: &str,
    eligible_path: Option<&str>,
) -> Result<(), String> {
    let manifest: SnapshotManifest = read_json(manifest_path)?;
    let raw = read(raw_path)?;
    let candidates: Vec<QuarantinedCandidate> = read_json(candidates_path)?;
    let policy: PromotionPolicy = read_json(policy_path)?;
    let eligible: Vec<EligibleFieldList> = match eligible_path {
        Some(path) => read_json(path)?,
        None => Vec::new(),
    };
    let allowed = default_runtime_data_licences();
    let report = lint_promotion(&PromotionLintInput {
        manifest: &manifest,
        raw_snapshot: &raw,
        candidates: &candidates,
        policy: &policy,
        allowed_runtime_licences: &allowed,
        eligible_fields: &eligible,
    });
    write_json(&report)?;
    if report.refuses() {
        return Err(format!(
            "promotion refused: {} violation(s)",
            report.violations.len()
        ));
    }
    Ok(())
}

fn verify(manifest_path: &str, raw_path: &str) -> Result<(), String> {
    let manifest: SnapshotManifest = read_json(manifest_path)?;
    let raw = read(raw_path)?;
    manifest.verify(&raw).map_err(|error| error.to_string())?;
    write_json(&serde_json::json!({
        "verified": true,
        "adapter_id": manifest.adapter_id,
        "source_id": manifest.source_id,
        "source_revision": manifest.source_revision,
        "record_count": manifest.record_count,
        "sha256": manifest.sha256,
    }))
}

fn canonicalize(path: &str) -> Result<(), String> {
    let candidates: Vec<QuarantinedCandidate> = read_json(path)?;
    let bytes = canonical_quarantine_bytes(candidates).map_err(|error| error.to_string())?;
    print!(
        "{}",
        String::from_utf8(bytes).expect("JSON serialization is UTF-8")
    );
    Ok(())
}

fn review(candidates_path: &str, policy_path: &str) -> Result<(), String> {
    let candidates: Vec<QuarantinedCandidate> = read_json(candidates_path)?;
    let policy: PromotionPolicy = read_json(policy_path)?;
    write_json(&review_candidates(candidates, &policy))
}

fn diff(old_path: &str, new_path: &str) -> Result<(), String> {
    let old: Vec<QuarantinedCandidate> = read_json(old_path)?;
    let new: Vec<QuarantinedCandidate> = read_json(new_path)?;
    let report = diff_quarantine(&old, &new).map_err(|error| error.to_string())?;
    write_json(&report)
}

fn read(path: &str) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("could not read {}: {error}", Path::new(path).display()))
}

fn read_json<T: DeserializeOwned>(path: &str) -> Result<T, String> {
    let bytes = read(path)?;
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("could not parse {}: {error}", Path::new(path).display()))
}

fn write_json(value: &impl Serialize) -> Result<(), String> {
    let output = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    println!("{output}");
    Ok(())
}

fn usage() -> &'static str {
    "usage:\n\
     \x20 quarantine-review verify <manifest.json> <raw-snapshot>\n\
     \x20 quarantine-review canonicalize <candidates.json>\n\
     \x20 quarantine-review review <candidates.json> <policy.json>\n\
     \x20 quarantine-review diff <old-candidates.json> <new-candidates.json>\n\
     \x20 quarantine-review lint <manifest.json> <raw-snapshot> <candidates.json> \
     <policy.json> [<eligible-fields.json>]"
}
