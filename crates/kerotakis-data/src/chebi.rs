//! BRD-011: the ChEBI identity and ontology adapter.
//!
//! ChEBI (Chemical Entities of Biological Interest, EMBL-EBI) is a manually
//! annotated ontology of small molecules. This adapter reads a *pinned release
//! snapshot* — a curated subset of reviewed three-star entities, checksummed by
//! a [`SnapshotManifest`] — and turns it into [`QuarantinedCandidate`]s. It
//! stops at quarantine like every BRD-003 importer: nothing here writes a
//! runtime record.
//!
//! Four disciplines are load-bearing and each has a test:
//!
//! 1. **Reviewed material only.** Three stars *and* a reviewed status, on the
//!    entity and independently on every ontology relation. ChEBI's `SUBMITTED`
//!    rows are third-party deposits that no curator has checked; they do not
//!    become candidates.
//! 2. **Protonation and tautomer families are reported, never merged.** The
//!    cross-source join key is the *full* Standard InChIKey, so acetic acid and
//!    acetate stay two records with two keys. Their relationship is published
//!    separately as a [`ChebiRelatedForm`] — see [`relationship_report`].
//! 3. **Roles are search tags and nothing else.** ChEBI's `has role` slice is
//!    biological/application annotation, not hazard data and not chemistry.
//!    [`lint_role_firewall`] is default-deny: an ontology-derived field may
//!    only land on a tag target, and *no* ChEBI field may land on a safety or
//!    reactivity target. See [`RoleFirewallViolation`].
//! 4. **Attribution rides on the record.** ChEBI is CC BY 4.0, which obliges us
//!    to attribute per distributed record. Every candidate carries an
//!    `attribution` field naming the database, the pinned release and the
//!    licence, so the text survives into a compiled pack instead of living only
//!    in a build script.
//!
//! ### Licence note
//!
//! The `README` shipped in ChEBI's flat-file directory calls the terms
//! "CC Attribution-ShareAlike 4.0". The `LICENSE` file sitting beside it is the
//! verbatim text of **CC BY 4.0** and contains no ShareAlike condition. The
//! operative grant is the licence text, so this adapter records `CC-BY-4.0`;
//! `provenance/sources.toml` carries the same finding. That distinction
//! matters here: ROADMAP-Webapp.md's 2026-08-23 decision keeps ShareAlike data
//! out of store builds entirely, so a mis-read of that README would have
//! blocked ChEBI from the runtime lane for no reason.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::adapter::{
    CandidateField, PromotionPolicy, QuarantinedCandidate, RuntimeFieldPolicy, SnapshotManifest,
};
use crate::provenance::{
    lint_promotion, EligibleFieldList, PromotionLintInput, ProvenanceLintReport,
};
use crate::schema::Dimension;

/// Adapter identifier. Matches the `adapter_id` in the pinned manifest.
pub const CHEBI_ADAPTER_ID: &str = "chebi-v1";
/// Source identifier, as used in `provenance/sources.toml`.
pub const CHEBI_SOURCE_ID: &str = "chebi";
/// SPDX expression for ChEBI content. See the licence note above.
pub const CHEBI_LICENCE: &str = "CC-BY-4.0";
/// Only fully reviewed ("three-star") entities are ingested.
pub const REVIEWED_STARS: u32 = 3;

/// ChEBI's curation statuses that count as reviewed. `SUBMITTED` is a
/// third-party deposit awaiting curation and is deliberately absent.
pub const REVIEWED_STATUSES: &[&str] = &["CHECKED", "OK"];

/// Name types ingested as aliases. `BRAND NAME` is excluded on purpose: those
/// are third-party trademarks, which a CC BY grant on the database does not
/// license us to reuse as product vocabulary.
pub const ALIAS_NAME_TYPES: &[&str] = &["SYNONYM", "IUPAC NAME", "INN", "UNIPROT NAME"];

/// Absolute tolerance, in daltons, between ChEBI's stated average mass and the
/// mass recomputed from its own formula.
///
/// Calibrated against the pinned snapshot: the largest disagreement across all
/// 81 recomputable entities is 0.007 Da (sulfur and chlorine, whose standard
/// atomic weights were revised after the ChEBI values were computed). This
/// tolerance sits ~7x above that noise floor and far below the >=1 Da error a
/// genuinely wrong formula produces.
pub const MASS_TOLERANCE_DA: f64 = 0.05;
/// Relative tolerance, applied alongside [`MASS_TOLERANCE_DA`] for heavy
/// entities. The observed noise floor is 8.3e-5.
pub const MASS_TOLERANCE_RELATIVE: f64 = 1.0e-3;

// ---------------------------------------------------------------------------
// Pinned snapshot
// ---------------------------------------------------------------------------

/// A pinned ChEBI release snapshot.
///
/// The snapshot is the *raw* extract: it keeps ChEBI's own markup, statuses and
/// star ratings untouched, so the review discipline this module applies is
/// visible in the diff rather than baked into the fixture.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChebiSnapshot {
    pub snapshot_schema: u32,
    pub database: String,
    /// ChEBI release number, e.g. `"253"`.
    pub release: String,
    pub release_date: String,
    pub retrieved: String,
    pub origin: String,
    pub licence: String,
    pub licence_url: String,
    /// The attribution string this release obliges us to carry.
    pub attribution: String,
    pub curation: String,
    pub source_tables: Vec<String>,
    pub entity_count: usize,
    pub entities: Vec<ChebiEntity>,
}

