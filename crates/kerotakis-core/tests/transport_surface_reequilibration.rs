//! Transport re-equilibrates every cell's bound pools, both ways.
//!
//! Production evidence: a charcoal cell loaded with methyl orange kept its
//! bound amount bit-for-bit through a clean-water flush. The transport loop
//! did call the adsorption rung after every step; the rung skipped the move
//! because each step's deficit was under the 0.1 mg EVENT threshold, which
//! was also gating the MATTER. These tests pin the repaired contract: the
//! flush desorbs toward the isotherm, the dye ledger closes exactly
//! (dissolved + bound everywhere = before + inflow), and a column without
//! a sorbent is untouched by the adsorption rung.

use kerotakis_core::adsorption::{self, AdsorptionEquilibrator};
use kerotakis_core::ops::{Event, Operator};
use kerotakis_core::species::{self, Phase, SpeciesId};
use kerotakis_core::units::{Kelvin, Moles};
use kerotakis_core::vessel::{Vessel, VesselId};
use kerotakis_core::Bench;

const DYE: &str = "methyl_orange";
const CARBON: &str = "activated_charcoal";
/// `add vN water 20mL`.
const WATER_20ML: f64 = 1.1069;

fn water_cell(vessel: &mut Vessel) {
    vessel.deposit(SpeciesId::new("water"), Moles(WATER_20ML), Phase::Liquid);
    vessel.temperature = Kelvin::STANDARD;
}

/// v0 carbon cell, v1/v2 water cells, v3 clean-water inlet, v4 receiver.
fn column(carbon_grams: f64, dye_in_v0: f64) -> Bench {
    let mut bench = Bench::new();
    for _ in 0..4 {
        bench.step(Operator::NewVessel { kind: None }).unwrap();
    }
    for vessel in &mut bench.vessels[..4] {
        water_cell(vessel);
    }
    if carbon_grams > 0.0 {
        let carbon = species::lookup(&SpeciesId::new(CARBON)).expect("registry carbon");
        bench.vessels[0].deposit(
            SpeciesId::new(CARBON),
            Moles(carbon_grams / carbon.molar_mass),
            Phase::Solid,
        );
    }
    if dye_in_v0 > 0.0 {
        bench.vessels[0].deposit(SpeciesId::new(DYE), Moles(dye_in_v0), Phase::Aqueous);
        adsorption::equilibrate(&mut bench.vessels[0]);
    }
    bench
}

fn flush(steps: u32) -> Operator {
    Operator::Transport {
        chain: vec![VesselId(0), VesselId(1), VesselId(2)],
        inlet: VesselId(3),
        receiver: VesselId(4),
        steps,
        courant: 0.5,
        drain: false,
    }
}

fn dye_everywhere(bench: &Bench) -> f64 {
    let id = SpeciesId::new(DYE);
    bench
        .vessels
        .iter()
        .map(|v| v.moles_of(&id).0 + v.adsorbed_moles(&id).0)
        .sum()
}

fn inflow_of(events: &[Event], species: &str) -> f64 {
    let id = SpeciesId::new(species);
    events
        .iter()
        .filter_map(|e| match e {
            Event::Transported { inflow_moles, .. } => Some(inflow_moles),
            _ => None,
        })
        .flatten()
        .filter(|(s, _)| *s == id)
        .map(|(_, m)| m.0)
        .sum()
}

/// The TR-006 shape: 0.02 g of carbon in a 20 mL cell holding ~0.86 mg.
#[test]
fn a_clean_water_flush_desorbs_toward_the_isotherm() {
    let mut bench = column(0.02, 2.9776e-6);
    let dye = SpeciesId::new(DYE);
    let bound_before = bench.vessels[0].adsorbed_moles(&dye).0;
    assert!(
        bound_before > 2e-6,
        "loading precondition: {bound_before:e}"
    );
    assert!(
        !adsorption::applies(&bench.vessels[0]),
        "the loaded cell starts on its isotherm"
    );
    let total_before = dye_everywhere(&bench);

    let events = bench
        .step_with(
            flush(6),
            &mut AdsorptionEquilibrator,
            &kerotakis_core::PermissiveScreen,
        )
        .unwrap();

    let bound_after = bench.vessels[0].adsorbed_moles(&dye).0;
    assert!(
        bound_after < 0.9 * bound_before,
        "three pore volumes of clean water must strip dye: {bound_before:e} -> {bound_after:e}"
    );
    assert!(bound_after > 0.0, "a Langmuir desorption is partial");
    // Dye left the bed for the downstream cells and the receiver.
    let downstream: f64 = (1..5).map(|i| bench.vessels[i].moles_of(&dye).0).sum();
    assert!(downstream > 0.5 * (bound_before - bound_after));
    // Every cell ends on its own isotherm (no stale bound pool anywhere).
    for (index, vessel) in bench.vessels[..3].iter().enumerate() {
        assert!(
            !adsorption::applies(vessel),
            "cell {index} left off its isotherm"
        );
    }
    // Exact ledger: dissolved + bound everywhere = before + inflow (clean
    // inlet supplies no dye).
    assert_eq!(inflow_of(&events, DYE), 0.0);
    let total_after = dye_everywhere(&bench);
    assert!(
        (total_after - total_before).abs() <= 1e-12 * total_before,
        "dye ledger: {total_before:e} -> {total_after:e}"
    );
}

