#![cfg(feature = "engine")]
//! Batch-3 transport trigger scripts through the standard native stack.
//!
//! The originals are reproduced verbatim (setup and transport lines only;
//! read-outs are skipped) as evidence. They were written when the script
//! default was an external constant-composition reservoir; the reservoir
//! tests below therefore append the explicit `reservoir` declaration, and
//! `Mode::Verbatim` runs the lines as written, which now drains the inlet
//! vessel (TR-006 follow-up: an undebited bench inlet duplicated every
//! species it supplied). They exposed three defects that
//! hold in any boundary mode: the reservoir's supply was not recorded, so the
//! open-column ledger could not be audited (tr-004, tr-006); reactions ran
//! once per command rather than per step (tr-008 let Cu2+ break through an
//! excess Fe bed); and ppm-level solvent-proxy differences refused the run
//! (tr-003, tr-007). Every original now closes its element ledger counting
//! the recorded `inflow_moles`. The same scripts with `drain` appended (or
//! verbatim, the default) exercise the finite feed: each feed holds one cell
//! volume, so a command asking for more is a typed `InletExhausted` refusal.

use std::collections::BTreeMap;

use kerotakis_core::ledger::ConservedLedger;
use kerotakis_core::transport::TransportError;
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

const TR003: &str = "register lv3
new
new
new
new
add v1 water 20mL
add v1 CaCO3 0.0005mol
add v2 water 20mL
add v2 CaCO3 0.0005mol
add v3 water 20mL
add v3 CaCO3 0.0005mol
add v4 water 20mL
add v4 HCl 0.0002mol
transport v1 v2 v3 from v4 to v5 steps 3 courant 1
measure v1 ph
transport v1 v2 v3 from v4 to v5 steps 15 courant 1
measure v5 ph";

const TR004: &str = "register lv3
new
new
new
new
add v1 water 20mL
add v2 water 20mL
add v2 AgNO3 0.0001mol
add v3 water 20mL
add v4 water 20mL
add v4 NaCl 0.0002mol
transport v1 v2 v3 from v4 to v5 steps 2 courant 0.5
inspect
transport v1 v2 v3 from v4 to v5 steps 6 courant 0.5";

const TR006: &str = "register lv3
new
new
new
new
new
add v1 water 20mL
add v1 activated_charcoal 0.02g
add v2 water 20mL
add v3 water 20mL
add v4 water 20mL
add v4 methyl_orange 0.001g
add v4 NaCl 0.0001mol
add v6 water 20mL
transport v1 v2 v3 from v4 to v5 steps 2 courant 0.5
inspect
transport v1 v2 v3 from v6 to v5 steps 6 courant 0.5";

const TR007: &str = "register lv3
new
new
new
new
add v1 water 20mL
add v1 gypsum 0.001mol
add v2 water 20mL
add v3 water 20mL
add v4 water 20mL
transport v1 v2 v3 from v4 to v5 steps 3 courant 1
inspect
transport v1 v2 v3 from v4 to v5 steps 5 courant 1";

const TR008: &str = "register lv3
new
new
new
new
add v1 water 20mL
add v1 Fe 0.001mol
add v2 water 20mL
add v3 water 20mL
add v4 water 20mL
add v4 CuSO4 0.0001mol
transport v1 v2 v3 from v4 to v5 steps 3 courant 0.5
inspect
transport v1 v2 v3 from v4 to v5 steps 6 courant 0.5";

/// The production stack, built exactly as the CLI builds it: the PHREEQC
/// tail wrapped by metal displacement and then by the phase pass. A bare
/// PHREEQC tail has no metal-displacement step at all, so Fe(s) + Cu2+
/// would never cement whatever the transport coupling did.
fn solver() -> SolverStack {
    let aqueous: Box<dyn Equilibrator> =
        Box::new(PhreeqcEquilibrator::new().expect("native engine"));
    let tail: Vec<Box<dyn Equilibrator>> = vec![Box::new(PhaseEquilibrator::wrapping(Box::new(
        DisplacementEquilibrator::wrapping(aqueous),
    )))];
    SolverStack::new(kerotakis_stack::standard_solvers(tail))
}

