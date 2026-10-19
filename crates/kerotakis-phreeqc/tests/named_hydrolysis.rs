//! Real native aqueous-tail controls for the explicit named to-completion verb.
#![cfg(feature = "engine")]
use kerotakis_core::{ledger::ConservedLedger, *};
use kerotakis_phreeqc::PhreeqcEquilibrator;
fn mol(v: &Vessel, key: &str) -> f64 {
    v.moles_of(&SpeciesId::new(key)).0
}
fn acetate(v: &Vessel) -> f64 {
    ["CH3COOH", "CH3COO-", "NaOAc"]
        .iter()
        .map(|key| mol(v, key))
        .sum()
}
fn step(b: &mut Bench, s: &mut SolverStack, op: Operator) -> Vec<Event> {
    let events = b.step_with(op, s, &PermissiveScreen).unwrap();
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::SolverFailed { .. })),
        "{events:?}"
    );
    events
}
fn add(b: &mut Bench, s: &mut SolverStack, key: &str, n: f64) {
    step(
        b,
        s,
        Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new(key),
            moles: Moles(n),
            at: None,
        },
    );
}
fn setup() -> (Bench, SolverStack) {
    let mut b = Bench::new();
    b.vessels[0].thermal_mode = ThermalMode::Thermostatted(Kelvin(298.15));
    let mut s =
        kerotakis_stack::standard_stack(vec![Box::new(PhreeqcEquilibrator::new().unwrap())]);
    add(&mut b, &mut s, "water", 1.1068553982792118);
    (b, s)
}
fn closed(a: &Vessel, b: &Vessel) {
    let x = ConservedLedger::from_vessel(a);
    let y = ConservedLedger::from_vessel(b);
    let keys: std::collections::BTreeSet<_> = x.elements.keys().chain(y.elements.keys()).collect();
    for k in keys {
        assert!(
            (x.elements.get(k).copied().unwrap_or(0.0) - y.elements.get(k).copied().unwrap_or(0.0))
                .abs()
                < 1e-9,
            "{k}: {x:?} -> {y:?}"
        );
    }
    assert!((x.charge - y.charge).abs() < 1e-9, "charge: {x:?} -> {y:?}");
}
#[test]
fn native_named_hydrolysis_closes_both_limits_and_both_alkali_counterions() {
    for counterion in ["NaOH", "KOH"] {
        for base in [0.005, 0.02] {
            let (mut b, mut s) = setup();
            add(&mut b, &mut s, "ethyl_acetate", 0.01);
            add(&mut b, &mut s, counterion, base);
            let before = b.vessels[0].clone();
            let events = step(
                &mut b,
                &mut s,
                Operator::React {
                    vessel: VesselId(0),
                    reaction: "saponification".into(),
                },
            );
            let after = b.vessels[0].clone();
            eprintln!(
                "NAMED_ALKALINE_STATE {}",
                serde_json::json!({"counterion":counterion,"base":base,"before":before,"after":after})
            );
            let expected = base.min(0.01);
            assert!(
                (mol(&after, "ethanol") - expected).abs() < 1e-9,
                "{events:?}"
            );
            assert!((mol(&after, "ethyl_acetate") - (0.01 - expected)).abs() < 1e-9);
            assert!((acetate(&after) - expected).abs() < 1e-9);
            closed(&before, &after);
            let again_events = step(
                &mut b,
                &mut s,
                Operator::React {
                    vessel: VesselId(0),
                    reaction: "saponification".into(),
                },
            );
            let again = &b.vessels[0];
            assert!(
                (mol(again, "ethanol") - expected).abs() < 1e-9,
                "{again_events:?}"
            );
            closed(&after, again);
        }
    }
}
#[test]
fn neutralization_order_reserves_the_same_hydroxide_for_named_cleavage() {
    for acid_first in [false, true] {
        let (mut b, mut s) = setup();
        add(&mut b, &mut s, "ethyl_acetate", 0.01);
        if acid_first {
            add(&mut b, &mut s, "HCl", 0.003);
        }
        add(&mut b, &mut s, "NaOH", 0.005);
        if !acid_first {
            add(&mut b, &mut s, "HCl", 0.003);
        }
        let before = b.vessels[0].clone();
        let events = step(
            &mut b,
            &mut s,
            Operator::React {
                vessel: VesselId(0),
                reaction: "saponification".into(),
            },
        );
        let after = &b.vessels[0];
        eprintln!(
            "NAMED_ALKALINE_ACID {}",
            serde_json::json!({"acid_first":acid_first,"before":before,"after":after})
        );
        assert!((mol(after, "ethanol") - 0.002).abs() < 1e-9, "{events:?}");
        assert!((mol(after, "ethyl_acetate") - 0.008).abs() < 1e-9);
        closed(&before, after);
    }
}
#[test]
fn real_weak_base_and_acetate_states_never_earn_caustic_conversion() {
    for recipe in [0, 1, 2] {
        let (mut b, mut s) = setup();
        add(&mut b, &mut s, "ethyl_acetate", 0.01);
        match recipe {
            0 => {
                add(&mut b, &mut s, "NaOAc", 0.005);
                add(&mut b, &mut s, "CH3COOH", 0.005);
            }
            1 => add(&mut b, &mut s, "Na2CO3", 0.001),
            _ => add(&mut b, &mut s, "NH3", 0.001),
        }
        let before = b.vessels[0].clone();
        let events = step(
            &mut b,
            &mut s,
            Operator::React {
                vessel: VesselId(0),
                reaction: "saponification".into(),
            },
        );
        let after = &b.vessels[0];
        eprintln!(
            "NAMED_ALKALINE_NEGATIVE {}",
            serde_json::json!({"recipe":recipe,"before":before,"after":after})
        );
        assert_eq!(mol(after, "ethanol"), 0.0, "{events:?}");
        assert!((mol(after, "ethyl_acetate") - 0.01).abs() < 1e-12);
        assert!(!events.iter().any(|e| matches!(e, Event::OrgReacted { .. })));
        closed(&before, after);
    }
}
