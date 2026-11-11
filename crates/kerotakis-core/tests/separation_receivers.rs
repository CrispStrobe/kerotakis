//! Separation verbs never leave a receiver silently empty (2026-11-09).
//!
//! Review cases MO-004, SE-002, SE-012, SE-007, SP-003 and SP-011 ended with
//! empty receivers. Each step must either move material with a closed
//! ledger or emit a typed not-modelled reason naming what is missing, and a
//! refused step changes neither vessel. MO-004 was also a real ownership
//! defect: liquid-labelled ethanol (the script path's label for ethanol in
//! water) and the ester were skipped by `extract` without a word.

use kerotakis_core::script::parse_op;
use kerotakis_core::*;

const TABLE: &str = include_str!("../../../docs/separation-coverage-20261109.json");

fn setup(lines: &[&str]) -> Bench {
    let mut bench = Bench::new();
    for line in lines {
        bench
            .step(parse_op(line).unwrap().expect("operator"))
            .unwrap();
    }
    bench
}

fn step(bench: &mut Bench, line: &str) -> Vec<Event> {
    bench
        .step(parse_op(line).unwrap().expect("operator"))
        .unwrap()
}

fn total(bench: &Bench, key: &str) -> f64 {
    let id = SpeciesId::new(key);
    bench.vessels.iter().map(|v| v.moles_of(&id).0).sum()
}

fn refusal<'a>(events: &'a [Event], key: &str) -> &'a phrase::Phrase {
    events
        .iter()
        .find_map(|event| match event {
            Event::NotYetModeled {
                reason: Some(p), ..
            } if p.key == key => Some(p),
            _ => None,
        })
        .unwrap_or_else(|| panic!("expected typed refusal {key}: {events:?}"))
}

fn assert_localised(phrase: &phrase::Phrase) {
    let en = phrase.render(i18n::Locale::EN);
    for locale in ["de", "fr"] {
        let text = phrase.render(i18n::Locale::parse(locale));
        assert_ne!(text, en, "{locale} falls back to English: {text}");
    }
}

/// The generic contract: a moved step has a typed result and a non-empty
/// receiver; a refused step has a typed reason and no vessel changed.
fn assert_moves_or_refuses(before: &serde_json::Value, bench: &Bench, events: &[Event]) {
    let moved = events
        .iter()
        .any(|e| matches!(e, Event::Extracted { .. } | Event::Distilled { .. }));
    if moved {
        return;
    }
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::NotYetModeled { .. })),
        "a separation that moved nothing must say why: {events:?}"
    );
    assert_eq!(&serde_json::to_value(&bench.vessels).unwrap(), before);
}

#[test]
fn separation_coverage_is_current() {
    let doc: serde_json::Value = serde_json::from_str(TABLE).unwrap();
    let hexane = SpeciesId::new("hexane");
    let mut extract = std::collections::BTreeMap::new();
    let mut distil = std::collections::BTreeMap::new();
    let mut miscible = std::collections::BTreeSet::new();
    for row in doc["species"].as_array().unwrap() {
        let key = row["species"].as_str().unwrap().to_string();
        if let Some(basis) = row["extract_hexane"]["basis"].as_str() {
            extract.insert(key.clone(), basis.to_string());
        }
        if let Some(basis) = row["distil"]["basis"].as_str() {
            distil.insert(key.clone(), basis.to_string());
        }
        if row["liquid_owner_admitted"].as_bool().unwrap() {
            miscible.insert(key);
        }
    }
    for data in species::REGISTRY {
        let id = SpeciesId::new(data.key);
        assert_eq!(
            bench::extraction_partition_basis(&id, &hexane),
            extract.get(data.key).map(String::as_str),
            "{}: extract basis differs from docs/separation-coverage-20261109.json",
            data.key
        );
        assert_eq!(
            volatility::still_property_basis(data.key),
            distil.get(data.key).map(String::as_str),
            "{}: distil basis differs from the generated table",
            data.key
        );
        assert_eq!(
            apparatus::is_water_miscible_liquid(&id),
            miscible.contains(data.key),
            "{}: liquid-owner admission differs from the generated table",
            data.key
        );
    }
    // Every admitted liquid owner must also have a partition, or admitting
    // it would only rename an unsupported solute.
    for key in &miscible {
        assert!(
            extract.contains_key(key),
            "{key} admitted without a partition"
        );
    }
}

