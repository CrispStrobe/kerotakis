//! A lesson's worth of chemistry, read back in every shipped language.
//!
//! The i18n gates that existed before this one all ask about the
//! CATALOGUE: is every key present, does every hole match, is any row
//! unreachable. They cannot ask the question a reader actually has —
//! *when I run something, is the answer in my language?* — because the
//! answer is assembled at render time out of a dozen events, and a key
//! can be present and still never reach the line it belongs to.
//!
//! Everything this file pins was found by hand, in one afternoon, by
//! running French scripts and reading the output:
//!
//!   * a conductivity reading with an English paragraph hanging off it;
//!   * `nothing rusts in this vessel` in the middle of French prose;
//!   * `v1:` where French writes `v1 :`, because the DEFAULT register's
//!     observation line had no key at all;
//!   * `(1 composants connus)`, and the same in German and English.
//!
//! Every test in the suite passed throughout. So the corpus runs, and
//! the rule is deliberately not a count: **a rendered line that is
//! byte-identical to its English is either notation or untranslated**,
//! and notation here means a reaction equation. A line that is neither
//! fails and names itself, which is the conversation worth having.
//!
//! The scripts are canonical English on purpose. Whether a learner can
//! TYPE their own language is a different question, answered by
//! `script.rs`'s own tests; this one is about what comes back.

use std::path::{Path, PathBuf};
use std::process::Command;

use kerotakis_core::Locale;

fn corpus() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("the workspace root")
        .join("tests/coverage/localised-transcripts");
    let mut scripts: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "lab"))
        .collect();
    scripts.sort();
    scripts
}

fn transcript(script: &Path, locale: Option<&str>) -> Vec<String> {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_kero"));
    cmd.arg("run").arg(script);
    cmd.env_remove("KERO_LANG");
    if let Some(code) = locale {
        cmd.arg("--lang").arg(code);
    }
    let out = cmd.output().expect("kero runs");
    assert!(
        out.status.success(),
        "{} refused in {:?}: {}",
        script.display(),
        locale,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.trim().is_empty())
        .collect()
}

/// Notation is the same in every language, and nothing else is.
///
/// `Zn + Cu+2 → Zn+2 + Cu` is not English. Translating it is how a
/// transcript stops being readable, so an identical line carrying a
/// reaction arrow is the expected case and needs no excuse.
fn is_notation(line: &str) -> bool {
    line.contains('→')
}

#[test]
fn the_corpus_runs_in_every_shipped_language() {
    let scripts = corpus();
    assert!(
        scripts.len() >= 12,
        "only {} scripts — the corpus shrank and this gate is measuring \
         almost nothing",
        scripts.len()
    );
    for script in &scripts {
        for locale in Locale::available() {
            // `transcript` asserts the run succeeded, which is the first
            // half of the claim: a lesson that refuses in one language is
            // not a translation problem, it is a broken bench.
            let lines = transcript(script, Some(locale.code()));
            assert!(
                !lines.is_empty(),
                "{} printed nothing in {}",
                script.display(),
                locale.code()
            );
        }
    }
}

#[test]
fn nothing_but_notation_reads_the_same_as_english() {
    let mut untranslated: Vec<String> = Vec::new();
    for script in corpus() {
        let english = transcript(&script, None);
        for locale in Locale::available() {
            if locale.is_english() {
                continue;
            }
            let theirs = transcript(&script, Some(locale.code()));
            for (en, them) in english.iter().zip(theirs.iter()) {
                if en == them && !is_notation(en) {
                    untranslated.push(format!(
                        "  {} [{}]  {}",
                        script.file_name().unwrap_or_default().to_string_lossy(),
                        locale.code(),
                        en.trim()
                    ));
                }
            }
        }
    }
    untranslated.sort();
    untranslated.dedup();
    assert!(
        untranslated.is_empty(),
        "{} rendered line(s) come back exactly as the English, and none of \
         them is an equation — so they are untranslated, not notation:\n{}",
        untranslated.len(),
        untranslated.join("\n")
    );
}
