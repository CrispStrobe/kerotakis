//! The repository provenance policy is executable through the public CLI.

use std::io::Write;
use std::process::Command;

fn repository_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn repository_source_manifest_passes_the_live_gate() {
    let root = repository_root();
    let out = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args([
            "provenance",
            "lint",
            "--manifest",
            root.join("provenance/sources.toml").to_str().unwrap(),
            "--root",
            root.to_str().unwrap(),
        ])
        .output()
        .expect("kero provenance lint runs");

    assert!(
        out.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    // BRD-011 added the ChEBI identity slice (CC BY 4.0, release 253) in the
    // quarantine lane, so the source count rises but the distributed count
    // does not: a quarantined snapshot ships nowhere.
    assert!(stdout.contains("9 sources valid"), "{stdout}");
    assert!(stdout.contains("8 distributed"), "{stdout}");
}

#[test]
fn command_rejects_a_non_allowlisted_runtime_source() {
    let root = repository_root();
    let temp = std::env::temp_dir().join(format!("kero-provenance-{}", std::process::id()));
    std::fs::create_dir_all(&temp).unwrap();
    let path = temp.join("sources.toml");
    let mut file = std::fs::File::create(&path).unwrap();
    writeln!(
        file,
        r#"schema = 1
policy = "store-permissive-v1"
scope = "integration test"

[[source]]
id = "share-alike-data"
name = "ShareAlike data"
kind = "data"
lane = "runtime-data"
decision = "approved"
licence = "CC-BY-SA-4.0"
origin = "https://example.invalid/data"
terms = "https://example.invalid/terms"
copyright = "Example"
retrieved = "2026-08-20"
attribution = "Example"
paths = ["Cargo.toml"]
allowed_outputs = ["runtime-data"]
targets = ["web"]
reviewer = "test"

[[source.checksum]]
path = "Cargo.toml"
sha256 = "f4f979ec38b2f5b113a223eee6b4e3ce0a70838e78735ca69cdf6871c02556e4""#
    )
    .unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args([
            "provenance",
            "lint",
            "--manifest",
            path.to_str().unwrap(),
            "--root",
            root.to_str().unwrap(),
        ])
        .output()
        .expect("kero provenance lint runs");

    assert!(!out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("CC-BY-SA-4.0"), "{stderr}");
    assert!(stderr.contains("not directly includable"), "{stderr}");
    std::fs::remove_dir_all(temp).ok();
}
