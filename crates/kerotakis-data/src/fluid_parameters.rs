//! BRD-031: quarantined fluid-model parameter import.
//!
//! This adapter deliberately stops at the BRD-003 quarantine boundary. It
//! parses a small, source-neutral PC-SAFT-shaped document, checks the pilot
//! identities and numeric shape, and emits provenance-complete candidates.
//! It does not contain, fetch, or promote third-party parameter values.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};

use crate::{
    default_runtime_data_licences, review_candidates, AdapterError, CandidateField, Dimension,
    PromotionPolicy, QuarantineReviewReport, QuarantinedCandidate, RuntimeFieldPolicy,
    SnapshotManifest,
};

pub const ADAPTER_ID: &str = "brd031-fluid-parameters-v1";

/// The deliberately small identity pilot. Values are identity metadata, not
/// thermodynamic parameters.
pub const PILOT_IDENTITIES: [(&str, &str); 6] = [
    ("water", "XLYOFNOQVPJJNP-UHFFFAOYSA-N"),
    ("CO2", "CURLTUGMZLYLDI-UHFFFAOYSA-N"),
    ("N2", "IJGRMHOSHXDMSA-UHFFFAOYSA-N"),
    ("O2", "MYMOFIZGZYHOMD-UHFFFAOYSA-N"),
    ("NH3", "QGZKDVFQNNGYKY-UHFFFAOYSA-N"),
    ("ethanol", "LFQSCWFLJHTTHZ-UHFFFAOYSA-N"),
];

const PILOT_CANONICAL_NAMES: [(&str, &str); 6] = [
    ("water", "water"),
    ("CO2", "carbon dioxide"),
    ("N2", "nitrogen"),
    ("O2", "oxygen"),
    ("NH3", "ammonia"),
    ("ethanol", "ethanol"),
];

#[derive(Debug, Deserialize)]
struct SourceDocument {
    source_id: String,
    source_revision: String,
    data_licence: String,
    records: Vec<SourceRecord>,
}

#[derive(Debug, Deserialize)]
struct SourceRecord {
    id: String,
    canonical_name: String,
    inchikey: String,
    pc_saft: PcSaftRecord,
}

#[derive(Debug, Deserialize)]
struct PcSaftRecord {
    molar_mass: Quantity,
    segment_count: Quantity,
    segment_diameter: Quantity,
    dispersion_energy_k: Quantity,
    #[serde(default)]
    association: Option<AssociationRecord>,
}

#[derive(Debug, Deserialize)]
struct AssociationRecord {
    kappa_ab: Quantity,
    epsilon_k_ab: Quantity,
    na: u32,
    nb: u32,
    nc: u32,
}

#[derive(Debug, Deserialize)]
struct Quantity {
    value: f64,
    unit: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum FluidParameterImportError {
    Snapshot {
        detail: String,
    },
    ManifestMismatch {
        field: String,
        expected: String,
        found: String,
    },
    MalformedDocument {
        detail: String,
    },
    MissingDocumentMetadata {
        field: String,
    },
    DuplicateRecord {
        id: String,
    },
    MissingCanonicalIdentity {
        id: String,
    },
    UnknownCanonicalIdentity {
        id: String,
    },
    IdentityMismatch {
        id: String,
        expected_inchikey: String,
        found_inchikey: String,
    },
    CanonicalNameMismatch {
        id: String,
        expected_name: String,
        found_name: String,
    },
    InvalidQuantity {
        id: String,
        field: String,
    },
    MissingAssociation {
        id: String,
    },
}

impl fmt::Display for FluidParameterImportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Snapshot { detail } => {
                write!(formatter, "snapshot verification failed: {detail}")
            }
            Self::ManifestMismatch {
                field,
                expected,
                found,
            } => write!(
                formatter,
                "snapshot manifest mismatch for {field}: expected {expected}, found {found}"
            ),
            Self::MalformedDocument { detail } => write!(formatter, "malformed document: {detail}"),
            Self::MissingDocumentMetadata { field } => {
                write!(formatter, "missing document metadata: {field}")
            }
            Self::DuplicateRecord { id } => write!(formatter, "duplicate fluid record: {id}"),
            Self::MissingCanonicalIdentity { id } => {
                write!(formatter, "missing canonical pilot identity: {id}")
            }
            Self::UnknownCanonicalIdentity { id } => {
                write!(formatter, "identity is outside the BRD-031 pilot: {id}")
            }
            Self::IdentityMismatch {
                id,
                expected_inchikey,
                found_inchikey,
            } => write!(
                formatter,
                "identity mismatch for {id}: expected {expected_inchikey}, found {found_inchikey}"
            ),
            Self::CanonicalNameMismatch {
                id,
                expected_name,
                found_name,
            } => write!(
                formatter,
                "canonical name mismatch for {id}: expected {expected_name}, found {found_name}"
            ),
            Self::InvalidQuantity { id, field } => {
                write!(formatter, "invalid quantity for {id}.{field}")
            }
            Self::MissingAssociation { id } => {
                write!(
                    formatter,
                    "associating pilot fluid lacks association parameters: {id}"
                )
            }
        }
    }
}

