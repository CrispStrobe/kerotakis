//! BRD-010: the PubChem adapter, end to end over the pinned 100-record
//! fixture.
//!
//! Every assertion here is about the *boundary*: what the adapter refuses,
//! what it reports rather than resolves, and the fact that a promotion dry run
//! over the fixture writes nothing into the runtime registry.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use kerotakis_data::*;
use serde_json::{json, Value};

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/quarantine/pubchem-v1")
}

fn raw_snapshot() -> Vec<u8> {
    std::fs::read(fixture().join("raw/snapshot.json")).expect("pinned snapshot")
}

fn manifest() -> SnapshotManifest {
    serde_json::from_slice(&std::fs::read(fixture().join("manifest.json")).expect("manifest"))
        .expect("manifest parses")
}

fn snapshot() -> PubchemSnapshot {
    parse_pubchem_snapshot(&raw_snapshot()).expect("snapshot parses")
}

fn import() -> PubchemImport {
    pubchem_import(&snapshot())
}

// ---------------------------------------------------------------------------
// The snapshot is pinned, and the fixture rebuilds from it byte for byte
// ---------------------------------------------------------------------------

#[test]
fn manifest_pins_the_snapshot_bytes() {
    let raw = raw_snapshot();
    let manifest = manifest();
    manifest.verify(&raw).expect("the pinned snapshot verifies");
    assert_eq!(manifest.adapter_id, PUBCHEM_ADAPTER_ID);
    assert_eq!(manifest.record_count, 100, "BRD-010 asks for 100 records");
}

#[test]
fn a_tampered_snapshot_is_refused() {
    let mut raw = raw_snapshot();
    raw.push(b' ');
    match manifest().verify(&raw) {
        Err(AdapterError::ChecksumMismatch { expected, actual }) => {
            assert_ne!(expected, actual);
        }
        other => panic!("a tampered snapshot must not verify: {other:?}"),
    }
}

#[test]
fn candidates_rebuild_byte_identically_from_the_snapshot() {
    let rebuilt = canonical_quarantine_bytes(import().candidates).expect("canonical bytes");
    let checked_in = std::fs::read(fixture().join("candidates.json")).expect("candidates.json");
    assert_eq!(
        String::from_utf8_lossy(&rebuilt),
        String::from_utf8_lossy(&checked_in),
        "the checked-in candidate fixture is no longer what the adapter derives \
         from the pinned snapshot; regenerate it in the same commit and say why"
    );
}

#[test]
fn the_checked_in_policy_and_eligible_lists_match_the_code() {
    let policy: PromotionPolicy =
        serde_json::from_slice(&std::fs::read(fixture().join("policy.json")).expect("policy"))
            .expect("policy parses");
    assert_eq!(policy, pubchem_promotion_policy());

    let eligible: Vec<EligibleFieldList> =
        serde_json::from_slice(&std::fs::read(fixture().join("eligible.json")).expect("eligible"))
            .expect("eligible parses");
    assert_eq!(
        eligible,
        pubchem_eligible_fields(&import().candidates, &policy)
    );
}

#[test]
fn a_snapshot_from_another_adapter_or_schema_is_refused() {
    let mut body: Value = serde_json::from_slice(&raw_snapshot()).unwrap();
    body["adapter_id"] = json!("chebi-v1");
    assert!(matches!(
        parse_pubchem_snapshot(&serde_json::to_vec(&body).unwrap()),
        Err(PubchemError::AdapterMismatch { .. })
    ));

    let mut body: Value = serde_json::from_slice(&raw_snapshot()).unwrap();
    body["schema"] = json!(99);
    assert!(matches!(
        parse_pubchem_snapshot(&serde_json::to_vec(&body).unwrap()),
        Err(PubchemError::UnsupportedSchema { found: 99, .. })
    ));

    assert!(matches!(
        parse_pubchem_snapshot(b"not json at all"),
        Err(PubchemError::Malformed(_))
    ));
}

// ---------------------------------------------------------------------------
// Fixture composition — the acceptance list, asserted rather than described
// ---------------------------------------------------------------------------