#[test]
fn liquid_labelled_ethanol_and_the_dissolved_ester_are_both_extracted() {
    // MO-004 shape: ethanol added by script is a liquid portion in water.
    let mut b = setup(&["add v1 water 100mL", "add v1 ethanol 0.001mol", "new"]);
    assert!(b.vessels[0]
        .contents
        .iter()
        .any(|p| p.species.0 == "ethanol" && p.phase == Phase::Liquid));
    b.vessels[0].deposit(
        SpeciesId::new("ethyl_acetate"),
        Moles(0.0005),
        Phase::Liquid,
    );
    let before = serde_json::to_value(&b.vessels).unwrap();
    let events = step(&mut b, "extract v1 v2 hexane 20mL");
    assert_moves_or_refuses(&before, &b, &events);
    let split = events
        .iter()
        .find_map(|e| match e {
            Event::Extracted { solutes, .. } => {
                solutes.iter().find(|s| s.species.0 == "ethanol").cloned()
            }
            _ => None,
        })
        .expect("ethanol is owned dissolved solute, not silently skipped");
    assert!(split.extracted.0 > 0.0 && split.remaining.0 > 0.0);
    assert!((split.extracted.0 + split.remaining.0 - 0.001).abs() < 1e-15);
    // Closed ledger: every species total is unchanged across both vessels,
    // and the receiver holds the dispensed hexane and the extract.
    assert!((total(&b, "ethanol") - 0.001).abs() < 1e-15);
    assert!((total(&b, "ethyl_acetate") - 0.0005).abs() < 1e-15);
    let receiver = b.vessel(VesselId(1)).unwrap();
    assert!(receiver.moles_of(&SpeciesId::new("hexane")).0 > 0.0);
    assert!((receiver.moles_of(&SpeciesId::new("ethanol")).0 - split.extracted.0).abs() < 1e-18);
    // Until 2026-11-10 the ester was named as unsupported here. It now has a
    // UNIFAC decomposition, and 0.0005 mol in 100 mL is far inside its
    // reviewed dissolved bound, so it is one owned solute and moves too.
    let ester = events
        .iter()
        .find_map(|e| match e {
            Event::Extracted { solutes, .. } => solutes
                .iter()
                .find(|s| s.species.0 == "ethyl_acetate")
                .cloned(),
            _ => None,
        })
        .expect("the dissolved ester is extracted, not skipped");
    // Both phases own part of it. The split direction follows the UNIFAC
    // estimate and the phase volumes; no reviewed hexane/water K pins it here.
    assert!(ester.extracted.0 > 0.0 && ester.remaining.0 > 0.0);
    assert!((ester.extracted.0 + ester.remaining.0 - 0.0005).abs() < 1e-15);
    assert!(
        (receiver.moles_of(&SpeciesId::new("ethyl_acetate")).0 - ester.extracted.0).abs() < 1e-18
    );
    assert!(!events
        .iter()
        .any(|e| matches!(e, Event::NotYetModeled { .. })));
}

#[test]
fn an_unparameterised_neutral_solute_refuses_naming_it_and_dispenses_nothing() {
    // SE-012 / SE-002 shape. Naphthalene was the fixture until it gained a
    // UNIFAC decomposition on 2026-11-10 (now covered positively in
    // organic_partition.rs); glucose is a dissolved neutral with none.
    let mut b = setup(&["add v1 water 50mL", "new"]);
    b.vessels[0].deposit(SpeciesId::new("glucose"), Moles(0.000002), Phase::Aqueous);
    let before = serde_json::to_value(&b.vessels).unwrap();
    let events = step(&mut b, "extract v1 v2 hexane 10mL");
    assert_moves_or_refuses(&before, &b, &events);
    let reason = refusal(&events, "not-modeled.no-distribution-coefficient");
    let text = reason.render(i18n::Locale::EN);
    assert!(text.contains("glucose"), "{text}");
    assert!(text.contains("receiver is unchanged"), "{text}");
    assert_localised(reason);
    assert_eq!(total(&b, "hexane"), 0.0);
}

