//! BRD-011: the ChEBI identity and ontology adapter.
//!
//! Every acceptance clause in BREADTH.md § BRD-011 has a test here:
//!
//! * *pinned-release reproducibility* — [`snapshot_matches_the_pinned_manifest`]
//!   and [`candidates_are_byte_reproducible_from_the_pinned_release`];
//! * *tautomer/protonation conflicts are reported rather than merged* —
//!   [`protonation_pairs_are_reported_never_merged`] and its negative control
//!   [`diastereomers_are_not_reported_as_a_protonation_family`];
//! * *attribution survives pack compilation* —
//!   [`attribution_reaches_a_compiled_pack_manifest`];
//! * *no biological role is converted into a reaction rule* —
//!   [`role_firewall_refuses_a_role_reaching_safety`] and friends.
//!
//! Set `BLESS_CHEBI_FIXTURES=1` to rewrite the generated fixtures after an
//! intentional change; the tests then re-verify them on the next run.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use kerotakis_data::chebi::{
    chebi_candidates, chebi_promotion_policy, conflict_report, fields, inchi_charge,
    inchikey_family, lint_chebi_promotion, lint_role_firewall, normalize_chebi_label,
    recompute_formula_mass, relationship_report, ChebiConflict, ChebiEntity, ChebiSnapshot,
    FormulaIssue, RelatedFormEvidence, RoleFirewallViolation, ALLOWED_ONTOLOGY_TARGETS,
    CHEBI_ADAPTER_ID, CHEBI_LICENCE, ONTOLOGY_DERIVED_FIELDS, RESERVED_TARGET_MARKERS,
};
use kerotakis_data::{
    canonical_quarantine_bytes, default_runtime_data_licences, identity_conflicts,
    review_candidate, review_candidates, snapshot_sha256, EligibleFieldList, ModelPackManifest,
    PackContents, PackLane, PromotionPolicy, QuarantinedCandidate, RuntimeFieldPolicy,
    SnapshotManifest,
};
use serde_json::{json, Value};

// Accessions used by name throughout, so a test reads as chemistry.
const WATER: &str = "CHEBI:15377";
const ACETIC_ACID: &str = "CHEBI:15366";
const ACETATE: &str = "CHEBI:30089";
const AMMONIA: &str = "CHEBI:16134";
const AMMONIUM: &str = "CHEBI:28938";
const CITRATE_3: &str = "CHEBI:16947";
const CITRATE_2: &str = "CHEBI:35808";
const CITRIC_ACID: &str = "CHEBI:30769";
const MALTOSE: &str = "CHEBI:17306";
const LACTOSE: &str = "CHEBI:17716";
const CELLULOSE: &str = "CHEBI:18246";
const AMYLOSE: &str = "CHEBI:28102";
const D_GLUCOSE: &str = "CHEBI:17634";
const NICOTINE: &str = "CHEBI:18723";
const CAFFEINE: &str = "CHEBI:27732";
const SODIUM_CHLORIDE: &str = "CHEBI:26710";
const CHLOROPHYLL_A: &str = "CHEBI:18230";

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/quarantine/chebi-v1")
}

fn read(name: &str) -> Vec<u8> {
    let path = fixture_dir().join(name);
    std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn manifest() -> SnapshotManifest {
    serde_json::from_slice(&read("manifest.json")).expect("manifest parses")
}

fn raw_snapshot() -> Vec<u8> {
    read(manifest().raw_artifact.as_str())
}

fn snapshot() -> ChebiSnapshot {
    ChebiSnapshot::verified(&manifest(), &raw_snapshot()).expect("pinned snapshot verifies")
}

/// Write a generated fixture when blessing, otherwise assert it is unchanged.
fn assert_fixture(name: &str, actual: &[u8]) {
    let path = fixture_dir().join(name);
    if std::env::var("BLESS_CHEBI_FIXTURES").is_ok() {
        std::fs::create_dir_all(path.parent().expect("fixture has a parent")).expect("mkdir");
        std::fs::write(&path, actual).expect("write fixture");
        return;
    }
    let expected = std::fs::read(&path).unwrap_or_else(|error| {
        panic!(
            "read {}: {error}\nre-run with BLESS_CHEBI_FIXTURES=1",
            path.display()
        )
    });
    assert_eq!(
        String::from_utf8_lossy(&expected),
        String::from_utf8_lossy(actual),
        "{} drifted from the pinned release; re-run with BLESS_CHEBI_FIXTURES=1 if intended",
        path.display()
    );
}

fn entity<'a>(snapshot: &'a ChebiSnapshot, accession: &str) -> &'a ChebiEntity {
    snapshot
        .entities
        .iter()
        .find(|entity| entity.chebi_accession == accession)
        .unwrap_or_else(|| panic!("{accession} is in the pinned snapshot"))
}

fn candidate(candidates: &[QuarantinedCandidate], accession: &str) -> QuarantinedCandidate {
    candidates
        .iter()
        .find(|candidate| candidate.external_record_id == accession)
        .unwrap_or_else(|| panic!("{accession} has a candidate"))
        .clone()
}

// ---------------------------------------------------------------------------
// Pinned-release reproducibility
// ---------------------------------------------------------------------------

#[test]
fn snapshot_matches_the_pinned_manifest() {
    let manifest = manifest();
    let raw = raw_snapshot();

    assert_eq!(manifest.adapter_id, CHEBI_ADAPTER_ID);
    assert_eq!(manifest.source_id, "chebi");
    assert_eq!(
        manifest.source_revision, "253",
        "the release must be pinned by number, not by 'latest'"
    );
    manifest.verify(&raw).expect("checksum and manifest agree");
    assert_eq!(snapshot_sha256(&raw), manifest.sha256);

    let snapshot = ChebiSnapshot::parse(&raw).expect("snapshot parses");
    assert_eq!(snapshot.release, manifest.source_revision);
    assert_eq!(snapshot.release_date, "2026-07-07");
    assert_eq!(snapshot.licence, CHEBI_LICENCE);
    assert_eq!(snapshot.entities.len() as u64, manifest.record_count);
    assert_eq!(snapshot.entity_count, snapshot.entities.len());
}

