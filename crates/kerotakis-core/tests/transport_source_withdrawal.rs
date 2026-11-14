//! Every inter-vessel transfer withdraws from its source exactly what it
//! credits to its destination (TR-006 follow-up).
//!
//! The formal review of batch3-diverse:tr-006 summed every vessel after two
//! `transport` commands and found the dye, Na and Cl doubled and 4.43 mol of
//! extra water: the script verb's default inlet was an undebited
//! constant-composition reservoir, so a vessel on the bench supplied matter
//! and kept it. The script default is now a finite feed (the inlet vessel is
//! debited by exactly what enters the column); only an explicit trailing
//! `reservoir` keeps the external boundary, whose supply the event reports
//! as a source term.
//!
//! The family claim held here, for every inter-vessel operator this bench
//! runs without a chemistry engine: the bench-wide per-element ledger and
//! the summed charge are unchanged by the transfer, and what the source lost
//! equals what the destinations gained. `tools/transport_source_withdrawal_coverage.py`
//! keeps the operator inventory and points each row at its test.

use std::collections::BTreeMap;

use kerotakis_core::ledger::ConservedLedger;
use kerotakis_core::ops::{Event, Operator};
use kerotakis_core::script::parse_op;
use kerotakis_core::species::{Phase, SpeciesId};
use kerotakis_core::transport::TransportError;
use kerotakis_core::units::{Kelvin, Moles};
use kerotakis_core::vessel::{Vessel, VesselId};
use kerotakis_core::{Bench, BenchError, Equilibrator, Locale, Refuses, SolveError};

/// 20 mL of water, the volume tr-006 gave every vessel.
const WATER_20ML: f64 = 1.1068553982792118;

struct Noop;

impl Equilibrator for Noop {
    fn name(&self) -> &'static str {
        "noop-test"
    }
    fn equilibrate(&mut self, _vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        Ok(Vec::new())
    }
}

fn water(vessel: &mut Vessel, volumes: f64) {
    vessel.deposit(
        SpeciesId::new("water"),
        Moles(WATER_20ML * volumes),
        Phase::Liquid,
    );
    vessel.temperature = Kelvin::STANDARD;
}

/// `count` vessels, all empty.
fn bench_of(count: usize) -> Bench {
    let mut bench = Bench::new();
    for _ in 1..count {
        bench.step(Operator::NewVessel { kind: None }).unwrap();
    }
    bench
}

/// The tr-006 column without the sorbent: v1..v3 water cells, v4 a salt
/// feed of `feed_volumes` cell volumes, v5 an empty receiver.
fn column(feed_volumes: f64) -> Bench {
    let mut bench = bench_of(5);
    for vessel in &mut bench.vessels[..3] {
        water(vessel, 1.0);
    }
    water(&mut bench.vessels[3], feed_volumes);
    bench.vessels[3].deposit(SpeciesId::new("NaCl"), Moles(1e-4), Phase::Aqueous);
    bench
}

fn step(bench: &mut Bench, line: &str) -> Result<Vec<Event>, BenchError> {
    let op = parse_op(line).unwrap().expect("an operator");
    bench.step_with(op, &mut Noop, &kerotakis_core::PermissiveScreen)
}

fn ledger(vessels: &[&Vessel]) -> (BTreeMap<String, f64>, f64) {
    let mut elements = BTreeMap::new();
    let mut charge = 0.0;
    for vessel in vessels {
        let one = ConservedLedger::from_vessel(vessel);
        for (element, moles) in one.elements {
            *elements.entry(element).or_insert(0.0) += moles;
        }
        charge += one.charge + vessel.solute_charge;
    }
    (elements, charge)
}

fn whole(bench: &Bench) -> (BTreeMap<String, f64>, f64) {
    ledger(&bench.vessels.iter().collect::<Vec<_>>())
}

fn assert_same(before: &(BTreeMap<String, f64>, f64), after: &(BTreeMap<String, f64>, f64)) {
    for element in before.0.keys().chain(after.0.keys()) {
        let b = before.0.get(element).copied().unwrap_or(0.0);
        let a = after.0.get(element).copied().unwrap_or(0.0);
        assert!(
            (a - b).abs() <= 1e-12 * b.abs().max(1e-12),
            "{element}: {a:e} after vs {b:e} before"
        );
    }
    assert!(
        (after.1 - before.1).abs() <= 1e-15,
        "charge: {:e} after vs {:e} before",
        after.1,
        before.1
    );
}

