//! Lane 3 (#836 generalised): every curated row that names a literal strong
//! acid/base reagent, and the certified reserve that also serves it.
//!
//! The coverage table is ENUMERATED from `curated::REACTIONS` and
//! `curated::ORG_REACTIONS` and compared with the checked-in fixture
//! (`fixtures/strong-reserve-coverage-20261027.json`, rendered in
//! `docs/reserve-coverage-20261027.md`), so a new literal-reagent row fails
//! here until its reserve route is reviewed.
use kerotakis_core::curated::{CuratedEquilibrator, ORG_REACTIONS, REACTIONS};
use kerotakis_core::solve::Equilibrator;
use kerotakis_core::strong_reserve;
use kerotakis_core::*;

const LITERAL: &[&str] = &[
    "HCl", "HBr", "HI", "HNO3", "H2SO4", "NaOH", "KOH", "Ba(OH)2",
];

fn gases(products: &[(&str, f64, Phase)]) -> Vec<String> {
    let mut g: Vec<String> = products
        .iter()
        .filter(|(_, _, p)| *p == Phase::Gas)
        .map(|(k, _, _)| k.to_string())
        .collect();
    g.sort();
    g
}

fn enumerate() -> Vec<serde_json::Value> {
    let mut rows = Vec::new();
    for r in REACTIONS {
        for (key, _) in r.reactants {
            if !LITERAL.contains(key) {
                continue;
            }
            let siblings: Vec<&str> = REACTIONS
                .iter()
                .filter(|s| s.acid_protons.is_some() && gases(s.products) == gases(r.products))
                .map(|s| s.equation)
                .collect();
            rows.push(serde_json::json!({
                "table": "REACTIONS",
                "equation": r.equation,
                "literal": key,
                "route": if siblings.is_empty() { "uncovered" } else { "acid-protons-sibling" },
                "siblings": siblings,
            }));
        }
    }
    for r in ORG_REACTIONS {
        for (key, _) in r.reactants {
            if LITERAL.contains(key) {
                rows.push(serde_json::json!({
                    "table": "ORG_REACTIONS",
                    "equation": r.equation,
                    "literal": key,
                    "route": if r.name == "saponification" { "named-strong-base-reserve" } else { "uncovered" },
                    "siblings": [],
                }));
            }
        }
    }
    rows
}

#[test]
fn literal_reagent_coverage_table_is_current() {
    let fixture: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "fixtures/strong-reserve-coverage-20261027.json"
    ))
    .unwrap();
    assert_eq!(
        enumerate(),
        fixture,
        "regenerate the reserve coverage fixture and docs table after review"
    );
    assert!(
        fixture.iter().all(|row| row["route"] != "uncovered"),
        "a literal strong reagent row has no certified reserve route"
    );
}

fn amount(v: &Vessel, key: &str) -> f64 {
    v.moles_of(&SpeciesId::new(key)).0
}
fn add(v: &mut Vessel, key: &str, n: f64, phase: Phase) {
    v.deposit(SpeciesId::new(key), Moles(n), phase);
}
fn wet() -> Vessel {
    let mut v = Vessel::new(VesselId(0), "certified reserve control");
    add(&mut v, "water", 2.0, Phase::Liquid);
    add(&mut v, "ethyl_acetate", 0.01, Phase::Liquid);
    v
}
fn saponify(v: Vessel) -> (Vessel, Vec<Event>) {
    let mut b = Bench::new();
    b.vessels[0] = v;
    let events = b
        .step_with(
            Operator::React {
                vessel: VesselId(0),
                reaction: "saponification".into(),
            },
            &mut SolverStack::new(vec![]),
            &PermissiveScreen,
        )
        .unwrap();
    (b.vessels.remove(0), events)
}
fn reacted(events: &[Event]) -> bool {
    events.iter().any(|e| matches!(e, Event::OrgReacted { .. }))
}

