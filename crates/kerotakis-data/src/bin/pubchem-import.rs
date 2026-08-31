//! BRD-010: run the PubChem adapter over a pinned snapshot.
//!
//! Build-time only, and read-only with respect to the runtime registry: every
//! subcommand writes to stdout and none of them touches
//! `data/registry/registry-source-v1.json`. Regenerating the checked-in
//! quarantine fixture is exactly:
//!
//! ```text
//! F=crates/kerotakis-data/tests/fixtures/quarantine/pubchem-v1
//! cargo run -q -p kerotakis-data --bin pubchem-import -- candidates $F/raw/snapshot.json > $F/candidates.json
//! cargo run -q -p kerotakis-data --bin pubchem-import -- policy                           > $F/policy.json
//! cargo run -q -p kerotakis-data --bin pubchem-import -- eligible  $F/raw/snapshot.json   > $F/eligible.json
//! cargo run -q -p kerotakis-data --bin pubchem-import -- report    $F/raw/snapshot.json   > $F/import-report.json
//! cargo run -q -p kerotakis-data --bin pubchem-import -- review    $F/raw/snapshot.json   > $F/review-report.json
//! cargo run -q -p kerotakis-data --bin pubchem-import -- lint $F/manifest.json $F/raw/snapshot.json
//! ```

use std::fs;
use std::path::Path;

use kerotakis_data::{
    canonical_quarantine_bytes, lint_promotion, parse_pubchem_snapshot, pubchem_candidate_licences,
    pubchem_eligible_fields, pubchem_import, pubchem_promotion_policy, review_candidates,
    PromotionLintInput, PubchemImport, PubchemSnapshot, SnapshotManifest,
};
use serde::Serialize;

fn main() {
    if let Err(error) = run(std::env::args().skip(1).collect()) {
        eprintln!("pubchem-import: {error}");
        std::process::exit(1);
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    match args.as_slice() {
        [command] if command == "policy" => write_json(&pubchem_promotion_policy()),
        [command, snapshot] if command == "candidates" => {
            let import = load(snapshot)?;
            let bytes =
                canonical_quarantine_bytes(import.candidates).map_err(|error| error.to_string())?;
            print!(
                "{}",
                String::from_utf8(bytes).expect("JSON serialization is UTF-8")
            );
            Ok(())
        }
        [command, snapshot] if command == "eligible" => {
            let import = load(snapshot)?;
            write_json(&pubchem_eligible_fields(
                &import.candidates,
                &pubchem_promotion_policy(),
            ))
        }
        [command, snapshot] if command == "report" => {
            let import = load(snapshot)?;
            // The candidates are their own artifact; the report is what a
            // reviewer reads, so it carries the observations instead.
            write_json(&serde_json::json!({
                "adapter_id": import.adapter_id,
                "source_revision": import.source_revision,
                "retrieved": import.retrieved,
                "record_count": import.records.len(),
                "finding_count": import.findings.len(),
                "synonym_conflict_count": import.synonym_conflicts.len(),
                "records": import.records,
                "findings": import.findings,
                "synonym_conflicts": import.synonym_conflicts,
            }))
        }
        [command, snapshot] if command == "review" => {
            let import = load(snapshot)?;
            write_json(&review_candidates(
                import.candidates,
                &pubchem_promotion_policy(),
            ))
        }
        [command, manifest, snapshot] if command == "lint" => lint(manifest, snapshot),
        _ => Err(usage().to_owned()),
    }
}

/// BRD-010's promotion dry run. The candidate lane is
/// [`pubchem_candidate_licences`], not the runtime lane: see its doc comment.
fn lint(manifest_path: &str, snapshot_path: &str) -> Result<(), String> {
    let manifest: SnapshotManifest = read_json(manifest_path)?;
    let raw = read(snapshot_path)?;
    let import = load(snapshot_path)?;
    let policy = pubchem_promotion_policy();
    let eligible = pubchem_eligible_fields(&import.candidates, &policy);
    let allowed = pubchem_candidate_licences();
    let report = lint_promotion(&PromotionLintInput {
        manifest: &manifest,
        raw_snapshot: &raw,
        candidates: &import.candidates,
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

fn load(path: &str) -> Result<PubchemImport, String> {
    let raw = read(path)?;
    let snapshot: PubchemSnapshot =
        parse_pubchem_snapshot(&raw).map_err(|error| error.to_string())?;
    Ok(pubchem_import(&snapshot))
}

fn read(path: &str) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("could not read {}: {error}", Path::new(path).display()))
}

fn read_json<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, String> {
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
     \x20 pubchem-import policy\n\
     \x20 pubchem-import candidates <raw-snapshot.json>\n\
     \x20 pubchem-import eligible   <raw-snapshot.json>\n\
     \x20 pubchem-import report     <raw-snapshot.json>\n\
     \x20 pubchem-import review     <raw-snapshot.json>\n\
     \x20 pubchem-import lint       <manifest.json> <raw-snapshot.json>"
}
