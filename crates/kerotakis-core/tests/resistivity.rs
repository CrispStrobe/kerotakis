//! Why electrical wire is copper and not iron (curiosity corpus mat-011).
//!
//! The bench already had a conductivity meter, and it was a Kohlrausch sum
//! over a solved speciation: a *solution* instrument. Asked about a copper
//! wire it said "no aqueous solution has been characterised", which is true
//! and answers nothing, because a metal does not conduct the way a salt
//! solution does. There are no ions moving through a wire.
//!
//! So the registry grew a curated `electrical_resistivity` for the dry
//! solids the shelf actually holds, and `conductivity::dry_solid_conductance`
//! reads it. These tests pin three things that a sign check would not: the
//! numbers themselves, the ordering they put the metals in, and — the point
//! of the whole exercise — the refusals, because a meter that answers a
//! question it cannot answer is worse than one that says nothing.

use kerotakis_core::conductivity::dry_solid_conductance;
use kerotakis_core::script::parse_op;
use kerotakis_core::solve::SolverStack;
use kerotakis_core::species::lookup_key;
use kerotakis_core::vessel::{Vessel, VesselId};
use kerotakis_core::{Bench, HonestyEquilibrator, MixingEquilibrator, PermissiveScreen};

fn run(commands: &[&str]) -> Bench {
    let mut bench = Bench::new();
    let mut solver = SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(HonestyEquilibrator),
    ]);
    for command in commands {
        let op = parse_op(command)
            .unwrap_or_else(|error| panic!("parse {command}: {error}"))
            .expect("operator");
        bench
            .step_with(op, &mut solver, &PermissiveScreen)
            .unwrap_or_else(|error| panic!("{command}: {error}"));
    }
    bench
}

fn vessel(bench: &Bench) -> &Vessel {
    bench.vessel(VesselId(0)).expect("v1")
}

/// The whole of mat-011, as a number rather than a slogan. Iron is not
/// "worse"; it is 5.8 times more resistive, so an iron wire of the same
/// gauge wastes 5.8 times the power carrying the same current.
#[test]
fn iron_is_almost_six_times_as_resistive_as_copper() {
    let copper = dry_solid_conductance(vessel(&run(&["add v1 Cu 1g"]))).expect("copper reads");
    let iron = dry_solid_conductance(vessel(&run(&["add v1 Fe 1g"]))).expect("iron reads");

    assert_eq!(copper.resistivity_ohm_m, 1.678e-8);
    assert_eq!(iron.resistivity_ohm_m, 9.71e-8);
    let ratio = iron.resistivity_ohm_m / copper.resistivity_ohm_m;
    assert!(
        (ratio - 5.786).abs() < 0.01,
        "iron/copper resistivity ratio is 5.79, got {ratio}"
    );
    // The reciprocal is what a conductance meter reads, and it must be the
    // reciprocal of the number the registry stores rather than a second
    // curated value that could drift away from it.
    assert!((copper.conductivity_s_per_m * copper.resistivity_ohm_m - 1.0).abs() < 1e-12);
    assert!(
        (copper.conductivity_s_per_m - 5.96e7).abs() / 5.96e7 < 0.005,
        "copper reads about 5.96e7 S/m, got {}",
        copper.conductivity_s_per_m
    );
}

/// Silver beats copper and copper beats everything else on the shelf. This
/// is the ordering a handbook column has, and getting it wrong would mean
/// the values were transcribed into the wrong rows.
#[test]
fn the_shelf_orders_the_way_the_handbook_does() {
    let rho = |key: &str| {
        let command = format!("add v1 {key} 1g");
        dry_solid_conductance(vessel(&run(&[command.as_str()])))
            .unwrap_or_else(|| panic!("{key} reads"))
            .resistivity_ohm_m
    };
    let ag = rho("Ag");
    let cu = rho("Cu");
    let al = rho("Al");
    let mg = rho("Mg");
    let zn = rho("Zn");
    let fe = rho("Fe");
    assert!(ag < cu, "silver is the least resistive metal: {ag} vs {cu}");
    assert!(
        cu < al && al < mg && mg < zn && zn < fe,
        "{cu} {al} {mg} {zn} {fe}"
    );
    // Silver's margin over copper is small — a few percent — which is why
    // the wire in the wall is copper. A test that only checked the order
    // would pass with silver ten times better, and that would be wrong.
    assert!(
        (cu / ag - 1.0) < 0.10,
        "silver beats copper by under 10%, got {}",
        cu / ag
    );
    // Aluminium is more resistive by volume and much lighter, which is the
    // whole overhead-line trade. Per unit MASS it wins, and the registry
    // holds both numbers, so the comparison is arithmetic rather than a
    // claim anybody typed.
    let density = |key: &str| lookup_key(key).expect("curated").density;
    assert!(
        al * density("Al") < cu * density("Cu"),
        "aluminium conducts better per kilogram than copper does"
    );
}

