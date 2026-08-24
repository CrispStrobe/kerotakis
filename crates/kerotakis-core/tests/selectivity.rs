//! EXP-50: Mechanistic selectivity rules — the classic condition matrix
//! reproduces textbook outcomes; changing one condition flips the product
//! and the boundary line says which rule fired.

use kerotakis_core::*;

fn add(bench: &mut Bench, key: &str, moles: f64) {
    bench
        .step(Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new(key),
            moles: Moles(moles),
            at: None,
        })
        .expect("add");
}

fn set_temperature(bench: &mut Bench, kelvin: f64) {
    bench.vessels[0].temperature = Kelvin(kelvin);
}

fn react_haloalkane(bench: &mut Bench) -> Vec<Event> {
    bench
        .step(
            script::parse_op("react v1 haloalkane")
                .expect("grammar")
                .expect("an operator"),
        )
        .expect("react")
}

fn moles_of(bench: &Bench, key: &str) -> f64 {
    bench
        .vessel(VesselId(0))
        .unwrap()
        .moles_of(&SpeciesId::new(key))
        .0
}

fn org_reacted_name(events: &[Event]) -> String {
    events
        .iter()
        .find_map(|e| match e {
            Event::OrgReacted { name, .. } => Some(name.clone()),
            _ => None,
        })
        .expect("an OrgReacted event")
}

fn org_reacted_boundary(events: &[Event]) -> String {
    events
        .iter()
        .find_map(|e| match e {
            Event::OrgReacted { boundary, .. } => Some(boundary.clone()),
            _ => None,
        })
        .expect("a boundary line")
}

fn mass_g(bench: &Bench) -> f64 {
    bench
        .vessel(VesselId(0))
        .unwrap()
        .contents
        .iter()
        .filter_map(|p| species::lookup(&p.species).map(|d| p.moles.0 * d.molar_mass))
        .sum()
}

// ── Registry ────────────────────────────────────────────────────────

#[test]
fn species_in_registry() {
    for key in [
        "bromoethane",
        "tert_butyl_bromide",
        "NaBr",
        "ethene",
        "tert_butanol",
        "isobutylene",
        "HBr",
    ] {
        assert!(
            species::lookup(&SpeciesId::new(key)).is_some(),
            "{key} not in registry"
        );
    }
}

// ── SN2: primary substrate + strong nucleophile at ambient ──────────

#[test]
fn sn2_primary_hydroxide_ambient() {
    let mut bench = Bench::new();
    add(&mut bench, "bromoethane", 0.01);
    add(&mut bench, "NaOH", 0.01);
    let events = react_haloalkane(&mut bench);
    let name = org_reacted_name(&events);
    assert_eq!(name, "haloalkane:SN2", "mechanism: {name}");
    assert!(
        (moles_of(&bench, "ethanol") - 0.01).abs() < 1e-12,
        "substitution product"
    );
    assert!(
        (moles_of(&bench, "NaBr") - 0.01).abs() < 1e-12,
        "leaving group salt"
    );
    assert!(moles_of(&bench, "bromoethane") < 1e-12, "substrate spent");
    let boundary = org_reacted_boundary(&events);
    assert!(boundary.contains("SN2"), "lv3 line says SN2");
    assert!(boundary.contains("March"), "provenance cites March");
}

// ── E2: same primary substrate + strong nucleophile, but HOT ────────

#[test]
fn e2_primary_hydroxide_hot() {
    let mut bench = Bench::new();
    add(&mut bench, "bromoethane", 0.01);
    add(&mut bench, "NaOH", 0.01);
    set_temperature(&mut bench, 373.15); // 100 °C
    let events = react_haloalkane(&mut bench);
    let name = org_reacted_name(&events);
    assert_eq!(name, "haloalkane:E2", "mechanism: {name}");
    assert!(moles_of(&bench, "ethanol") < 1e-12, "NO substitution");
    assert!(
        events.iter().any(
            |e| matches!(e, Event::GasEvolved { species, .. } | Event::GasContained { species, .. }
                if species == &SpeciesId::new("ethene"))
        ),
        "ethene gas produced"
    );
    let boundary = org_reacted_boundary(&events);
    assert!(boundary.contains("E2"), "lv3 line says E2");
}