fn structure_counts(import: &PubchemImport) -> BTreeMap<&'static str, usize> {
    let mut counts = BTreeMap::new();
    for record in &import.records {
        let key = match record.structure {
            StructureClass::Single => "single",
            StructureClass::Ion { .. } => "ion",
            StructureClass::Salt { .. } => "salt",
            StructureClass::Hydrate { .. } => "hydrate",
            StructureClass::Mixture { .. } => "mixture",
            StructureClass::Unparsed { .. } => "unparsed",
        };
        *counts.entry(key).or_default() += 1;
    }
    counts
}

#[test]
fn the_fixture_covers_every_class_the_acceptance_names() {
    let import = import();
    assert_eq!(import.records.len(), 100);
    let counts = structure_counts(&import);

    assert!(
        counts.get("salt").copied().unwrap_or(0) >= 15,
        "salts: {counts:?}"
    );
    assert!(
        counts.get("hydrate").copied().unwrap_or(0) >= 8,
        "hydrates: {counts:?}"
    );
    assert!(
        counts.get("mixture").copied().unwrap_or(0) >= 2,
        "mixtures: {counts:?}"
    );

    // Isotopes: records whose InChI carries an isotopic layer, or whose
    // structure spells an isotope out.
    let isotopes = import
        .candidates
        .iter()
        .filter(|candidate| {
            candidate
                .fields
                .get("standard_inchi")
                .and_then(|field| field.value.as_str())
                .is_some_and(|inchi| inchi.contains("/i"))
                || candidate
                    .fields
                    .get("isomeric_smiles")
                    .and_then(|field| field.value.as_str())
                    .is_some_and(|smiles| smiles.contains("[2H]") || smiles.contains("[3H]"))
        })
        .count();
    assert!(isotopes >= 5, "isotopically labelled records: {isotopes}");

    // Stereochemistry: pairs sharing a skeleton but differing in the
    // InChIKey's second block, which is exactly where stereo lives.
    let mut by_skeleton: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for record in &import.records {
        if let Some(key) = &record.inchikey {
            if let Some((skeleton, rest)) = key.split_once('-') {
                by_skeleton
                    .entry(skeleton.to_owned())
                    .or_default()
                    .insert(rest.to_owned());
            }
        }
    }
    let stereo_pairs = by_skeleton
        .values()
        .filter(|variants| variants.len() > 1)
        .count();
    assert!(
        stereo_pairs >= 4,
        "stereochemistry pairs sharing a skeleton: {stereo_pairs}"
    );

    // Conflicting synonyms across records.
    assert!(
        import.synonym_conflicts.len() >= 10,
        "synonyms claimed by more than one record: {}",
        import.synonym_conflicts.len()
    );
}

#[test]
fn a_mixture_returned_for_a_name_is_reported_not_taken() {
    let import = import();
    let mixtures: Vec<&PubchemFinding> = import
        .findings
        .iter()
        .filter(|finding| matches!(finding, PubchemFinding::MixtureRecord { .. }))
        .collect();
    assert!(
        mixtures.len() >= 2,
        "the fixture must contain reported mixtures: {mixtures:?}"
    );
    // Each one arrived because a seed *name* was resolved, which is the
    // hazard: a name is not a promise of a single substance.
    for finding in &mixtures {
        if let PubchemFinding::MixtureRecord {
            resolved_from,
            components,
            ..
        } = finding
        {
            assert!(!resolved_from.is_empty(), "{finding:?}");
            assert!(*components >= 2, "{finding:?}");
        }
    }

    // And a name that denotes a mixture in the world but that PubChem answers
    // with a single pure compound is visible too, because two different seed
    // names landed on one record.
    assert!(
        import
            .findings
            .iter()
            .any(|finding| matches!(finding, PubchemFinding::SharedNameResolution { .. })),
        "several names resolving to one record must be reported"
    );
}

// ---------------------------------------------------------------------------
// The field allowlist: every rejection class, demonstrated
// ---------------------------------------------------------------------------

fn reviews() -> Vec<PromotionReview> {
    review_candidates(import().candidates, &pubchem_promotion_policy()).reviews
}

fn rejected_fields(reviews: &[PromotionReview]) -> BTreeMap<String, usize> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for review in reviews {
        for rejection in &review.rejected {
            *counts.entry(rejection.field.clone()).or_default() += 1;
        }
    }
    counts
}

