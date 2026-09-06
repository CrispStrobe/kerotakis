//! The gate: the engine may not emit a word the German catalogue lacks.
//!
//! Every translation path in this codebase falls back **per string**, on
//! purpose — that is what lets a half-finished translation ship. It is
//! also why a missing term is invisible: an untranslated species renders
//! its English name inside an otherwise German sentence, and nothing
//! anywhere reports it. A coverage percentage does not help, because the
//! percentage is computed over the keys that exist, not over the words the
//! engine can produce.
//!
//! So this file does the only thing that works: it walks the engine's own
//! registries and asks, for each term they can put on a screen, whether
//! German has a word for it. Adding a species, an instrument, an event or
//! a colour without German fails here rather than shipping silently.
//!
//! **Nothing below is a list.** Every inventory is read from the type or
//! the table that produces the term. A hand-maintained list of things to
//! check is a list someone will forget to extend at exactly the moment it
//! mattered — which is how the 80 terms this gate found on its first run
//! reached a German reader in English, with every existing gate green
//! over all of them. Five of the eighty were merged to main WHILE this
//! file was being written, and it caught them on the rebase.
//!
//! # What is covered
//!
//! | Inventory | Source of truth | Catalogue |
//! |---|---|---|
//! | species names | `species::registry()` | `species.*` |
//! | glassware | `vessel::VESSEL_KINDS` | `glassware.*` |
//! | instruments | `Instrument` variants → `render::instrument_name` | `instrument.*` |
//! | phases | `Phase` variants → `render::phase_key` | `phase.*` |
//! | phase-change verbs | `Phase`×`Phase` → `render::phase_change_verb` | `verb.*` |
//! | hazard severities | `Severity` variants | `severity.*` |
//! | appearance words | `SpeciesData.appearance` | `appearance.*` |
//! | flame colours | `SpeciesData.flame_colour` | `flame.*` |
//! | events | `Event` serde variant names | `event.*` |
//! | relations | `relations::RELATIONS` | `*_de` fields |
//! | command verbs | `script::VERBS` + `VERB_SYNONYMS` | `script-verb.*` |
//!
//! The last row is the newest and the odd one out: it is not about a word
//! the engine SAYS but about a word a learner may TYPE. It was excluded on
//! purpose for as long as the parser accepted English only — a German verb
//! in the catalogue would have been a word in front of a parser that
//! refused it. The parser now reads these tables (`script::alias_index`),
//! so a verb without German is a verb a German learner cannot reach, which
//! is the same failure this file exists to catch.
//!
//! Two more inventories are gated elsewhere and deliberately not repeated
//! here: hazardous vapours (`senses::ODORS`) and the incompatibility
//! matrix's hazard sentences, in `tests/hazard_locale.rs` and in
//! `kerotakis-safety` respectively — the latter because the matrix is
//! private to that crate and walking it needs to be a unit test.
//!
//! # What is consciously NOT covered, and why
//!
//! - **Refusal reasons that interpolate a value.** `bench.rs` composes
//!   these as finished English sentences and puts the sentence in the
//!   event, so there is no key to enumerate — only a wire-format change
//!   would give one. The fixed reasons are exact catalogue entries and
//!   `render_locale.rs` pins them. See I18N.md, "The bench's refusals".
//! - **LV2 and LV3 evidence lines.** Twelve of these are still inside a
//!   bare `format!`. They are numeric evidence for a reader who has asked
//!   for the working, not the sentence a learner reads, and converting
//!   them is a separate job that `tools/engine-locale-lint.py` already
//!   counts and reports. This gate requires LV1 — the learner-facing
//!   sentence — of every event.
//! - **`appearance::observe`'s composed sentence.** `Appearance.words` is
//!   prose assembled in `appearance.rs`, not a term drawn from a table.
//!   It needs call-site keys before any catalogue can reach it.
//! - **The codex and the interface bundles.** Different surfaces with
//!   their own gates (`tools/codex-locale-lint.py`, `localeBundles.test.ts`).

use kerotakis_core::ops::{Event, Instrument};
use kerotakis_core::render::{instrument_name, phase_change_verb, phase_key};
use kerotakis_core::solve::Severity;
use kerotakis_core::species::{self, Phase};
use kerotakis_core::vessel::VESSEL_KINDS;
use kerotakis_core::Locale;

