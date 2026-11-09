//! Transport ledger, per-step reaction coupling and the documented
//! hydraulic volume tolerance.
//!
//! The default inlet is an external constant-composition reservoir (as in a
//! PHREEQC TRANSPORT boundary); what it supplies is now recorded on the
//! event as `inflow_moles`, so the open-column ledger closes for every
//! species: cells + receiver = before + inflow. A finite feed vessel is the
//! declared `drain` mode, in which feed + cells + receiver is constant to
//! 1e-12 relative. In both modes local chemistry runs after every step.

use kerotakis_core::ops::{Event, Operator};
use kerotakis_core::species::{Phase, SpeciesId};
use kerotakis_core::transport::{TransportError, HYDRAULIC_VOLUME_RELATIVE_TOLERANCE};
use kerotakis_core::units::{Kelvin, Moles};
use kerotakis_core::vessel::{Vessel, VesselId};
use kerotakis_core::{Bench, BenchError, CellChain, Equilibrator, SolveError};

const WATER: f64 = 5.5509;

struct Noop;

impl Equilibrator for Noop {
    fn name(&self) -> &'static str {
        "noop-test"
    }
    fn equilibrate(&mut self, _vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        Ok(Vec::new())
    }
}

/// A toy cementation: any aqueous `toy-ion` meeting the solid `toy-bed`
/// deposits as immobile `toy-deposit` and releases one aqueous
/// `toy-product` per mole, limited by the bed. It stands for any fast local
/// reaction whose stationary reagent must see every pore volume.
struct ToyCementation {
    calls: usize,
}

impl Equilibrator for ToyCementation {
    fn name(&self) -> &'static str {
        "toy-cementation"
    }
    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.calls += 1;
        let bed = SpeciesId::new("toy-bed");
        let ion = SpeciesId::new("toy-ion");
        let bed_moles = vessel
            .contents
            .iter()
            .filter(|p| p.species == bed && p.phase == Phase::Solid)
            .map(|p| p.moles.0)
            .sum::<f64>();
        let ion_moles = vessel
            .contents
            .iter()
            .filter(|p| p.species == ion && p.phase == Phase::Aqueous)
            .map(|p| p.moles.0)
            .sum::<f64>();
        let extent = bed_moles.min(ion_moles);
        if extent > 0.0 {
            vessel.withdraw_phase(&bed, Moles(extent), Phase::Solid);
            vessel.withdraw_phase(&ion, Moles(extent), Phase::Aqueous);
            vessel.deposit(SpeciesId::new("toy-deposit"), Moles(extent), Phase::Solid);
            vessel.deposit(SpeciesId::new("toy-product"), Moles(extent), Phase::Aqueous);
        }
        Ok(Vec::new())
    }
}

fn water_cell(vessel: &mut Vessel, water: f64) {
    vessel.deposit(SpeciesId::new("water"), Moles(water), Phase::Liquid);
    vessel.temperature = Kelvin::STANDARD;
}

/// Three water cells (v0..v2), a feed (v3) and an empty receiver (v4).
fn column(feed_volumes: f64, feed_solutes: &[(&str, f64)]) -> Bench {
    let mut bench = Bench::new();
    water_cell(&mut bench.vessels[0], WATER);
    for index in 1..=4 {
        bench.step(Operator::NewVessel { kind: None }).unwrap();
        if index < 3 {
            water_cell(&mut bench.vessels[index], WATER);
        }
    }
    water_cell(&mut bench.vessels[3], WATER * feed_volumes);
    for (species, moles) in feed_solutes {
        bench.vessels[3].deposit(SpeciesId::new(species), Moles(*moles), Phase::Aqueous);
    }
    bench.vessels[3].solute_charge = 0.0;
    bench
}

fn transport(steps: u32, courant: f64, drain: bool) -> Operator {
    Operator::Transport {
        chain: vec![VesselId(0), VesselId(1), VesselId(2)],
        inlet: VesselId(3),
        receiver: VesselId(4),
        steps,
        courant,
        drain,
    }
}

fn bench_total(bench: &Bench, species: &str) -> f64 {
    let id = SpeciesId::new(species);
    bench.vessels.iter().map(|v| v.moles_of(&id).0).sum()
}

fn assert_relative(actual: f64, expected: f64, tolerance: f64, what: &str) {
    let scale = expected.abs().max(f64::MIN_POSITIVE);
    assert!(
        (actual - expected).abs() <= tolerance * scale,
        "{what}: {actual:e} vs {expected:e} (relative {:e})",
        (actual - expected).abs() / scale
    );
}

