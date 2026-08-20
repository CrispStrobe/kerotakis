//! The numbers written in prose about the codex must match the codex.
//!
//! `kero codex lint` replays every number inside a codex entry, on the
//! principle that prose which must match a computed value is a check rather
//! than decoration. The project's own documentation was exempt from that
//! principle, and it drifted exactly as you would expect: README claimed 66
//! entries, 27 models and 113 concepts against an actual 80, 28 and 130,
//! and two copies of a phase-coverage figure in PLAN went stale within a day
//! of being written.
//!
//! These are the load-bearing figures — the ones a reader uses to decide
//! whether the project is worth their time — so they get the same treatment
//! as the codex's own claims.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/<crate>/ is two below the root")
        .to_path_buf()
}

fn codex() -> kerotakis_codex::Codex {
    let dir = repo_root().join("codex");
    let mut all = kerotakis_codex::Codex::default();
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("codex/ is readable")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "toml"))
        .collect();
    files.sort();
    for f in &files {
        let text = std::fs::read_to_string(f).expect("codex file is readable");
        let mut c = kerotakis_codex::Codex::parse(&text)
            .unwrap_or_else(|e| panic!("{}: {e}", f.display()));
        all.reactions.append(&mut c.reactions);
        all.models.append(&mut c.models);
    }
    all
}

/// Every figure below is quoted verbatim from README, so a stale number
/// fails here rather than misleading a reader.
#[test]
fn readme_counts_match_the_codex() {
    let c = codex();
    let readme = std::fs::read_to_string(repo_root().join("README.md")).expect("README.md");
    for (claim, actual) in [
        (
            format!("carries {} model", c.models.len()),
            c.models.len(),
        ),
        (
            format!(
                "**{} reaction entries, {} models and {} concepts**",
                c.reactions.len(),
                c.models.len(),
                c.concept_index().len()
            ),
            c.reactions.len(),
        ),
    ] {
        assert!(
            readme.contains(&claim),
            "README does not say {claim:?} — the codex now holds {} entries, \
             {} models and {} concepts (actual for this claim: {actual}). \
             Update README.md rather than this test.",
            c.reactions.len(),
            c.models.len(),
            c.concept_index().len(),
        );
    }
}