fn de() -> Locale {
    Locale::parse("de")
}

/// The German catalogue, flattened the way the engine flattens it.
///
/// `Locale::lookup` answers about one key; a few checks below need to ask
/// whether ANY key exists under a prefix, which no lookup can answer.
fn catalogue() -> toml::Table {
    include_str!("../i18n/de.toml")
        .parse::<toml::Table>()
        .expect("de.toml parses")
}

/// Every variant name of a serde enum, as serde itself spells them.
///
/// Deserialising a tag no variant claims makes serde list the ones that
/// exist: "unknown variant `…`, expected one of `a`, `b`". That error is
/// generated from the enum, so it stays correct when someone adds a
/// variant — which is the entire point. A hand-written list of events is
/// a list that goes stale on the commit that needed it most.
fn serde_variants(
    probe: serde_json::Value,
    deserialize: fn(serde_json::Value) -> String,
) -> Vec<String> {
    let message = deserialize(probe);
    // Everything serde put in backticks, minus the tag we invented. That
    // covers all three shapes the message takes — "expected `a`",
    // "expected `a` or `b`", "expected one of `a`, `b`, `c`" — without
    // this test needing to know which enum has how many variants.
    let names: Vec<String> = message
        .split('`')
        .skip(1)
        .step_by(2)
        .filter(|name| *name != PROBE)
        .map(str::to_string)
        .collect();
    assert!(
        !names.is_empty(),
        "could not read the variants out of serde's error — the message shape \
         must have changed, and this gate is now checking nothing: {message}"
    );
    names
}

/// A tag no variant will ever claim.
const PROBE: &str = "__kerotakis_i18n_probe__";

fn err_of<T: serde::de::DeserializeOwned>(value: serde_json::Value) -> String {
    match serde_json::from_value::<T>(value) {
        Ok(_) => panic!("the probe tag deserialised — pick one nothing claims"),
        Err(e) => e.to_string(),
    }
}

fn report(what: &str, prefix: &str, missing: Vec<String>) {
    assert!(
        missing.is_empty(),
        "{} {what} the engine can put on screen have no German.\n\
         Add each to `[{prefix}]` in crates/kerotakis-core/i18n/de.toml:\n  {}",
        missing.len(),
        missing.join("\n  ")
    );
}

// ── The value-keyed tables ──────────────────────────────────────────
//
// Each of these is looked up by its ENGLISH TEXT rather than by a key
// naming a place, because the English is what the engine has in hand at
// the point it writes the sentence.

/// Every species the shelf can name.
///
/// The single largest inventory, and the one that grows every time
/// somebody adds chemistry. On this gate's first run seventeen species —
/// bromoethane, sucrose, the iron hydroxides, the whole tert-butyl
/// bromide hydrolysis set — were rendering their English names inside
/// German journal lines.
///
/// The interface had German for all seventeen. `i18n.test.ts` walks the
/// same registry export and fails without it; the engine had no
/// equivalent, so one substance was `Saccharose` on the shelf and
/// `sucrose` in the sentence about it. Two dictionaries, one gated.
#[test]
fn every_species_the_registry_names_has_german() {
    let de = de();
    let mut missing: Vec<String> = species::registry()
        .iter()
        .map(|s| s.name)
        .filter(|name| de.lookup(&format!("species.{name}")).is_none())
        .map(str::to_string)
        .collect();
    missing.sort_unstable();
    missing.dedup();
    report("species", "species", missing);
}

/// Every verb a learner may type.
///
/// The canonical script stays English — a lesson and a saved session are
/// English lines wherever they are replayed — and what a language gets is
/// an alias read at parse time and rewritten away before anything is
/// stored. A verb with no row here is one a German learner has to know
/// the English for, and the point of the layer is that they do not.
///
/// `VERB_SYNONYMS` is included because a synonym is a whole verb to the
/// person typing it: `look` is how a young learner meets the bench.
/// `VERB_SPELLINGS` is not: `distill` is `distil` with another letter in
/// it, and no language but English has both.
#[test]
fn every_verb_the_grammar_accepts_has_german() {
    let de = de();
    let mut missing: Vec<String> = kerotakis_core::script::VERBS
        .iter()
        .map(|(verb, _)| *verb)
        .chain(kerotakis_core::script::VERB_SYNONYMS.iter().copied())
        .filter(|verb| de.lookup(&format!("script-verb.{verb}")).is_none())
        .map(str::to_string)
        .collect();
    missing.sort_unstable();
    missing.dedup();
    report("command verbs", "script-verb", missing);
}