fn elements(bench: &Bench, vessels: &[usize]) -> BTreeMap<String, f64> {
    let mut total = BTreeMap::new();
    for index in vessels {
        let Ok(vessel) = bench.vessel(VesselId(*index)) else {
            continue;
        };
        for (element, moles) in ConservedLedger::from_vessel(vessel).elements {
            *total.entry(element).or_insert(0.0) += moles;
        }
    }
    total
}

fn all_vessels(bench: &Bench) -> Vec<usize> {
    bench.vessels.iter().map(|v| v.id.0).collect()
}

fn assert_closed(before: &BTreeMap<String, f64>, after: &BTreeMap<String, f64>, which: &[&str]) {
    for element in which {
        let b = before.get(*element).copied().unwrap_or(0.0);
        let a = after.get(*element).copied().unwrap_or(0.0);
        assert!(b > 0.0, "{element} must be present");
        assert!(
            (a - b).abs() <= 1e-9 * b,
            "{element}: {a:e} after vs {b:e} before"
        );
    }
}

/// Element totals of a command's recorded boundary inflow.
fn inflow_elements(events: &[Event]) -> BTreeMap<String, f64> {
    let Some(Event::Transported { inflow_moles, .. }) = events
        .iter()
        .find(|e| matches!(e, Event::Transported { .. }))
    else {
        panic!("transport must report its ledger: {events:?}");
    };
    let mut probe = Vessel::new(VesselId(999), "inflow probe");
    for (species, moles) in inflow_moles {
        let phase = if species.0 == "water" {
            Phase::Liquid
        } else {
            Phase::Aqueous
        };
        probe.deposit(species.clone(), *moles, phase);
    }
    ConservedLedger::from_vessel(&probe).elements
}

/// Open-boundary closure: after = before + recorded inflow.
fn assert_open_closed(before: &Bench, after: &Bench, events: &[Event], which: &[&str]) {
    let mut expected = elements(before, &all_vessels(before));
    for (element, moles) in inflow_elements(events) {
        *expected.entry(element).or_insert(0.0) += moles;
    }
    assert_closed(&expected, &elements(after, &all_vessels(after)), which);
}

/// One outcome per transport line, in order.
enum Outcome {
    Moved,
    Exhausted,
}

/// How each transport line is declared before it is parsed.
#[derive(Clone, Copy)]
enum Mode {
    /// The line exactly as the original wrote it (the finite-feed default).
    Verbatim,
    /// `reservoir` appended: the external constant-composition boundary.
    Reservoir,
    /// `drain` appended: the finite feed, stated explicitly.
    Drain,
}

/// Run setup and transport lines, declaring each transport line per
/// `mode`. `check` sees the bench before and after each successful
/// transport, with its events.
fn run(
    script: &str,
    mode: Mode,
    mut check: impl FnMut(usize, &Bench, &Bench, &[Event]),
) -> Vec<Outcome> {
    let mut bench = Bench::new();
    let mut solver = solver();
    let mut outcomes = Vec::new();
    for line in script.lines() {
        let verb = line.split_whitespace().next().unwrap_or("");
        if !matches!(verb, "new" | "add" | "transport") {
            continue;
        }
        let line = match (verb, mode) {
            ("transport", Mode::Reservoir) => format!("{line} reservoir"),
            ("transport", Mode::Drain) => format!("{line} drain"),
            _ => line.to_string(),
        };
        let op = kerotakis_core::script::parse_op(&line)
            .unwrap()
            .expect("an operator");
        if verb != "transport" {
            bench.step_with(op, &mut solver, &PermissiveScreen).unwrap();
            continue;
        }
        let before = bench.clone();
        match bench.step_with(op, &mut solver, &PermissiveScreen) {
            Ok(events) => {
                assert!(
                    !events
                        .iter()
                        .any(|event| matches!(event, Event::SolverFailed { .. })),
                    "{line}: {events:?}"
                );
                check(outcomes.len(), &before, &bench, &events);
                outcomes.push(Outcome::Moved);
            }
            Err(BenchError::Transport(TransportError::InletExhausted { .. })) => {
                outcomes.push(Outcome::Exhausted);
            }
            Err(other) => panic!("{line}: unexpected refusal {other:?}"),
        }
    }
    outcomes
}