/// Graphite conducts — a pencil line completes a circuit — and it is still
/// two orders of magnitude worse than the worst metal here. The row is an
/// order of magnitude and says so; the test only asks for what an order of
/// magnitude can support.
#[test]
fn graphite_is_a_conductor_and_a_poor_one() {
    let graphite = dry_solid_conductance(vessel(&run(&["add v1 graphite 1g"]))).expect("reads");
    let iron = dry_solid_conductance(vessel(&run(&["add v1 Fe 1g"]))).expect("reads");
    assert!(graphite.resistivity_ohm_m > 100.0 * iron.resistivity_ohm_m);
    assert!(
        graphite
            .boundary
            .is_some_and(|note| note.contains("ORDER OF MAGNITUDE")),
        "the graphite row must announce that it is not a measurement"
    );
}

/// Every reading carries the book it came from, and that book says the
/// lane is not cleared. A number printed without its provenance is a claim
/// the reader cannot check.
#[test]
fn every_reading_carries_its_citation_and_its_caveat() {
    let reading = dry_solid_conductance(vessel(&run(&["add v1 Cu 1g"]))).expect("reads");
    assert!(reading.source.contains("electrical-resistivity tranche v1"));
    assert!(
        reading.source.contains("PENDING REVIEW"),
        "the tranche's provenance lane is not cleared and the reading must say so"
    );
    assert!(reading.boundary.is_some_and(|note| !note.is_empty()));
}

/// The four refusals. Each is a case where an answer would be wrong rather
/// than merely absent.
#[test]
fn the_meter_refuses_what_it_cannot_read() {
    // A solid with no curated resistivity. Table salt is an insulator dry
    // and a conductor dissolved, and this bench claims neither.
    assert!(dry_solid_conductance(vessel(&run(&["add v1 NaCl 1g"]))).is_none());
    // Two metals touching are a circuit with a geometry; this reading has
    // no geometry.
    assert!(dry_solid_conductance(vessel(&run(&["add v1 Cu 1g", "add v1 Zn 1g"]))).is_none());
    // A wet wire conducts through the film on it, which is neither model.
    assert!(dry_solid_conductance(vessel(&run(&["add v1 Cu 1g", "add v1 water 50mL"]))).is_none());
    // And an empty vessel reads nothing at all.
    assert!(dry_solid_conductance(vessel(&run(&[]))).is_none());
}

/// The solution meter is untouched. A copper wire standing in salt water is
/// a solution measurement, because that is what the probe would read.
#[test]
fn an_aqueous_vessel_stays_the_solution_meters_business() {
    let bench = run(&["add v1 water 100mL", "add v1 NaCl 1g", "add v1 Cu 1g"]);
    assert!(dry_solid_conductance(vessel(&bench)).is_none());
}

// ── The bench arm: `measure <vessel> conductivity` on a dry metal ────────

#[test]
fn the_meter_on_the_bench_reads_a_dry_wire_in_siemens_per_metre() {
    use kerotakis_core::*;
    let mut bench = Bench::new();
    let v = VesselId(0);
    bench
        .step(Operator::Add {
            vessel: v,
            species: SpeciesId::new("Cu"),
            moles: Moles(0.0157),
            at: None,
        })
        .expect("add");
    let events = bench
        .step(Operator::Measure {
            vessel: v,
            instrument: Instrument::ConductivityMeter,
        })
        .expect("measure");
    let reading = events
        .iter()
        .find_map(|e| match e {
            Event::Measured { value, unit, .. } => Some((*value, unit.clone())),
            _ => None,
        })
        .expect("a reading, not a refusal");
    assert_eq!(reading.1, "S/m");
    assert!(
        (reading.0 - 1.0 / 1.678e-8).abs() / reading.0 < 1e-9,
        "{}",
        reading.0
    );
}

#[test]
fn the_meter_on_the_bench_names_the_missing_datum_for_an_uncurated_solid() {
    use kerotakis_core::*;
    let mut bench = Bench::new();
    let v = VesselId(0);
    bench
        .step(Operator::Add {
            vessel: v,
            species: SpeciesId::new("NaCl"),
            moles: Moles(0.01),
            at: None,
        })
        .expect("add");
    let events = bench
        .step(Operator::Measure {
            vessel: v,
            instrument: Instrument::ConductivityMeter,
        })
        .expect("measure");
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::NotYetModeled { what, .. } if what.contains("no electrical resistivity")
        )),
        "{events:?}"
    );
    assert!(!events.iter().any(|e| matches!(e, Event::Measured { .. })));
}
