//! The engine's own words, in the reader's language.
//!
//! The catalogue translates itself with `_de` sibling keys, but the lines
//! the engine composes — the vessel summary, the journal — are built here
//! out of fragments, so no amount of data can reach them. This is where
//! the engine keeps its vocabulary.
//!
//! # Adding a language
//!
//! Two steps, and neither touches an existing translation:
//!
//! 1. Copy `i18n/de.toml` to `i18n/<code>.toml` and translate the values.
//! 2. Add one line to [`CATALOGUES`].
//!
//! That is the whole contract. Two translators working on two languages
//! never edit the same file, which is the property that makes the work
//! parallelisable — it is why the previous shape, a `say(en, de)` taking
//! both languages as arguments, had to go: it could not express a third
//! language at all without changing every call site.
//!
//! # Why the English stays in the Rust
//!
//! Call sites read `locale.t("vessel.open", ", open to atmosphere")`. The
//! English is the source text: keeping it inline means the line is legible
//! without opening another file, a reviewer can see what is being said,
//! and there is always something to fall back on. A key missing from a
//! catalogue renders English — never a blank, never the key itself.
//!
//! # Why keys name a place, not a phrase
//!
//! `vessel.open`, not `open-to-atmosphere`. Rewording the English source
//! then does not orphan every translation of it, which is the failure that
//! makes message catalogues rot.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

/// Every language the engine can speak, and its catalogue.
///
/// Add a line here to add a language. The file is parsed once, lazily, on
/// the first line rendered in that language.
const CATALOGUES: &[(&str, &str)] = &[("de", include_str!("../i18n/de.toml"))];

/// Which language the engine speaks when it turns state into prose.
///
/// Beside [`crate::render::Register`], deliberately: the register says how
/// much to say and this says in what language, and they are the same kind
/// of decision about the same sentence. It belongs to the engine rather
/// than to each host because every host renders the same line — if the
/// CLI, the web app and the Mac app each translated for themselves they
/// would drift into three vocabularies, and PROTOCOL.md's rule that the UI
/// must not be able to tell the transports apart covers the words as much
/// as the numbers.
///
/// A code rather than an enum variant per language, so that adding one is
/// a data change. `Locale::parse` accepts anything and falls back to
/// English: someone whose system is set to a language nobody has
/// translated should see the language we do have, not an error where the
/// bench used to be.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Locale(&'static str);

impl Default for Locale {
    fn default() -> Self {
        Locale::EN
    }
}

impl Locale {
    /// The source language. Always available, never looked up.
    pub const EN: Locale = Locale("en");

    /// Parse a BCP-47-ish tag (`de`, `de-AT`, `fr-CA`) against the
    /// languages actually shipped. Unknown input is English.
    pub fn parse(tag: &str) -> Locale {
        let t = tag.trim().to_ascii_lowercase();
        let primary = t.split(['-', '_']).next().unwrap_or("");
        CATALOGUES
            .iter()
            .find(|(code, _)| *code == primary)
            .map(|(code, _)| Locale(code))
            .unwrap_or(Locale::EN)
    }

