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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuarantineReviewReport {
    pub schema: u32,
    pub reviews: Vec<PromotionReview>,
    pub identity_conflicts: Vec<IdentityConflict>,
}

/// Review a complete fixture in stable record order.
pub fn review_candidates(
    mut candidates: Vec<QuarantinedCandidate>,
    policy: &PromotionPolicy,
) -> QuarantineReviewReport {
    candidates.sort_by(|a, b| {
        (&a.adapter_id, &a.external_record_id).cmp(&(&b.adapter_id, &b.external_record_id))
    });
    let identity_conflicts = identity_conflicts(&candidates);
    let reviews = candidates
        .iter()
        .map(|candidate| review_candidate(candidate, policy))
        .collect();
    QuarantineReviewReport {
        schema: ADAPTER_SCHEMA_VERSION,
        reviews,
        identity_conflicts,
    }
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuarantineRefreshDiff {
    pub schema: u32,
    pub old_record_count: usize,
    pub new_record_count: usize,
    pub added_records: Vec<QuarantineRecordKey>,
    pub removed_records: Vec<QuarantineRecordKey>,
    pub changed_records: Vec<QuarantineRecordChange>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct QuarantineRecordKey {
    pub adapter_id: String,
    pub external_record_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuarantineRecordChange {
    pub adapter_id: String,
    pub external_record_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity_key: Option<ValueChange<Option<String>>>,
    pub fields: Vec<QuarantineFieldChange>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValueChange<T> {
    pub old: T,
    pub new: T,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "change", rename_all = "snake_case")]
pub enum QuarantineFieldChange {
    Added {
        field: String,
        new: CandidateField,
    },
    Removed {
        field: String,
        old: CandidateField,
    },
    Changed {
        field: String,
        old: CandidateField,
        new: CandidateField,
    },
}

/// Compare two pinned candidate fixtures without promoting either one.
pub fn diff_quarantine(
    old: &[QuarantinedCandidate],
    new: &[QuarantinedCandidate],
) -> Result<QuarantineRefreshDiff, AdapterError> {
    let old_by_key = candidates_by_key(old)?;
    let new_by_key = candidates_by_key(new)?;
    let old_keys: BTreeSet<_> = old_by_key.keys().cloned().collect();
    let new_keys: BTreeSet<_> = new_by_key.keys().cloned().collect();

    let added_records = new_keys.difference(&old_keys).cloned().collect();
    let removed_records = old_keys.difference(&new_keys).cloned().collect();
    let mut changed_records = Vec::new();

    for key in old_keys.intersection(&new_keys) {
        let old_candidate = old_by_key[key];
        let new_candidate = new_by_key[key];
        let identity_key =
            (old_candidate.identity_key != new_candidate.identity_key).then(|| ValueChange {
                old: old_candidate.identity_key.clone(),
                new: new_candidate.identity_key.clone(),
            });
        let old_fields: BTreeSet<_> = old_candidate.fields.keys().cloned().collect();
        let new_fields: BTreeSet<_> = new_candidate.fields.keys().cloned().collect();
        let mut fields = Vec::new();
        for field in old_fields.union(&new_fields) {
            match (
                old_candidate.fields.get(field),
                new_candidate.fields.get(field),
            ) {
                (None, Some(new)) => fields.push(QuarantineFieldChange::Added {
                    field: field.clone(),
                    new: new.clone(),
                }),
                (Some(old), None) => fields.push(QuarantineFieldChange::Removed {
                    field: field.clone(),
                    old: old.clone(),
                }),
                (Some(old), Some(new)) if old != new => {
                    fields.push(QuarantineFieldChange::Changed {
                        field: field.clone(),
                        old: old.clone(),
                        new: new.clone(),
                    });
                }
                _ => {}
            }
        }
        if identity_key.is_some() || !fields.is_empty() {
            changed_records.push(QuarantineRecordChange {
                adapter_id: key.adapter_id.clone(),
                external_record_id: key.external_record_id.clone(),
                identity_key,
                fields,
            });
        }
    }

    Ok(QuarantineRefreshDiff {
        schema: ADAPTER_SCHEMA_VERSION,
        old_record_count: old.len(),
        new_record_count: new.len(),
        added_records,
        removed_records,
        changed_records,
    })
}

fn candidates_by_key(
    candidates: &[QuarantinedCandidate],
) -> Result<BTreeMap<QuarantineRecordKey, &QuarantinedCandidate>, AdapterError> {
    let mut by_key = BTreeMap::new();
    for candidate in candidates {
        let key = QuarantineRecordKey {
            adapter_id: candidate.adapter_id.clone(),
            external_record_id: candidate.external_record_id.clone(),
        };
        if by_key.insert(key.clone(), candidate).is_some() {
            return Err(AdapterError::DuplicateRecord {
                adapter_id: key.adapter_id,
                external_record_id: key.external_record_id,
            });
        }
    }
    Ok(by_key)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterError {
    UnsupportedSchema {
        found: u32,
        expected: u32,
    },
    MissingManifestField(&'static str),
    EmptySnapshot,
    ChecksumMismatch {
        expected: String,
        actual: String,
    },
    DuplicateRecord {
        adapter_id: String,
        external_record_id: String,
    },
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
            Self::DuplicateRecord {
                adapter_id,
                external_record_id,
            } => write!(
                formatter,
                "duplicate quarantine record {adapter_id}/{external_record_id}"
            ),
        }
    }
}

impl std::error::Error for AdapterError {}
