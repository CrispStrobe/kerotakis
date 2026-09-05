//! The engine's own prose, in German (I18N-5).
//!
//! The catalogue's `_de` siblings cannot reach these lines: they are
//! composed here out of fragments, so the engine has to know the reader's
//! language. These tests pin the two things that are easy to get subtly
//! wrong — the decimal comma, which must reach the measurements and not
//! the identifiers, and the fallback, which must leave an untranslated
//! word in English rather than dropping it.

use kerotakis_core::render::{render_vessel, render_vessel_in, Register};
use kerotakis_core::vessel::VesselId;
use kerotakis_core::Locale;
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
        render_vessel_in(&v, Register::LV2, Locale::EN)
    );
    let line = &render_vessel(&v, Register::LV2)[0];
    assert!(line.contains("beaker"), "{line}");
    assert!(line.contains("open to atmosphere"), "{line}");
    assert!(line.contains("25.00 °C"), "{line}");
}

#[test]
fn german_names_the_glassware_and_the_boundary() {
    let line = render_vessel_in(&beaker(), Register::LV2, Locale::parse("de"))
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
    let line = render_vessel_in(&beaker(), Register::LV2, Locale::parse("de"))
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
    let lines = render_vessel_in(&beaker(), Register::LV2, Locale::parse("de"));
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
    let line = render_vessel_in(&v, Register::LV2, Locale::parse("de"))
        .into_iter()
        .next()
        .unwrap();
    assert!(line.contains("spectrophotometer cell"), "{line}");
    // …and the rest of the sentence is still German.
    assert!(line.contains("zur Atmosphäre offen"), "{line}");
}

#[test]
fn locale_parsing_falls_back_to_english_rather_than_failing() {
    assert_eq!(Locale::parse("de"), Locale::parse("de"));
    assert_eq!(Locale::parse("de-DE"), Locale::parse("de"));
    assert_eq!(Locale::parse("DE-at"), Locale::parse("de"));
    assert_eq!(Locale::parse("en"), Locale::EN);
    // A language nobody has translated to should show the language we do
    // have, not an error and not an empty screen.
    assert_eq!(Locale::parse("fr"), Locale::EN);
    assert_eq!(Locale::parse(""), Locale::EN);
}

/// A beaker holding one portion, for the contents-row tests.
fn beaker_with_water() -> Vessel {
    use kerotakis_core::species::{Phase, SpeciesId};
    use kerotakis_core::vessel::Portion;
    use kerotakis_core::Moles;

    let mut v = Vessel::new(VesselId(0), "beaker");
    v.contents.push(Portion {
        // The registry is keyed by the species NAME, not its formula:
        // SpeciesId("H2O") misses and falls back to printing the id.
        species: SpeciesId("water".into()),
        moles: Moles(11.0686),
        phase: Phase::Liquid,
    });
    v
}

#[test]
fn contents_rows_are_german_too_including_their_decimals() {
    let lines = render_vessel_in(&beaker_with_water(), Register::LV2, Locale::parse("de"));
    let row = lines
        .iter()
        .find(|l| l.contains("mol"))
        .expect("a contents row");
    assert!(row.contains("11,0686"), "decimal comma missing: {row}");
    assert!(!row.contains("11.0686"), "the point survived: {row}");
    assert!(row.contains("Wasser"), "species not translated: {row}");
    assert!(row.contains("flüssig"), "phase not translated: {row}");
    assert!(!row.contains("Liquid"), "the Debug phase leaked: {row}");
}

#[test]
fn english_contents_rows_are_unchanged() {
    let row = render_vessel(&beaker_with_water(), Register::LV2)
        .into_iter()
        .find(|l| l.contains("mol"))
        .expect("a contents row");
    assert!(row.contains("11.0686"), "{row}");
    assert!(row.contains("water"), "{row}");
    assert!(row.contains("Liquid"), "{row}");
}

/// One "you added water" event, for the journal tests.
fn water_added() -> kerotakis_core::ops::Event {
    use kerotakis_core::ops::Event;
    use kerotakis_core::species::SpeciesId;
    use kerotakis_core::Moles;

    Event::Added {
        vessel: VesselId(0),
        species: SpeciesId("water".into()),
        moles: Moles(11.0686),
        total_after: None,
    }
}

