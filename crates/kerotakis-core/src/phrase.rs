//! A sentence the engine composes, in the reader's language.
//!
//! `crate::i18n` translates a string the call site wrote down. This is for
//! the other half of the engine's voice: the lines it BUILDS — what a
//! vessel looks like, why a metal did not react — where the sentence is
//! not in the source at all, only the parts of it are.
//!
//! Those lines were English everywhere, in every language, and the reason
//! was structural rather than a missing key. `appearance.rs` had no
//! `Locale` and could not have used one: it reached the reader through an
//! EVENT, and by the time anything knew who was reading, the words had
//! been chosen and the values baked in. A finished sentence on the wire is
//! not an untranslated string, it is an untranslatable one — the same
//! observation `crate::refusal` makes about the bench saying no, and the
//! same answer.
//!
//! So the engine emits the RECIPE and the host cooks it. See [`Phrase`].

use serde::{Deserialize, Serialize};

use crate::i18n::Locale;

/// One clause of an observation: a catalogue key, its English source, and
/// the slots to fill.
///
/// # Why the sentence, and not the fragment, is the unit
///
/// Every `look` line was English inside an otherwise German lesson, and
/// the reason was structural rather than a missing key: this module built
/// its sentences by concatenation — `format!("there is {list} in the
/// beaker")`, a `" and "` between the names, a colour word glued in front
/// of each one. The obvious repair, translating `"there is"`, `" and "`
/// and the colour separately and gluing the German together, does not
/// work and would have been worse than nothing:
///
/// * **Order.** French puts the colour AFTER the noun — *chlorure
///   d'argent blanc*. No translation of the two fragments can move them.
/// * **Inflection.** A German attributive adjective takes the noun's
///   gender: *weißes Natron* but *weiße Citronensäure*. The colour word
///   cannot be translated once and reused, because it is not one word.
/// * **Connectives.** `", "` and `" and "` are a list GRAMMAR, not two
///   strings; Japanese uses `、` and no final conjunction at all.
///
/// So the unit of translation is the whole clause with named holes —
/// exactly the shape `locale.fill` has served everywhere else — and a
/// language that wants a different order writes a different template.
/// The slot values are themselves translatable, which is the part a flat
/// `fill` at the call site could not express: a deposit's name is a
/// species, its colour is an appearance word, and each is looked up in
/// the reader's language at the moment the sentence is composed.
///
/// The clauses are carried in the event, not the finished sentence, so
/// the host renders them in the session's locale — the same division
/// `render.rs` already uses, where the bench reports facts and the
/// renderer chooses the words.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Phrase {
    /// Where this is said — `appearance.deposit-in-liquid`.
    pub key: String,
    /// The English source text, with `{name}` holes. Shown when a
    /// catalogue has no entry for `key`, so a half-translated language
    /// renders half German rather than blanks.
    pub en: String,
    /// The holes, in no particular order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub slots: Vec<(String, Slot)>,
}

impl Phrase {
    pub fn new(key: &str, en: &str, slots: Vec<(String, Slot)>) -> Phrase {
        Phrase {
            key: key.to_string(),
            en: en.to_string(),
            slots,
        }
    }

    pub fn bare(key: &str, en: &str) -> Phrase {
        Phrase::new(key, en, Vec::new())
    }

    /// This clause in `locale`, holes filled.
    pub fn render(&self, locale: Locale) -> String {
        let filled: Vec<(String, String)> = self
            .slots
            .iter()
            .map(|(name, slot)| (name.clone(), slot.render(locale)))
            .collect();
        let vars: Vec<(&str, &str)> = filled
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
            .collect();
        locale.fill_owned(&self.key, &self.en, &vars)
    }
}