impl std::error::Error for FluidParameterImportError {}

impl From<AdapterError> for FluidParameterImportError {
    fn from(error: AdapterError) -> Self {
        Self::Snapshot {
            detail: error.to_string(),
        }
    }
}

/// Deterministic output of the offline snapshot lane. This artifact remains
/// quarantined: it is evidence for review, not a runtime parameter pack.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FluidParameterImport {
    pub manifest: SnapshotManifest,
    pub candidates: Vec<QuarantinedCandidate>,
    pub report: QuarantineReviewReport,
}

impl FluidParameterImport {
    /// A non-empty rejection/conflict set makes the native generator exit
    /// unsuccessfully after printing the report, so automation fails closed.
    pub fn refuses(&self) -> bool {
        !self.report.identity_conflicts.is_empty()
            || self
                .report
                .reviews
                .iter()
                .any(|review| !review.rejected.is_empty())
    }
}

/// Verify and import a pinned, local snapshot without any filesystem or
/// network access. Native callers own I/O; WASM callers can supply bytes.
pub fn import_verified_snapshot(
    manifest: &SnapshotManifest,
    raw: &[u8],
) -> Result<FluidParameterImport, FluidParameterImportError> {
    manifest.verify(raw)?;
    let document: SourceDocument = serde_json::from_slice(raw).map_err(|error| {
        FluidParameterImportError::MalformedDocument {
            detail: error.to_string(),
        }
    })?;
    require_manifest_match("adapter_id", ADAPTER_ID, &manifest.adapter_id)?;
    require_manifest_match("source_id", &manifest.source_id, &document.source_id)?;
    require_manifest_match(
        "source_revision",
        &manifest.source_revision,
        &document.source_revision,
    )?;
    require_manifest_match(
        "record_count",
        &manifest.record_count.to_string(),
        &document.records.len().to_string(),
    )?;

    let candidates = parse_source_document(raw)?;
    let report = promotion_report(candidates.clone());
    Ok(FluidParameterImport {
        manifest: manifest.clone(),
        candidates,
        report,
    })
}

fn require_manifest_match(
    field: &str,
    expected: &str,
    found: &str,
) -> Result<(), FluidParameterImportError> {
    if expected == found {
        Ok(())
    } else {
        Err(FluidParameterImportError::ManifestMismatch {
            field: field.to_owned(),
            expected: expected.to_owned(),
            found: found.to_owned(),
        })
    }
}