#[test]
fn cas_only_identifiers_never_cross_the_promotion_boundary() {
    let reviews = reviews();
    let rejected = rejected_fields(&reviews);
    assert!(
        rejected.get("cas_registry_numbers").copied().unwrap_or(0) >= 50,
        "CAS numbers must be rejected on most records: {rejected:?}"
    );
    for review in &reviews {
        assert!(
            review
                .rejected
                .iter()
                .filter(|rejection| rejection.field == "cas_registry_numbers")
                .all(|rejection| rejection.reason == FieldRejectionReason::UnallowlistedField),
            "CAS must be refused at the field allowlist, with that reason"
        );
        for accepted in review.accepted.values() {
            assert!(
                !accepted.source_field.contains("Synonym"),
                "a depositor synonym reached the accepted set on {}",
                review.external_record_id
            );
        }
        assert!(
            !review.accepted.contains_key("cas_rn"),
            "no CAS target may ever be accepted"
        );
    }
}

#[test]
fn depositor_material_and_database_descriptors_are_rejected_by_name() {
    let rejected = rejected_fields(&reviews());
    for field in [
        "depositor_supplied_synonyms",
        "registry_identifiers",
        "exact_mass",
        "tpsa",
        "complexity",
        "hbond_donor_count",
        "hbond_acceptor_count",
        "rotatable_bond_count",
        "heavy_atom_count",
    ] {
        assert!(
            rejected.get(field).copied().unwrap_or(0) > 0,
            "{field} must appear as a rejection: {rejected:?}"
        );
    }
    assert!(
        rejected.get("xlogp").copied().unwrap_or(0) > 0,
        "XLogP is present on part of the fixture and must be rejected there"
    );
}

#[test]
fn no_depositor_annotation_is_allowlisted() {
    let reviews = reviews();
    let mut annotation_rejections = 0;
    for review in &reviews {
        for rejection in &review.rejected {
            if rejection.field.starts_with("boiling_point__") {
                assert_eq!(
                    rejection.reason,
                    FieldRejectionReason::UnallowlistedField,
                    "{}",
                    rejection.field
                );
                annotation_rejections += 1;
            }
        }
        assert!(
            !review.accepted.contains_key("boiling_point"),
            "no experimental boiling point is promotable from this snapshot"
        );
    }
    assert!(
        annotation_rejections >= 40,
        "the fixture carries depositor annotations to refuse: {annotation_rejections}"
    );
}

#[test]
fn prose_annotations_are_never_parsed_into_numbers() {
    for candidate in &import().candidates {
        for (name, field) in &candidate.fields {
            if !name.starts_with("boiling_point__") {
                continue;
            }
            if field.value.is_string() {
                assert!(
                    field.unit.is_none(),
                    "{name} on {} carries prose but claims a unit",
                    candidate.external_record_id
                );
            } else {
                assert!(
                    field.unit.is_some(),
                    "{name} on {} is numeric but carries no unit",
                    candidate.external_record_id
                );
            }
        }
    }
}

#[test]
fn the_annotation_source_licence_travels_with_the_value() {
    let import = import();
    let sources: BTreeSet<String> = import
        .findings
        .iter()
        .filter_map(|finding| match finding {
            PubchemFinding::AnnotationSourceNotCleared { source_name, .. } => {
                Some(source_name.clone())
            }
            _ => None,
        })
        .collect();
    assert!(
        sources.len() >= 5,
        "several upstream annotation sources must be named: {sources:?}"
    );

    // The one source whose licence note is unambiguous keeps its SPDX id; the
    // rest keep a LicenseRef naming them, so "not cleared" is legible.
    let mut saw_cc_by = false;
    let mut saw_licence_ref = false;
    for candidate in &import.candidates {
        for (name, field) in &candidate.fields {
            if !name.starts_with("boiling_point__") {
                continue;
            }
            if field.licence == "CC-BY-4.0" {
                saw_cc_by = true;
            } else {
                assert!(
                    field.licence.starts_with("LicenseRef-PubChem-Annotation-"),
                    "unexpected annotation licence {}",
                    field.licence
                );
                saw_licence_ref = true;
            }
        }
    }
    assert!(saw_cc_by && saw_licence_ref);
}