#[test]
fn the_journal_speaks_german_and_counts_in_german() {
    use kerotakis_core::render::render_event_in;

    let de = Locale::parse("de");
    let event = water_added();

    let lv1 = render_event_in(&event, Register::LV1, de);
    assert!(lv1.contains("Du gibst"), "{lv1}");
    assert!(lv1.contains("Wasser"), "{lv1}");
    assert!(!lv1.contains("You add"), "{lv1}");

    // The decimal comma is what the shell's regex layer cannot do: by the
    // time a line reaches engineText.ts the number is already formatted.
    // So a comma here is proof the sentence came from the catalogue.
    let lv2 = render_event_in(&event, Register::LV2, de);
    assert!(lv2.contains("11,0686"), "wanted a comma: {lv2}");
    assert!(!lv2.contains("11.0686"), "the point survived: {lv2}");
}

#[test]
fn the_english_journal_is_untouched() {
    use kerotakis_core::render::render_event;

    let event = water_added();
    assert_eq!(render_event(&event, Register::LV1), "You add water to v1.");
    assert!(render_event(&event, Register::LV2).contains("11.0686"));
}

#[test]
fn a_refusal_reason_is_german_too() {
    use kerotakis_core::ops::Event;
    use kerotakis_core::render::render_event_in;

    let event = Event::NotYetModeled {
        cause: kerotakis_core::ops::NotModelledCause::NothingToActOn,
        vessel: VesselId(0),
        what: "nothing to evaporate — no water in the vessel".into(),
    };
    let line = render_event_in(&event, Register::LV2, Locale::parse("de"));
    assert!(line.contains("nichts zu verdampfen"), "{line}");
    assert!(
        !line.contains("nothing to evaporate"),
        "English survived: {line}"
    );
}

#[test]
fn an_untranslated_refusal_keeps_its_english() {
    // A reason with an interpolated value cannot be matched by text, so it
    // must come through readable rather than blank. The sentence is then
    // German with an English clause in it, which is visible and honest —
    // losing the clause would not be.
    use kerotakis_core::ops::Event;
    use kerotakis_core::render::render_event_in;

    let event = Event::NotYetModeled {
        cause: kerotakis_core::ops::NotModelledCause::NothingToActOn,
        vessel: VesselId(0),
        what: "nothing here can be electrolysed: no ions".into(),
    };
    let line = render_event_in(&event, Register::LV2, Locale::parse("de"));
    assert!(
        line.contains("noch nicht modelliert"),
        "the frame is German: {line}"
    );
    assert!(line.contains("nothing here can be electrolysed"), "{line}");
}

#[test]
fn electrolysis_operating_point_is_german_and_keeps_numeric_evidence() {
    use kerotakis_core::ops::Event;
    use kerotakis_core::render::render_event_in;
    use kerotakis_core::species::SpeciesId;
    use kerotakis_core::Moles;

    let event = Event::Electrolysed {
        vessel: VesselId(0),
        species: SpeciesId("copper".into()),
        amps: 0.5,
        seconds: 120.0,
        coulombs: 60.0,
        electrons: Moles(0.000622),
        moles: Moles(0.000311),
        grams: 0.0198,
        per_ion: 2.0,
    };
    let line = render_event_in(&event, Register::LV2, Locale::parse("de"));
    assert!(line.contains("0,500 A für 120 s"), "{line}");
    assert!(line.contains("Kupfer"), "{line}");
    assert!(!line.contains(" A for "), "{line}");
}

#[test]
fn bounded_acid_metal_cell_is_german_and_names_its_assumption() {
    use kerotakis_core::ops::Event;
    use kerotakis_core::render::render_event_in;

    let event = Event::AcidMetalCellVoltage {
        anode: VesselId(0),
        cathode: VesselId(1),
        volts: 0.652,
        ph: 1.86,
    };
    let rendered = render_event_in(&event, Register::LV2, Locale::parse("de"));
    assert!(rendered.contains("Einheitsaktivität"), "{rendered}");
    assert!(rendered.contains("0,652 V"), "{rendered}");
    assert!(rendered.contains("Innenwiderstand"), "{rendered}");
}

// ── The leak sweep (I18N): English fragments inside German sentences ──
//
// Each of these was reported from the live German deploy. They share one
// shape: a fragment composed in Rust and dropped into a German template,
// which no amount of catalogue work can reach. A test per fragment, so a
// regression names the fragment rather than "the German is wrong".

