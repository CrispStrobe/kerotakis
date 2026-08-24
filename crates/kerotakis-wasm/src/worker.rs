//! WEB-002/003/004: Module Worker protocol, model-pack splitting, and offline install.
//!
//! Defines the message protocol between the main thread and the
//! chemistry worker, model-pack manifest format, and cache verification.

use serde::{Deserialize, Serialize};

// ── WEB-002: Worker message protocol ──────────────────────────────

/// A command from the main thread to the chemistry worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum WorkerCommand {
    /// Run a single operator step.
    Step { operator_json: String },
    /// Run a full .lab script.
    RunScript { script: String },
    /// Load pre-warmed cache data.
    LoadCache { url: String },
    /// Validate a single line without executing it.
    Parse { line: String },
    /// Cancel the current operation.
    Cancel,
    /// Reset the bench.
    Reset,
    /// Change the register level.
    SetRegister { level: String },
    /// Load a model pack.
    LoadPack { manifest_url: String },
}

/// A response from the chemistry worker to the main thread.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkerResponse {
    /// The operation completed.
    Done { result_json: String },
    /// Progress update during a long operation.
    Progress { fraction: f64, message: String },
    /// The operation failed.
    Error { message: String },
    /// The operation was cancelled.
    Cancelled,
}

// ── WEB-003: Model-pack manifest ──────────────────────────────────

/// A signed model-pack manifest for independent delivery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPackManifest {
    /// Pack identity: "core-aqueous", "phase", "combustion", etc.
    pub pack_id: String,
    /// Semantic version of the pack contents.
    pub version: String,
    /// SHA-256 of the packed payload.
    pub content_hash: String,
    /// Minimum app version that can load this pack.
    pub min_app_version: String,
    /// Data licence summary.
    pub licence: String,
    /// Compressed payload size in bytes.
    pub compressed_size: usize,
    /// Whether this pack is required for basic functionality.
    pub required: bool,
}

/// Split the full model set into independent packs.
pub fn core_packs() -> Vec<ModelPackManifest> {
    vec![
        ModelPackManifest {
            pack_id: "core-aqueous".into(),
            version: "1.0.0".into(),
            content_hash: String::new(), // computed at build time
            min_app_version: "0.0.1".into(),
            licence: "AGPL-3.0 + USGS public domain data".into(),
            compressed_size: 0,
            required: true,
        },
        ModelPackManifest {
            pack_id: "phase".into(),
            version: "1.0.0".into(),
            content_hash: String::new(),
            min_app_version: "0.0.1".into(),
            licence: "AGPL-3.0".into(),
            compressed_size: 0,
            required: false,
        },
        ModelPackManifest {
            pack_id: "combustion".into(),
            version: "1.0.0".into(),
            content_hash: String::new(),
            min_app_version: "0.0.1".into(),
            licence: "AGPL-3.0 + NASA public domain data".into(),
            compressed_size: 0,
            required: false,
        },
        ModelPackManifest {
            pack_id: "structures".into(),
            version: "1.0.0".into(),
            content_hash: String::new(),
            min_app_version: "0.0.1".into(),
            licence: "AGPL-3.0 + MIT (chematic)".into(),
            compressed_size: 0,
            required: false,
        },
        ModelPackManifest {
            pack_id: "spectra".into(),
            version: "1.0.0".into(),
            content_hash: String::new(),
            min_app_version: "0.0.1".into(),
            licence: "AGPL-3.0".into(),
            compressed_size: 0,
            required: false,
        },
    ]
}

// ── WEB-004: Offline install verification ─────────────────────────

/// Verify that all required packs are cached.
pub fn verify_offline_install(
    manifests: &[ModelPackManifest],
    cached_hashes: &[&str],
) -> Result<(), Vec<String>> {
    let mut missing = Vec::new();
    for pack in manifests {
        if pack.required && !cached_hashes.contains(&pack.content_hash.as_str()) {
            missing.push(format!("required pack '{}' not cached", pack.pack_id));
        }
    }
    if missing.is_empty() {
        Ok(())
    } else {
        Err(missing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_command_serializes() {
        let cmd = WorkerCommand::Step {
            operator_json: r#"{"op":"new_vessel"}"#.into(),
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("step"));
        let loaded: WorkerCommand = serde_json::from_str(&json).unwrap();
        assert!(matches!(loaded, WorkerCommand::Step { .. }));
    }

    #[test]
    fn worker_response_serializes() {
        let resp = WorkerResponse::Progress {
            fraction: 0.5,
            message: "halfway".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let loaded: WorkerResponse = serde_json::from_str(&json).unwrap();
        assert!(matches!(loaded, WorkerResponse::Progress { .. }));
    }

    #[test]
    fn core_packs_has_required() {
        let packs = core_packs();
        assert!(packs.iter().any(|p| p.required));
    }

    #[test]
    fn offline_verify_rejects_missing_required() {
        let packs = core_packs();
        let result = verify_offline_install(&packs, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn cancel_command_serializes() {
        let cmd = WorkerCommand::Cancel;
        let json = serde_json::to_string(&cmd).unwrap();
        let loaded: WorkerCommand = serde_json::from_str(&json).unwrap();
        assert!(matches!(loaded, WorkerCommand::Cancel));
    }

    #[test]
    fn parse_command_serializes() {
        let cmd = WorkerCommand::Parse {
            line: "add v1 water 100mL".into(),
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("parse"));
        let loaded: WorkerCommand = serde_json::from_str(&json).unwrap();
        assert!(matches!(loaded, WorkerCommand::Parse { .. }));
    }
}
