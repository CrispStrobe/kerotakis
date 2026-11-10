//! The registry-solid coverage lane (2026-11-10): common laboratory
//! precipitates the routed databases already defined, but which had no
//! registry solid, so a supersaturated beaker deposited nothing. The family
//! scan found it for AgBr, AgI, Ag2CO3 and BaCO3; the coverage table is
//! `docs/registry-solid-coverage-20261110.md`.
//!
//! Every verdict here is the bench observable — a `Precipitated` event of
//! the named registry species — through the same solver stack the
//! qualitative-analysis tests use. Nothing restates a solubility product:
//! the amounts are chosen an order of magnitude or more past saturation, so
//! the assertion is about the solid existing on the shelf, not about the
//! third digit of its log K.
//!
//! KBr is not a registry bottle; sodium bromide supplies the bromide, which
//! is the same net-ionic reaction (Ag+ + Br- -> AgBr).

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

fn run(adds: &[(&str, f64)]) -> (Bench, Vec<Event>) {
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
    (bench, events)
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

fn solid_in_vessel(bench: &Bench, species: &str) -> f64 {
    bench.vessels[0]
        .contents
        .iter()
        .filter(|p| p.phase == Phase::Solid && p.species.0 == species)
        .map(|p| p.moles.0)
        .sum()
}

const WATER: (&str, f64) = ("water", 5.0);

fn assert_precipitates(adds: &[(&str, f64)], species: &str, at_least: f64) {
    let (bench, events) = run(adds);
    let event = precipitated(&events, species);
    assert!(
        event > at_least,
        "{species}: expected > {at_least} mol precipitated, events: {events:?}"
    );
    let held = solid_in_vessel(&bench, species);
    assert!(
        held > at_least,
        "{species}: the readback must hold the solid as `{species}`, vessel holds {held} mol: {:?}",
        bench.vessels[0].contents
    );
}

#[test]
fn silver_nitrate_and_bromide_give_silver_bromide() {
    assert_precipitates(&[WATER, ("NaBr", 0.012), ("AgNO3", 0.01)], "AgBr", 0.009);
}

#[test]
fn silver_nitrate_and_potassium_iodide_give_silver_iodide() {
    assert_precipitates(&[WATER, ("KI", 0.012), ("AgNO3", 0.01)], "AgI", 0.009);
}

#[test]
fn barium_chloride_and_soda_give_barium_carbonate() {
    assert_precipitates(&[WATER, ("Na2CO3", 0.012), ("BaCl2", 0.01)], "BaCO3", 0.009);
}

#[test]
fn silver_nitrate_and_soda_give_silver_carbonate() {
    // 0.02 mol Ag+ against 0.012 mol carbonate in 90 g of water is
    // hundreds of times past Ag2CO3's solubility; most of the silver must
    // come down as the carbonate.
    assert_precipitates(
        &[WATER, ("Na2CO3", 0.012), ("AgNO3", 0.02)],
        "Ag2CO3",
        0.008,
    );
}

#[test]
fn lead_nitrate_and_potassium_iodide_give_lead_iodide() {
    assert_precipitates(&[WATER, ("KI", 0.03), ("Pb(NO3)2", 0.01)], "PbI2", 0.008);
}

#[test]
fn lead_nitrate_and_sodium_sulfate_give_lead_sulfate() {
    assert_precipitates(
        &[WATER, ("Na2SO4", 0.012), ("Pb(NO3)2", 0.01)],
        "PbSO4",
        0.009,
    );
}

#[test]
fn lead_nitrate_and_soda_give_lead_carbonate() {
    assert_precipitates(
        &[WATER, ("Na2CO3", 0.012), ("Pb(NO3)2", 0.01)],
        "PbCO3",
        0.009,
    );
}

/// Negative: with the carbonate already driven off by excess acid there is
/// nothing for silver to precipitate as the carbonate. Sulfuric acid is
/// used because a chloride acid would precipitate AgCl and hide the point.
#[test]
fn acidified_carbonate_gives_no_silver_carbonate() {
    let (bench, events) = run(&[WATER, ("Na2CO3", 0.005), ("H2SO4", 0.02), ("AgNO3", 0.01)]);
    assert!(
        precipitated(&events, "Ag2CO3") < 1e-6,
        "silver carbonate from an acid beaker: {events:?}"
    );
    assert!(solid_in_vessel(&bench, "Ag2CO3") < 1e-6);
}

/// Negative: a silver solution with no precipitating anion stays clear of
/// every new silver solid.
#[test]
fn silver_nitrate_alone_deposits_none_of_the_new_silver_solids() {
    let (_, events) = run(&[WATER, ("NaNO3", 0.01), ("AgNO3", 0.01)]);
    for species in ["AgBr", "AgI", "Ag2CO3", "Ag3PO4"] {
        assert!(
            precipitated(&events, species) < 1e-6,
            "{species} from silver nitrate alone: {events:?}"
        );
    }
}
