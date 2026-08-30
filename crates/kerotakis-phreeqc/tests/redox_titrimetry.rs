//! EXP-39's acceptance: the classic standardisation, run by the engine.
//!
//! A burette of potassium permanganate against a weighed primary
//! standard of oxalic acid in dilute sulfuric acid. Two endpoints, one
//! flask, and the same answer from both — which is the point of having
//! two: a self-indicating endpoint that a child can see and a
//! potentiometric one that a meter can read must agree, or one of them
//! is lying.

#![cfg(feature = "engine")]

use kerotakis_core::script::parse_op;
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

/// 0.0002 mol of oxalic acid needs 0.00008 mol of permanganate — two for
/// five — which a 0.02 mol/L burette delivers in exactly 4.00 mL.
const ANALYTE_MOL: f64 = 0.0002;
const BURETTE_M: f64 = 0.02;
const STOICHIOMETRIC_ML: f64 = 4.0;
/// One drop from a burette. The claim is that the endpoint lands within
/// one of these of the stoichiometric volume, which is the same claim a
/// practical makes about its own titration.
const ONE_DROP_ML: f64 = 0.05;

fn add(bench: &mut Bench, stack: &mut SolverStack, v: VesselId, key: &str, moles: f64) {
    bench
        .step_with(
            Operator::Add {
                vessel: v,
                species: SpeciesId::new(key),
                moles: Moles(moles),
                at: None,
            },
            stack,
            &PermissiveScreen,
        )
        .expect("step");
}

fn stack() -> SolverStack {
    let aqueous = PhreeqcEquilibrator::new().expect("engine");
    SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(aqueous)]))
}

/// The flask as the practical sets it up: the standard, the acid, and
/// enough water to see through.
fn flask(bench: &mut Bench, stack: &mut SolverStack) -> VesselId {
    let v = VesselId(0);
    add(bench, stack, v, "water", 5.55);
    add(bench, stack, v, "H2SO4", 0.002);
    add(bench, stack, v, "H2C2O4", ANALYTE_MOL);
    v
}

/// What a titration reports back. A named struct rather than a tuple:
/// clippy's `type_complexity` is right that four anonymous fields is one
/// too many to read.
struct Run {
    ml: f64,
    reached: bool,
    pe_curve: Vec<(f64, f64)>,
}

fn titrated(events: &[Event]) -> Option<Run> {
    events.iter().find_map(|e| match e {
        Event::Titrated {
            total_volume,
            endpoint_reached,
            pe_curve,
            ..
        } => Some(Run {
            ml: total_volume.0 * 1000.0,
            reached: endpoint_reached.unwrap_or(false),
            pe_curve: pe_curve.clone(),
        }),
        _ => None,
    })
}

fn run(line: &str) -> (Vec<Event>, Bench) {
    let mut bench = Bench::new();
    let mut s = stack();
    flask(&mut bench, &mut s);
    let events = bench
        .step_with(
            parse_op(line).expect("parse").expect("operator"),
            &mut s,
            &PermissiveScreen,
        )
        .expect("titrate");
    (events, bench)
}

// ── The self-indicating endpoint ────────────────────────────────────

/// The endpoint every school textbook describes: the first drop of
/// permanganate that is not decolourised leaves a permanent pink.
///
/// Nothing here declares a visibility threshold. The colour comes out of
/// the registry's ε(λ) for MnO4-, Beer-Lambert over the flask's own path
/// length and the CIE observer — the same pipeline that paints the
/// bench — and the endpoint is where the word for the liquid stops being
/// the word it was before the burette was opened.
#[test]
fn permanganate_standardised_against_oxalic_acid_stops_within_one_drop() {
    let (events, bench) = run("titrate v1 KMnO4 0.02M 0.05mL until colour persists max 150");
    let run = titrated(&events).unwrap_or_else(|| panic!("{events:?}"));
    let (ml, reached) = (run.ml, run.reached);
    assert!(reached, "the endpoint must be reached: {events:?}");
    assert!(
        (ml - STOICHIOMETRIC_ML).abs() <= ONE_DROP_ML + 1e-9,
        "endpoint at {ml:.3} mL, stoichiometric {STOICHIOMETRIC_ML:.3} mL \
         (increments of {ONE_DROP_ML} mL)"
    );

    // The value claim, spelled the way a practical spells it: the
    // burette's concentration recovered from the volume it took.
    let recovered = 0.4 * ANALYTE_MOL / (ml / 1000.0);
    assert!(
        (recovered - BURETTE_M).abs() / BURETTE_M < 0.02,
        "standardisation recovers {recovered:.5} mol/L against a {BURETTE_M} mol/L burette"
    );

    // And the flask really is pink: the endpoint is a statement about
    // the vessel, not a counter in the loop.
    let seen = appearance::observe(bench.vessel(VesselId(0)).expect("vessel"));
    assert!(
        !seen.words.contains("colourless"),
        "the flask must be visibly coloured at the endpoint: {}",
        seen.words
    );
}

