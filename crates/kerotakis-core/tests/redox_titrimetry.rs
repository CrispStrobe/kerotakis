//! EXP-39: redox titrimetry — the grammar, the curated chemistry, and
//! the refusals. The engine-backed acceptance (a permanganate burette
//! standardised against oxalic acid) lives in `kerotakis-phreeqc`,
//! because it needs the aqueous solver to be real.

use kerotakis_core::ops::{Compare, Endpoint};
use kerotakis_core::script::parse_op;
use kerotakis_core::*;

fn op(line: &str) -> Operator {
    parse_op(line)
        .unwrap_or_else(|e| panic!("{line}: {e}"))
        .unwrap_or_else(|| panic!("{line}: parsed to no operator"))
}

fn refuse(line: &str) -> String {
    match parse_op(line) {
        Err(e) => e,
        Ok(other) => panic!("{line} should not parse, got {other:?}"),
    }
}

// ── Grammar ─────────────────────────────────────────────────────────

/// CAP-12's line means exactly what it always meant, down to the
/// endpoint being absent from the wire.
#[test]
fn the_ph_endpoint_is_unchanged_and_stays_the_default() {
    let parsed = op("titrate v1 NaOH 1M 1mL until ph 7");
    match &parsed {
        Operator::Titrate {
            target_ph,
            endpoint,
            max_steps,
            ..
        } => {
            assert_eq!(*target_ph, 7.0);
            assert_eq!(*endpoint, Endpoint::Ph);
            assert_eq!(*max_steps, 100);
        }
        other => panic!("{other:?}"),
    }
    let json = serde_json::to_string(&parsed).expect("serialise");
    assert!(
        !json.contains("endpoint"),
        "a pH titration must serialise exactly as it did before EXP-39: {json}"
    );
}

#[test]
fn the_potentiometric_endpoint_parses_with_every_comparison() {
    for (line, want) in [
        ("titrate v1 KMnO4 0.02M 0.1mL until pe > 8", Compare::Above),
        (
            "titrate v1 KMnO4 0.02M 0.1mL until pe >= 8",
            Compare::AtLeast,
        ),
        ("titrate v1 KMnO4 0.02M 0.1mL until pe < 8", Compare::Below),
        (
            "titrate v1 KMnO4 0.02M 0.1mL until pe <= 8",
            Compare::AtMost,
        ),
        (
            "titrate v1 KMnO4 0.02M 0.1mL until pe above 8",
            Compare::Above,
        ),
        (
            "titrate v1 KMnO4 0.02M 0.1mL until pe below 8",
            Compare::Below,
        ),
    ] {
        match op(line) {
            Operator::Titrate { endpoint, .. } => assert_eq!(
                endpoint,
                Endpoint::Pe {
                    compare: want,
                    value: 8.0
                },
                "{line}"
            ),
            other => panic!("{line}: {other:?}"),
        }
    }
}

#[test]
fn the_self_indicating_endpoint_parses_in_both_spellings() {
    for line in [
        "titrate v1 KMnO4 0.02M 0.1mL until colour persists",
        "titrate v1 KMnO4 0.02M 0.1mL until color persists",
    ] {
        match op(line) {
            Operator::Titrate { endpoint, .. } => {
                assert_eq!(endpoint, Endpoint::ColourPersists, "{line}")
            }
            other => panic!("{line}: {other:?}"),
        }
    }
}

#[test]
fn max_still_follows_every_endpoint() {
    for (line, steps) in [
        ("titrate v1 NaOH 1M 1mL until ph 7 max 50", 50),
        ("titrate v1 KMnO4 0.02M 0.1mL until pe > 8 max 250", 250),
        (
            "titrate v1 KMnO4 0.02M 0.1mL until colour persists max 300",
            300,
        ),
    ] {
        match op(line) {
            Operator::Titrate { max_steps, .. } => assert_eq!(max_steps, steps, "{line}"),
            other => panic!("{line}: {other:?}"),
        }
    }
}

