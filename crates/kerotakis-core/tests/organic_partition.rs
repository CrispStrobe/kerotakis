//! Organic hexane/water partition family (2026-11-10).
//!
//! Review cases MO-004, SE-002 and SE-012: acetic acid, ethyl acetate and
//! naphthalene had no hexane/water distribution coefficient, so extraction
//! moved only ethanol (MO-004) or nothing at all (SE-002/SE-012). The fix is
//! one mechanism: UNIFAC group decompositions in `bench::partition_groups`
//! (shared with the chromatograph), a reviewed dissolved bound for partly
//! miscible liquid owners, and neutral-form-only partitioning for a weak
//! acid. `docs/organic-partition-coverage-20261110.json` is the table.

use kerotakis_core::script::parse_op;
use kerotakis_core::*;

const TABLE: &str = include_str!("../../../docs/organic-partition-coverage-20261110.json");

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

fn split<'a>(events: &'a [Event], key: &str) -> Option<&'a ops::ExtractionSplit> {
    events.iter().find_map(|e| match e {
        Event::Extracted { solutes, .. } => solutes.iter().find(|s| s.species.0 == key),
        _ => None,
    })
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

fn retained<'a>(events: &'a [Event], key: &str) -> Option<&'a ops::RetainedSolute> {
    events.iter().find_map(|e| match e {
        Event::Extracted { retained, .. } => retained.iter().find(|r| r.species.0 == key),
        _ => None,
    })
}

fn has_refusal(events: &[Event], key: &str) -> bool {
    events
        .iter()
        .any(|event| matches!(event, Event::NotYetModeled { reason: Some(p), .. } if p.key == key))
}

fn assert_localised(phrase: &phrase::Phrase) {
    let en = phrase.render(i18n::Locale::EN);
    for locale in ["de", "fr"] {
        let text = phrase.render(i18n::Locale::parse(locale));
        assert_ne!(text, en, "{locale} falls back to English: {text}");
    }
}

fn molar_volume(key: &str) -> f64 {
    species::lookup(&SpeciesId::new(key))
        .and_then(|s| s.molar_volume_l_per_mol())
        .unwrap()
}

#[test]
fn organic_partition_coverage_is_current() {
    let doc: serde_json::Value = serde_json::from_str(TABLE).unwrap();
    assert_eq!(doc["octanol_proxy_used"], false);
    let hexane = SpeciesId::new("hexane");
    let to_concentration = molar_volume("water") / molar_volume("hexane");
    let rows = doc["species"].as_array().unwrap();
    assert!(rows.len() >= 8);
    for row in rows {
        let key = row["species"].as_str().unwrap();
        let id = SpeciesId::new(key);
        assert_eq!(
            bench::extraction_partition_basis(&id, &hexane),
            Some("unifac-group-model"),
            "{key}"
        );
        // The runtime K, read off a real extraction of a trace amount.
        let mut b = setup(&["add v1 water 100mL", "new"]);
        b.vessels[0].deposit(id.clone(), Moles(1e-7), Phase::Aqueous);
        let events = step(&mut b, "extract v1 v2 hexane 20mL");
        let s = split(&events, key).unwrap_or_else(|| panic!("{key}: {events:?}"));
        let expected = row["mole_fraction_ratio_298"].as_f64().unwrap() * to_concentration;
        assert!(
            ((s.partition_k - expected) / expected).abs() < 1e-5,
            "{key}: runtime K {} vs table {expected}",
            s.partition_k
        );
        assert!(
            (s.extracted.0 + s.remaining.0 - 1e-7).abs() < 1e-20,
            "{key}: ledger"
        );
        assert!(s.provenance.contains("Fredenslund"), "{key}");
    }
    // A partly miscible liquid is admitted only alongside a partition.
    for row in apparatus::PARTLY_MISCIBLE_LIQUIDS {
        assert!(
            bench::extraction_partition_basis(&SpeciesId::new(row.species), &hexane).is_some(),
            "{} admitted without a partition",
            row.species
        );
    }
}

