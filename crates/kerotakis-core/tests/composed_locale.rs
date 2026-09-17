//! The gate for the sentences the engine BUILDS (I18N-7, I18N-8).
//!
//! `tests/i18n_coverage.rs` walks the registries and asks whether German
//! has a word for each TERM the engine can put on a screen. It says so
//! itself, in the list of what it consciously does not cover:
//!
//! > **`appearance::observe`'s composed sentence.** `Appearance.words` is
//! > prose assembled in `appearance.rs`, not a term drawn from a table.
//! > It needs call-site keys before any catalogue can reach it.
//!
//! It has them now, and this is that gate. Two halves, because a composed
//! sentence can be untranslated in two different ways and only one of
//! them is visible from the source:
//!
//! 1. **Static.** Every `Phrase::new`/`Phrase::bare` key written anywhere
//!    in the engine must exist in every shipped catalogue, and the
//!    translation's holes must match the source's. The denominator is
//!    read out of the Rust, not maintained by hand — a list of keys to
//!    check is a list someone forgets to extend on the commit that needed
//!    it, which is exactly how #505 reported `models.toml` at 100% German
//!    with 325 of 409 strings in English.
//! 2. **Dynamic.** A real beaker is observed and the clause tree it
//!    produces is walked: every key AND every term slot in it must
//!    resolve in German. This is the half that catches a sentence the
//!    source spells correctly and the catalogue answers in English —
//!    a colour word nobody translated, a species with no German name.
//!
//! Adding French makes both halves fail until `fr.toml` carries the rows,
//! which is the intended behaviour: the gate is over `Locale::available`,
//! so a new language is checked the moment it is registered.

use std::collections::BTreeSet;

use kerotakis_core::phrase::{Phrase, Slot};
use kerotakis_core::species::Phase;
use kerotakis_core::{appearance, Locale, Moles, SpeciesId, Vessel, VesselId};

/// Every file that composes a sentence out of `Phrase`.
///
/// Source text rather than a registry, because a `Phrase` is written at a
/// call site and there is no table to walk. Adding a file here is the one
/// piece of bookkeeping this gate needs; forgetting to costs coverage,
/// never correctness, and `unknown_composers_are_declared` below fails if
/// a new file starts composing without being listed.
const COMPOSERS: &[(&str, &str)] = &[
    ("appearance.rs", include_str!("../src/appearance.rs")),
    ("displacement.rs", include_str!("../src/displacement.rs")),
    ("solve.rs", include_str!("../src/solve.rs")),
    ("nonaqueous.rs", include_str!("../src/nonaqueous.rs")),
];

/// `Phrase::new("key", "english …"` and the `bare` form, as (key, en).
///
/// A regex would need a crate; the shape is fixed enough to scan for.
fn phrase_literals() -> Vec<(&'static str, String, String)> {
    let mut out = Vec::new();
    for (file, src) in COMPOSERS {
        for opener in ["Phrase::new(", "Phrase::bare("] {
            let mut rest: &str = src;
            while let Some(at) = rest.find(opener) {
                rest = &rest[at + opener.len()..];
                let Some((key, after_key)) = next_literal(rest) else {
                    continue;
                };
                let Some((en, _)) = next_literal(after_key) else {
                    continue;
                };
                out.push((*file, key, en));
            }
        }
    }
    assert!(
        out.len() >= 15,
        "the scanner found only {} phrases — the call shape must have \
         changed, and this gate is now checking almost nothing",
        out.len()
    );
    out
}

/// The next `"…"` in `text`, with Rust's line continuations joined, and
/// what follows it.
fn next_literal(text: &str) -> Option<(String, &str)> {
    let open = text.find('"')?;
    let mut out = String::new();
    let mut indices = text[open + 1..].char_indices();
    while let Some((i, c)) = indices.next() {
        match c {
            '"' => return Some((out, &text[open + 1 + i + 1..])),
            '\\' => match indices.next()?.1 {
                '\n' => {
                    // A `\` at end of line: Rust eats the newline and the
                    // indentation that follows it.
                    for (_, c) in indices.by_ref() {
                        if !c.is_whitespace() {
                            out.push(c);
                            break;
                        }
                    }
                }
                'n' => out.push('\n'),
                other => out.push(other),
            },
            other => out.push(other),
        }
    }
    None
}

fn holes(template: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        let Some(close) = rest[open..].find('}').map(|i| open + i) else {
            break;
        };
        found.insert(rest[open + 1..close].to_string());
        rest = &rest[close + 1..];
    }
    found
}

