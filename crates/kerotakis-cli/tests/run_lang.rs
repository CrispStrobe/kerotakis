//! `kero run` reads a file in the language it was typed in.
//!
//! The alias layer landed for the REPL (`kero repl --lang de`) and stopped
//! there: `run` treated a `.lab` file as canonical English on purpose, so
//! a learner who could type `zugeben v1 Wasser 100mL` at the prompt could
//! not put the same line in a file. That default is load-bearing — every
//! shipped lesson, the corpus and the replay cache are canonical English
//! and none of them may change meaning because of an environment variable
//! — so this does not move it. It adds two ways to say the file is not:
//! `--lang de` (or `KERO_LANG`) from outside, and a `lang de` first line
//! from inside, for a German-authored lesson that has to survive being
//! shared.
//!
//! What every test here is really about is the invariant underneath:
//! **the canonical script stays English**. A German file and its English
//! twin are the same run, because the rewrite happens before anything
//! executes and the bench logs and exports the canonical form.
//!
//! Where that invariant LIVES moved on 2026-09-20. It used to be pinned
//! on stdout prose, and it was only true there because the CLI called the
//! English-only render wrappers: `kero run --lang de` parsed German and
//! answered `v1: +5.5343 mol water` while the GUI answered `v1: +5,5343
//! mol Wasser`. Prose is the half of the conversation the READER owns, so
//! it now follows the invocation. The canonical form is `--json`, which
//! is structure and carries no prose at all, and that is what the twin
//! assertions below compare.
//!
//! One case keeps prose English on purpose: a `lang de` DIRECTIVE on a
//! file's first line describes the FILE, not whoever runs it. A shipped
//! German-authored lesson still reads back canonical English, which is
//! what makes it shareable.

use std::process::Command;

/// The same four steps, typed twice.
///
/// Every German line here is one `script.rs` already round-trips in its
/// own unit tests, so a failure in this file is about `run` and not about
/// the alias tables.
const ENGLISH: &str = "\
register lv2
add v1 water 100mL
heat v1 10kJ on candle
measure v1 balance
";

const GERMAN: &str = "\
register lv2
zugeben v1 Wasser 100mL
erhitzen v1 10kJ auf kerze
messen v1 waage
";

fn scratch(name: &str, body: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join("kero-run-lang");
    std::fs::create_dir_all(&dir).expect("scratch dir");
    let path = dir.join(name);
    std::fs::write(&path, body).expect("write script");
    path
}

fn kero(args: &[&str], env: &[(&str, &str)]) -> (String, String, bool) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_kero"));
    cmd.args(args);
    // A stale KERO_LANG in the developer's own shell must not decide what
    // this test measures, so every case states its environment.
    cmd.env_remove("KERO_LANG");
    for (key, value) in env {
        cmd.env(key, value);
    }
    let out = cmd.output().expect("kero runs");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.success(),
    )
}

/// The German file and the English one are the same run.
///
/// Compared on `--json`, which is the contract: the operator, the events
/// and the scene, none of which is prose. If the alias layer ever let a
/// German word through to the bench, or rewrote a step it should not
/// have, these two streams stop matching.
#[test]
fn a_german_script_run_with_lang_executes_the_english_twins_steps() {
    let english = scratch("twin-en.lab", ENGLISH);
    let german = scratch("twin-de.lab", GERMAN);

    let (want, err, ok) = kero(&["run", "--json", english.to_str().unwrap()], &[]);
    assert!(ok, "the English twin runs: {err}");
    assert!(!want.trim().is_empty(), "it prints something");

    let (got, err, ok) = kero(
        &["run", "--json", "--lang", "de", german.to_str().unwrap()],
        &[],
    );
    assert!(ok, "the German twin runs: {err}");
    assert_eq!(
        got, want,
        "a German script must run the steps its English twin does"
    );
    // The canonical form is in there too: the bench logs the English
    // operator whatever was typed at it.
    assert!(
        !got.contains("zugeben") && !got.contains("erhitzen"),
        "the bench logs the canonical operator:\n{got}"
    );
}

