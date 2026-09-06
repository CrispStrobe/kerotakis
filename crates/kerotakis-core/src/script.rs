//! The `.lab` script grammar: one line, one bench command.
//!
//! This lives in the core so that every client runs the *same* lessons —
//! the CLI, the wasm build, and anything later. A lesson is data, and its
//! grammar is part of the engine rather than of one front end.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use crate::i18n::Locale;
use crate::material::{self, MaterialBasis, MaterialRecipe};
use crate::ops::{Compare, Endpoint, Instrument, Operator};
use crate::species::{self, SpeciesData, SpeciesId};
use crate::units::{Grams, Joules, Kelvin, Liters, Moles, Pascal};
use crate::vessel::{VesselId, VESSEL_KINDS};

/// Parse one bench command into an operator. Meta commands (register,
/// inspect) return `None` — they are session state, not bench state.
/// The grammar's public inventory: every verb `parse_op` accepts, with a
/// canonical example line (GUI-029). A UI's affordance manifest is checked
/// against this list by the protocol conformance suite, and the test at
/// the bottom of this file keeps each example honest against the parser.
/// Aliases share their canonical verb's row.
pub const VERBS: &[(&str, &str)] = &[
    ("new", "new"),
    ("remove", "remove v1"),
    ("add", "add v1 water 100mL"),
    ("stock", "stock NaCl 0.5mol"),
    ("heat", "heat v1 10kJ"),
    ("cool", "cool v1 10kJ"),
    ("wait", "wait 30s"),
    ("ignite", "ignite v1"),
    ("stir", "stir v1 500rpm 10s"),
    ("seal", "seal v1 500mL"),
    ("regulate", "regulate v1 1.5bar 500mL"),
    ("sweep", "sweep v1 1bar"),
    ("open", "open v1"),
    ("filter", "filter v1 v2"),
    ("evaporate", "evaporate v1 0.5"),
    ("decant", "decant v1 v2 0.5"),
    ("drain", "drain v1 v2"),
    ("distil", "distil v1 v2 0.5"),
    ("measure", "measure v1 ph"),
    ("chromatograph", "chromatograph v1"),
    ("electrolyse", "electrolyse v1 0.5A 30min"),
    ("cell", "cell v1 v2"),
    ("grind", "grind v1 NaCl 50um"),
    ("centrifuge", "centrifuge v1 3000rpm 60s 8cm 100g"),
    ("irradiate", "irradiate v1 254nm 10W/m2"),
    ("dilute", "dilute v1 100mL"),
    ("titrate", "titrate v1 NaOH 1M 1mL until ph 7"),
    ("magnet", "magnet v1 v2"),
    ("transport", "transport v1 v2 v3 from v4 to v5 steps 3"),
    ("react", "react v1 esterification"),
    ("test", "test v1 pop"),
    ("particles", "particles v1"),
    ("smell", "smell v1"),
];

/// The verbs the parser accepts that `VERBS` does not list, because the
/// inventory keeps one row per idea and each of these is a second word
/// for a row that is already there — `look` for the eyes, `mix` for a
/// two-vessel pour, `voltmeter` for touching two half-cells together.
///
/// A whole verb to the person typing it, so each needs its own word in
/// every language: `i18n_coverage` gates them beside `VERBS`.
pub const VERB_SYNONYMS: &[&str] = &["look", "observe", "waft", "zoom", "mix", "voltmeter"];

/// The same word, spelt the other way. Nothing to translate — a language
/// that is not English has one spelling of its own word — so these are
/// kept apart from the synonyms above and gated by neither.
pub const VERB_SPELLINGS: &[&str] = &["distill", "electrolyze"];

/// The words that ask the SHELL something rather than the bench. They
/// never become operators, and they are English words this grammar
/// already spends, so no alias may claim one.
pub const SESSION_WORDS: &[&str] = &[
    "register",
    "inspect",
    "explain",
    "species",
    "help",
    "structure",
    "identify",
    "coverage",
];

/// Every word `measure` accepts, and the instrument it names.
///
/// A table rather than a match arm because two other things now have to
/// read it: the alias layer, which may not hand a language a word this
/// list already spends, and the test that walks it. A match arm is
/// readable and unenumerable, and the second property is the expensive
/// one.
pub const INSTRUMENT_WORDS: &[(&str, Instrument)] = &[
    ("thermometer", Instrument::Thermometer),
    ("temp", Instrument::Thermometer),
    ("balance", Instrument::Balance),
    ("mass", Instrument::Balance),
    ("ph", Instrument::PhMeter),
    ("phmeter", Instrument::PhMeter),
    ("eyes", Instrument::Eyes),
    ("look", Instrument::Eyes),
    ("pressure", Instrument::PressureGauge),
    ("gauge", Instrument::PressureGauge),
    ("volume", Instrument::VolumeMeter),
    ("conductivity", Instrument::ConductivityMeter),
    ("density", Instrument::Densitometer),
    ("hydrometer", Instrument::Densitometer),
    ("densitometer", Instrument::Densitometer),
    ("spectrophotometer", Instrument::Spectrophotometer),
    ("uvvis", Instrument::Spectrophotometer),
    ("calorimeter", Instrument::Calorimeter),
    ("chromatograph", Instrument::Chromatograph),
    ("column", Instrument::Chromatograph),
    ("geiger", Instrument::GeigerCounter),
    // EXP-33. Both spellings, because a learner types the quantity and a
    // technician types the apparatus.
    ("melting_point", Instrument::MeltingPointApparatus),
    ("melting-point", Instrument::MeltingPointApparatus),
    ("mp", Instrument::MeltingPointApparatus),
    ("boiling_point", Instrument::BoilingPointApparatus),
    ("boiling-point", Instrument::BoilingPointApparatus),
    ("bp", Instrument::BoilingPointApparatus),
];

/// The instrument a word names, or nothing.
pub fn instrument_by_word(word: &str) -> Option<Instrument> {
    INSTRUMENT_WORDS
        .iter()
        .find(|(name, _)| *name == word)
        .map(|(_, instrument)| *instrument)
}

/// The classical gas tests `test <vessel> <name>` runs. Kept beside the
/// match that reads them so the alias layer can see the English.
pub const GAS_TEST_WORDS: &[&str] = &["pop", "splint", "limewater", "litmus"];

/// The flames `heat … on <source>` accepts, as `HeatSource::by_name`
/// spells them.
pub const HEAT_SOURCE_WORDS: &[&str] = &[
    "burner",
    "bunsen",
    "bunsenburner",
    "bunsen-burner",
    "hotplate",
    "hot-plate",
    "plate",
    "candle",
];

/// The small words that hold a command together and mean nothing on
/// their own.
pub const GRAMMAR_WORDS: &[&str] = &[
    "until", "into", "from", "to", "steps", "courant", "max", "stages", "persists", "colour",
    "color", "ph", "pe", "above", "below", "on", "with", "@",
];

/// Is this word already a verb of the English grammar?
pub fn is_canonical_verb(word: &str) -> bool {
    let word = word.to_ascii_lowercase();
    let word = word.as_str();
    VERBS.iter().any(|(verb, _)| *verb == word)
        || VERB_SYNONYMS.contains(&word)
        || VERB_SPELLINGS.contains(&word)
        || SESSION_WORDS.contains(&word)
}

/// Is this word already spent somewhere else in the English grammar —
/// an instrument, a gas test, a flame, a glassware kind, a joining word,
/// or the key of a species on the shelf?
///
/// Species keys are compared in lower case on purpose. `PE` is
/// polyethylene and `pe` is the redox endpoint; a language that wanted
/// `pe` for something of its own would collide with both, and the point
/// of this check is to catch exactly that before it reaches a learner.
fn is_canonical_word(word: &str) -> bool {
    let lower = word.to_ascii_lowercase();
    let lower = lower.as_str();
    INSTRUMENT_WORDS.iter().any(|(name, _)| *name == lower)
        || GAS_TEST_WORDS.contains(&lower)
        || HEAT_SOURCE_WORDS.contains(&lower)
        || GRAMMAR_WORDS.contains(&lower)
        || VESSEL_KINDS.iter().any(|(kind, _)| *kind == lower)
        || is_canonical_verb(lower)
        || species::registry()
            .iter()
            .any(|data| data.key.eq_ignore_ascii_case(lower))
}

// ── The alias layer (I18N) ──────────────────────────────────────────
//
// The canonical script is English and stays English: a lesson, a saved
// session, the operator log, the corpus and the replay cache all carry
// English lines, so a session typed in German replays byte-identically
// on a machine that has never heard of German. What a language gets is
// an alias layer READ AT PARSE TIME and rewritten away before anything
// is stored.
//
// Every alias is data. `crates/kerotakis-core/i18n/<code>.toml` carries
// `[script-verb]`, `[script-instrument]`, `[script-test]`,
// `[script-source]` and `[script-word]`, keyed by the canonical token
// with a comma-separated list of that language's words for it; species
// come from `[species]` read backwards and glassware from `[glassware]`.
// Adding French is adding `fr.toml`. There is no German in this file,
// and there must not be — a match arm here is a language the next
// translator cannot add without a Rust change.