/// Below the endpoint the flask stays colourless, which is the other
/// half of the same claim: a titration that were pink from the first
/// drop would "reach" its endpoint immediately and report a nonsense
/// volume.
#[test]
fn the_flask_is_colourless_up_to_the_endpoint() {
    let (events, bench) = run("titrate v1 KMnO4 0.02M 0.05mL until colour persists max 60");
    let reached = titrated(&events)
        .unwrap_or_else(|| panic!("{events:?}"))
        .reached;
    assert!(
        !reached,
        "60 increments is 3.00 mL, still short of 4.00 mL: {events:?}"
    );
    let seen = appearance::observe(bench.vessel(VesselId(0)).expect("vessel"));
    assert!(
        seen.words.contains("colourless"),
        "before equivalence every drop is decolourised: {}",
        seen.words
    );
}

// ── The potentiometric endpoint ─────────────────────────────────────

/// An open flask pins its potential from the air, not from the couple —
/// and that is a finding about the bench, not a bug in the endpoint.
///
/// The engine equilibrates an open vessel against atmospheric oxygen
/// (`ATMOSPHERIC_LOG_PO2`, log10 0.21), and the O2/H2O couple is a far
/// stronger buffer than a trace of manganese. So a pe is pinned from the
/// very first increment and it is already high: `until pe > 5` is
/// satisfied at the first drop, nowhere near the 4.00 mL equivalence.
///
/// The honest reading is not "the potentiometric endpoint is broken". It
/// is that a thermodynamic model of a beaker standing open reports the
/// potential of the *beaker*, which is the oxygen's — and a real
/// potentiometric titration excludes air for exactly that reason. The
/// test pins the behaviour so it cannot change silently, and prints the
/// curve so the number is on the record rather than in a commit message.
#[test]
fn an_open_flask_pins_its_potential_from_the_air() {
    let (events, _) = run("titrate v1 KMnO4 0.02M 0.05mL until pe > 5 max 150");
    let run_result = titrated(&events).unwrap_or_else(|| panic!("{events:?}"));
    assert!(run_result.reached, "a potential is pinned: {events:?}");
    let first = *run_result
        .pe_curve
        .first()
        .expect("a potential from the first increment");
    eprintln!("open-flask pe curve: {:?}", run_result.pe_curve);
    assert!(
        first.1 > 15.0,
        "the air, not the couple, sets it — expected a high pe, got {first:?}"
    );
    assert!(
        run_result.ml < 1.0,
        "an oxygen-buffered flask satisfies `pe > 5` long before equivalence \
         ({:.3} mL against {STOICHIOMETRIC_ML:.2} mL); if this ever lands near \
         equivalence the atmospheric coupling has changed and the claim above \
         needs rewriting",
        run_result.ml
    );
}

