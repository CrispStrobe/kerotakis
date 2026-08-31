//! BRD-031: generate a deterministic quarantine artifact from local files.
//!
//! This native-only binary performs all filesystem access. The reusable
//! importer accepts byte slices and is safe to compile into browser targets.

use std::fs;
use std::path::Path;

use kerotakis_data::fluid_parameters::import_verified_snapshot;
use kerotakis_data::SnapshotManifest;

fn main() {
    if let Err(error) = run(std::env::args().skip(1).collect()) {
        eprintln!("fluid-parameter-import: {error}");
        std::process::exit(1);
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let [command, manifest_path, source_path] = args.as_slice() else {
        return Err(usage().to_owned());
    };
    if command != "generate" {
        return Err(usage().to_owned());
    }
    let manifest: SnapshotManifest = read_json(manifest_path)?;
    let raw = read(source_path)?;
    let import = import_verified_snapshot(&manifest, &raw).map_err(|error| error.to_string())?;
    let refuses = import.refuses();
    let output = serde_json::to_string_pretty(&import).map_err(|error| error.to_string())?;
    println!("{output}");
    if refuses {
        return Err("promotion refused by the deterministic review report".to_owned());
    }
    Ok(())
}

fn read(path: &str) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("could not read {}: {error}", Path::new(path).display()))
}

fn read_json<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, String> {
    let bytes = read(path)?;
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("could not parse {}: {error}", Path::new(path).display()))
}

fn usage() -> &'static str {
    "usage: fluid-parameter-import generate <manifest.json> <local-source.json>"
}