#[test]
fn acetic_acid_moves_only_as_its_neutral_form_so_ph_decides_the_share() {
    // Low pH: the acid is essentially all neutral CH3COOH.
    let mut acid = setup(&["add v1 water 100mL", "new"]);
    acid.vessels[0].deposit(SpeciesId::new("CH3COOH"), Moles(0.001), Phase::Aqueous);
    let low = step(&mut acid, "extract v1 v2 hexane 20mL");
    let moved_low = split(&low, "CH3COOH")
        .expect("neutral acid partitions")
        .extracted
        .0;

    // pH about 7.8 (acetate buffer far above pKa 4.76): the same total is
    // ~0.1 % neutral, the rest acetate with its sodium counter-ion.
    let mut base = setup(&["add v1 water 100mL", "new"]);
    for (key, n) in [("CH3COOH", 1e-6), ("CH3COO-", 0.000999), ("Na+", 0.000999)] {
        base.vessels[0].deposit(SpeciesId::new(key), Moles(n), Phase::Aqueous);
    }
    let high = step(&mut base, "extract v1 v2 hexane 20mL");
    let moved_high = split(&high, "CH3COOH")
        .map(|s| s.extracted.0)
        .unwrap_or(0.0);

    assert!(moved_low > 0.0);
    assert!(
        moved_high < moved_low / 100.0,
        "acetate must not ride along: low pH {moved_low}, high pH {moved_high}"
    );
    // The conjugate base and its counter-ion stay in the water, named.
    let source = base.vessel(VesselId(0)).unwrap();
    assert!((source.moles_of(&SpeciesId::new("CH3COO-")).0 - 0.000999).abs() < 1e-18);
    let receiver = base.vessel(VesselId(1)).unwrap();
    assert_eq!(receiver.moles_of(&SpeciesId::new("CH3COO-")).0, 0.0);
    assert_eq!(receiver.moles_of(&SpeciesId::new("Na+")).0, 0.0);
    // 2026-11-11 (SE-002 lane): the ions are no longer a missing
    // coefficient; they are retained by the stated no-ion-transfer rule.
    assert!(retained(&high, "CH3COO-").is_some());
    assert!(retained(&high, "Na+").is_some());
    assert!(!has_refusal(
        &high,
        "not-modeled.extraction-without-coefficient"
    ));
    // Acetic acid prefers water (K < 1): most stays behind even at low pH.
    let s = split(&low, "CH3COOH").unwrap();
    assert!(s.partition_k < 1.0 && s.remaining.0 > s.extracted.0);
}

#[test]
fn ethyl_acetate_beyond_its_dissolved_bound_stays_a_named_liquid_owner() {
    // 0.2 mol (17.6 g) in 100 mL is above the 7 g/100 g bound: a separate
    // ester layer may exist, so it is named, not silently moved.
    let mut b = setup(&["add v1 water 100mL", "add v1 ethanol 0.001mol", "new"]);
    b.vessels[0].deposit(SpeciesId::new("ethyl_acetate"), Moles(0.2), Phase::Liquid);
    let events = step(&mut b, "extract v1 v2 hexane 20mL");
    assert!(split(&events, "ethyl_acetate").is_none());
    assert!(split(&events, "ethanol").is_some());
    let named = refusal(&events, "not-modeled.extraction-without-coefficient");
    assert!(named.render(i18n::Locale::EN).contains("ethyl_acetate"));
    assert_localised(named);
    assert_eq!(
        b.vessel(VesselId(1))
            .unwrap()
            .moles_of(&SpeciesId::new("ethyl_acetate"))
            .0,
        0.0
    );
}

#[test]
fn ethyl_acetate_within_its_bound_partitions_into_hexane() {
    let mut b = setup(&["add v1 water 100mL", "new"]);
    b.vessels[0].deposit(
        SpeciesId::new("ethyl_acetate"),
        Moles(0.000667),
        Phase::Liquid,
    );
    let events = step(&mut b, "extract v1 v2 hexane 20mL");
    let s = split(&events, "ethyl_acetate").expect("dissolved ester is owned");
    assert!(
        s.partition_k > 1.0,
        "the ester prefers hexane: {}",
        s.partition_k
    );
    assert!((s.extracted.0 + s.remaining.0 - 0.000667).abs() < 1e-15);
    assert!(!events
        .iter()
        .any(|e| matches!(e, Event::NotYetModeled { .. })));
}

#[test]
fn naphthalene_leaves_the_salt_behind() {
    // SE-002 shape, with the aqueous state written down directly.
    let mut b = setup(&["add v1 water 100mL", "new"]);
    for (key, n) in [("naphthalene", 5e-6), ("Na+", 0.001), ("Cl-", 0.001)] {
        b.vessels[0].deposit(SpeciesId::new(key), Moles(n), Phase::Aqueous);
    }
    let events = step(&mut b, "extract v1 v2 hexane 20mL");
    let s = split(&events, "naphthalene").expect("naphthalene partitions");
    assert!(s.partition_k > 100.0);
    assert!(s.extracted.0 > 0.99 * 5e-6);
    let receiver = b.vessel(VesselId(1)).unwrap();
    assert_eq!(receiver.moles_of(&SpeciesId::new("Na+")).0, 0.0);
    assert_eq!(receiver.moles_of(&SpeciesId::new("Cl-")).0, 0.0);
    // SE-002 (2026-11-11): the salt staying aqueous is a stated rule in
    // the extraction event, not a missing coefficient.
    for ion in ["Na+", "Cl-"] {
        let kept = retained(&events, ion).unwrap_or_else(|| panic!("{ion} retained: {events:?}"));
        assert!((kept.moles.0 - 0.001).abs() < 1e-15);
        assert!(kept.rule.contains("ion-pairing agent"));
        assert!(kept.provenance.contains("Rydberg"));
    }
    assert!(!has_refusal(
        &events,
        "not-modeled.extraction-without-coefficient"
    ));
    let source = b.vessel(VesselId(0)).unwrap();
    assert!((source.moles_of(&SpeciesId::new("Na+")).0 - 0.001).abs() < 1e-18);
}

