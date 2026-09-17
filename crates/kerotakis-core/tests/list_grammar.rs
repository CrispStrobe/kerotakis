//! A list of names is a GRAMMAR, not a string with commas in it.
//!
//! Eight sites composed one with `Slot::text(names.join(", "))`, which
//! puts the separator in the Rust where no catalogue can reach it and
//! leaves out the final conjunction entirely. (Eight, settled from the
//! source: I18N-10 counted nine and was right about the ninth, which is
//! `particles.rs`' `", and {n} more"` — and that one is not a `Slot` at
//! all, so it is not here. See ROADMAP-GUI.md.) `Slot::List` renders *a, b and c*
//! from two catalogue rows — `look.list-separator` and `look.list-final`
//! — so a language that uses no final conjunction writes `"{head}{last}"`
//! and gets what it wants without a line of Rust.
//!
//! Converting them CHANGED the English, which is why this was a pass of
//! its own: *a, b, c* became *a, b and c*, and `lessons.json` moved by
//! exactly that one word.

use kerotakis_core::phrase::{Phrase, Slot};
use kerotakis_core::Locale;

fn rendered(items: &[&str], locale: Locale) -> String {
    Phrase::new(
        "look.spectral-gap",
        "Colour is incomplete: no absorption spectrum for {species}.",
        vec![("species".to_string(), Slot::texts(items.iter().copied()))],
    )
    .render(locale)
}

#[test]
fn one_two_and_three_items_read_as_a_list_in_english() {
    assert_eq!(
        rendered(&["a"], Locale::EN),
        "Colour is incomplete: no absorption spectrum for a."
    );
    assert_eq!(
        rendered(&["a", "b"], Locale::EN),
        "Colour is incomplete: no absorption spectrum for a and b."
    );
    assert_eq!(
        rendered(&["a", "b", "c"], Locale::EN),
        "Colour is incomplete: no absorption spectrum for a, b and c."
    );
}

/// The point of the exercise: the conjunction is the reader's. German
/// says *und*, and it says it because `look.list-final` says so — not
/// because any Rust knows about German.
#[test]
fn the_conjunction_comes_from_the_catalogue() {
    let german = rendered(&["a", "b", "c"], Locale::parse("de"));
    assert!(german.contains("a, b und c"), "{german}");
    assert!(!german.contains(" and "), "{german}");
}

/// The stranded-solute sentence, which is the one the golden holds: the
/// names are registry display names, so each is a TERM looked up in the
/// reader's language and only then joined.
#[test]
fn stranded_solutes_names_each_solute_as_a_term() {
    let phrase = kerotakis_core::solve::stranded_solutes(&["sucrose", "sodium chloride"]);
    let english = phrase.render(Locale::EN);
    assert!(
        english.contains("sucrose and sodium chloride"),
        "the list grammar, not a comma: {english}"
    );
    let german = phrase.render(Locale::parse("de"));
    assert!(
        german.contains("Saccharose") || german.contains("Natriumchlorid"),
        "the SOLUTES are translated too, not only the frame: {german}"
    );
    assert!(german.contains(" und "), "{german}");
}