#[test]
fn finite_feed_conserves_every_species_across_feed_cells_and_effluent() {
    // Three cell volumes of feed, delivered as ten steps at Cf=0.3.
    let solutes = [("NaCl", 0.03), ("passive-tracer", 7.0e-6), ("water", 0.0)];
    let mut bench = column(3.0, &solutes[..2]);
    let before: Vec<f64> = solutes
        .iter()
        .map(|(s, _)| bench_total(&bench, s))
        .collect();
    let feed_before = bench.vessels[3].clone();

    let events = bench
        .step_with(
            transport(10, 0.3, true),
            &mut Noop,
            &kerotakis_core::PermissiveScreen,
        )
        .unwrap();

    for ((species, _), before) in solutes.iter().zip(&before) {
        assert_relative(bench_total(&bench, species), *before, 1e-12, species);
    }
    // The feed lost exactly 3 cell volumes x 0.3 x 10 / 3 = all but 0 ...
    // precisely: 10 x 0.3 = 3 cell volumes, i.e. the whole feed.
    let id = SpeciesId::new("NaCl");
    assert!(bench.vessels[3].moles_of(&id).0 <= 1e-12 * 0.03);
    // ... and the event records the inflow the feed lost.
    let Some(Event::Transported {
        inflow_moles,
        drain,
        ..
    }) = events
        .iter()
        .find(|e| matches!(e, Event::Transported { .. }))
    else {
        panic!("transport must report its ledger: {events:?}");
    };
    assert!(*drain);
    for species in ["NaCl", "passive-tracer", "water"] {
        let id = SpeciesId::new(species);
        let inflow: f64 = inflow_moles
            .iter()
            .filter(|(s, _)| *s == id)
            .map(|(_, m)| m.0)
            .sum();
        let lost = feed_before.moles_of(&id).0 - bench.vessels[3].moles_of(&id).0;
        assert_relative(inflow, lost, 1e-12, species);
    }
}

#[test]
fn a_feed_is_not_duplicated_into_the_column() {
    // One cell volume of feed, exactly delivered: the salt is in the column
    // and receiver, not ALSO still in the feed (the old 2x inventory).
    let mut bench = column(1.0, &[("NaCl", 1e-4)]);
    bench
        .step_with(
            transport(2, 0.5, true),
            &mut Noop,
            &kerotakis_core::PermissiveScreen,
        )
        .unwrap();
    assert_relative(bench_total(&bench, "NaCl"), 1e-4, 1e-12, "NaCl");
    assert_eq!(bench.vessels[3].moles_of(&SpeciesId::new("NaCl")).0, 0.0);
    assert_eq!(bench.vessels[3].moles_of(&SpeciesId::new("water")).0, 0.0);
}

#[test]
fn an_exhausted_feed_is_a_typed_atomic_refusal() {
    let mut bench = column(1.0, &[("NaCl", 1e-4)]);
    let before = serde_json::to_value(&bench.vessels).unwrap();
    let log = bench.log.len();
    let error = bench
        .step_with(
            transport(3, 0.5, true),
            &mut Noop,
            &kerotakis_core::PermissiveScreen,
        )
        .unwrap_err();
    assert!(
        matches!(
            error,
            BenchError::Transport(TransportError::InletExhausted { step: 3, .. })
        ),
        "{error:?}"
    );
    assert_eq!(serde_json::to_value(&bench.vessels).unwrap(), before);
    assert_eq!(bench.log.len(), log);
}

#[test]
fn the_default_reservoir_is_not_drained_and_its_inflow_is_recorded() {
    let mut bench = column(1.0, &[("NaCl", 1e-4)]);
    let events = bench
        .step_with(
            transport(4, 0.5, false),
            &mut Noop,
            &kerotakis_core::PermissiveScreen,
        )
        .unwrap();
    assert_eq!(bench.vessels[3].moles_of(&SpeciesId::new("NaCl")).0, 1e-4);
    let Some(Event::Transported {
        inflow_moles,
        effluent_moles,
        drain,
        ..
    }) = events
        .iter()
        .find(|e| matches!(e, Event::Transported { .. }))
    else {
        panic!("{events:?}");
    };
    assert!(!drain);
    // Open-boundary ledger: column + receiver hold exactly the inflow.
    let id = SpeciesId::new("NaCl");
    let inflow: f64 = inflow_moles
        .iter()
        .filter(|(s, _)| *s == id)
        .map(|(_, m)| m.0)
        .sum();
    assert_relative(inflow, 2e-4, 1e-12, "reservoir inflow");
    let held: f64 = [0, 1, 2, 4]
        .iter()
        .map(|i| bench.vessels[*i].moles_of(&id).0)
        .sum();
    assert_relative(held, inflow, 1e-12, "reservoir column + receiver");
    let effluent: f64 = effluent_moles
        .iter()
        .filter(|(s, _)| *s == id)
        .map(|(_, m)| m.0)
        .sum();
    assert_relative(
        bench.vessels[4].moles_of(&id).0,
        effluent,
        1e-12,
        "receiver",
    );
}

