//! BRD-003: the promotion lint that stands between quarantine and a runtime
//! pack.
//!
//! `tools/provenance-lint.sh` audits the *repository*: vendored files, their
//! checksums, and the lanes in `provenance/sources.toml`. This lints the
//! *flow*: a pinned snapshot, the candidates an adapter derived from it, the
//! promotion policy, and the eligible-field list a reviewer signed off. The
//! two compose — neither repeats the other's checks.
//!
//! Every importer (BRD-010, BRD-011, BRD-013, BRD-060) calls
//! [`lint_promotion`] as a library function; `quarantine-review lint` is the
//! same check as a command, exiting non-zero when the report refuses.
//!
//! A refusal is never advisory. [`ProvenanceLintReport::refuses`] is the gate;
//! a promotion that proceeds past a refusal is a bug in the caller.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::adapter::{
    normalize_candidate_quantity, AdapterError, PromotionPolicy, QuarantinedCandidate,
    SnapshotManifest, ADAPTER_SCHEMA_VERSION,
};

/// The licences reviewed data may carry into a distributed runtime pack.
///
/// `CONTRIBUTING.md` and BREADTH's inherited rules are the authority: shipped
/// data is CC0 or CC BY 4.0, plus the USGS User Rights Notice already cleared
/// for runtime sources. Copyleft and ShareAlike data stays in an oracle lane.
/// Callers may pass a narrower set; a wider one is a licence decision, not a
/// code change.
pub fn default_runtime_data_licences() -> BTreeSet<String> {
    ["CC0-1.0", "CC-BY-4.0", "LicenseRef-USGS"]
        .into_iter()
        .map(str::to_owned)
        .collect()
}

/// The fields of one quarantined record a reviewer declared eligible for
/// promotion. Naming a field the record does not carry is a refusal, not a
/// no-op: it means the review was written against a different snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EligibleFieldList {
    pub adapter_id: String,
    pub external_record_id: String,
    pub fields: Vec<String>,
}

/// Everything the lint needs. Borrowed so an importer can hand over the
/// artifacts it already holds without cloning a snapshot.
#[derive(Debug, Clone, Copy)]
pub struct PromotionLintInput<'a> {
    pub manifest: &'a SnapshotManifest,
    pub raw_snapshot: &'a [u8],
    pub candidates: &'a [QuarantinedCandidate],
    pub policy: &'a PromotionPolicy,
    pub allowed_runtime_licences: &'a BTreeSet<String>,
    pub eligible_fields: &'a [EligibleFieldList],
}

/// One reason a quarantine→promotion flow must not proceed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "violation", rename_all = "snake_case")]
pub enum ProvenanceViolation {
    /// The raw snapshot bytes do not hash to what the manifest pins.
    SnapshotChecksumMismatch { expected: String, actual: String },
    /// The manifest itself is unusable — wrong schema, empty field, no records.
    SnapshotManifestRejected { detail: String },
    /// A candidate claims an adapter the pinned snapshot was not taken from.
    SnapshotAdapterMismatch {
        adapter_id: String,
        external_record_id: String,
        manifest_adapter_id: String,
    },
    /// More candidates than the snapshot claims records; the candidates cannot
    /// all have come from it.
    CandidatesExceedSnapshot {
        candidates: usize,
        record_count: u64,
    },
    /// Two candidates share one adapter/record key.
    DuplicateCandidate {
        adapter_id: String,
        external_record_id: String,
    },
    /// A field destined for runtime has no source or no licence.
    MissingFieldProvenance {
        adapter_id: String,
        external_record_id: String,
        field: String,
        /// Which part of the provenance is absent.
        missing: String,
    },
    /// A field destined for runtime carries a licence outside the runtime lane.
    LicenceNotAllowedForRuntime {
        adapter_id: String,
        external_record_id: String,
        field: String,
        licence: String,
    },
    /// The promotion policy itself would admit a non-runtime licence.
    PolicyAdmitsNonRuntimeLicence {
        source_field: String,
        licence: String,
    },
    /// A quantity field does not converge on the reviewed unit vocabulary.
    UnitNotNormalized {
        adapter_id: String,
        external_record_id: String,
        field: String,
        unit: String,
        detail: String,
    },
    /// An eligible-field list names a record that is not in the quarantine set.
    EligibleRecordNotInQuarantine {
        adapter_id: String,
        external_record_id: String,
    },
    /// An eligible-field list names a field the record does not carry.
    EligibleFieldNotOnRecord {
        adapter_id: String,
        external_record_id: String,
        field: String,
    },
    /// An eligible-field list names a field the promotion policy never
    /// allowlisted, so nothing could promote it anyway.
    EligibleFieldNotAllowlisted {
        adapter_id: String,
        external_record_id: String,
        field: String,
    },
    /// An eligible-field list repeats a field.
    EligibleFieldRepeated {
        adapter_id: String,
        external_record_id: String,
        field: String,
    },
}

/// The lint's verdict. Deterministic: the violation order depends only on the
/// inputs, so a report can be checked in and diffed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceLintReport {
    pub schema: u32,
    pub adapter_id: String,
    pub checked_records: usize,
    pub checked_fields: usize,
    pub violations: Vec<ProvenanceViolation>,
}

impl ProvenanceLintReport {
    /// Whether promotion must stop here.
    pub fn refuses(&self) -> bool {
        !self.violations.is_empty()
    }