/// One language's rewrite tables: the first word, and every word after
/// it.
#[derive(Default)]
struct AliasIndex {
    verbs: HashMap<String, String>,
    words: HashMap<String, String>,
    /// The other direction, for the words a UI OFFERS rather than the
    /// words it accepts: canonical verb → the first alias its translator
    /// listed, and canonical substance or glassware → its name in this
    /// language. Only those two: a hint is worth having because it is a
    /// line the learner could have typed, and `until pe > 8` stays
    /// itself in every language.
    verb_display: HashMap<String, String>,
    word_display: HashMap<String, String>,
}

/// Claim `alias` for `canonical`, honouring the two rules.
///
/// English wins: a word this grammar already spends is never taken over
/// by a translation of something else. And an alias claimed twice is
/// dropped rather than resolved, because the alternative is a bench that
/// does one of two things depending on which section was read first.
fn claim(
    map: &mut HashMap<String, String>,
    dropped: &mut HashSet<String>,
    alias: &str,
    canonical: &str,
    already_english: impl Fn(&str) -> bool,
) {
    let alias = alias.trim().to_lowercase();
    if alias.is_empty() || alias.contains(char::is_whitespace) || dropped.contains(&alias) {
        return;
    }
    if already_english(&alias) {
        return;
    }
    match map.get(&alias).cloned() {
        Some(existing) if existing.as_str() != canonical => {
            map.remove(&alias);
            dropped.insert(alias);
        }
        Some(_) => {}
        None => {
            map.insert(alias, canonical.to_string());
        }
    }
}

fn split_aliases(list: &str) -> impl Iterator<Item = &str> {
    list.split(',').map(str::trim).filter(|a| !a.is_empty())
}

fn build_alias_index(locale: Locale) -> AliasIndex {
    let mut index = AliasIndex::default();
    let mut dropped_verbs = HashSet::new();
    let mut dropped_words = HashSet::new();
    // Every (canonical, alias) pair in the order the catalogue lists it,
    // so the display pass below can take the FIRST alias that survived —
    // the one the translator put first — without depending on a map's
    // iteration order.
    let mut verb_order: Vec<(String, String)> = Vec::new();
    let mut name_order: Vec<(String, String)> = Vec::new();

    for (canonical, list) in locale.section("script-verb") {
        debug_assert!(
            is_canonical_verb(canonical),
            "i18n/{}.toml [script-verb] names '{canonical}', which is no verb of this grammar",
            locale.code()
        );
        if !is_canonical_verb(canonical) {
            continue;
        }
        for alias in split_aliases(list) {
            claim(
                &mut index.verbs,
                &mut dropped_verbs,
                alias,
                canonical,
                is_canonical_verb,
            );
            verb_order.push((canonical.to_string(), alias.to_string()));
        }
    }

    for section in [
        "script-instrument",
        "script-test",
        "script-source",
        "script-word",
    ] {
        for (canonical, list) in locale.section(section) {
            debug_assert!(
                is_canonical_word(canonical),
                "i18n/{}.toml [{section}] names '{canonical}', which this grammar never accepts",
                locale.code()
            );
            for alias in split_aliases(list) {
                claim(
                    &mut index.words,
                    &mut dropped_words,
                    alias,
                    canonical,
                    is_canonical_word,
                );
            }
        }
    }

    // Glassware and species need no table of their own: the catalogue
    // already carries both, for the sentences the bench writes. Read
    // backwards they are the words a learner may type — which is the
    // whole reason a translator should never have to write a name twice.
    for (kind, _) in VESSEL_KINDS {
        if let Some(name) = locale.lookup(&format!("glassware.{kind}")) {
            claim(
                &mut index.words,
                &mut dropped_words,
                name,
                kind,
                is_canonical_word,
            );
            name_order.push(((*kind).to_string(), name.to_string()));
        }
    }
    for data in species::registry() {
        let Some(name) = locale.lookup(&format!("species.{}", data.name)) else {
            continue;
        };
        // The shelf has two halves and German sometimes has one word for
        // both: `Ammoniumnitrat` is the pure substance `NH4NO3` and also
        // the bottle `ammonium_nitrate`, which English tells apart by
        // spelling them differently. Rule 1 applies across the halves as
        // well — the word is left alone, and the bottle answers it the
        // way it always has.
        if material::lookup(name, None).is_some() {
            dropped_words.insert(name.to_lowercase());
            index.words.remove(&name.to_lowercase());
            continue;
        }
        claim(
            &mut index.words,
            &mut dropped_words,
            name,
            data.key,
            is_canonical_word,
        );
        name_order.push((data.key.to_string(), name.to_string()));
    }

    // The display pass: the first alias that actually survived, which is
    // never a word that was dropped for colliding with English or with
    // another canonical token. A hint the parser would refuse is worse
    // than no hint.
    for (canonical, alias) in verb_order {
        if index.verbs.get(&alias.to_lowercase()) == Some(&canonical) {
            index.verb_display.entry(canonical).or_insert(alias);
        }
    }
    for (canonical, name) in name_order {
        if index.words.get(&name.to_lowercase()) == Some(&canonical) {
            index.word_display.entry(canonical).or_insert(name);
        }
    }
    index
}

/// The rewrite tables for one language, built once.
fn alias_index(locale: Locale) -> &'static AliasIndex {
    static INDEXES: OnceLock<HashMap<&'static str, AliasIndex>> = OnceLock::new();
    static EMPTY: OnceLock<AliasIndex> = OnceLock::new();
    INDEXES
        .get_or_init(|| {
            Locale::available()
                .into_iter()
                .filter(|locale| !locale.is_english())
                .map(|locale| (locale.code(), build_alias_index(locale)))
                .collect()
        })
        .get(locale.code())
        .unwrap_or_else(|| EMPTY.get_or_init(AliasIndex::default))
}

/// Rewrite a command line from `locale` into the canonical English
/// grammar, or `None` when it already is canonical.
///
/// Word by word, and never over a word the English grammar itself
/// spends: a line that parses in English parses to exactly the same
/// operator in every language, which is what keeps the shipped lessons
/// and the replay cache out of this entirely.
///
/// Materials are looked up rather than tabulated. `material::lookup`
/// already matches a recipe's `aliases.<lang>` in any language — it is
/// how `add v1 Milch 100mL` has always worked — so the only thing
/// missing was the rewrite back to `whole_milk` for the log, and a pack
/// loaded at runtime gets it for free.
pub fn canonical_line_in(line: &str, locale: Locale) -> Option<String> {
    if locale.is_english() {
        return None;
    }
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }
    let index = alias_index(locale);
    let mut changed = false;
    let mut out: Vec<String> = Vec::new();
    for (position, word) in trimmed.split_whitespace().enumerate() {
        let lower = word.to_lowercase();
        let canonical = if position == 0 {
            if is_canonical_verb(word) {
                None
            } else {
                index.verbs.get(&lower).cloned()
            }
        } else if is_canonical_word(word) || species::lookup_key(word).is_some() {
            None
        } else {
            index.words.get(&lower).cloned().or_else(|| {
                // Not a number, not a vessel: those are most of a line and
                // none of them is ever a bottle on the shelf.
                if word.starts_with(|c: char| c.is_ascii_digit()) {
                    return None;
                }
                material::lookup(word, None)
                    // A bottle the English grammar can already name keeps
                    // the name that was typed. Rewriting `milk` to
                    // `whole_milk` because a German is at the keyboard
                    // would make one lesson two scripts, and the point of
                    // this layer is that it makes none.
                    .filter(|recipe| !recipe.matches(word, Some("en")))
                    .map(|recipe| recipe.canonical_key)
            })
        };
        match canonical {
            Some(canonical) => {
                changed = true;
                out.push(canonical);
            }
            None => out.push(word.to_string()),
        }
    }
    changed.then(|| out.join(" "))
}

