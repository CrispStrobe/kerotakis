//! Shared quarantine contract for external breadth-data adapters.
//!
//! Adapters stop here. A successful review makes fields eligible for a later,
//! human-authored registry change; it never writes runtime records.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

pub const ADAPTER_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotManifest {
    pub schema: u32,
    pub adapter_id: String,
    pub source_id: String,
    pub source_revision: String,
    pub retrieved: String,
    pub raw_artifact: String,
    pub record_count: u64,
    /// Lowercase hexadecimal SHA-256 of the raw snapshot bytes.
    pub sha256: String,
}

impl SnapshotManifest {
    pub fn verify(&self, raw: &[u8]) -> Result<(), AdapterError> {
        if self.schema != ADAPTER_SCHEMA_VERSION {
            return Err(AdapterError::UnsupportedSchema {
                found: self.schema,
                expected: ADAPTER_SCHEMA_VERSION,
            });
        }
        for (name, value) in [
            ("adapter_id", &self.adapter_id),
            ("source_id", &self.source_id),
            ("source_revision", &self.source_revision),
            ("retrieved", &self.retrieved),
            ("raw_artifact", &self.raw_artifact),
        ] {
            if value.trim().is_empty() {
                return Err(AdapterError::MissingManifestField(name));
            }
        }
        if self.record_count == 0 {
            return Err(AdapterError::EmptySnapshot);
        }
        let actual = snapshot_sha256(raw);
        if self.sha256 != actual {
            return Err(AdapterError::ChecksumMismatch {
                expected: self.sha256.clone(),
                actual,
            });
        }
        Ok(())
    }
}