    /// The report as a `Result`, for callers that would rather use `?`.
    pub fn into_result(self) -> Result<Self, Self> {
        if self.refuses() {
            Err(self)
        } else {
            Ok(self)
        }
    }
}

/// Refuse a quarantine→promotion flow that would carry unattributable,
/// wrongly licensed, unpinned, or misdescribed data into a runtime pack.
pub fn lint_promotion(input: &PromotionLintInput<'_>) -> ProvenanceLintReport {
    let mut violations = Vec::new();

    match input.manifest.verify(input.raw_snapshot) {
        Ok(()) => {}
        Err(AdapterError::ChecksumMismatch { expected, actual }) => {
            violations.push(ProvenanceViolation::SnapshotChecksumMismatch { expected, actual });
        }
        Err(error) => violations.push(ProvenanceViolation::SnapshotManifestRejected {
            detail: error.to_string(),
        }),
    }

    if input.candidates.len() as u64 > input.manifest.record_count {
        violations.push(ProvenanceViolation::CandidatesExceedSnapshot {
            candidates: input.candidates.len(),
            record_count: input.manifest.record_count,
        });
    }

    for (source_field, rule) in &input.policy.fields {
        for licence in &rule.allowed_licences {
            if !input.allowed_runtime_licences.contains(licence) {
                violations.push(ProvenanceViolation::PolicyAdmitsNonRuntimeLicence {
                    source_field: source_field.clone(),
                    licence: licence.clone(),
                });
            }
        }
    }

    let mut by_key: BTreeMap<(&str, &str), &QuarantinedCandidate> = BTreeMap::new();
    let mut checked_fields = 0;
    for candidate in input.candidates {
        let key = (
            candidate.adapter_id.as_str(),
            candidate.external_record_id.as_str(),
        );
        if by_key.insert(key, candidate).is_some() {
            violations.push(ProvenanceViolation::DuplicateCandidate {
                adapter_id: candidate.adapter_id.clone(),
                external_record_id: candidate.external_record_id.clone(),
            });
        }
        if candidate.adapter_id != input.manifest.adapter_id {
            violations.push(ProvenanceViolation::SnapshotAdapterMismatch {
                adapter_id: candidate.adapter_id.clone(),
                external_record_id: candidate.external_record_id.clone(),
                manifest_adapter_id: input.manifest.adapter_id.clone(),
            });
        }

        for (field_name, field) in &candidate.fields {
            // Only fields with a runtime target are destined for a pack; the
            // rest are reported by review as unallowlisted and never promoted.
            let Some(rule) = input.policy.fields.get(field_name) else {
                continue;
            };
            checked_fields += 1;

            for (missing, present) in [
                (
                    "source_record_id",
                    !candidate.source_record_id.trim().is_empty(),
                ),
                ("source_field", !field.source_field.trim().is_empty()),
                ("licence", !field.licence.trim().is_empty()),
            ] {
                if !present {
                    violations.push(ProvenanceViolation::MissingFieldProvenance {
                        adapter_id: candidate.adapter_id.clone(),
                        external_record_id: candidate.external_record_id.clone(),
                        field: field_name.clone(),
                        missing: missing.to_owned(),
                    });
                }
            }

            if !field.licence.trim().is_empty()
                && !input.allowed_runtime_licences.contains(&field.licence)
            {
                violations.push(ProvenanceViolation::LicenceNotAllowedForRuntime {
                    adapter_id: candidate.adapter_id.clone(),
                    external_record_id: candidate.external_record_id.clone(),
                    field: field_name.clone(),
                    licence: field.licence.clone(),
                });
            }

            if let Err(reason) = normalize_candidate_quantity(field, rule) {
                violations.push(ProvenanceViolation::UnitNotNormalized {
                    adapter_id: candidate.adapter_id.clone(),
                    external_record_id: candidate.external_record_id.clone(),
                    field: field_name.clone(),
                    unit: field.unit.clone().unwrap_or_default(),
                    detail: format!("{reason:?}"),
                });
            }
        }
    }

    for list in input.eligible_fields {
        let key = (list.adapter_id.as_str(), list.external_record_id.as_str());
        let Some(candidate) = by_key.get(&key) else {
            violations.push(ProvenanceViolation::EligibleRecordNotInQuarantine {
                adapter_id: list.adapter_id.clone(),
                external_record_id: list.external_record_id.clone(),
            });
            continue;
        };
        let mut seen = BTreeSet::new();
        for field in &list.fields {
            if !seen.insert(field.as_str()) {
                violations.push(ProvenanceViolation::EligibleFieldRepeated {
                    adapter_id: list.adapter_id.clone(),
                    external_record_id: list.external_record_id.clone(),
                    field: field.clone(),
                });
                continue;
            }
            if !candidate.fields.contains_key(field) {
                violations.push(ProvenanceViolation::EligibleFieldNotOnRecord {
                    adapter_id: list.adapter_id.clone(),
                    external_record_id: list.external_record_id.clone(),
                    field: field.clone(),
                });
                continue;
            }
            if !input.policy.fields.contains_key(field) {
                violations.push(ProvenanceViolation::EligibleFieldNotAllowlisted {
                    adapter_id: list.adapter_id.clone(),
                    external_record_id: list.external_record_id.clone(),
                    field: field.clone(),
                });
            }
        }
    }

    ProvenanceLintReport {
        schema: ADAPTER_SCHEMA_VERSION,
        adapter_id: input.manifest.adapter_id.clone(),
        checked_records: input.candidates.len(),
        checked_fields,
        violations,
    }
}
