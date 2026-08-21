//! AQ-012: a finite sodium softener column under conservative advection.

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

const CELL_COUNT: usize = 4;
const WATER_MOLES: f64 = 5.5509;
const RESIN_CAPACITY_EQ: f64 = 5e-4;
const FEED_CALCIUM_MOLES: f64 = 2.5e-4;

fn sodium_resin(cell: usize) -> ExchangeSites {
    ExchangeSites {
        label: format!("sodium resin {cell}"),
        dry_mass: Grams(1.0),
        capacity: Moles(RESIN_CAPACITY_EQ),
        occupancy: vec![ExchangeOccupancy {
            ion: ExchangeIon::Sodium,
            moles: Moles(RESIN_CAPACITY_EQ),
        }],
    }
}

fn column_cell(cell: usize) -> Vessel {
    let mut vessel = Vessel::new(VesselId(cell), format!("softener cell {cell}"));
    vessel.deposit(SpeciesId::new("water"), Moles(WATER_MOLES), Phase::Liquid);
    vessel.exchanges.push(sodium_resin(cell));
    vessel
}

fn hard_water_feed(solver: &mut PhreeqcEquilibrator) -> Vessel {
    let mut feed = Vessel::new(VesselId(999), "calcium feed");
    feed.deposit(SpeciesId::new("water"), Moles(WATER_MOLES), Phase::Liquid);
    feed.deposit(
        SpeciesId::new("Ca+2"),
        Moles(FEED_CALCIUM_MOLES),
        Phase::Aqueous,
    );
    feed.deposit(
        SpeciesId::new("Cl-"),
        Moles(2.0 * FEED_CALCIUM_MOLES),
        Phase::Aqueous,
    );
    solver
        .equilibrate(&mut feed)
        .expect("equilibrated hard-water feed");
    feed
}

fn bound(vessel: &Vessel, ion: ExchangeIon) -> f64 {
    vessel
        .exchanges
        .iter()
        .map(|exchange| exchange.bound(ion).0)
        .sum()
}

fn cell_inventory(vessel: &Vessel, ion: ExchangeIon) -> f64 {
    vessel.moles_of(&ion.species()).0 + bound(vessel, ion)
}

fn chain_inventory(chain: &CellChain, ion: ExchangeIon) -> f64 {
    chain
        .cells()
        .iter()
        .map(|cell| cell_inventory(cell, ion))
        .sum()
}

fn parcel_inventory(parcel: &MobileParcel, ion: ExchangeIon) -> f64 {
    parcel.moles_of(&ion.species()).0
}

fn assert_step_ledger(
    before: f64,
    after: f64,
    injected: f64,
    effluent: f64,
    ion: ExchangeIon,
    pore_volume: usize,
) {
    let residual = before + injected - after - effluent;
    assert!(
        residual.abs() < 2e-8,
        "{ion:?} ledger failed after pore volume {pore_volume}: before={before:.12e}, injected={injected:.12e}, after={after:.12e}, effluent={effluent:.12e}, residual={residual:.12e}"
    );
}

#[test]
fn sodium_softener_has_a_finite_calcium_breakthrough_curve() {
    let mut solver = PhreeqcEquilibrator::new().expect("engine");
    let feed = hard_water_feed(&mut solver);
    let feed_calcium = feed.moles_of(&ExchangeIon::Calcium.species()).0;
    assert!(feed_calcium > 0.99 * FEED_CALCIUM_MOLES);
    let mut chain = CellChain::new((0..CELL_COUNT).map(column_cell).collect()).unwrap();
    let mut breakthrough = Vec::new();

    for pore_volume in 1..=12 {
        let calcium_before = chain_inventory(&chain, ExchangeIon::Calcium);
        let sodium_before = chain_inventory(&chain, ExchangeIon::Sodium);
        let step = chain
            .advance_reactive(&feed, 1.0, &mut solver)
            .expect("transport and local exchange equilibrium");
        assert_eq!(step.reactions.len(), CELL_COUNT);
        assert!(chain.cells().iter().all(|cell| cell.solution.is_some()));

        let calcium_after = chain_inventory(&chain, ExchangeIon::Calcium);
        let sodium_after = chain_inventory(&chain, ExchangeIon::Sodium);
        assert_step_ledger(
            calcium_before,
            calcium_after,
            parcel_inventory(&step.transport.injected, ExchangeIon::Calcium),
            parcel_inventory(&step.transport.effluent, ExchangeIon::Calcium),
            ExchangeIon::Calcium,
            pore_volume,
        );
        assert_step_ledger(
            sodium_before,
            sodium_after,
            parcel_inventory(&step.transport.injected, ExchangeIon::Sodium),
            parcel_inventory(&step.transport.effluent, ExchangeIon::Sodium),
            ExchangeIon::Sodium,
            pore_volume,
        );

        breakthrough
            .push(parcel_inventory(&step.transport.effluent, ExchangeIon::Calcium) / feed_calcium);
    }

    assert!(
        breakthrough[0] < 1e-8,
        "unused column must initially retain calcium: {breakthrough:?}"
    );
    assert!(
        breakthrough[11] > 0.8,
        "finite resin must eventually approach feed concentration: {breakthrough:?}"
    );
    assert!(
        breakthrough[11] > breakthrough[5] + 0.25,
        "the result must be a rising breakthrough curve: {breakthrough:?}"
    );
    assert!(chain
        .cells()
        .iter()
        .all(|cell| cell.exchanges.iter().all(ExchangeSites::has_valid_capacity)));
    assert!(
        chain
            .cells()
            .iter()
            .map(|cell| bound(cell, ExchangeIon::Calcium))
            .sum::<f64>()
            > 0.0
    );
}
