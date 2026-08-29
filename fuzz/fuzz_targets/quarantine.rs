#![no_main]
use libfuzzer_sys::fuzz_target;

use kerotakis_data::{
    canonical_quarantine_bytes, default_runtime_data_licences, diff_quarantine, lint_promotion,
    normalize_quantity, normalize_unit, review_candidates, EligibleFieldList, PromotionLintInput,
    PromotionPolicy, QuarantinedCandidate, SnapshotManifest,
};

// BRD-003's external-bytes surface: snapshot manifests, candidate fixtures,
// promotion policies, eligible-field lists, and the unit spellings a source
// writes by hand. All five are read from files an adapter fetched, so every
// one of them is attacker-shaped input to the reviewer's own machine.
// A typed refusal is the correct outcome; a panic is a finding.
fuzz_target!(|data: &[u8]| {
    if let Ok(manifest) = serde_json::from_slice::<SnapshotManifest>(data) {
        // Verification hashes the bytes and compares; it must never panic on
        // a manifest that claims anything at all.
        let _ = manifest.verify(data);
        let _ = manifest.verify(b"");
    }

    if let Ok(candidates) = serde_json::from_slice::<Vec<QuarantinedCandidate>>(data) {
        let _ = diff_quarantine(&candidates, &[]);
        let _ = diff_quarantine(&[], &candidates);
        let _ = diff_quarantine(&candidates, &candidates);

        if let Ok(bytes) = canonical_quarantine_bytes(candidates.clone()) {
            // Canonical serialization is what "rebuilding from a pinned
            // snapshot is byte-identical" rests on, so re-reading it must
            // reproduce exactly the same bytes.
            let reparsed = serde_json::from_slice::<Vec<QuarantinedCandidate>>(&bytes)
                .expect("canonical quarantine bytes must parse back");
            let again = canonical_quarantine_bytes(reparsed)
                .expect("canonical bytes must re-serialize");
            assert_eq!(bytes, again, "canonical quarantine bytes are not stable");
        }

        let policy = PromotionPolicy::default();
        let _ = review_candidates(candidates.clone(), &policy);

        let manifest = SnapshotManifest {
            schema: 1,
            adapter_id: "fuzz".into(),
            source_id: "fuzz".into(),
            source_revision: "fuzz".into(),
            retrieved: "fuzz".into(),
            raw_artifact: "fuzz".into(),
            record_count: 1,
            sha256: String::new(),
        };
        let eligible: Vec<EligibleFieldList> = candidates
            .iter()
            .map(|candidate| EligibleFieldList {
                adapter_id: candidate.adapter_id.clone(),
                external_record_id: candidate.external_record_id.clone(),
                fields: candidate.fields.keys().cloned().collect(),
            })
            .collect();
        let allowed = default_runtime_data_licences();
        let report = lint_promotion(&PromotionLintInput {
            manifest: &manifest,
            raw_snapshot: data,
            candidates: &candidates,
            policy: &policy,
            allowed_runtime_licences: &allowed,
            eligible_fields: &eligible,
        });
        // The manifest above pins an empty checksum, so the lint must always
        // refuse: a promotion gate that can be talked into silence is the
        // whole risk this target exists to rule out.
        assert!(report.refuses(), "an unpinned snapshot was not refused");
    }

    if let Ok(policy) = serde_json::from_slice::<PromotionPolicy>(data) {
        let _ = review_candidates(Vec::new(), &policy);
    }

    if let Ok(text) = std::str::from_utf8(data) {
        let _ = normalize_unit(text);
        for value in [0.0_f64, -1.0, 1e308, f64::MIN_POSITIVE] {
            let _ = normalize_quantity(value, text);
        }
        // Sources write "1.03 g/cm3" as one string more often than they
        // should; splitting it is the caller's job, but neither half may
        // panic the normalizer.
        if let Some((number, unit)) = text.split_once(' ') {
            if let Ok(value) = number.parse::<f64>() {
                let _ = normalize_quantity(value, unit);
            }
        }
    }
});
