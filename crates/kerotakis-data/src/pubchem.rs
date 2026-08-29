//! BRD-010: the PubChem identity and approved-property adapter.
//!
//! This module turns a pinned PubChem snapshot into BRD-003
//! [`QuarantinedCandidate`]s. It stops there. Nothing here writes
//! `registry-source-v1.json`, and nothing here decides that a field may ship:
//! that decision is [`PromotionPolicy`], reviewed by a person, applied by
//! [`review_candidates`](crate::review_candidates) and gated by
//! [`lint_promotion`](crate::lint_promotion).
//!
//! # What "found in PubChem" does and does not mean
//!
//! PubChem is two things stacked in one record, and the licence differs:
//!
//! * a **computed/curated core** — CID, the two SMILES flavours, the Standard
//!   InChI and InChIKey, formula, formal charge, the masses, the LexiChem
//!   IUPAC name and PubChem's own record title. This is NCBI/NLM's own work
//!   and the tree books it as `LicenseRef-PubChem-Public-Domain`.
//! * **depositor-supplied material** — the synonym list (PubChem's own data
//!   model calls it "Depositor-Supplied Synonyms") and every experimental
//!   annotation, each of which keeps its depositor's terms.
//!
//! The adapter therefore never treats "PubChem returned it" as a licence. The
//! depositor synonym list is parsed, classified and reported, but no entry in
//! it is offered for promotion; the only names offered are the two the core
//! layer produces. Each annotation is carried as its own candidate field,
//! tagged with the upstream annotation source and that source's licence note,
//! so a reviewer clears a *source*, not a database.
//!
//! # Deliberate refusals
//!
//! * **No free-text parsing.** An annotation whose value arrives as prose
//!   (`"78.29 °C @760 [mm Hg]"`) is carried verbatim as a string with no unit.
//!   It is never turned into a number here. In this snapshot that is every
//!   annotation from every source except one, and that one is licensed
//!   `CC-BY-NC-4.0`, so no experimental boiling point is promotable at all —
//!   [`PubchemFinding`] says so per source rather than leaving it implied.
//! * **No synonym heuristics on the promotion path.** Distinguishing a trade
//!   name from a common name by shape would be a guess. The classification in
//!   [`classify_synonym`] exists to make the *rejected* list readable; it
//!   cannot promote anything, because no depositor synonym is allowlisted.
//! * **No CAS.** A CAS Registry Number is a proprietary identifier. Those that
//!   appear in the depositor list are separated into their own candidate field
//!   so that the refusal is visible by name rather than by omission.
//!
//! # A note on the SMILES property names
//!
//! BREADTH § BRD-010 says "canonical/isomeric SMILES", which was PUG REST's
//! own vocabulary when the task was written. The service has since renamed the
//! two properties: what it called `CanonicalSMILES` is now `ConnectivitySMILES`
//! and what it called `IsomericSMILES` is now plain `SMILES`. The adapter
//! requests the current names and books them as `connectivity_smiles` and
//! `isomeric_smiles`, so the candidate field says which of the two it is
//! without depending on which spelling the service used that year.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::adapter::{CandidateField, PromotionPolicy, QuarantinedCandidate, RuntimeFieldPolicy};
use crate::provenance::{default_runtime_data_licences, EligibleFieldList};
use crate::schema::Dimension;

/// The adapter id every candidate and the snapshot manifest carry.
pub const PUBCHEM_ADAPTER_ID: &str = "pubchem-v1";

/// The licence the tree already books PubChem's own computed layer under
/// (DATA-007). It is **not** in [`default_runtime_data_licences`]: promoting
/// PubChem core data into a shipped pack is a licence review, not a code
/// change. See [`pubchem_candidate_licences`].
pub const PUBCHEM_CORE_LICENCE: &str = "LicenseRef-PubChem-Public-Domain";

/// The documented unit of PubChem's mass properties. PUG REST returns the
/// numbers without a unit, so the adapter attaches the one the property table
/// documentation fixes rather than inferring anything from the value.
pub const PUBCHEM_MASS_UNIT: &str = "g/mol";

/// The snapshot layout `tools/fetch-pubchem-snapshot.py` writes.
pub const PUBCHEM_SNAPSHOT_SCHEMA: u32 = 1;

// ---------------------------------------------------------------------------
// Snapshot
// ---------------------------------------------------------------------------

/// One pinned PubChem retrieval: every response body the fetcher received,
/// in the order it received them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PubchemSnapshot {
    pub schema: u32,
    pub adapter_id: String,
    pub service: String,
    pub retrieved: String,
    pub synonym_cap: usize,
    pub annotation_heading: String,
    #[serde(default)]
    pub fidelity: String,
    pub resolutions: Vec<NameResolution>,
    pub responses: Vec<SnapshotResponse>,
}

/// What one seed name resolved to on the retrieval date.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NameResolution {
    pub name: String,
    /// Why the seed is in the fixture. Documentation for a reviewer; the
    /// adapter never reads it as data.
    #[serde(default)]
    pub class: String,
    pub cids: Vec<u64>,
}

/// One pinned response body, tagged with the request family that produced it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapshotResponse {
    pub kind: String,
    pub url: String,
    #[serde(default)]
    pub status: u16,
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub cid: Option<u64>,
    #[serde(default)]
    pub heading: Option<String>,
    #[serde(default)]
    pub synonym_total_by_cid: BTreeMap<String, u64>,
    pub body: Value,
}

/// Everything that can stop a snapshot from being read at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PubchemError {
    /// The bytes are not the JSON object this adapter version writes.
    Malformed(String),
    /// A snapshot from a future or past fetcher layout.
    UnsupportedSchema { found: u32, expected: u32 },
    /// The snapshot was taken by a different adapter.
    AdapterMismatch { found: String, expected: String },
}

impl std::fmt::Display for PubchemError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Malformed(detail) => write!(formatter, "unreadable PubChem snapshot: {detail}"),
            Self::UnsupportedSchema { found, expected } => write!(
                formatter,
                "unsupported PubChem snapshot schema {found}; expected {expected}"
            ),
            Self::AdapterMismatch { found, expected } => write!(
                formatter,
                "PubChem snapshot claims adapter {found}; expected {expected}"
            ),
        }
    }
}

impl std::error::Error for PubchemError {}