/// A cleared source that delivers a real quantity does promote — the machinery
/// works, it is the pinned data that has nothing eligible in it. This candidate
/// is **synthetic**, built here and marked as such; it is not PubChem data.
#[test]
fn a_cleared_structured_annotation_would_be_accepted() {
    let mut fields = BTreeMap::new();
    fields.insert(
        "boiling_point__ilo_who_international_chemical_safety_cards_icscs".to_owned(),
        CandidateField::new(
            json!(78.24),
            "synthetic: ILO-WHO ICSC shaped annotation",
            "CC-BY-4.0",
        )
        .with_unit("°C"),
    );
    let candidate = QuarantinedCandidate {
        adapter_id: PUBCHEM_ADAPTER_ID.to_owned(),
        source_record_id: "synthetic-record".to_owned(),
        external_record_id: "SYNTHETIC-ICSC".to_owned(),
        identity_key: None,
        fields,
    };
    let mut policy = pubchem_promotion_policy();
    policy.fields.insert(
        "boiling_point__ilo_who_international_chemical_safety_cards_icscs".to_owned(),
        RuntimeFieldPolicy::new("boiling_point", ["CC-BY-4.0"])
            .with_dimension(Dimension::Temperature),
    );
    let review = review_candidate(&candidate, &policy);
    assert!(review.rejected.is_empty(), "{review:?}");
    let accepted = &review.accepted["boiling_point"];
    assert_eq!(accepted.source_unit.as_deref(), Some("°C"));
    assert_eq!(
        accepted.unit.as_ref().map(|unit| unit.symbol.as_str()),
        Some("K")
    );
    let kelvin = accepted.value.as_f64().unwrap();
    assert!((kelvin - 351.39).abs() < 1e-6, "{kelvin}");
}

#[test]
fn every_rejection_class_the_framework_defines_is_exercised() {
    let mut seen: BTreeSet<&'static str> = BTreeSet::new();
    let note = |seen: &mut BTreeSet<&'static str>, reason: &FieldRejectionReason| {
        seen.insert(match reason {
            FieldRejectionReason::UnallowlistedField => "unallowlisted_field",
            FieldRejectionReason::LicenceNotAllowed { .. } => "licence_not_allowed",
            FieldRejectionReason::MissingProvenance => "missing_provenance",
            FieldRejectionReason::TargetCollision { .. } => "target_collision",
            FieldRejectionReason::MissingUnit { .. } => "missing_unit",
            FieldRejectionReason::NonNumericQuantity { .. } => "non_numeric_quantity",
            FieldRejectionReason::UnitNotNormalized { .. } => "unit_not_normalized",
        });
    };

    // The pinned fixture supplies the allowlist refusal on its own.
    for review in reviews() {
        for rejection in &review.rejected {
            note(&mut seen, &rejection.reason);
        }
    }

    // The rest are planted, because a healthy snapshot does not contain them.
    let policy = policy_with_collision();
    for candidate in planted_injuries() {
        for rejection in &review_candidate(&candidate, &policy).rejected {
            note(&mut seen, &rejection.reason);
        }
    }

    let expected: BTreeSet<&str> = [
        "unallowlisted_field",
        "licence_not_allowed",
        "missing_provenance",
        "target_collision",
        "missing_unit",
        "non_numeric_quantity",
        "unit_not_normalized",
    ]
    .into_iter()
    .collect();
    assert_eq!(seen, expected, "unexercised rejection classes");
}