/// A canonical example line as a learner of `locale` may type it, or
/// `None` when it is already the line they would write.
///
/// The inverse of [`canonical_line_in`], and only for what a UI OFFERS —
/// the command bar's hints and its placeholder. The verb and the
/// substance names change; the numbers, the units and the joining words
/// do not, because those are the same in every language this grammar has
/// and because a hint must be a line the parser takes. That last is a
/// test, not a hope: every hint is fed back through `canonical_line_in`
/// and must come out as the example it was made from.
pub fn example_in(line: &str, locale: Locale) -> Option<String> {
    if locale.is_english() {
        return None;
    }
    let index = alias_index(locale);
    let mut changed = false;
    let mut out: Vec<String> = Vec::new();
    for (position, word) in line.split_whitespace().enumerate() {
        let display = if position == 0 {
            index.verb_display.get(word)
        } else {
            index.word_display.get(word)
        };
        match display {
            Some(display) => {
                changed = true;
                out.push(display.clone());
            }
            None => out.push(word.to_string()),
        }
    }
    changed.then(|| out.join(" "))
}

/// One line as the bench understood it: the canonical English form to
/// log, and the operator it makes.
#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    /// What to echo, log and save. Always canonical, whatever was typed.
    pub canonical: String,
    /// `None` for a blank line, a comment, or a word the shell answers.
    pub operator: Option<Operator>,
}

/// Parse one line typed in `locale`.
///
/// The alias rewrite is tried first and the raw line second, so an alias
/// can never take a meaning English already had — a word that collides
/// is dropped when the index is built, and if a rewritten line does not
/// parse the untouched one still gets its turn.
pub fn parse_command(line: &str, locale: Locale) -> Result<Command, ParseError> {
    let canonical = canonical_line_in(line, locale);
    let attempt = canonical.as_deref().unwrap_or(line);
    match parse_op_typed(attempt) {
        Ok(operator) => Ok(Command {
            canonical: attempt.trim().to_string(),
            operator,
        }),
        Err(error) if canonical.is_some() => match parse_op_typed(line) {
            Ok(operator) => Ok(Command {
                canonical: line.trim().to_string(),
                operator,
            }),
            Err(_) => Err(localised(error, locale)),
        },
        Err(error) => Err(localised(error, locale)),
    }
}

/// The unknown-verb refusal, in the learner's language.
///
/// The English says "try 'help'", and a learner who has just typed a
/// German word has no reason to expect an English help screen to answer
/// them — so the German names the verbs themselves. Every other refusal
/// is left as it is: they are about a number or a name, not about the
/// vocabulary, and translating a grammar's whole error surface is a
/// different job from letting a learner type their own words.
fn localised(error: ParseError, locale: Locale) -> ParseError {
    if locale.is_english() {
        return error;
    }
    let Some(word) = error
        .detail
        .strip_prefix("unknown command '")
        .and_then(|rest| rest.split('\'').next())
    else {
        return error;
    };
    // The FIRST alias the catalogue lists for a verb, which is the one
    // its translator put first — and `Locale::section` sorts, so the
    // sentence is the same on every run and in every host.
    let index = alias_index(locale);
    let rows = locale.section("script-verb");
    let first_alias = |verb: &str| -> Option<String> {
        let list: &'static str = rows.iter().find(|(canonical, _)| *canonical == verb)?.1;
        split_aliases(list)
            .find(|alias| index.verbs.get(&alias.to_lowercase()).map(String::as_str) == Some(verb))
            .map(str::to_string)
    };
    let mut verbs: Vec<String> = VERBS
        .iter()
        .map(|(verb, _)| match first_alias(verb) {
            Some(alias) => format!("{alias} ({verb})"),
            None => (*verb).to_string(),
        })
        .collect();
    verbs.sort_unstable();
    let detail = locale.fill(
        "script.unknown-verb",
        "unknown command '{word}' — the bench knows these verbs: {verbs}",
        &[("word", word), ("verbs", &verbs.join(", "))],
    );
    ParseError {
        kind: error.kind,
        detail,
    }
}

/// The one usage line for `titrate`, kept in one place now that the verb
/// has three endpoints (EXP-39).
const TITRATE_USAGE: &str = "usage: titrate <vessel> <titrant> [<c>M] <step><mL|L> until \
                             <ph <target> | pe <op> <value> | colour persists> [max <n>]";

/// Refuse a number the operator log could not carry.
///
/// Rust parses `1e999` into `f64::INFINITY` without complaint and serde_json
/// refuses to write it, so a titration with an infinite target would run and
/// then make the bench unable to save itself. The grammar fuzz target found
/// this on the endpoint arm; the pH arm had the same hole since CAP-12.
/// KID-1: what to say when a name resolves to nothing.
///
/// The old text was `unknown species or material 'vinegar' (see 'species')`
/// — and `species` lists species only, so a newcomer looking for a household
/// bottle was sent to the one command that could not show it. The shelf has
/// two halves; the message now names both, and offers the closest thing it
/// actually holds.
pub fn unknown_ingredient(name: &str) -> String {
    match nearest_ingredient(name) {
        Some(hit) => format!(
            "unknown species or material '{name}' — did you mean '{hit}'? \
             ('species' lists the pure substances, 'materials' the household \
             and school bottles, 'find {name}' searches both)"
        ),
        None => format!(
            "unknown species or material '{name}' \
             ('species' lists the pure substances, 'materials' the household \
             and school bottles, 'find <word>' searches both)"
        ),
    }
}

/// The closest name the shelf actually carries, or nothing rather than a
/// guess: a suggestion further than a third of the query's length away is
/// noise, and noise in an error message is worse than silence.
///
/// `BRD-002`'s cabinet search answers this question already, and it answers
/// it better than a distance ever could — `vinegar` is not a typo for
/// `white_vinegar_5_percent`, it is a *substring* of it, and every learner
/// who reaches for the shorter word means the longer one. So the search
/// runs first, and edit distance is only the fallback for a genuine
/// misspelling like `watr`.
fn nearest_ingredient(name: &str) -> Option<String> {
    if let Some(hit) = crate::cabinet::search(name, 1).into_iter().next() {
        return Some(hit.key);
    }
    let query = name.to_lowercase();
    let budget = (query.chars().count() / 3).clamp(1, 5);
    let mut best: Option<(usize, String)> = None;
    let mut consider = |candidate: &str| {
        let distance = edit_distance(&query, &candidate.to_lowercase());
        if distance <= budget && best.as_ref().is_none_or(|(d, _)| distance < *d) {
            best = Some((distance, candidate.to_string()));
        }
    };
    for data in species::REGISTRY {
        consider(data.key);
    }
    for recipe in material::all() {
        consider(&recipe.canonical_key);
    }
    best.map(|(_, hit)| hit)
}

/// Levenshtein distance over chars, two rows at a time. Small inputs only —
/// this runs once, on the error path.
fn edit_distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    let mut current = vec![0usize; b.len() + 1];
    for (i, ca) in a.chars().enumerate() {
        current[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let substitution = previous[j] + usize::from(ca != *cb);
            current[j + 1] = substitution.min(previous[j + 1] + 1).min(current[j] + 1);
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[b.len()]
}

fn finite(value: f64, what: &str) -> Result<(), String> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(format!(
            "{what} must be a finite number — '{value}' cannot be written to \
             the operator log the bench saves itself with"
        ))
    }
}

/// The widest a *logarithmic* endpoint can sensibly be.
///
/// pH and pe are exponents. A pH of 6.7e49 is not a strong acid, it is a
/// number that got into a chemistry slot, and carrying it costs precision
/// in the operator log for no chemistry at all — the grammar fuzz target
/// produced exactly that. Ten to the ninety-ninth is already far past
/// anything an aqueous solver represents, so the range is generous and
/// the refusal is about arithmetic, not about taste.
const LOG_SCALE_LIMIT: f64 = 99.0;

fn log_scale(value: f64, what: &str) -> Result<(), String> {
    finite(value, what)?;
    if value.abs() <= LOG_SCALE_LIMIT {
        Ok(())
    } else {
        Err(format!(
            "{what} must lie within ±{LOG_SCALE_LIMIT} — {value} is an \
             exponent no aqueous solver represents"
        ))
    }
}

/// The pH slot's filler for the endpoints that do not consult it. Neutral
/// rather than a sentinel: `Operator::Titrate::target_ph` stays an ordinary
/// finite pH so the operator log stays plain JSON.
const NEUTRAL_PH: f64 = 7.0;

/// Stable parse failure classes for corpus coverage and clients. The legacy
/// `parse_op` API remains source-compatible; new callers should prefer
/// `parse_op_typed` when the reason is part of their data contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParseErrorKind {
    UnknownSpecies,
    UnknownReaction,
    InvalidSyntax,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{detail}")]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub detail: String,
}