/// Every key a composer writes exists in every shipped catalogue.
///
/// This is the denominator that makes the percentage honest: it is the
/// count of keys in the SOURCE, not the count of keys in the catalogue.
#[test]
fn every_composed_key_is_translated() {
    let phrases = phrase_literals();
    for locale in Locale::available().into_iter().filter(|l| !l.is_english()) {
        let missing: Vec<String> = phrases
            .iter()
            .filter(|(_, key, _)| locale.lookup(key).is_none())
            .map(|(file, key, en)| format!("  {key}  ({file})  = \"{en}\""))
            .collect();
        assert!(
            missing.is_empty(),
            "{} of {} sentences the engine composes have no {} translation.\n\
             These are not missing strings in a catalogue — they are the lines\n\
             the engine writes itself, and a reader of {} sees them in English\n\
             inside an otherwise translated lesson.\n\n{}",
            missing.len(),
            phrases.len(),
            locale.code(),
            locale.code(),
            missing.join("\n"),
        );
    }
}

/// A translation may reorder the holes; it may not invent or drop one.
///
/// `Locale::fill` leaves an unknown placeholder as written, so `{vesel}`
/// in a catalogue reaches the screen verbatim. `tools/i18n-holes-lint.py`
/// checks this for `render.rs`; nothing checked it for a `Phrase`.
#[test]
fn a_translation_fills_the_same_holes() {
    for (file, key, en) in phrase_literals() {
        for locale in Locale::available().into_iter().filter(|l| !l.is_english()) {
            let Some(translated) = locale.lookup(&key) else {
                continue; // reported by the test above
            };
            assert_eq!(
                holes(&en),
                holes(translated),
                "{} translation of {key} ({file}) asks for different holes than \
                 the English does.\n  en: {en}\n  {}: {translated}",
                locale.code(),
                locale.code(),
            );
        }
    }
}

/// A file that composes without being declared above is invisible to this
/// gate, so find it.
#[test]
fn unknown_composers_are_declared() {
    let declared: BTreeSet<&str> = COMPOSERS.iter().map(|(name, _)| *name).collect();
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut undeclared = Vec::new();
    let mut stack = vec![dir];
    while let Some(path) = stack.pop() {
        for entry in std::fs::read_dir(&path).expect("src is readable") {
            let entry = entry.expect("a directory entry");
            let p = entry.path();
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            if p.extension().is_none_or(|e| e != "rs") {
                continue;
            }
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            // `phrase.rs` defines the constructors; its own doc examples
            // are not sentences the engine says.
            if name == "phrase.rs" || declared.contains(name) {
                continue;
            }
            let text = std::fs::read_to_string(&p).expect("a source file");
            if text.contains("Phrase::new(") || text.contains("Phrase::bare(") {
                undeclared.push(name.to_string());
            }
        }
    }
    assert!(
        undeclared.is_empty(),
        "these files compose sentences but are not in COMPOSERS, so nothing \
         checks that their keys are translated: {undeclared:?}"
    );
}

/// The curated organic-solvent verdicts, whose key is built from the table
/// row rather than written as a literal.
///
/// The static scanner above cannot see these — there is no key literal to
/// find — so the table itself is the denominator, read from the `const`
/// the engine actually uses. A metal added to `INERT_IN_SOLVENT` without
/// German fails here on the commit that added it.
#[test]
fn every_curated_solvent_verdict_is_translated() {
    use kerotakis_core::nonaqueous::INERT_IN_SOLVENT;
    for locale in Locale::available().into_iter().filter(|l| !l.is_english()) {
        let missing: Vec<String> = INERT_IN_SOLVENT
            .iter()
            .filter(|(metal, solvent, _)| {
                locale
                    .lookup(&format!("inert-in-solvent.{metal}-{solvent}"))
                    .is_none()
            })
            .map(|(metal, solvent, why)| format!("  {metal}-{solvent} = \"{why}\""))
            .collect();
        assert!(
            missing.is_empty(),
            "{} of {} curated solvent verdicts have no {} translation:\n{}",
            missing.len(),
            INERT_IN_SOLVENT.len(),
            locale.code(),
            missing.join("\n"),
        );
    }
}

// --- The dynamic half: a real beaker, walked.

fn unresolved(clause: &Phrase, locale: Locale, out: &mut Vec<String>) {
    if locale.lookup(&clause.key).is_none() {
        out.push(format!("phrase {}  = \"{}\"", clause.key, clause.en));
    }
    for (_, slot) in &clause.slots {
        unresolved_slot(slot, locale, out);
    }
}

fn unresolved_slot(slot: &Slot, locale: Locale, out: &mut Vec<String>) {
    match slot {
        Slot::Text { .. } | Slot::Number { .. } => {}
        Slot::Term { section, en } => {
            if locale.lookup(&format!("{section}.{en}")).is_none() {
                out.push(format!("term {section}.{en}"));
            }
        }
        Slot::Phrase { phrase } => unresolved(phrase, locale, out),
        Slot::List { items } => {
            for item in items {
                unresolved_slot(item, locale, out);
            }
        }
    }
}

fn beaker(items: &[(&str, f64, Phase)]) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    for (key, moles, phase) in items {
        v.deposit(SpeciesId::new(key), Moles(*moles), *phase);
    }
    v
}