/// Read a pinned snapshot. Attacker-shaped input: every failure is typed.
pub fn parse_pubchem_snapshot(raw: &[u8]) -> Result<PubchemSnapshot, PubchemError> {
    let snapshot: PubchemSnapshot =
        serde_json::from_slice(raw).map_err(|error| PubchemError::Malformed(error.to_string()))?;
    if snapshot.schema != PUBCHEM_SNAPSHOT_SCHEMA {
        return Err(PubchemError::UnsupportedSchema {
            found: snapshot.schema,
            expected: PUBCHEM_SNAPSHOT_SCHEMA,
        });
    }
    if snapshot.adapter_id != PUBCHEM_ADAPTER_ID {
        return Err(PubchemError::AdapterMismatch {
            found: snapshot.adapter_id,
            expected: PUBCHEM_ADAPTER_ID.to_owned(),
        });
    }
    Ok(snapshot)
}

// ---------------------------------------------------------------------------
// Structure classification
// ---------------------------------------------------------------------------

/// One dot-separated component of a PubChem SMILES, with the formal charge
/// read off its bracket atoms.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SmilesComponent {
    pub smiles: String,
    pub charge: i64,
}

impl SmilesComponent {
    /// PubChem writes water as a bare `O` (and heavy water with explicit
    /// isotope labels, which is a different component and stays one).
    pub fn is_water(&self) -> bool {
        self.charge == 0 && matches!(self.smiles.as_str(), "O" | "[OH2]" | "[H]O[H]")
    }
}

/// What a record's structure actually is, as opposed to what its name
/// suggests. A [`StructureClass::Mixture`] is the case BRD-010 must report
/// rather than quietly import.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "structure", rename_all = "snake_case")]
pub enum StructureClass {
    /// One neutral component.
    Single,
    /// One component carrying a net charge — an ion, not a substance.
    Ion { charge: i64 },
    /// Several components, all charged, summing to neutral.
    Salt { components: usize },
    /// A salt or molecule written with explicit waters of crystallization.
    Hydrate { waters: usize, components: usize },
    /// Several neutral components: an alloy, a mixed acid, a formulation.
    /// PubChem answering a *name* with one of these is the hazard the
    /// acceptance calls out.
    Mixture { components: usize },
    /// The SMILES did not split cleanly. Never guessed at.
    Unparsed { detail: String },
}

/// Split a SMILES on its top-level `.` separators and read each component's
/// formal charge.
///
/// This is a lexer, not a chemistry parser: it tracks bracket and parenthesis
/// depth so a `.` inside `[...]` or `(...)` is not a separator, and reads the
/// `+`/`-` charge suffix of each bracket atom. It never infers bonds, valence
/// or aromaticity, and it never repairs input.
pub fn smiles_components(smiles: &str) -> Result<Vec<SmilesComponent>, String> {
    let trimmed = smiles.trim();
    if trimmed.is_empty() {
        return Err("empty SMILES".to_owned());
    }
    let mut components = Vec::new();
    let mut current = String::new();
    let mut charge = 0i64;
    let mut brackets = 0i32;
    let mut parens = 0i32;
    let mut characters = trimmed.chars().peekable();

    while let Some(character) = characters.next() {
        match character {
            '[' => {
                brackets += 1;
                current.push(character);
            }
            ']' => {
                brackets -= 1;
                if brackets < 0 {
                    return Err(format!("unbalanced ']' in {smiles:?}"));
                }
                current.push(character);
            }
            '(' => {
                parens += 1;
                current.push(character);
            }
            ')' => {
                parens -= 1;
                if parens < 0 {
                    return Err(format!("unbalanced ')' in {smiles:?}"));
                }
                current.push(character);
            }
            '+' | '-' if brackets > 0 => {
                current.push(character);
                let sign = if character == '+' { 1 } else { -1 };
                // `[Cu+2]` and the older `[Cu++]` both occur upstream.
                let mut magnitude = 0i64;
                while let Some(digit) = characters.peek().and_then(|c| c.to_digit(10)) {
                    magnitude = magnitude
                        .saturating_mul(10)
                        .saturating_add(i64::from(digit));
                    current.push(characters.next().expect("peeked"));
                }
                if magnitude == 0 {
                    magnitude = 1;
                    while characters.peek() == Some(&character) {
                        magnitude += 1;
                        current.push(characters.next().expect("peeked"));
                    }
                }
                charge = charge.saturating_add(sign * magnitude);
            }
            '.' if brackets == 0 && parens == 0 => {
                if current.is_empty() {
                    return Err(format!("empty component in {smiles:?}"));
                }
                components.push(SmilesComponent {
                    smiles: std::mem::take(&mut current),
                    charge,
                });
                charge = 0;
            }
            other => current.push(other),
        }
    }

    if brackets != 0 || parens != 0 {
        return Err(format!("unbalanced brackets in {smiles:?}"));
    }
    if current.is_empty() {
        return Err(format!("trailing separator in {smiles:?}"));
    }
    components.push(SmilesComponent {
        smiles: current,
        charge,
    });
    Ok(components)
}

/// Classify a record's structure from its SMILES.
pub fn classify_smiles(smiles: &str) -> StructureClass {
    let components = match smiles_components(smiles) {
        Ok(components) => components,
        Err(detail) => return StructureClass::Unparsed { detail },
    };
    let total: i64 = components.iter().map(|component| component.charge).sum();
    let waters = components
        .iter()
        .filter(|component| component.is_water())
        .count();
    let solutes: Vec<&SmilesComponent> = components
        .iter()
        .filter(|component| !component.is_water())
        .collect();

    if components.len() == 1 {
        return if total == 0 {
            StructureClass::Single
        } else {
            StructureClass::Ion { charge: total }
        };
    }
    if total != 0 {
        return StructureClass::Ion { charge: total };
    }
    let all_solutes_charged =
        !solutes.is_empty() && solutes.iter().all(|component| component.charge != 0);
    if waters > 0 && (all_solutes_charged || solutes.len() == 1) {
        return StructureClass::Hydrate {
            waters,
            components: components.len(),
        };
    }
    if waters == 0 && all_solutes_charged {
        return StructureClass::Salt {
            components: components.len(),
        };
    }
    StructureClass::Mixture {
        components: components.len(),
    }
}

// ---------------------------------------------------------------------------
// Synonyms
// ---------------------------------------------------------------------------

