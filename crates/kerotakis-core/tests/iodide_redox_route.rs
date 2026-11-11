//! Oxidant/iodide redox route (2026-11-11): the Fe³⁺/I⁻ equilibrium (and the
//! generic family of equilibrium-limited plain ion couples against iodide)
//! must run on the inventory the bench actually holds — undissociated KI or
//! FeCl3 from this step's addition, not only the bare ions a previous
//! aqueous readback left behind — and the triiodide split behind an iodine
//! absorbance must be reported with the reading.
//!
//! `iodide_redox_coverage_is_current` holds the runtime to the generated
//! table `docs/iodide-redox-coverage-20261111.json`
//! (`python3 tools/iodide_redox_coverage.py` regenerates it).

use kerotakis_core::halogen_redox::{self, PairStatus};
use kerotakis_core::iodine_chemistry as iodine;
use kerotakis_core::solve::Equilibrator;
use kerotakis_core::*;

const TABLE: &str = include_str!("../../../docs/iodide-redox-coverage-20261111.json");

#[test]
fn iodide_redox_coverage_is_current() {
    let doc: serde_json::Value = serde_json::from_str(TABLE).unwrap();
    assert_eq!(
        doc["e0_i2aq_iodide_v"].as_f64().unwrap(),
        iodine::E0_I2AQ_IODIDE_V
    );
    let rows = doc["pairs"].as_array().unwrap();
    let runtime: Vec<_> = halogen_redox::pairs()
        .iter()
        .filter(|p| p.reductant == "I2/I-")
        .collect();
    assert_eq!(rows.len(), runtime.len());
    for (row, pair) in rows.iter().zip(runtime) {
        assert_eq!(row["oxidant"], pair.oxidant);
        assert_eq!(row["halogen_status"], pair.status.key(), "{}", pair.oxidant);
        let solved = iodine::solves_pair(pair.oxidant, pair.reductant);
        assert_eq!(
            row["handling"] == "equilibrium_solver",
            solved,
            "{}",
            pair.oxidant
        );
        if solved {
            let half = halogen_redox::half_by_id(pair.oxidant).unwrap();
            assert!(
                (row["log_k"].as_f64().unwrap() - iodine::couple_log_k(half)).abs() < 1e-4,
                "{}",
                pair.oxidant
            );
        }
        if row["handling"] == "complete_writer" {
            assert_eq!(pair.status, PairStatus::Admitted);
        }
    }
    // Fe³⁺/Fe²⁺ is the solved couple on the current table.
    assert!(iodine::solves_pair("Fe+3/Fe+2", "I2/I-"));
    assert!(!iodine::solves_pair("IO3-/I-", "I2/I-"));
}

fn water(ml: f64) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("water"), Moles(ml / 18.015), Phase::Liquid);
    v
}

fn aq(v: &mut Vessel, key: &str, n: f64) {
    v.deposit(SpeciesId::new(key), Moles(n), Phase::Aqueous);
}

fn m(v: &Vessel, key: &str) -> f64 {
    v.contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == Phase::Aqueous)
        .map(|p| p.moles.0)
        .sum()
}

fn react(v: &mut Vessel) -> Vec<String> {
    v.solute_charge = displacement::solute_charge(v);
    CuratedEquilibrator
        .equilibrate(v)
        .unwrap()
        .into_iter()
        .filter_map(|e| match e {
            Event::ReactionOccurred { equation, .. } => Some(equation),
            _ => None,
        })
        .collect()
}

/// Element totals over every portion except water (formula-weighted).
fn element(v: &Vessel, el: &str) -> f64 {
    v.contents
        .iter()
        .filter(|p| p.species.0 != "water")
        .filter_map(|p| {
            let data = species::lookup(&p.species)?;
            let f = stoich::parse_formula(data.formula).ok()?;
            Some(p.moles.0 * f.counts.get(el).copied().unwrap_or(0.0))
        })
        .sum()
}

fn net_charge(v: &Vessel) -> f64 {
    v.contents
        .iter()
        .filter_map(|p| {
            let data = species::lookup(&p.species)?;
            let f = stoich::parse_formula(data.formula).ok()?;
            Some(p.moles.0 * f.charge)
        })
        .sum()
}