/// Every word `measure` accepts, and every classical gas test.
///
/// The instrument tables are two: `[instrument]` is the name the bench
/// PRINTS in a reading, `[script-instrument]` is the word a learner
/// TYPES. Both are gated, because having one and not the other is a
/// bench that reports in German what it will only be asked for in
/// English.
#[test]
fn every_instrument_and_gas_test_word_has_german() {
    let de = de();
    let mut missing: Vec<String> = kerotakis_core::script::INSTRUMENT_WORDS
        .iter()
        .map(|(word, _)| *word)
        .chain(kerotakis_core::script::GAS_TEST_WORDS.iter().copied())
        // Only the first spelling of each is gated: `temp`, `mp` and
        // `melting-point` are shorthands for a word that is already in
        // the table, and asking a translator for German for every
        // abbreviation of the same instrument is asking for noise.
        .filter(|word| {
            !matches!(
                *word,
                "temp"
                    | "mass"
                    | "phmeter"
                    | "look"
                    | "gauge"
                    | "hydrometer"
                    | "densitometer"
                    | "uvvis"
                    | "column"
                    | "melting-point"
                    | "mp"
                    | "boiling-point"
                    | "bp"
            )
        })
        .filter(|word| de.lookup(&format!("script-instrument.{word}")).is_none())
        .filter(|word| de.lookup(&format!("script-test.{word}")).is_none())
        .map(str::to_string)
        .collect();
    missing.sort_unstable();
    missing.dedup();
    report("instrument and test words", "script-instrument", missing);
}

/// Every kind of glassware the bench can actually draw.
///
/// `VESSEL_KINDS` is the list `Vessel::new` accepts, and it holds the
/// SHORT labels — `tube`, `cylinder` — while the catalogue had only the
/// long names beside them, `test tube` and `measuring cylinder`. So the
/// two vessels a learner reaches for after the beaker both introduced
/// themselves in English. Nothing was missing from the file; the file was
/// keyed on words the engine never emits.
#[test]
fn every_glassware_kind_the_bench_draws_has_german() {
    let de = de();
    let missing: Vec<String> = VESSEL_KINDS
        .iter()
        .map(|(kind, _)| *kind)
        .filter(|kind| de.lookup(&format!("glassware.{kind}")).is_none())
        .map(str::to_string)
        .collect();
    report("glassware kinds", "glassware", missing);
}

/// Every instrument that can name itself in a reading.
#[test]
fn every_instrument_has_german() {
    let de = de();
    let missing: Vec<String> = serde_variants(
        serde_json::Value::String(PROBE.to_string()),
        err_of::<Instrument>,
    )
    .into_iter()
    .map(|tag| {
        let instrument: Instrument = serde_json::from_value(serde_json::Value::String(tag.clone()))
            .unwrap_or_else(|e| panic!("serde named a variant it cannot parse: {tag}: {e}"));
        instrument_name(instrument)
    })
    .filter(|name| de.lookup(&format!("instrument.{name}")).is_none())
    .map(str::to_string)
    .collect();
    report("instruments", "instrument", missing);
}

/// What a precipitate or a plating looks like.
///
/// The colour is the observation — "a pale blue solid appears" is the
/// whole lesson of a precipitation — and every one of the forty-seven
/// words in the registry was being dropped into the German sentence in
/// English. This inventory had no catalogue section at all before this
/// gate; `render::appearance_word` now looks them up.
#[test]
fn every_appearance_word_has_german() {
    let de = de();
    let mut missing: Vec<String> = species::registry()
        .iter()
        .filter_map(|s| s.appearance)
        // The word `render.rs` uses for a species the registry does not
        // describe. It reaches the same sentence, so it is checked here.
        .chain(std::iter::once("new"))
        .filter(|word| de.lookup(&format!("appearance.{word}")).is_none())
        .map(str::to_string)
        .collect();
    missing.sort_unstable();
    missing.dedup();
    report("appearance words", "appearance", missing);
}

