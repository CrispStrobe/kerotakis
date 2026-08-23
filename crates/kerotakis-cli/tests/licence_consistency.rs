// LIC-001: Pin the licence texts so they cannot drift apart.
//
// The LICENSE, NOTICE, and CONTRIBUTING.md must all agree on:
// - The additional permission applies only to binaries published by the
//   copyright holders
// - Curated data shipped in binaries is CC BY 4.0 or CC0 (not CC BY-SA)

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn license_additional_permission_says_copyright_holders() {
    let license =
        std::fs::read_to_string(workspace_root().join("LICENSE")).expect("LICENSE must exist");
    assert!(
        license.contains("binaries published by the copyright holders"),
        "LICENSE must limit the store permission to copyright-holder binaries"
    );
    assert!(
        license.contains("Third parties redistributing this software remain bound by the"),
        "LICENSE must state third parties remain under AGPL"
    );
}

#[test]
fn notice_limits_to_copyright_holders() {
    let notice =
        std::fs::read_to_string(workspace_root().join("NOTICE")).expect("NOTICE must exist");
    assert!(
        notice.contains("binaries published by the copyright holders"),
        "NOTICE must limit the store permission to copyright-holder binaries"
    );
}

#[test]
fn contributing_references_notice() {
    let contributing = std::fs::read_to_string(workspace_root().join("CONTRIBUTING.md"))
        .expect("CONTRIBUTING.md must exist");
    assert!(
        contributing.contains("app-store additional permission set out in NOTICE"),
        "CONTRIBUTING.md must reference NOTICE for the permission text"
    );
    assert!(
        contributing.contains("exercisable by the") && contributing.contains("copyright holders"),
        "CONTRIBUTING.md must say the grant is exercisable by copyright holders"
    );
}

#[test]
fn shipped_data_is_cc_by_or_cc0_not_by_sa() {
    let notice =
        std::fs::read_to_string(workspace_root().join("NOTICE")).expect("NOTICE must exist");
    // The curated data section must say CC BY / CC0, not CC BY-SA
    assert!(
        notice.contains("CC BY 4.0 or CC0"),
        "NOTICE curated-data section must specify CC BY 4.0 or CC0 for shipped data"
    );
    // And must NOT say shipped data is CC BY-SA
    let curated_section = notice
        .split("Curated data")
        .nth(1)
        .expect("NOTICE must have a 'Curated data' section");
    let section_end = curated_section
        .find("\n\n")
        .unwrap_or(curated_section.len());
    let curated_text = &curated_section[..section_end];
    assert!(
        !curated_text.contains("CC BY-SA 4.0, separately from the AGPL"),
        "shipped curated data must not be labelled CC BY-SA"
    );
}

#[test]
fn contributing_data_section_says_cc_by_cc0() {
    let contributing = std::fs::read_to_string(workspace_root().join("CONTRIBUTING.md"))
        .expect("CONTRIBUTING.md must exist");
    assert!(
        contributing.contains("CC BY 4.0 or CC0 1.0"),
        "CONTRIBUTING.md must specify CC BY 4.0 or CC0 for shipped data"
    );
    assert!(
        contributing.contains("CC BY-SA material") && contributing.contains("published separately"),
        "CONTRIBUTING.md must say BY-SA material is published separately"
    );
}