fn shape(outcomes: &[Outcome]) -> Vec<bool> {
    outcomes
        .iter()
        .map(|o| matches!(o, Outcome::Moved))
        .collect()
}

fn element_in(bench: &Bench, vessel: usize, element: &str) -> f64 {
    elements(bench, &[vessel])
        .get(element)
        .copied()
        .unwrap_or(0.0)
}

#[test]
fn tr006_original_ledger_closes_with_recorded_inflow() {
    let outcomes = run(TR006, Mode::Reservoir, |_, before, after, events| {
        assert_open_closed(before, after, events, &["Na", "Cl"]);
    });
    assert_eq!(shape(&outcomes), vec![true, true]);
}

/// Both tr-006 commands, the elements only the dye and salt carry: the bound
/// dye on the charcoal is a ledger term (`ConservedLedger` counts
/// `adsorbed`), so N, S, Na and Cl close against before + inflow. Carbon is
/// left out: an open beaker may exchange CO2 with the air.
#[test]
fn tr006_original_closes_dye_and_bound_pool_elements_per_command() {
    let outcomes = run(TR006, Mode::Reservoir, |_, before, after, events| {
        assert_open_closed(before, after, events, &["N", "S", "Na", "Cl"]);
        assert!(matches!(
            events
                .iter()
                .find(|e| matches!(e, Event::Transported { .. })),
            Some(Event::Transported {
                inlet: Some(_),
                drain: false,
                ..
            })
        ));
    });
    assert_eq!(shape(&outcomes), vec![true, true]);
}

#[test]
fn tr006_drained_feed_counts_salt_once() {
    let outcomes = run(TR006, Mode::Drain, |index, before, after, _| {
        let all = all_vessels(before);
        assert_closed(
            &elements(before, &all),
            &elements(after, &all),
            &["Na", "Cl"],
        );
        if index == 0 {
            // Chloride, not sodium: the dye is a sodium salt too.
            let cl_feed = element_in(after, 3, "Cl");
            assert!(cl_feed <= 1e-15, "feed still holds {cl_feed:e} mol Cl");
            let column: f64 = [0, 1, 2, 4]
                .iter()
                .map(|v| element_in(after, *v, "Cl"))
                .sum();
            assert!((column - 1e-4).abs() <= 1e-12, "column Cl {column:e}");
        }
    });
    // The second command asks a 20 mL feed for three cell volumes.
    assert_eq!(shape(&outcomes), vec![true, false]);
}

#[test]
fn tr004_original_records_reservoir_inflow_and_keeps_silver() {
    let outcomes = run(TR004, Mode::Reservoir, |_, before, after, events| {
        assert_open_closed(before, after, events, &["Ag", "Na", "Cl", "N"]);
        assert!(inflow_elements(events).get("Na").copied().unwrap_or(0.0) > 0.0);
        // The reservoir itself is not drawn down.
        assert_eq!(element_in(before, 3, "Na"), element_in(after, 3, "Na"));
    });
    assert_eq!(shape(&outcomes), vec![true, true]);
}

#[test]
fn tr004_drained_feed_is_finite() {
    let outcomes = run(TR004, Mode::Drain, |_, before, after, events| {
        let all = all_vessels(before);
        assert_closed(
            &elements(before, &all),
            &elements(after, &all),
            &["Ag", "Na", "Cl", "N"],
        );
        let lost = element_in(before, 3, "Na") - element_in(after, 3, "Na");
        let reported = inflow_elements(events).get("Na").copied().unwrap_or(0.0);
        assert!((lost - reported).abs() <= 1e-12 * lost.max(1e-30));
    });
    assert_eq!(shape(&outcomes), vec![true, false]);
}