#[test]
fn salt_forms_are_derived_from_the_registry() {
    let ferric = iodine::ion_form_keys("Fe+3");
    assert_eq!(ferric[0], "Fe+3");
    assert!(ferric.iter().any(|k| k == "FeCl3"), "{ferric:?}");
    assert!(!ferric.iter().any(|k| k == "FeSO4"), "{ferric:?}");
    let ferrous = iodine::ion_form_keys("Fe+2");
    assert!(ferrous.iter().any(|k| k == "FeSO4"), "{ferrous:?}");
    assert!(!ferrous.iter().any(|k| k == "FeCl3"), "{ferrous:?}");
    let iodide = iodine::ion_form_keys("I-");
    assert!(iodide.iter().any(|k| k == "KI"), "{iodide:?}");
}

#[test]
fn ferric_ion_oxidises_undissociated_potassium_iodide() {
    // RX-004 at the KI step: Fe³⁺ already read back as the ion, KI still
    // the dissolved salt the aqueous route has not yet split.
    let mut v = water(100.0);
    aq(&mut v, "Fe+3", 4e-4);
    aq(&mut v, "Cl-", 2.2e-3);
    aq(&mut v, "H+", 1e-3);
    aq(&mut v, "KI", 4e-4);
    let (fe, i, k, cl, q) = (
        element(&v, "Fe"),
        element(&v, "I"),
        element(&v, "K"),
        element(&v, "Cl"),
        net_charge(&v),
    );
    assert_eq!(react(&mut v), vec!["2 Fe³⁺ + 2 I⁻ → 2 Fe²⁺ + I₂"]);
    let fe2 = m(&v, "Fe+2");
    let i2 = m(&v, "I2");
    assert!(fe2 > 0.5 * 4e-4, "{fe2}");
    assert!((fe2 - 2.0 * i2).abs() < 1e-16, "electron closure");
    // The reacted share of KI left its potassium as K⁺.
    assert!((m(&v, "K+") + m(&v, "KI") - 4e-4).abs() < 1e-16);
    assert!(m(&v, "K+") > 0.0);
    for (name, before, after) in [
        ("Fe", fe, element(&v, "Fe")),
        ("I", i, element(&v, "I")),
        ("K", k, element(&v, "K")),
        ("Cl", cl, element(&v, "Cl")),
        ("charge", q, net_charge(&v)),
    ] {
        assert!(
            (before - after).abs() < 1e-15,
            "{name}: {before} -> {after}"
        );
    }
    // At K with the triiodide split.
    let litres = v.liquid_volume().0;
    let s = iodine::split(i2, iodine::dissolved_iodide(&v), litres);
    let fe3 = m(&v, "Fe+3");
    let q = (fe2 / litres).powi(2) * (s.iodine / litres)
        / ((fe3 / litres).powi(2) * (s.iodide / litres).powi(2));
    assert!(
        (q.log10() - iodine::ferric_iodide_log_k()).abs() < 1e-6,
        "{q}"
    );
    assert!(react(&mut v).is_empty(), "idempotent at equilibrium");
}

#[test]
fn undissociated_ferric_chloride_oxidises_iodide_and_frees_its_chloride() {
    let mut v = water(100.0);
    aq(&mut v, "I-", 1e-3);
    aq(&mut v, "K+", 1e-3);
    aq(&mut v, "FeCl3", 4e-4);
    let (fe, cl, q) = (element(&v, "Fe"), element(&v, "Cl"), net_charge(&v));
    assert_eq!(react(&mut v), vec!["2 Fe³⁺ + 2 I⁻ → 2 Fe²⁺ + I₂"]);
    let fe2 = m(&v, "Fe+2");
    assert!(fe2 > 0.5 * 4e-4, "{fe2}");
    // Every reacted FeCl3 released three Cl⁻.
    assert!((m(&v, "Cl-") - 3.0 * (4e-4 - m(&v, "FeCl3"))).abs() < 1e-16);
    assert!((element(&v, "Fe") - fe).abs() < 1e-16);
    assert!((element(&v, "Cl") - cl).abs() < 1e-16);
    assert!((net_charge(&v) - q).abs() < 1e-15);
}

#[test]
fn ferrous_sulfate_and_iodide_do_not_react() {
    // EC-008's ferrous arm: same iron total, no oxidising power.
    let mut v = water(100.0);
    aq(&mut v, "FeSO4", 4e-4);
    aq(&mut v, "KI", 1e-3);
    assert!(react(&mut v).is_empty());
    assert_eq!(m(&v, "I2"), 0.0);
    assert_eq!(m(&v, "Fe+3"), 0.0);
}

