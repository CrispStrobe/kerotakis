//! Actual named Bench operator on explicitly constructed canonical inventories.
//! These controls are not new corpus experiments or measured kinetic results.
use kerotakis_core::{ledger::ConservedLedger, *};

fn amount(v: &Vessel, key: &str) -> f64 {
    v.moles_of(&SpeciesId::new(key)).0
}
fn acetate(v: &Vessel) -> f64 {
    ["CH3COO-", "CH3COOH", "NaOAc"]
        .iter()
        .map(|k| amount(v, k))
        .sum()
}
fn add(v: &mut Vessel, key: &str, n: f64, phase: Phase) {
    v.deposit(SpeciesId::new(key), Moles(n), phase);
}
fn wet() -> Vessel {
    let mut v = Vessel::new(VesselId(0), "named alkaline control");
    add(&mut v, "water", 2.0, Phase::Liquid);
    add(&mut v, "ethyl_acetate", 0.01, Phase::Liquid);
    v
}
fn solved(base: f64) -> Vessel {
    let mut v = wet();
    add(&mut v, "Na+", base, Phase::Aqueous);
    add(&mut v, "base_equivalents", base, Phase::Aqueous);
    v.free_hydroxide = base;
    v.solute_charge = kerotakis_core::displacement::solute_charge(&v);
    v
}
fn react(v: Vessel) -> (Vessel, Vec<Event>) {
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
fn closes(before: &Vessel, after: &Vessel) {
    let a = ConservedLedger::from_vessel(before);
    let b = ConservedLedger::from_vessel(after);
    let keys: std::collections::BTreeSet<_> = a.elements.keys().chain(b.elements.keys()).collect();
    for k in keys {
        assert!(
            (a.elements.get(k).copied().unwrap_or(0.0) - b.elements.get(k).copied().unwrap_or(0.0))
                .abs()
                < 1e-10,
            "{k}: {a:?} -> {b:?}"
        );
    }
    assert!(
        (a.charge - b.charge).abs() < 1e-12,
        "charge: {} -> {}",
        a.charge,
        b.charge
    );
}
#[test]
fn solved_base_and_ester_limits_use_the_analytical_reserve() {
    for base in [0.0001, 0.005, 0.02] {
        let v = solved(base);
        let expected = base.min(0.01);
        let (after, events) = react(v.clone());
        assert!(
            (amount(&after, "ethanol") - expected).abs() < 1e-12,
            "{events:?}"
        );
        assert!((amount(&after, "ethyl_acetate") - (0.01 - expected)).abs() < 1e-12);
        assert!((acetate(&after) - expected).abs() < 1e-12);
        assert_eq!(
            amount(&after, "Na+"),
            base,
            "do not introduce a second sodium counterion"
        );
        assert_eq!(amount(&after, "NaOAc"), 0.0);
        closes(&v, &after);
    }
}
#[test]
fn literal_sodium_hydroxide_keeps_its_original_balanced_conversion() {
    let mut v = wet();
    add(&mut v, "NaOH", 0.005, Phase::Solid);
    let (after, _) = react(v.clone());
    assert!((amount(&after, "ethanol") - 0.005).abs() < 1e-12);
    assert!((acetate(&after) - 0.005).abs() < 1e-12);
    closes(&v, &after);
}
#[test]
fn potassium_hydroxide_uses_the_same_net_ionic_sink_without_sodium() {
    let mut v = wet();
    add(&mut v, "KOH", 0.005, Phase::Solid);
    let (after, events) = react(v.clone());
    assert!(
        (amount(&after, "ethanol") - 0.005).abs() < 1e-12,
        "{events:?}"
    );
    assert!((amount(&after, "K+") - 0.005).abs() < 1e-12);
    assert_eq!(amount(&after, "Na+"), 0.0);
    assert_eq!(amount(&after, "NaOAc"), 0.0);
    closes(&v, &after);
}
#[test]
fn literal_hydroxide_and_basis_are_owned_once_each() {
    let mut v = solved(0.002);
    add(&mut v, "K+", 0.003, Phase::Aqueous);
    add(&mut v, "OH-", 0.003, Phase::Aqueous);
    let (after, events) = react(v.clone());
    assert!(
        (amount(&after, "ethanol") - 0.005).abs() < 1e-12,
        "{events:?}"
    );
    closes(&v, &after);
}
#[test]
fn acetate_buffer_basis_and_measured_hydroxide_are_not_a_caustic_reserve() {
    let mut v = solved(0.005);
    add(&mut v, "CH3COOH", 0.01, Phase::Aqueous);
    v.free_hydroxide = 1e-10;
    let before = serde_json::to_value(&v).unwrap();
    let (after, events) = react(v);
    assert_eq!(before, serde_json::to_value(&after).unwrap(), "{events:?}");
    assert!(!events.iter().any(|e| matches!(e, Event::OrgReacted { .. })));
}
#[test]
fn excess_strong_base_after_the_entire_acetate_reservoir_is_spent_only_once() {
    let mut v = solved(0.015);
    add(&mut v, "CH3COOH", 0.01, Phase::Aqueous);
    let (after, events) = react(v.clone());
    assert!(
        (amount(&after, "ethanol") - 0.005).abs() < 1e-12,
        "{events:?}"
    );
    assert!((acetate(&after) - 0.015).abs() < 1e-12);
    closes(&v, &after);
    let (again, _) = react(after.clone());
    assert_eq!(
        serde_json::to_value(&again).unwrap(),
        serde_json::to_value(&after).unwrap()
    );
}
#[test]
fn neutralized_salt_and_bare_cation_cannot_supply_hydroxide() {
    for chloride in [0.0, 0.005] {
        let mut v = wet();
        add(&mut v, "Na+", 0.005, Phase::Aqueous);
        if chloride > 0.0 {
            add(&mut v, "Cl-", chloride, Phase::Aqueous);
        }
        let before = serde_json::to_value(&v).unwrap();
        let (after, _) = react(v);
        assert_eq!(before, serde_json::to_value(&after).unwrap());
    }
}
#[test]
fn unknown_acid_base_reservoirs_refuse_atomically_even_beside_literal_base() {
    for key in ["NH3", "Na2CO3", "glucose"] {
        let mut v = wet();
        add(&mut v, "NaOH", 0.005, Phase::Solid);
        add(&mut v, key, 0.001, Phase::Aqueous);
        let before = serde_json::to_value(&v).unwrap();
        let (after, events) = react(v);
        assert_eq!(
            before,
            serde_json::to_value(&after).unwrap(),
            "{key}: {events:?}"
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::NotYetModeled { .. })),
            "{events:?}"
        );
    }
}
#[test]
fn serialization_does_not_recreate_spent_capacity_or_require_a_live_solution_cache() {
    let v = solved(0.005);
    let (after, events) = react(v);
    assert!(
        (amount(&after, "ethanol") - 0.005).abs() < 1e-12,
        "{events:?}"
    );
    let restored: Vessel = serde_json::from_value(serde_json::to_value(&after).unwrap()).unwrap();
    let (again, _) = react(restored);
    assert_eq!(
        serde_json::to_value(&again).unwrap(),
        serde_json::to_value(&after).unwrap()
    );
}