/// Candidates carrying one deliberate injury each. They are built here, never
/// checked in as if they were PubChem's answer.
fn planted_injuries() -> Vec<QuarantinedCandidate> {
    let base = |id: &str, fields: BTreeMap<String, CandidateField>| QuarantinedCandidate {
        adapter_id: PUBCHEM_ADAPTER_ID.to_owned(),
        source_record_id: "PubChem CID 962".to_owned(),
        external_record_id: id.to_owned(),
        identity_key: Some("XLYOFNOQVPJJNP-UHFFFAOYSA-N".to_owned()),
        fields,
    };
    let one = |name: &str, field: CandidateField| {
        let mut fields = BTreeMap::new();
        fields.insert(name.to_owned(), field);
        fields
    };

    let mut collision = BTreeMap::new();
    collision.insert(
        "iupac_name".to_owned(),
        CandidateField::new(
            json!("oxidane"),
            "PropertyTable.IUPACName",
            PUBCHEM_CORE_LICENCE,
        ),
    );
    // A second source field aimed at the same runtime target.
    collision.insert(
        "iupac_name_duplicate".to_owned(),
        CandidateField::new(json!("water"), "PropertyTable.Title", PUBCHEM_CORE_LICENCE),
    );

    let mut missing_provenance = one(
        "molecular_formula",
        CandidateField::new(json!("H2O"), "PropertyTable.MolecularFormula", ""),
    );
    missing_provenance.insert(
        "standard_inchikey".to_owned(),
        CandidateField::new(
            json!("XLYOFNOQVPJJNP-UHFFFAOYSA-N"),
            "",
            PUBCHEM_CORE_LICENCE,
        ),
    );

    vec![
        base(
            "PLANTED-licence",
            one(
                "molecular_formula",
                CandidateField::new(
                    json!("H2O"),
                    "PropertyTable.MolecularFormula",
                    "CC-BY-SA-4.0",
                ),
            ),
        ),
        base("PLANTED-provenance", missing_provenance),
        base("PLANTED-collision", collision),
        base(
            "PLANTED-missing-unit",
            one(
                "molar_mass",
                CandidateField::new(
                    json!(18.015),
                    "PropertyTable.MolecularWeight",
                    PUBCHEM_CORE_LICENCE,
                ),
            ),
        ),
        base(
            "PLANTED-non-numeric",
            one(
                "molar_mass",
                CandidateField::new(
                    json!("eighteen"),
                    "PropertyTable.MolecularWeight",
                    PUBCHEM_CORE_LICENCE,
                )
                .with_unit("g/mol"),
            ),
        ),
        base(
            "PLANTED-unit",
            one(
                "molar_mass",
                CandidateField::new(
                    json!(18.015),
                    "PropertyTable.MolecularWeight",
                    PUBCHEM_CORE_LICENCE,
                )
                .with_unit("smoots per fortnight"),
            ),
        ),
    ]
}

// The collision case needs a policy rule for the duplicate source field.
fn policy_with_collision() -> PromotionPolicy {
    let mut policy = pubchem_promotion_policy();
    policy.fields.insert(
        "iupac_name_duplicate".to_owned(),
        RuntimeFieldPolicy::new("iupac_name", [PUBCHEM_CORE_LICENCE]),
    );
    policy
}

// ---------------------------------------------------------------------------
// The promotion dry run
// ---------------------------------------------------------------------------

#[test]
fn the_promotion_lint_passes_for_the_eligible_fields() {
    let import = import();
    let policy = pubchem_promotion_policy();
    let eligible = pubchem_eligible_fields(&import.candidates, &policy);
    let allowed = pubchem_candidate_licences();
    let report = lint_promotion(&PromotionLintInput {
        manifest: &manifest(),
        raw_snapshot: &raw_snapshot(),
        candidates: &import.candidates,
        policy: &policy,
        allowed_runtime_licences: &allowed,
        eligible_fields: &eligible,
    });
    assert!(
        !report.refuses(),
        "the pinned fixture must pass its own dry run: {:#?}",
        report.violations
    );
    assert_eq!(report.checked_records, 100);
    assert!(report.checked_fields >= 1_000, "{report:?}");
}