#[test]
fn iodine_and_ferrous_sulfate_run_back_to_the_same_constant() {
    let mut v = water(100.0);
    aq(&mut v, "FeSO4", 4e-4);
    aq(&mut v, "I2", 2e-4);
    let q0 = net_charge(&v);
    assert_eq!(react(&mut v), vec!["2 Fe²⁺ + I₂ → 2 Fe³⁺ + 2 I⁻"]);
    let fe3 = m(&v, "Fe+3");
    assert!(fe3 > 0.0);
    assert!((m(&v, "SO4-2") - (4e-4 - m(&v, "FeSO4"))).abs() < 1e-16);
    assert!((element(&v, "Fe") - 4e-4).abs() < 1e-16);
    assert!((element(&v, "I") - 4e-4).abs() < 1e-16);
    assert!((net_charge(&v) - q0).abs() < 1e-15);
}

#[test]
fn ferric_iodide_outside_the_medium_is_named_not_silent() {
    let mut v = water(100.0);
    aq(&mut v, "Fe+3", 4e-4);
    aq(&mut v, "I-", 1e-3);
    v.temperature = Kelvin(353.15);
    assert!(react(&mut v).is_empty());
    let keys: Vec<String> = halogen_redox::unpriced_notes(&v)
        .into_iter()
        .filter_map(|e| match e {
            Event::NotYetModeled {
                reason: Some(r), ..
            } => Some(r.key.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(keys, vec!["not-modeled.redox-pair-outside-medium"]);
}

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(CuratedEquilibrator),
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

#[test]
fn bench_ferric_chloride_then_potassium_iodide_forms_ferrous_iron_and_iodine() {
    for order in [["FeCl3", "KI"], ["KI", "FeCl3"]] {
        let mut bench = Bench::new();
        let mut s = stack();
        add(&mut bench, &mut s, "water", 5.534);
        add(&mut bench, &mut s, order[0], 4e-4);
        let events = add(&mut bench, &mut s, order[1], 4e-4);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::ReactionOccurred { equation, .. }
                if equation == "2 Fe³⁺ + 2 I⁻ → 2 Fe²⁺ + I₂")),
            "{order:?}: {events:?}"
        );
        let v = bench.vessel(VesselId(0)).unwrap();
        assert!(m(v, "Fe+2") > 0.0, "{order:?}");
        assert!(m(v, "I2") > 0.0, "{order:?}");
        assert!((element(v, "Fe") - 4e-4).abs() < 1e-15, "{order:?}");
        assert!((element(v, "I") - 4e-4).abs() < 1e-15, "{order:?}");
    }
}

fn measure(v: Vessel) -> Vec<Event> {
    let mut bench = Bench::new();
    bench.vessels[0] = v;
    bench
        .step_with(
            Operator::Measure {
                vessel: VesselId(0),
                instrument: Instrument::Spectrophotometer,
            },
            &mut SolverStack::new(vec![]),
            &PermissiveScreen,
        )
        .unwrap()
}

#[test]
fn absorbance_reading_names_the_triiodide_split_it_integrates() {
    // MO-012: I2 in KI. The reading and its I3⁻ speciation come from one
    // function, so I3⁻ is reported wherever the colour assumes it.
    let mut v = water(100.0);
    aq(&mut v, "KI", 1e-3);
    aq(&mut v, "I2", 1e-5);
    let split = iodine::optical_split(&v).expect("dissolved iodine");
    assert!(split.triiodide > 0.0 && split.iodine > 0.0);
    assert!((split.iodine + split.triiodide - 1e-5).abs() < 1e-18);
    let events = measure(v);
    let reason = events
        .iter()
        .find_map(|e| match e {
            Event::Measured {
                note_reason: Some(r),
                note: Some(_),
                ..
            } => Some(r.clone()),
            _ => None,
        })
        .expect("reading with its iodine speciation");
    assert_eq!(reason.key, "measured.iodine-triiodide-split");
    let en = reason.render(Locale::EN);
    assert!(en.contains(&format!("{:.4e}", split.triiodide)), "{en}");
    assert!(en.contains("I₃⁻"), "{en}");
    for locale in [Locale::parse("de"), Locale::parse("fr")] {
        assert_ne!(reason.render(locale), en);
    }
}

#[test]
fn reduced_iodine_reading_carries_no_iodine_note() {
    let mut v = water(100.0);
    aq(&mut v, "KI", 1e-3);
    assert!(iodine::optical_split(&v).is_none());
    let events = measure(v);
    assert!(events.iter().any(|e| matches!(
        e,
        Event::Measured {
            note_reason: None,
            ..
        }
    )));
}