pub fn parse_op_typed(line: &str) -> Result<Option<Operator>, ParseError> {
    let words = line.split_whitespace().collect::<Vec<_>>();
    let kind = match words.as_slice() {
        ["add", _, species, ..]
            if species::lookup_key(species).is_none()
                && crate::nuclide::lookup_notation(species).is_none()
                && material::lookup(species, None).is_none() =>
        {
            ParseErrorKind::UnknownSpecies
        }
        // BRD-002: `stock` names a shelf entry in the same vocabulary
        // `add` does, so an unknown one fails the same way.
        ["stock", key, ..]
            if species::lookup_key(key).is_none() && material::lookup(key, None).is_none() =>
        {
            ParseErrorKind::UnknownSpecies
        }
        ["react", _, reaction, ..]
            if !crate::curated::ORG_REACTIONS
                .iter()
                .any(|candidate| candidate.name == *reaction) =>
        {
            ParseErrorKind::UnknownReaction
        }
        _ => ParseErrorKind::InvalidSyntax,
    };
    parse_op_untyped(line).map_err(|detail| ParseError { kind, detail })
}

/// Compatibility parser. Prefer [`parse_op_typed`] when callers must retain a
/// machine-readable distinction between an unknown identity and bad grammar.
pub fn parse_op(line: &str) -> Result<Option<Operator>, String> {
    parse_op_typed(line).map_err(|error| error.detail)
}