/// How one entry of PubChem's depositor-supplied synonym list reads.
///
/// Every variant is refused for promotion — the list is depositor material as
/// a whole. The classification exists so the refusal report distinguishes "a
/// proprietary registry number" from "somebody's product name", which is what
/// a reviewer needs in order to decide whether a *different* name source is
/// worth acquiring.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "synonym_class", rename_all = "snake_case")]
pub enum SynonymClass {
    /// A CAS Registry Number whose check digit validates. Proprietary.
    CasRegistryNumber,
    /// An identifier from some other registry, or a structure string.
    RegistryIdentifier { scheme: String },
    /// A name a depositor supplied. Could be a common name, could be a trade
    /// name; telling them apart by shape would be a guess, so the adapter
    /// does not try and promotes neither.
    DepositorSuppliedName,
}

/// Whether a CAS-shaped string is really a CAS Registry Number.
///
/// The last digit is a check digit: the sum of each preceding digit times its
/// position from the right, modulo 10.
fn cas_check_digit_valid(text: &str) -> bool {
    let mut parts = text.split('-');
    let (Some(first), Some(second), Some(third), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return false;
    };
    if !(2..=7).contains(&first.len()) || second.len() != 2 || third.len() != 1 {
        return false;
    }
    let all_digits = first
        .chars()
        .chain(second.chars())
        .chain(third.chars())
        .all(|character| character.is_ascii_digit());
    if !all_digits {
        return false;
    }
    if first.starts_with('0') {
        return false;
    }
    let digits: Vec<u32> = first
        .chars()
        .chain(second.chars())
        .filter_map(|c| c.to_digit(10))
        .collect();
    let Some(check) = third.chars().next().and_then(|c| c.to_digit(10)) else {
        return false;
    };
    let sum: u32 = digits
        .iter()
        .rev()
        .enumerate()
        .map(|(index, digit)| (u32::try_from(index).unwrap_or(u32::MAX).saturating_add(1)) * digit)
        .sum();
    sum % 10 == check
}

/// Registry prefixes that mark a synonym as somebody's identifier rather than
/// a name. This list only has to be good enough to make the *rejected* list
/// readable — nothing here is on a promotion path, so a miss costs report
/// quality and never data.
const REGISTRY_PREFIXES: &[&str] = &[
    "CHEBI:",
    "CHEMBL",
    "SCHEMBL",
    "DTXSID",
    "DTXCID",
    "UNII",
    "MFCD",
    "EINECS",
    "NSC",
    "HSDB",
    "AKOS",
    "CCRIS",
    "BRN ",
    "EC ",
    "UN ",
    "CAS-",
    "NCGC",
    "ZINC",
    "BDBM",
    "FT-",
    "CS-",
    "AI3-",
    "WLN:",
    "EPA PESTICIDE",
];

fn registry_scheme(text: &str) -> Option<String> {
    let upper = text.to_ascii_uppercase();
    // Structure strings are identifiers, not names.
    if upper.starts_with("INCHI=") {
        return Some("inchi".to_owned());
    }
    if is_inchikey(text) {
        return Some("inchikey".to_owned());
    }
    // A Wikidata QID, but not every word beginning in Q.
    if let Some(digits) = upper.strip_prefix('Q') {
        if !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()) {
            return Some("wikidata".to_owned());
        }
    }
    if let Some(prefix) = REGISTRY_PREFIXES
        .iter()
        .find(|prefix| upper.starts_with(**prefix))
    {
        return Some(
            prefix
                .trim_end_matches([':', ' ', '-'])
                .to_ascii_lowercase(),
        );
    }
    // A bare EC number, `231-598-3`.
    let segments: Vec<&str> = text.split('-').collect();
    if segments.len() == 3
        && segments[0].len() == 3
        && segments[1].len() == 3
        && segments[2].len() == 1
        && segments
            .iter()
            .all(|segment| segment.chars().all(|c| c.is_ascii_digit()))
    {
        return Some("ec-number".to_owned());
    }
    None
}

fn is_inchikey(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() == 27
        && bytes[14] == b'-'
        && bytes[25] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 14 || index == 25 || byte.is_ascii_uppercase())
}

/// Read one depositor-supplied synonym.
pub fn classify_synonym(synonym: &str) -> SynonymClass {
    let text = synonym.trim();
    if cas_check_digit_valid(text) {
        return SynonymClass::CasRegistryNumber;
    }
    if let Some(scheme) = registry_scheme(text) {
        return SynonymClass::RegistryIdentifier { scheme };
    }
    SynonymClass::DepositorSuppliedName
}

// ---------------------------------------------------------------------------
// Findings
// ---------------------------------------------------------------------------

/// Something about the snapshot a reviewer has to see. A finding never blocks
/// the pipeline by itself — the promotion policy and
/// [`lint_promotion`](crate::lint_promotion) do that — it is the part of the
/// import that would otherwise be lost as "nothing to report".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "finding", rename_all = "snake_case")]
pub enum PubchemFinding {
    /// A seed name PubChem answered with no record at all.
    NameResolvedToNoRecord { name: String },
    /// A seed name PubChem answered with several records.
    NameResolvedToMultipleRecords { name: String, cids: Vec<u64> },
    /// Several seed names landed on one record. The names are not synonyms by
    /// virtue of that: `vinegar` and `acetic acid` are not the same substance.
    SharedNameResolution { cid: u64, names: Vec<String> },
    /// The record is a multi-component neutral system — an alloy, a mixed
    /// acid, a formulation. Reported, never silently taken as a substance.
    MixtureRecord {
        cid: u64,
        title: String,
        components: usize,
        smiles: String,
        resolved_from: Vec<String>,
    },
    /// The record is a bare ion or an unbalanced fragment set.
    IonRecord {
        cid: u64,
        title: String,
        charge: i64,
    },
    /// The SMILES did not lex. No structure claim is made about the record.
    StructureNotParsed {
        cid: u64,
        smiles: String,
        detail: String,
    },
    /// PubChem's declared `Charge` and the charge summed off the SMILES
    /// disagree. Both are kept; neither is chosen.
    DeclaredChargeDisagreesWithStructure {
        cid: u64,
        declared: i64,
        from_smiles: i64,
    },
    /// The core layer produced no IUPAC name for this record.
    MissingIupacName { cid: u64, title: String },
    /// A property the adapter asked for was absent from the response.
    MissingProperty { cid: u64, property: String },
    /// An annotation arrived from a source whose terms are not cleared for
    /// this lane. The upstream licence note is carried verbatim so the
    /// clearing decision can be made from the evidence.
    AnnotationSourceNotCleared {
        cid: u64,
        heading: String,
        source_name: String,
        licence_note: Option<String>,
        licence_url: Option<String>,
        structured: bool,
    },
    /// One source gave several values for one record and heading. The first
    /// in document order is carried; the rest are named here rather than
    /// averaged, ranked or dropped in silence.
    AnnotationEntriesElided {
        cid: u64,
        heading: String,
        source_name: String,
        kept: usize,
        total: usize,
    },
    /// The pinned snapshot holds fewer synonyms than PubChem returned. See
    /// `tools/fetch-pubchem-snapshot.py` for why, and the snapshot for the
    /// full body's checksum.
    SynonymListTruncated { cid: u64, kept: u64, total: u64 },
}

