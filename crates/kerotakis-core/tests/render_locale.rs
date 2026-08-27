//! The engine's own prose, in German (I18N-5).
//!
//! The catalogue's `_de` siblings cannot reach these lines: they are
//! composed here out of fragments, so the engine has to know the reader's
//! language. These tests pin the two things that are easy to get subtly
//! wrong — the decimal comma, which must reach the measurements and not
//! the identifiers, and the fallback, which must leave an untranslated
//! word in English rather than dropping it.

use kerotakis_core::render::{render_vessel, render_vessel_in, Locale, Register};
use kerotakis_core::vessel::VesselId;
use kerotakis_core::Vessel;

fn beaker() -> Vessel {
    // The default temperature is 25 °C, which is the number these tests
    // want anyway; setting it explicitly would only assert on the setter.
    Vessel::new(VesselId(0), "beaker")
}

#[test]
fn the_english_rendering_is_unchanged() {
    // The old signature still means English, which is what keeps the
    // eight existing callers — mostly tests asserting on English prose —
    // from needing to care about locale at all.
    let v = beaker();
    assert_eq!(
        render_vessel(&v, Register::LV2),
        render_vessel_in(&v, Register::LV2, Locale::En)
    );
    let line = &render_vessel(&v, Register::LV2)[0];
    assert!(line.contains("beaker"), "{line}");
    assert!(line.contains("open to atmosphere"), "{line}");
    assert!(line.contains("25.00 °C"), "{line}");
}

#[test]
fn german_names_the_glassware_and_the_boundary() {
    let line = render_vessel_in(&beaker(), Register::LV2, Locale::De)
        .into_iter()
        .next()
        .unwrap();
    assert!(line.contains("Becherglas"), "{line}");
    assert!(line.contains("zur Atmosphäre offen"), "{line}");
    assert!(line.contains("Flüssigkeit"), "{line}");
    assert!(!line.contains("beaker"), "English leaked: {line}");
    assert!(!line.contains("liquid"), "English leaked: {line}");
}

#[test]
fn german_writes_the_decimal_comma_but_keeps_the_vessel_id() {
    let line = render_vessel_in(&beaker(), Register::LV2, Locale::De)
        .into_iter()
        .next()
        .unwrap();
    assert!(line.contains("25,00 °C"), "wanted a decimal comma: {line}");
    assert!(!line.contains("25.00"), "point survived in prose: {line}");
    // `v1` has no point, but a compartment id like `v1.2` does, and it is
    // a name rather than a quantity — swapping its separator would rename
    // the vessel. The id is emitted outside the number() call for exactly
    // this reason, so pin it.
    assert!(line.starts_with("v1 ("), "{line}");
}

#[test]
fn an_empty_vessel_says_so_in_german() {
    let lines = render_vessel_in(&beaker(), Register::LV2, Locale::De);
    assert!(
        lines.iter().any(|l| l.contains("(leer)")),
        "no German empty marker in {lines:?}"
    );
    assert!(
        !lines.iter().any(|l| l.contains("(empty)")),
        "English empty marker survived in {lines:?}"
    );
}

#[test]
fn an_untranslated_vessel_keeps_its_english_name() {
    // Better a word we have than a word we do not: a vessel missing from
    // the table must come through readable, not blank.
    let v = Vessel::new(VesselId(0), "spectrophotometer cell");
    let line = render_vessel_in(&v, Register::LV2, Locale::De)
        .into_iter()
        .next()
        .unwrap();
    assert!(line.contains("spectrophotometer cell"), "{line}");
    // …and the rest of the sentence is still German.
    assert!(line.contains("zur Atmosphäre offen"), "{line}");
}

#[test]
fn locale_parsing_falls_back_to_english_rather_than_failing() {
    assert_eq!(Locale::parse("de"), Locale::De);
    assert_eq!(Locale::parse("de-DE"), Locale::De);
    assert_eq!(Locale::parse("DE-at"), Locale::De);
    assert_eq!(Locale::parse("en"), Locale::En);
    // A language nobody has translated to should show the language we do
    // have, not an error and not an empty screen.
    assert_eq!(Locale::parse("fr"), Locale::En);
    assert_eq!(Locale::parse(""), Locale::En);
}