/// Every flame-test colour.
///
/// Also asserted in `hazard_locale.rs`, from the other direction. Kept
/// here because this file is meant to be the one place that lists what
/// the engine can emit, and a reader checking coverage should not have to
/// know that flame colours live somewhere else.
#[test]
fn every_flame_colour_has_german() {
    let de = de();
    let mut missing: Vec<String> = species::registry()
        .iter()
        .filter_map(|s| s.flame_colour)
        .filter(|colour| de.lookup(&format!("flame.{colour}")).is_none())
        .map(str::to_string)
        .collect();
    missing.sort_unstable();
    missing.dedup();
    report("flame colours", "flame", missing);
}

/// The four phases, and the verbs for moving between them.
///
/// `{:?}` on a phase printed "Liquid" in every language once, which is
/// how these got a table. The verbs are separate because German inflects
/// them — gefror, schmolz, siedete — where English appends nothing.
#[test]
fn every_phase_and_phase_change_verb_has_german() {
    let de = de();
    let phases: Vec<Phase> = serde_variants(
        serde_json::Value::String(PROBE.to_string()),
        err_of::<Phase>,
    )
    .into_iter()
    .map(|tag| {
        serde_json::from_value(serde_json::Value::String(tag.clone()))
            .unwrap_or_else(|e| panic!("serde named a phase it cannot parse: {tag}: {e}"))
    })
    .collect();

    let missing: Vec<String> = phases
        .iter()
        .map(|p| phase_key(*p))
        // Phase keys name a place, so they go through `t` and the whole
        // key is what must exist — `phase.gas`, not `gas`.
        .filter(|key| de.lookup(key).is_none())
        .map(str::to_string)
        .collect();
    report("phases", "phase", missing);

    let mut verbs: Vec<String> = Vec::new();
    for from in &phases {
        for to in &phases {
            let verb = phase_change_verb(*from, *to);
            if de.lookup(&format!("verb.{verb}")).is_none() {
                verbs.push(verb.to_string());
            }
        }
    }
    verbs.sort_unstable();
    verbs.dedup();
    report("phase-change verbs", "verb", verbs);
}

/// Both words the hazard banner can use for how bad this is.
#[test]
fn every_hazard_severity_has_german() {
    let de = de();
    let missing: Vec<String> = serde_variants(
        serde_json::Value::String(PROBE.to_string()),
        err_of::<Severity>,
    )
    .into_iter()
    .map(|tag| {
        let severity: Severity = serde_json::from_value(serde_json::Value::String(tag.clone()))
            .unwrap_or_else(|e| panic!("serde named a severity it cannot parse: {tag}: {e}"));
        // The catalogue is keyed by the Debug spelling, which is what
        // `render.rs` has at the call site.
        format!("{severity:?}")
    })
    .filter(|name| de.lookup(&format!("severity.{name}")).is_none())
    .collect();
    report("hazard severities", "severity", missing);
}

// ── The events ──────────────────────────────────────────────────────

/// Every event the engine can emit says SOMETHING in German.
///
/// The other tests ask whether a word has a translation. This one asks a
/// blunter question: is there any German for this event at all? A new
/// `Event` variant whose renderer reaches for `format!` is not merely
/// untranslated but untranslatABLE, and no amount of work in the .toml
/// changes that — so the check has to start from the enum, not the file.
///
/// It found twelve. Seven were the fermentation, emulsion, curdling,
/// surface-spread, surface-colour and material-layer events, added after
/// the last translation pass. The other five — the spill and container
/// breakage events — were merged to main while this file was being
/// written, and this test caught them on the rebase, the same afternoon.
/// `engine-locale-lint.py` reported 100% coverage over every one of them,
/// truthfully: the coverage was measured over the keys that existed.
///
/// LV1 only, deliberately — see the header for why the evidence
/// registers are a separate job.
#[test]
fn every_event_the_engine_can_emit_has_a_german_sentence() {
    let catalogue = catalogue();
    let events = catalogue
        .get("event")
        .and_then(toml::Value::as_table)
        .expect("de.toml has an [event] section");

    let missing: Vec<String> =
        serde_variants(serde_json::json!({ "event": PROBE }), err_of::<Event>)
            .into_iter()
            .map(|tag| {
                // The enum is `rename_all = "snake_case"`; the catalogue spells
                // the same names in kebab-case, because that is what reads well
                // in a dotted key.
                let key = tag.replace('_', "-");
                (tag, key)
            })
            .filter(|(_, key)| {
                !events.keys().any(|k| {
                    k.strip_prefix(key.as_str())
                        .is_some_and(|rest| rest.starts_with(".lv1"))
                })
            })
            .map(|(tag, key)| format!("{tag}  (wanted a key like \"{key}.lv1\")"))
            .collect();

    assert!(
        missing.is_empty(),
        "{} event(s) the engine can emit have no German LV1 sentence.\n\
         If the renderer still uses `format!` for that arm, the sentence is \
         untranslatable and the call site needs `locale.fill` first:\n  {}",
        missing.len(),
        missing.join("\n  ")
    );
}