/// One ChEBI entity, as the pinned flat files describe it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChebiEntity {
    pub chebi_accession: String,
    /// ChEBI's display name, which carries presentation markup.
    pub name: String,
    #[serde(default)]
    pub ascii_name: Option<String>,
    #[serde(default)]
    pub definition: Option<String>,
    pub stars: u32,
    pub status: String,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub modified_on: Option<String>,
    #[serde(default)]
    pub formula: Option<String>,
    #[serde(default)]
    pub charge: Option<i64>,
    /// Average molecular mass in daltons.
    #[serde(default)]
    pub mass: Option<f64>,
    #[serde(default)]
    pub monoisotopic_mass: Option<f64>,
    #[serde(default)]
    pub chemical_data_autogenerated: Option<bool>,
    #[serde(default)]
    pub standard_inchi: Option<String>,
    #[serde(default)]
    pub standard_inchi_key: Option<String>,
    #[serde(default)]
    pub smiles: Option<String>,
    #[serde(default)]
    pub synonyms: Vec<ChebiSynonym>,
    #[serde(default)]
    pub relations: Vec<ChebiRelation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChebiSynonym {
    pub name: String,
    #[serde(rename = "type")]
    pub name_type: String,
    pub status: String,
    #[serde(default)]
    pub language: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChebiRelation {
    /// One of `has_role`, `is_a`, `is_conjugate_acid_of`,
    /// `is_conjugate_base_of`, `is_tautomer_of`.
    pub relation: String,
    pub target: String,
    pub target_name: String,
    pub status: String,
}

impl ChebiEntity {
    /// Whether ChEBI's curators have reviewed this entity.
    #[must_use]
    pub fn is_reviewed(&self) -> bool {
        self.stars == REVIEWED_STARS && REVIEWED_STATUSES.contains(&self.status.as_str())
    }

    /// Relations of one kind that are themselves curator-reviewed.
    fn reviewed_relations<'a>(&'a self, kind: &'a str) -> impl Iterator<Item = &'a ChebiRelation> {
        self.relations.iter().filter(move |relation| {
            relation.relation == kind && REVIEWED_STATUSES.contains(&relation.status.as_str())
        })
    }
}

impl ChebiSnapshot {
    /// Parse pinned snapshot bytes.
    ///
    /// # Errors
    /// Returns [`ChebiError::Malformed`] if the bytes are not a snapshot
    /// document, and [`ChebiError::UnsupportedSnapshotSchema`] if they describe
    /// a schema this build does not implement.
    pub fn parse(raw: &[u8]) -> Result<Self, ChebiError> {
        let snapshot: Self = serde_json::from_slice(raw)
            .map_err(|error| ChebiError::Malformed(error.to_string()))?;
        if snapshot.snapshot_schema != CHEBI_SNAPSHOT_SCHEMA {
            return Err(ChebiError::UnsupportedSnapshotSchema {
                found: snapshot.snapshot_schema,
                expected: CHEBI_SNAPSHOT_SCHEMA,
            });
        }
        Ok(snapshot)
    }

    /// Verify the snapshot against its manifest, then parse it.
    ///
    /// This is the entry point an importer should use: it refuses bytes whose
    /// checksum does not match the pinned release before interpreting them.
    ///
    /// # Errors
    /// Returns [`ChebiError::Manifest`] when the manifest rejects the bytes,
    /// [`ChebiError::AdapterMismatch`] when the manifest belongs to a different
    /// adapter, and the parse errors above otherwise.
    pub fn verified(manifest: &SnapshotManifest, raw: &[u8]) -> Result<Self, ChebiError> {
        manifest
            .verify(raw)
            .map_err(|error| ChebiError::Manifest(error.to_string()))?;
        if manifest.adapter_id != CHEBI_ADAPTER_ID {
            return Err(ChebiError::AdapterMismatch {
                found: manifest.adapter_id.clone(),
                expected: CHEBI_ADAPTER_ID,
            });
        }
        Self::parse(raw)
    }

    /// The reviewed entities, in pinned-release order.
    pub fn reviewed(&self) -> impl Iterator<Item = &ChebiEntity> {
        self.entities.iter().filter(|entity| entity.is_reviewed())
    }
}

/// Snapshot document schema this build understands.
pub const CHEBI_SNAPSHOT_SCHEMA: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChebiError {
    Malformed(String),
    UnsupportedSnapshotSchema {
        found: u32,
        expected: u32,
    },
    Manifest(String),
    AdapterMismatch {
        found: String,
        expected: &'static str,
    },
}

impl std::fmt::Display for ChebiError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Malformed(detail) => write!(formatter, "malformed ChEBI snapshot: {detail}"),
            Self::UnsupportedSnapshotSchema { found, expected } => write!(
                formatter,
                "unsupported ChEBI snapshot schema {found}; expected {expected}"
            ),
            Self::Manifest(detail) => write!(formatter, "ChEBI snapshot manifest: {detail}"),
            Self::AdapterMismatch { found, expected } => write!(
                formatter,
                "snapshot manifest is for adapter {found}, not {expected}"
            ),
        }
    }
}

impl std::error::Error for ChebiError {}

// ---------------------------------------------------------------------------
// Label normalization
// ---------------------------------------------------------------------------