/// The owner's own playback, in reverse: look at a beaker and check that
/// nothing in the sentence is still English.
#[test]
fn no_english_reaches_a_german_look() {
    let de = Locale::parse("de");
    let scenes: Vec<(&str, Vessel)> = vec![
        ("an empty beaker", Vessel::new(VesselId(0), "beaker")),
        ("water", beaker(&[("water", 5.55, Phase::Liquid)])),
        (
            "copper sulfate",
            beaker(&[
                ("water", 5.55, Phase::Liquid),
                ("Cu+2", 0.05, Phase::Aqueous),
            ]),
        ),
        (
            "a silver chloride precipitate",
            beaker(&[("water", 5.55, Phase::Liquid), ("AgCl", 0.01, Phase::Solid)]),
        ),
        (
            "zinc and copper at the bottom",
            beaker(&[
                ("water", 5.55, Phase::Liquid),
                ("Zn", 0.02, Phase::Solid),
                ("Cu", 0.01, Phase::Solid),
            ]),
        ),
        (
            "gas coming off",
            beaker(&[("water", 5.55, Phase::Liquid), ("CO2", 0.05, Phase::Gas)]),
        ),
        (
            "dilute permanganate",
            beaker(&[
                ("water", 55.5, Phase::Liquid),
                ("MnO4-", 1e-5, Phase::Aqueous),
            ]),
        ),
    ];
    let mut faults: Vec<String> = Vec::new();
    for (what, vessel) in &scenes {
        let seen = appearance::observe(vessel);
        assert!(
            !seen.clauses.is_empty(),
            "{what}: observed nothing to say, so this scene checks nothing"
        );
        let mut here = Vec::new();
        for clause in &seen.clauses {
            unresolved(clause, de, &mut here);
        }
        for note in &seen.notes {
            unresolved(note, de, &mut here);
        }
        for fault in here {
            faults.push(format!("  {what}: {fault}"));
        }
        // And the whole point: the German is not the English.
        assert_ne!(
            seen.say(de),
            seen.words,
            "{what}: the German look line is identical to the English one"
        );
    }
    assert!(
        faults.is_empty(),
        "German is missing {} of the parts these look lines are made of:\n{}",
        faults.len(),
        faults.join("\n"),
    );
}

/// The owner's line, end to end: *v1 Zink reagiert nicht — zinc should
/// dissolve in this acid by the series…*
///
/// The refusal's NAME was German and its REASON was not, in the same
/// sentence, which is what made it worth a task of its own.
#[test]
fn the_inert_verdict_explains_itself_in_german() {
    use kerotakis_core::ops::Event;
    use kerotakis_core::{render_events_in, Register};

    let reason = Phrase::new(
        "inert.hydrogen-overpotential",
        "{name} should dissolve in this acid by the series (driving force {driving} V), but hydrogen has to form on {name}, and on that surface it costs an overpotential of about {eta} V. Kinetically blocked on the timescale of a lesson, not thermodynamically inert — the difference between a bench and a battery",
        vec![
            ("name".to_string(), Slot::term("species", "zinc")),
            ("driving".to_string(), Slot::number("+0.62".to_string())),
            ("eta".to_string(), Slot::number("0.72".to_string())),
        ],
    );
    let english = reason.render(Locale::EN);
    let event = Event::Inert {
        vessel: VesselId(0),
        species: SpeciesId::new("Zn"),
        why: english.clone(),
        computed: true,
        spent: None,
        reason: Some(reason),
    };
    // The English is generated from the very template a translation
    // replaces, so the codex entry that quotes it verbatim still matches.
    assert!(
        english
            .starts_with("zinc should dissolve in this acid by the series (driving force +0.62 V)"),
        "{english}"
    );

    let de = Locale::parse("de");
    let line = render_events_in(&[event], Register::LV2, de).join(" ");
    assert!(
        line.contains("Überspannung") && line.contains("Triebkraft"),
        "the reason should be German: {line}"
    );
    assert!(
        !line.contains("should dissolve") && !line.contains("overpotential"),
        "English survived into the German verdict: {line}"
    );
    // German writes +0,62 V. The precision is the engine's decision and
    // the separator is the reader's, and this is where they meet.
    assert!(line.contains("+0,62"), "{line}");
}

/// An `Appearance` that predates the clause list — replayed from an old
/// event, or hand-built by a test — still says something.
#[test]
fn an_appearance_without_clauses_falls_back_to_its_english() {
    let seen = appearance::Appearance {
        spectral_gaps: Vec::new(),
        liquid: None,
        cloudiness: 0.0,
        deposit: None,
        bubbling: false,
        words: "The beaker is empty.".to_string(),
        clauses: Vec::new(),
        notes: Vec::new(),
    };
    assert_eq!(seen.say(Locale::parse("de")), "The beaker is empty.");
}
