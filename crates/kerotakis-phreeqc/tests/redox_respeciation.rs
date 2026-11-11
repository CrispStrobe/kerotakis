//! Held fast-redox split respeciation (EC-008 / RX-004, 2026-11-11).
//!
//! After the curated `2 Fe³⁺ + 2 I⁻ → 2 Fe²⁺ + I₂` step the ledger holds iron
//! in two states with no second fast element to couple to. The aqueous view
//! (speciation, redox block, pe) must be solved from that reacted split, not
//! from an O2(g)-fixed pe that re-oxidises ferrous iron with oxygen the bench
//! never booked. Where the split cannot be posed, a typed refusal
//! (`not-modeled.redox-split-not-respeciated`) says so.
//!
//! `respeciation_coverage_is_current` holds
//! `docs/redox-respeciation-coverage-20261111.json`
//! (`python3 tools/redox_respeciation_coverage.py` regenerates it).

const TABLE: &str = include_str!("../../../docs/redox-respeciation-coverage-20261111.json");
const REFUSAL: &str = "not-modeled.redox-split-not-respeciated";

#[test]
fn respeciation_coverage_is_current() {
    let doc: serde_json::Value = serde_json::from_str(TABLE).unwrap();
    let fast: Vec<&str> = doc["fast_redox"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(fast, kerotakis_phreeqc::fast_redox_elements());
    let safe: Vec<&str> = doc["safe_partners"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(safe, kerotakis_phreeqc::respeciation_safe_partners());
    assert_eq!(doc["refusal_key"], REFUSAL);
    let rows = doc["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 2 * fast.len());
    for row in rows {
        let db = row["database"].as_str().unwrap();
        let el = row["element"].as_str().unwrap();
        let table: Vec<i32> = row["posable_states"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_i64().unwrap() as i32)
            .collect();
        assert_eq!(
            table,
            kerotakis_phreeqc::posable_fast_redox_states(db, el),
            "{db} {el}"
        );
    }
    // The EC-008/RX-004 couple is posable on the default route.
    assert_eq!(
        kerotakis_phreeqc::posable_fast_redox_states("wateq4f", "Fe"),
        vec![2, 3]
    );
}

#[cfg(feature = "engine")]
mod engine {
    use super::REFUSAL;
    use kerotakis_core::*;
    use kerotakis_phreeqc::PhreeqcEquilibrator;

    fn add(bench: &mut Bench, eq: &mut PhreeqcEquilibrator, key: &str, moles: f64) -> Vec<Event> {
        bench
            .step_with(
                Operator::Add {
                    vessel: VesselId(0),
                    species: SpeciesId::new(key),
                    moles: Moles(moles),
                    at: None,
                },
                eq,
                &PermissiveScreen,
            )
            .expect("step")
    }

    fn fraction(states: &[RedoxState], element: &str, oxidation: i32) -> f64 {
        let total: f64 = states
            .iter()
            .filter(|s| s.element == element)
            .map(|s| s.molality)
            .sum();
        let want: f64 = states
            .iter()
            .filter(|s| s.element == element && s.oxidation == oxidation)
            .map(|s| s.molality)
            .sum();
        if total > 0.0 {
            want / total
        } else {
            0.0
        }
    }

    fn refused(events: &[Event]) -> bool {
        events.iter().any(|e| {
            matches!(e, Event::NotYetModeled { reason: Some(reason), .. } if reason.key == REFUSAL)
        })
    }

    /// Ferrous and ferric iron held 3:1 by the ledger (the post-reaction
    /// RX-004 shape: 0.32 mmol Fe(II) / 0.08 mmol Fe(III)), in acid, open to
    /// the room. The reported split must be the ledger's, not an O2-set one.
    fn held_split(extra: &[(&str, f64)]) -> (Vec<Event>, Vec<RedoxState>) {
        let mut eq = PhreeqcEquilibrator::new().expect("engine");
        let mut bench = Bench::new();
        let mut events = Vec::new();
        events.extend(add(&mut bench, &mut eq, "water", 5.534));
        events.extend(add(&mut bench, &mut eq, "HCl", 0.001));
        for (key, moles) in extra {
            events.extend(add(&mut bench, &mut eq, key, *moles));
        }
        events.extend(add(&mut bench, &mut eq, "FeSO4", 3.2e-4));
        events.extend(add(&mut bench, &mut eq, "FeCl3", 0.8e-4));
        events.extend(add(&mut bench, &mut eq, "KI", 0.8e-4));
        let v = bench.vessel(VesselId(0)).expect("vessel");
        let redox = v
            .solution
            .as_ref()
            .map(|s| s.redox.clone())
            .unwrap_or_default();
        (events, redox)
    }

    #[test]
    fn held_ferrous_ferric_split_is_speciated_from_the_ledger() {
        let (events, redox) = held_split(&[]);
        assert!(!refused(&events), "{events:?}");
        let ferric = fraction(&redox, "Fe", 3);
        assert!(
            (ferric - 0.20).abs() < 0.01,
            "the view must carry the ledger's 20% ferric, got {:.1}%: {redox:?}",
            ferric * 100.0
        );
    }

    /// Nitrate is not a safe partner: at the iron couple's potential its
    /// reduction is thermodynamically favoured, so posing the split would
    /// publish a denitrification. The view refuses, typed.
    #[test]
    fn held_split_beside_nitrate_is_refused_typed() {
        let (events, _) = held_split(&[("NaNO3", 0.001)]);
        assert!(refused(&events), "{events:?}");
    }

    /// A single held state is the existing `FAST_REDOX` pin, untouched: no
    /// refusal, and ferrous iron stays ferrous.
    #[test]
    fn single_held_state_is_unchanged_and_not_refused() {
        let mut eq = PhreeqcEquilibrator::new().expect("engine");
        let mut bench = Bench::new();
        let mut events = Vec::new();
        events.extend(add(&mut bench, &mut eq, "water", 5.534));
        events.extend(add(&mut bench, &mut eq, "H2SO4", 0.001));
        events.extend(add(&mut bench, &mut eq, "FeSO4", 4e-4));
        events.extend(add(&mut bench, &mut eq, "KI", 1e-3));
        assert!(!refused(&events), "{events:?}");
        let redox = bench
            .vessel(VesselId(0))
            .unwrap()
            .solution
            .as_ref()
            .map(|s| s.redox.clone())
            .unwrap_or_default();
        assert!(fraction(&redox, "Fe", 2) > 0.98, "{redox:?}");
    }
}