/// One synonym that more than one record claims. The acceptance calls for
/// these to exist in the fixture and to be visible rather than merged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SynonymConflict {
    /// Case-folded, since `Glucose` and `glucose` are the same claim.
    pub synonym: String,
    pub cids: Vec<u64>,
}

// ---------------------------------------------------------------------------
// Import
// ---------------------------------------------------------------------------

/// What one record looks like after parsing, before any policy is applied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PubchemRecordSummary {
    pub cid: u64,
    pub title: String,
    pub structure: StructureClass,
    pub resolved_from: Vec<String>,
    pub inchi: Option<String>,
    pub inchikey: Option<String>,
    pub isomeric_smiles: Option<String>,
    pub depositor_synonyms_kept: usize,
    pub cas_registry_numbers: usize,
    pub registry_identifiers: usize,
    pub annotation_entries: usize,
}

/// The adapter's whole output: candidates for review, plus everything a
/// reviewer would otherwise have to rediscover by hand.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PubchemImport {
    pub adapter_id: String,
    pub source_revision: String,
    pub retrieved: String,
    pub records: Vec<PubchemRecordSummary>,
    pub candidates: Vec<QuarantinedCandidate>,
    pub findings: Vec<PubchemFinding>,
    pub synonym_conflicts: Vec<SynonymConflict>,
}

fn string_field(record: &Value, key: &str) -> Option<String> {
    record.get(key)?.as_str().map(str::to_owned)
}