fn parse_op_untyped(line: &str) -> Result<Option<Operator>, String> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return Ok(None);
    }
    let words: Vec<&str> = line.split_whitespace().collect();
    let op = match words[0] {
        // Session commands: they ask the SHELL something, not the bench,
        // so they are not operators and never enter the log. `particles`
        // used to be among them and is now an operator — it asks the
        // VESSEL what is in it, which is a bench question, and keeping it
        // here meant no script could pose it.
        "register" | "inspect" | "explain" | "species" | "help" | "structure" | "identify"
        | "coverage" => return Ok(None),
        // `react v1 esterification` — apply a named curated organic
        // transformation. The name is checked here so a typo fails at
        // parse time, with the shelf listed.
        "react" => {
            if words.len() < 3 {
                return Err("usage: react <vessel> <reaction> (see curated::ORG_REACTIONS)".into());
            }
            let vessel = parse_vessel(words[1])?;
            let name = words[2];
            if !crate::curated::ORG_REACTIONS.iter().any(|r| r.name == name)
                && !crate::selectivity::is_selectivity_verb(name)
            {
                let mut known: Vec<&str> = crate::curated::ORG_REACTIONS
                    .iter()
                    .map(|r| r.name)
                    .collect();
                known.push(crate::selectivity::VERB_NAME);
                return Err(format!(
                    "unknown reaction '{name}' — curated: {}",
                    known.join(", ")
                ));
            }
            Operator::React {
                vessel,
                reaction: name.to_string(),
            }
        }
        "new" => match words.get(1) {
            None => Operator::NewVessel { kind: None },
            Some(kind) => {
                if !crate::vessel::VESSEL_KINDS.iter().any(|(k, _)| k == kind) {
                    let known: Vec<&str> = crate::vessel::VESSEL_KINDS
                        .iter()
                        .map(|(k, _)| *k)
                        .collect();
                    return Err(format!(
                        "unknown vessel kind '{kind}' — known: {}",
                        known.join(", ")
                    ));
                }
                Operator::NewVessel {
                    kind: Some((*kind).to_string()),
                }
            }
        },
        "remove" => {
            if words.len() != 2 {
                return Err("usage: remove <vessel>".into());
            }
            Operator::RemoveVessel {
                vessel: parse_vessel(words[1])?,
            }
        }
        "add" => {
            if words.len() < 4 {
                return Err("usage: add <vessel> <species> <amount><mol|g|mL> [@ <T>C]".into());
            }
            let vessel = parse_vessel(words[1])?;
            // EXP-49: El-A notation with a curated nuclide entry routes
            // to the tracer ledger, not the chemical registry.
            if crate::nuclide::lookup_notation(words[2]).is_some() {
                let amount = words[3];
                let moles = amount
                    .strip_suffix("mol")
                    .and_then(|v| v.parse::<f64>().ok())
                    .ok_or_else(|| {
                        format!(
                            "nuclide amounts are stated in moles (got '{amount}') — \
                             tracer scale, e.g. 1e-9mol"
                        )
                    })?;
                return Ok(Some(Operator::SpikeNuclide {
                    vessel,
                    nuclide: words[2].to_string(),
                    moles: Moles(moles),
                }));
            }
            if let Some(data) = species::lookup_key(words[2]) {
                Operator::Add {
                    vessel,
                    species: SpeciesId::new(words[2]),
                    moles: parse_amount(words[3], data)?,
                    at: parse_at(&words[4..])?,
                }
            } else if let Some(recipe) = material::lookup(words[2], None) {
                let total_amount = parse_material_amount(words[3], &recipe)?;
                Operator::AddMaterial {
                    vessel,
                    material: recipe.canonical_key.clone(),
                    recipe_id: recipe.id,
                    recipe_version: recipe.version,
                    total_amount,
                    basis: recipe.basis,
                    sample_seed: 0,
                    at: parse_at(&words[4..])?,
                }
            } else {
                return Err(unknown_ingredient(words[2]));
            }
        }
        // BRD-002: `stock NaCl 0.5mol` / `stock vinegar 250mL` — fill one
        // bottle to a finite level. The amount goes through exactly the
        // same reader `add` uses, so a bottle and the dispenses against it
        // are counted in one unit and no conversion is invented here.
        "stock" => {
            if words.len() < 3 {
                return Err("usage: stock <species|material> <amount><mol|g|mL>".into());
            }
            let amount = if let Some(data) = species::lookup_key(words[1]) {
                parse_amount(words[2], data)?.0
            } else if let Some(recipe) = material::lookup(words[1], None) {
                parse_material_amount(words[2], &recipe)?
            } else {
                return Err(unknown_ingredient(words[1]));
            };
            let key = species::lookup_key(words[1])
                .map(|data| data.key.to_string())
                .or_else(|| material::lookup(words[1], None).map(|recipe| recipe.canonical_key))
                .expect("one of the two lookups above resolved");
            Operator::StockShelf { key, amount }
        }
        "heat" | "cool" => {
            if words.len() < 3 {
                return Err(format!("usage: {} <vessel> <energy><J|kJ>", words[0]));
            }
            let vessel = parse_vessel(words[1])?;
            let energy = parse_energy(words[2])?;
            if words[0] == "heat" {
                // `heat v1 40kJ` names no apparatus and keeps its old
                // meaning exactly: the bench default, a laboratory burner.
                // `heat v1 40kJ on candle` (or `mit Kerze`) names one, and
                // what a named source changes is how hot it can get.
                let named = words.iter().skip(3).find(|word| {
                    !matches!(
                        word.to_ascii_lowercase().as_str(),
                        "on" | "with" | "auf" | "mit" | "über" | "ueber"
                    )
                });
                let source = match named {
                    Some(word) => {
                        Some(crate::apparatus::HeatSource::by_name(word).ok_or_else(|| {
                            format!(
                                "unknown heat source \"{word}\": try burner, candle or hotplate"
                            )
                        })?)
                    }
                    None => None,
                };
                Operator::Heat {
                    vessel,
                    energy,
                    source,
                }
            } else {
                Operator::Cool { vessel, energy }
            }
        }
        "wait" => {
            // `wait 30s` — the clock the rate experiments need.
            let raw = words.get(1).ok_or("usage: wait <n><s|min|h>")?;
            Operator::Wait {
                seconds: parse_duration_seconds(raw)?,
            }
        }
        "ignite" => Operator::Ignite {
            vessel: parse_vessel(words.get(1).ok_or("usage: ignite <vessel>")?)?,
        },
        "stir" => {
            if words.len() > 4 {
                return Err("usage: stir <vessel> [<rpm>rpm] [<duration><s|min>]".into());
            }
            let vessel = parse_vessel(
                words
                    .get(1)
                    .ok_or("usage: stir <vessel> [<rpm>rpm] [<duration><s|min>]")?,
            )?;
            let rpm = words.get(2).map_or(Ok(500.0), |raw| {
                raw.strip_suffix("rpm")
                    .unwrap_or(raw)
                    .parse::<f64>()
                    .map_err(|_| format!("bad stir speed '{raw}'"))
            })?;
            let seconds = words
                .get(3)
                .map_or(Ok(10.0), |raw| parse_duration_seconds(raw))?;
            Operator::Stir {
                vessel,
                rpm,
                seconds,
            }
        }
        "seal" => {
            if words.len() != 3 {
                return Err("usage: seal <vessel> <headspace-volume><mL|L>".into());
            }
            Operator::Seal {
                vessel: parse_vessel(words[1])?,
                headspace_volume: parse_volume(words[2])?,
            }
        }
        "regulate" => {
            if words.len() != 4 {
                return Err(
                    "usage: regulate <vessel> <pressure><Pa|kPa|bar|atm> <initial-volume><mL|L>"
                        .into(),
                );
            }
            Operator::Regulate {
                vessel: parse_vessel(words[1])?,
                pressure: parse_pressure(words[2])?,
                initial_volume: parse_volume(words[3])?,
            }
        }
        "sweep" => {
            if words.len() != 3 {
                return Err("usage: sweep <vessel> <pressure><Pa|kPa|bar|atm>".into());
            }
            Operator::Sweep {
                vessel: parse_vessel(words[1])?,
                pressure: parse_pressure(words[2])?,
            }
        }
        "open" => Operator::Open {
            vessel: parse_vessel(words.get(1).ok_or("usage: open <vessel>")?)?,
        },
        "filter" => {
            if words.len() < 3 {
                return Err("usage: filter <from> <to>".into());
            }
            Operator::Filter {
                from: parse_vessel(words[1])?,
                to: parse_vessel(words[2])?,
            }
        }
        "magnet" => {
            if words.len() < 3 {
                return Err("usage: magnet <from> <to>".into());
            }
            Operator::Magnet {
                from: parse_vessel(words[1])?,
                to: parse_vessel(words[2])?,
            }
        }
        "evaporate" => {
            if words.len() < 3 {
                return Err("usage: evaporate <vessel> <fraction>".into());
            }
            Operator::Evaporate {
                vessel: parse_vessel(words[1])?,
                fraction: words[2]
                    .parse()
                    .map_err(|_| format!("bad fraction '{}'", words[2]))?,
            }
        }
        "decant" => {
            if words.len() < 4 {
                return Err("usage: decant <from> <to> <fraction>".into());
            }
            Operator::Decant {
                from: parse_vessel(words[1])?,
                to: parse_vessel(words[2])?,
                fraction: words[3]
                    .parse()
                    .map_err(|_| format!("bad fraction '{}'", words[3]))?,
            }
        }
        "drain" => {
            if words.len() < 3 {
                return Err("usage: drain <from> <to>".into());
            }
            Operator::Drain {
                from: parse_vessel(words[1])?,
                to: parse_vessel(words[2])?,
            }
        }
        "distil" | "distill" => {
            if words.len() < 4 {
                return Err(
                    "usage: distil <from> <to> <fraction | energy J|kJ> [stages <n>]".into(),
                );
            }
            let (fraction, energy) = if let Some(kj) = words[3].strip_suffix("kJ") {
                let v: f64 = kj
                    .parse()
                    .map_err(|_| format!("bad energy '{}'", words[3]))?;
                (None, Some(Joules(v * 1000.0)))
            } else if let Some(j) = words[3].strip_suffix('J') {
                let v: f64 = j
                    .parse()
                    .map_err(|_| format!("bad energy '{}'", words[3]))?;
                (None, Some(Joules(v)))
            } else {
                let f: f64 = words[3]
                    .parse()
                    .map_err(|_| format!("bad fraction '{}'", words[3]))?;
                (Some(f), None)
            };
            let stages = match (words.get(4), words.get(5)) {
                (Some(&"stages"), Some(n)) => {
                    n.parse().map_err(|_| format!("bad stage count '{n}'"))?
                }
                (None, _) => 1,
                _ => return Err("after the amount, only `stages <n>` may follow".into()),
            };
            Operator::Distil {
                from: parse_vessel(words[1])?,
                to: parse_vessel(words[2])?,
                fraction,
                energy,
                stages,
            }
        }
        // `look v1` — the youngest interaction there is.
        "look" | "observe" => Operator::Measure {
            vessel: parse_vessel(words.get(1).copied().unwrap_or("v1"))?,
            instrument: Instrument::Eyes,
        },
        "measure" => {
            if words.len() < 3 {
                return Err("usage: measure <vessel> <thermometer|balance|ph>".into());
            }
            Operator::Measure {
                vessel: parse_vessel(words[1])?,
                instrument: match instrument_by_word(words[2]) {
                    Some(instrument) => instrument,
                    None => return Err(format!("unknown instrument '{}'", words[2])),
                },
            }
        }
        // `smell v1` — waft, never huff. The taught technique is the verb.
        "smell" | "waft" => Operator::Smell {
            vessel: parse_vessel(words.get(1).copied().unwrap_or("v1"))?,
        },
        // `test v1 pop` — apply a classical gas test to the headspace.
        // BRD-001: `particles` is an operator so a SCRIPT can ask what a
        // learner can ask in the REPL. `zoom` is the long-standing alias.
        "particles" | "zoom" => {
            let vessel = parse_vessel(words.get(1).copied().unwrap_or("v1"))?;
            Operator::Particles { vessel }
        }
        "test" => {
            let vessel = parse_vessel(words.get(1).copied().unwrap_or("v1"))?;
            let test_name = words
                .get(2)
                .copied()
                .ok_or("usage: test <vessel> pop|splint|limewater|litmus")?;
            let test = match test_name {
                "pop" => crate::gas_tests::GasTest::Pop,
                "splint" => crate::gas_tests::GasTest::GlowingSplint,
                "limewater" => crate::gas_tests::GasTest::Limewater,
                "litmus" => crate::gas_tests::GasTest::DampLitmus,
                _ => {
                    return Err(format!(
                        "unknown gas test '{test_name}' — options: pop, splint, limewater, litmus"
                    ));
                }
            };
            Operator::TestGas { vessel, test }
        }
        // `chromatograph v1` — inject the solution onto the column and
        // read the peak table. Sugar for `measure v1 chromatograph`,
        // first-class because running a separation is a verb in any lab.
        "chromatograph" => Operator::Measure {
            vessel: parse_vessel(words.get(1).copied().unwrap_or("v1"))?,
            instrument: Instrument::Chromatograph,
        },
        // `cell v1 v2` — touch the wires of two half-cells together and
        // read the voltmeter. Nothing flows; the reading is the prediction.
        "electrolyse" | "electrolyze" => {
            // `electrolyse v1 0.5A 600s` — a current and a clock, which is
            // exactly what the practical gives you.
            if words.len() < 4 {
                return Err("usage: electrolyse <vessel> <current>A <time><s|min|h>".into());
            }
            let vessel = parse_vessel(words[1])?;
            let amps = parse_suffixed(words[2], &[("a", 1.0), ("ma", 1e-3), ("", 1.0)], "current")?;
            let seconds = parse_suffixed(
                words[3],
                &[
                    ("s", 1.0),
                    ("sec", 1.0),
                    ("secs", 1.0),
                    ("seconds", 1.0),
                    ("min", 60.0),
                    ("mins", 60.0),
                    ("minutes", 60.0),
                    ("h", 3600.0),
                    ("hr", 3600.0),
                    ("hours", 3600.0),
                    ("", 1.0),
                ],
                "time",
            )?;
            Operator::Electrolyse {
                vessel,
                amps,
                seconds,
            }
        }
        "cell" | "voltmeter" => {
            if words.len() < 3 {
                return Err("usage: cell <vessel> <vessel>".into());
            }
            Operator::Cell {
                a: parse_vessel(words[1])?,
                b: parse_vessel(words[2])?,
            }
        }
        "grind" => {
            // `grind v1 NaCl 50um` — set particle size for heterogeneous rates
            if words.len() < 4 {
                return Err("usage: grind <vessel> <species> <diameter>um".into());
            }
            let vessel = parse_vessel(words[1])?;
            let species_key = words[2];
            let _ = species::lookup_key(species_key)
                .ok_or_else(|| format!("unknown species '{species_key}'"))?;
            let diameter = parse_suffixed(
                words[3],
                &[("um", 1.0), ("μm", 1.0), ("mm", 1000.0), ("", 1.0)],
                "diameter",
            )?;
            Operator::Grind {
                vessel,
                species: SpeciesId::new(species_key),
                diameter_um: diameter,
            }
        }
        "centrifuge" => {
            if words.len() < 5 {
                return Err("usage: centrifuge <vessel> <rpm>rpm <time>s <radius>cm".into());
            }
            Operator::Centrifuge {
                vessel: parse_vessel(words[1])?,
                rpm: parse_suffixed(words[2], &[("rpm", 1.0), ("", 1.0)], "rotation speed")?,
                seconds: parse_duration_seconds(words[3])?,
                rotor_radius_m: parse_suffixed(
                    words[4],
                    &[("mm", 0.001), ("cm", 0.01), ("m", 1.0), ("", 0.01)],
                    "rotor radius",
                )?,
                counterbalance_g: words
                    .get(5)
                    .map(|value| parse_suffixed(value, &[("g", 1.0), ("", 1.0)], "counterbalance"))
                    .transpose()?,
            }
        }
        "irradiate" => {
            // `irradiate v1 254nm 10W/m2` — turn on UV lamp
            if words.len() < 4 {
                return Err("usage: irradiate <vessel> <wavelength>nm <irradiance>W/m2".into());
            }
            let vessel = parse_vessel(words[1])?;
            let wavelength = parse_suffixed(words[2], &[("nm", 1.0), ("", 1.0)], "wavelength")?;
            let irradiance = parse_suffixed(words[3], &[("w/m2", 1.0), ("", 1.0)], "irradiance")?;
            Operator::Irradiate {
                vessel,
                wavelength_nm: wavelength,
                irradiance_w_m2: irradiance,
            }
        }
        "dilute" => {
            if words.len() < 3 {
                return Err("usage: dilute <vessel> <volume><mL|L>".into());
            }
            Operator::Dilute {
                vessel: parse_vessel(words[1])?,
                volume: parse_volume(words[2])?,
            }
        }
        "titrate" => {
            // titrate v1 NaOH 1mL until ph 7          (1 mol/L standard)
            // titrate v1 NaOH 0.1M 1mL until ph 7 max 200
            // titrate v1 KMnO4 0.02M 0.1mL until pe > 8       (EXP-39)
            // titrate v1 KMnO4 0.02M 0.1mL until colour persists
            //
            // The burette holds a *standard solution*, not the pure
            // substance: `<c>M` states its concentration, defaulting to
            // 1 mol/L — the convention every titration practical prints
            // on the bottle. (Delivering pure titrant by volume would
            // dose ~50× per mL for NaOH and leap the whole curve in one
            // step, which is what this grammar replaced.)
            if words.len() < 7 {
                return Err(TITRATE_USAGE.into());
            }
            let vessel = parse_vessel(words[1])?;
            let titrant_key = words[2];
            let _ = species::lookup_key(titrant_key)
                .ok_or_else(|| format!("unknown species '{titrant_key}' (see 'species')"))?;
            let (concentration, rest) = match words[3].strip_suffix(['M', 'm']) {
                Some(c) if c.parse::<f64>().is_ok() => (c.parse::<f64>().unwrap(), &words[4..]),
                _ => (1.0, &words[3..]),
            };
            if concentration <= 0.0 {
                return Err("titrant concentration must be positive".into());
            }
            if rest.len() < 4 {
                return Err(TITRATE_USAGE.into());
            }
            let step = parse_volume(rest[0])?;
            finite(step.0, "burette increment")?;
            finite(concentration, "titrant concentration")?;
            if rest[1] != "until" {
                return Err(TITRATE_USAGE.into());
            }
            // EXP-39: three endpoints. `ph` is CAP-12's and keeps its
            // exact spelling and its exact meaning — a crossing, in
            // whichever direction the curve arrives from. The two redox
            // endpoints are inequalities, because past equivalence a
            // potential keeps climbing and a colour keeps standing.
            let (endpoint, target_ph, tail) = match rest[2] {
                "ph" => {
                    let target: f64 = rest[3]
                        .parse()
                        .map_err(|_| format!("bad pH target '{}'", rest[3]))?;
                    // `"1e999".parse::<f64>()` is `Ok(inf)`, and serde_json
                    // cannot write an infinity — so an endpoint like that
                    // parses, runs, and then produces an operator log the
                    // bench cannot save. The grammar fuzz target found it.
                    log_scale(target, "pH target")?;
                    (Endpoint::Ph, target, &rest[4..])
                }
                "pe" => {
                    if rest.len() < 5 {
                        return Err("usage: titrate <vessel> <titrant> [<c>M] <step> until \
                                    pe <op> <value> [max <n>], where <op> is > >= < <="
                            .into());
                    }
                    let compare = match rest[3] {
                        ">" | "above" => Compare::Above,
                        ">=" => Compare::AtLeast,
                        "<" | "below" => Compare::Below,
                        "<=" => Compare::AtMost,
                        other => {
                            return Err(format!(
                                "'{other}' is not a comparison — write `until pe > 8`, \
                                 `>=`, `<` or `<=` (or the words `above`/`below`)"
                            ))
                        }
                    };
                    let value: f64 = rest[4]
                        .parse()
                        .map_err(|_| format!("bad pe target '{}'", rest[4]))?;
                    log_scale(value, "pe target")?;
                    (Endpoint::Pe { compare, value }, NEUTRAL_PH, &rest[5..])
                }
                "colour" | "color" => {
                    if rest.get(3) != Some(&"persists") {
                        return Err("usage: titrate <vessel> <titrant> [<c>M] <step> until \
                                    colour persists [max <n>]"
                            .into());
                    }
                    (Endpoint::ColourPersists, NEUTRAL_PH, &rest[4..])
                }
                other => {
                    return Err(format!(
                        "'{other}' is not an endpoint — this bench titrates until \
                         `ph <target>`, `pe <op> <value>`, or `colour persists`"
                    ))
                }
            };
            let max_steps = match (tail.first(), tail.get(1)) {
                (Some(&"max"), Some(n)) => {
                    n.parse().map_err(|_| format!("bad max step count '{n}'"))?
                }
                (None, _) => 100,
                _ => return Err("after the endpoint, only `max <n>` may follow".into()),
            };
            Operator::Titrate {
                vessel,
                titrant: SpeciesId::new(titrant_key),
                concentration,
                step,
                target_ph,
                max_steps,
                endpoint,
            }
        }
        "mix" => {
            // mix v1 0.5 v2 0.5 into v3
            if words.len() < 7 {
                return Err(
                    "usage: mix <vessel-a> <frac-a> <vessel-b> <frac-b> into <target>".into(),
                );
            }
            let a = parse_vessel(words[1])?;
            let fraction_a: f64 = words[2]
                .parse()
                .map_err(|_| format!("bad fraction '{}'", words[2]))?;
            let b = parse_vessel(words[3])?;
            let fraction_b: f64 = words[4]
                .parse()
                .map_err(|_| format!("bad fraction '{}'", words[4]))?;
            if words[5] != "into" {
                return Err(
                    "usage: mix <vessel-a> <frac-a> <vessel-b> <frac-b> into <target>".into(),
                );
            }
            let into = parse_vessel(words[6])?;
            Operator::Mix {
                a,
                b,
                into,
                fraction_a,
                fraction_b,
            }
        }
        "transport" => {
            // transport v1 v2 v3 from v4 to v5 steps 5 [courant 0.5]
            let from_pos = words.iter().position(|&w| w == "from");
            let to_pos = words.iter().position(|&w| w == "to");
            let steps_pos = words.iter().position(|&w| w == "steps");
            let (from_pos, to_pos, steps_pos) = match (from_pos, to_pos, steps_pos) {
                (Some(f), Some(t), Some(s)) => (f, t, s),
                _ => {
                    return Err(
                        "usage: transport <v1> [v2 ...] from <inlet> to <receiver> steps <n> [courant <f>]"
                            .into(),
                    )
                }
            };
            if from_pos < 2 {
                return Err("transport needs at least one cell vessel before 'from'".into());
            }
            let chain: Vec<VesselId> = words[1..from_pos]
                .iter()
                .map(|w| parse_vessel(w))
                .collect::<Result<_, _>>()?;
            let inlet = parse_vessel(
                words
                    .get(from_pos + 1)
                    .ok_or("expected inlet vessel after 'from'")?,
            )?;
            let receiver = parse_vessel(
                words
                    .get(to_pos + 1)
                    .ok_or("expected receiver vessel after 'to'")?,
            )?;
            let steps: u32 = words
                .get(steps_pos + 1)
                .ok_or("expected step count after 'steps'")?
                .parse()
                .map_err(|_| {
                    format!(
                        "bad step count '{}'",
                        words.get(steps_pos + 1).unwrap_or(&"")
                    )
                })?;
            let courant_pos = words.iter().position(|&w| w == "courant");
            let courant: f64 = match courant_pos {
                Some(cp) => words
                    .get(cp + 1)
                    .ok_or("expected Courant fraction after 'courant'")?
                    .parse()
                    .map_err(|_| {
                        format!(
                            "bad Courant fraction '{}'",
                            words.get(cp + 1).unwrap_or(&"")
                        )
                    })?,
                None => 1.0,
            };
            Operator::Transport {
                chain,
                inlet,
                receiver,
                steps,
                courant,
            }
        }
        other => return Err(format!("unknown command '{other}' (try 'help')")),
    };
    Ok(Some(op))
}