/// What goes in a hole.
///
/// Three of the four are translatable, which is the whole reason a slot is
/// a type rather than a `String`: the composer knows that *silver
/// chloride* is a species and *white* is an appearance word, and only it
/// can know. By the time a sentence is a string that information is gone
/// and no catalogue can get it back.
/// Externally tagged, the serde default: an internally-tagged enum needs
/// `deserialize_any` and would rule out every non-self-describing format
/// the transports may reach for.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Slot {
    /// Final text — a vessel's own label, a formula. Never translated:
    /// `v1` is a NAME and `2H⁺/H₂` is notation.
    Text { text: String },
    /// A measured value, already formatted to the precision the engine
    /// means to show, and given the reader's decimal separator on the way
    /// out — German writes +0,62 V.
    ///
    /// Separate from `Text` for the reason `Refusal::with_number` is
    /// separate from `Refusal::with`: the precision is a property of the
    /// measurement and the separator is a property of the reader, and
    /// neither decision is made in the other's place.
    Number { text: String },
    /// A term the catalogue names under `<section>.<en>` — a species, a
    /// material, a colour word. Falls back to the English, so an
    /// untranslated species appears in English rather than vanishing.
    Term { section: String, en: String },
    /// A clause nested in a clause: `{colour} {name}` inside a list
    /// inside "there is {list} at the bottom".
    Phrase { phrase: Box<Phrase> },
    /// Several of the above, joined by the catalogue's list grammar.
    List { items: Vec<Slot> },
}

impl Slot {
    pub fn text(value: impl Into<String>) -> Slot {
        Slot::Text { text: value.into() }
    }

    /// A value already formatted to the precision the engine means.
    pub fn number(formatted: impl Into<String>) -> Slot {
        Slot::Number {
            text: formatted.into(),
        }
    }

    pub fn term(section: &str, en: impl Into<String>) -> Slot {
        Slot::Term {
            section: section.to_string(),
            en: en.into(),
        }
    }

    pub fn phrase(phrase: Phrase) -> Slot {
        Slot::Phrase {
            phrase: Box::new(phrase),
        }
    }

    fn render(&self, locale: Locale) -> String {
        match self {
            Slot::Text { text } => text.clone(),
            Slot::Number { text } => locale.number(text.clone()),
            Slot::Term { section, en } => locale
                .lookup(&format!("{section}.{en}"))
                .map_or_else(|| en.clone(), str::to_string),
            Slot::Phrase { phrase } => phrase.render(locale),
            Slot::List { items } => {
                let parts: Vec<String> = items.iter().map(|item| item.render(locale)).collect();
                join_list(&parts, locale)
            }
        }
    }
}

/// `a`, `a and b`, `a, b and c` — in the reader's list grammar.
///
/// Two keys rather than a hardcoded `" and "`: the separator between all
/// but the last, and a template for the final join. French needs only the
/// words changed; a language that puts no conjunction at all writes
/// `"{head}{last}"` and gets what it wants without a line of Rust.
fn join_list(parts: &[String], locale: Locale) -> String {
    match parts.split_last() {
        None => String::new(),
        Some((last, [])) => last.clone(),
        Some((last, rest)) => {
            let separator = locale.t("look.list-separator", ", ");
            locale.fill(
                "look.list-final",
                "{head} and {last}",
                &[("head", &rest.join(separator)), ("last", last)],
            )
        }
    }
}

/// The clauses as one sentence, in `locale`.
///
/// The clause separator and the full stop are catalogue entries too. They
/// look like punctuation rather than language right up until the language
/// is one that does not use them — Chinese ends a sentence with `。` —
/// and a `push('.')` in the Rust is exactly the kind of English-shaped
/// assumption this whole change exists to remove.
pub fn compose(clauses: &[Phrase], locale: Locale) -> String {
    if clauses.is_empty() {
        return String::new();
    }
    let rendered: Vec<String> = clauses.iter().map(|c| c.render(locale)).collect();
    let mut text = rendered.join(locale.t("look.clause-separator", ", "));
    text.push_str(locale.t("look.full-stop", "."));
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => text,
    }
}