/// `source lost == destinations gained`, per element.
fn assert_debit_equals_credit(
    before: &Bench,
    after: &Bench,
    source: usize,
    destinations: &[usize],
) {
    let lost = |element: &str| {
        ledger(&[&before.vessels[source]])
            .0
            .get(element)
            .copied()
            .unwrap_or(0.0)
            - ledger(&[&after.vessels[source]])
                .0
                .get(element)
                .copied()
                .unwrap_or(0.0)
    };
    let gained = |element: &str| -> f64 {
        destinations
            .iter()
            .map(|d| {
                ledger(&[&after.vessels[*d]])
                    .0
                    .get(element)
                    .copied()
                    .unwrap_or(0.0)
                    - ledger(&[&before.vessels[*d]])
                        .0
                        .get(element)
                        .copied()
                        .unwrap_or(0.0)
            })
            .sum()
    };
    let (elements, _) = whole(before);
    let mut moved = false;
    for element in elements.keys() {
        let (l, g) = (lost(element), gained(element));
        assert!(
            (l - g).abs() <= 1e-12 * l.abs().max(g.abs()).max(1e-12),
            "{element}: source lost {l:e}, destinations gained {g:e}"
        );
        moved |= l > 0.0;
    }
    assert!(moved, "the transfer must move something");
}

#[test]
fn transport_script_default_is_a_finite_feed_and_reservoir_is_explicit() {
    let parse = |line: &str| match parse_op(line).unwrap().unwrap() {
        Operator::Transport { drain, .. } => drain,
        other => panic!("{line}: {other:?}"),
    };
    assert!(parse(
        "transport v1 v2 v3 from v4 to v5 steps 2 courant 0.5"
    ));
    assert!(parse("transport v1 from v2 to v3 steps 1"));
    assert!(parse(
        "transport v1 from v2 to v3 steps 1 courant 0.5 drain"
    ));
    assert!(!parse(
        "transport v1 from v2 to v3 steps 1 courant 0.5 reservoir"
    ));
    assert!(!parse("transport v1 from v2 to v3 steps 1 reservoir"));
    for malformed in [
        "transport v1 from v2 to v3 steps 1 reservoir drain",
        "transport v1 from v2 to v3 steps 1 drain reservoir",
        "transport v1 from v2 to v3 steps 1 reservoir courant 0.5",
    ] {
        assert!(
            parse_op(malformed).is_err(),
            "silently accepted: {malformed}"
        );
    }
}

/// The tr-006 first command, verbatim: the inlet is debited by exactly the
/// recorded inflow, the cells and receiver gain exactly that, and the bench
/// as a whole is unchanged per element and in charge.
#[test]
fn transport_default_debits_inlet_by_exactly_the_destination_credit() {
    let mut bench = column(1.0);
    let before = bench.clone();
    let events = step(
        &mut bench,
        "transport v1 v2 v3 from v4 to v5 steps 2 courant 0.5",
    )
    .unwrap();
    assert_same(&whole(&before), &whole(&bench));
    assert_debit_equals_credit(&before, &bench, 3, &[0, 1, 2, 4]);
    let Some(Event::Transported {
        drain,
        inflow_moles,
        inlet,
        ..
    }) = events
        .iter()
        .find(|e| matches!(e, Event::Transported { .. }))
    else {
        panic!("transport must report its ledger: {events:?}");
    };
    assert!(*drain);
    assert_eq!(*inlet, Some(VesselId(3)));
    for (species, moles) in inflow_moles {
        let lost = before.vessels[3].moles_of(species).0 - bench.vessels[3].moles_of(species).0;
        assert!(
            (lost - moles.0).abs() <= 1e-12 * moles.0.max(1e-30),
            "{}: inlet lost {lost:e}, event reports {:e}",
            species.0,
            moles.0
        );
    }
}

/// A command asking a finite feed for more than it holds is refused as a
/// whole, nothing moves, and the refusal has its own key in EN/DE/FR.
#[test]
fn exhausted_feed_is_a_typed_localized_refusal_and_moves_nothing() {
    let mut bench = column(1.0);
    let before = bench.clone();
    let error = step(
        &mut bench,
        "transport v1 v2 v3 from v4 to v5 steps 6 courant 0.5",
    )
    .unwrap_err();
    assert!(matches!(
        error,
        BenchError::Transport(TransportError::InletExhausted { step: 3, .. })
    ));
    assert_eq!(error.refusal().key, "error.transport-feed-exhausted");
    let english = error.localize(Locale::EN);
    assert!(
        english.contains("reservoir") && english.contains("step 3"),
        "{english}"
    );
    for code in ["de", "fr"] {
        let localized = error.localize(Locale::parse(code));
        assert_ne!(localized, english, "{code} fell back to English");
        assert!(!localized.contains('{'), "unfilled hole: {localized}");
        assert!(
            localized.contains('3') && localized.contains("reservoir"),
            "{localized}"
        );
    }
    for (a, b) in before.vessels.iter().zip(&bench.vessels) {
        assert_eq!(a.contents, b.contents, "a refused command moved matter");
    }
}