/// Strip ChEBI's presentation markup and normalize its typographic dashes.
///
/// ChEBI names carry HTML for display — `<small>D</small>-glucose`,
/// `NAD<small><sup>+</small></sup>` — and use U+2212 MINUS SIGN and U+2012
/// FIGURE DASH in charge suffixes such as `citrate(3−)`. Both would make an
/// otherwise-identical name compare unequal to anything a user types, so they
/// are folded to ASCII `-` here.
///
/// Everything else is preserved deliberately: Greek letters (`α`, `β`), arrows
/// (`(1→4)-β-D-glucan`) and umlauts in the German synonyms are part of the
/// chemistry or the language, not presentation.
#[must_use]
pub fn normalize_chebi_label(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut in_tag = false;
    for ch in raw.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if in_tag => {}
            // U+2212 MINUS SIGN, U+2012 FIGURE DASH, U+2013 EN DASH.
            '\u{2212}' | '\u{2012}' | '\u{2013}' => out.push('-'),
            _ => out.push(ch),
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

// ---------------------------------------------------------------------------
// Independent identity recomputation
// ---------------------------------------------------------------------------

/// Standard atomic weights (IUPAC 2021, conventional values) for the elements
/// the pinned snapshot uses.
///
/// Deliberately local and deliberately *not* ChEBI's own table: the point of
/// recomputing a mass is to disagree with the source when the source is wrong,
/// which a shared table could not do. It is scoped to the fixture's elements so
/// an entity outside that scope is reported as unrecomputable rather than
/// silently mis-massed.
const ATOMIC_WEIGHTS: &[(&str, f64)] = &[
    ("H", 1.008),
    ("C", 12.011),
    ("N", 14.007),
    ("O", 15.999),
    ("Na", 22.989_769_28),
    ("Mg", 24.305),
    ("P", 30.973_761_998),
    ("S", 32.06),
    ("Cl", 35.45),
    ("K", 39.098_3),
    ("Ca", 40.078),
];

fn atomic_weight(symbol: &str) -> Option<f64> {
    ATOMIC_WEIGHTS
        .iter()
        .find(|(element, _)| *element == symbol)
        .map(|(_, weight)| *weight)
}

/// Why a ChEBI formula could not be turned back into a mass.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "issue", rename_all = "snake_case")]
pub enum FormulaIssue {
    /// The formula contains an indeterminate repeat count, as polymers do:
    /// `(C6H10O5)n`. No finite mass follows from it.
    IndeterminateRepeat,
    /// An element outside this adapter's reviewed weight table.
    UnknownElement { symbol: String },
    /// The string is not a formula this parser accepts.
    Malformed { detail: String },
}

impl std::fmt::Display for FormulaIssue {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IndeterminateRepeat => {
                write!(formatter, "indeterminate repeat count (polymer formula)")
            }
            Self::UnknownElement { symbol } => {
                write!(formatter, "no reviewed atomic weight for element {symbol}")
            }
            Self::Malformed { detail } => write!(formatter, "malformed formula: {detail}"),
        }
    }
}

/// Recompute an average molecular mass from a ChEBI formula.
///
/// Accepts ChEBI's flat-file dialect: dot-separated components (`Cl.Na`,
/// `CO3.Ca`), parenthesised groups with a repeat count, and an optional leading
/// component multiplier (`2H2O`).
///
/// # Errors
/// Returns the [`FormulaIssue`] describing why no finite mass follows.
pub fn recompute_formula_mass(formula: &str) -> Result<f64, FormulaIssue> {
    let mut total = 0.0;
    for component in formula.split('.') {
        if component.is_empty() {
            return Err(FormulaIssue::Malformed {
                detail: "empty component".into(),
            });
        }
        total += component_mass(component)?;
    }
    Ok(total)
}

fn component_mass(component: &str) -> Result<f64, FormulaIssue> {
    let chars: Vec<char> = component.chars().collect();
    let mut index = 0;

    // Optional leading multiplier, e.g. the `2` in `2H2O`.
    let mut multiplier = 1.0;
    let digits = take_digits(&chars, &mut index);
    if !digits.is_empty() {
        multiplier = digits.parse::<f64>().map_err(|_| FormulaIssue::Malformed {
            detail: format!("component multiplier {digits}"),
        })?;
    }

    // Parenthesised groups nest, so accumulate on a stack.
    let mut stack = vec![0.0_f64];
    while index < chars.len() {
        match chars[index] {
            '(' => {
                stack.push(0.0);
                index += 1;
            }
            ')' => {
                index += 1;
                let count = take_count(&chars, &mut index)?;
                let group = stack.pop().ok_or_else(|| FormulaIssue::Malformed {
                    detail: "unbalanced ')'".into(),
                })?;
                let outer = stack.last_mut().ok_or_else(|| FormulaIssue::Malformed {
                    detail: "unbalanced ')'".into(),
                })?;
                *outer += group * count;
            }
            ch if ch.is_ascii_uppercase() => {
                let mut symbol = String::from(ch);
                index += 1;
                if index < chars.len() && chars[index].is_ascii_lowercase() {
                    symbol.push(chars[index]);
                    index += 1;
                }
                let weight =
                    atomic_weight(&symbol).ok_or(FormulaIssue::UnknownElement { symbol })?;
                let count = take_count(&chars, &mut index)?;
                *stack.last_mut().expect("stack always has a frame") += weight * count;
            }
            other => {
                return Err(FormulaIssue::Malformed {
                    detail: format!("unexpected character {other:?}"),
                })
            }
        }
    }

    if stack.len() != 1 {
        return Err(FormulaIssue::Malformed {
            detail: "unclosed '('".into(),
        });
    }
    Ok(stack[0] * multiplier)
}