#[test]
fn condition_flip_primary_temperature_changes_mechanism() {
    let mut b_cold = Bench::new();
    add(&mut b_cold, "bromoethane", 0.01);
    add(&mut b_cold, "NaOH", 0.01);
    let cold_name = org_reacted_name(&react_haloalkane(&mut b_cold));

    let mut b_hot = Bench::new();
    add(&mut b_hot, "bromoethane", 0.01);
    add(&mut b_hot, "NaOH", 0.01);
    set_temperature(&mut b_hot, 373.15);
    let hot_name = org_reacted_name(&react_haloalkane(&mut b_hot));

    assert_ne!(
        cold_name, hot_name,
        "heating flips mechanism: cold={cold_name} hot={hot_name}"
    );
    assert_eq!(cold_name, "haloalkane:SN2");
    assert_eq!(hot_name, "haloalkane:E2");
}

// ── SN1: tertiary substrate + weak nucleophile at ambient ───────────

#[test]
fn sn1_tertiary_water_ambient() {
    let mut bench = Bench::new();
    add(&mut bench, "tert_butyl_bromide", 0.01);
    add(&mut bench, "water", 5.0);
    let events = react_haloalkane(&mut bench);
    let name = org_reacted_name(&events);
    assert_eq!(name, "haloalkane:SN1", "mechanism: {name}");
    assert!(
        (moles_of(&bench, "tert_butanol") - 0.01).abs() < 1e-12,
        "substitution product"
    );
    assert!(
        (moles_of(&bench, "HBr") - 0.01).abs() < 1e-12,
        "HBr produced"
    );
    let boundary = org_reacted_boundary(&events);
    assert!(boundary.contains("SN1"), "lv3 line says SN1");
}

// ── E1: same tertiary substrate + weak nucleophile, but HOT ─────────

#[test]
fn e1_tertiary_water_hot() {
    let mut bench = Bench::new();
    add(&mut bench, "tert_butyl_bromide", 0.01);
    add(&mut bench, "water", 5.0);
    set_temperature(&mut bench, 373.15);
    let events = react_haloalkane(&mut bench);
    let name = org_reacted_name(&events);
    assert_eq!(name, "haloalkane:E1", "mechanism: {name}");
    assert!(
        moles_of(&bench, "tert_butanol") < 1e-12,
        "NO substitution at high temp"
    );
    assert!(
        events.iter().any(
            |e| matches!(e, Event::GasEvolved { species, .. } | Event::GasContained { species, .. }
                if species == &SpeciesId::new("isobutylene"))
        ),
        "isobutylene gas produced"
    );
    let boundary = org_reacted_boundary(&events);
    assert!(boundary.contains("E1"), "lv3 line says E1");
}

#[test]
fn condition_flip_tertiary_temperature_changes_mechanism() {
    let mut b_cold = Bench::new();
    add(&mut b_cold, "tert_butyl_bromide", 0.01);
    add(&mut b_cold, "water", 5.0);
    let cold_name = org_reacted_name(&react_haloalkane(&mut b_cold));

    let mut b_hot = Bench::new();
    add(&mut b_hot, "tert_butyl_bromide", 0.01);
    add(&mut b_hot, "water", 5.0);
    set_temperature(&mut b_hot, 373.15);
    let hot_name = org_reacted_name(&react_haloalkane(&mut b_hot));

    assert_ne!(
        cold_name, hot_name,
        "heating flips mechanism: cold={cold_name} hot={hot_name}"
    );
    assert_eq!(cold_name, "haloalkane:SN1");
    assert_eq!(hot_name, "haloalkane:E1");
}

// ── E2: tertiary substrate + strong nucleophile → E2 (not SN2) ──────