/// The flag may come before the path, and the path is still found.
///
/// `run` read `args[1]` positionally, which was right for as long as it
/// took no option with a value; `kero run --lang de lesson.lab` would
/// have tried to open a file called `--lang`.
#[test]
fn the_flag_may_precede_the_path() {
    let english = scratch("order-en.lab", ENGLISH);
    let german = scratch("order-de.lab", GERMAN);
    let (want, _, ok) = kero(&["run", "--json", english.to_str().unwrap()], &[]);
    assert!(ok);
    let (got, err, ok) = kero(
        &["run", "--json", "--lang=de", german.to_str().unwrap()],
        &[],
    );
    assert!(ok, "--lang=de before the path: {err}");
    assert_eq!(got, want);
}

/// `KERO_LANG` says the same thing from the environment.
#[test]
fn kero_lang_is_honoured_by_run() {
    let english = scratch("env-en.lab", ENGLISH);
    let german = scratch("env-de.lab", GERMAN);
    let (want, _, ok) = kero(&["run", "--json", english.to_str().unwrap()], &[]);
    assert!(ok);
    let (got, err, ok) = kero(
        &["run", "--json", german.to_str().unwrap()],
        &[("KERO_LANG", "de-AT")],
    );
    assert!(ok, "KERO_LANG=de-AT: {err}");
    assert_eq!(got, want, "a regional tag resolves to its language");
}

/// A German-authored lesson declares itself, and needs no flag.
///
/// This is the case the flag cannot cover: a file is shared, and whoever
/// runs it next does not know what it was written in. `lang de` is a
/// property of the file, the way `register lv2` is.
///
/// And because it is a property of the FILE, it moves the parser and not
/// the reader: this comparison is on stdout prose, unlike the three
/// above, because a declared German lesson must still read back in
/// English to an English reader who was handed it.
#[test]
fn a_lang_directive_in_the_file_needs_no_flag() {
    let english = scratch("decl-en.lab", ENGLISH);
    let declared = scratch("decl-de.lab", &format!("lang de\n{GERMAN}"));
    let (want, _, ok) = kero(&["run", english.to_str().unwrap()], &[]);
    assert!(ok);
    let (got, err, ok) = kero(&["run", declared.to_str().unwrap()], &[]);
    assert!(ok, "a declared file runs on its own: {err}");
    assert_eq!(got, want);
    assert!(
        !got.contains("Wasser"),
        "the file's language is not the reader's:\n{got}"
    );
}

/// English stays the default, and a German file without either says so.
///
/// The whole point of the default is that nothing shipped changes: this
/// is the test that would fail if `run` started guessing.
#[test]
fn english_remains_the_default() {
    let german = scratch("bare-de.lab", GERMAN);
    let (_, err, ok) = kero(&["run", german.to_str().unwrap()], &[]);
    assert!(!ok, "an undeclared German file is not silently understood");
    assert!(
        err.contains("zugeben"),
        "and the refusal names the word it did not know: {err}"
    );
}

/// An English script is untouched by `--lang de`.
///
/// Rule 2 of the alias layer — English wins — is what makes the flag safe
/// to set globally: a word the English grammar already spends is never
/// taken over by a translation. Every shipped lesson and the replay cache
/// rest on this.
#[test]
fn an_english_script_is_the_same_run_under_lang_de() {
    let english = scratch("stable-en.lab", ENGLISH);
    let (plain, _, ok) = kero(&["run", "--json", english.to_str().unwrap()], &[]);
    assert!(ok);
    let (translated, err, ok) = kero(
        &["run", "--json", "--lang", "de", english.to_str().unwrap()],
        &[],
    );
    assert!(ok, "{err}");
    assert_eq!(plain, translated);
}

/// A language the bench does not ship is refused, not silently ignored.
///
/// `Locale::parse` answers English for anything it cannot find, which is
/// the right answer for an environment variable someone else set and the
/// wrong one for a line an author wrote on purpose: parsing a French
/// lesson as English fails later, somewhere else, with a message about a
/// verb.
#[test]
fn an_unknown_language_directive_is_refused_by_name() {
    let script = scratch("unknown.lab", "lang fr\nadd v1 water 100mL\n");
    let (_, err, ok) = kero(&["run", script.to_str().unwrap()], &[]);
    assert!(!ok, "an unshipped language is refused");
    assert!(
        err.contains("unknown language") && err.contains("de"),
        "and the refusal names what there is: {err}"
    );
}