#[test]
fn ions_alone_are_extracted_as_nothing_and_kept_in_the_water() {
    // SP-011 reduced arm (2026-11-11): after thiosulfate reduces the iodine
    // only ions remain. The extraction proceeds: hexane is dispensed into the
    // receiver, every ion stays in the water and is named, nothing moves.
    let mut b = setup(&["add v1 water 100mL", "new"]);
    for (key, n) in [
        ("I-", 0.0012),
        ("K+", 0.001),
        ("Na+", 0.0004),
        ("S4O6-2", 0.0001),
    ] {
        b.vessels[0].deposit(SpeciesId::new(key), Moles(n), Phase::Aqueous);
    }
    let events = step(&mut b, "extract v1 v2 hexane 0.1mol stages 1");
    let (solutes, retained) = events
        .iter()
        .find_map(|e| match e {
            Event::Extracted {
                solutes, retained, ..
            } => Some((solutes, retained)),
            _ => None,
        })
        .expect("an ion-only feed is still extracted");
    assert!(solutes.is_empty(), "no ion partitions: {solutes:?}");
    // Every ion is reported retained by the stated ion rule, not refused.
    for ion in ["I-", "K+", "Na+", "S4O6-2"] {
        assert!(
            retained.iter().any(|r| r.species.0 == ion),
            "{ion} not reported retained: {retained:?}"
        );
    }
    assert!(!events
        .iter()
        .any(|e| matches!(e, Event::NotYetModeled { .. })));
    let receiver = b.vessel(VesselId(1)).unwrap();
    assert!((receiver.moles_of(&SpeciesId::new("hexane")).0 - 0.1).abs() < 1e-12);
    for (key, n) in [
        ("I-", 0.0012),
        ("K+", 0.001),
        ("Na+", 0.0004),
        ("S4O6-2", 0.0001),
    ] {
        let id = SpeciesId::new(key);
        assert_eq!(receiver.moles_of(&id).0, 0.0, "{key} crossed into hexane");
        assert!((b.vessel(VesselId(0)).unwrap().moles_of(&id).0 - n).abs() < 1e-15);
    }
}

#[test]
fn ions_beside_an_unparameterised_neutral_still_refuse_and_dispense_nothing() {
    // Negative control: the ion allowance never covers a neutral solute
    // that has no coefficient.
    let mut b = setup(&["add v1 water 100mL", "new"]);
    for (key, n) in [("K+", 0.001), ("I-", 0.001), ("glucose", 1e-4)] {
        b.vessels[0].deposit(SpeciesId::new(key), Moles(n), Phase::Aqueous);
    }
    let before = serde_json::to_value(&b.vessels).unwrap();
    let events = step(&mut b, "extract v1 v2 hexane 0.1mol stages 1");
    assert_moves_or_refuses(&before, &b, &events);
    let text = refusal(&events, "not-modeled.no-distribution-coefficient").render(i18n::Locale::EN);
    assert!(text.contains("glucose"), "{text}");
    assert_eq!(total(&b, "hexane"), 0.0);
}

#[test]
fn a_volatile_without_still_properties_refuses_naming_it() {
    // SE-007 acetic-acid arm.
    let mut b = setup(&["add v1 water 100mL", "add v1 CH3COOH 0.005mol", "new"]);
    let before = serde_json::to_value(&b.vessels).unwrap();
    let events = step(&mut b, "distil v1 v2 0.2 stages 3");
    assert_moves_or_refuses(&before, &b, &events);
    let reason = refusal(&events, "not-modeled.no-reviewed-still-properties");
    assert!(reason.render(i18n::Locale::EN).contains("CH3COOH"));
    assert_localised(reason);
}

#[test]
fn a_staged_cut_either_closes_its_ledger_or_names_the_precision_limit() {
    // SP-003 shape: 4 stages enrich ethanol so strongly that a 10 % cut asks
    // for more ethanol than the pot holds; the stripped residue falls below
    // the distillate's resolution. A smaller cut at the same stage count is
    // representable and must close its ledger.
    for (fraction, must_move) in [("0.1", false), ("0.01", true)] {
        let mut b = setup(&["add v1 water 50g", "add v1 ethanol 5g", "new"]);
        let water0 = total(&b, "water");
        let ethanol0 = total(&b, "ethanol");
        let before = serde_json::to_value(&b.vessels).unwrap();
        let events = step(&mut b, &format!("distil v1 v2 {fraction} stages 4"));
        assert_moves_or_refuses(&before, &b, &events);
        let moved = events.iter().any(|e| matches!(e, Event::Distilled { .. }));
        if must_move {
            assert!(moved, "{events:?}");
        }
        if moved {
            assert!((total(&b, "water") - water0).abs() < 1e-12);
            assert!((total(&b, "ethanol") - ethanol0).abs() < 1e-12);
            assert!(
                b.vessel(VesselId(1))
                    .unwrap()
                    .moles_of(&SpeciesId::new("ethanol"))
                    .0
                    > 0.0
            );
        } else {
            let reason = refusal(&events, "not-modeled.still-condensate-precision");
            assert!(reason.render(i18n::Locale::EN).contains("fewer stages"));
            assert_localised(reason);
        }
    }
}