#[test]
fn e2_tertiary_hydroxide_ambient() {
    let mut bench = Bench::new();
    add(&mut bench, "tert_butyl_bromide", 0.01);
    add(&mut bench, "NaOH", 0.01);
    let events = react_haloalkane(&mut bench);
    let name = org_reacted_name(&events);
    assert_eq!(
        name, "haloalkane:E2",
        "tertiary + strong base → E2 (SN2 blocked by steric hindrance)"
    );
    assert!(
        events.iter().any(
            |e| matches!(e, Event::GasEvolved { species, .. } | Event::GasContained { species, .. }
                if species == &SpeciesId::new("isobutylene"))
        ),
        "isobutylene gas produced"
    );
    let boundary = org_reacted_boundary(&events);
    assert!(
        boundary.contains("steric"),
        "boundary explains steric hindrance"
    );
}

#[test]
fn condition_flip_substrate_class_changes_mechanism() {
    let mut b_primary = Bench::new();
    add(&mut b_primary, "bromoethane", 0.01);
    add(&mut b_primary, "NaOH", 0.01);
    let primary_name = org_reacted_name(&react_haloalkane(&mut b_primary));

    let mut b_tertiary = Bench::new();
    add(&mut b_tertiary, "tert_butyl_bromide", 0.01);
    add(&mut b_tertiary, "NaOH", 0.01);
    let tertiary_name = org_reacted_name(&react_haloalkane(&mut b_tertiary));

    assert_ne!(
        primary_name, tertiary_name,
        "substrate class flips mechanism: primary={primary_name} tertiary={tertiary_name}"
    );
    assert_eq!(primary_name, "haloalkane:SN2");
    assert_eq!(tertiary_name, "haloalkane:E2");
}

// ── Conservation ────────────────────────────────────────────────────

#[test]
fn sn2_conserves_mass() {
    let mut bench = Bench::new();
    add(&mut bench, "bromoethane", 0.05);
    add(&mut bench, "NaOH", 0.05);
    let before = mass_g(&bench);
    react_haloalkane(&mut bench);
    let after = mass_g(&bench);
    assert!(
        (after - before).abs() < 1e-6,
        "mass conserved: {before} g → {after} g"
    );
}

#[test]
fn sn1_conserves_mass() {
    let mut bench = Bench::new();
    add(&mut bench, "tert_butyl_bromide", 0.05);
    add(&mut bench, "water", 5.0);
    let before = mass_g(&bench);
    react_haloalkane(&mut bench);
    let after = mass_g(&bench);
    assert!(
        (after - before).abs() < 1e-6,
        "mass conserved: {before} g → {after} g"
    );
}

// ── Refusal ─────────────────────────────────────────────────────────

#[test]
fn refuses_when_no_substrate() {
    let mut bench = Bench::new();
    add(&mut bench, "NaOH", 0.01);
    add(&mut bench, "water", 5.0);
    let events = react_haloalkane(&mut bench);
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::NotYetModeled { what, .. } if what.contains("substrate")
        )),
        "refuses out loud naming the missing substrate"
    );
    assert!(
        !events.iter().any(|e| matches!(e, Event::OrgReacted { .. })),
        "no reaction occurs"
    );
}

#[test]
fn refuses_when_no_nucleophile() {
    let mut bench = Bench::new();
    add(&mut bench, "bromoethane", 0.01);
    let events = react_haloalkane(&mut bench);
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::NotYetModeled { what, .. } if what.contains("nucleophile")
        )),
        "refuses out loud naming the missing nucleophile"
    );
}

// ── Parse ───────────────────────────────────────────────────────────

#[test]
fn haloalkane_parses() {
    assert!(script::parse_op("react v1 haloalkane")
        .expect("grammar")
        .is_some());
}

#[test]
fn unknown_reaction_still_lists_haloalkane() {
    let err = script::parse_op("react v1 transmutation").unwrap_err();
    assert!(
        err.contains("haloalkane"),
        "parse error lists haloalkane verb, got: {err}"
    );
}