/// The per-step deficit that froze the bed, pinned directly: a cell 0.08 mg
/// off its isotherm (under the 0.1 mg event threshold) still moves.
#[test]
fn a_sub_event_threshold_deficit_still_moves_matter() {
    let mut cell = Vessel::new(VesselId(0), "cell");
    water_cell(&mut cell);
    let carbon = species::lookup(&SpeciesId::new(CARBON)).unwrap();
    cell.deposit(
        SpeciesId::new(CARBON),
        Moles(0.02 / carbon.molar_mass),
        Phase::Solid,
    );
    let dye = SpeciesId::new(DYE);
    cell.deposit(dye.clone(), Moles(2.9776e-6), Phase::Aqueous);
    adsorption::equilibrate(&mut cell);
    let bound = cell.adsorbed_moles(&dye).0;
    // Replace the pore water with clean water, as one full flush would.
    let free = cell.moles_of(&dye).0;
    cell.withdraw(&dye, Moles(free));
    assert!(adsorption::applies(&cell));
    let _ = adsorption::equilibrate(&mut cell);
    let after = cell.adsorbed_moles(&dye).0;
    assert!(after < bound, "{bound:e} -> {after:e}");
    assert!(
        ((after + cell.moles_of(&dye).0) - bound).abs() <= 1e-15 + 1e-12 * bound,
        "release is exact"
    );
    assert!(!adsorption::applies(&cell), "one call reaches the isotherm");
}

/// A dye front reaching a carbon cell is still taken up step by step.
#[test]
fn a_dye_front_is_adsorbed_as_it_arrives() {
    let mut bench = column(0.02, 0.0);
    bench.vessels[3].deposit(SpeciesId::new(DYE), Moles(3.0e-6), Phase::Aqueous);
    let total_before = dye_everywhere(&bench);
    let events = bench
        .step_with(
            flush(2),
            &mut AdsorptionEquilibrator,
            &kerotakis_core::PermissiveScreen,
        )
        .unwrap();
    let dye = SpeciesId::new(DYE);
    assert!(bench.vessels[0].adsorbed_moles(&dye).0 > 0.0);
    let inflow = inflow_of(&events, DYE);
    assert!(inflow > 0.0);
    // The inlet is a constant reservoir: it still holds its dye.
    let total_after = dye_everywhere(&bench);
    assert!(
        (total_after - (total_before + inflow)).abs() <= 1e-12 * (total_before + inflow),
        "open-column ledger: {total_before:e} + {inflow:e} -> {total_after:e}"
    );
}

/// No sorbent anywhere: the rung neither applies nor writes a ledger.
#[test]
fn a_column_without_surfaces_is_untouched_by_the_adsorption_rung() {
    let mut bench = column(0.0, 0.0);
    bench.vessels[3].deposit(SpeciesId::new("NaCl"), Moles(1e-4), Phase::Aqueous);
    bench.vessels[3].deposit(SpeciesId::new(DYE), Moles(1e-6), Phase::Aqueous);
    let mut reference = bench.clone();
    bench
        .step_with(
            flush(4),
            &mut AdsorptionEquilibrator,
            &kerotakis_core::PermissiveScreen,
        )
        .unwrap();
    struct Noop;
    impl kerotakis_core::Equilibrator for Noop {
        fn name(&self) -> &'static str {
            "noop-test"
        }
        fn equilibrate(
            &mut self,
            _vessel: &mut Vessel,
        ) -> Result<Vec<Event>, kerotakis_core::SolveError> {
            Ok(Vec::new())
        }
    }
    reference
        .step_with(flush(4), &mut Noop, &kerotakis_core::PermissiveScreen)
        .unwrap();
    for (a, b) in bench.vessels.iter().zip(&reference.vessels) {
        assert!(a.adsorbed.is_empty());
        for species in ["NaCl", DYE, "water"] {
            let id = SpeciesId::new(species);
            assert_eq!(a.moles_of(&id).0, b.moles_of(&id).0, "{species}");
        }
    }
}
