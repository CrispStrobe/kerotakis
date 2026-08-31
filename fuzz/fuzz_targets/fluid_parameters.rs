#![no_main]
use libfuzzer_sys::fuzz_target;

use kerotakis_data::fluid_parameters::{import_verified_snapshot, parse_source_document};
use kerotakis_data::{snapshot_sha256, SnapshotManifest};

// BRD-031's source parser and verified snapshot boundary consume untrusted
// local bytes. Malformed input must produce a typed refusal, never a panic.
fuzz_target!(|data: &[u8]| {
    let _ = parse_source_document(data);
    let manifest = SnapshotManifest {
        schema: 1,
        adapter_id: "brd031-fluid-parameters-v1".into(),
        source_id: "fuzz".into(),
        source_revision: "fuzz".into(),
        retrieved: "fuzz".into(),
        raw_artifact: "fuzz.json".into(),
        record_count: 1,
        sha256: snapshot_sha256(data),
    };
    let _ = import_verified_snapshot(&manifest, data);
});
