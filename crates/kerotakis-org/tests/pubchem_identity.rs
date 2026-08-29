//! BRD-010: recompute the pinned PubChem fixture's Standard InChIKeys with the
//! official IUPAC InChI library and pin the per-record verdict.
//!
//! The recomputation lives here, not in `kerotakis-data`, because the official
//! library is a C dependency behind `--features native-inchi` and the data
//! crate deliberately stays dependency-free. The *verdict* is checked in as
//! `identity-crosscheck.json` next to the fixture, so:
//!
//! * `cargo test -p kerotakis-data` reads the pinned report in every build,
//!   including the ones without a C toolchain, and
//! * this test — run by `tools/preflight.sh` alongside CAP-13's own identity
//!   gate — is what proves the pinned report really came from the reference
//!   implementation.
//!
//! Regenerate after a fixture refresh with:
//!
//! ```text
//! KEROTAKIS_WRITE_IDENTITY_CROSSCHECK=1 \
//!   cargo test -p kerotakis-org --features native-inchi --test pubchem_identity
//! ```
//!
//! A disagreement is never repaired here. It is recorded as
//! [`IdentityOutcome::Conflicts`] and surfaces through
//! `IdentityCrossCheckReport::identity_conflicts`, which is BRD-003's own
//! conflict shape.

#![cfg(feature = "native-inchi")]

use std::path::{Path, PathBuf};

use kerotakis_data::{
    cross_check_identity, parse_pubchem_snapshot, pubchem_import, IdentityCrossCheckReport,
};
use kerotakis_org::inchi_validate::{native_inchikey_from_inchi, native_inchikey_from_smiles};

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../kerotakis-data/tests/fixtures/quarantine/pubchem-v1")
}

#[test]
fn pubchem_inchikeys_are_recomputed_by_the_official_library() {
    let raw = std::fs::read(fixture().join("raw/snapshot.json")).expect("pinned snapshot");
    let snapshot = parse_pubchem_snapshot(&raw).expect("snapshot parses");
    let import = pubchem_import(&snapshot);

    let report = cross_check_identity(
        &import,
        |smiles| native_inchikey_from_smiles(smiles).map_err(|error| error.to_string()),
        |inchi| native_inchikey_from_inchi(inchi).map_err(|error| error.to_string()),
    );

    let path = fixture().join("identity-crosscheck.json");
    let serialized = serde_json::to_string_pretty(&report).expect("report serializes") + "\n";

    if std::env::var_os("KEROTAKIS_WRITE_IDENTITY_CROSSCHECK").is_some() {
        std::fs::write(&path, &serialized).expect("write identity crosscheck");
        eprintln!(
            "wrote {}\n  published InChI re-keyed: {} agree, {} conflict, {} not recomputed\n  \
             structure round-trip:     {} agree, {} conflict, {} not recomputed \
             ({} of the conflicts keep the record's skeleton block)",
            path.display(),
            report.from_published_inchi.agreements,
            report.from_published_inchi.conflicts,
            report.from_published_inchi.not_recomputed,
            report.from_structure.agreements,
            report.from_structure.conflicts,
            report.from_structure.not_recomputed,
            report.skeleton_preserving_conflicts(),
        );
        return;
    }

    let pinned: IdentityCrossCheckReport =
        serde_json::from_slice(&std::fs::read(&path).expect("pinned identity crosscheck"))
            .expect("pinned report parses");

    assert_eq!(
        report, pinned,
        "the official InChI library no longer reproduces the pinned identity \
         cross-check. If the fixture was refreshed, regenerate with \
         KEROTAKIS_WRITE_IDENTITY_CROSSCHECK=1 and explain the movement in the \
         same commit; if it was not, this is an identity regression."
    );

    // The check has to have actually run: a report of a hundred "could not
    // recompute" rows would satisfy equality while checking nothing.
    assert!(
        report.from_structure.agreements >= 50,
        "the structure round-trip agreed on only {} of {} records",
        report.from_structure.agreements,
        report.checked
    );

    // Every record's own published key must hash from its own published
    // InChI. This one has nothing of ours in the path, so a failure here is a
    // statement about the snapshot and must not be waved through.
    assert_eq!(
        report.from_published_inchi.conflicts,
        0,
        "a PubChem record's published InChIKey does not hash from its own \
         published Standard InChI: {:#?}",
        report
            .records
            .iter()
            .filter(|record| matches!(
                record.from_published_inchi,
                kerotakis_data::IdentityOutcome::Conflicts { .. }
            ))
            .collect::<Vec<_>>()
    );
    assert_eq!(report.from_published_inchi.agreements, report.checked);
}