fn take_digits(chars: &[char], index: &mut usize) -> String {
    let start = *index;
    while *index < chars.len() && chars[*index].is_ascii_digit() {
        *index += 1;
    }
    chars[start..*index].iter().collect()
}

/// A subscript: an integer, an absent one meaning 1, or `n` — the polymer
/// marker that makes the whole formula indeterminate.
fn take_count(chars: &[char], index: &mut usize) -> Result<f64, FormulaIssue> {
    if *index < chars.len() && chars[*index] == 'n' {
        return Err(FormulaIssue::IndeterminateRepeat);
    }
    let digits = take_digits(chars, index);
    if digits.is_empty() {
        return Ok(1.0);
    }
    digits.parse::<f64>().map_err(|_| FormulaIssue::Malformed {
        detail: format!("subscript {digits}"),
    })
}

/// Net charge implied by a Standard InChI's `/q` and `/p` layers.
///
/// `/q` is the per-component formal charge and `/p` the proton
/// added/removed count; both are `;`-separated across the components of a
/// multi-component InChI and may carry a `count*value` multiplier. Sodium
/// chloride, `InChI=1S/ClH.Na/h1H;/q;+1/p-1`, nets to zero only when both
/// layers are read — which is exactly why this is an independent check on
/// ChEBI's stated charge rather than a restatement of it.
#[must_use]
pub fn inchi_charge(inchi: &str) -> Option<i64> {
    let mut total = 0;
    let mut seen = false;
    for prefix in ["/q", "/p"] {
        let Some(start) = inchi.find(prefix) else {
            continue;
        };
        let rest = &inchi[start + prefix.len()..];
        let layer = rest.split('/').next().unwrap_or("");
        seen = true;
        for part in layer.split(';') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            let (repeat, value) = match part.split_once('*') {
                Some((count, value)) => (count.parse::<i64>().ok()?, value),
                None => (1, part),
            };
            total += repeat * value.parse::<i64>().ok()?;
        }
    }
    seen.then_some(total)
}

// ---------------------------------------------------------------------------
// Protonation / tautomer families — reported, never merged
// ---------------------------------------------------------------------------

/// The InChIKey prefix shared by every protonation and tautomer state of one
/// structure: the 14-character skeleton block *and* the 10-character
/// stereo/isotope block.
///
/// Using the skeleton block alone would be wrong, and the pinned fixture proves
/// it: maltose and lactose share `GUBGYTABKSRVRQ` but differ in the second
/// block because they are diastereomers, not protonation states of one
/// another. So do cellulose and amylose. Requiring both blocks separates the
/// 14 genuine families from those two look-alikes.
#[must_use]
pub fn inchikey_family(key: &str) -> Option<&str> {
    let bytes = key.as_bytes();
    if bytes.len() != 27 || bytes[14] != b'-' || bytes[25] != b'-' {
        return None;
    }
    Some(&key[..25])
}

/// How a pair of ChEBI entities is known to be related.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelatedFormEvidence {
    /// ChEBI asserts the relation and the structures agree.
    OntologyAndStructure,
    /// ChEBI asserts the relation; no shared structural family confirms it,
    /// usually because one side has no single defined structure.
    OntologyOnly,
    /// The InChIKeys place both in one family but ChEBI asserts no *direct*
    /// relation — typical of a multi-step protonation chain such as citric
    /// acid to citrate(3-), which ChEBI links through the intermediates.
    StructureOnly,
}

/// A reported relationship between two distinct ChEBI identities.
///
/// This type exists so a protonation or tautomer pair has somewhere to go
/// *other* than a merge. Both members keep their own ChEBI ID and their own
/// InChIKey; this records that a reviewer should look at them together.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChebiRelatedForm {
    /// The two accessions, ordered so the report is stable.
    pub left: String,
    pub right: String,
    pub left_name: String,
    pub right_name: String,
    /// The InChIKeys, which differ — that difference is the point.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left_identity_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub right_identity_key: Option<String>,
    /// The ChEBI relation names asserted between them, if any.
    pub asserted_relations: Vec<String>,
    pub evidence: RelatedFormEvidence,
}

/// The related-form report for a snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChebiRelationshipReport {
    pub adapter_id: String,
    pub source_revision: String,
    pub related_forms: Vec<ChebiRelatedForm>,
}

impl ChebiRelationshipReport {
    /// Related forms involving a given accession.
    pub fn involving<'a>(
        &'a self,
        accession: &'a str,
    ) -> impl Iterator<Item = &'a ChebiRelatedForm> {
        self.related_forms
            .iter()
            .filter(move |form| form.left == accession || form.right == accession)
    }
}

