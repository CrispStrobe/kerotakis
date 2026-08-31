#![no_main]
use libfuzzer_sys::fuzz_target;

use kerotakis_data::{
    classify_smiles, classify_synonym, cross_check_identity, lint_promotion,
    parse_pubchem_snapshot, pubchem_candidate_licences, pubchem_eligible_fields, pubchem_import,
    pubchem_promotion_policy, review_candidates, smiles_components, PromotionLintInput,
    SnapshotManifest, StructureClass,
};

// BRD-010's external-bytes surface. A pinned PubChem snapshot is a file a
// fetcher wrote from the open internet, and the two lexers below read strings
// that upstream depositors typed by hand. Every one of them is attacker-shaped
// input to a reviewer's machine: a typed refusal is correct, a panic is a
// finding.
fuzz_target!(|data: &[u8]| {
    if let Ok(snapshot) = parse_pubchem_snapshot(data) {
        let import = pubchem_import(&snapshot);
        let policy = pubchem_promotion_policy();
        let eligible = pubchem_eligible_fields(&import.candidates, &policy);
        let allowed = pubchem_candidate_licences();

        // The eligible list this adapter derives is by construction a subset
        // of what the policy allowlists and of what the records carry, so it
        // must never be able to produce an eligibility violation.
        let manifest = SnapshotManifest {
            schema: 1,
            adapter_id: import.adapter_id.clone(),
            source_id: "fuzz".into(),
            source_revision: "fuzz".into(),
            retrieved: "fuzz".into(),
            raw_artifact: "fuzz".into(),
            record_count: u64::MAX,
            sha256: kerotakis_data::snapshot_sha256(data),
        };
        let report = lint_promotion(&PromotionLintInput {
            manifest: &manifest,
            raw_snapshot: data,
            candidates: &import.candidates,
            policy: &policy,
            allowed_runtime_licences: &allowed,
            eligible_fields: &eligible,
        });
        for violation in &report.violations {
            let kind = serde_json::to_value(violation)
                .ok()
                .and_then(|value| value["violation"].as_str().map(str::to_owned))
                .unwrap_or_default();
            assert!(
                !kind.starts_with("eligible_"),
                "the adapter's own eligible list must never be self-inconsistent: {violation:?}"
            );
        }

        let _ = review_candidates(import.candidates.clone(), &policy);
        // Recomputation oracles that answer nonsense must still produce a
        // report, and must never be read as an agreement. Both routes are
        // driven: one answers garbage, one refuses outright.
        let crosscheck = cross_check_identity(
            &import,
            |smiles| Ok(smiles.to_uppercase()),
            |_| Err("fuzz: no library here".to_owned()),
        );
        for route in [&crosscheck.from_structure, &crosscheck.from_published_inchi] {
            assert_eq!(
                crosscheck.checked,
                route.agreements
                    + route.conflicts
                    + route.not_recomputed
                    + route.no_snapshot_identity,
                "every record must land in exactly one outcome per route"
            );
        }
        // A refusing oracle can never manufacture an agreement or a conflict.
        assert_eq!(crosscheck.from_published_inchi.agreements, 0);
        assert_eq!(crosscheck.from_published_inchi.conflicts, 0);
        assert!(crosscheck.skeleton_preserving_conflicts() <= crosscheck.from_structure.conflicts);
    }

    if let Ok(text) = std::str::from_utf8(data) {
        let _ = classify_synonym(text);
        let classification = classify_smiles(text);
        match smiles_components(text) {
            Ok(components) => {
                assert!(!components.is_empty());
                assert!(
                    !matches!(classification, StructureClass::Unparsed { .. }),
                    "a SMILES that lexed must not classify as unparsed"
                );
            }
            Err(_) => assert!(
                matches!(classification, StructureClass::Unparsed { .. }),
                "a SMILES that did not lex must not be given a structure claim"
            ),
        }
        for line in text.lines() {
            let _ = classify_synonym(line);
            let _ = smiles_components(line);
        }
    }
});