/// Parse and validate a complete six-identity pilot document.
///
/// Candidate order is canonical identity order, independent of source array
/// order. Source paths use stable record IDs rather than array indexes.
pub fn parse_source_document(
    bytes: &[u8],
) -> Result<Vec<QuarantinedCandidate>, FluidParameterImportError> {
    let document: SourceDocument = serde_json::from_slice(bytes).map_err(|error| {
        FluidParameterImportError::MalformedDocument {
            detail: error.to_string(),
        }
    })?;
    for (field, value) in [
        ("source_id", document.source_id.as_str()),
        ("source_revision", document.source_revision.as_str()),
        ("data_licence", document.data_licence.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(FluidParameterImportError::MissingDocumentMetadata {
                field: field.to_owned(),
            });
        }
    }

    let expected: BTreeMap<&str, &str> = PILOT_IDENTITIES.into_iter().collect();
    let expected_names: BTreeMap<&str, &str> = PILOT_CANONICAL_NAMES.into_iter().collect();
    let mut records = BTreeMap::new();
    for record in document.records {
        if !expected.contains_key(record.id.as_str()) {
            return Err(FluidParameterImportError::UnknownCanonicalIdentity { id: record.id });
        }
        let id = record.id.clone();
        if records.insert(id.clone(), record).is_some() {
            return Err(FluidParameterImportError::DuplicateRecord { id });
        }
    }

    PILOT_IDENTITIES
        .into_iter()
        .map(|(id, expected_key)| {
            let record = records.remove(id).ok_or_else(|| {
                FluidParameterImportError::MissingCanonicalIdentity { id: id.to_owned() }
            })?;
            if record.inchikey != expected_key {
                return Err(FluidParameterImportError::IdentityMismatch {
                    id: id.to_owned(),
                    expected_inchikey: expected_key.to_owned(),
                    found_inchikey: record.inchikey,
                });
            }
            let expected_name = expected_names[id];
            if record.canonical_name != expected_name {
                return Err(FluidParameterImportError::CanonicalNameMismatch {
                    id: id.to_owned(),
                    expected_name: expected_name.to_owned(),
                    found_name: record.canonical_name,
                });
            }
            candidate(
                record,
                &document.source_id,
                &document.source_revision,
                &document.data_licence,
            )
        })
        .collect()
}

fn candidate(
    record: SourceRecord,
    source_id: &str,
    revision: &str,
    licence: &str,
) -> Result<QuarantinedCandidate, FluidParameterImportError> {
    let id = record.id;
    let prefix = format!("records[id={id}]");
    let mut fields = BTreeMap::from([
        (
            "canonical_name".to_owned(),
            CandidateField::new(
                Value::String(record.canonical_name),
                format!("{prefix}.canonical_name"),
                licence,
            ),
        ),
        (
            "inchikey".to_owned(),
            CandidateField::new(
                Value::String(record.inchikey.clone()),
                format!("{prefix}.inchikey"),
                licence,
            ),
        ),
    ]);
    let association = record.pc_saft.association;
    for (name, quantity) in [
        ("pc_saft.molar_mass", record.pc_saft.molar_mass),
        ("pc_saft.segment_count", record.pc_saft.segment_count),
        ("pc_saft.segment_diameter", record.pc_saft.segment_diameter),
        (
            "pc_saft.dispersion_energy_k",
            record.pc_saft.dispersion_energy_k,
        ),
    ] {
        if !quantity.value.is_finite() || quantity.value <= 0.0 || quantity.unit.trim().is_empty() {
            return Err(FluidParameterImportError::InvalidQuantity {
                id: id.clone(),
                field: name.to_owned(),
            });
        }
        let value = Number::from_f64(quantity.value)
            .map(Value::Number)
            .ok_or_else(|| FluidParameterImportError::InvalidQuantity {
                id: id.clone(),
                field: name.to_owned(),
            })?;
        fields.insert(
            name.to_owned(),
            CandidateField::new(value, format!("{prefix}.{name}"), licence)
                .with_unit(quantity.unit),
        );
    }
    if matches!(id.as_str(), "water" | "ethanol" | "NH3") && association.is_none() {
        return Err(FluidParameterImportError::MissingAssociation { id });
    }
    if let Some(association) = association {
        if association.na == 0 && association.nb == 0 && association.nc == 0 {
            return Err(FluidParameterImportError::InvalidQuantity {
                id: id.clone(),
                field: "pc_saft.association.site_count".to_owned(),
            });
        }
        for (name, quantity) in [
            ("pc_saft.association.kappa_ab", association.kappa_ab),
            ("pc_saft.association.epsilon_k_ab", association.epsilon_k_ab),
        ] {
            insert_quantity(&mut fields, &id, &prefix, name, quantity, licence)?;
        }
        for (name, count) in [
            ("pc_saft.association.na", association.na),
            ("pc_saft.association.nb", association.nb),
            ("pc_saft.association.nc", association.nc),
        ] {
            fields.insert(
                name.to_owned(),
                CandidateField::new(
                    Value::Number(Number::from(count)),
                    format!("{prefix}.{name}"),
                    licence,
                )
                .with_unit("1"),
            );
        }
    }
    Ok(QuarantinedCandidate {
        adapter_id: ADAPTER_ID.to_owned(),
        source_record_id: format!("{source_id}@{revision}/{id}"),
        external_record_id: id,
        identity_key: Some(record.inchikey),
        fields,
    })
}