/// A grammar that accepts nonsense teaches nonsense. Each refusal names
/// what was wrong rather than reprinting the usage line alone.
#[test]
fn malformed_endpoints_are_refused_with_a_reason() {
    let unknown = refuse("titrate v1 KMnO4 0.02M 0.1mL until eh 0.8");
    assert!(unknown.contains("endpoint"), "{unknown}");
    assert!(unknown.contains("colour persists"), "{unknown}");

    let bad_op = refuse("titrate v1 KMnO4 0.02M 0.1mL until pe ~ 8");
    assert!(bad_op.contains("comparison"), "{bad_op}");

    let bad_value = refuse("titrate v1 KMnO4 0.02M 0.1mL until pe > purple");
    assert!(bad_value.contains("pe target"), "{bad_value}");

    let truncated = refuse("titrate v1 KMnO4 0.02M 0.1mL until pe >");
    assert!(truncated.contains("usage"), "{truncated}");

    let not_persisting = refuse("titrate v1 KMnO4 0.02M 0.1mL until colour appears");
    assert!(
        not_persisting.contains("colour persists"),
        "{not_persisting}"
    );

    let trailing = refuse("titrate v1 KMnO4 0.02M 0.1mL until colour persists soon");
    assert!(trailing.contains("max"), "{trailing}");
}

/// The hole the grammar fuzz target found, on every numeric slot.
///
/// Rust parses `1e999` into `f64::INFINITY` without complaint and
/// serde_json refuses to write one, so before this guard a titration
/// with an infinite target parsed, ran, and produced an operator log the
/// bench could not save itself with. The pH slot had carried the same
/// hole since CAP-12.
#[test]
fn a_non_finite_endpoint_is_refused_rather_than_saved() {
    // A volume cannot reach infinity through `1e999`: `split_unit` cuts at
    // the first letter, so the `e` ends the number and the rest is read as
    // a unit. It gets there through sheer length instead.
    let vast = "9".repeat(400);
    for line in [
        "titrate v1 NaOH 1M 1mL until ph 1e999".to_string(),
        "titrate v1 KMnO4 0.02M 0.1mL until pe > 1e999".to_string(),
        "titrate v1 KMnO4 0.02M 0.1mL until pe < -1e999".to_string(),
        "titrate v1 KMnO4 1e999M 0.1mL until colour persists".to_string(),
        format!("titrate v1 KMnO4 0.02M {vast}mL until colour persists"),
    ] {
        let why = refuse(&line);
        assert!(
            why.contains("finite"),
            "{line} must be refused for being unwritable: {why}"
        );
    }
    // The fuzz target's second find: finite, but an exponent no solver
    // represents. It round-tripped badly through the operator log, and a
    // pH of 6.7e49 was never chemistry to begin with.
    for line in [
        "titrate v1 KMnO4 0.02M 0.1mL until ph \
         66666765555555555555555555555555555555555555555555.555555555555555552",
        "titrate v1 KMnO4 0.02M 0.1mL until pe > 1e40",
        "titrate v1 NaOH 1M 1mL until ph -1000",
    ] {
        let why = refuse(line);
        assert!(
            why.contains("exponent"),
            "{line} must be refused as out of range: {why}"
        );
    }
    // The realistic extremes stay legal: superacids and strong oxidants
    // live well inside the bound.
    for line in [
        "titrate v1 NaOH 1M 1mL until ph -2",
        "titrate v1 NaOH 1M 1mL until ph 15",
        "titrate v1 KMnO4 0.02M 0.1mL until pe > 21",
        "titrate v1 KMnO4 0.02M 0.1mL until pe < -12",
    ] {
        op(line);
    }

    // And the ordinary lines still parse.
    for line in [
        "titrate v1 NaOH 1M 1mL until ph 7",
        "titrate v1 KMnO4 0.02M 0.1mL until pe > 8",
    ] {
        let parsed = op(line);
        serde_json::to_string(&parsed).expect("an accepted operator serialises");
    }
}