pub fn parse_vessel(word: &str) -> Result<VesselId, String> {
    let digits = word.trim_start_matches('v');
    let n: usize = digits
        .parse()
        .map_err(|_| format!("bad vessel '{word}' (use v1, v2, …)"))?;
    if n == 0 {
        return Err("vessels are numbered from v1".into());
    }
    Ok(VesselId(n - 1))
}

/// `0.5mol`, `10g`, `100mL` (unit required, so units are never guessed).
pub fn parse_amount(word: &str, data: &SpeciesData) -> Result<Moles, String> {
    let (value, unit) = split_unit(word)?;
    match unit {
        "mol" => Ok(Moles(value)),
        // Household amounts. A child does not weigh things in grams, and
        // demanding they do is the fastest way to lose them. These are
        // ordinary kitchen measures, stated as such.
        "spoon" | "spoons" | "tsp" => Ok(data.moles_from_grams(Grams(value * 5.0))),
        "pinch" | "pinches" => Ok(data.moles_from_grams(Grams(value * 0.3))),
        "cup" | "cups" => Ok(data.moles_from_liters(Liters(value * 0.25))),
        "splash" | "splashes" => Ok(data.moles_from_liters(Liters(value * 0.02))),
        "drop" | "drops" => Ok(data.moles_from_liters(Liters(value * 0.00005))),
        "g" => Ok(data.moles_from_grams(Grams(value))),
        "mL" | "ml" => Ok(data.moles_from_liters(Liters(value / 1000.0))),
        "L" | "l" => Ok(data.moles_from_liters(Liters(value))),
        other => Err(format!(
            "unknown amount '{other}' — try g, mL, L, mol, or a kitchen measure: spoon, pinch, cup, splash, drop"
        )),
    }
}