fn number_field(record: &Value, key: &str) -> Option<f64> {
    match record.get(key)? {
        Value::Number(number) => number.as_f64(),
        // PUG REST returns the masses as strings.
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
}

fn integer_field(record: &Value, key: &str) -> Option<i64> {
    match record.get(key)? {
        Value::Number(number) => number.as_i64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
}

fn core(value: Value, source_field: impl Into<String>) -> CandidateField {
    CandidateField::new(value, source_field, PUBCHEM_CORE_LICENCE)
}

fn slug(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last_underscore = true;
    for character in text.chars() {
        if character.is_ascii_alphanumeric() {
            out.push(character.to_ascii_lowercase());
            last_underscore = false;
        } else if !last_underscore {
            out.push('_');
            last_underscore = true;
        }
    }
    while out.ends_with('_') {
        out.pop();
    }
    if out.is_empty() {
        "unnamed".to_owned()
    } else {
        out
    }
}

/// Turn a pinned snapshot into quarantine candidates and a review report.
pub fn pubchem_import(snapshot: &PubchemSnapshot) -> PubchemImport {
    let mut findings = Vec::new();

    // 1. Which record did each seed name land on?
    let mut resolved_from: BTreeMap<u64, Vec<String>> = BTreeMap::new();
    for resolution in &snapshot.resolutions {
        match resolution.cids.len() {
            0 => findings.push(PubchemFinding::NameResolvedToNoRecord {
                name: resolution.name.clone(),
            }),
            1 => {}
            _ => findings.push(PubchemFinding::NameResolvedToMultipleRecords {
                name: resolution.name.clone(),
                cids: resolution.cids.clone(),
            }),
        }
        for cid in &resolution.cids {
            resolved_from
                .entry(*cid)
                .or_default()
                .push(resolution.name.clone());
        }
    }
    for (cid, names) in &resolved_from {
        if names.len() > 1 {
            findings.push(PubchemFinding::SharedNameResolution {
                cid: *cid,
                names: names.clone(),
            });
        }
    }

    // 2. Index the pinned bodies.
    let mut properties: BTreeMap<u64, Value> = BTreeMap::new();
    let mut synonyms: BTreeMap<u64, Vec<String>> = BTreeMap::new();
    let mut synonym_totals: BTreeMap<u64, u64> = BTreeMap::new();
    let mut annotations: BTreeMap<u64, Vec<AnnotationEntry>> = BTreeMap::new();

    for response in &snapshot.responses {
        match response.kind.as_str() {
            "property_table" => {
                let rows = response
                    .body
                    .pointer("/PropertyTable/Properties")
                    .and_then(Value::as_array);
                for row in rows.into_iter().flatten() {
                    if let Some(cid) = row.get("CID").and_then(Value::as_u64) {
                        properties.insert(cid, row.clone());
                    }
                }
            }
            "synonyms" => {
                let rows = response
                    .body
                    .pointer("/InformationList/Information")
                    .and_then(Value::as_array);
                for row in rows.into_iter().flatten() {
                    let Some(cid) = row.get("CID").and_then(Value::as_u64) else {
                        continue;
                    };
                    let kept: Vec<String> = row
                        .get("Synonym")
                        .and_then(Value::as_array)
                        .map(|list| {
                            list.iter()
                                .filter_map(Value::as_str)
                                .map(str::to_owned)
                                .collect()
                        })
                        .unwrap_or_default();
                    let total = response
                        .synonym_total_by_cid
                        .get(&cid.to_string())
                        .copied()
                        .unwrap_or(kept.len() as u64);
                    if total > kept.len() as u64 {
                        findings.push(PubchemFinding::SynonymListTruncated {
                            cid,
                            kept: kept.len() as u64,
                            total,
                        });
                    }
                    synonym_totals.insert(cid, total);
                    synonyms.insert(cid, kept);
                }
            }
            "pug_view" => {
                let Some(cid) = response.cid else { continue };
                let heading = response.heading.clone().unwrap_or_default();
                annotations
                    .entry(cid)
                    .or_default()
                    .extend(read_annotations(&response.body, &heading));
            }
            _ => {}
        }
    }

    // 3. Build a candidate per record.
    let mut records = Vec::new();
    let mut candidates = Vec::new();
    let mut synonym_owners: BTreeMap<String, BTreeSet<u64>> = BTreeMap::new();

    for (cid, row) in &properties {
        let cid = *cid;
        let title = string_field(row, "Title").unwrap_or_default();
        let names = resolved_from.get(&cid).cloned().unwrap_or_default();
        let mut fields: BTreeMap<String, CandidateField> = BTreeMap::new();
        let source_record = format!("PubChem CID {cid}");

        fields.insert(
            "cid".into(),
            core(Value::String(cid.to_string()), "PropertyTable.CID"),
        );

        // --- core identity ---
        for (field, key) in [
            ("molecular_formula", "MolecularFormula"),
            ("connectivity_smiles", "ConnectivitySMILES"),
            ("isomeric_smiles", "SMILES"),
            ("standard_inchi", "InChI"),
            ("standard_inchikey", "InChIKey"),
            ("iupac_name", "IUPACName"),
            ("pubchem_title", "Title"),
        ] {
            match string_field(row, key) {
                Some(value) => {
                    fields.insert(
                        field.into(),
                        core(Value::String(value), format!("PropertyTable.{key}")),
                    );
                }
                None => {
                    if key == "IUPACName" {
                        findings.push(PubchemFinding::MissingIupacName {
                            cid,
                            title: title.clone(),
                        });
                    }
                    findings.push(PubchemFinding::MissingProperty {
                        cid,
                        property: key.to_owned(),
                    });
                }
            }
        }

        // The two names that come from PubChem's own layer rather than a
        // depositor. This is the whole promotable name set.
        let neutral: Vec<Value> = ["Title", "IUPACName"]
            .iter()
            .filter_map(|key| string_field(row, key))
            .map(Value::String)
            .collect();
        if !neutral.is_empty() {
            fields.insert(
                "depositor_neutral_names".into(),
                core(
                    Value::Array(neutral),
                    "PropertyTable.Title + PropertyTable.IUPACName",
                ),
            );
        }

        // --- masses, with the unit the property table documents ---
        for (field, key) in [
            ("molar_mass", "MolecularWeight"),
            ("monoisotopic_mass", "MonoisotopicMass"),
            ("exact_mass", "ExactMass"),
        ] {
            match number_field(row, key).and_then(serde_json::Number::from_f64) {
                Some(number) => {
                    fields.insert(
                        field.into(),
                        core(Value::Number(number), format!("PropertyTable.{key}"))
                            .with_unit(PUBCHEM_MASS_UNIT),
                    );
                }
                None => findings.push(PubchemFinding::MissingProperty {
                    cid,
                    property: key.to_owned(),
                }),
            }
        }

        // --- formal charge, and the structure it is checked against ---
        let declared_charge = integer_field(row, "Charge");
        if let Some(charge) = declared_charge {
            fields.insert(
                "formal_charge".into(),
                core(Value::Number(charge.into()), "PropertyTable.Charge"),
            );
        } else {
            findings.push(PubchemFinding::MissingProperty {
                cid,
                property: "Charge".to_owned(),
            });
        }

        let isomeric = string_field(row, "SMILES").unwrap_or_default();
        let structure = classify_smiles(&isomeric);
        match &structure {
            StructureClass::Mixture { components } => {
                findings.push(PubchemFinding::MixtureRecord {
                    cid,
                    title: title.clone(),
                    components: *components,
                    smiles: isomeric.clone(),
                    resolved_from: names.clone(),
                });
            }
            StructureClass::Ion { charge } => findings.push(PubchemFinding::IonRecord {
                cid,
                title: title.clone(),
                charge: *charge,
            }),
            StructureClass::Unparsed { detail } => {
                findings.push(PubchemFinding::StructureNotParsed {
                    cid,
                    smiles: isomeric.clone(),
                    detail: detail.clone(),
                });
            }
            StructureClass::Single
            | StructureClass::Salt { .. }
            | StructureClass::Hydrate { .. } => {}
        }
        if let (Some(declared), Ok(components)) = (declared_charge, smiles_components(&isomeric)) {
            let summed: i64 = components.iter().map(|component| component.charge).sum();
            if summed != declared {
                findings.push(PubchemFinding::DeclaredChargeDisagreesWithStructure {
                    cid,
                    declared,
                    from_smiles: summed,
                });
            }
        }
        fields.insert(
            "structure_class".into(),
            core(
                serde_json::to_value(&structure).unwrap_or(Value::Null),
                "derived from PropertyTable.SMILES",
            ),
        );

        // --- non-allowlisted computed descriptors, carried so their refusal
        //     is visible by name (the DATA-007 `patent_count` discipline) ---
        for (field, key) in [
            ("xlogp", "XLogP"),
            ("tpsa", "TPSA"),
            ("complexity", "Complexity"),
            ("hbond_donor_count", "HBondDonorCount"),
            ("hbond_acceptor_count", "HBondAcceptorCount"),
            ("rotatable_bond_count", "RotatableBondCount"),
            ("heavy_atom_count", "HeavyAtomCount"),
        ] {
            if let Some(value) = row.get(key) {
                fields.insert(
                    field.into(),
                    core(value.clone(), format!("PropertyTable.{key}")),
                );
            }
        }

        // --- depositor synonyms: classified, reported, never promotable ---
        let kept = synonyms.get(&cid).cloned().unwrap_or_default();
        let mut cas = Vec::new();
        let mut registry = Vec::new();
        let mut depositor_names = Vec::new();
        for synonym in &kept {
            match classify_synonym(synonym) {
                SynonymClass::CasRegistryNumber => cas.push(Value::String(synonym.clone())),
                SynonymClass::RegistryIdentifier { .. } => {
                    registry.push(Value::String(synonym.clone()));
                }
                SynonymClass::DepositorSuppliedName => {
                    depositor_names.push(Value::String(synonym.clone()));
                    synonym_owners
                        .entry(synonym.to_lowercase())
                        .or_default()
                        .insert(cid);
                }
            }
        }
        let cas_count = cas.len();
        let registry_count = registry.len();
        let depositor_count = depositor_names.len();
        if !cas.is_empty() {
            fields.insert(
                "cas_registry_numbers".into(),
                CandidateField::new(
                    Value::Array(cas),
                    "synonyms/JSON InformationList.Information.Synonym",
                    "LicenseRef-CAS-Proprietary",
                ),
            );
        }
        if !registry.is_empty() {
            fields.insert(
                "registry_identifiers".into(),
                CandidateField::new(
                    Value::Array(registry),
                    "synonyms/JSON InformationList.Information.Synonym",
                    "LicenseRef-PubChem-Depositor-Supplied",
                ),
            );
        }
        if !depositor_names.is_empty() {
            fields.insert(
                "depositor_supplied_synonyms".into(),
                CandidateField::new(
                    Value::Array(depositor_names),
                    "synonyms/JSON InformationList.Information.Synonym",
                    "LicenseRef-PubChem-Depositor-Supplied",
                ),
            );
        }

        // --- annotations, one candidate field per upstream source ---
        let entries = annotations.get(&cid).cloned().unwrap_or_default();
        let mut by_source: BTreeMap<String, Vec<&AnnotationEntry>> = BTreeMap::new();
        for entry in &entries {
            by_source
                .entry(entry.source_name.clone())
                .or_default()
                .push(entry);
        }
        for (source_name, group) in &by_source {
            let first = group[0];
            if group.len() > 1 {
                findings.push(PubchemFinding::AnnotationEntriesElided {
                    cid,
                    heading: first.heading.clone(),
                    source_name: source_name.clone(),
                    kept: 1,
                    total: group.len(),
                });
            }
            findings.push(PubchemFinding::AnnotationSourceNotCleared {
                cid,
                heading: first.heading.clone(),
                source_name: source_name.clone(),
                licence_note: first.licence_note.clone(),
                licence_url: first.licence_url.clone(),
                structured: first.number.is_some(),
            });
            let field_name = format!("{}__{}", slug(&first.heading), slug(source_name));
            let licence = annotation_licence(first);
            let candidate = match (first.number, &first.unit) {
                (Some(number), Some(unit)) => serde_json::Number::from_f64(number)
                    .map(|number| {
                        CandidateField::new(
                            Value::Number(number),
                            first.source_field(cid),
                            licence.clone(),
                        )
                        .with_unit(unit)
                    })
                    .unwrap_or_else(|| {
                        CandidateField::new(
                            Value::String(first.text.clone()),
                            first.source_field(cid),
                            licence.clone(),
                        )
                    }),
                // Prose. Carried verbatim, never parsed into a number.
                _ => CandidateField::new(
                    Value::String(first.text.clone()),
                    first.source_field(cid),
                    licence.clone(),
                ),
            };
            fields.insert(field_name, candidate);
        }

        records.push(PubchemRecordSummary {
            cid,
            title: title.clone(),
            structure,
            resolved_from: names,
            inchi: string_field(row, "InChI"),
            inchikey: string_field(row, "InChIKey"),
            isomeric_smiles: string_field(row, "SMILES"),
            depositor_synonyms_kept: depositor_count,
            cas_registry_numbers: cas_count,
            registry_identifiers: registry_count,
            annotation_entries: entries.len(),
        });

        candidates.push(QuarantinedCandidate {
            adapter_id: PUBCHEM_ADAPTER_ID.to_owned(),
            source_record_id: source_record,
            external_record_id: format!("CID{cid}"),
            identity_key: string_field(row, "InChIKey"),
            fields,
        });
    }

    let synonym_conflicts = synonym_owners
        .into_iter()
        .filter(|(_, cids)| cids.len() > 1)
        .map(|(synonym, cids)| SynonymConflict {
            synonym,
            cids: cids.into_iter().collect(),
        })
        .collect();

    findings.sort_by_key(|finding| serde_json::to_string(finding).unwrap_or_default());
    candidates.sort_by(|a, b| a.external_record_id.cmp(&b.external_record_id));

    PubchemImport {
        adapter_id: PUBCHEM_ADAPTER_ID.to_owned(),
        source_revision: format!("pug-rest-retrieved-{}", snapshot.retrieved),
        retrieved: snapshot.retrieved.clone(),
        records,
        candidates,
        findings,
        synonym_conflicts,
    }
}

/// One PUG View annotation, flattened out of the section tree.
#[derive(Debug, Clone, PartialEq)]
struct AnnotationEntry {
    heading: String,
    source_name: String,
    licence_note: Option<String>,
    licence_url: Option<String>,
    text: String,
    number: Option<f64>,
    unit: Option<String>,
}

impl AnnotationEntry {
    fn source_field(&self, cid: u64) -> String {
        format!(
            "pug_view/compound/{cid} {} ({})",
            self.heading, self.source_name
        )
    }
}

/// The licence an annotation carries.
///
/// Only a source that states an unambiguous SPDX-nameable licence gets one.
/// Everything else keeps a `LicenseRef-*` naming the source, so the refusal
/// reads as "this source is not cleared" and not as "this data has no
/// licence".
fn annotation_licence(entry: &AnnotationEntry) -> String {
    match entry.licence_note.as_deref().map(str::trim) {
        Some("Creative Commons CC BY 4.0") => "CC-BY-4.0".to_owned(),
        _ => format!("LicenseRef-PubChem-Annotation-{}", slug(&entry.source_name)),
    }
}

fn read_annotations(body: &Value, heading: &str) -> Vec<AnnotationEntry> {
    let Some(record) = body.get("Record") else {
        return Vec::new();
    };
    let mut references: BTreeMap<u64, (&Value, String)> = BTreeMap::new();
    if let Some(list) = record.get("Reference").and_then(Value::as_array) {
        for reference in list {
            let Some(number) = reference.get("ReferenceNumber").and_then(Value::as_u64) else {
                continue;
            };
            let source = reference
                .get("SourceName")
                .and_then(Value::as_str)
                .unwrap_or("<unnamed source>")
                .to_owned();
            references.insert(number, (reference, source));
        }
    }

    let mut entries = Vec::new();
    let mut stack = vec![record];
    let mut sections = Vec::new();
    while let Some(node) = stack.pop() {
        if let Some(children) = node.get("Section").and_then(Value::as_array) {
            for child in children.iter().rev() {
                stack.push(child);
            }
        }
        sections.push(node);
    }
    for section in sections {
        let section_heading = section
            .get("TOCHeading")
            .and_then(Value::as_str)
            .unwrap_or(heading);
        let Some(information) = section.get("Information").and_then(Value::as_array) else {
            continue;
        };
        for item in information {
            let reference_number = item.get("ReferenceNumber").and_then(Value::as_u64);
            let (reference, source_name) = reference_number
                .and_then(|number| references.get(&number))
                .map(|(reference, source)| (Some(*reference), source.clone()))
                .unwrap_or((None, "<unattributed>".to_owned()));
            let value = item.get("Value");
            let number = value
                .and_then(|value| value.pointer("/Number/0"))
                .and_then(Value::as_f64);
            let unit = value
                .and_then(|value| value.get("Unit"))
                .and_then(Value::as_str)
                .map(str::to_owned);
            let text = value
                .and_then(|value| value.pointer("/StringWithMarkup/0/String"))
                .and_then(Value::as_str)
                .map(str::to_owned)
                .or_else(|| match (number, &unit) {
                    (Some(number), Some(unit)) => Some(format!("{number} {unit}")),
                    (Some(number), None) => Some(number.to_string()),
                    _ => None,
                })
                .unwrap_or_default();
            if text.is_empty() && number.is_none() {
                continue;
            }
            entries.push(AnnotationEntry {
                heading: section_heading.to_owned(),
                source_name,
                licence_note: reference
                    .and_then(|reference| reference.get("LicenseNote"))
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                licence_url: reference
                    .and_then(|reference| reference.get("LicenseURL"))
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                text,
                number,
                unit,
            });
        }
    }
    entries
}

// ---------------------------------------------------------------------------
// Policy
// ---------------------------------------------------------------------------

/// The reviewed field allowlist for BRD-010.
///
/// Only PubChem's own computed/curated core is here. Read the absences as
/// decisions:
///
/// * `exact_mass` and the computed descriptors (`xlogp`, `tpsa`,
///   `complexity`, the counts) are database-model outputs, not measurements
///   the app can stand behind — the same call DATA-007 made about
///   `patent_count`.
/// * `cas_registry_numbers` is a proprietary identifier scheme.
/// * `depositor_supplied_synonyms` and `registry_identifiers` are depositor
///   material; PubChem does not license them onward.
/// * **No annotation field is allowlisted.** In the pinned snapshot, every
///   depositor that states a runtime-lane licence delivers its boiling point
///   as prose, and the one depositor that delivers a structured quantity
///   licenses it `CC-BY-NC-4.0`. So no experimental physical property from
///   PubChem is promotable today, and the import says so per source rather
///   than shipping a number whose terms have not been cleared.
pub fn pubchem_promotion_policy() -> PromotionPolicy {
    let licences = [PUBCHEM_CORE_LICENCE];
    let mut fields = BTreeMap::new();
    for (source_field, target_field) in [
        ("cid", "identifiers.pubchem_cid"),
        ("molecular_formula", "formula"),
        ("connectivity_smiles", "smiles_connectivity"),
        ("isomeric_smiles", "smiles_isomeric"),
        ("standard_inchi", "inchi"),
        ("standard_inchikey", "canonical_key"),
        ("iupac_name", "iupac_name"),
        ("pubchem_title", "display_name"),
        ("depositor_neutral_names", "synonyms"),
        ("formal_charge", "formal_charge"),
        ("structure_class", "structure_class"),
    ] {
        fields.insert(
            source_field.to_owned(),
            RuntimeFieldPolicy::new(target_field, licences),
        );
    }
    for (source_field, target_field) in [
        ("molar_mass", "molar_mass"),
        ("monoisotopic_mass", "monoisotopic_mass"),
    ] {
        fields.insert(
            source_field.to_owned(),
            RuntimeFieldPolicy::new(target_field, licences).with_dimension(Dimension::MolarMass),
        );
    }
    PromotionPolicy { fields }
}

/// The licences BRD-010's dry run admits.
///
/// This is [`default_runtime_data_licences`] plus [`PUBCHEM_CORE_LICENCE`],
/// and the difference is deliberate and narrow: BRD-010 produces **candidates**,
/// and a candidate lane has to be able to hold PubChem's core layer in order
/// for a reviewer to look at it at all. Adding `LicenseRef-PubChem-Public-Domain`
/// to the *runtime* set in `provenance.rs` is a licence decision about shipping,
/// and this task does not make it: nothing here promotes into
/// `registry-source-v1.json`.
pub fn pubchem_candidate_licences() -> BTreeSet<String> {
    let mut licences = default_runtime_data_licences();
    licences.insert(PUBCHEM_CORE_LICENCE.to_owned());
    licences
}

/// The fields of each candidate that the allowlist would actually promote.
///
/// A reviewer signs off an [`EligibleFieldList`] by hand in general; this
/// builds the mechanical maximum — every allowlisted field the record really
/// carries — so the dry run exercises the full width of the policy rather
/// than a convenient subset.
pub fn pubchem_eligible_fields(
    candidates: &[QuarantinedCandidate],
    policy: &PromotionPolicy,
) -> Vec<EligibleFieldList> {
    candidates
        .iter()
        .map(|candidate| EligibleFieldList {
            adapter_id: candidate.adapter_id.clone(),
            external_record_id: candidate.external_record_id.clone(),
            fields: candidate
                .fields
                .keys()
                .filter(|field| policy.fields.contains_key(*field))
                .cloned()
                .collect(),
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Identity cross-check
// ---------------------------------------------------------------------------

/// What happened when the record's Standard InChIKey was recomputed from its
/// structure by the official IUPAC library.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum IdentityOutcome {
    /// The recomputed key equals the one PubChem published.
    Agrees,
    /// It does not. This is never resolved here: the record keeps both keys
    /// and the disagreement travels to the review report.
    Conflicts { recomputed: String },
    /// No key could be recomputed — the structure is outside what the
    /// toolchain reads (bare metals, isotopic labels, multi-component
    /// systems). Not a conflict, and not silently an agreement either.
    NotRecomputed { detail: String },
    /// The snapshot itself has no InChIKey or no structure to check against.
    NoSnapshotIdentity,
}

/// One record's identity check, along both independent routes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityCrossCheck {
    pub external_record_id: String,
    pub cid: u64,
    pub smiles: String,
    pub snapshot_inchi: String,
    pub snapshot_inchikey: String,
    /// The official library's key for PubChem's **own** published InChI
    /// string. A conflict here is a claim about the upstream record: its
    /// published key does not hash from its published structure.
    pub from_published_inchi: IdentityOutcome,
    /// The official library's key for the record's structure, taken the long
    /// way round: SMILES → molfile → InChI → key. A conflict here can equally
    /// be a limitation of that bridge, so it is reported separately and never
    /// conflated with the check above.
    pub from_structure: IdentityOutcome,
}

/// How one route scored across the fixture.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityCheckSummary {
    pub agreements: usize,
    pub conflicts: usize,
    pub not_recomputed: usize,
    pub no_snapshot_identity: usize,
}

impl IdentityCheckSummary {
    fn tally(outcomes: impl Iterator<Item = IdentityOutcome>) -> Self {
        let mut summary = Self::default();
        for outcome in outcomes {
            match outcome {
                IdentityOutcome::Agrees => summary.agreements += 1,
                IdentityOutcome::Conflicts { .. } => summary.conflicts += 1,
                IdentityOutcome::NotRecomputed { .. } => summary.not_recomputed += 1,
                IdentityOutcome::NoSnapshotIdentity => summary.no_snapshot_identity += 1,
            }
        }
        summary
    }
}

/// The whole fixture's identity check, in a shape that can be checked in and
/// diffed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityCrossCheckReport {
    pub adapter_id: String,
    pub checked: usize,
    /// The record's own published InChI, re-keyed. This is the check that
    /// speaks about PubChem.
    pub from_published_inchi: IdentityCheckSummary,
    /// The record's structure, round-tripped. This check also exercises our
    /// own SMILES bridge, so a conflict is not by itself an upstream fault.
    pub from_structure: IdentityCheckSummary,
    pub records: Vec<IdentityCrossCheck>,
}

impl IdentityCrossCheckReport {
    /// Every disagreement, as BRD-003 [`IdentityConflict`](crate::IdentityConflict)
    /// rows, so they land in the same review report as every other identity
    /// disagreement instead of in a PubChem-shaped side channel.
    ///
    /// Both routes are reported. `differing_fields` names which one spoke, so
    /// a reviewer can tell "PubChem's record disagrees with itself" from "our
    /// structure bridge did not reproduce it" without reading the code.
    pub fn identity_conflicts(&self) -> Vec<crate::IdentityConflict> {
        let mut conflicts = Vec::new();
        for record in &self.records {
            for (route, outcome) in [
                (
                    "standard_inchikey/from_published_inchi",
                    &record.from_published_inchi,
                ),
                ("standard_inchikey/from_structure", &record.from_structure),
            ] {
                if let IdentityOutcome::Conflicts { recomputed } = outcome {
                    conflicts.push(crate::IdentityConflict {
                        identity_key: record.snapshot_inchikey.clone(),
                        records: vec![
                            record.external_record_id.clone(),
                            format!("official-inchi:{recomputed}"),
                        ],
                        differing_fields: vec![route.to_owned()],
                    });
                }
            }
        }
        conflicts
    }

    /// Conflicts where the recomputed key shares the record's skeleton block —
    /// the same connectivity, a different stereo/isotope layer. Triage only:
    /// it explains a conflict, it never excuses one.
    pub fn skeleton_preserving_conflicts(&self) -> usize {
        self.records
            .iter()
            .filter(|record| match &record.from_structure {
                IdentityOutcome::Conflicts { recomputed } => {
                    skeleton(recomputed) == skeleton(&record.snapshot_inchikey)
                }
                _ => false,
            })
            .count()
    }
}

/// The first block of an InChIKey: the connectivity hash.
fn skeleton(key: &str) -> &str {
    key.split('-').next().unwrap_or(key)
}

/// Recompute every record's Standard InChIKey along both routes and report
/// agreement per record.
///
/// The two recomputations are supplied by the caller because the official
/// IUPAC library is a C dependency that `kerotakis-data` deliberately does not
/// take; `kerotakis-org`'s `native-inchi` feature provides the real ones.
///
/// * `rekey_published_inchi` hashes the record's own published Standard InChI
///   string. Nothing of ours stands between the record and the answer, so a
///   conflict here is a statement about the upstream record.
/// * `recompute_from_structure` goes SMILES → molfile → InChI → key. It is the
///   stronger check and also the more fragile one, because it exercises our
///   own bridge.
///
/// A recomputation that fails is [`IdentityOutcome::NotRecomputed`], never an
/// assumed agreement. Nothing is ever resolved here.
pub fn cross_check_identity(
    import: &PubchemImport,
    mut recompute_from_structure: impl FnMut(&str) -> Result<String, String>,
    mut rekey_published_inchi: impl FnMut(&str) -> Result<String, String>,
) -> IdentityCrossCheckReport {
    let verdict = |result: Result<String, String>, published: &str| match result {
        Ok(recomputed) if recomputed == published => IdentityOutcome::Agrees,
        Ok(recomputed) => IdentityOutcome::Conflicts { recomputed },
        Err(detail) => IdentityOutcome::NotRecomputed { detail },
    };

    let mut records = Vec::new();
    for summary in &import.records {
        let (Some(key), Some(smiles)) = (&summary.inchikey, &summary.isomeric_smiles) else {
            records.push(IdentityCrossCheck {
                external_record_id: format!("CID{}", summary.cid),
                cid: summary.cid,
                smiles: summary.isomeric_smiles.clone().unwrap_or_default(),
                snapshot_inchi: summary.inchi.clone().unwrap_or_default(),
                snapshot_inchikey: summary.inchikey.clone().unwrap_or_default(),
                from_published_inchi: IdentityOutcome::NoSnapshotIdentity,
                from_structure: IdentityOutcome::NoSnapshotIdentity,
            });
            continue;
        };
        let from_published_inchi = match &summary.inchi {
            Some(inchi) if !inchi.is_empty() => verdict(rekey_published_inchi(inchi), key),
            _ => IdentityOutcome::NoSnapshotIdentity,
        };
        records.push(IdentityCrossCheck {
            external_record_id: format!("CID{}", summary.cid),
            cid: summary.cid,
            smiles: smiles.clone(),
            snapshot_inchi: summary.inchi.clone().unwrap_or_default(),
            snapshot_inchikey: key.clone(),
            from_published_inchi,
            from_structure: verdict(recompute_from_structure(smiles), key),
        });
    }

    IdentityCrossCheckReport {
        adapter_id: import.adapter_id.clone(),
        checked: records.len(),
        from_published_inchi: IdentityCheckSummary::tally(
            records.iter().map(|r| r.from_published_inchi.clone()),
        ),
        from_structure: IdentityCheckSummary::tally(
            records.iter().map(|r| r.from_structure.clone()),
        ),
        records,
    }
}