pub fn snapshot_sha256(raw: &[u8]) -> String {
    let digest = Sha256::digest(raw);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CandidateField {
    pub value: Value,
    /// Exact field/path in the pinned source record.
    pub source_field: String,
    /// SPDX expression or reviewed project-local `LicenseRef-*`.
    pub licence: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuarantinedCandidate {
    pub adapter_id: String,
    pub source_record_id: String,
    pub external_record_id: String,
    /// Standard InChIKey or another adapter-declared stable review key.
    #[serde(default)]
    pub identity_key: Option<String>,
    pub fields: BTreeMap<String, CandidateField>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeFieldPolicy {
    pub target_field: String,
    pub allowed_licences: BTreeSet<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotionPolicy {
    /// Source-field name to its explicitly reviewed runtime policy.
    pub fields: BTreeMap<String, RuntimeFieldPolicy>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReviewedField {
    pub value: Value,
    pub source_record_id: String,
    pub source_field: String,
    pub licence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum FieldRejectionReason {
    UnallowlistedField,
    LicenceNotAllowed { licence: String },
    MissingProvenance,
    TargetCollision { target_field: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldRejection {
    pub field: String,
    pub reason: FieldRejectionReason,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromotionReview {
    pub adapter_id: String,
    pub external_record_id: String,
    pub accepted: BTreeMap<String, ReviewedField>,
    pub rejected: Vec<FieldRejection>,
}

/// Apply an explicit field-and-licence allowlist without mutating a registry.
pub fn review_candidate(
    candidate: &QuarantinedCandidate,
    policy: &PromotionPolicy,
) -> PromotionReview {
    let mut accepted = BTreeMap::new();
    let mut rejected = Vec::new();
    let mut target_counts: BTreeMap<&str, usize> = BTreeMap::new();
    for field_name in candidate.fields.keys() {
        if let Some(rule) = policy.fields.get(field_name) {
            *target_counts.entry(rule.target_field.as_str()).or_default() += 1;
        }
    }

    for (field_name, field) in &candidate.fields {
        let Some(rule) = policy.fields.get(field_name) else {
            rejected.push(FieldRejection {
                field: field_name.clone(),
                reason: FieldRejectionReason::UnallowlistedField,
            });
            continue;
        };
        if candidate.source_record_id.trim().is_empty()
            || field.source_field.trim().is_empty()
            || field.licence.trim().is_empty()
        {
            rejected.push(FieldRejection {
                field: field_name.clone(),
                reason: FieldRejectionReason::MissingProvenance,
            });
            continue;
        }
        if target_counts.get(rule.target_field.as_str()) != Some(&1) {
            rejected.push(FieldRejection {
                field: field_name.clone(),
                reason: FieldRejectionReason::TargetCollision {
                    target_field: rule.target_field.clone(),
                },
            });
            continue;
        }
        if !rule.allowed_licences.contains(&field.licence) {
            rejected.push(FieldRejection {
                field: field_name.clone(),
                reason: FieldRejectionReason::LicenceNotAllowed {
                    licence: field.licence.clone(),
                },
            });
            continue;
        }
        accepted.insert(
            rule.target_field.clone(),
            ReviewedField {
                value: field.value.clone(),
                source_record_id: candidate.source_record_id.clone(),
                source_field: field.source_field.clone(),
                licence: field.licence.clone(),
            },
        );
    }

    PromotionReview {
        adapter_id: candidate.adapter_id.clone(),
        external_record_id: candidate.external_record_id.clone(),
        accepted,
        rejected,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdentityConflict {
    pub identity_key: String,
    pub records: Vec<String>,
    /// Fields whose candidate values do not all agree.
    pub differing_fields: Vec<String>,
}

/// Group competing source records instead of silently picking one.
pub fn identity_conflicts(candidates: &[QuarantinedCandidate]) -> Vec<IdentityConflict> {
    let mut groups: BTreeMap<&str, Vec<&QuarantinedCandidate>> = BTreeMap::new();
    for candidate in candidates {
        if let Some(key) = candidate
            .identity_key
            .as_deref()
            .filter(|key| !key.is_empty())
        {
            groups.entry(key).or_default().push(candidate);
        }
    }

    groups
        .into_iter()
        .filter_map(|(identity_key, mut group)| {
            if group.len() < 2 {
                return None;
            }
            group.sort_by(|a, b| a.external_record_id.cmp(&b.external_record_id));
            let field_names: BTreeSet<&str> = group
                .iter()
                .flat_map(|candidate| candidate.fields.keys().map(String::as_str))
                .collect();
            let differing_fields = field_names
                .into_iter()
                .filter(|field_name| {
                    let values: BTreeSet<String> = group
                        .iter()
                        .map(|candidate| {
                            candidate
                                .fields
                                .get(*field_name)
                                .map(|field| field.value.to_string())
                                .unwrap_or_else(|| "<missing>".into())
                        })
                        .collect();
                    values.len() > 1
                })
                .map(str::to_owned)
                .collect();
            Some(IdentityConflict {
                identity_key: identity_key.to_owned(),
                records: group
                    .iter()
                    .map(|candidate| candidate.external_record_id.clone())
                    .collect(),
                differing_fields,
            })
        })
        .collect()
}

/// Stable bytes for a checked-in quarantine fixture or refresh diff.
pub fn canonical_quarantine_bytes(
    mut candidates: Vec<QuarantinedCandidate>,
) -> Result<Vec<u8>, serde_json::Error> {
    candidates.sort_by(|a, b| {
        (&a.adapter_id, &a.external_record_id).cmp(&(&b.adapter_id, &b.external_record_id))
    });
    let mut bytes = serde_json::to_vec_pretty(&candidates)?;
    bytes.push(b'\n');
    Ok(bytes)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterError {
    UnsupportedSchema { found: u32, expected: u32 },
    MissingManifestField(&'static str),
    EmptySnapshot,
    ChecksumMismatch { expected: String, actual: String },
}

impl std::fmt::Display for AdapterError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedSchema { found, expected } => {
                write!(
                    formatter,
                    "unsupported adapter schema {found}; expected {expected}"
                )
            }
            Self::MissingManifestField(field) => {
                write!(formatter, "snapshot manifest field {field} is empty")
            }
            Self::EmptySnapshot => write!(formatter, "snapshot manifest has no records"),
            Self::ChecksumMismatch { expected, actual } => {
                write!(
                    formatter,
                    "snapshot checksum mismatch: expected {expected}, got {actual}"
                )
            }
        }
    }
}

impl std::error::Error for AdapterError {}