/// Every new endpoint survives the operator log, which is the save file.
#[test]
fn the_new_endpoints_round_trip_through_json() {
    for line in [
        "titrate v1 KMnO4 0.02M 0.1mL until pe >= 12.5 max 40",
        "titrate v1 KMnO4 0.02M 0.1mL until colour persists max 40",
    ] {
        let parsed = op(line);
        let json = serde_json::to_string(&parsed).expect("serialise");
        let back: Operator = serde_json::from_str(&json).expect("deserialise");
        assert_eq!(back, parsed, "{line}");
    }
}

/// A payload written before EXP-39 has no `endpoint` field at all, and
/// must still mean the pH titration it meant then.
#[test]
fn a_pre_exp39_payload_still_means_a_ph_titration() {
    let legacy = r#"{"op":"titrate","vessel":0,"titrant":"NaOH","concentration":1.0,
                     "step":0.001,"target_ph":7.0,"max_steps":100}"#;
    let back: Operator = serde_json::from_str(legacy).expect("legacy payload");
    match back {
        Operator::Titrate {
            endpoint,
            target_ph,
            ..
        } => {
            assert_eq!(endpoint, Endpoint::Ph);
            assert_eq!(target_ph, 7.0);
        }
        other => panic!("{other:?}"),
    }
}

// ── The curated chemistry ───────────────────────────────────────────

/// The row this bench fires, and the textbook row it is the basic form
/// of, balance as the same reaction.
///
/// The acidic equation is what a student writes; the row is written with
/// six hydroxides instead of six protons, because a vessel here has no
/// proton portion. Both must balance, and the difference between them
/// must be exactly six waters — which is the definition of the two forms
/// being one reaction.
#[test]
fn the_oxalate_rows_balance_and_so_does_the_textbook_form() {
    let acidic = stoich::parse_equation("2 MnO4- + 5 H2C2O4 + 6 H+ → 2 Mn+2 + 10 CO2 + 8 H2O")
        .expect("parse the acidic textbook equation");
    assert!(acidic.is_balanced(), "{:?}", acidic.element_imbalance());

    let basic = stoich::parse_equation("2 MnO4- + 5 H2C2O4 → 2 Mn+2 + 10 CO2 + 2 H2O + 6 OH-")
        .expect("parse the row as written");
    assert!(basic.is_balanced(), "{:?}", basic.element_imbalance());

    let from_the_salt =
        stoich::parse_equation("2 KMnO4 + 5 H2C2O4 → 2 Mn+2 + 2 K+ + 10 CO2 + 2 H2O + 6 OH-")
            .expect("parse the solid-titrant row");
    assert!(
        from_the_salt.is_balanced(),
        "{:?}",
        from_the_salt.element_imbalance()
    );
}

/// Both rows are actually installed, and they name the species the
/// registry knows rather than a plausible-looking string.
#[test]
fn both_oxalate_rows_are_installed() {
    let rows: Vec<_> = curated::REACTIONS
        .iter()
        .filter(|r| r.reactants.iter().any(|(k, _)| *k == "H2C2O4"))
        .collect();
    assert_eq!(rows.len(), 2, "the ion route and the solid route");
    for row in rows {
        for (key, _) in row.reactants {
            assert!(species::lookup_key(key).is_some(), "reactant {key}");
        }
        for (key, _, _) in row.products {
            assert!(species::lookup_key(key).is_some(), "product {key}");
        }
    }
}