/// Convert a user amount into a recipe's declared basis. Mass-fraction
/// materials may accept volume only when the reviewed recipe supplies a bulk
/// density; we never invent one at parse time.
pub fn parse_material_amount(word: &str, recipe: &MaterialRecipe) -> Result<f64, String> {
    let (value, unit) = split_unit(word)?;
    if !value.is_finite() || value <= 0.0 {
        return Err("material amount must be positive".into());
    }
    match recipe.basis {
        MaterialBasis::MassFraction => match unit {
            "g" => Ok(value),
            "mL" | "ml" => recipe
                .bulk_density
                .as_ref()
                .map(|density| value * density.value)
                .ok_or_else(|| {
                    format!(
                        "material '{}' has no reviewed bulk density; add it by mass (g)",
                        recipe.canonical_key
                    )
                }),
            "L" | "l" => recipe
                .bulk_density
                .as_ref()
                .map(|density| value * 1000.0 * density.value)
                .ok_or_else(|| {
                    format!(
                        "material '{}' has no reviewed bulk density; add it by mass (g)",
                        recipe.canonical_key
                    )
                }),
            other => Err(format!(
                "mass-fraction material '{}' accepts g, mL, or L (got '{other}')",
                recipe.canonical_key
            )),
        },
        MaterialBasis::MoleFraction => match unit {
            "mol" => Ok(value),
            other => Err(format!(
                "mole-fraction material '{}' accepts mol (got '{other}')",
                recipe.canonical_key
            )),
        },
        MaterialBasis::VolumeFraction => match unit {
            "mL" | "ml" => Ok(value),
            "L" | "l" => Ok(value * 1000.0),
            other => Err(format!(
                "volume-fraction material '{}' accepts mL or L (got '{other}')",
                recipe.canonical_key
            )),
        },
    }
}

pub fn parse_energy(word: &str) -> Result<Joules, String> {
    let (value, unit) = split_unit(word)?;
    match unit {
        "J" | "j" => Ok(Joules(value)),
        "kJ" | "kj" => Ok(Joules(value * 1000.0)),
        other => Err(format!("unknown energy unit '{other}' (J, kJ)")),
    }
}

pub fn parse_volume(word: &str) -> Result<Liters, String> {
    let (value, unit) = split_unit(word)?;
    if value <= 0.0 {
        return Err("headspace volume must be positive".into());
    }
    match unit {
        "mL" | "ml" => Ok(Liters(value / 1000.0)),
        "L" | "l" => Ok(Liters(value)),
        other => Err(format!("unknown volume unit '{other}' (mL, L)")),
    }
}

pub fn parse_pressure(word: &str) -> Result<Pascal, String> {
    let (value, unit) = split_unit(word)?;
    if value <= 0.0 {
        return Err("pressure must be positive".into());
    }
    match unit {
        "Pa" | "pa" => Ok(Pascal(value)),
        "kPa" | "kpa" => Ok(Pascal(value * 1_000.0)),
        "bar" => Ok(Pascal(value * 100_000.0)),
        "atm" => Ok(Pascal(value * Pascal::ATMOSPHERIC.0)),
        other => Err(format!(
            "unknown pressure unit '{other}' (Pa, kPa, bar, atm)"
        )),
    }
}

/// Optional trailing `@ 60C` / `@ 333K` on `add`.
pub fn parse_at(words: &[&str]) -> Result<Option<Kelvin>, String> {
    match words {
        [] => Ok(None),
        ["@", t] => {
            let (value, unit) = split_unit(t)?;
            match unit {
                "C" | "c" => Ok(Some(Kelvin::from_celsius(value))),
                "K" | "k" => Ok(Some(Kelvin(value))),
                other => Err(format!("unknown temperature unit '{other}' (C, K)")),
            }
        }
        _ => Err("temperature goes last: … @ 60C".into()),
    }
}

fn parse_duration_seconds(raw: &str) -> Result<f64, String> {
    parse_suffixed(
        raw,
        &[
            ("", 1.0),
            ("s", 1.0),
            ("sec", 1.0),
            ("secs", 1.0),
            ("seconds", 1.0),
            ("min", 60.0),
            ("mins", 60.0),
            ("minutes", 60.0),
            ("h", 3600.0),
            ("hr", 3600.0),
            ("hours", 3600.0),
        ],
        "duration",
    )
}

fn split_unit(word: &str) -> Result<(f64, &str), String> {
    let split = word
        .find(|c: char| c.is_ascii_alphabetic())
        .ok_or_else(|| format!("'{word}' needs a unit suffix"))?;
    let value: f64 = word[..split]
        .parse()
        .map_err(|_| format!("bad number in '{word}'"))?;
    Ok((value, &word[split..]))
}

/// A number with a unit suffix, matched longest-first so `ms` cannot be
/// read as `m`. Shared by the operators that take a physical quantity.
fn parse_suffixed(raw: &str, units: &[(&str, f64)], what: &str) -> Result<f64, String> {
    let digits: String = raw
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    let value: f64 = digits.parse().map_err(|_| format!("bad {what} '{raw}'"))?;
    let suffix = raw[digits.len()..].trim().to_ascii_lowercase();
    let mut best: Option<f64> = None;
    for (name, scale) in units {
        if suffix == *name {
            best = Some(*scale);
            break;
        }
    }
    match best {
        Some(scale) if value > 0.0 => Ok(value * scale),
        Some(_) => Err(format!("{what} must be positive")),
        None => Err(format!("unknown {what} unit '{suffix}'")),
    }
}

#[cfg(test)]
mod localised_grammar {
    use super::*;

    fn de() -> Locale {
        Locale::parse("de")
    }

    /// A canonical line for every verb an alias table may name — the
    /// inventory's own example where there is one, and a written line for
    /// the synonyms the inventory deliberately does not list twice.
    fn example_for(verb: &str) -> String {
        if let Some((_, example)) = VERBS.iter().find(|(name, _)| *name == verb) {
            return (*example).to_string();
        }
        match verb {
            "look" | "observe" | "waft" | "zoom" => format!("{verb} v1"),
            "mix" => "mix v1 0.5 v2 0.5 into v3".to_string(),
            "distill" => "distill v1 v2 0.5".to_string(),
            "electrolyze" => "electrolyze v1 0.5A 30min".to_string(),
            "voltmeter" => "voltmeter v1 v2".to_string(),
            other => panic!("no example line for '{other}'"),
        }
    }

    /// The property the whole layer rests on: an English line means the
    /// same thing in every language, byte for byte.
    ///
    /// Every shipped lesson, the corpus, the operator log and the replay
    /// cache are English lines. If a translation could rewrite one of
    /// them, a German learner would replay a different script from an
    /// English one — so the rewrite refuses to touch a word this grammar
    /// already spends, and this walks the inventory to prove it.
    #[test]
    fn a_canonical_line_is_never_rewritten() {
        for locale in Locale::available() {
            for (_, example) in VERBS {
                assert_eq!(
                    canonical_line_in(example, locale),
                    None,
                    "{} rewrote the canonical line '{example}'",
                    locale.code()
                );
            }
        }
    }

