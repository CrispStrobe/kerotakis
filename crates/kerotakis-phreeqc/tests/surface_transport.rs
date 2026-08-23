//! AQ-013: typed surface transport against PHREEQC's development-only oracle.

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::{databases, Phreeqc, PhreeqcEquilibrator};

const CELL_COUNT: usize = 4;
const SHIFTS: usize = 20;
const WATER_MOLES: f64 = 5.5509;
const WATER_KG: f64 = WATER_MOLES * 18.015 / 1000.0;
const FEED_ZINC_MOLES: f64 = 1e-4;

fn hfo(cell: usize) -> SurfaceSites {
    SurfaceSites {
        label: format!("HFO cell {cell}"),
        model: SurfaceModel::HydrousFerricOxide,
        mass: Grams(0.09),
        specific_area_m2_per_g: 600.0,
        strong_capacity: Moles(5e-6),
        weak_capacity: Moles(2e-4),
        occupancy: Vec::new(),
        water_release: Moles(0.0),
    }
}

fn column_cell(cell: usize) -> Vessel {
    let mut vessel = Vessel::new(VesselId(cell), format!("adsorption cell {cell}"));
    vessel.deposit(SpeciesId::new("water"), Moles(WATER_MOLES), Phase::Liquid);
    vessel.surfaces.push(hfo(cell));
    vessel
}

fn zinc_sulfate_feed(solver: &mut PhreeqcEquilibrator) -> Vessel {
    let mut feed = Vessel::new(VesselId(999), "zinc sulfate feed");
    feed.deposit(SpeciesId::new("water"), Moles(WATER_MOLES), Phase::Liquid);
    feed.deposit(
        SpeciesId::new("Zn+2"),
        Moles(FEED_ZINC_MOLES),
        Phase::Aqueous,
    );
    feed.deposit(
        SpeciesId::new("SO4-2"),
        Moles(FEED_ZINC_MOLES),
        Phase::Aqueous,
    );
    solver
        .equilibrate(&mut feed)
        .expect("equilibrated zinc-sulfate feed");
    feed
}

fn bound(vessel: &Vessel, sorbate: SurfaceSorbate) -> f64 {
    vessel
        .surfaces
        .iter()
        .map(|surface| surface.bound(sorbate).0)
        .sum()
}

fn cell_inventory(vessel: &Vessel, sorbate: SurfaceSorbate) -> f64 {
    vessel.moles_of(&sorbate.species()).0 + bound(vessel, sorbate)
}

fn chain_inventory(chain: &CellChain, sorbate: SurfaceSorbate) -> f64 {
    chain
        .cells()
        .iter()
        .map(|cell| cell_inventory(cell, sorbate))
        .sum()
}

fn parcel_inventory(parcel: &MobileParcel, sorbate: SurfaceSorbate) -> f64 {
    parcel.moles_of(&sorbate.species()).0
}

fn assert_step_ledger(
    before: f64,
    after: f64,
    injected: f64,
    effluent: f64,
    sorbate: SurfaceSorbate,
    shift: usize,
    chain: &CellChain,
) {
    let residual = before + injected - after - effluent;
    let cells: Vec<_> = chain
        .cells()
        .iter()
        .map(|cell| {
            (
                cell.moles_of(&sorbate.species()).0,
                bound(cell, sorbate),
                cell.surfaces
                    .iter()
                    .map(|surface| surface.water_release.0)
                    .sum::<f64>(),
            )
        })
        .collect();
    assert!(
        residual.abs() < 2e-8,
        "{sorbate:?} ledger failed after shift {shift}: before={before:.12e}, injected={injected:.12e}, after={after:.12e}, effluent={effluent:.12e}, residual={residual:.12e}, cells(dissolved,bound,released_water)={cells:?}"
    );
}

fn typed_front() -> Vec<f64> {
    let mut solver = PhreeqcEquilibrator::new().expect("engine");
    let feed = zinc_sulfate_feed(&mut solver);
    let feed_molality = feed.moles_of(&SurfaceSorbate::Zinc.species()).0
        / (feed.moles_of(&SpeciesId::new("water")).0 * 18.015 / 1000.0);
    let mut chain = CellChain::new((0..CELL_COUNT).map(column_cell).collect()).unwrap();
    let mut front = Vec::with_capacity(SHIFTS);

    for shift in 1..=SHIFTS {
        let zinc_before = chain_inventory(&chain, SurfaceSorbate::Zinc);
        let sulfate_before = chain_inventory(&chain, SurfaceSorbate::Sulfate);
        let step = chain
            .advance_reactive(&feed, 1.0, &mut solver)
            .expect("transport and local surface equilibrium");

        assert_step_ledger(
            zinc_before,
            chain_inventory(&chain, SurfaceSorbate::Zinc),
            parcel_inventory(&step.transport.injected, SurfaceSorbate::Zinc),
            parcel_inventory(&step.transport.effluent, SurfaceSorbate::Zinc),
            SurfaceSorbate::Zinc,
            shift,
            &chain,
        );
        assert_step_ledger(
            sulfate_before,
            chain_inventory(&chain, SurfaceSorbate::Sulfate),
            parcel_inventory(&step.transport.injected, SurfaceSorbate::Sulfate),
            parcel_inventory(&step.transport.effluent, SurfaceSorbate::Sulfate),
            SurfaceSorbate::Sulfate,
            shift,
            &chain,
        );

        let outlet = chain.cells().last().unwrap();
        let water_kg = outlet.moles_of(&SpeciesId::new("water")).0 * 18.015 / 1000.0;
        front.push(outlet.moles_of(&SurfaceSorbate::Zinc.species()).0 / water_kg / feed_molality);
    }

    assert!(chain
        .cells()
        .iter()
        .all(|cell| cell.surfaces.iter().all(SurfaceSites::has_valid_capacity)));
    front
}

