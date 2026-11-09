#![cfg(feature = "engine")]
//! Batch-3 transport trigger scripts through the standard native stack.
//!
//! The originals are reproduced verbatim (setup and transport lines only;
//! read-outs are skipped) as evidence. They exposed four defects in one
//! mechanism: the feed vessel was never drained (tr-006 counted NaCl twice,
//! tr-004 delivered four cell volumes from a one-cell feed), reactions ran
//! once per command rather than per step (tr-008 let Cu2+ break through an
//! excess Fe bed), and ppm-level solvent-proxy differences refused the run
//! (tr-003, tr-007). Each feed holds one cell volume, so a command that asks
//! for more is now a typed `InletExhausted` refusal; the scripts' own
//! forecasts describe a repeated boundary injection, which is exercised by
//! the same script with the boundary explicitly declared `reservoir`.

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

fn solver() -> SolverStack {
    SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
        PhreeqcEquilibrator::new().expect("native engine"),
    )]))
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

/// One outcome per transport line, in order.
enum Outcome {
    Moved(Vec<Event>),
    Exhausted,
}

/// Run setup and transport lines; `reservoir` appends the declaration to
/// every transport line. `check` sees the bench before and after each
/// successful transport.
fn run(
    script: &str,
    reservoir: bool,
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
        let line = if verb == "transport" && reservoir {
            format!("{line} reservoir")
        } else {
            line.to_string()
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
                outcomes.push(Outcome::Moved(events));
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
        .map(|o| matches!(o, Outcome::Moved(_)))
        .collect()
}

fn element_in(bench: &Bench, vessel: usize, element: &str) -> f64 {
    elements(bench, &[vessel])
        .get(element)
        .copied()
        .unwrap_or(0.0)
}

#[test]
fn tr006_feed_salt_and_dye_are_counted_once() {
    let outcomes = run(TR006, false, |index, before, after, _| {
        let all = all_vessels(before);
        assert_closed(
            &elements(before, &all),
            &elements(after, &all),
            &["Na", "Cl"],
        );
        if index == 0 {
            // One cell volume delivered exactly: the feed is empty, the
            // 1e-4 mol of chloride is in the column (not also in the feed).
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
fn tr004_feed_is_drained_and_inflow_recorded() {
    let outcomes = run(TR004, false, |_, before, after, events| {
        let all = all_vessels(before);
        assert_closed(
            &elements(before, &all),
            &elements(after, &all),
            &["Ag", "Na", "Cl", "N"],
        );
        let Some(Event::Transported { inflow_moles, .. }) = events
            .iter()
            .find(|e| matches!(e, Event::Transported { .. }))
        else {
            panic!("{events:?}");
        };
        assert!(!inflow_moles.is_empty(), "inflow must be recorded");
        // Na is a spectator: what the feed lost is what the inflow says.
        let lost = element_in(before, 3, "Na") - element_in(after, 3, "Na");
        let reported: f64 = inflow_moles
            .iter()
            .filter(|(s, _)| s.0.contains("Na"))
            .map(|(_, m)| m.0)
            .sum();
        assert!((lost - reported).abs() <= 1e-12 * lost.max(1e-30));
    });
    assert_eq!(shape(&outcomes), vec![true, false]);
}

#[test]
fn tr004_declared_reservoir_retains_agcl_in_the_column() {
    let outcomes = run(TR004, true, |_, before, after, _| {
        // Silver enters only from the initial cell: it is conserved across
        // the column and receiver (the reservoir carries none).
        let ag_before: f64 = all_vessels(before)
            .iter()
            .map(|v| element_in(before, *v, "Ag"))
            .sum();
        let ag_after: f64 = all_vessels(after)
            .iter()
            .map(|v| element_in(after, *v, "Ag"))
            .sum();
        assert!((ag_after - ag_before).abs() <= 1e-9 * ag_before);
    });
    assert_eq!(shape(&outcomes), vec![true, true]);
}

#[test]
fn tr008_excess_iron_bed_retains_copper_per_step() {
    let outcomes = run(TR008, true, |_, before, after, _| {
        let all = all_vessels(before);
        // Fe and S are only internal (bed, sulfate from feed is open
        // inflow), so close Fe over the whole bench.
        assert_closed(&elements(before, &all), &elements(after, &all), &["Fe"]);
        let cu_in_receiver = element_in(after, 4, "Cu");
        let cu_in_column: f64 = [0, 1, 2].iter().map(|v| element_in(after, *v, "Cu")).sum();
        assert!(cu_in_column > 0.0);
        assert!(
            cu_in_receiver <= 1e-3 * cu_in_column,
            "Cu broke through an excess Fe bed: {cu_in_receiver:e} out vs {cu_in_column:e} held"
        );
    });
    assert_eq!(shape(&outcomes), vec![true, true]);
    // Without the declaration the 20 mL feed cannot supply 1.5 volumes.
    assert_eq!(
        shape(&run(TR008, false, |_, _, _, _| {})),
        vec![false, false]
    );
}

#[test]
fn tr003_and_tr007_are_not_refused_for_solvent_proxy_rounding() {
    // tr-003: inlet 0.02 L vs cell 0.019999968 L (1.6 ppm). tr-007: gypsum
    // hydrate water moves cell 1 by ~564 ppm. Both are inside the documented
    // hydraulic tolerance; the finite feed is then the binding limit.
    for (script, conserved) in [(TR003, &["Ca"][..]), (TR007, &["Ca", "S"][..])] {
        let finite = run(script, false, |_, _, _, _| {});
        assert_eq!(shape(&finite), vec![false, false]);
        let declared = run(script, true, |_, before, after, _| {
            let all = all_vessels(before);
            assert_closed(&elements(before, &all), &elements(after, &all), conserved);
        });
        assert_eq!(shape(&declared), vec![true, true]);
    }
}