#[test]
fn reactions_run_after_every_step_so_an_excess_bed_retains_the_ion() {
    // Bed of 1e-3 mol in cell 0; the default reservoir carries 1e-4 mol per
    // cell volume, and 9 x 0.5 = 4.5 cell volumes deliver 4.5e-4 mol, well
    // under the bed.
    let mut bench = column(1.0, &[("toy-ion", 1e-4)]);
    bench.vessels[0].deposit(SpeciesId::new("toy-bed"), Moles(1e-3), Phase::Solid);
    let mut solver = ToyCementation { calls: 0 };
    bench
        .step_with(
            transport(9, 0.5, false),
            &mut solver,
            &kerotakis_core::PermissiveScreen,
        )
        .unwrap();
    // Every step re-equilibrated all three cells (eight in-loop passes, the
    // ninth is the ordinary post-operator pass over touched vessels).
    assert!(solver.calls >= 9 * 3, "calls {}", solver.calls);
    let ion = SpeciesId::new("toy-ion");
    let deposit = SpeciesId::new("toy-deposit");
    assert_eq!(bench.vessels[4].moles_of(&ion).0, 0.0, "no breakthrough");
    for cell in 1..=2 {
        assert_eq!(bench.vessels[cell].moles_of(&ion).0, 0.0);
    }
    assert_relative(
        bench.vessels[0].moles_of(&deposit).0,
        4.5e-4,
        1e-12,
        "deposit retained in the bed cell",
    );
    // The product is mobile and moves on.
    assert!(bench.vessels[4].moles_of(&SpeciesId::new("toy-product")).0 > 0.0);
    // Atom-level closure of the toy chemistry over every vessel.
    // Open ledger: everything except the unchanged reservoir (v3) holds
    // exactly the 4.5e-4 mol the reservoir supplied.
    assert_eq!(bench.vessels[3].moles_of(&ion).0, 1e-4);
    let ion_side = bench_total(&bench, "toy-ion") - 1e-4 + bench_total(&bench, "toy-deposit");
    assert_relative(ion_side, 4.5e-4, 1e-12, "ion + deposit");
    let bed_side = bench_total(&bench, "toy-bed") + bench_total(&bench, "toy-deposit");
    assert_relative(bed_side, 1e-3, 1e-12, "bed + deposit");
}

#[test]
fn hydraulic_volume_tolerance_is_relative_and_documented() {
    fn cell(id: usize, water: f64) -> Vessel {
        let mut vessel = Vessel::new(VesselId(id), format!("cell {id}"));
        water_cell(&mut vessel, water);
        vessel
    }
    assert_eq!(HYDRAULIC_VOLUME_RELATIVE_TOLERANCE, 1e-3);
    // Hydrate water from a dissolving mineral: ~564 ppm. Same geometry.
    assert!(CellChain::new(vec![cell(0, WATER * (1.0 + 5.64e-4)), cell(1, WATER)]).is_ok());
    // A real geometry difference is still refused.
    assert!(matches!(
        CellChain::new(vec![cell(0, WATER), cell(1, WATER * 1.002)]),
        Err(TransportError::NonUniformCellVolume { cell: 1, .. })
    ));
    // A feed that is short by solvent-proxy rounding still delivers a
    // full cell volume; one that is materially short is refused.
    let mut chain = CellChain::new(vec![cell(0, WATER)]).unwrap();
    let mut feed = cell(9, WATER * (1.0 - 1.6e-6));
    chain.advance_from_feed(&mut feed, 1.0, 1).unwrap();
    assert_eq!(feed.moles_of(&SpeciesId::new("water")).0, 0.0);
    let mut chain = CellChain::new(vec![cell(0, WATER)]).unwrap();
    let mut short = cell(9, WATER * 0.99);
    assert!(matches!(
        chain.advance_from_feed(&mut short, 1.0, 1),
        Err(TransportError::InletExhausted { step: 1, .. })
    ));
    assert_eq!(short.moles_of(&SpeciesId::new("water")).0, WATER * 0.99);
}
