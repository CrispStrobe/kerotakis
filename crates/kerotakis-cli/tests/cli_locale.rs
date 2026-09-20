//! The terminal bench answers in the reader's language.
//!
//! The GUI and the REPL run the same engine over the same catalogue, and
//! for one missing argument they did not say the same thing. Typing
//! `add v1 water 100mL` into a German session printed
//!
//! ```text
//!   v1: +5.5343 mol water
//! ```
//!
//! where the browser, given the identical command, printed
//!
//! ```text
//!   v1: +5,5343 mol Wasser
//! ```
//!
//! Two differences and one cause. `crates/kerotakis-cli/src/main.rs` held
//! `self.locale` and called `render_events(&events, self.register)` — the
//! English-only wrapper, defined as `render_events_in(…, Locale::EN)`.
//! The decimal separator rides the same locale, through `Locale::number`,
//! so passing it fixes the comma at the same time as the words. This is
//! the welded-prose defect one level up: the host knew the reader's
//! language and did not pass it on.
//!
//! A golden string would not have caught it — the English golden string
//! was correct. What catches it is the pair: the same script run twice,
//! in two languages, must **agree on every number and differ on every
//! sentence**. That shape is also what a third language gets for free,
//! which is the point of the rule it guards: adding French is one
//! `i18n/fr.toml` and no code.

use std::process::Command;

/// Two steps and a look. Enough to reach three different composers — the
/// event renderer, the provenance clause and the appearance phrase — and
/// short enough that a failure names the line rather than a wall.
const SCRIPT: &str = "\
register lv2
add v1 water 100mL
look v1
";

fn scratch(name: &str, body: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join("kero-cli-locale");
    std::fs::create_dir_all(&dir).expect("scratch dir");
    let path = dir.join(name);
    std::fs::write(&path, body).expect("write script");
    path
}

fn run(args: &[&str]) -> String {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_kero"));
    cmd.args(args);
    // A stale KERO_LANG in whoever's shell must not decide what this
    // measures, so each case states its own language.
    cmd.env_remove("KERO_LANG");
    let out = cmd.output().expect("kero runs");
    assert!(
        out.status.success(),
        "kero {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Every digit of a line, separators and words thrown away.
///
/// `5.5343` and `5,5343` both reduce to `55343`, which is how a number
/// can be checked for having survived translation without the check
/// caring which separator the language writes.
fn digits(line: &str) -> String {
    line.chars().filter(char::is_ascii_digit).collect()
}

/// Words that are the same in every language the bench ships, so their
/// presence says nothing about whether a line was translated.
const LANGUAGE_NEUTRAL: &[&str] = &["phreeqc", "usgs", "nacl", "kgw", "kero"];

/// Does this line contain a word whose spelling a translator owns?
fn carries_prose(line: &str) -> bool {
    line.split(|c: char| !c.is_ascii_alphabetic()).any(|word| {
        word.len() >= 4 && !LANGUAGE_NEUTRAL.contains(&word.to_ascii_lowercase().as_str())
    })
}

/// The pair test: same structure, different sentences.
///
/// This is the assertion the whole file exists for, and it is deliberately
/// not a golden string. A golden string pins what German says today; this
/// pins that German is not English, line for line, while the numbers
/// underneath are identical — which is the only property that stays true
/// when somebody improves a translation, and the only one a fourth
/// language inherits without a new test.
#[test]
fn a_german_run_and_an_english_run_agree_on_structure_and_differ_on_prose() {
    let script = scratch("pair.lab", SCRIPT);
    let english = run(&["run", script.to_str().unwrap()]);
    let german = run(&["run", "--lang", "de", script.to_str().unwrap()]);

    let en: Vec<&str> = english.lines().collect();
    let de: Vec<&str> = german.lines().collect();
    assert!(!en.is_empty(), "the script prints something");
    assert_eq!(
        en.len(),
        de.len(),
        "a translation adds and removes no lines\nEN:\n{english}\nDE:\n{german}"
    );

    for (english_line, german_line) in en.iter().zip(&de) {
        assert_eq!(
            digits(english_line),
            digits(german_line),
            "the same run must compute the same numbers, whatever the \
             separator:\n  EN {english_line}\n  DE {german_line}"
        );
        if carries_prose(english_line) {
            assert_ne!(
                english_line, german_line,
                "a line carrying prose reached a German reader in \
                 English — either the host did not pass the locale, or \
                 i18n/de.toml has no row for it:\n  {english_line}"
            );
        }
    }
}

/// The reported line, pinned whole — comma included.
///
/// One golden string is still worth having beside the pair test, because
/// the pair test cannot tell a German sentence from a French one. This is
/// the exact line the owner typed and the exact answer they should have
/// got, and `+5.5343` appearing anywhere in a German transcript is the
/// bug returning by its own name.
#[test]
fn the_german_bench_writes_the_decimal_comma() {
    let script = scratch("comma.lab", "register lv2\nadd v1 water 100mL\n");
    let german = run(&["run", "--lang", "de", script.to_str().unwrap()]);
    assert!(
        german.contains("+5,5343 mol Wasser"),
        "the reported line, in the reader's language:\n{german}"
    );
    assert!(
        !german.contains("5.5343"),
        "the decimal point rides the locale too:\n{german}"
    );
}

/// English is untouched, which is what makes the change safe to ship.
#[test]
fn english_is_unchanged() {
    let script = scratch("plain.lab", "register lv2\nadd v1 water 100mL\n");
    let english = run(&["run", script.to_str().unwrap()]);
    assert!(
        english.contains("+5.5343 mol water"),
        "the default is still the canonical transcript:\n{english}"
    );
}

/// `zoom`'s header travelled with its drawing.
///
/// `Census::render` took a locale in September; the `println!` above it
/// did not, so a German session read a German drawing under an English
/// title. Header and caption are one surface and are checked as one.
#[test]
fn the_particle_drawings_header_is_translated() {
    let script = scratch(
        "particles.lab",
        "register lv2\nadd v1 water 100mL\nadd v1 NaCl 0.05mol\nparticles v1\n",
    );
    let german = run(&["run", "--lang", "de", script.to_str().unwrap()]);
    assert!(
        german.contains("was die Teilchen tun"),
        "the header above the drawing:\n{german}"
    );
    assert!(
        !german.contains("what the particles are doing"),
        "and not beside it:\n{german}"
    );
}

// ---------------------------------------------------------------------
// The guard. The bug was one argument, in one call, and nothing in the
// tree could see it: the English-only wrappers compile, run and print
// perfectly good English. So the source itself is checked.

/// Every `Locale::EN` the CLI is allowed to contain, and why.
///
/// A line is listed by the substring that identifies it, so the list
/// survives the file being edited around it. Adding a row here is the
/// deliberate act the bug skipped: it costs a sentence explaining who
/// reads this string and why they read it in English.
const JUSTIFIED: &[(&str, &str)] = &[
    (
        "main.rs",
        // `typing_language`'s fallback: an unknown tag answers English
        // rather than erroring, so someone whose shell names a language
        // nobody has translated gets the bench they had.
        "flag.map_or(kerotakis_core::Locale::EN",
    ),
    (
        "mcp.rs",
        // `explain` over MCP, at both call sites. Its consumer is a tool,
        // not a reader, and a caller that parses this text must not have
        // the language change under it. The REPL passes its own locale to
        // the same function; this does not, and says so in place.
        "explain_text(",
    ),
];

/// Every `.rs` file the CLI crate compiles, as (file name, body).
fn cli_sources() -> Vec<(String, String)> {
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut out = Vec::new();
    let mut stack = vec![src];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("read the CLI source tree") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                let name = path
                    .file_name()
                    .expect("a file")
                    .to_string_lossy()
                    .into_owned();
                out.push((name, std::fs::read_to_string(&path).expect("read source")));
            }
        }
    }
    out.sort();
    out
}