/// Report protonation and tautomer families without merging any of them.
///
/// Two independent signals are combined: ChEBI's own
/// `is_conjugate_acid_of` / `is_conjugate_base_of` / `is_tautomer_of`
/// assertions, and shared [`inchikey_family`] membership. Where they disagree
/// the report says so via [`RelatedFormEvidence`] rather than picking a winner.
#[must_use]
pub fn relationship_report(snapshot: &ChebiSnapshot) -> ChebiRelationshipReport {
    const FAMILY_RELATIONS: &[&str] = &[
        "is_conjugate_acid_of",
        "is_conjugate_base_of",
        "is_tautomer_of",
    ];

    let reviewed: Vec<&ChebiEntity> = snapshot.reviewed().collect();
    let in_set: BTreeSet<&str> = reviewed
        .iter()
        .map(|entity| entity.chebi_accession.as_str())
        .collect();

    // Pair -> asserted relation names.
    let mut asserted: BTreeMap<(String, String), BTreeSet<String>> = BTreeMap::new();
    for entity in &reviewed {
        for kind in FAMILY_RELATIONS {
            for relation in entity.reviewed_relations(kind) {
                if !in_set.contains(relation.target.as_str()) {
                    continue;
                }
                let pair = ordered_pair(&entity.chebi_accession, &relation.target);
                asserted.entry(pair).or_default().insert((*kind).to_owned());
            }
        }
    }

    // Structural families by shared InChIKey prefix.
    let mut families: BTreeMap<&str, Vec<&ChebiEntity>> = BTreeMap::new();
    for entity in &reviewed {
        if let Some(family) = entity
            .standard_inchi_key
            .as_deref()
            .and_then(inchikey_family)
        {
            families.entry(family).or_default().push(entity);
        }
    }
    let mut structural: BTreeSet<(String, String)> = BTreeSet::new();
    for members in families.values() {
        for (index, left) in members.iter().enumerate() {
            for right in &members[index + 1..] {
                structural.insert(ordered_pair(&left.chebi_accession, &right.chebi_accession));
            }
        }
    }

    let by_accession: BTreeMap<&str, &ChebiEntity> = reviewed
        .iter()
        .map(|entity| (entity.chebi_accession.as_str(), *entity))
        .collect();

    let mut pairs: BTreeSet<(String, String)> = asserted.keys().cloned().collect();
    pairs.extend(structural.iter().cloned());

    let related_forms = pairs
        .into_iter()
        .filter_map(|pair| {
            let left = by_accession.get(pair.0.as_str())?;
            let right = by_accession.get(pair.1.as_str())?;
            let relations = asserted.get(&pair);
            let evidence = match (relations.is_some(), structural.contains(&pair)) {
                (true, true) => RelatedFormEvidence::OntologyAndStructure,
                (true, false) => RelatedFormEvidence::OntologyOnly,
                (false, true) => RelatedFormEvidence::StructureOnly,
                (false, false) => return None,
            };
            Some(ChebiRelatedForm {
                left: pair.0.clone(),
                right: pair.1.clone(),
                left_name: normalize_chebi_label(&left.name),
                right_name: normalize_chebi_label(&right.name),
                left_identity_key: left.standard_inchi_key.clone(),
                right_identity_key: right.standard_inchi_key.clone(),
                asserted_relations: relations
                    .map(|set| set.iter().cloned().collect())
                    .unwrap_or_default(),
                evidence,
            })
        })
        .collect();

    ChebiRelationshipReport {
        adapter_id: CHEBI_ADAPTER_ID.to_owned(),
        source_revision: snapshot.release.clone(),
        related_forms,
    }
}

fn ordered_pair(left: &str, right: &str) -> (String, String) {
    if left <= right {
        (left.to_owned(), right.to_owned())
    } else {
        (right.to_owned(), left.to_owned())
    }
}

// ---------------------------------------------------------------------------
// Conflict report
// ---------------------------------------------------------------------------

/// A disagreement between what ChEBI states and what its own data implies.
///
/// These are reported, not repaired. An importer that "fixed" a mass by
/// overwriting it would destroy the evidence that the pinned release needs a
/// curator's attention.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "conflict", rename_all = "snake_case")]
pub enum ChebiConflict {
    /// The stated average mass disagrees with the mass recomputed from the
    /// stated formula, beyond the atomic-weight-revision tolerance.
    MassDisagreesWithFormula {
        chebi_id: String,
        formula: String,
        stated_mass: f64,
        recomputed_mass: f64,
        difference: f64,
        tolerance: f64,
    },
    /// A mass is stated but the formula yields none — a polymer's
    /// indeterminate repeat, or an element outside the reviewed table. The
    /// stated number cannot be corroborated, so it must not be promoted as if
    /// it had been.
    MassNotRecomputable {
        chebi_id: String,
        formula: String,
        stated_mass: f64,
        issue: FormulaIssue,
    },
    /// The stated charge disagrees with the charge implied by the Standard
    /// InChI's `/q` and `/p` layers.
    ChargeDisagreesWithStructure {
        chebi_id: String,
        stated_charge: i64,
        structural_charge: i64,
    },
    /// The entity carries no Standard InChIKey, so it has no cross-source join
    /// key. Typical of an ontology class such as `D-glucose`, which covers
    /// several defined structures rather than being one.
    NoIdentityKey { chebi_id: String, name: String },
    /// The entity carries no formula or mass at all.
    NoChemicalData { chebi_id: String, name: String },
}

impl ChebiConflict {
    /// The accession this conflict is about.
    #[must_use]
    pub fn chebi_id(&self) -> &str {
        match self {
            Self::MassDisagreesWithFormula { chebi_id, .. }
            | Self::MassNotRecomputable { chebi_id, .. }
            | Self::ChargeDisagreesWithStructure { chebi_id, .. }
            | Self::NoIdentityKey { chebi_id, .. }
            | Self::NoChemicalData { chebi_id, .. } => chebi_id,
        }
    }
}