// ── The relations ───────────────────────────────────────────────────

/// Every relation carries its German with it.
///
/// This surface does not use the catalogue at all: the German rides in
/// `purpose_de` / `validity_de` / `source_de` beside the English, and
/// `tEngine` in the shell picks the sibling field. Same silent failure
/// though — an empty `_de` falls back to English and looks like a
/// deliberate choice — so it is gated the same way.
///
/// `validity` is the field this most matters for: it is the sentence
/// saying where the formula stops being true, and a learner who cannot
/// read it is the learner who misuses the equation.
#[test]
fn every_relation_carries_its_german() {
    let mut missing: Vec<String> = Vec::new();
    for r in kerotakis_core::relations::RELATIONS {
        for (field, en, de) in [
            ("purpose", r.purpose, r.purpose_de),
            ("validity", r.validity, r.validity_de),
            ("source", r.source, r.source_de),
        ] {
            if de.trim().is_empty() {
                missing.push(format!("{}.{field}_de is empty", r.name));
            } else if de == en {
                // Not always wrong — a formula name can be identical —
                // but it is how an un-translated copy-paste looks, and
                // every relation shipped so far does differ.
                missing.push(format!("{}.{field}_de is the English verbatim", r.name));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "{} relation field(s) have no German:\n  {}",
        missing.len(),
        missing.join("\n  ")
    );
}

// ── The gate's own wiring ───────────────────────────────────────────

/// The variant reader actually reads variants.
///
/// Everything above rests on serde's error message keeping its shape. If
/// serde reworded it, `serde_variants` would return an empty list and
/// every event and instrument would pass by checking nothing — the
/// quietest way for a gate to die. The empty case panics, and this pins
/// a name the enum really has so that a rewording fails here first, with
/// an explanation, rather than as a mystery.
#[test]
fn the_variant_reader_is_not_silently_reading_nothing() {
    let instruments = serde_variants(
        serde_json::Value::String(PROBE.to_string()),
        err_of::<Instrument>,
    );
    assert!(
        instruments.iter().any(|v| v == "ph_meter"),
        "serde no longer lists variants the way this gate reads them: {instruments:?}"
    );
    let events = serde_variants(serde_json::json!({ "event": PROBE }), err_of::<Event>);
    assert!(
        events.len() > 50,
        "the Event enum has {} variants by this reading, which cannot be right: {events:?}",
        events.len()
    );
}

/// The German a species gets is the German the journal prints.
///
/// A catalogue entry is evidence that a string exists, never that anyone
/// sees it — this codebase has been wrong about that five times, and all
/// five are written up in I18N.md. So one term is followed all the way
/// through a rendered line, to keep the lookups above honest about the
/// path they claim to be checking.
#[test]
fn a_gated_term_reaches_the_rendered_line() {
    use kerotakis_core::render::{render_event_in, Register};
    use kerotakis_core::vessel::VesselId;
    use kerotakis_core::SpeciesId;

    // Sucrose was one of the seventeen this gate found.
    let event = Event::Dissolved {
        vessel: VesselId(0),
        species: SpeciesId::new("sucrose"),
        moles: kerotakis_core::units::Moles(0.1),
    };
    let line = render_event_in(&event, Register::LV1, de());
    assert!(
        line.contains("Saccharose"),
        "the catalogue has the word but the renderer does not use it: {line}"
    );
    assert!(!line.contains("sucrose"), "English leaked: {line}");
}
