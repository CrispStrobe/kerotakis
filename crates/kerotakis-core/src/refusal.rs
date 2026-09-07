//! A refusal the learner can read in their own language.
//!
//! The bench says no often, and saying no well is most of what a teaching
//! engine does. Until now it said no in English: [`crate::bench::BenchError`]
//! composed a finished sentence with `format!`, the hosts called
//! `to_string()` on it, and the sentence travelled to the browser as an
//! exception message. A German session therefore read *"no vessel v2 —
//! make it first with `new`"* in the middle of an otherwise German bench.
//!
//! There was nothing to translate, and that is the point worth naming: a
//! finished sentence on the wire is not an untranslated string, it is an
//! **untranslatable** one. No catalogue can reach inside it, because by
//! the time anything knows the language the words have already been
//! chosen and the values baked into them.
//!
//! # The shape
//!
//! A refusal is a **key**, the **English source text** with named holes,
//! and the **values** for those holes:
//!
//! ```ignore
//! Refusal::new("error.no-such-vessel", "no vessel {vessel} — make it first …")
//!     .with("vessel", VesselId(1))
//! ```
//!
//! which renders through [`Locale::fill`] exactly as every event already
//! does. English is what the call site wrote; German is one row in
//! `i18n/de.toml`; French is the same row in a file that does not exist
//! yet, and adding it is a file rather than a code change.
//!
//! # Why the English stays at the call site
//!
//! Same reason it does for events (see [`crate::i18n`]): it is the source
//! text, it keeps the line legible without opening another file, and it is
//! the per-key fallback. `Display` for a refusal is `render(Locale::EN)`,
//! so the English sentence a CLI user, a log line and a golden fixture see
//! is *generated from the same template the translation replaces*. There
//! is no second copy of the English to drift.
//!
//! # Why the values are pre-formatted strings
//!
//! `{imbalance_g:.2}` is a decision about how much precision to show, and
//! it belongs to the engine, not to a translator. So the caller formats
//! and the catalogue only chooses the sentence. Numbers passed through
//! [`Refusal::with_number`] additionally get the reader's decimal
//! separator — German writes 0,10 g — the same way `render.rs` does it.
//! Identifiers do not: `v1` is a vessel's NAME and a comma would rename
//! it.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::i18n::Locale;

/// A refusal, in the shape that can be translated: a key, its English, and
/// what goes in the holes.
///
/// Serialises as `{"key": …, "en": …, "params": {…}}` so a host that would
/// rather write the sentence itself can, and so a log carries the reason
/// in a form later code can still group and count. Nothing deserialises it
/// — a refusal is produced, sent, and read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Refusal {
    /// A stable name for the PLACE this refusal is written, never for the
    /// phrase — `error.no-such-vessel`, not `make-it-first-with-new` — so
    /// that rewording the English does not orphan every translation of it.
    pub key: &'static str,
    /// The English source text, with named holes.
    pub en: &'static str,
    /// What goes in the holes, already formatted.
    pub params: BTreeMap<&'static str, String>,
    /// Which of those holes hold a number, and therefore want the
    /// reader's decimal separator. Not on the wire: it is about how to
    /// render, not about what was refused.
    #[serde(skip)]
    numeric: BTreeSet<&'static str>,
}

impl Refusal {
    /// A refusal with no holes.
    pub fn new(key: &'static str, en: &'static str) -> Self {
        Refusal {
            key,
            en,
            params: BTreeMap::new(),
            numeric: BTreeSet::new(),
        }
    }

    /// Fill one hole with something that already knows how to write itself
    /// — a vessel id, a species, a unit label.
    #[must_use]
    pub fn with(mut self, name: &'static str, value: impl std::fmt::Display) -> Self {
        self.params.insert(name, value.to_string());
        self
    }

    /// Fill one hole with a number, pre-formatted to the precision the
    /// engine means to show.
    ///
    /// Separate from [`Refusal::with`] because the decimal separator is a
    /// property of the reader's language and the precision is a property
    /// of the measurement: `format!("{x:.2}")` here, `0,10` on a German
    /// screen, and neither decision made in the other's place.
    #[must_use]
    pub fn with_number(mut self, name: &'static str, formatted: String) -> Self {
        self.numeric.insert(name);
        self.params.insert(name, formatted);
        self
    }

    /// The sentence, in `locale`.
    ///
    /// A key the catalogue has not got renders the English at the call
    /// site — per key, never per language, so a half-written translation
    /// reads half German rather than not at all.
    pub fn render(&self, locale: Locale) -> String {
        let localized: Vec<(&str, String)> = self
            .params
            .iter()
            .map(|(name, value)| {
                let value = if self.numeric.contains(name) {
                    locale.number(value.clone())
                } else {
                    value.clone()
                };
                (*name, value)
            })
            .collect();
        let vars: Vec<(&str, &str)> = localized
            .iter()
            .map(|(name, value)| (*name, value.as_str()))
            .collect();
        locale.fill(self.key, self.en, &vars)
    }
}

/// The English, so that `to_string()` on anything holding a refusal keeps
/// meaning what it meant: the source sentence, generated from the very
/// template a translation replaces.
impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.render(Locale::EN))
    }
}

/// Something that can say why it refused, in any language.
///
/// A trait rather than an inherent method so that a host can take
/// `&dyn Refuses` and not care which layer said no — the bench, the
/// parser, or a solver — which is what keeps the two bindings' error paths
/// one path rather than two that have to be kept in step by hand.
pub trait Refuses {
    /// The refusal, as key and holes.
    fn refusal(&self) -> Refusal;

    /// The refusal as a sentence in `locale`.
    fn localize(&self, locale: Locale) -> String {
        self.refusal().render(locale)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_is_the_call_site_text() {
        let r = Refusal::new("error.probe", "no vessel {vessel}").with("vessel", "v2");
        assert_eq!(r.to_string(), "no vessel v2");
        assert_eq!(r.render(Locale::EN), "no vessel v2");
    }

    #[test]
    fn a_key_without_german_still_reads() {
        // The fallback that lets a translation ship unfinished.
        let r = Refusal::new("error.nothing-translated-here", "amount must be positive");
        assert_eq!(r.render(Locale::parse("de")), "amount must be positive");
    }

    #[test]
    fn a_number_gets_the_readers_separator_and_an_identifier_does_not() {
        // `v1.2` is not a number; it is a name, and a comma renames it.
        let r = Refusal::new("error.probe", "{vessel}: {amount}")
            .with("vessel", "v1.2")
            .with_number("amount", format!("{:.2}", 0.1));
        assert_eq!(r.render(Locale::EN), "v1.2: 0.10");
        // No German for `error.probe`, so the English template stands and
        // only the number moves — which is exactly the split being tested.
        assert_eq!(r.render(Locale::parse("de")), "v1.2: 0,10");
    }

    #[test]
    fn params_are_ordered_so_the_wire_is_stable() {
        let r = Refusal::new("error.probe", "{b} {a}")
            .with("b", "2")
            .with("a", "1");
        let json = serde_json::to_string(&r).expect("a refusal serialises");
        assert!(json.contains(r#""params":{"a":"1","b":"2"}"#), "{json}");
        // `numeric` is a rendering detail, not part of what was refused.
        assert!(!json.contains("numeric"), "{json}");
    }
}