/// Cross-check every reviewed entity's stated identity against a recomputation
/// of it, in pinned-release order.
#[must_use]
pub fn conflict_report(snapshot: &ChebiSnapshot) -> Vec<ChebiConflict> {
    let mut conflicts = Vec::new();
    for entity in snapshot.reviewed() {
        let chebi_id = entity.chebi_accession.clone();

        match (&entity.formula, entity.mass) {
            (Some(formula), Some(stated_mass)) => match recompute_formula_mass(formula) {
                Ok(recomputed_mass) => {
                    let difference = (recomputed_mass - stated_mass).abs();
                    let tolerance =
                        MASS_TOLERANCE_DA.max(stated_mass.abs() * MASS_TOLERANCE_RELATIVE);
                    if difference > tolerance {
                        conflicts.push(ChebiConflict::MassDisagreesWithFormula {
                            chebi_id: chebi_id.clone(),
                            formula: formula.clone(),
                            stated_mass,
                            recomputed_mass,
                            difference,
                            tolerance,
                        });
                    }
                }
                Err(issue) => conflicts.push(ChebiConflict::MassNotRecomputable {
                    chebi_id: chebi_id.clone(),
                    formula: formula.clone(),
                    stated_mass,
                    issue,
                }),
            },
            (None, None) => conflicts.push(ChebiConflict::NoChemicalData {
                chebi_id: chebi_id.clone(),
                name: normalize_chebi_label(&entity.name),
            }),
            _ => {}
        }

        if let (Some(stated_charge), Some(structural_charge)) = (
            entity.charge,
            entity.standard_inchi.as_deref().and_then(inchi_charge),
        ) {
            if stated_charge != structural_charge {
                conflicts.push(ChebiConflict::ChargeDisagreesWithStructure {
                    chebi_id: chebi_id.clone(),
                    stated_charge,
                    structural_charge,
                });
            }
        }

        if entity
            .standard_inchi_key
            .as_deref()
            .unwrap_or("")
            .is_empty()
        {
            conflicts.push(ChebiConflict::NoIdentityKey {
                chebi_id,
                name: normalize_chebi_label(&entity.name),
            });
        }
    }
    conflicts
}

// ---------------------------------------------------------------------------
// Candidates
// ---------------------------------------------------------------------------

/// Candidate field names this adapter emits.
pub mod fields {
    pub const CHEBI_ID: &str = "chebi_id";
    pub const CANONICAL_NAME: &str = "canonical_name";
    pub const DEFINITION: &str = "definition";
    pub const SYNONYMS: &str = "synonyms";
    pub const FORMULA: &str = "formula";
    pub const CHARGE: &str = "charge";
    pub const AVERAGE_MASS: &str = "average_mass";
    pub const STANDARD_INCHI: &str = "standard_inchi";
    pub const STANDARD_INCHI_KEY: &str = "standard_inchi_key";
    /// Ontology-derived. Firewalled — see [`super::lint_role_firewall`].
    pub const SEARCH_TAGS_FROM_ROLES: &str = "search_tags_from_roles";
    /// Ontology-derived. Firewalled.
    pub const SEARCH_TAGS_FROM_PARENTS: &str = "search_tags_from_parents";
    pub const ATTRIBUTION: &str = "attribution";
}

/// The candidate fields derived from ChEBI's ontology graph.
///
/// Membership here is what [`lint_role_firewall`] gates. Adding a field to this
/// list without giving it a tag target is a refusal, by design.
pub const ONTOLOGY_DERIVED_FIELDS: &[&str] = &[
    fields::SEARCH_TAGS_FROM_ROLES,
    fields::SEARCH_TAGS_FROM_PARENTS,
];

/// The only runtime targets an ontology-derived field may reach.
pub const ALLOWED_ONTOLOGY_TARGETS: &[&str] = &["search_tags", "class_tags"];

/// Turn a verified snapshot into quarantined candidates.
///
/// One candidate per reviewed entity, in pinned-release order. Unreviewed
/// entities are dropped rather than marked, because a candidate is an offer to
/// promote and an unreviewed ChEBI deposit is not one.
///
/// The `identity_key` is the *full* Standard InChIKey. Entities without one —
/// ontology classes such as `D-glucose` — get `None` rather than a fabricated
/// key; [`conflict_report`] records each of them.
#[must_use]
pub fn chebi_candidates(snapshot: &ChebiSnapshot) -> Vec<QuarantinedCandidate> {
    let attribution = snapshot.attribution.clone();
    snapshot
        .reviewed()
        .map(|entity| candidate_for(snapshot, entity, &attribution))
        .collect()
}