/// The reaction fires, in the ratio the half-equations give: two
/// permanganate for five oxalic acid, five electrons each way.
#[test]
fn permanganate_oxidises_oxalic_acid_in_the_two_to_five_ratio() {
    let mut bench = Bench::new();
    let mut stack = SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(CuratedEquilibrator),
        Box::new(HonestyEquilibrator),
    ]);
    let v = VesselId(0);
    for (key, moles) in [("water", 5.55), ("H2C2O4", 0.005), ("MnO4-", 0.001)] {
        bench
            .step_with(
                Operator::Add {
                    vessel: v,
                    species: SpeciesId::new(key),
                    moles: Moles(moles),
                    at: None,
                },
                &mut stack,
                &PermissiveScreen,
            )
            .expect("step");
    }
    let vessel = bench.vessel(v).expect("vessel");
    let left = |key: &str| vessel.moles_of(&SpeciesId::new(key)).0;
    // 0.001 mol MnO4- consumes 0.0025 mol H2C2O4 and leaves 0.0025.
    assert!(
        (left("H2C2O4") - 0.0025).abs() < 1e-9,
        "oxalic acid left: {}",
        left("H2C2O4")
    );
    assert!(left("MnO4-") < 1e-12, "permanganate is spent");
    assert!(
        (left("Mn+2") - 0.001).abs() < 1e-9,
        "manganese(II) made: {}",
        left("Mn+2")
    );
}

/// Mass is the invariant, and the hydroxide form is what keeps it. Every
/// element in the flask is where it was before the reaction, which is the
/// whole reason the row is not written with a proton it cannot book.
#[test]
fn the_curated_row_conserves_every_element() {
    let mut bench = Bench::new();
    let mut stack = SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(CuratedEquilibrator),
        Box::new(HonestyEquilibrator),
    ]);
    let v = VesselId(0);
    let element = |bench: &Bench, symbol: &str| -> f64 {
        let vessel = bench.vessel(v).expect("vessel");
        let mut total = 0.0;
        for portion in &vessel.contents {
            if let Some(data) = species::lookup(&portion.species) {
                if let Ok(f) = stoich::parse_formula(data.formula) {
                    total += portion.moles.0 * f.counts.get(symbol).copied().unwrap_or(0.0);
                }
            }
        }
        // Gas that left the beaker is still matter this bench accounted
        // for; the headspace holds it in a sealed vessel and the event
        // stream names it in an open one.
        total
    };
    for (key, moles) in [("water", 5.55), ("H2C2O4", 0.005)] {
        bench
            .step_with(
                Operator::Add {
                    vessel: v,
                    species: SpeciesId::new(key),
                    moles: Moles(moles),
                    at: None,
                },
                &mut stack,
                &PermissiveScreen,
            )
            .expect("step");
    }
    let before: Vec<f64> = ["H", "C", "O", "Mn"]
        .iter()
        .map(|e| element(&bench, e))
        .collect();
    let events = bench
        .step_with(
            Operator::Add {
                vessel: v,
                species: SpeciesId::new("MnO4-"),
                moles: Moles(0.001),
                at: None,
            },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("step");
    let escaped: f64 = events
        .iter()
        .filter_map(|e| match e {
            Event::GasEvolved { species, moles, .. } if species.0 == "CO2" => Some(moles.0),
            _ => None,
        })
        .sum();
    // Adding 0.001 mol MnO4- brings 0.001 Mn and 0.004 O in with it;
    // 10 CO2 per 2 MnO4- leave, carrying 0.005 C and 0.010 O.
    let after: Vec<f64> = ["H", "C", "O", "Mn"]
        .iter()
        .map(|e| element(&bench, e))
        .collect();
    assert!((escaped - 0.005).abs() < 1e-9, "CO2 evolved: {escaped}");
    assert!(
        (after[0] - before[0]).abs() < 1e-9,
        "H: {before:?} {after:?}"
    );
    assert!(
        (after[1] - (before[1] - escaped)).abs() < 1e-9,
        "C: {before:?} {after:?}"
    );
    assert!(
        (after[2] - (before[2] + 0.004 - 2.0 * escaped)).abs() < 1e-9,
        "O: {before:?} {after:?}"
    );
    assert!(
        (after[3] - (before[3] + 0.001)).abs() < 1e-9,
        "Mn: {before:?} {after:?}"
    );
}

