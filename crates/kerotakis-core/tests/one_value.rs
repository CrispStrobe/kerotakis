//! One value for water's molar mass, and one place it comes from.
//!
//! PR #608 attached a propagated CIAAW interval to `molar-mass/water` and
//! then found that the bench never read the record the band was attached to:
//! water's molar mass lived as a Rust literal in fourteen sites across seven
//! files, in three spellings that hid **two different numbers**. 18.015 is
//! the registry's own record and the sum of the IUPAC/CIAAW 2021 conventional
//! atomic weights (2 × 1.008 + 15.999); 18.01528 is 2 × 1.00794 + 15.9994,
//! the pre-2009 values, and it sat in `constants.rs` under a comment claiming
//! the 2021 ones.
//!
//! Both lie inside the interval, so neither was a typo and the disagreement
//! was invisible until something computed what the digits were allowed to be.
//! That is exactly the shape of defect this project has paid for before —
//! one quantity, two derivations, nothing keeping them equal.
//!
//! These tests are the mechanism that keeps them equal.

use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The generated constant IS the registry record, and the kg/mol form is the
/// same number converted rather than re-entered.
///
/// `build.rs` writes both out of `data/registry/registry-source-v1.json`, so
/// this cannot fail without the generator having gone wrong — which is the
/// point: it pins the generator, not the arithmetic.
#[test]
fn the_engines_molar_mass_of_water_is_the_registrys() {
    let record = kerotakis_core::species::lookup_key("water")
        .expect("water is in the registry")
        .molar_mass;
    assert_eq!(
        record,
        kerotakis_core::constants::WATER_MOLAR_MASS_G_PER_MOL,
        "the generated constant and the registry table disagree"
    );
    assert_eq!(
        kerotakis_core::constants::WATER_MOLAR_MASS_KG_PER_MOL,
        record / 1000.0,
        "kg/mol is the g/mol value converted, not a second literal"
    );
}

/// The two latent heats are their records too.
///
/// These had no registry record at all until 2026-09-15, which is why the
/// accuracy corpus could not quantify the term that dominates every band in
/// the colligative family.
#[test]
fn the_solvents_latent_heats_are_registry_records() {
    let registry: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(workspace_root().join("data/registry/registry-source-v1.json"))
            .expect("the shipped registry"),
    )
    .expect("the registry parses");
    let value = |id: &str| -> f64 {
        registry["phase_thermodynamics"]
            .as_array()
            .expect("phase thermodynamics")
            .iter()
            .find(|record| record["id"].as_str() == Some(id))
            .unwrap_or_else(|| panic!("`{id}` is not a record in the shipped registry"))
            ["quantity"]["value"]
            .as_f64()
            .expect("a value")
    };
    assert_eq!(
        value("enthalpy-of-fusion/water"),
        kerotakis_core::states::WATER_H_FUS
    );
    assert_eq!(
        value("enthalpy-of-vaporisation/water"),
        kerotakis_core::states::WATER_H_VAP
    );
}

/// Every spelling of water's molar mass that the engine used to carry.
///
/// `18.015` catches `18.015`, `18.0152`, `18.01528` and `18.015_28` alike;
/// the other two catch the kg/mol forms with and without the digit separator.
const SPELLINGS: &[&str] = &["18.015", "18_015", "0.018015", "0.018_015"];

/// Source files where a literal is the point, with the reason.
///
/// A short allowlist is better than a blunt rule here: the alternative is
/// either an exemption comment convention nobody remembers, or no check at
/// all. Every entry has to say why.
const ALLOWED: &[(&str, &str)] = &[
    (
        "crates/kerotakis-org/src/lib.rs",
        "a unit test asserting that the vendored chematic descriptor recomputes \
         H2O's molecular weight as 18.015 to within 0.1. It is checking a THIRD \
         PARTY's arithmetic, so reading our constant would make it tautological.",
    ),
    (
        "crates/kerotakis-registry-export/src/lib.rs",
        "the file that AUTHORS the registry records. Its citation strings quote \
         the number as prose - the arithmetic that turns 2256.30 int. J/g into \
         a molar enthalpy, for one - and writing the number down is this file's \
         job rather than a copy of somebody else's.",
    ),
];

/// No crate's `src/` carries a copy of water's molar mass.
///
/// Comments are exempt and deliberately so: several of them quote the old
/// numbers in order to explain what went wrong, and a rule that forbade
/// saying "18.01528" would delete the account of why 18.01528 was wrong.
/// What the rule forbids is a literal that something COMPUTES with.
#[test]
fn no_crate_source_file_carries_its_own_molar_mass_of_water() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let mut visited = 0usize;
    for crate_dir in std::fs::read_dir(root.join("crates")).expect("the crates directory") {
        let src = crate_dir.expect("a crate directory").path().join("src");
        if src.is_dir() {
            scan(&src, &root, &mut offenders, &mut visited);
        }
    }
    assert!(
        visited > 50,
        "the scan found only {visited} source files, so it is not scanning what \
         it thinks it is"
    );
    assert!(
        offenders.is_empty(),
        "these lines carry their own copy of water's molar mass instead of \
         reading `kerotakis_core::constants::WATER_MOLAR_MASS_G_PER_MOL` (or \
         the kg/mol form). One quantity with two derivations and nothing \
         keeping them equal is the defect #610 closed:\n{}",
        offenders.join("\n")
    );
}

fn scan(dir: &Path, root: &Path, offenders: &mut Vec<String>, visited: &mut usize) {
    for entry in std::fs::read_dir(dir).expect("a source directory") {
        let path = entry.expect("a directory entry").path();
        if path.is_dir() {
            scan(&path, root, offenders, visited);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .expect("a path under the workspace root")
            .to_string_lossy()
            .replace('\\', "/");
        if ALLOWED.iter().any(|(file, _)| *file == relative) {
            continue;
        }
        *visited += 1;
        let text = std::fs::read_to_string(&path).expect("a readable source file");
        for (number, line) in text.lines().enumerate() {
            // A whole-line comment is exempt; a trailing one is not. The rule
            // is deliberately crude in that direction: a line that computes
            // with the literal AND explains itself is still a line that
            // computes with the literal.
            if line.trim_start().starts_with("//") {
                continue;
            }
            if SPELLINGS.iter().any(|spelling| line.contains(spelling)) {
                offenders.push(format!("  {relative}:{}: {}", number + 1, line.trim()));
            }
        }
    }
}