#[test]
fn a_tampered_snapshot_is_refused() {
    let manifest = manifest();
    let mut raw = raw_snapshot();
    // Flip one byte of the payload: the release is pinned by content, so any
    // edit — including a benign-looking one — must stop the import.
    let position = raw.len() / 2;
    raw[position] ^= 0x20;

    let error = ChebiSnapshot::verified(&manifest, &raw).expect_err("tampering is refused");
    assert!(
        error.to_string().contains("checksum"),
        "expected a checksum refusal, got {error}"
    );
}

#[test]
fn a_manifest_for_another_adapter_is_refused() {
    let mut manifest = manifest();
    manifest.adapter_id = "pubchem-v1".into();
    let raw = raw_snapshot();
    // The checksum still matches; the adapter identity does not.
    let error = ChebiSnapshot::verified(&manifest, &raw).expect_err("adapter mismatch is refused");
    assert!(
        error.to_string().contains("pubchem-v1"),
        "expected an adapter refusal, got {error}"
    );
}

#[test]
fn candidates_are_byte_reproducible_from_the_pinned_release() {
    let snapshot = snapshot();
    let candidates = chebi_candidates(&snapshot);
    assert_eq!(candidates.len(), 87, "one candidate per reviewed entity");

    let canonical = canonical_quarantine_bytes(candidates.clone()).expect("candidates serialize");
    // Deriving twice from the same bytes must give the same bytes: no map
    // iteration order, no timestamp, no HashMap leaking into the output.
    let again = canonical_quarantine_bytes(chebi_candidates(
        &ChebiSnapshot::parse(&raw_snapshot()).unwrap(),
    ))
    .expect("candidates serialize");
    assert_eq!(canonical, again, "derivation is not deterministic");

    let field_count: usize = candidates
        .iter()
        .map(|candidate| candidate.fields.len())
        .sum();
    let digest = json!({
        "schema": 1,
        "adapter_id": CHEBI_ADAPTER_ID,
        "source_revision": snapshot.release,
        "record_count": candidates.len(),
        "field_count": field_count,
        "canonical_sha256": snapshot_sha256(&canonical),
    });
    // The digest, not the 400 KB candidate dump, is what the repository pins:
    // it proves byte-exact reproducibility at a size a reviewer can read.
    assert_fixture(
        "candidates-digest.json",
        format!("{}\n", serde_json::to_string_pretty(&digest).unwrap()).as_bytes(),
    );

    // A readable slice is committed too, so the candidate shape is reviewable
    // in a diff and the refresh-diff tooling has something to chew on.
    let sample: Vec<QuarantinedCandidate> =
        [ACETIC_ACID, ACETATE, WATER, CELLULOSE, D_GLUCOSE, CAFFEINE]
            .iter()
            .map(|accession| candidate(&candidates, accession))
            .collect();
    assert_fixture(
        "candidates-sample.json",
        &canonical_quarantine_bytes(sample).expect("sample serializes"),
    );
}