#[test]
fn the_promotion_lint_refuses_the_planted_violations() {
    let import = import();
    let policy = policy_with_collision();
    let mut candidates = import.candidates.clone();
    candidates.extend(planted_injuries());
    // A duplicate key, and a candidate claiming a different adapter.
    let duplicate = candidates[0].clone();
    candidates.push(duplicate);
    let mut foreign = candidates[1].clone();
    foreign.adapter_id = "chebi-v1".to_owned();
    candidates.push(foreign);

    let mut eligible = pubchem_eligible_fields(&candidates, &policy);
    eligible.push(EligibleFieldList {
        adapter_id: PUBCHEM_ADAPTER_ID.to_owned(),
        external_record_id: "CID-does-not-exist".to_owned(),
        fields: vec!["molecular_formula".to_owned()],
    });
    eligible.push(EligibleFieldList {
        adapter_id: PUBCHEM_ADAPTER_ID.to_owned(),
        external_record_id: "CID962".to_owned(),
        fields: vec![
            "molecular_formula".to_owned(),
            "molecular_formula".to_owned(),
            "not_a_field_on_this_record".to_owned(),
            "cas_registry_numbers".to_owned(),
        ],
    });

    let allowed = pubchem_candidate_licences();
    let report = lint_promotion(&PromotionLintInput {
        manifest: &manifest(),
        raw_snapshot: &raw_snapshot(),
        candidates: &candidates,
        policy: &policy,
        allowed_runtime_licences: &allowed,
        eligible_fields: &eligible,
    });
    assert!(report.refuses());

    let kinds: BTreeSet<String> = report
        .violations
        .iter()
        .map(|violation| {
            serde_json::to_value(violation).unwrap()["violation"]
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect();
    for expected in [
        "candidates_exceed_snapshot",
        "duplicate_candidate",
        "snapshot_adapter_mismatch",
        "missing_field_provenance",
        "licence_not_allowed_for_runtime",
        "unit_not_normalized",
        "eligible_record_not_in_quarantine",
        "eligible_field_not_on_record",
        "eligible_field_not_allowlisted",
        "eligible_field_repeated",
    ] {
        assert!(kinds.contains(expected), "missing {expected}: {kinds:?}");
    }
}

#[test]
fn an_unpinned_snapshot_refuses_even_with_clean_candidates() {
    let import = import();
    let policy = pubchem_promotion_policy();
    let mut manifest = manifest();
    manifest.sha256 = "0".repeat(64);
    let allowed = pubchem_candidate_licences();
    let report = lint_promotion(&PromotionLintInput {
        manifest: &manifest,
        raw_snapshot: &raw_snapshot(),
        candidates: &import.candidates,
        policy: &policy,
        allowed_runtime_licences: &allowed,
        eligible_fields: &[],
    });
    assert!(report.refuses());
}

#[test]
fn the_runtime_licence_lane_is_not_widened_by_this_task() {
    assert!(
        !default_runtime_data_licences().contains(PUBCHEM_CORE_LICENCE),
        "BRD-010 must not add PubChem's LicenseRef to the shipped-data lane"
    );
    assert!(pubchem_candidate_licences().contains(PUBCHEM_CORE_LICENCE));
    assert!(default_runtime_data_licences().is_subset(&pubchem_candidate_licences()));
}

#[test]
fn nothing_from_this_adapter_reached_the_runtime_registry() {
    let registry =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/registry/registry-source-v1.json");
    let text = std::fs::read_to_string(registry).expect("runtime registry");
    assert!(
        !text.contains(PUBCHEM_ADAPTER_ID),
        "BRD-010 produces candidates only; the registry must not mention {PUBCHEM_ADAPTER_ID}"
    );
}

// ---------------------------------------------------------------------------
// Identity cross-check
// ---------------------------------------------------------------------------

#[test]
fn the_identity_cross_check_is_pinned_and_self_consistent() {
    let pinned: IdentityCrossCheckReport = serde_json::from_slice(
        &std::fs::read(fixture().join("identity-crosscheck.json")).expect("identity crosscheck"),
    )
    .expect("identity crosscheck parses");

    let import = import();
    assert_eq!(pinned.checked, import.records.len());
    for route in [&pinned.from_published_inchi, &pinned.from_structure] {
        assert_eq!(
            route.agreements + route.conflicts + route.not_recomputed + route.no_snapshot_identity,
            pinned.checked,
            "every record must land in exactly one outcome per route"
        );
    }

    // The route with nothing of ours in it must be clean: every record's
    // published key hashes from its own published InChI. A conflict here would
    // be an upstream identity fault, not a toolchain limitation.
    assert_eq!(pinned.from_published_inchi.conflicts, 0);
    assert_eq!(pinned.from_published_inchi.agreements, pinned.checked);

    // Replay the report through the same function using the pinned answers as
    // the oracle: the bookkeeping, not the chemistry, is what this asserts.
    // Keying by structure only works because no two records share one.
    let distinct: BTreeSet<&str> = pinned
        .records
        .iter()
        .map(|record| record.smiles.as_str())
        .collect();
    assert_eq!(distinct.len(), pinned.records.len());

    let by_smiles: BTreeMap<&str, &IdentityCrossCheck> = pinned
        .records
        .iter()
        .map(|record| (record.smiles.as_str(), record))
        .collect();
    let by_inchi: BTreeMap<&str, &IdentityCrossCheck> = pinned
        .records
        .iter()
        .map(|record| (record.snapshot_inchi.as_str(), record))
        .collect();
    let replay = |record: Option<&&IdentityCrossCheck>,
                  outcome: fn(&IdentityCrossCheck) -> &IdentityOutcome| {
        let Some(record) = record else {
            return Err("no pinned answer".to_owned());
        };
        match outcome(record) {
            IdentityOutcome::Agrees => Ok(record.snapshot_inchikey.clone()),
            IdentityOutcome::Conflicts { recomputed } => Ok(recomputed.clone()),
            IdentityOutcome::NotRecomputed { detail } => Err(detail.clone()),
            IdentityOutcome::NoSnapshotIdentity => Err("no snapshot identity".to_owned()),
        }
    };
    let replayed = cross_check_identity(
        &import,
        |smiles| replay(by_smiles.get(smiles), |record| &record.from_structure),
        |inchi| replay(by_inchi.get(inchi), |record| &record.from_published_inchi),
    );
    assert_eq!(replayed, pinned, "the pinned report is not reproducible");

    // Conflicts travel into BRD-003's own conflict shape rather than being
    // resolved here. Both routes are represented.
    assert_eq!(
        pinned.identity_conflicts().len(),
        pinned.from_published_inchi.conflicts + pinned.from_structure.conflicts
    );

    // The check has to actually have run on a real slice of the fixture.
    assert!(
        pinned.from_structure.agreements >= 50,
        "the official library agreed on too few records to be a check: {:?}",
        pinned.from_structure
    );
}

/// The structure round-trip's conflicts are triaged, not hand-waved: the
/// fixture pins how many of them keep the record's connectivity block, which
/// is the signature of a lost stereo/isotope layer rather than a different
/// molecule. If that number moves, the bridge's behaviour moved.
#[test]
fn structure_conflicts_are_triaged_by_skeleton() {
    let pinned: IdentityCrossCheckReport = serde_json::from_slice(
        &std::fs::read(fixture().join("identity-crosscheck.json")).expect("identity crosscheck"),
    )
    .expect("identity crosscheck parses");

    let skeleton_preserving = pinned.skeleton_preserving_conflicts();
    assert!(
        skeleton_preserving <= pinned.from_structure.conflicts,
        "triage cannot exceed the conflicts it explains"
    );
    // Every skeleton-preserving conflict must really be stereo/isotope: the
    // recomputed key's second block is the "no stereo, no isotope" hash.
    for record in &pinned.records {
        if let IdentityOutcome::Conflicts { recomputed } = &record.from_structure {
            let same_skeleton =
                recomputed.split('-').next() == record.snapshot_inchikey.split('-').next();
            if same_skeleton {
                assert!(
                    recomputed.contains("-UHFFFAOYSA-"),
                    "{} keeps its skeleton but differs for some reason other than \
                     a dropped stereo/isotope layer: {} -> {}",
                    record.external_record_id,
                    record.snapshot_inchikey,
                    recomputed
                );
            }
        }
    }
}

#[test]
fn a_recomputation_failure_is_not_an_agreement() {
    let import = import();
    let report = cross_check_identity(
        &import,
        |_| Err("toolchain declined".to_owned()),
        |_| Err("toolchain declined".to_owned()),
    );
    for route in [report.from_structure, report.from_published_inchi] {
        assert_eq!(route.agreements, 0);
        assert_eq!(route.conflicts, 0);
        assert_eq!(route.not_recomputed, report.checked);
    }
    assert!(report.identity_conflicts().is_empty());
}

#[test]
fn a_disagreeing_recomputation_is_a_conflict_not_a_correction() {
    let import = import();
    let report = cross_check_identity(
        &import,
        |_| Ok("AAAAAAAAAAAAAA-BBBBBBBBBB-C".to_owned()),
        |_| Ok("AAAAAAAAAAAAAA-BBBBBBBBBB-C".to_owned()),
    );
    assert_eq!(report.from_structure.agreements, 0);
    assert_eq!(report.from_structure.conflicts, report.checked);
    // Both routes disagree, so every record contributes two conflict rows.
    let conflicts = report.identity_conflicts();
    assert_eq!(conflicts.len(), report.checked * 2);
    assert_eq!(
        conflicts[0].differing_fields,
        vec!["standard_inchikey/from_published_inchi".to_owned()]
    );
    assert_eq!(
        conflicts[1].differing_fields,
        vec!["standard_inchikey/from_structure".to_owned()]
    );
    // None of them is silently applied to the candidate.
    assert!(report
        .records
        .iter()
        .all(|record| record.snapshot_inchikey != "AAAAAAAAAAAAAA-BBBBBBBBBB-C"));
}

// ---------------------------------------------------------------------------
// The two small lexers
// ---------------------------------------------------------------------------

#[test]
fn smiles_components_reads_charges_without_guessing() {
    let components = smiles_components("[Na+].[Cl-]").unwrap();
    assert_eq!(components.len(), 2);
    assert_eq!(components[0].charge, 1);
    assert_eq!(components[1].charge, -1);

    let components = smiles_components("[Cu+2].[O-]S(=O)(=O)[O-]").unwrap();
    assert_eq!(components[0].charge, 2);
    assert_eq!(components[1].charge, -2);

    // The older doubled-sign spelling.
    assert_eq!(smiles_components("[Ca++]").unwrap()[0].charge, 2);

    // A `.` inside brackets is not a separator, and unbalanced input is a
    // typed error rather than a silent split.
    assert!(smiles_components("[Na+].").is_err());
    assert!(smiles_components("").is_err());
    assert!(smiles_components("C(C").is_err());
    assert!(smiles_components("C)C").is_err());
    assert!(smiles_components("[Na+").is_err());
}

#[test]
fn structures_are_classified_not_assumed() {
    assert_eq!(classify_smiles("O"), StructureClass::Single);
    assert_eq!(classify_smiles("[Na+]"), StructureClass::Ion { charge: 1 });
    assert_eq!(
        classify_smiles("[Na+].[Cl-]"),
        StructureClass::Salt { components: 2 }
    );
    assert_eq!(
        classify_smiles("O.O.O.O.O.[O-]S(=O)(=O)[O-].[Cu+2]"),
        StructureClass::Hydrate {
            waters: 5,
            components: 7
        }
    );
    assert_eq!(
        classify_smiles("[Cu].[Zn].[Pb]"),
        StructureClass::Mixture { components: 3 }
    );
    assert_eq!(
        classify_smiles("[N+](=O)(O)[O-].Cl.Cl.Cl"),
        StructureClass::Mixture { components: 4 }
    );
    assert!(matches!(
        classify_smiles("[[["),
        StructureClass::Unparsed { .. }
    ));
}

#[test]
fn cas_numbers_are_recognised_by_their_check_digit() {
    for cas in ["7732-18-5", "64-17-5", "50-78-2", "7647-14-5"] {
        assert_eq!(
            classify_synonym(cas),
            SynonymClass::CasRegistryNumber,
            "{cas}"
        );
    }
    // Right shape, wrong check digit: not claimed as a CAS number.
    assert_ne!(
        classify_synonym("7732-18-4"),
        SynonymClass::CasRegistryNumber
    );
    assert_eq!(
        classify_synonym("CHEBI:15377"),
        SynonymClass::RegistryIdentifier {
            scheme: "chebi".to_owned()
        }
    );
    assert_eq!(
        classify_synonym("XLYOFNOQVPJJNP-UHFFFAOYSA-N"),
        SynonymClass::RegistryIdentifier {
            scheme: "inchikey".to_owned()
        }
    );
    assert_eq!(
        classify_synonym("Distilled water"),
        SynonymClass::DepositorSuppliedName
    );
}

#[test]
fn only_pubchems_own_names_are_offered_for_promotion() {
    let reviews = reviews();
    let mut checked = 0;
    for review in &reviews {
        let Some(accepted) = review.accepted.get("synonyms") else {
            continue;
        };
        checked += 1;
        assert_eq!(
            accepted.source_field, "PropertyTable.Title + PropertyTable.IUPACName",
            "the promotable name set is PubChem's own layer only"
        );
        let names = accepted.value.as_array().expect("array of names");
        assert!(!names.is_empty() && names.len() <= 2);
    }
    assert_eq!(checked, 100);
}