#[test]
fn a_contained_gas_names_its_species_in_german() {
    use kerotakis_core::ops::Event;
    use kerotakis_core::render::render_event_in;
    use kerotakis_core::species::SpeciesId;
    use kerotakis_core::Moles;

    // "0,0108 mol carbon dioxide gebildet und verbleibt im geschlossenen
    // Gasraum" — the numbers went through the catalogue and the substance
    // did not, which is the one word the sentence is about.
    let event = Event::GasContained {
        vessel: VesselId(0),
        species: SpeciesId("CO2".into()),
        moles: Moles(0.0108),
    };
    for register in [Register::LV2, Register::LV3] {
        let line = render_event_in(&event, register, Locale::parse("de"));
        assert!(line.contains("Kohlenstoffdioxid"), "{line}");
        assert!(!line.contains("carbon dioxide"), "{line}");
    }
}

#[test]
fn an_absorbed_or_evolved_gas_names_its_species_in_german() {
    use kerotakis_core::ops::Event;
    use kerotakis_core::render::render_event_in;
    use kerotakis_core::species::SpeciesId;
    use kerotakis_core::Moles;

    // The two siblings of the arm above, which had the same hole.
    let evolved = Event::GasEvolved {
        vessel: VesselId(0),
        species: SpeciesId("CO2".into()),
        moles: Moles(0.0108),
    };
    let absorbed = Event::GasAbsorbed {
        vessel: VesselId(0),
        species: SpeciesId("CO2".into()),
        moles: Moles(0.0108),
    };
    for event in [evolved, absorbed] {
        let line = render_event_in(&event, Register::LV2, Locale::parse("de"));
        assert!(line.contains("Kohlenstoffdioxid"), "{line}");
        assert!(!line.contains("carbon dioxide"), "{line}");
    }
}

#[test]
fn an_ignition_reports_its_flame_colour_in_german_at_every_level() {
    use kerotakis_core::ops::Event;
    use kerotakis_core::render::render_event_in;

    // LV1 already looked the colour up; LV2 composed " — orange-red
    // flame" in Rust, so the German line ended in two English words.
    let event = Event::Ignited {
        vessel: VesselId(0),
        flame: Some("orange-red".into()),
        energy_j: Some(1234.0),
    };
    let de = Locale::parse("de");
    for register in [Register::LV1, Register::LV2] {
        let line = render_event_in(&event, register, de);
        assert!(line.contains("orangerot"), "{line}");
        assert!(!line.contains("orange-red"), "{line}");
        assert!(!line.contains("flame"), "{line}");
    }
    // The energy beside it is a measurement, so it takes the comma.
    let lv2 = render_event_in(&event, Register::LV2, de);
    assert!(lv2.contains("1,23 kJ"), "{lv2}");
    assert!(!lv2.contains("released"), "{lv2}");
}

#[test]
fn the_english_ignition_line_is_unchanged() {
    use kerotakis_core::ops::Event;
    use kerotakis_core::render::render_event;

    let event = Event::Ignited {
        vessel: VesselId(0),
        flame: Some("orange-red".into()),
        energy_j: Some(1234.0),
    };
    assert_eq!(
        render_event(&event, Register::LV2),
        "v1: ignited — orange-red flame · 1.23 kJ released"
    );
}

#[test]
fn a_temperature_change_counts_in_german() {
    use kerotakis_core::ops::Event;
    use kerotakis_core::render::render_event_in;
    use kerotakis_core::Kelvin;

    // "25.0 °C → 30.5 °C" — the whole line was two numbers, and they were
    // the one thing not going through the catalogue.
    let event = Event::TemperatureChanged {
        vessel: VesselId(0),
        from: Kelvin::from_celsius(25.0),
        to: Kelvin::from_celsius(30.5),
    };
    let lv2 = render_event_in(&event, Register::LV2, Locale::parse("de"));
    assert!(lv2.contains("25,0 °C"), "{lv2}");
    assert!(lv2.contains("30,5 °C"), "{lv2}");
    assert!(!lv2.contains("25.0"), "the point survived: {lv2}");

    // And English keeps the point, including the vessel's own identifier.
    let en = kerotakis_core::render::render_event(&event, Register::LV2);
    assert_eq!(en, "v1: 25.0 °C → 30.5 °C");
}