// ── Refusals ────────────────────────────────────────────────────────

/// The self-indicating endpoint over a beaker with no aqueous solver
/// says so, once, and does not silently report a finished titration.
#[test]
fn a_colour_endpoint_without_a_solver_says_what_is_missing() {
    let mut bench = Bench::new();
    bench
        .step(Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new("water"),
            moles: Moles(5.55),
            at: None,
        })
        .expect("step");
    let events = bench
        .step(op(
            "titrate v1 KMnO4 0.02M 0.1mL until colour persists max 5",
        ))
        .expect("step");
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::NotYetModeled { what, .. } if what.contains("aqueous solver")
        )),
        "{events:?}"
    );
}

/// EXP-39's refusal, in the shape CAP-12's scope asked for: the burette
/// runs out of budget, and the reason is named rather than left to the
/// reader to infer from a curve that stops.
#[test]
fn an_unreachable_colour_endpoint_refuses_and_says_why() {
    let mut bench = Bench::new();
    let mut stack = SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(CuratedEquilibrator),
        Box::new(HonestyEquilibrator),
    ]);
    let v = VesselId(0);
    bench
        .step_with(
            Operator::Add {
                vessel: v,
                species: SpeciesId::new("water"),
                moles: Moles(5.55),
                at: None,
            },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("step");
    // NaOH has no curated absorption spectrum, so no amount of it can
    // ever end a titration by colour. The bench must say that.
    let events = bench
        .step_with(
            op("titrate v1 NaOH 0.1M 1mL until colour persists max 3"),
            &mut stack,
            &PermissiveScreen,
        )
        .expect("step");
    let refusal = events.iter().find_map(|e| match e {
        Event::NotYetModeled { what, .. } => Some(what.clone()),
        _ => None,
    });
    let refusal = refusal.unwrap_or_else(|| panic!("a reason was owed: {events:?}"));
    assert!(!refusal.is_empty(), "{refusal}");
}

// ── Narration ───────────────────────────────────────────────────────

/// Three registers, and none of them narrates a redox titration as if
/// its point were the pH.
#[test]
fn a_redox_titration_narrates_in_three_registers() {
    for endpoint in [
        Endpoint::ColourPersists,
        Endpoint::Pe {
            compare: Compare::Above,
            value: 8.0,
        },
    ] {
        let event = Event::Titrated {
            vessel: VesselId(0),
            titrant: SpeciesId::new("KMnO4"),
            concentration: 0.02,
            steps: 81,
            total_volume: units::Liters(0.00405),
            final_ph: 1.42,
            curve: vec![(0.0, 1.4), (4.05, 1.42)],
            pe_curve: vec![(4.05, 14.2)],
            endpoint_reached: Some(true),
            endpoint,
        };
        for level in [1, 2, 3] {
            let text = render::render_event(&event, render::Register(level));
            assert!(!text.is_empty(), "level {level} of {endpoint:?}");
            assert!(
                !text.contains("the pH reaches"),
                "level {level} of {endpoint:?} narrates the wrong quantity: {text}"
            );
        }
        let expert = render::render_event(&event, render::Register(3));
        assert!(
            expert.contains("pe"),
            "the expert register owes pe: {expert}"
        );
    }

    // And a pH titration still reads exactly as it did.
    let ph = Event::Titrated {
        vessel: VesselId(0),
        titrant: SpeciesId::new("NaOH"),
        concentration: 1.0,
        steps: 5,
        total_volume: units::Liters(0.005),
        final_ph: 7.1,
        curve: vec![(0.0, 2.0), (5.0, 7.1)],
        pe_curve: Vec::new(),
        endpoint_reached: Some(true),
        endpoint: Endpoint::Ph,
    };
    assert!(render::render_event(&ph, render::Register(1)).contains("pH"));
}
