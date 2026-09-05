//! Electrolysis of the solution itself, where there is no metal half-cell.
//!
//! The electrolyser modelled one thing: a metal standing in a solution of
//! its own ion. That is a real cell and it is not the school one. Salt water
//! needs no metal at all — two carbon rods, hydrogen at one and chlorine at
//! the other — and the bench refused it with a sentence that was accurate
//! about the model and wrong about the chemistry:
//!
//!     nothing here can be electrolysed: v1 holds neither a metal of the
//!     series nor a dissolved metal ion, so there is nothing to be an
//!     electrode
//!
//! Two questions, kept separate because they are separate. HOW MUCH is
//! arithmetic — `n = I·t/F`, with the Faraday constant that was already
//! here for the activity series. WHAT is the activity series itself, which
//! already holds the number that decides which species is easiest to
//! reduce or oxidise.

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;
use kerotakis_safety::ReactiveGroupScreen;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(CuratedEquilibrator),
        Box::new(PhreeqcEquilibrator::new().expect("engine")),
        Box::new(HonestyEquilibrator),
    ])
}

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
            &ReactiveGroupScreen,
        )
        .expect("step");
}

fn run_cell(bench: &mut Bench, stack: &mut SolverStack, v: VesselId) -> Vec<Event> {
    bench
        .step_with(
            Operator::Electrolyse {
                vessel: v,
                amps: 0.5,
                seconds: 1800.0,
            },
            stack,
            &ReactiveGroupScreen,
        )
        .expect("step")
}

fn gas(events: &[Event], key: &str) -> f64 {
    events
        .iter()
        .filter_map(|e| match e {
            Event::GasEvolved { species, moles, .. }
            | Event::GasContained { species, moles, .. }
                if species.0 == key =>
            {
                Some(moles.0)
            }
            _ => None,
        })
        .sum()
}

/// Brine gives hydrogen, chlorine, and caustic soda. No metal involved.
///
/// This is the chloralkali process, and the alkali is not a detail — it is
/// what the cell is FOR, and it is why the water around the cathode turns
/// phenolphthalein pink in the school demonstration.
#[test]
fn salt_water_gives_hydrogen_chlorine_and_alkali() {
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 5.5343);
    add(&mut bench, &mut stack, v, "NaCl", 0.01);
    let events = run_cell(&mut bench, &mut stack, v);

    // 0.5 A for 1800 s is 900 C, which is 900/96485 = 9.33e-3 mol of
    // electrons. Two per molecule of hydrogen, two per molecule of
    // chlorine, so the same amount of each — which is the observation the
    // experiment is famous for.
    let h2 = gas(&events, "H2");
    let cl2 = gas(&events, "Cl2");
    assert!(
        (h2 - 4.66e-3).abs() < 2e-4,
        "9.33e-3 mol of electrons makes 4.66e-3 mol of hydrogen, got {h2:.5}"
    );
    assert!(
        (cl2 - h2).abs() < 1e-6,
        "one chlorine per hydrogen at this current: {cl2:.5} against {h2:.5}"
    );

    // And the cell has turned strongly alkaline. Sodium stays, chloride
    // leaves as gas, hydroxide is what is left.
    let ph = bench
        .vessel(v)
        .unwrap()
        .solution
        .clone()
        .expect("solution")
        .ph;
    assert!(
        ph > 12.0,
        "the chloralkali cell makes caustic soda; pH is {ph:.2}"
    );
}

/// Copper sulfate plates copper and gives oxygen, on inert electrodes.
///
/// The cathode reduces whatever is easiest, and a metal ion counts only
/// when it is easier to reduce than water. Copper is (E° +0.342) and plates
/// out on a carbon rod; sodium is not (E° −2.71) and never does, which is
/// why the brine cell above gives hydrogen instead. One rule, and the
/// activity series already held the number.
#[test]
fn copper_sulfate_plates_copper_and_gives_oxygen() {
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 5.5343);
    add(&mut bench, &mut stack, v, "CuSO4", 0.01);
    let events = run_cell(&mut bench, &mut stack, v);

    let plated = events
        .iter()
        .find_map(|e| match e {
            Event::Electrolysed { species, moles, .. } if species.0 == "Cu" => Some(moles.0),
            _ => None,
        })
        .unwrap_or_else(|| panic!("copper should plate: {events:?}"));
    assert!(
        (plated - 4.66e-3).abs() < 2e-4,
        "two electrons per copper: 9.33e-3 mol of them plates 4.66e-3 mol, got {plated:.5}"
    );
    // Four electrons per molecule of oxygen, so a quarter as much again.
    let o2 = gas(&events, "O2");
    assert!(
        (o2 - 2.33e-3).abs() < 2e-4,
        "2 H₂O → O₂ + 4 H⁺ + 4 e⁻ gives 2.33e-3 mol, got {o2:.5}"
    );
    // No chlorine anywhere — there is no chloride to oxidise. Sulfate is
    // not oxidised at these potentials and water goes instead, which is
    // the difference between this cell and the brine one.
    assert_eq!(gas(&events, "Cl2"), 0.0, "no chloride, no chlorine");
}

/// Pure water is not electrolysed, because pure water does not conduct.
///
/// A bench that electrolysed it would be teaching that it does. The
/// refusal is the answer here, not a gap.
#[test]
fn pure_water_is_an_insulator_and_says_so() {
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 5.5343);
    let events = run_cell(&mut bench, &mut stack, v);

    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::NotYetModeled { .. })),
        "no electrolyte, no current, and it must say so: {events:?}"
    );
    assert_eq!(gas(&events, "H2"), 0.0, "nothing comes off pure water");
    assert_eq!(gas(&events, "O2"), 0.0, "nothing comes off pure water");
}

/// An inert sulfate electrolyte lets the school cell split water without
/// introducing a competing electrode product.
#[test]
fn sodium_sulfate_water_gives_two_hydrogen_per_oxygen_and_spends_water() {
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 5.5343);
    add(&mut bench, &mut stack, v, "Na2SO4", 0.01);
    let before = bench.vessel(v).unwrap().mass().0;
    let events = run_cell(&mut bench, &mut stack, v);
    let h2 = gas(&events, "H2");
    let o2 = gas(&events, "O2");
    assert!((h2 / o2 - 2.0).abs() < 1e-9, "H2={h2}, O2={o2}");
    assert_eq!(gas(&events, "Cl2"), 0.0, "sulfate is not chloride");
    assert!(
        bench.vessel(v).unwrap().mass().0 < before,
        "open-cell gases carry mass away"
    );
}