    /// The language tag, as it would be written in `lang=`.
    pub fn code(self) -> &'static str {
        self.0
    }

    pub fn is_english(self) -> bool {
        self.0 == "en"
    }

    /// Every language the engine ships, English first.
    pub fn available() -> Vec<Locale> {
        std::iter::once(Locale::EN)
            .chain(CATALOGUES.iter().map(|(code, _)| Locale(code)))
            .collect()
    }

    /// What to say for `key`, falling back to the English at the call site.
    ///
    /// The fallback is per key, not per language: a catalogue that has
    /// translated half its keys renders half German and half English
    /// rather than nothing, which is what lets a translation ship while it
    /// is still being written.
    pub fn t(self, key: &str, en: &'static str) -> &'static str {
        if self.is_english() {
            return en;
        }
        catalogue(self.0)
            .and_then(|c| c.get(key))
            .map(String::as_str)
            .unwrap_or(en)
    }

    /// A message with named placeholders filled in.
    ///
    /// ```ignore
    /// locale.fill(
    ///     "event.added.lv1",
    ///     "You add {what} to {vessel}.",
    ///     &[("what", name), ("vessel", vessel)],
    /// )
    /// ```
    ///
    /// Named, not positional, and this is the whole reason the method
    /// exists rather than a `format!` at each call site. `format!("You add
    /// {name} to {vessel}")` hardcodes ENGLISH WORD ORDER into the code: a
    /// language that puts the vessel first, or that needs the verb last,
    /// cannot be expressed by any translation of the fragments, only by
    /// rewriting the call. With named holes the translator writes the
    /// whole sentence in their own order and the code does not care.
    ///
    /// A placeholder with no value is left as it is written rather than
    /// blanked, so a typo in a catalogue shows up as `{vesel}` on screen —
    /// visible, reportable, and obviously a bug — instead of a hole the
    /// reader silently mis-reads as intended.
    pub fn fill(self, key: &str, en: &'static str, vars: &[(&str, &str)]) -> String {
        let template = self.t(key, en);
        if vars.is_empty() || !template.contains('{') {
            return template.to_string();
        }
        let mut out = String::with_capacity(template.len() + 16);
        let mut rest = template;
        while let Some(open) = rest.find('{') {
            let Some(close) = rest[open..].find('}').map(|i| open + i) else {
                break;
            };
            let name = &rest[open + 1..close];
            match vars.iter().find(|(k, _)| *k == name) {
                Some((_, value)) => {
                    out.push_str(&rest[..open]);
                    out.push_str(value);
                }
                // Unknown name: keep the braces, so it reads as a fault.
                None => out.push_str(&rest[..=close]),
            }
            rest = &rest[close + 1..];
        }
        out.push_str(rest);
        out
    }

    /// What to say for `key` when there is no English source line to fall
    /// back to — a lookup keyed by a value rather than by a place, such as
    /// a vessel's own label.
    ///
    /// Returns `None` rather than inventing a word, so the caller can keep
    /// the original: a vessel nobody has named in German should read in
    /// English inside an otherwise German sentence, not vanish.
    pub fn lookup(self, key: &str) -> Option<&'static str> {
        if self.is_english() {
            return None;
        }
        catalogue(self.0)
            .and_then(|c| c.get(key))
            .map(String::as_str)
    }

    /// German writes 1,5 where English writes 1.5.
    ///
    /// Prose only. Identifiers keep the point — `v1.2` is a vessel's NAME,
    /// and swapping its separator renames it — so callers apply this to
    /// the measured part of a line and not to the whole of it.
    pub fn number(self, text: String) -> String {
        match self.0 {
            "en" => text,
            // Every language shipped so far that is not English uses the
            // comma. When one arrives that does not, this becomes a field
            // in the catalogue rather than a match arm.
            _ => text.replace('.', ","),
        }
    }
}

/// Parse a catalogue once, on first use.
fn catalogue(code: &str) -> Option<&'static HashMap<String, String>> {
    static PARSED: OnceLock<HashMap<&'static str, HashMap<String, String>>> = OnceLock::new();
    PARSED
        .get_or_init(|| {
            CATALOGUES
                .iter()
                .map(|(code, text)| (*code, flatten(text, code)))
                .collect()
        })
        .get(code)
}

