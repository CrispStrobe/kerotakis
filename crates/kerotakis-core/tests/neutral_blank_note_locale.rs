//! Owned blank-water caveats reach real rendered measurements in every locale.
use kerotakis_core::render::{render_event_in, Register};
use kerotakis_core::*;

fn sample(glucose: bool) -> Bench {
    let mut bench = Bench::new();
    bench.vessels[0].deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
    if glucose {
        bench.vessels[0].deposit(SpeciesId::new("glucose"), Moles(0.001), Phase::Aqueous);
    }
    bench
}

fn measure(bench: &mut Bench) -> Vec<Event> {
    bench
        .step(script::parse_op("measure v1 ph").unwrap().unwrap())
        .unwrap()
}

fn localized_blank(locale: Locale, expected: &str) {
    for glucose in [false, true] {
        let mut bench = sample(glucose);
        let before = serde_json::to_value(&bench.vessels).unwrap();
        let events = measure(&mut bench);
        assert_eq!(serde_json::to_value(&bench.vessels).unwrap(), before);
        let event = events
            .iter()
            .find(|e| {
                matches!(
                    e,
                    Event::Measured {
                        instrument: Instrument::PhMeter,
                        ..
                    }
                )
            })
            .expect("admitted blank has its unchanged pH measurement");
        for register in [Register::LV2, Register::LV3] {
            let text = render_event_in(event, register, locale);
            assert!(text.contains(expected), "{text}");
            assert!(
                !text.contains("listed neutral unspeciated solutes"),
                "{text}"
            );
        }
    }
}

#[test]
fn blank_ph_caveat_is_in_the_actual_german_measurement() {
    localized_blank(Locale::parse("de"), "Autoprotolyse");
}

#[test]
fn blank_ph_caveat_is_in_the_actual_french_measurement() {
    localized_blank(Locale::parse("fr"), "autoprotolyse");
}

#[test]
fn english_blank_reading_and_legacy_caveat_are_preserved() {
    for glucose in [false, true] {
        let mut bench = sample(glucose);
        let events = measure(&mut bench);
        let (value, unit, note) = events
            .iter()
            .find_map(|e| match e {
                Event::Measured {
                    instrument: Instrument::PhMeter,
                    value,
                    unit,
                    note,
                    ..
                } => Some((*value, unit, note.as_deref().unwrap())),
                _ => None,
            })
            .unwrap();
        assert_eq!(value, 7.0);
        assert_eq!(unit, "pH");
        assert_eq!(note, "ideal 25 °C water-autoprotolysis baseline; listed neutral unspeciated solutes are assumed to add no acidity or basicity. Dissolved CO₂, activity effects, temperature dependence and trace contamination are outside this blank");
    }
}

#[test]
fn localization_does_not_expand_the_admitted_blank_domain() {
    for scenario in [0, 1, 2] {
        let mut bench = sample(false);
        match scenario {
            0 => bench.vessels[0].temperature = Kelvin(310.0),
            1 => bench.vessels[0].deposit(SpeciesId::new("HClO"), Moles(0.001), Phase::Liquid),
            _ => bench.vessels[0].contents.clear(),
        }
        let events = measure(&mut bench);
        assert!(!events.iter().any(|e| matches!(
            e,
            Event::Measured {
                instrument: Instrument::PhMeter,
                ..
            }
        )));
        assert!(events
            .iter()
            .any(|e| matches!(e, Event::NotYetModeled { .. })));
    }
}
