//! The bench refuses in the reader's language.
//!
//! "many warnings/errors/info strings are NOT showing in German, like 'no
//! vessel v2'" — reported against the live German deploy, and true. Every
//! [`BenchError`] arm composed a finished English sentence, both hosts
//! called `to_string()` on it, and the sentence reached the browser as an
//! exception message. There was no key to translate, so no catalogue could
//! reach it and no coverage gate could report it missing: the refusals were
//! not untranslated, they were untranslatable.
//!
//! Three claims are kept here.
//!
//! 1. **The English did not move.** A CLI user, a log line and every golden
//!    fixture see exactly the sentence they saw before. This is what makes
//!    the change safe to land: the new template *generates* the old string.
//! 2. **German arrives, with the values intact.** `v2` is a name and stays
//!    `v2`; a measured number picks up the German decimal comma.
//! 3. **Both bindings do it.** The browser reaches the engine through wasm
//!    and the App Store build reaches it through Tauri, and a translation
//!    that only one of them applies is the failure I18N.md records under
//!    "two hosts, and only one of them used to speak German".
//!
//! The catalogue-coverage half — a key with no German fails — lives in
//! `i18n_coverage.rs` with the other inventories.

use kerotakis_core::bench::BenchError;
use kerotakis_core::species::SpeciesId;
use kerotakis_core::stock::StockUnit;
use kerotakis_core::vessel::VesselId;
use kerotakis_core::{Locale, Refuses};

fn de() -> Locale {
    Locale::parse("de")
}

/// One of every arm, so the wording below is pinned rather than described.
///
/// This is a list, which the rest of the i18n gates go out of their way not
/// to be — and here it is the point: a golden of the English text cannot be
/// derived from the thing it is guarding. `every_refusal_key_has_german` in
/// `i18n_coverage.rs` reads the keys out of the source instead, so a new
/// arm still fails a gate even if nobody adds it here.
fn one_of_each() -> Vec<(BenchError, &'static str)> {
    vec![
        (
            BenchError::NoSuchVessel(VesselId(1)),
            "no vessel v2 — make it first with `new`, which creates the next free vessel",
        ),
        (
            BenchError::UnknownSpecies(SpeciesId::new("unobtainium")),
            "unknown species 'unobtainium' — not in the registry",
        ),
        (
            BenchError::UnknownMaterial("moon dust".into()),
            "unknown material 'moon dust' — not in the recipe registry",
        ),
        (
            BenchError::MaterialRecipeMismatch,
            "material recipe identity does not match the pinned operator",
        ),
        (BenchError::NonPositiveAmount, "amount must be positive"),
        (
            BenchError::UnstockableKey("unobtainium".into()),
            "nothing on the shelf is called 'unobtainium' — it is neither a species nor a material",
        ),
        (
            BenchError::StockExhausted {
                key: "vinegar".into(),
                requested: 50.0,
                remaining: 20.0,
                unit: StockUnit::Millilitre,
            },
            "the 'vinegar' bottle holds 20 mL, and 50 mL was asked for",
        ),
        (BenchError::BadFraction, "fraction must be within 0..=1"),
        (
            BenchError::SelfTransfer,
            "source and target vessel are the same",
        ),
        (
            BenchError::VesselNotEmpty(VesselId(1)),
            "vessel v2 is not empty — transfer or dispose of its contents first",
        ),
        (
            BenchError::LastVessel,
            "the last vessel must stay on the bench",
        ),
        (
            BenchError::BrokenVessel(VesselId(0)),
            "vessel v1 is broken and cannot be used",
        ),
        (
            BenchError::NoSuchSpill,
            "no spill exists at the requested destination",
        ),
        (
            BenchError::SolidNotPresent {
                vessel: VesselId(0),
                species: SpeciesId::new("chalk"),
            },
            "vessel v1 contains no solid chalk to grind — grinding changes a solid's particle \
             size, so it has to happen before the solid dissolves, not after",
        ),
        (
            BenchError::CentrifugeUnavailable("it holds no liquid".into()),
            "centrifuge cannot run this vessel: it holds no liquid",
        ),
        (
            BenchError::CentrifugeImbalance {
                sample_g: 5.0,
                counterbalance_g: 4.5,
                imbalance_g: 0.5,
            },
            "centrifuge rotor is 0.50 g out of balance (sample 5.00 g, counterbalance 4.50 g); \
             match within 0.10 g",
        ),
        (
            BenchError::Kinetics(kerotakis_core::kinetics::IntegrationError::NoProgress {
                network: "iodine-clock".into(),
            }),
            "implicit integration for network 'iodine-clock' stopped without advancing time",
        ),
        (
            BenchError::Transport(kerotakis_core::transport::TransportError::EmptyChain),
            "a 1-D cell chain needs at least one cell",
        ),
    ]
}

