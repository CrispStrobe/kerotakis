//! LIC-009: Model-pack manifest schema.
//!
//! A model pack is a distributable bundle of compiled registry data,
//! mechanisms, parameters, and their notices. The manifest identifies
//! what's inside, how to verify it, and what app version can load it.

use serde::{Deserialize, Serialize};

/// Manifest for a distributable model pack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPackManifest {
    /// Unique identifier for this pack (e.g. "core-aqueous-v1").
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Semantic version of the pack content.
    pub version: String,
    /// SHA-256 content hash of the pack payload.
    pub content_hash: String,
    /// Engine ABI version required to load this pack.
    pub engine_abi: String,
    /// Data schema version (matches RegistryDocument.schema).
    pub data_schema: u32,
    /// SPDX licence expression for the pack's aggregate content.
    pub licence: String,
    /// Attribution text (notices from all included sources).
    pub attribution: String,
    /// URL where the pack can be fetched (if remote).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    /// Cryptographic signature (base64-encoded, if signed).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    /// Minimum app version required to use this pack.
    pub min_app_version: String,
    /// Distribution lane: which targets may include this pack.
    pub lane: PackLane,
    /// What the pack contains.
    pub contents: PackContents,
}

/// Distribution lane for model packs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackLane {
    /// Shipped with every official binary.
    Core,
    /// Downloaded on demand by the app.
    Optional,
    /// Development/testing only — never in a release.
    Development,
}

/// Summary of what a pack contains.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackContents {
    pub species_count: usize,
    pub phase_thermo_records: usize,
    pub mechanisms: usize,
    pub model_parameters: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub included_databases: Vec<String>,
}

/// Reject reasons for packs that fail validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackRejectReason {
    UnsignedPack,
    UnapprovedLicence(String),
    SchemaMismatch { expected: u32, got: u32 },
    HashMismatch,
    AppTooOld { required: String, current: String },
    DevelopmentOnlyInRelease,
}

impl std::fmt::Display for PackRejectReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsignedPack => write!(f, "pack is not signed"),
            Self::UnapprovedLicence(l) => write!(f, "unapproved licence: {l}"),
            Self::SchemaMismatch { expected, got } => {
                write!(f, "schema version mismatch: expected {expected}, got {got}")
            }
            Self::HashMismatch => write!(f, "content hash mismatch"),
            Self::AppTooOld { required, current } => {
                write!(f, "app version {current} too old; pack requires {required}")
            }
            Self::DevelopmentOnlyInRelease => {
                write!(f, "development pack cannot be included in a release build")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_round_trips() {
        let manifest = ModelPackManifest {
            id: "core-aqueous-v1".into(),
            name: "Core Aqueous Pack".into(),
            version: "1.0.0".into(),
            content_hash: "abc123".into(),
            engine_abi: "0.1".into(),
            data_schema: 1,
            licence: "AGPL-3.0-or-later AND (MIT OR Apache-2.0)".into(),
            attribution: "PHREEQC (USGS), MY-BASIC (MIT)".into(),
            source_url: None,
            signature: None,
            min_app_version: "0.1.0".into(),
            lane: PackLane::Core,
            contents: PackContents {
                species_count: 75,
                phase_thermo_records: 238,
                mechanisms: 0,
                model_parameters: 103,
                included_databases: vec!["phreeqc.dat".into()],
            },
        };
        let json = serde_json::to_string_pretty(&manifest).unwrap();
        let loaded: ModelPackManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.id, "core-aqueous-v1");
        assert_eq!(loaded.lane, PackLane::Core);
        assert_eq!(loaded.contents.species_count, 75);
    }

    #[test]
    fn reject_reasons_display() {
        assert!(PackRejectReason::UnsignedPack.to_string().contains("not signed"));
        assert!(PackRejectReason::DevelopmentOnlyInRelease
            .to_string()
            .contains("development"));
    }
}
