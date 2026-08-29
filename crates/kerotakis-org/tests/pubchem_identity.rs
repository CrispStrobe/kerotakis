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
use kerotakis_org::inchi_validate::native_inchikey_from_smiles;

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../kerotakis-data/tests/fixtures/quarantine/pubchem-v1")
}

#[test]
fn pubchem_inchikeys_are_recomputed_by_the_official_library() {
    let raw = std::fs::read(fixture().join("raw/snapshot.json")).expect("pinned snapshot");
    let snapshot = parse_pubchem_snapshot(&raw).expect("snapshot parses");
    let import = pubchem_import(&snapshot);

    let report = cross_check_identity(&import, |smiles| {
        native_inchikey_from_smiles(smiles).map_err(|error| error.to_string())
    });

    let path = fixture().join("identity-crosscheck.json");
    let serialized = serde_json::to_string_pretty(&report).expect("report serializes") + "\n";

    if std::env::var_os("KEROTAKIS_WRITE_IDENTITY_CROSSCHECK").is_some() {
        std::fs::write(&path, &serialized).expect("write identity crosscheck");
        eprintln!(
            "wrote {} ({} agree, {} conflict, {} not recomputed)",
            path.display(),
            report.agreements,
            report.conflicts,
            report.not_recomputed
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
        report.agreements >= 50,
        "the cross-check agreed on only {} of {} records",
        report.agreements,
        report.checked
    );
}
