//! WEB-003 model-pack manifests + WEB-004 offline verification —
//! shared by every host (moved from kerotakis-wasm::worker so the
//! native shell can answer hello.packs without a wasm dependency).

use serde::{Deserialize, Serialize};

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