/// No English is pinned into the CLI by accident.
#[test]
fn every_hardcoded_english_locale_in_the_cli_is_justified() {
    let mut unexplained = Vec::new();
    for (name, body) in cli_sources() {
        for (number, line) in body.lines().enumerate() {
            if !line.contains("Locale::EN") {
                continue;
            }
            // A doc comment naming the wrapper is prose about the rule,
            // not an instance of it.
            if line.trim_start().starts_with("//") {
                continue;
            }
            let allowed = JUSTIFIED
                .iter()
                .any(|(file, needle)| name == *file && line.contains(*needle));
            if !allowed {
                unexplained.push(format!("{name}:{}: {}", number + 1, line.trim()));
            }
        }
    }
    assert!(
        unexplained.is_empty(),
        "a hard-coded English locale with no stated reader. Either pass \
         the session's locale, or add the site to JUSTIFIED with a \
         comment saying who reads it:\n{}",
        unexplained.join("\n")
    );
}

/// And none of the English-only render wrappers is called from the CLI.
///
/// `render_events`, `render_vessel`, `render_event` and `render_ionic`
/// each exist beside an `_in` twin that takes a locale; the bare name is
/// the twin with `Locale::EN` welded in. They are kept for the engine's
/// own tests, which assert on English prose on purpose — so they cannot
/// simply be deleted, which is exactly why the CLI needs a rule.
#[test]
fn the_cli_calls_no_english_only_render_wrapper() {
    const WRAPPERS: &[&str] = &[
        "render_events(",
        "render_vessel(",
        "render_event(",
        "render_ionic(",
    ];
    let mut found = Vec::new();
    for (name, body) in cli_sources() {
        for (number, line) in body.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            for wrapper in WRAPPERS {
                if line.contains(*wrapper) {
                    found.push(format!("{name}:{}: {}", number + 1, line.trim()));
                }
            }
        }
    }
    assert!(
        found.is_empty(),
        "the CLI holds the reader's language; call the `_in` twin and \
         pass it:\n{}",
        found.join("\n")
    );
}