#[test]
fn a_dissolution_counts_in_german_and_names_what_dissolved() {
    use kerotakis_core::ops::Event;
    use kerotakis_core::render::render_event_in;
    use kerotakis_core::species::SpeciesId;
    use kerotakis_core::Moles;

    let event = Event::Dissolved {
        vessel: VesselId(0),
        species: SpeciesId("NaCl".into()),
        moles: Moles(0.1234),
    };
    let lv2 = render_event_in(&event, Register::LV2, Locale::parse("de"));
    assert!(lv2.contains("Natriumchlorid"), "{lv2}");
    assert!(lv2.contains("0,1234"), "{lv2}");
    assert!(!lv2.contains("dissolved"), "{lv2}");
}

/// The catalogue may not hold a key nobody asks for — and, worse, may not
/// hold one under the WRONG prefix.
///
/// Two chromatogram sentences were written under `[flame]`, so their key
/// was `flame.chromatographed.lv1-nothing-separated` while the engine
/// asked for `event.chromatographed.lv1-nothing-separated`. The German
/// existed, was correct, and never rendered. `engine-locale-lint.py`
/// could not see it either: `flame.*` is looked up by value and therefore
/// exempt from its orphan check, so the misfiling hid inside the
/// exemption. This asserts the shape of the section rather than the one
/// bug — no `[flame]` entry may contain a dot.
#[test]
fn no_flame_colour_is_really_a_dotted_key_from_another_section() {
    let catalogue: toml::Table = include_str!("../i18n/de.toml")
        .parse()
        .expect("de.toml parses");
    let flame = catalogue["flame"].as_table().expect("[flame] is a table");
    let misfiled: Vec<&String> = flame.keys().filter(|k| k.contains('.')).collect();
    assert!(
        misfiled.is_empty(),
        "a dotted key under [flame] is a key filed in the wrong section — \
         it will never be found under the prefix it names: {misfiled:?}"
    );
}

/// Every prefix the engine looks a value up under must have a section.
///
/// `locale.lookup(&format!("transition.{kind}"))` had no `[transition]`
/// section at all, so a German reader was told "v1 melting point:
/// Saccharose melts bei 185,0 °C" — two English words in the middle of a
/// German line. Nothing reported it: `i18n_coverage.rs` walks the
/// registries it can enumerate, and a lookup keyed by a value under a
/// prefix is exactly the shape it cannot; `engine-locale-lint.py` treats
/// these prefixes as satisfied by definition, so an EMPTY one is
/// indistinguishable from a full one.
///
/// So this reads the prefixes out of the source rather than listing them,
/// and asks only the question those two gates cannot: does the section
/// exist and does it hold anything. Coverage within a section stays the
/// business of the inventory that can enumerate it.
#[test]
fn every_prefix_the_engine_looks_up_has_a_german_section() {
    let source = include_str!("../src/render.rs");
    let catalogue: toml::Table = include_str!("../i18n/de.toml")
        .parse()
        .expect("de.toml parses");

    let mut prefixes: Vec<&str> = source
        .match_indices(".lookup(&format!(\"")
        .filter_map(|(at, marker)| {
            let rest = &source[at + marker.len()..];
            rest.find(".{").map(|end| &rest[..end])
        })
        .collect();
    prefixes.sort_unstable();
    prefixes.dedup();
    assert!(
        prefixes.len() >= 10,
        "the lookup shape must have changed — this gate is now checking \
         almost nothing: {prefixes:?}"
    );

    let empty: Vec<&str> = prefixes
        .iter()
        .copied()
        .filter(|prefix| {
            let (section, nested) = prefix
                .split_once('.')
                .map_or((*prefix, None), |(s, n)| (s, Some(n)));
            let Some(table) = catalogue.get(section).and_then(toml::Value::as_table) else {
                return true;
            };
            match nested {
                // `transition.outcome.{word}` flattens to keys that BEGIN
                // "outcome." inside `[transition]`.
                Some(inner) => !table.keys().any(|k| k.starts_with(&format!("{inner}."))),
                None => table.is_empty(),
            }
        })
        .collect();
    assert!(
        empty.is_empty(),
        "the engine looks words up under these prefixes and de.toml answers \
         nothing, so each renders its English inside a German sentence: {empty:?}"
    );
}
