//! BRD-041: the shipped gas mechanism packs, reached from the bench's slow
//! clock. `kerotakis-core/tests/gas_mechanism_packs.rs` proves the packs
//! against the integrator; this file proves the route a `wait` takes to
//! them, and the heat that comes out.

use kerotakis_core::clock::{advance, ClockContext};
use kerotakis_core::kinetics::packs::shipped;
use kerotakis_core::species::Phase;
use kerotakis_core::units::{Kelvin, Liters, Moles};
use kerotakis_core::vessel::{Headspace, Vessel, VesselId};
use kerotakis_core::{Event, SpeciesId};

fn reactor(volume_litres: f64, temperature_k: f64, feeds: &[(&str, f64)]) -> Vessel {
    let mut vessel = Vessel::new(VesselId(0), "reactor");
    vessel.temperature = Kelvin(temperature_k);
    vessel.headspace = Headspace::Sealed {
        volume: Liters(volume_litres),
    };
    for (species, moles) in feeds {
        vessel.deposit(SpeciesId::new(species), Moles(*moles), Phase::Gas);
    }
    vessel.refresh_pressure();
    vessel
}

fn moles(vessel: &Vessel, species: &str) -> f64 {
    vessel.moles_of(&SpeciesId::new(species)).0
}

#[test]
fn a_hot_seeded_hydrogen_vessel_burns_on_the_clock_and_heats_up() {
    let mut feeds = vec![("H2", 2.0e-3), ("O2", 1.0e-3), ("N2", 4.0e-3)];
    for radical in ["H", "O", "OH", "HO2", "H2O2", "water"] {
        feeds.push((radical, 1e-7));
    }
    let mut vessel = reactor(1.0, 1200.0, &feeds);
    let mut events = Vec::new();
    advance(&mut vessel, 1.0e-2, ClockContext::default(), &mut events).expect("the clock advances");

    let reacted: Vec<&String> = events
        .iter()
        .filter_map(|e| match e {
            Event::Reacted { reaction, .. } => Some(reaction),
            _ => None,
        })
        .collect();
    assert!(
        !reacted.is_empty(),
        "the pack's steps should report: {events:?}"
    );
    assert!(
        moles(&vessel, "H2") < 0.2e-3,
        "most hydrogen burned: {}",
        moles(&vessel, "H2")
    );
    assert!(
        moles(&vessel, "water") > 1.8e-3,
        "into water: {}",
        moles(&vessel, "water")
    );
    let heat = events.iter().find_map(|e| match e {
        Event::ReactionHeatReleased {
            reaction, energy_j, ..
        } if reaction == "h2-o2-skeletal-v1" => Some(*energy_j),
        _ => None,
    });
    // 2 H2 + O2 → 2 H2O(g): 483.6 kJ per 2 mol H2 → ~484 J for 2 mmol.
    let heat = heat.expect("the burn's heat is reported");
    assert!((400.0..500.0).contains(&heat), "released {heat} J");
    assert!(
        vessel.temperature.0 > 1200.0,
        "an adiabatic vessel warms: {} K",
        vessel.temperature.0
    );
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::TemperatureChanged { .. })));
}

#[test]
fn a_cold_fuel_and_air_mixture_does_nothing_on_the_clock() {
    let mut vessel = reactor(1.0, 298.15, &[("methane", 0.01), ("O2", 0.03), ("N2", 0.1)]);
    let mut events = Vec::new();
    advance(&mut vessel, 3600.0, ClockContext::default(), &mut events).expect("advances");
    assert!(
        !events.iter().any(|e| matches!(
            e,
            Event::Reacted { .. } | Event::ReactionHeatReleased { .. }
        )),
        "{events:?}"
    );
    assert!((moles(&vessel, "methane") - 0.01).abs() < 1e-12);
    assert!((vessel.temperature.0 - 298.15).abs() < 1e-9);
}

#[test]
fn a_vessel_with_one_pack_species_is_not_asked() {
    let lone = reactor(1.0, 1200.0, &[("O2", 0.01)]);
    for pack in shipped() {
        assert!(!pack.matches(&lone), "{}", pack.id);
    }
    // Two inert-to-each-other species match a pack and integrate to
    // nothing, which is also what happens on a bench.
    let mut vessel = reactor(1.0, 1200.0, &[("O2", 0.01), ("N2", 0.04)]);
    let mut events = Vec::new();
    advance(&mut vessel, 60.0, ClockContext::default(), &mut events).expect("advances");
    assert!(!events.iter().any(|e| matches!(e, Event::Reacted { .. })));
}
