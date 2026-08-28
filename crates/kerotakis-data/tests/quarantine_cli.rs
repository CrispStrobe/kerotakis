use std::path::{Path, PathBuf};
use std::process::Command;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/quarantine/synthetic-v1")
        .join(name)
}

fn command(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_quarantine-review"))
        .args(args)
        .output()
        .expect("run quarantine-review")
}

#[test]
fn pinned_fixture_verifies_and_reviews_offline() {
    let manifest = fixture("manifest.json");
    let raw = fixture("raw/snapshot.json");
    let verified = command(&["verify", manifest.to_str().unwrap(), raw.to_str().unwrap()]);
    assert!(verified.status.success());
    let verified_json: serde_json::Value = serde_json::from_slice(&verified.stdout).unwrap();
    assert_eq!(verified_json["verified"], true);

    let candidates = fixture("candidates-new.json");
    let policy = fixture("policy.json");
    let reviewed = command(&[
        "review",
        candidates.to_str().unwrap(),
        policy.to_str().unwrap(),
    ]);
    assert!(reviewed.status.success());
    let review_json: serde_json::Value = serde_json::from_slice(&reviewed.stdout).unwrap();
    assert_eq!(
        review_json["reviews"][0]["accepted"]["formula"]["value"],
        "H2O"
    );
}

#[test]
fn refresh_diff_is_machine_readable_and_does_not_write_a_pack() {
    let old = fixture("candidates-old.json");
    let new = fixture("candidates-new.json");
    let output = command(&["diff", old.to_str().unwrap(), new.to_str().unwrap()]);
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["changed_records"][0]["fields"][0]["change"], "added");
    assert_eq!(json["changed_records"][0]["fields"][0]["field"], "formula");
}

#[test]
fn malformed_or_unpinned_input_fails_closed() {
    let manifest = fixture("manifest.json");
    let candidates = fixture("candidates-new.json");
    let output = command(&[
        "verify",
        manifest.to_str().unwrap(),
        candidates.to_str().unwrap(),
    ]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("checksum mismatch"));
}