/// Test-only oracle. The shipped app does not call PHREEQC `TRANSPORT`;
/// production uses `CellChain::advance_reactive` above.
fn phreeqc_transport_oracle() -> Vec<f64> {
    let feed_molality = FEED_ZINC_MOLES / WATER_KG;
    let input = format!(
        r#"
PRINT
    -selected_output false
SOLUTION 0 zinc sulfate feed
    units mol/kgw
    temp 25
    pH 7
    Zn {feed_molality:.12e}
    S(6) {feed_molality:.12e}
    -water {WATER_KG:.12e}
SOLUTION 1-{CELL_COUNT} initial column water
    units mol/kgw
    temp 25
    pH 7
    -water {WATER_KG:.12e}
SURFACE 1-{CELL_COUNT}
    Hfo_sOH 5.000000000000e-6 600 0.09
    Hfo_wOH 2.000000000000e-4
    -equilibrate 1
END
PRINT
    -selected_output true
    -status false
SELECTED_OUTPUT 1
    -reset false
    -high_precision false
    -totals Zn
TRANSPORT
    -cells {CELL_COUNT}
    -shifts {SHIFTS}
    -time_step 1
    -flow_direction forward
    -boundary_conditions flux flux
    -lengths {CELL_COUNT}*1
    -dispersivities {CELL_COUNT}*0
    -diffusion_coefficient 0
    -correct_disp true
    -punch_cells {CELL_COUNT}
    -punch_frequency 1
    -print_cells {CELL_COUNT}
    -print_frequency {SHIFTS}
    -warnings false
END
"#
    );
    let mut engine = Phreeqc::with_database(databases::WATEQ4F).expect("load oracle database");
    engine.run(&input).expect("PHREEQC TRANSPORT oracle");
    let rows = engine.selected_output();
    let header = rows.first().expect("selected-output header");
    let zinc = header
        .iter()
        .position(|heading| heading == "Zn")
        .expect("dissolved-zinc column");
    let mut curve: Vec<f64> = rows
        .iter()
        .skip(1)
        .filter_map(|row| row.get(zinc)?.parse::<f64>().ok())
        .map(|molality| molality / feed_molality)
        .collect();
    assert_eq!(curve.len(), SHIFTS + 1, "oracle rows: {rows:#?}");
    let initial = curve.remove(0);
    assert!(
        initial.abs() < 1e-12,
        "PHREEQC's pre-transport outlet must be zinc-free, got {initial}"
    );
    curve
}

fn half_breakthrough(curve: &[f64]) -> usize {
    curve
        .iter()
        .position(|fraction| *fraction >= 0.5)
        .unwrap_or(curve.len())
}

#[test]
fn typed_hfo_front_tracks_phreeqc_transport_oracle() {
    let typed = typed_front();
    let oracle = phreeqc_transport_oracle();

    assert!(
        typed[0] < 1e-8 && typed[SHIFTS - 1] > 0.8,
        "typed column must show finite breakthrough: {typed:?}"
    );
    let typed_half = half_breakthrough(&typed);
    let oracle_half = half_breakthrough(&oracle);
    assert!(
        typed_half.abs_diff(oracle_half) <= 1,
        "half-breakthrough must agree within one shift: typed={typed:?}, oracle={oracle:?}"
    );
    // At four cells and Courant one, the independently implemented schemes
    // resolve the advancing front at one-sample granularity. Gate the whole
    // curve while retaining a ceiling for any single normalized sample.
    let deltas: Vec<f64> = typed
        .iter()
        .zip(&oracle)
        .map(|(typed, oracle)| (typed - oracle).abs())
        .collect();
    let mean_delta = deltas.iter().sum::<f64>() / deltas.len() as f64;
    let max_delta = deltas.iter().copied().fold(0.0_f64, f64::max);
    assert!(
        mean_delta < 0.025 && max_delta < 0.25,
        "normalized outlet fronts differ by mean={mean_delta:.6}, max={max_delta:.6}: typed={typed:?}, oracle={oracle:?}"
    );
}
