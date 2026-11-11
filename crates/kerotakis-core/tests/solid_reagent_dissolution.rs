//! Solid organic reagents go through the dissolution route (2026-11-11,
//! D3-AN-004).
//!
//! Ascorbic acid added to water as a solid was said to be "in contact with
//! liquid: no wired solver models this dissolution/reaction" and then
//! reacted with iodine straight out of the solid. Now: a registry organic
//! solid with a cited aqueous solubility dissolves like any other solid;
//! one without is refused with a typed reason; a curated row that still
//! draws on an undissolved organic solid in water is labelled solid-phase.
//! `docs/organic-solid-dissolution-coverage-20261111.json` is the table.

use kerotakis_core::*;

const TABLE: &str = include_str!("../../../docs/organic-solid-dissolution-coverage-20261111.json");

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(CuratedEquilibrator),
        Box::new(kerotakis_core::nonaqueous::NonAqueousEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn add(bench: &mut Bench, stack: &mut SolverStack, key: &str, moles: f64) -> Vec<Event> {
    bench
        .step_with(
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new(key),
                moles: Moles(moles),
                at: None,
            },
            stack,
            &PermissiveScreen,
        )
        .expect("step")
}

fn phase_moles(bench: &Bench, key: &str, phase: Phase) -> f64 {
    bench
        .vessel(VesselId(0))
        .unwrap()
        .contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}

fn reason_keys(events: &[Event]) -> Vec<String> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::NotYetModeled {
                reason: Some(p), ..
            } => Some(p.key.to_string()),
            _ => None,
        })
        .collect()
}

#[test]
fn ascorbic_acid_dissolves_before_it_reacts_with_iodine() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 2.2137);
    let added = add(&mut bench, &mut s, "ascorbic_acid", 3e-5);
    assert!(
        (phase_moles(&bench, "ascorbic_acid", Phase::Aqueous) - 3e-5).abs() < 1e-15,
        "{added:?}"
    );
    assert_eq!(phase_moles(&bench, "ascorbic_acid", Phase::Solid), 0.0);
    let keys = reason_keys(&added);
    assert!(
        !keys
            .iter()
            .any(|k| k == "not-modeled.no-dissolution-solver"),
        "{keys:?}"
    );
    assert!(
        !keys
            .iter()
            .any(|k| k == "not-modeled.organic-solid-no-solubility"),
        "{keys:?}"
    );

    let reacted = add(&mut bench, &mut s, "I2", 2e-5);
    assert!(reacted
        .iter()
        .any(|e| matches!(e, Event::ReactionOccurred { equation, .. } if equation.contains("I₂"))));
    // It was dissolved, so the reaction is a solution step, not a solid one.
    assert!(!reason_keys(&reacted)
        .iter()
        .any(|k| k == "not-modeled.solid-phase-reaction"));
    assert_eq!(phase_moles(&bench, "ascorbic_acid", Phase::Solid), 0.0);
}

#[test]
fn ascorbic_acid_beyond_its_reviewed_solubility_stays_partly_solid() {
    // About 29 g per 100 mL at 20 °C: 1 mol (176 g) in ~40 mL cannot all go.
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 2.2137);
    add(&mut bench, &mut s, "ascorbic_acid", 1.0);
    let dissolved = phase_moles(&bench, "ascorbic_acid", Phase::Aqueous);
    let solid = phase_moles(&bench, "ascorbic_acid", Phase::Solid);
    assert!(dissolved > 0.0 && solid > 0.0, "{dissolved} / {solid}");
    assert!((dissolved + solid - 1.0).abs() < 1e-12);
}

#[test]
fn an_organic_solid_without_a_reviewed_solubility_is_a_typed_refusal() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 2.2137);
    let events = add(&mut bench, &mut s, "maltose", 1e-4);
    let phrase = events
        .iter()
        .find_map(|event| match event {
            Event::NotYetModeled {
                reason: Some(p), ..
            } if p.key == "not-modeled.organic-solid-no-solubility" => Some(p),
            _ => None,
        })
        .unwrap_or_else(|| panic!("typed refusal: {events:?}"));
    let en = phrase.render(i18n::Locale::EN);
    for locale in ["de", "fr"] {
        assert_ne!(phrase.render(i18n::Locale::parse(locale)), en);
    }
    assert_eq!(phase_moles(&bench, "maltose", Phase::Aqueous), 0.0);
}

#[test]
fn the_solid_phase_label_applies_only_to_undissolvable_organic_solids_in_water() {
    let mut wet = Vessel::new(VesselId(0), "wet");
    wet.deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
    assert!(solve::needs_solid_phase_label(
        &wet,
        &SpeciesId::new("maltose")
    ));
    assert!(!solve::needs_solid_phase_label(
        &wet,
        &SpeciesId::new("ascorbic_acid")
    ));
    assert!(!solve::needs_solid_phase_label(
        &wet,
        &SpeciesId::new("NaHCO3")
    ));
    let dry = Vessel::new(VesselId(1), "dry");
    assert!(!solve::needs_solid_phase_label(
        &dry,
        &SpeciesId::new("maltose")
    ));
    let label = solve::solid_phase_reaction_label(&SpeciesId::new("maltose"), "X → Y");
    assert!(label.render(i18n::Locale::EN).contains("solid-phase"));
    assert_ne!(
        label.render(i18n::Locale::parse("de")),
        label.render(i18n::Locale::EN)
    );
}

#[test]
fn the_coverage_table_names_every_registry_organic_solid() {
    let table: serde_json::Value = serde_json::from_str(TABLE).unwrap();
    let rows = table["rows"].as_array().unwrap();
    for data in species::registry() {
        if data.standard_phase != Phase::Solid
            || !organic_solid_solubility::is_organic_molecular(data)
        {
            continue;
        }
        let row = rows
            .iter()
            .find(|row| row["species"] == data.key)
            .unwrap_or_else(|| panic!("{} missing from the coverage table", data.key));
        let has = data.aqueous_solubility_at(298.15).is_some();
        assert_eq!(
            row["dissolves_by_route"].as_bool(),
            Some(has),
            "{}",
            data.key
        );
    }
}