#[test]
fn ions_alone_against_hexane_extract_nothing_and_say_why() {
    // Only dissolved ions: the extraction runs (fresh hexane is dispensed),
    // nothing moves, and every ion is reported retained by the rule.
    let mut b = setup(&["add v1 water 100mL", "new"]);
    for (key, n) in [("K+", 0.001), ("SO4-2", 0.0005)] {
        b.vessels[0].deposit(SpeciesId::new(key), Moles(n), Phase::Aqueous);
    }
    let events = step(&mut b, "extract v1 v2 hexane 20mL");
    assert_eq!(retained(&events, "K+").unwrap().class, "monatomic cation");
    assert_eq!(
        retained(&events, "SO4-2").unwrap().class,
        "polyatomic inorganic ion"
    );
    assert!(!events
        .iter()
        .any(|e| matches!(e, Event::NotYetModeled { .. })));
    let receiver = b.vessel(VesselId(1)).unwrap();
    assert!(receiver.moles_of(&SpeciesId::new("hexane")).0 > 0.0);
    assert_eq!(receiver.moles_of(&SpeciesId::new("K+")).0, 0.0);
    let rendered =
        render::render_events_in(&events, render::Register::LV3, i18n::Locale::EN).join("\n");
    assert!(rendered.contains("stay in the water"), "{rendered}");
}

#[test]
fn ions_against_a_polar_solvent_are_still_a_typed_refusal() {
    // The rule is about hydrocarbons only: ethyl acetate is not one, so
    // Na+/Cl- keep the missing-coefficient refusal there.
    let mut b = setup(&["add v1 water 100mL", "new"]);
    for (key, n) in [("Na+", 0.001), ("Cl-", 0.001)] {
        b.vessels[0].deposit(SpeciesId::new(key), Moles(n), Phase::Aqueous);
    }
    let events = step(&mut b, "extract v1 v2 ethyl_acetate 20mL");
    assert!(retained(&events, "Na+").is_none());
    assert!(retained(&events, "Cl-").is_none());
}

#[test]
fn se012_recovery_depends_on_phase_ratio_not_scale() {
    let recovery = |water: &str, n: f64, hexane: &str| {
        let mut b = setup(&[&format!("add v1 water {water}"), "new"]);
        b.vessels[0].deposit(SpeciesId::new("naphthalene"), Moles(n), Phase::Aqueous);
        let events = step(&mut b, &format!("extract v1 v2 hexane {hexane}"));
        split(&events, "naphthalene")
            .unwrap()
            .single_stage_efficiency
    };
    let small = recovery("50mL", 2e-6, "10mL");
    let large = recovery("100mL", 4e-6, "20mL");
    assert!((small - large).abs() < 1e-3, "{small} vs {large}");
}

#[test]
fn an_unknown_neutral_solute_is_still_a_typed_refusal_beside_a_moved_one() {
    let mut b = setup(&["add v1 water 100mL", "new"]);
    b.vessels[0].deposit(SpeciesId::new("naphthalene"), Moles(5e-6), Phase::Aqueous);
    b.vessels[0].deposit(SpeciesId::new("glucose"), Moles(1e-4), Phase::Aqueous);
    let events = step(&mut b, "extract v1 v2 hexane 20mL");
    assert!(split(&events, "naphthalene").is_some());
    assert!(split(&events, "glucose").is_none());
    let named = refusal(&events, "not-modeled.extraction-without-coefficient");
    assert!(named.render(i18n::Locale::EN).contains("glucose"));
    assert_localised(named);
}

#[test]
fn chromatography_of_the_hexane_layer_is_a_localised_not_modelled_reason() {
    // MO-004's second measurement: the receiver holds hexane and no water.
    let mut b = setup(&["add v1 water 100mL", "new"]);
    b.vessels[0].deposit(
        SpeciesId::new("ethyl_acetate"),
        Moles(0.000667),
        Phase::Liquid,
    );
    step(&mut b, "extract v1 v2 hexane 20mL");
    let events = step(&mut b, "chromatograph v2");
    let reason = refusal(
        &events,
        "not-modeled.chromatography-needs-an-aqueous-sample",
    );
    assert_localised(reason);
    // The aqueous raffinate still chromatographs, now with an ester peak.
    let events = step(&mut b, "chromatograph v1");
    let peaks = events
        .iter()
        .find_map(|e| match e {
            Event::Chromatographed { peaks, .. } => Some(peaks),
            _ => None,
        })
        .expect("aqueous raffinate has a chromatogram");
    assert!(peaks.iter().any(|p| p.species.0 == "ethyl_acetate"));
}