fn candidate_for(
    snapshot: &ChebiSnapshot,
    entity: &ChebiEntity,
    attribution: &str,
) -> QuarantinedCandidate {
    let accession = &entity.chebi_accession;
    // Source paths are keyed by accession, not by array index: a later release
    // reorders entities, and a provenance path that drifts is worse than none.
    let path = |leaf: &str| format!("entities[{accession}].{leaf}");

    // (candidate field, source leaf, value, source unit spelling).
    let mut entries: Vec<(&str, &str, Value, Option<&str>)> = vec![
        (
            fields::CHEBI_ID,
            "chebi_accession",
            Value::String(accession.clone()),
            None,
        ),
        (
            fields::CANONICAL_NAME,
            "name",
            Value::String(normalize_chebi_label(&entity.name)),
            None,
        ),
    ];
    if let Some(definition) = entity.definition.as_deref() {
        entries.push((
            fields::DEFINITION,
            "definition",
            Value::String(normalize_chebi_label(definition)),
            None,
        ));
    }

    let synonyms: Vec<Value> = entity
        .synonyms
        .iter()
        .filter(|synonym| {
            ALIAS_NAME_TYPES.contains(&synonym.name_type.as_str())
                && REVIEWED_STATUSES.contains(&synonym.status.as_str())
        })
        .map(|synonym| {
            json!({
                "name": normalize_chebi_label(&synonym.name),
                "name_type": synonym.name_type,
                "language": synonym.language,
            })
        })
        .collect();
    if !synonyms.is_empty() {
        entries.push((fields::SYNONYMS, "synonyms", Value::Array(synonyms), None));
    }

    if let Some(formula) = entity.formula.as_deref() {
        entries.push((
            fields::FORMULA,
            "formula",
            Value::String(formula.to_owned()),
            None,
        ));
    }
    if let Some(charge) = entity.charge {
        entries.push((fields::CHARGE, "charge", Value::from(charge), None));
    }
    if let Some(mass) = entity.mass {
        // ChEBI states average molecular mass in daltons; the unit spelling is
        // recorded verbatim and normalized at review, never rewritten here.
        entries.push((fields::AVERAGE_MASS, "mass", Value::from(mass), Some("Da")));
    }
    if let Some(inchi) = entity.standard_inchi.as_deref() {
        entries.push((
            fields::STANDARD_INCHI,
            "standard_inchi",
            Value::String(inchi.to_owned()),
            None,
        ));
    }
    if let Some(key) = entity.standard_inchi_key.as_deref() {
        entries.push((
            fields::STANDARD_INCHI_KEY,
            "standard_inchi_key",
            Value::String(key.to_owned()),
            None,
        ));
    }

    let roles = ontology_tags(entity, "has_role");
    if !roles.is_empty() {
        entries.push((
            fields::SEARCH_TAGS_FROM_ROLES,
            "relations[has_role]",
            Value::Array(roles),
            None,
        ));
    }
    let parents = ontology_tags(entity, "is_a");
    if !parents.is_empty() {
        entries.push((
            fields::SEARCH_TAGS_FROM_PARENTS,
            "relations[is_a]",
            Value::Array(parents),
            None,
        ));
    }

    // CC BY obliges attribution on every distributed record, so it travels as
    // an ordinary promotable field rather than as build-script trivia.
    entries.push((
        fields::ATTRIBUTION,
        "attribution",
        Value::String(attribution.to_owned()),
        None,
    ));

    let fields = entries
        .into_iter()
        .map(|(name, leaf, value, unit)| {
            let mut field = CandidateField::new(value, path(leaf), CHEBI_LICENCE);
            if let Some(unit) = unit {
                field = field.with_unit(unit);
            }
            (name.to_owned(), field)
        })
        .collect();

    QuarantinedCandidate {
        adapter_id: CHEBI_ADAPTER_ID.to_owned(),
        source_record_id: format!("{}:{}", snapshot.release, accession),
        external_record_id: accession.clone(),
        identity_key: entity
            .standard_inchi_key
            .as_deref()
            .filter(|key| !key.is_empty())
            .map(str::to_owned),
        fields,
    }
}

fn ontology_tags(entity: &ChebiEntity, kind: &str) -> Vec<Value> {
    entity
        .reviewed_relations(kind)
        .map(|relation| {
            json!({
                "chebi_id": relation.target,
                "label": normalize_chebi_label(&relation.target_name),
            })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Promotion policy and the role firewall
// ---------------------------------------------------------------------------

/// The reviewed promotion policy for ChEBI candidates.
///
/// Ontology-derived fields land on tag targets only. Nothing here reaches a
/// safety or reactivity target, and [`lint_role_firewall`] enforces that
/// independently of this function so a later edit cannot quietly widen it.
#[must_use]
pub fn chebi_promotion_policy() -> PromotionPolicy {
    let licences = || [CHEBI_LICENCE];
    let mut policy = PromotionPolicy::default();
    let mut add = |source: &str, target: &str| {
        policy.fields.insert(
            source.to_owned(),
            RuntimeFieldPolicy::new(target, licences()),
        );
    };

    add(fields::CHEBI_ID, "external_identifier_chebi");
    add(fields::CANONICAL_NAME, "name");
    add(fields::DEFINITION, "description");
    add(fields::SYNONYMS, "aliases");
    add(fields::FORMULA, "formula");
    add(fields::CHARGE, "charge");
    add(fields::STANDARD_INCHI, "standard_inchi");
    add(fields::STANDARD_INCHI_KEY, "standard_inchi_key");
    add(fields::SEARCH_TAGS_FROM_ROLES, "search_tags");
    add(fields::SEARCH_TAGS_FROM_PARENTS, "class_tags");
    add(fields::ATTRIBUTION, "attribution");

    policy.fields.insert(
        fields::AVERAGE_MASS.to_owned(),
        RuntimeFieldPolicy::new("molar_mass", licences()).with_dimension(Dimension::MolarMass),
    );
    policy
}

/// Runtime target names ChEBI data must never reach.
///
/// Matched as substrings so a variant spelling — `safety_flags`,
/// `ghs_hazard_class`, `reactivity_notes` — is caught rather than slipping past
/// a fixed list. ChEBI carries no hazard assessment and no kinetics; a role
/// such as "neurotoxin" is a curated biological annotation, and turning it into
/// a safety claim or a reaction rule would invent an authority ChEBI never
/// asserted.
pub const RESERVED_TARGET_MARKERS: &[&str] = &[
    "safety",
    "hazard",
    "ghs",
    "precaution",
    "toxicity",
    "reactivity",
    "reaction_rule",
    "incompat",
    "flammab",
    "pictogram",
];

/// A refusal from the role firewall.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "violation", rename_all = "snake_case")]
pub enum RoleFirewallViolation {
    /// An ontology-derived field is aimed somewhere other than a tag target.
    /// Default-deny: the target does not have to look dangerous to be refused.
    OntologyFieldTargetsNonTag {
        source_field: String,
        target_field: String,
        allowed_targets: Vec<String>,
    },
    /// Any ChEBI field aimed at a safety or reactivity target.
    ReservedTargetFromChebi {
        source_field: String,
        target_field: String,
        marker: String,
    },
    /// A tag target fed by something that is not an ontology-derived field.
    /// Keeps the tag lane honest in the other direction: if a mass could land
    /// on `search_tags`, the firewall's guarantee would be about field names
    /// rather than about data.
    TagTargetFromNonOntologyField {
        source_field: String,
        target_field: String,
    },
}

impl std::fmt::Display for RoleFirewallViolation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OntologyFieldTargetsNonTag {
                source_field,
                target_field,
                allowed_targets,
            } => write!(
                formatter,
                "ontology-derived field {source_field} targets {target_field}; \
                 ChEBI ontology may only reach {}",
                allowed_targets.join(", ")
            ),
            Self::ReservedTargetFromChebi {
                source_field,
                target_field,
                marker,
            } => write!(
                formatter,
                "field {source_field} targets {target_field}, a reserved \
                 safety/reactivity target (matched {marker:?}); ChEBI asserts no \
                 hazard or kinetic claim"
            ),
            Self::TagTargetFromNonOntologyField {
                source_field,
                target_field,
            } => write!(
                formatter,
                "non-ontology field {source_field} targets tag field {target_field}"
            ),
        }
    }
}