#[test]
fn barium_hydroxide_is_a_two_equivalent_certified_base() {
    let mut v = wet();
    add(&mut v, "Ba(OH)2", 0.002, Phase::Solid);
    let (after, events) = saponify(v);
    assert!(
        (amount(&after, "ethanol") - 0.004).abs() < 1e-12,
        "{events:?}"
    );
    assert!((amount(&after, "Ba+2") - 0.002).abs() < 1e-12);
    assert!((amount(&after, "CH3COO-") - 0.004).abs() < 1e-12);
    assert_eq!(amount(&after, "Ba(OH)2"), 0.0);
}

#[test]
fn sulfate_is_a_two_equivalent_spectator_anion() {
    let mut v = wet();
    add(&mut v, "Na2SO4", 0.001, Phase::Solid);
    add(&mut v, "NaOH", 0.003, Phase::Solid);
    let (after, events) = saponify(v);
    assert!(
        (amount(&after, "ethanol") - 0.003).abs() < 1e-12,
        "{events:?}"
    );
    assert!((amount(&after, "Na2SO4") - 0.001).abs() < 1e-12);
}

#[test]
fn exactly_neutralised_base_leaves_no_reserve_boundary() {
    let mut v = wet();
    add(&mut v, "NaOH", 0.003, Phase::Solid);
    add(&mut v, "HBr", 0.003, Phase::Liquid);
    let (after, events) = saponify(v);
    assert!(!reacted(&events), "{events:?}");
    assert_eq!(amount(&after, "ethanol"), 0.0);
}

#[test]
fn carbonate_and_ammonia_buffers_are_refused_by_name_not_counted() {
    for (buffer, phase, class) in [
        ("NaHCO3", Phase::Solid, "carbonate"),
        ("NH3", Phase::Aqueous, "ammonia"),
        ("H3PO4", Phase::Liquid, "phosphate"),
    ] {
        let mut v = wet();
        add(&mut v, "NaOH", 0.005, Phase::Solid);
        add(&mut v, buffer, 0.001, phase);
        let (after, events) = saponify(v);
        assert!(!reacted(&events), "{buffer}: {events:?}");
        assert_eq!(amount(&after, "ethanol"), 0.0);
        assert!(
            format!("{events:?}").contains(class),
            "{buffer}: refusal must name the {class} buffer: {events:?}"
        );
    }
}

fn bleach_with(acid: &str, extra: Option<(&str, f64)>) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "bleach and acid");
    add(&mut v, "water", 5.0, Phase::Liquid);
    add(&mut v, "Na+", 0.02, Phase::Aqueous);
    add(&mut v, "ClO-", 0.01, Phase::Aqueous);
    add(&mut v, "Cl-", 0.01, Phase::Aqueous);
    add(&mut v, acid, 0.004, Phase::Liquid);
    if let Some((k, n)) = extra {
        add(&mut v, k, n, Phase::Aqueous);
    }
    v.solute_charge = displacement::solute_charge(&v);
    v
}

#[test]
fn literal_hydrobromic_acid_drives_the_solved_bleach_row() {
    let mut v = bleach_with("HBr", None);
    assert!((strong_reserve::certified_literal_acid_protons(&v) - 0.004).abs() < 1e-15);
    let mut solver = CuratedEquilibrator;
    assert!(solver.applies(&v));
    solver.equilibrate(&mut v).unwrap();
    assert_eq!(amount(&v, "HBr"), 0.0);
    assert!((amount(&v, "Br-") - 0.004).abs() < 1e-12);
    assert!((amount(&v, "ClO-") - 0.008).abs() < 1e-12);
    assert!((amount(&v, "Cl-") - 0.008).abs() < 1e-12);
    assert!(v.solute_charge.abs() < 1e-12, "{}", v.solute_charge);
}

#[test]
fn a_carbonate_buffer_blocks_the_literal_acid_certificate() {
    let v = bleach_with("HBr", Some(("HCO3-", 1e-4)));
    assert_eq!(strong_reserve::certified_literal_acid_protons(&v), 0.0);
    assert!(!CuratedEquilibrator.applies(&v));
}

#[test]
fn sulfuric_acid_is_not_an_acid_side_carrier() {
    let v = bleach_with("H2SO4", None);
    assert_eq!(strong_reserve::certified_literal_acid_protons(&v), 0.0);
    assert!(!CuratedEquilibrator.applies(&v));
}