/// `[vessel] open = "…"` becomes `vessel.open`.
///
/// Sections are for the humans editing the file; the code asks for a
/// dotted path. A malformed catalogue yields an empty map rather than a
/// panic — a broken translation must not take the engine down, it must
/// render English.
fn flatten(text: &str, code: &str) -> HashMap<String, String> {
    let parsed: toml::Value = match toml::from_str(text) {
        Ok(v) => v,
        Err(e) => {
            debug_assert!(false, "i18n/{code}.toml does not parse: {e}");
            return HashMap::new();
        }
    };
    let mut out = HashMap::new();
    if let toml::Value::Table(top) = parsed {
        for (section, body) in top {
            match body {
                toml::Value::Table(entries) => {
                    for (k, v) in entries {
                        if let Some(s) = v.as_str() {
                            out.insert(format!("{section}.{k}"), s.to_string());
                        }
                    }
                }
                toml::Value::String(s) => {
                    out.insert(section, s);
                }
                _ => {}
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_shipped_catalogue_parses() {
        // A catalogue that does not parse degrades to English silently,
        // which is the right behaviour at runtime and the wrong thing to
        // discover in production.
        for (code, text) in CATALOGUES {
            let map = flatten(text, code);
            assert!(!map.is_empty(), "i18n/{code}.toml parsed to nothing");
        }
    }

    #[test]
    fn an_unknown_tag_is_english() {
        // Deliberately not a real language code: pinning "fr" here would
        // fail the day someone adds French, from a test that is not about
        // French. `zz` is reserved for exactly this.
        assert_eq!(Locale::parse("zz"), Locale::EN);
        assert_eq!(Locale::parse(""), Locale::EN);
        assert_eq!(Locale::parse("   "), Locale::EN);
    }

    #[test]
    fn a_regional_tag_finds_its_language() {
        assert_eq!(Locale::parse("de-AT").code(), "de");
        assert_eq!(Locale::parse("de_CH").code(), "de");
        assert_eq!(Locale::parse("DE").code(), "de");
    }

    #[test]
    fn a_missing_key_renders_the_english_at_the_call_site() {
        let de = Locale::parse("de");
        assert_eq!(
            de.t("vessel.open", ", open to atmosphere"),
            ", zur Atmosphäre offen"
        );
        assert_eq!(
            de.t("nothing.translated.yet", "the English original"),
            "the English original"
        );
    }

    #[test]
    fn lookup_declines_rather_than_inventing() {
        let de = Locale::parse("de");
        assert_eq!(de.lookup("glassware.beaker"), Some("Becherglas"));
        assert_eq!(de.lookup("glassware.spectrophotometer cell"), None);
        // English never looks anything up: its words are already at the
        // call site.
        assert_eq!(Locale::EN.lookup("glassware.beaker"), None);
    }

    #[test]
    fn fill_substitutes_named_holes() {
        let en = Locale::EN;
        assert_eq!(
            en.fill(
                "x",
                "You add {what} to {vessel}.",
                &[("what", "salt"), ("vessel", "v1")]
            ),
            "You add salt to v1."
        );
    }

    #[test]
    fn a_translation_may_reorder_the_holes() {
        // The point of named placeholders. A positional format string
        // fixes English word order in the CODE; here the translated
        // sentence puts the vessel first and nothing else has to change.
        let en = Locale::EN;
        let reordered = en.fill(
            "x",
            "{vessel} gets {what}.",
            &[("what", "salt"), ("vessel", "v1")],
        );
        assert_eq!(reordered, "v1 gets salt.");
    }

    #[test]
    fn an_unknown_hole_stays_visible() {
        // A typo in a catalogue should read as a fault on screen, not as a
        // gap the reader silently takes for intentional.
        let en = Locale::EN;
        assert_eq!(
            en.fill("x", "in {vesel}", &[("vessel", "v1")]),
            "in {vesel}"
        );
    }

    #[test]
    fn a_repeated_hole_is_filled_every_time() {
        let en = Locale::EN;
        assert_eq!(en.fill("x", "{v} into {v}", &[("v", "v1")]), "v1 into v1");
    }

    #[test]
    fn available_lists_english_first() {
        let all = Locale::available();
        assert_eq!(all[0], Locale::EN);
        assert!(all.iter().any(|l| l.code() == "de"));
    }
}