/// Refuse any promotion policy that would let a ChEBI role escape the tag lane.
///
/// Deterministic in policy order, so a report can be diffed.
#[must_use]
pub fn lint_role_firewall(policy: &PromotionPolicy) -> Vec<RoleFirewallViolation> {
    let mut violations = Vec::new();
    for (source_field, rule) in &policy.fields {
        let target = rule.target_field.as_str();
        let ontology_derived = ONTOLOGY_DERIVED_FIELDS.contains(&source_field.as_str());
        let tag_target = ALLOWED_ONTOLOGY_TARGETS.contains(&target);

        if let Some(marker) = RESERVED_TARGET_MARKERS
            .iter()
            .find(|marker| target.to_ascii_lowercase().contains(**marker))
        {
            violations.push(RoleFirewallViolation::ReservedTargetFromChebi {
                source_field: source_field.clone(),
                target_field: rule.target_field.clone(),
                marker: (*marker).to_owned(),
            });
        }

        if ontology_derived && !tag_target {
            violations.push(RoleFirewallViolation::OntologyFieldTargetsNonTag {
                source_field: source_field.clone(),
                target_field: rule.target_field.clone(),
                allowed_targets: ALLOWED_ONTOLOGY_TARGETS
                    .iter()
                    .map(|target| (*target).to_owned())
                    .collect(),
            });
        }
        if tag_target && !ontology_derived {
            violations.push(RoleFirewallViolation::TagTargetFromNonOntologyField {
                source_field: source_field.clone(),
                target_field: rule.target_field.clone(),
            });
        }
    }
    violations
}

/// The full ChEBI promotion verdict: the BRD-003 lint plus this adapter's
/// firewall and identity cross-checks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChebiLintReport {
    pub provenance: ProvenanceLintReport,
    pub role_firewall: Vec<RoleFirewallViolation>,
    /// Reported for a reviewer's attention; these do not by themselves refuse.
    pub conflicts: Vec<ChebiConflict>,
    pub related_forms: ChebiRelationshipReport,
}

impl ChebiLintReport {
    /// Whether promotion must stop.
    ///
    /// Conflicts and related forms are *reports*: a protonation pair is normal
    /// chemistry, not an error. A provenance violation or a firewall breach is
    /// a refusal.
    #[must_use]
    pub fn refuses(&self) -> bool {
        self.provenance.refuses() || !self.role_firewall.is_empty()
    }
}

/// Run the complete ChEBI promotion dry run over a pinned snapshot.
///
/// Composes [`lint_promotion`] — which owns checksum, licence-lane, unit and
/// eligible-field checking — with the ChEBI-specific role firewall, so neither
/// check restates the other.
#[must_use]
pub fn lint_chebi_promotion(
    manifest: &SnapshotManifest,
    raw_snapshot: &[u8],
    snapshot: &ChebiSnapshot,
    candidates: &[QuarantinedCandidate],
    policy: &PromotionPolicy,
    allowed_runtime_licences: &BTreeSet<String>,
    eligible_fields: &[EligibleFieldList],
) -> ChebiLintReport {
    let provenance = lint_promotion(&PromotionLintInput {
        manifest,
        raw_snapshot,
        candidates,
        policy,
        allowed_runtime_licences,
        eligible_fields,
    });
    ChebiLintReport {
        provenance,
        role_firewall: lint_role_firewall(policy),
        conflicts: conflict_report(snapshot),
        related_forms: relationship_report(snapshot),
    }
}