/// Negative control: an explicitly declared reservoir is NOT debited, and
/// the bench grows by exactly the source term the event reports.
#[test]
fn declared_reservoir_keeps_inlet_and_reports_exact_source_term() {
    let mut bench = column(1.0);
    let before = bench.clone();
    let events = step(
        &mut bench,
        "transport v1 v2 v3 from v4 to v5 steps 6 courant 0.5 reservoir",
    )
    .unwrap();
    assert_eq!(before.vessels[3].contents, bench.vessels[3].contents);
    let Some(Event::Transported {
        drain: false,
        inflow_moles,
        ..
    }) = events
        .iter()
        .find(|e| matches!(e, Event::Transported { .. }))
    else {
        panic!("a declared reservoir must say so: {events:?}");
    };
    let mut source = Vessel::new(VesselId(99), "source term");
    for (species, moles) in inflow_moles {
        let phase = if species.0 == "water" {
            Phase::Liquid
        } else {
            Phase::Aqueous
        };
        source.deposit(species.clone(), *moles, phase);
    }
    let mut expected = whole(&before);
    for (element, moles) in ConservedLedger::from_vessel(&source).elements {
        *expected.0.entry(element).or_insert(0.0) += moles;
    }
    assert_same(&expected, &whole(&bench));
}

#[test]
fn decant_debits_source_by_exactly_the_destination_credit() {
    let mut bench = bench_of(2);
    water(&mut bench.vessels[0], 1.0);
    bench.vessels[0].deposit(SpeciesId::new("NaCl"), Moles(1e-4), Phase::Aqueous);
    let before = bench.clone();
    step(&mut bench, "decant v1 v2 0.4").unwrap();
    assert_same(&whole(&before), &whole(&bench));
    assert_debit_equals_credit(&before, &bench, 0, &[1]);
}

#[test]
fn filter_debits_source_by_exactly_the_filtrate_credit() {
    let mut bench = bench_of(2);
    water(&mut bench.vessels[0], 1.0);
    bench.vessels[0].deposit(SpeciesId::new("NaCl"), Moles(1e-4), Phase::Aqueous);
    bench.vessels[0].deposit(SpeciesId::new("CaCO3"), Moles(1e-3), Phase::Solid);
    let before = bench.clone();
    step(&mut bench, "filter v1 v2").unwrap();
    assert_same(&whole(&before), &whole(&bench));
    assert_debit_equals_credit(&before, &bench, 0, &[1]);
}

#[test]
fn magnet_debits_source_by_exactly_the_attracted_credit() {
    let mut bench = bench_of(2);
    water(&mut bench.vessels[0], 1.0);
    bench.vessels[0].deposit(SpeciesId::new("Fe"), Moles(1e-3), Phase::Solid);
    let before = bench.clone();
    step(&mut bench, "magnet v1 v2").unwrap();
    assert_same(&whole(&before), &whole(&bench));
    assert_debit_equals_credit(&before, &bench, 0, &[1]);
}

#[test]
fn mix_debits_both_sources_by_exactly_the_receiver_credit() {
    let mut bench = bench_of(3);
    water(&mut bench.vessels[0], 1.0);
    bench.vessels[0].deposit(SpeciesId::new("NaCl"), Moles(1e-4), Phase::Aqueous);
    water(&mut bench.vessels[1], 1.0);
    let before = bench.clone();
    step(&mut bench, "mix v1 0.5 v2 0.25 into v3").unwrap();
    assert_same(&whole(&before), &whole(&bench));
    // Chlorine has a single source here, so its debit and credit pair up
    // exactly; the whole-bench check above covers the shared water.
    let cl = |b: &Bench, v: usize| ledger(&[&b.vessels[v]]).0.get("Cl").copied().unwrap_or(0.0);
    let lost = cl(&before, 0) - cl(&bench, 0);
    let gained = cl(&bench, 2) - cl(&before, 2);
    assert!(lost > 0.0 && (lost - gained).abs() <= 1e-12 * lost);
}