/// Not one word of English moved.
///
/// `thiserror`'s `#[error(...)]` attribute is gone and the sentences now
/// live in `Refusal::new`, which is a rewrite of every message on the
/// bench's refusal path. The whole safety argument for doing it that way is
/// that `Display` is `render(Locale::EN)` over the same template, so the
/// English is generated rather than copied — and this is where that claim
/// is checked, against the strings as they shipped.
#[test]
fn the_english_is_exactly_what_it_was() {
    for (error, english) in one_of_each() {
        assert_eq!(error.to_string(), english, "the English wording moved");
        assert_eq!(
            error.localize(Locale::EN),
            english,
            "Display and localize(EN) disagree, so there are two copies of the English"
        );
    }
}

/// The report, answered.
#[test]
fn no_vessel_v2_reads_german_and_keeps_its_name() {
    let refused = BenchError::NoSuchVessel(VesselId(1));
    let line = refused.localize(de());
    assert!(
        line.contains("kein Gefäß"),
        "still English on a German bench: {line}"
    );
    assert!(
        !line.contains("no vessel"),
        "English survived the translation: {line}"
    );
    // `v2` is a NAME. A translation that localises it has renamed a vessel
    // the learner can see on the bench, which is worse than English.
    assert!(line.contains("v2"), "the vessel lost its name: {line}");
}

/// Every arm says something in German, and says it with its values in it.
///
/// The blunt version of the claim: it is not enough for the key to exist,
/// the rendered line must differ from the English and must still carry
/// whatever the learner needs to act on.
#[test]
fn every_arm_renders_german_without_losing_its_values() {
    let mut english_still: Vec<String> = Vec::new();
    for (error, english) in one_of_each() {
        let german = error.localize(de());
        // The two pass-through arms deliberately keep the nested error's
        // own English — `{detail}` is a frame waiting for the day those
        // types get keys, not a translation.
        if matches!(error, BenchError::Kinetics(_) | BenchError::Transport(_)) {
            assert_eq!(german, english, "a pass-through arm changed its text");
            continue;
        }
        if german == english {
            english_still.push(format!("{}: {english}", error.refusal().key));
        }
        // A hole the catalogue misspells reads as `{vesel}` on screen
        // rather than vanishing; nothing here should be showing one.
        assert!(
            !german.contains('{'),
            "an unfilled hole reached the reader: {german}"
        );
    }
    assert!(
        english_still.is_empty(),
        "{} refusal(s) still render English in a German session:\n  {}",
        english_still.len(),
        english_still.join("\n  ")
    );
}

/// German writes 0,50 — and `v1` is still `v1`.
///
/// The split that `Refusal::with` / `with_number` exists for. Both of these
/// are in one sentence here, which is why the decision cannot be made once
/// for the whole refusal.
#[test]
fn a_measured_number_gets_the_german_comma_and_an_identifier_does_not() {
    let imbalanced = BenchError::CentrifugeImbalance {
        sample_g: 5.0,
        counterbalance_g: 4.5,
        imbalance_g: 0.5,
    };
    let line = imbalanced.localize(de());
    assert!(
        line.contains("0,50"),
        "German decimal comma missing: {line}"
    );
    assert!(!line.contains("0.50"), "English decimal point left: {line}");

    let broken = BenchError::BrokenVessel(VesselId(0));
    assert!(
        broken.localize(de()).contains("v1"),
        "a vessel id must not be reformatted as a number"
    );
}

/// Both bindings render the refusal; neither stringifies it.
///
/// I18N.md's "two hosts, and only one of them used to speak German" is the
/// most expensive mistake this codebase has made about translation: the
/// engine's whole catalogue applied to the browser and to nothing else, for
/// as long as the native binding imported the English-only renderers. The
/// refusal path is the same shape and would fail the same way, so it is
/// scraped rather than trusted.
///
/// `to_string()` on a bench error is the English sentence by construction,
/// so its presence on either host's step path is the bug itself.
#[test]
fn both_bindings_localize_the_refusal_rather_than_stringify_it() {
    for (host, source) in [
        ("wasm", include_str!("../../kerotakis-wasm/src/lib.rs")),
        (
            "native (Tauri)",
            include_str!("../../../web/app/src-tauri/src/lib.rs"),
        ),
    ] {
        assert!(
            source.contains("localize(self.locale)"),
            "the {host} binding never localises a refusal — a German session \
             reads English wherever the bench says no"
        );
        // The step path specifically. Serde and IO failures elsewhere in
        // these files are programmer-facing and stay as they are.
        let step = source
            .split("step_with(")
            .nth(1)
            .unwrap_or_else(|| panic!("the {host} binding no longer calls step_with"));
        let tail: String = step.chars().take(600).collect();
        assert!(
            !tail.contains("e.to_string()"),
            "the {host} binding stringifies the refusal from its step path, \
             which is the English sentence: {tail}"
        );
    }
}

/// A language nobody has written falls back per key, not per language.
#[test]
fn an_unshipped_language_reads_english_rather_than_nothing() {
    // `zz` is reserved and will never be a language, so this test does not
    // start failing on the day someone adds French.
    let line = BenchError::NonPositiveAmount.localize(Locale::parse("zz"));
    assert_eq!(line, "amount must be positive");
}