#[test]
fn tr008_original_excess_iron_bed_retains_copper_per_step() {
    let outcomes = run(TR008, Mode::Reservoir, |_, before, after, events| {
        assert_open_closed(before, after, events, &["Fe", "Cu", "S"]);
        let cu_in_receiver = element_in(after, 4, "Cu");
        let cu_in_column: f64 = [0, 1, 2].iter().map(|v| element_in(after, *v, "Cu")).sum();
        assert!(cu_in_column > 0.0);
        assert!(
            cu_in_receiver <= 1e-3 * cu_in_column,
            "Cu broke through an excess Fe bed: {cu_in_receiver:e} out vs {cu_in_column:e} held"
        );
    });
    assert_eq!(shape(&outcomes), vec![true, true]);
    // A drained 20 mL feed cannot supply 1.5 cell volumes.
    assert_eq!(
        shape(&run(TR008, Mode::Drain, |_, _, _, _| {})),
        vec![false, false]
    );
}

#[test]
fn tr003_and_tr007_are_not_refused_for_solvent_proxy_rounding() {
    // tr-003: inlet 0.02 L vs cell 0.019999968 L (1.6 ppm). tr-007: gypsum
    // hydrate water moves cell 1 by ~564 ppm. Both are inside the documented
    // hydraulic tolerance.
    for (script, conserved) in [(TR003, &["Ca"][..]), (TR007, &["Ca", "S"][..])] {
        let original = run(script, Mode::Reservoir, |_, before, after, events| {
            assert_open_closed(before, after, events, conserved);
        });
        assert_eq!(shape(&original), vec![true, true]);
        // Drained, the one-volume feed is the binding limit.
        assert_eq!(
            shape(&run(script, Mode::Drain, |_, _, _, _| {})),
            vec![false, false]
        );
    }
}

/// The verbatim tr-006 script (no boundary word) drains its bench inlet:
/// whatever entered the column left v4, so the whole bench closes per
/// element with NO source term, the transported event says `drain`, and its
/// recorded inflow equals exactly what the inlet lost. The second command
/// asks a 20 mL rinse vessel for three cell volumes and is refused, leaving
/// the bench as it was rather than inventing two volumes of water.
#[test]
fn tr006_verbatim_inlet_is_debited_by_what_entered_the_column() {
    let outcomes = run(TR006, Mode::Verbatim, |_, before, after, events| {
        let all = all_vessels(before);
        assert_closed(
            &elements(before, &all),
            &elements(after, &all),
            &["N", "S", "Na", "Cl"],
        );
        let Some(Event::Transported {
            drain,
            inlet: Some(inlet),
            ..
        }) = events
            .iter()
            .find(|e| matches!(e, Event::Transported { .. }))
        else {
            panic!("transport must report its ledger: {events:?}");
        };
        assert!(*drain, "a verbatim bench inlet must be a finite feed");
        let inflow = inflow_elements(events);
        for element in ["Na", "Cl", "N", "S"] {
            let lost = element_in(before, inlet.0, element) - element_in(after, inlet.0, element);
            let reported = inflow.get(element).copied().unwrap_or(0.0);
            assert!(reported > 0.0, "{element} must enter the column");
            assert!(
                (lost - reported).abs() <= 1e-12 * reported,
                "{element}: inlet lost {lost:e}, column received {reported:e}"
            );
        }
    });
    assert_eq!(shape(&outcomes), vec![true, false]);
}

/// Negative control: the same lines declared `reservoir` keep the inlet
/// whole, and the bench grows by exactly the recorded source term.
#[test]
fn tr006_declared_reservoir_is_not_debited_and_states_its_source_term() {
    let outcomes = run(TR006, Mode::Reservoir, |_, before, after, events| {
        assert_open_closed(before, after, events, &["Na", "Cl", "N", "S"]);
        let inlet = match events
            .iter()
            .find(|e| matches!(e, Event::Transported { .. }))
        {
            Some(Event::Transported {
                drain: false,
                inlet: Some(inlet),
                ..
            }) => inlet.0,
            other => panic!("a declared reservoir must say so: {other:?}"),
        };
        assert_eq!(
            element_in(before, inlet, "Cl"),
            element_in(after, inlet, "Cl")
        );
    });
    assert_eq!(shape(&outcomes), vec![true, true]);
}