    /// Every alias parses to the operator its canonical verb does, and
    /// reports the canonical line back.
    #[test]
    fn every_german_verb_alias_means_its_canonical_verb() {
        let de = de();
        let rows = de.section("script-verb");
        assert!(!rows.is_empty(), "the German catalogue lists no verbs");
        for (verb, list) in rows {
            let example = example_for(verb);
            let expected = parse_op(&example).expect("the example parses");
            let mut used = 0;
            for alias in split_aliases(list) {
                let mut words: Vec<&str> = example.split_whitespace().collect();
                words[0] = alias;
                let line = words.join(" ");
                let command = parse_command(&line, de)
                    .unwrap_or_else(|e| panic!("'{line}' did not parse: {e}"));
                assert_eq!(command.operator, expected, "'{line}' is not '{example}'");
                assert_eq!(command.canonical, example, "'{line}' logged the wrong line");
                used += 1;
            }
            assert!(used > 0, "[script-verb] {verb} lists no usable alias");
        }
    }

    /// An instrument, a gas test, a flame and a glassware kind, each named
    /// in German after a German verb.
    #[test]
    fn german_reaches_past_the_verb() {
        let de = de();
        for (typed, canonical) in [
            ("messen v1 waage", "measure v1 balance"),
            ("messen v1 ph-wert", "measure v1 ph"),
            ("prüfen v1 kalkwasser", "test v1 limewater"),
            ("erhitzen v1 10kJ auf kerze", "heat v1 10kJ on candle"),
            ("neu reagenzglas", "new tube"),
        ] {
            let command =
                parse_command(typed, de).unwrap_or_else(|e| panic!("'{typed}' did not parse: {e}"));
            assert_eq!(command.canonical, canonical, "'{typed}'");
            assert_eq!(
                command.operator,
                parse_op(canonical).expect("the canonical line parses"),
                "'{typed}' is not '{canonical}'"
            );
        }
    }

    /// A species by its German name, and a material by its German alias.
    /// Both land on the canonical key, so the log is the same script an
    /// English learner would have written.
    #[test]
    fn german_names_resolve_to_the_canonical_key() {
        let de = de();
        let water = parse_command("zugeben v1 Wasser 100mL", de).expect("German water");
        assert_eq!(water.canonical, "add v1 water 100mL");
        assert_eq!(water.operator, parse_op("add v1 water 100mL").unwrap());

        let salt = parse_command("zugeben v1 Natriumchlorid 1g", de).expect("German salt");
        assert_eq!(salt.canonical, "add v1 NaCl 1g");

        let milk = parse_command("zugeben v1 Milch 100mL", de).expect("German milk");
        assert_eq!(milk.canonical, "add v1 whole_milk 100mL");
        assert_eq!(milk.operator, parse_op("add v1 whole_milk 100mL").unwrap());
    }

    /// The ambiguity policy, both halves.
    ///
    /// `magnet` and `voltmeter` are spelt the same in German, so the
    /// German spelling never registers — and never has to. A word claimed
    /// by two canonical tokens is dropped rather than guessed at.
    #[test]
    fn english_wins_and_a_word_claimed_twice_is_dropped() {
        let de = de();
        assert_eq!(canonical_line_in("magnet v1 v2", de), None);
        assert_eq!(
            canonical_line_in("magnetisieren v1 v2", de).as_deref(),
            Some("magnet v1 v2")
        );

        let mut index = HashMap::new();
        let mut dropped = HashSet::new();
        claim(&mut index, &mut dropped, "probe", "ph", |_| false);
        claim(&mut index, &mut dropped, "probe", "balance", |_| false);
        assert_eq!(index.get("probe"), None, "a word claimed twice must go");
        claim(&mut index, &mut dropped, "probe", "ph", |_| false);
        assert_eq!(index.get("probe"), None, "and must not come back");
        claim(&mut index, &mut dropped, "waage", "balance", |_| true);
        assert_eq!(index.get("waage"), None, "English wins");
    }

    /// A first word nobody knows is refused in the learner's language,
    /// naming the verbs rather than an English help screen.
    #[test]
    fn the_unknown_verb_refusal_speaks_german() {
        let error = parse_command("blubbern v1 wasser", de()).unwrap_err();
        assert!(
            error.detail.contains("unbekannter Befehl 'blubbern'"),
            "{}",
            error.detail
        );
        assert!(
            error.detail.contains("zugeben (add)"),
            "the German refusal must name the German verbs: {}",
            error.detail
        );
        // English is untouched.
        assert_eq!(
            parse_command("blubbern v1 wasser", Locale::EN)
                .unwrap_err()
                .detail,
            "unknown command 'blubbern' (try 'help')"
        );
    }

    /// An English line still parses for a German learner: the rewrite is
    /// tried first, and the raw line still gets its turn.
    #[test]
    fn english_still_works_in_a_german_session() {
        let de = de();
        for (_, example) in VERBS {
            let command = parse_command(example, de)
                .unwrap_or_else(|e| panic!("'{example}' did not parse in German: {e}"));
            assert_eq!(command.canonical, *example);
            assert_eq!(command.operator, parse_op(example).unwrap());
        }
    }

    /// Every hint a UI may offer is a line the parser takes, and it means
    /// exactly the example it was made from.
    ///
    /// A command bar that suggests `zugeben v1 Wasser 100mL` and then
    /// refuses it would be worse than one that suggested nothing, so the
    /// round trip is gated rather than trusted.
    #[test]
    fn every_hint_is_a_line_the_parser_takes() {
        let de = de();
        let mut localised = 0;
        for (_, example) in VERBS {
            let Some(hint) = example_in(example, de) else {
                continue;
            };
            localised += 1;
            assert_eq!(
                canonical_line_in(&hint, de).as_deref(),
                Some(*example),
                "the hint '{hint}' does not mean '{example}'"
            );
            let command =
                parse_command(&hint, de).unwrap_or_else(|e| panic!("'{hint}' was refused: {e}"));
            assert_eq!(command.canonical, *example);
        }
        assert!(
            localised > 20,
            "only {localised} of the inventory's examples have a German form"
        );
    }

    /// The tables the alias layer reads the English out of must be the
    /// English the parser actually accepts.
    #[test]
    fn the_english_tables_are_what_the_parser_takes() {
        for (word, instrument) in INSTRUMENT_WORDS {
            match parse_op(&format!("measure v1 {word}")) {
                Ok(Some(Operator::Measure {
                    instrument: got, ..
                })) => {
                    assert_eq!(got, *instrument, "measure v1 {word}");
                }
                other => panic!("measure v1 {word}: {other:?}"),
            }
        }
        for word in GAS_TEST_WORDS {
            parse_op(&format!("test v1 {word}")).unwrap_or_else(|e| panic!("test v1 {word}: {e}"));
        }
        for word in HEAT_SOURCE_WORDS {
            parse_op(&format!("heat v1 1kJ on {word}"))
                .unwrap_or_else(|e| panic!("heat v1 1kJ on {word}: {e}"));
        }
    }
}

#[cfg(test)]
mod grammar_inventory {
    use super::*;

    /// Every inventory row's example must parse to an operator, and its
    /// first word must be the row's verb — the inventory cannot claim a
    /// grammar the parser does not have.
    #[test]
    fn every_verb_example_parses() {
        for (verb, example) in VERBS {
            assert_eq!(
                example.split_whitespace().next(),
                Some(*verb),
                "inventory row '{verb}' must exemplify its own verb"
            );
            match parse_op(example) {
                Ok(Some(_)) => {}
                other => panic!("VERBS example '{example}' did not parse: {other:?}"),
            }
        }
    }

    /// Glassware kinds parse into the vessel label, and an unknown kind
    /// is refused with the list.
    #[test]
    fn vessel_kinds_parse_and_unknowns_refuse() {
        match parse_op("new tube") {
            Ok(Some(Operator::NewVessel { kind: Some(k) })) => assert_eq!(k, "tube"),
            other => panic!("new tube: {other:?}"),
        }
        let err = parse_op("new saucepan").unwrap_err();
        assert!(err.contains("beaker"), "refusal lists kinds: {err}");
    }

    /// The inventory is unique and non-trivial.
    #[test]
    fn the_inventory_is_well_formed() {
        let mut seen = std::collections::HashSet::new();
        for (verb, _) in VERBS {
            assert!(seen.insert(verb), "duplicate inventory verb '{verb}'");
        }
        assert!(
            VERBS.len() >= 25,
            "the inventory lost verbs: {}",
            VERBS.len()
        );
    }

    #[test]
    fn typed_errors_distinguish_identity_reaction_and_grammar_gaps() {
        assert_eq!(
            parse_op_typed("add v1 dragon-slime 1g").unwrap_err().kind,
            ParseErrorKind::UnknownSpecies
        );
        assert_eq!(
            parse_op_typed("react v1 transmutation").unwrap_err().kind,
            ParseErrorKind::UnknownReaction
        );
        assert_eq!(
            parse_op_typed("heat v1 eventually").unwrap_err().kind,
            ParseErrorKind::InvalidSyntax
        );
        assert!(matches!(
            parse_op_typed("add v1 water 10mL"),
            Ok(Some(Operator::Add { .. }))
        ));
    }
}