#[test]
fn every_candidate_carries_identity_and_provenance() {
    let snapshot = snapshot();
    for candidate in chebi_candidates(&snapshot) {
        assert_eq!(candidate.adapter_id, CHEBI_ADAPTER_ID);
        assert!(
            candidate.source_record_id.starts_with("253:"),
            "{} does not name the pinned release",
            candidate.external_record_id
        );
        assert!(candidate.external_record_id.starts_with("CHEBI:"));
        assert!(
            candidate.fields.contains_key(fields::CHEBI_ID),
            "{} lost its external identifier",
            candidate.external_record_id
        );
        for (name, field) in &candidate.fields {
            assert_eq!(
                field.licence, CHEBI_LICENCE,
                "{}/{name} carries the wrong licence",
                candidate.external_record_id
            );
            assert!(
                field
                    .source_field
                    .starts_with(&format!("entities[{}].", candidate.external_record_id)),
                "{}/{name} has a source path that does not name its record: {}",
                candidate.external_record_id,
                field.source_field
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Reviewed material only
// ---------------------------------------------------------------------------

#[test]
fn only_reviewed_three_star_entities_become_candidates() {
    let mut snapshot = snapshot();
    assert!(
        snapshot.entities.iter().all(|entity| entity.stars == 3),
        "the curated snapshot is three-star throughout"
    );

    // Plant a two-star deposit and an unreviewed three-star one.
    let mut two_star = entity(&snapshot, WATER).clone();
    two_star.chebi_accession = "CHEBI:9999901".into();
    two_star.stars = 2;
    let mut submitted = entity(&snapshot, WATER).clone();
    submitted.chebi_accession = "CHEBI:9999902".into();
    submitted.status = "SUBMITTED".into();
    snapshot.entities.push(two_star);
    snapshot.entities.push(submitted);

    let ids: BTreeSet<String> = chebi_candidates(&snapshot)
        .into_iter()
        .map(|candidate| candidate.external_record_id)
        .collect();
    assert!(
        !ids.contains("CHEBI:9999901"),
        "a two-star entity became a candidate"
    );
    assert!(
        !ids.contains("CHEBI:9999902"),
        "an unreviewed entity became a candidate"
    );
    assert_eq!(ids.len(), 87);
}

#[test]
fn unreviewed_ontology_relations_are_not_ingested() {
    let snapshot = snapshot();
    // Caffeine carries a mix of CHECKED and SUBMITTED role assertions in the
    // pinned release, which makes it the honest test of the relation filter.
    let caffeine = entity(&snapshot, CAFFEINE);
    let submitted: Vec<&str> = caffeine
        .relations
        .iter()
        .filter(|relation| relation.relation == "has_role" && relation.status == "SUBMITTED")
        .map(|relation| relation.target.as_str())
        .collect();
    assert!(
        !submitted.is_empty(),
        "fixture no longer exercises unreviewed relations"
    );

    let candidates = chebi_candidates(&snapshot);
    let tags = candidate(&candidates, CAFFEINE)
        .fields
        .get(fields::SEARCH_TAGS_FROM_ROLES)
        .expect("caffeine has role tags")
        .value
        .clone();
    let ingested: BTreeSet<String> = tags
        .as_array()
        .expect("tags are an array")
        .iter()
        .map(|tag| tag["chebi_id"].as_str().unwrap().to_owned())
        .collect();
    for target in submitted {
        assert!(
            !ingested.contains(target),
            "unreviewed role {target} reached the candidate"
        );
    }
}

// ---------------------------------------------------------------------------
// Tautomer / protonation: reported, never merged
// ---------------------------------------------------------------------------

#[test]
fn protonation_pairs_are_reported_never_merged() {
    let snapshot = snapshot();
    let candidates = chebi_candidates(&snapshot);

    let acid = candidate(&candidates, ACETIC_ACID);
    let base = candidate(&candidates, ACETATE);

    // Two ChEBI IDs, two InChIKeys, two candidates. Nothing is merged.
    assert_ne!(acid.external_record_id, base.external_record_id);
    assert_eq!(
        acid.identity_key.as_deref(),
        Some("QTBSBXVTEAMEQO-UHFFFAOYSA-N")
    );
    assert_eq!(
        base.identity_key.as_deref(),
        Some("QTBSBXVTEAMEQO-UHFFFAOYSA-M")
    );
    assert_ne!(
        acid.identity_key, base.identity_key,
        "the join key must separate an acid from its conjugate base"
    );

    // The BRD-003 identity grouper — which merges on a shared key — must find
    // nothing at all. That is the "never merged" half of the acceptance.
    assert!(
        identity_conflicts(&candidates).is_empty(),
        "no two ChEBI identities may share a join key: {:?}",
        identity_conflicts(&candidates)
    );

    // The "reported" half: the relationship lives in its own report.
    let report = relationship_report(&snapshot);
    let form = report
        .related_forms
        .iter()
        .find(|form| form.left == ACETIC_ACID && form.right == ACETATE)
        .expect("acetic acid / acetate is reported as a related form");
    assert_eq!(form.evidence, RelatedFormEvidence::OntologyAndStructure);
    assert!(form
        .asserted_relations
        .iter()
        .any(|relation| relation.starts_with("is_conjugate_")));
    assert_ne!(form.left_identity_key, form.right_identity_key);
}

#[test]
fn the_related_form_report_covers_the_expected_families() {
    let snapshot = snapshot();
    let report = relationship_report(&snapshot);

    let pairs: BTreeSet<(String, String)> = report
        .related_forms
        .iter()
        .map(|form| (form.left.clone(), form.right.clone()))
        .collect();

    // Ontology and structure agree.
    assert!(pairs.contains(&(AMMONIA.into(), AMMONIUM.into())));
    // Structure only: ChEBI links citric acid to citrate(3-) through the
    // intermediate protonation states, not directly.
    let citrate = report
        .related_forms
        .iter()
        .find(|form| form.left == CITRATE_3 && form.right == CITRIC_ACID)
        .expect("citric acid / citrate(3-) share an InChIKey family");
    assert_eq!(citrate.evidence, RelatedFormEvidence::StructureOnly);
    assert!(citrate.asserted_relations.is_empty());

    // Ontology only: citrate(2-) is an ontology class with no single defined
    // structure, so no structural family can corroborate the assertion.
    let stepwise = report
        .related_forms
        .iter()
        .find(|form| form.left == CITRATE_2 || form.right == CITRATE_2)
        .expect("citrate(2-) is related to citrate(3-)");
    assert_eq!(stepwise.evidence, RelatedFormEvidence::OntologyOnly);
    assert!(stepwise.left_identity_key.is_none() || stepwise.right_identity_key.is_none());

    assert!(
        report.related_forms.len() >= 15,
        "expected the full family set, got {}",
        report.related_forms.len()
    );
    assert_eq!(report.source_revision, "253");
}

#[test]
fn diastereomers_are_not_reported_as_a_protonation_family() {
    let snapshot = snapshot();
    let report = relationship_report(&snapshot);

    // Negative control. Maltose and lactose share the 14-character InChIKey
    // skeleton block, and so do cellulose and amylose. They are diastereomers,
    // not protonation states, and a skeleton-only heuristic would merge them.
    for (left, right) in [(MALTOSE, LACTOSE), (CELLULOSE, AMYLOSE)] {
        let left_key = entity(&snapshot, left).standard_inchi_key.clone().unwrap();
        let right_key = entity(&snapshot, right).standard_inchi_key.clone().unwrap();
        assert_eq!(
            left_key[..14],
            right_key[..14],
            "{left}/{right} should still share a skeleton block"
        );
        assert_ne!(
            inchikey_family(&left_key),
            inchikey_family(&right_key),
            "{left}/{right} must differ once the stereo block is included"
        );
        assert!(
            !report
                .related_forms
                .iter()
                .any(|form| (form.left == left && form.right == right)
                    || (form.left == right && form.right == left)),
            "{left}/{right} were wrongly reported as one family"
        );
    }
}

#[test]
fn inchikey_family_needs_a_well_formed_key() {
    assert_eq!(
        inchikey_family("QTBSBXVTEAMEQO-UHFFFAOYSA-N"),
        Some("QTBSBXVTEAMEQO-UHFFFAOYSA")
    );
    assert_eq!(inchikey_family("too-short"), None);
    assert_eq!(inchikey_family(""), None);
    assert_eq!(inchikey_family("QTBSBXVTEAMEQOXUHFFFAOYSAXN"), None);
}

// ---------------------------------------------------------------------------
// Conflict report: stated identity versus recomputed identity
// ---------------------------------------------------------------------------

#[test]
fn recomputed_mass_agrees_with_the_pinned_release() {
    let snapshot = snapshot();
    let conflicts = conflict_report(&snapshot);

    // No entity in the pinned release has a mass that disagrees with its own
    // formula: the tolerance is wide enough for atomic-weight revisions and no
    // wider.
    let disagreements: Vec<&ChebiConflict> = conflicts
        .iter()
        .filter(|conflict| matches!(conflict, ChebiConflict::MassDisagreesWithFormula { .. }))
        .collect();
    assert!(
        disagreements.is_empty(),
        "unexpected mass disagreements: {disagreements:?}"
    );

    // Spot-check the recomputation itself rather than trusting the absence.
    let water = recompute_formula_mass("H2O").unwrap();
    assert!((water - 18.015).abs() < 1e-9, "H2O recomputed as {water}");
    let salt = recompute_formula_mass("Cl.Na").unwrap();
    assert!((salt - 58.4398).abs() < 1e-3, "Cl.Na recomputed as {salt}");
    let chlorophyll = recompute_formula_mass("C55H72MgN4O5").unwrap();
    assert!(
        (chlorophyll - 893.509).abs() < 0.01,
        "chlorophyll a recomputed as {chlorophyll}"
    );
    assert_eq!(
        entity(&snapshot, CHLOROPHYLL_A).formula.as_deref(),
        Some("C55H72MgN4O5")
    );
}

#[test]
fn a_planted_mass_error_reaches_the_conflict_report() {
    let mut snapshot = snapshot();
    // Water, restated as if it weighed what methane does.
    let water = snapshot
        .entities
        .iter_mut()
        .find(|entity| entity.chebi_accession == WATER)
        .unwrap();
    water.mass = Some(16.043);

    let conflict = conflict_report(&snapshot)
        .into_iter()
        .find(|conflict| {
            conflict.chebi_id() == WATER
                && matches!(conflict, ChebiConflict::MassDisagreesWithFormula { .. })
        })
        .expect("a 2 Da error must be reported");
    match conflict {
        ChebiConflict::MassDisagreesWithFormula {
            formula,
            stated_mass,
            recomputed_mass,
            difference,
            tolerance,
            ..
        } => {
            assert_eq!(formula, "H2O");
            assert!((stated_mass - 16.043).abs() < 1e-9);
            assert!((recomputed_mass - 18.015).abs() < 1e-9);
            assert!(difference > tolerance);
        }
        other => panic!("wrong conflict: {other:?}"),
    }
}

#[test]
fn indeterminate_polymer_masses_are_reported_not_promoted() {
    let snapshot = snapshot();
    let conflicts = conflict_report(&snapshot);

    // Cellulose and amylose are `(C6H10O5)n.H2O`: ChEBI states 180.156 Da,
    // which is a single glucose residue, not the polymer. No finite mass
    // follows from the formula, so the number cannot be corroborated.
    for accession in [CELLULOSE, AMYLOSE] {
        let conflict = conflicts
            .iter()
            .find(|conflict| {
                conflict.chebi_id() == accession
                    && matches!(conflict, ChebiConflict::MassNotRecomputable { .. })
            })
            .unwrap_or_else(|| panic!("{accession} must report an unrecomputable mass"));
        match conflict {
            ChebiConflict::MassNotRecomputable { formula, issue, .. } => {
                assert!(formula.contains(")n"), "{formula} is not a polymer formula");
                assert_eq!(*issue, FormulaIssue::IndeterminateRepeat);
            }
            other => panic!("wrong conflict: {other:?}"),
        }
    }

    assert_eq!(
        recompute_formula_mass("(C6H10O5)n.H2O"),
        Err(FormulaIssue::IndeterminateRepeat)
    );
}

#[test]
fn charge_is_cross_checked_against_the_inchi_layers() {
    let snapshot = snapshot();

    // A multi-component salt nets to zero only when /q and /p are both read.
    // Reading /p alone would report a false conflict on every salt here.
    assert_eq!(inchi_charge("InChI=1S/ClH.Na/h1H;/q;+1/p-1"), Some(0));
    assert_eq!(
        inchi_charge("InChI=1S/CH2O3.Ca/c2-1(3)4;/h(H2,2,3,4);/q;+2/p-2"),
        Some(0)
    );
    assert_eq!(
        inchi_charge("InChI=1S/C2H4O2/c1-2(3)4/h1H3,(H,3,4)/p-1"),
        Some(-1)
    );
    assert_eq!(inchi_charge("InChI=1S/H2O/h1H2"), None);

    let conflicts = conflict_report(&snapshot);
    let charge_conflicts: Vec<&ChebiConflict> = conflicts
        .iter()
        .filter(|conflict| matches!(conflict, ChebiConflict::ChargeDisagreesWithStructure { .. }))
        .collect();
    assert!(
        charge_conflicts.is_empty(),
        "the pinned release has no charge disagreements: {charge_conflicts:?}"
    );
    assert_eq!(entity(&snapshot, SODIUM_CHLORIDE).charge, Some(0));
}

#[test]
fn a_planted_charge_error_reaches_the_conflict_report() {
    let mut snapshot = snapshot();
    let acetate = snapshot
        .entities
        .iter_mut()
        .find(|entity| entity.chebi_accession == ACETATE)
        .unwrap();
    acetate.charge = Some(0);

    let found = conflict_report(&snapshot).into_iter().any(|conflict| {
        matches!(
            conflict,
            ChebiConflict::ChargeDisagreesWithStructure {
                ref chebi_id,
                stated_charge: 0,
                structural_charge: -1,
            } if chebi_id == ACETATE
        )
    });
    assert!(
        found,
        "a charge restated against the InChI must be reported"
    );
}

#[test]
fn entities_without_a_join_key_are_reported() {
    let snapshot = snapshot();
    let conflicts = conflict_report(&snapshot);

    let keyless: BTreeSet<&str> = conflicts
        .iter()
        .filter_map(|conflict| match conflict {
            ChebiConflict::NoIdentityKey { chebi_id, .. } => Some(chebi_id.as_str()),
            _ => None,
        })
        .collect();
    // ChEBI's `D-glucose` is an ontology class covering several defined
    // structures, so it has no single InChIKey. Inventing one would be the
    // silent-merge failure this adapter exists to avoid.
    assert!(
        keyless.contains(D_GLUCOSE),
        "expected D-glucose among {keyless:?}"
    );
    assert!(keyless.contains(CITRATE_2));
    assert_eq!(keyless.len(), 9);

    let candidates = chebi_candidates(&snapshot);
    for accession in &keyless {
        assert!(
            candidate(&candidates, accession).identity_key.is_none(),
            "{accession} must not carry a fabricated join key"
        );
    }

    // Nicotine is a class with no chemical data at all in this release.
    assert!(conflicts.iter().any(|conflict| matches!(
        conflict,
        ChebiConflict::NoChemicalData { chebi_id, .. } if chebi_id == NICOTINE
    )));
}

// ---------------------------------------------------------------------------
// Formula parser and label normalization
// ---------------------------------------------------------------------------

#[test]
fn the_formula_parser_accepts_chebis_dialect() {
    assert!((recompute_formula_mass("C2H4O2").unwrap() - 60.052).abs() < 1e-9);
    assert!((recompute_formula_mass("CO3.Ca").unwrap() - 100.083).abs() < 0.01);
    assert!((recompute_formula_mass("HO.Na").unwrap() - 39.997).abs() < 0.01);
    // A leading component multiplier and a parenthesised group with a count.
    let hydrate = recompute_formula_mass("CaO.2H2O").unwrap();
    assert!((hydrate - (40.078 + 15.999 + 2.0 * 18.015)).abs() < 1e-9);
    let group = recompute_formula_mass("(CH3)2O").unwrap();
    assert!((group - (2.0 * (12.011 + 3.0 * 1.008) + 15.999)).abs() < 1e-9);

    assert_eq!(
        recompute_formula_mass("C6H12Xx"),
        Err(FormulaIssue::UnknownElement {
            symbol: "Xx".into()
        })
    );
    assert!(matches!(
        recompute_formula_mass("(C6H10O5"),
        Err(FormulaIssue::Malformed { .. })
    ));
    assert!(matches!(
        recompute_formula_mass("C6H10O5)"),
        Err(FormulaIssue::Malformed { .. })
    ));
    assert!(matches!(
        recompute_formula_mass("H2O..CO2"),
        Err(FormulaIssue::Malformed { .. })
    ));
}

#[test]
fn labels_lose_markup_but_keep_chemistry() {
    // Presentation markup goes.
    assert_eq!(
        normalize_chebi_label("<small>D</small>-glucose"),
        "D-glucose"
    );
    assert_eq!(
        normalize_chebi_label("NAD<small><sup>+</small></sup>"),
        "NAD+"
    );
    assert_eq!(
        normalize_chebi_label("(<i>S</i>)-malic acid"),
        "(S)-malic acid"
    );
    assert_eq!(
        normalize_chebi_label("chlorophyll <em>a</em>"),
        "chlorophyll a"
    );
    // U+2212 MINUS SIGN folds to ASCII so a typed name can match.
    assert_eq!(normalize_chebi_label("citrate(3\u{2212})"), "citrate(3-)");
    // Chemistry and language survive: Greek letters, arrows, umlauts.
    assert_eq!(
        normalize_chebi_label("(1\u{2192}4)-\u{3b2}-D-glucan"),
        "(1\u{2192}4)-\u{3b2}-D-glucan"
    );
    assert_eq!(normalize_chebi_label("Essigs\u{e4}ure"), "Essigs\u{e4}ure");
    assert_eq!(normalize_chebi_label("  spaced   out  "), "spaced out");
}

#[test]
fn candidate_names_and_aliases_are_normalized() {
    let snapshot = snapshot();
    let candidates = chebi_candidates(&snapshot);

    let glucose = candidate(&candidates, D_GLUCOSE);
    assert_eq!(
        glucose.fields[fields::CANONICAL_NAME].value,
        Value::String("D-glucose".into())
    );

    // Cellulose is only reachable by synonym: its ChEBI display name is
    // "(1->4)-beta-D-glucan". A name-only lookup would have missed it.
    let cellulose = candidate(&candidates, CELLULOSE);
    let aliases = cellulose.fields[fields::SYNONYMS].value.as_array().unwrap();
    // ChEBI's synonym casing is inconsistent ("Cellulose" here), so a search
    // index folds case; the point is that the familiar name is present at all.
    assert!(
        aliases.iter().any(|alias| alias["name"]
            .as_str()
            .is_some_and(|name| name.eq_ignore_ascii_case("cellulose"))),
        "cellulose must be reachable through its synonyms"
    );
    // German aliases are ingested, which BRD-012's locale work depends on.
    assert!(
        candidates.iter().any(|candidate| candidate
            .fields
            .get(fields::SYNONYMS)
            .and_then(|field| field.value.as_array())
            .is_some_and(|aliases| aliases
                .iter()
                .any(|alias| alias["language"].as_str() == Some("de")))),
        "no German aliases survived ingestion"
    );
    // Trademarks do not.
    for candidate in &candidates {
        let Some(aliases) = candidate
            .fields
            .get(fields::SYNONYMS)
            .and_then(|field| field.value.as_array())
        else {
            continue;
        };
        assert!(
            aliases
                .iter()
                .all(|alias| alias["name_type"].as_str() != Some("BRAND NAME")),
            "{} ingested a brand name",
            candidate.external_record_id
        );
    }
}

// ---------------------------------------------------------------------------
// The role firewall
// ---------------------------------------------------------------------------

#[test]
fn the_reviewed_policy_passes_the_firewall() {
    let policy = chebi_promotion_policy();
    assert!(
        lint_role_firewall(&policy).is_empty(),
        "the shipped policy must pass its own firewall: {:?}",
        lint_role_firewall(&policy)
    );
    // Roles reach tag targets and nothing else.
    for field in ONTOLOGY_DERIVED_FIELDS {
        let target = &policy.fields[*field].target_field;
        assert!(
            ALLOWED_ONTOLOGY_TARGETS.contains(&target.as_str()),
            "{field} targets {target}"
        );
    }
    assert_fixture(
        "policy.json",
        format!("{}\n", serde_json::to_string_pretty(&policy).unwrap()).as_bytes(),
    );
}

#[test]
fn role_firewall_refuses_a_role_reaching_safety() {
    // The planted violation: a curator decides ChEBI's "has role" annotations
    // are close enough to hazard data. They are not — a role such as
    // "neurotoxin" is a biological annotation, not a risk assessment.
    let mut policy = chebi_promotion_policy();
    policy.fields.insert(
        fields::SEARCH_TAGS_FROM_ROLES.to_owned(),
        RuntimeFieldPolicy::new("safety_flags", [CHEBI_LICENCE]),
    );

    let violations = lint_role_firewall(&policy);
    assert!(
        violations.iter().any(|violation| matches!(
            violation,
            RoleFirewallViolation::ReservedTargetFromChebi { source_field, target_field, .. }
                if source_field == fields::SEARCH_TAGS_FROM_ROLES && target_field == "safety_flags"
        )),
        "the firewall must name the reserved target: {violations:?}"
    );
    assert!(
        violations.iter().any(|violation| matches!(
            violation,
            RoleFirewallViolation::OntologyFieldTargetsNonTag { source_field, .. }
                if source_field == fields::SEARCH_TAGS_FROM_ROLES
        )),
        "the firewall must also refuse on default-deny grounds: {violations:?}"
    );

    // And the refusal is load-bearing: the whole promotion stops.
    let snapshot = snapshot();
    let candidates = chebi_candidates(&snapshot);
    let report = lint_chebi_promotion(
        &manifest(),
        &raw_snapshot(),
        &snapshot,
        &candidates,
        &policy,
        &default_runtime_data_licences(),
        &[],
    );
    assert!(report.refuses(), "a firewall breach must refuse promotion");

    assert_fixture(
        "policy-role-breach.json",
        format!("{}\n", serde_json::to_string_pretty(&policy).unwrap()).as_bytes(),
    );
}

#[test]
fn role_firewall_refuses_every_reserved_target_spelling() {
    // Default-deny is about the shape of the target, not a fixed list, so
    // variant spellings are caught too.
    for target in [
        "safety_flags",
        "ghs_hazard_class",
        "reactivity_notes",
        "reaction_rule_family",
        "precautionary_statements",
        "acute_toxicity",
        "incompatible_with",
        "flammability_rating",
        "hazard_pictograms",
    ] {
        let mut policy = chebi_promotion_policy();
        policy.fields.insert(
            fields::SEARCH_TAGS_FROM_ROLES.to_owned(),
            RuntimeFieldPolicy::new(target, [CHEBI_LICENCE]),
        );
        let violations = lint_role_firewall(&policy);
        assert!(
            violations.iter().any(|violation| matches!(
                violation,
                RoleFirewallViolation::ReservedTargetFromChebi { target_field, .. }
                    if target_field == target
            )),
            "{target} slipped past the firewall: {violations:?}"
        );
    }
}

#[test]
fn no_chebi_field_at_all_may_reach_a_reserved_target() {
    // Not just roles: the formula, the name, anything. ChEBI asserts no hazard
    // or kinetic claim, so nothing it supplies may be dressed up as one.
    for source in [fields::FORMULA, fields::CANONICAL_NAME, fields::DEFINITION] {
        let mut policy = chebi_promotion_policy();
        policy.fields.insert(
            source.to_owned(),
            RuntimeFieldPolicy::new("reactivity_class", [CHEBI_LICENCE]),
        );
        assert!(
            lint_role_firewall(&policy).iter().any(|violation| matches!(
                violation,
                RoleFirewallViolation::ReservedTargetFromChebi { source_field, .. }
                    if source_field == source
            )),
            "{source} reached a reactivity target unchallenged"
        );
    }
}

#[test]
fn the_tag_lane_is_closed_in_both_directions() {
    // A non-ontology field must not land on a tag target either; otherwise the
    // firewall would only be a guarantee about field names.
    let mut policy = chebi_promotion_policy();
    policy.fields.insert(
        fields::FORMULA.to_owned(),
        RuntimeFieldPolicy::new("search_tags", [CHEBI_LICENCE]),
    );
    assert!(
        lint_role_firewall(&policy).iter().any(|violation| matches!(
            violation,
            RoleFirewallViolation::TagTargetFromNonOntologyField { source_field, .. }
                if source_field == fields::FORMULA
        )),
        "a non-ontology field reached the tag lane"
    );
}

#[test]
fn roles_stay_inside_the_candidate_metadata() {
    let snapshot = snapshot();
    let policy = chebi_promotion_policy();
    let candidates = chebi_candidates(&snapshot);

    // Follow the data, not just the policy: after a real review, role-derived
    // values must land on tag targets and nowhere else.
    //
    // The check is structural, not textual. Role *vocabulary* legitimately
    // overlaps chemical vocabulary — caffeine's `is_a` parent is
    // "trimethylxanthine" and its synonym list contains
    // "1,3,7-Trimethylxanthine" — so a substring search would report a leak
    // where there is only a shared word. What must not happen is a tag
    // *structure* appearing under a target that is not a tag target.
    let candidate = candidate(&candidates, CAFFEINE);
    let review = review_candidate(&candidate, &policy);

    let is_tag_array = |value: &Value| {
        value.as_array().is_some_and(|tags| {
            !tags.is_empty()
                && tags
                    .iter()
                    .all(|tag| tag.get("chebi_id").is_some() && tag.get("label").is_some())
        })
    };

    // The role array arrives intact, on the tag target the policy names.
    let roles = &candidate.fields[fields::SEARCH_TAGS_FROM_ROLES].value;
    assert!(is_tag_array(roles), "caffeine lost its role tags");
    assert_eq!(&review.accepted["search_tags"].value, roles);
    let parents = &candidate.fields[fields::SEARCH_TAGS_FROM_PARENTS].value;
    assert_eq!(&review.accepted["class_tags"].value, parents);

    for (target, field) in &review.accepted {
        if ALLOWED_ONTOLOGY_TARGETS.contains(&target.as_str()) {
            continue;
        }
        assert_ne!(&field.value, roles, "the role array reached {target}");
        assert_ne!(&field.value, parents, "the parent array reached {target}");
        assert!(
            !is_tag_array(&field.value),
            "an ontology tag structure reached the non-tag target {target}"
        );
        // And no reserved target exists in the accepted set in the first place.
        assert!(
            !RESERVED_TARGET_MARKERS
                .iter()
                .any(|marker| target.to_ascii_lowercase().contains(marker)),
            "the review produced a reserved target: {target}"
        );
    }
}

// ---------------------------------------------------------------------------
// Attribution
// ---------------------------------------------------------------------------

#[test]
fn every_candidate_carries_cc_by_attribution() {
    let snapshot = snapshot();
    for candidate in chebi_candidates(&snapshot) {
        let attribution = candidate
            .fields
            .get(fields::ATTRIBUTION)
            .unwrap_or_else(|| panic!("{} lost its attribution", candidate.external_record_id));
        let text = attribution.value.as_str().expect("attribution is text");
        assert!(text.contains("ChEBI"), "attribution names the database");
        assert!(
            text.contains("253"),
            "attribution names the pinned release, as CC BY 4.0 asks"
        );
        assert!(text.contains("CC BY 4.0"), "attribution names the licence");
        assert_eq!(attribution.licence, CHEBI_LICENCE);
    }
}

#[test]
fn attribution_reaches_a_compiled_pack_manifest() {
    let snapshot = snapshot();
    let policy = chebi_promotion_policy();
    let candidates = chebi_candidates(&snapshot);
    let report = review_candidates(candidates.clone(), &policy);

    // Attribution is an ordinary promotable field, so it comes out the other
    // side of review with the rest of the record rather than being reattached
    // by hand later.
    let notices: BTreeSet<String> = report
        .reviews
        .iter()
        .filter_map(|review| review.accepted.get("attribution"))
        .filter_map(|field| field.value.as_str().map(str::to_owned))
        .collect();
    assert_eq!(
        notices.len(),
        1,
        "one pinned release yields one notice, deduplicated"
    );
    assert_eq!(report.reviews.len(), candidates.len());

    // Compile it into what a distributable pack would carry.
    let attribution = notices.iter().next().unwrap().clone();
    let pack = ModelPackManifest {
        id: "chebi-identity-v1".into(),
        name: "ChEBI identity slice".into(),
        version: "0.1.0".into(),
        content_hash: snapshot_sha256(
            &canonical_quarantine_bytes(candidates.clone()).expect("serialize"),
        ),
        engine_abi: "1".into(),
        data_schema: 1,
        licence: CHEBI_LICENCE.into(),
        attribution: attribution.clone(),
        source_url: Some("https://www.ebi.ac.uk/chebi".into()),
        signature: None,
        min_app_version: "0.1.0".into(),
        lane: PackLane::Development,
        contents: PackContents {
            species_count: candidates.len(),
            ..PackContents::default()
        },
    };

    // Round-trip it: the notice must survive serialization into the artifact a
    // user would actually receive.
    let bytes = serde_json::to_vec(&pack).expect("pack manifest serializes");
    let restored: ModelPackManifest = serde_json::from_slice(&bytes).expect("pack manifest parses");
    assert_eq!(restored.attribution, attribution);
    assert!(restored.attribution.contains("ChEBI"));
    assert!(restored.attribution.contains("253"));
    assert!(restored.attribution.contains("CC BY 4.0"));
    assert_eq!(restored.licence, CHEBI_LICENCE);

    // And the per-record trail is still there, so a reviewer can go from the
    // pack notice back to the exact source field.
    let water = report
        .reviews
        .iter()
        .find(|review| review.external_record_id == WATER)
        .expect("water was reviewed");
    let field = &water.accepted["attribution"];
    assert_eq!(field.source_record_id, format!("253:{WATER}"));
    assert_eq!(field.source_field, format!("entities[{WATER}].attribution"));
    assert_eq!(field.licence, CHEBI_LICENCE);
}

// ---------------------------------------------------------------------------
// Promotion dry run
// ---------------------------------------------------------------------------

fn eligible_for(accession: &str, fields: &[&str]) -> EligibleFieldList {
    EligibleFieldList {
        adapter_id: CHEBI_ADAPTER_ID.to_owned(),
        external_record_id: accession.to_owned(),
        fields: fields.iter().map(|field| (*field).to_owned()).collect(),
    }
}

#[test]
fn the_promotion_dry_run_passes_for_eligible_fields() {
    let snapshot = snapshot();
    let candidates = chebi_candidates(&snapshot);
    let policy = chebi_promotion_policy();
    let eligible = vec![
        eligible_for(
            WATER,
            &[
                fields::CHEBI_ID,
                fields::CANONICAL_NAME,
                fields::FORMULA,
                fields::AVERAGE_MASS,
                fields::STANDARD_INCHI_KEY,
                fields::ATTRIBUTION,
            ],
        ),
        eligible_for(
            ACETIC_ACID,
            &[
                fields::CANONICAL_NAME,
                fields::SEARCH_TAGS_FROM_ROLES,
                fields::ATTRIBUTION,
            ],
        ),
    ];
    assert_fixture(
        "eligible.json",
        format!("{}\n", serde_json::to_string_pretty(&eligible).unwrap()).as_bytes(),
    );

    let report = lint_chebi_promotion(
        &manifest(),
        &raw_snapshot(),
        &snapshot,
        &candidates,
        &policy,
        &default_runtime_data_licences(),
        &eligible,
    );
    assert!(
        !report.refuses(),
        "the reviewed flow must pass: {:?} / {:?}",
        report.provenance.violations,
        report.role_firewall
    );
    assert_eq!(report.provenance.adapter_id, CHEBI_ADAPTER_ID);
    assert_eq!(report.provenance.checked_records, candidates.len());

    // Conflicts and related forms are reports, not refusals: a protonation
    // pair and a polymer with an indeterminate formula are normal chemistry.
    assert!(!report.conflicts.is_empty());
    assert!(!report.related_forms.related_forms.is_empty());

    // The mass promotes through the reviewed unit vocabulary: "Da" normalizes
    // onto MolarMass rather than being taken on trust.
    let review = review_candidate(&candidate(&candidates, WATER), &policy);
    let mass = &review.accepted["molar_mass"];
    assert_eq!(mass.source_unit.as_deref(), Some("Da"));
    assert!(mass.unit.is_some(), "the mass lost its canonical unit");
    assert!((mass.value.as_f64().unwrap() - 18.015).abs() < 1e-9);
    assert!(review.rejected.is_empty(), "{:?}", review.rejected);
}

#[test]
fn the_promotion_dry_run_refuses_planted_violations() {
    let snapshot = snapshot();
    let candidates = chebi_candidates(&snapshot);
    let policy = chebi_promotion_policy();

    // (a) An eligible list written against a different snapshot.
    let invalid = vec![
        eligible_for(WATER, &[fields::CANONICAL_NAME, "boiling_point"]),
        eligible_for("CHEBI:99999999", &[fields::CANONICAL_NAME]),
    ];
    assert_fixture(
        "eligible-invalid.json",
        format!("{}\n", serde_json::to_string_pretty(&invalid).unwrap()).as_bytes(),
    );
    let report = lint_chebi_promotion(
        &manifest(),
        &raw_snapshot(),
        &snapshot,
        &candidates,
        &policy,
        &default_runtime_data_licences(),
        &invalid,
    );
    assert!(report.refuses(), "an eligible list must be checked");
    assert!(
        report.role_firewall.is_empty(),
        "this breach is not a firewall one"
    );

    // (b) A policy that would admit a licence outside the runtime lane.
    let mut wide = chebi_promotion_policy();
    wide.fields.insert(
        fields::DEFINITION.to_owned(),
        RuntimeFieldPolicy::new("description", ["CC-BY-SA-4.0"]),
    );
    let report = lint_chebi_promotion(
        &manifest(),
        &raw_snapshot(),
        &snapshot,
        &candidates,
        &wide,
        &default_runtime_data_licences(),
        &[],
    );
    assert!(
        report.refuses(),
        "ShareAlike must not enter the runtime lane"
    );

    // (c) A candidate whose provenance was stripped.
    let mut tainted = candidates.clone();
    let water = tainted
        .iter_mut()
        .find(|candidate| candidate.external_record_id == WATER)
        .unwrap();
    water
        .fields
        .get_mut(fields::FORMULA)
        .unwrap()
        .source_field
        .clear();
    let report = lint_chebi_promotion(
        &manifest(),
        &raw_snapshot(),
        &snapshot,
        &tainted,
        &policy,
        &default_runtime_data_licences(),
        &[eligible_for(WATER, &[fields::FORMULA])],
    );
    assert!(
        report.refuses(),
        "a field without a source path must refuse"
    );

    // (d) Candidates that could not have come from the pinned snapshot.
    let mut manifest = manifest();
    manifest.record_count = 2;
    let report = lint_chebi_promotion(
        &manifest,
        &raw_snapshot(),
        &snapshot,
        &candidates,
        &policy,
        &default_runtime_data_licences(),
        &[],
    );
    assert!(report.refuses(), "more candidates than records must refuse");
}

#[test]
fn nothing_here_touches_the_runtime_registry() {
    // A guard against scope creep: the adapter's whole output is candidates,
    // reports and a policy. If a future change makes it emit registry
    // documents, this test is where that shows up.
    let snapshot = snapshot();
    let candidates = chebi_candidates(&snapshot);
    let policy: PromotionPolicy = chebi_promotion_policy();

    let targets: BTreeSet<&str> = policy
        .fields
        .values()
        .map(|rule| rule.target_field.as_str())
        .collect();
    // Targets are *named*, not written: review produces a map keyed by them.
    let review = review_candidate(&candidate(&candidates, WATER), &policy);
    for target in review.accepted.keys() {
        assert!(targets.contains(target.as_str()));
    }

    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for candidate in &candidates {
        for field in candidate.fields.keys() {
            *counts.entry(field.as_str()).or_default() += 1;
        }
    }
    // Every emitted field is one the policy knows about; no stowaways.
    for field in counts.keys() {
        assert!(
            policy.fields.contains_key(*field),
            "{field} is emitted but unreviewed"
        );
    }
    assert_eq!(counts[fields::ATTRIBUTION], candidates.len());
    assert_eq!(counts[fields::CHEBI_ID], candidates.len());
}
