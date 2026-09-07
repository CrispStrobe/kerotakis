use kerotakis_core::*;

fn deposit(bench: &mut Bench, key: &str, n: f64) {
    bench.vessels[0].deposit(
        SpeciesId::new(key),
        Moles(n),
        species::lookup_key(key).unwrap().standard_phase,
    );
}

fn request(bench: &mut Bench, name: &str) -> Vec<Event> {
    bench
        .step_with(
            Operator::React {
                vessel: VesselId(0),
                reaction: name.into(),
            },
            &mut SolverStack::new(vec![]),
            &PermissiveScreen,
        )
        .unwrap()
}

#[test]
fn repeated_equilibrium_reports_zero_conversion_with_remaining_reactants() {
    for scale in [0.001, 1.0, 1000.0] {
        let mut bench = Bench::new();
        for (key, n) in [
            ("CH3COOH", 0.017),
            ("ethanol", 0.009),
            ("ethyl_acetate", 0.002),
            ("water", 0.004),
        ] {
            deposit(&mut bench, key, n * scale);
        }
        request(&mut bench, "esterification");
        assert!(bench.vessels[0].moles_of(&SpeciesId::new("CH3COOH")).0 > 0.0);
        assert!(bench.vessels[0].moles_of(&SpeciesId::new("ethanol")).0 > 0.0);
        let before = serde_json::to_value(&bench.vessels[0].contents).unwrap();
        let events = request(&mut bench, "esterification");
        assert_eq!(
            before,
            serde_json::to_value(&bench.vessels[0].contents).unwrap()
        );
        assert!(events.iter().any(|e| matches!(e, Event::OrgReacted { extent: Moles(0.0), boundary, .. } if boundary.contains("inventory unchanged"))));
        assert!(!events
            .iter()
            .any(|e| matches!(e, Event::NotYetModeled { .. })));
    }
}

#[test]
fn empty_single_reagent_and_below_threshold_equilibrium_feeds_have_no_direction() {
    for key in [
        None,
        Some("CH3COOH"),
        Some("ethanol"),
        Some("ethyl_acetate"),
        Some("water"),
    ] {
        let mut bench = Bench::new();
        if let Some(key) = key {
            deposit(&mut bench, key, 0.01);
        }
        let before = serde_json::to_value(&bench.vessels[0].contents).unwrap();
        let events = request(&mut bench, "esterification");
        assert_eq!(
            before,
            serde_json::to_value(&bench.vessels[0].contents).unwrap()
        );
        assert!(events.iter().any(|e| matches!(
            e,
            Event::NotYetModeled {
                cause: ops::NotModelledCause::NothingToActOn,
                ..
            }
        )));
        assert!(!events.iter().any(|e| matches!(e, Event::OrgReacted { .. })));
    }
    let mut bench = Bench::new();
    for key in ["CH3COOH", "ethanol", "ethyl_acetate", "water"] {
        deposit(&mut bench, key, 1e-13);
    }
    let events = request(&mut bench, "esterification");
    assert!(events.iter().any(|e| matches!(
        e,
        Event::NotYetModeled {
            cause: ops::NotModelledCause::NothingToActOn,
            ..
        }
    )));
    assert!(!events.iter().any(|e| matches!(e, Event::OrgReacted { .. })));
}

#[test]
fn absent_or_depleted_to_completion_feed_is_not_reported_as_equilibrium() {
    for ester in [0.0, 0.004] {
        let mut bench = Bench::new();
        deposit(&mut bench, "ethyl_acetate", ester);
        let before = serde_json::to_value(&bench.vessels[0].contents).unwrap();
        let events = request(&mut bench, "saponification");
        assert_eq!(
            before,
            serde_json::to_value(&bench.vessels[0].contents).unwrap()
        );
        assert!(events.iter().any(|e| matches!(
            e,
            Event::NotYetModeled {
                cause: ops::NotModelledCause::NothingToActOn,
                ..
            }
        )));
        assert!(!events.iter().any(|e| matches!(e, Event::OrgReacted { .. })));
    }
    let mut bench = Bench::new();
    deposit(&mut bench, "ethyl_acetate", 0.004);
    deposit(&mut bench, "NaOH", 0.002);
    request(&mut bench, "saponification");
    let events = request(&mut bench, "saponification");
    assert!(events.iter().any(|e| matches!(
        e,
        Event::NotYetModeled {
            cause: ops::NotModelledCause::NothingToActOn,
            ..
        }
    )));
}

#[test]
fn products_only_can_react_in_reverse_and_unknown_reaction_is_single_refusal() {
    let mut bench = Bench::new();
    deposit(&mut bench, "ethyl_acetate", 0.01);
    deposit(&mut bench, "water", 0.02);
    let events = request(&mut bench, "esterification");
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::OrgReacted { extent, .. } if extent.0 < 0.0)));
    assert!(!events
        .iter()
        .any(|e| matches!(e, Event::NotYetModeled { .. })));
    let events = request(&mut bench, "unregistered-reaction");
    assert!(matches!(
        events.as_slice(),
        [Event::NotYetModeled {
            cause: ops::NotModelledCause::NotParameterised,
            ..
        }]
    ));
}