fn insert_quantity(
    fields: &mut BTreeMap<String, CandidateField>,
    id: &str,
    prefix: &str,
    name: &str,
    quantity: Quantity,
    licence: &str,
) -> Result<(), FluidParameterImportError> {
    if !quantity.value.is_finite() || quantity.value <= 0.0 || quantity.unit.trim().is_empty() {
        return Err(FluidParameterImportError::InvalidQuantity {
            id: id.to_owned(),
            field: name.to_owned(),
        });
    }
    let value = Number::from_f64(quantity.value)
        .map(Value::Number)
        .ok_or_else(|| FluidParameterImportError::InvalidQuantity {
            id: id.to_owned(),
            field: name.to_owned(),
        })?;
    fields.insert(
        name.to_owned(),
        CandidateField::new(value, format!("{prefix}.{name}"), licence).with_unit(quantity.unit),
    );
    Ok(())
}

/// Fixed promotion policy. The source document cannot widen this allowlist.
pub fn promotion_policy() -> PromotionPolicy {
    let allowed: BTreeSet<String> = default_runtime_data_licences();
    let rule = |target: &str, dimension| RuntimeFieldPolicy {
        target_field: target.to_owned(),
        allowed_licences: allowed.clone(),
        target_dimension: dimension,
    };
    PromotionPolicy {
        fields: BTreeMap::from([
            ("canonical_name".into(), rule("identity.name", None)),
            ("inchikey".into(), rule("identity.inchikey", None)),
            (
                "pc_saft.molar_mass".into(),
                rule("model.pc_saft.molar_mass", Some(Dimension::MolarMass)),
            ),
            (
                "pc_saft.segment_count".into(),
                rule(
                    "model.pc_saft.segment_count",
                    Some(Dimension::Dimensionless),
                ),
            ),
            (
                "pc_saft.segment_diameter".into(),
                rule(
                    "model.pc_saft.segment_diameter",
                    Some(Dimension::MolecularLength),
                ),
            ),
            (
                "pc_saft.dispersion_energy_k".into(),
                rule(
                    "model.pc_saft.dispersion_energy_k",
                    Some(Dimension::Temperature),
                ),
            ),
            (
                "pc_saft.association.kappa_ab".into(),
                rule(
                    "model.pc_saft.association.kappa_ab",
                    Some(Dimension::Dimensionless),
                ),
            ),
            (
                "pc_saft.association.epsilon_k_ab".into(),
                rule(
                    "model.pc_saft.association.epsilon_k_ab",
                    Some(Dimension::Temperature),
                ),
            ),
            (
                "pc_saft.association.na".into(),
                rule(
                    "model.pc_saft.association.na",
                    Some(Dimension::Dimensionless),
                ),
            ),
            (
                "pc_saft.association.nb".into(),
                rule(
                    "model.pc_saft.association.nb",
                    Some(Dimension::Dimensionless),
                ),
            ),
            (
                "pc_saft.association.nc".into(),
                rule(
                    "model.pc_saft.association.nc",
                    Some(Dimension::Dimensionless),
                ),
            ),
        ]),
    }
}

/// Produce the deterministic BRD-003 review report; this never promotes data.
pub fn promotion_report(candidates: Vec<QuarantinedCandidate>) -> QuarantineReviewReport {
    review_candidates(candidates, &promotion_policy())
}
