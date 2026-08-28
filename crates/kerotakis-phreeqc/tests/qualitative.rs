//! EXP-30 — qualitative inorganic analysis: the classic bench verdicts,
//! each cell engine-verified before it was pinned. The observable is the
//! event a learner would log — a precipitate of a named species, a gas —
//! not an internal number, because the whole point of qualitative analysis
//! is that the beaker itself is the readout.
//!
//! The iron rows are the hard-won ones. Iron(III) plus lye precipitates
//! iron(III) hydroxide only once the candidate list stopped being
//! database-blind (wateq4f spells the phase `Fe(OH)3(a)` where the global
//! dedupe kept minteq's `Ferrihydrite`). Iron(II) plus lye precipitates
//! iron(II) hydroxide — the green solid of the schoolbook — only because
//! three separate leaks were closed: a ferric phase is not admitted
//! against ferrous totals without a redox partner, an uncoupled element's
//! oxidation state is pinned inside the solve, and the readback returns
//! the dissolved total in the state it was added in.

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;
use kerotakis_safety::ReactiveGroupScreen;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(CuratedEquilibrator),
        Box::new(kerotakis_core::nonaqueous::NonAqueousEquilibrator),
        Box::new(kerotakis_core::hmix::MixingEnthalpyEquilibrator),
        Box::new(PhreeqcEquilibrator::new().expect("engine")),
        Box::new(HonestyEquilibrator),
    ])
}

/// Run the additions into one beaker and collect every event.
fn run(adds: &[(&str, f64)]) -> Vec<Event> {
    let mut bench = Bench::new();
    let mut solvers = stack();
    let mut events = Vec::new();
    for (key, moles) in adds {
        events.extend(
            bench
                .step_with(
                    Operator::Add {
                        vessel: VesselId(0),
                        species: SpeciesId::new(key),
                        moles: Moles(*moles),
                        at: None,
                    },
                    &mut solvers,
                    &ReactiveGroupScreen,
                )
                .unwrap_or_else(|e| panic!("ADD {key}: {e}")),
        );
    }
    events
}

fn precipitated(events: &[Event], species: &str) -> f64 {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Precipitated {
                species: s, moles, ..
            } if s.0 == species => Some(moles.0),
            _ => None,
        })
        .sum()
}

fn gas_evolved(events: &[Event], species: &str) -> f64 {
    events
        .iter()
        .filter_map(|e| match e {
            Event::GasEvolved {
                species: s, moles, ..
            } if s.0 == species => Some(moles.0),
            _ => None,
        })
        .sum()
}

fn no_observable(events: &[Event]) {
    for e in events {
        if let Event::Precipitated { species, moles, .. } = e {
            assert!(
                moles.0 <= 1e-6,
                "unexpected precipitate {} ({})",
                species.0,
                moles.0
            );
        }
        if let Event::GasEvolved { species, moles, .. } = e {
            assert!(
                moles.0 <= 1e-6,
                "unexpected gas {} ({})",
                species.0,
                moles.0
            );
        }
    }
}

const WATER: (&str, f64) = ("water", 5.0);

#[test]
fn copper_hydroxide_is_the_blue_precipitate() {
    let events = run(&[WATER, ("CuSO4", 0.01), ("NaOH", 0.03)]);
    assert!(precipitated(&events, "Cu(OH)2") > 0.009, "{events:?}");
}

#[test]
fn ferric_iron_gives_the_red_brown_hydroxide() {
    let events = run(&[WATER, ("Fe+3", 0.01), ("NaOH", 0.04)]);
    assert!(precipitated(&events, "Fe(OH)3") > 0.009, "{events:?}");
}

#[test]
fn ferrous_iron_gives_the_green_hydroxide_not_the_ferric_one() {
    // The distinguishing observation of the Fe²⁺/Fe³⁺ pair: there is no
    // oxidant in this beaker, so the ferric hydroxide must NOT appear.
    let events = run(&[WATER, ("FeSO4", 0.01), ("NaOH", 0.03)]);
    assert!(precipitated(&events, "Fe(OH)2") > 0.009, "{events:?}");
    assert!(
        precipitated(&events, "Fe(OH)3") < 1e-6,
        "iron oxidised with no oxidant in the beaker: {events:?}"
    );
}

#[test]
fn magnesium_and_zinc_hydroxides_precipitate() {
    let events = run(&[WATER, ("MgSO4", 0.01), ("NaOH", 0.03)]);
    assert!(precipitated(&events, "Mg(OH)2") > 0.009, "{events:?}");
    let events = run(&[WATER, ("ZnSO4", 0.01), ("NaOH", 0.02)]);
    assert!(precipitated(&events, "Zn(OH)2") > 0.009, "{events:?}");
}

#[test]
fn silver_chloride_curdles_out_of_brine() {
    let events = run(&[WATER, ("NaCl", 0.01), ("AgNO3", 0.012)]);
    assert!(precipitated(&events, "AgCl") > 0.009, "{events:?}");
}

#[test]
fn carbonate_effervesces_with_acid() {
    let events = run(&[WATER, ("Na2CO3", 0.01), ("HCl", 0.03)]);
    assert!(gas_evolved(&events, "CO2") > 0.009, "{events:?}");
}

#[test]
fn lead_chloride_stays_dissolved_when_dilute() {
    // The conditional verdict, verified against the engine's own
    // solubility product: at 0.1 mol/kgw lead and 0.3 mol/kgw chloride
    // the ion product sits under Ksp, so the "white precipitate with
    // chloride" of the tables — written for concentrated spot tests —
    // correctly does not appear.
    let events = run(&[WATER, ("Pb(NO3)2", 0.01), ("NaCl", 0.03)]);
    no_observable(&events);
}