/// Sweeping the air out does not hand the potential to the couple — it
/// hands back PHREEQC's default. This is EXP-39's reported gap, pinned.
///
/// With the atmosphere removed the only redox element left is manganese,
/// and the curated row puts all of it at Mn(II) until equivalence. One
/// oxidation state is not a couple: there is nothing for the electron
/// balance to bracket. What comes back is not a withheld pe, which is
/// what `redox.rs::the_equivalence_point_reports_no_potential` shows the
/// engine can do — it is a flat 4.0 at every one of 150 increments,
/// which is the pe of the *input*, republished as an answer.
///
/// So the potentiometric endpoint is wired, honest, and unusable on this
/// system: open, the flask reads its oxygen (~19.0, flat); swept, it
/// reads the default (4.0, flat). Neither is a titration curve. The
/// endpoint mode is right; what is missing is an engine that withholds
/// pe when nothing determines it, exactly as it already does at an
/// equivalence point. This test exists so that the day the engine stops
/// republishing its input, it fails and says so.
#[test]
fn a_swept_flask_reports_the_default_pe_rather_than_the_couple() {
    let mut bench = Bench::new();
    let mut s = stack();
    let v = flask(&mut bench, &mut s);
    bench
        .step_with(
            Operator::Sweep {
                vessel: v,
                pressure: kerotakis_core::units::Pascal(101_325.0),
            },
            &mut s,
            &PermissiveScreen,
        )
        .expect("sweep");
    let events = bench
        .step_with(
            Operator::Titrate {
                vessel: v,
                titrant: SpeciesId::new("KMnO4"),
                concentration: BURETTE_M,
                step: kerotakis_core::units::Liters(ONE_DROP_ML / 1000.0),
                target_ph: 7.0,
                max_steps: 150,
                // A threshold nothing can reach, so the burette walks the
                // whole curve and the shape of it is what is reported.
                endpoint: kerotakis_core::ops::Endpoint::Pe {
                    compare: kerotakis_core::ops::Compare::Above,
                    value: 1.0e6,
                },
            },
            &mut s,
            &PermissiveScreen,
        )
        .expect("titrate");
    let run = titrated(&events).unwrap_or_else(|| panic!("{events:?}"));
    assert!(!run.reached, "1e6 is not a reachable potential");
    assert!(run.pe_curve.len() > 100, "the burette must walk the curve");
    let spread = run
        .pe_curve
        .iter()
        .map(|(_, pe)| *pe)
        .fold(f64::NEG_INFINITY, f64::max)
        - run
            .pe_curve
            .iter()
            .map(|(_, pe)| *pe)
            .fold(f64::INFINITY, f64::min);
    assert!(
        spread < 1e-9,
        "the swept curve is flat today; a spread of {spread} means the \
         couple has started setting the potential and this test's whole \
         story needs rewriting (gladly)"
    );
    assert!(
        (run.pe_curve[0].1 - 4.0).abs() < 1e-6,
        "and flat at PHREEQC's input default of pe 4, not at a computed \
         value: got {}",
        run.pe_curve[0].1
    );

    // The refusal still tells the truth about it: a potential *was*
    // pinned, so the reason must be the target, not the couple.
    let why = events
        .iter()
        .find_map(|e| match e {
            Event::NotYetModeled { what, .. } if what.contains("pe") => Some(what.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("a reason was owed: {events:?}"));
    assert!(
        why.contains("last potential this flask pinned"),
        "the flask did pin a potential; the refusal must not claim otherwise: {why}"
    );
}

/// The curve carries the potential additively, and the chart contract
/// picks it up as its own claim rather than a second series on a pH axis.
#[test]
fn the_pe_curve_reaches_the_chart_contract() {
    let (events, _) = run("titrate v1 KMnO4 0.02M 0.5mL until pe > 5 max 20");
    let pe_curve = titrated(&events)
        .unwrap_or_else(|| panic!("{events:?}"))
        .pe_curve;
    let charts = chart::charts_for_events(&events);
    // The pH chart is unconditional and unchanged.
    assert!(
        charts.iter().any(|c| c.y.label == "pH"),
        "the pH curve is still a chart: {charts:?}"
    );
    let redox = charts.iter().find(|c| c.y.label == "pe");
    if pe_curve.len() < 2 {
        // A flask that pinned a potential fewer than twice has a
        // reading, not a curve. The contract must claim no picture —
        // which is the assertion, not a skip.
        assert!(
            redox.is_none(),
            "{} pinned potentials must not become a chart: {charts:?}",
            pe_curve.len()
        );
        return;
    }
    let redox = redox.unwrap_or_else(|| panic!("a pe chart was earned: {charts:?}"));
    assert_eq!(redox.x.unit.as_deref(), Some("mL"));
    assert_eq!(redox.series[0].points().len(), pe_curve.len());
    assert!(!redox.provenance.is_empty());
}

// ── Refusals ────────────────────────────────────────────────────────

/// A potentiometric endpoint in a flask with no redox couple at all.
///
/// This is the failure that most deserves an explanation: pe is not low
/// here, it is *undefined* — there are no electrons to balance — and
/// reporting "pe never got high enough" would invent a measurement to
/// explain a missing one.
#[test]
fn a_potentiometric_endpoint_without_a_couple_refuses_and_says_why() {
    let mut bench = Bench::new();
    let mut s = stack();
    let v = VesselId(0);
    add(&mut bench, &mut s, v, "water", 5.55);
    add(&mut bench, &mut s, v, "HCl", 0.01);
    let events = bench
        .step_with(
            parse_op("titrate v1 NaOH 0.1M 1mL until pe > 8 max 5")
                .expect("parse")
                .expect("operator"),
            &mut s,
            &PermissiveScreen,
        )
        .expect("titrate");
    let run = titrated(&events).unwrap_or_else(|| panic!("{events:?}"));
    let (reached, pe_curve) = (run.reached, run.pe_curve);
    assert!(!reached, "there is no endpoint to reach here");
    assert!(
        pe_curve.is_empty(),
        "and no potential to report: {pe_curve:?}"
    );
    let refusal = events
        .iter()
        .find_map(|e| match e {
            Event::NotYetModeled { what, .. } if what.contains("pe") => Some(what.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("a reason was owed: {events:?}"));
    assert!(
        refusal.contains("undefined"),
        "the refusal must distinguish undefined from low: {refusal}"
    );
}

/// CAP-12's pH endpoint, unchanged, on the flask it was written for. The
/// point of the test is the *absence* of change: EXP-39 must not have
/// moved the equivalence point of an acid-base titration by a step.
#[test]
fn the_ph_endpoint_still_finds_the_equivalence_point() {
    let mut bench = Bench::new();
    let mut s = stack();
    let v = VesselId(0);
    add(&mut bench, &mut s, v, "water", 27.75);
    add(&mut bench, &mut s, v, "HCl", 0.01);
    let events = bench
        .step_with(
            parse_op("titrate v1 NaOH 1M 1mL until ph 7 max 50")
                .expect("parse")
                .expect("operator"),
            &mut s,
            &PermissiveScreen,
        )
        .expect("titrate");
    let run = titrated(&events).unwrap_or_else(|| panic!("{events:?}"));
    let (ml, reached) = (run.ml, run.reached);
    assert!(reached, "{events:?}");
    assert!(
        (ml - 10.0).abs() <= 1.0 + 1e-9,
        "0.01 mol of acid needs 10 mL of 1 mol/L base; got {ml} mL"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::NotYetModeled { what, .. } if what.contains("pe"))),
        "a pH titration must not acquire a redox apology: {events:?}"
    );
}
