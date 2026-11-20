//! G4: thermal decomposition of carbonates and hydroxides as a CLASS.
//!
//! Nothing here names a reaction. Each member is heated through the same
//! open-air thermal route, which finds the oxide and the gas by Gibbs
//! minimisation over the records its registry species map to: NASA CEA's
//! file for chalk, magnesite, slaked lime, brucite and copper(II) hydroxide,
//! and the Robie & Hemingway (1995) supplement for smithsonite and zincite.
//! The table below is only the test's expectation of what that search
//! should land on.
//!
//! Every row also checks the metal: it must all be accounted for between
//! the parent solid and its oxide, because a decomposition that loses or
//! invents metal is not a decomposition.

use kerotakis_cea::ThermalEquilibrator;
use kerotakis_core::*;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(ThermalEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn step(bench: &mut Bench, stack: &mut SolverStack, op: Operator) -> Vec<Event> {
    bench.step_with(op, stack, &PermissiveScreen).expect("step")
}

fn add(bench: &mut Bench, stack: &mut SolverStack, key: &str, moles: f64) {
    step(
        bench,
        stack,
        Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new(key),
            moles: Moles(moles),
            at: None,
        },
    );
}

fn heat(bench: &mut Bench, stack: &mut SolverStack, kj: f64) -> Vec<Event> {
    step(
        bench,
        stack,
        Operator::Heat {
            vessel: VesselId(0),
            energy: Joules(kj * 1000.0),
            source: None,
        },
    )
}

struct Member {
    parent: &'static str,
    oxide: &'static str,
    gas: &'static str,
    /// Metal atoms per formula unit of parent and oxide.
    metal_per_parent: f64,
    metal_per_oxide: f64,
    moles: f64,
    kj: f64,
}

const CLASS: &[Member] = &[
    // Carbonates → oxide + CO2.
    Member {
        parent: "CaCO3",
        oxide: "CaO",
        gas: "CO2",
        metal_per_parent: 1.0,
        metal_per_oxide: 1.0,
        moles: 0.1,
        kj: 40.0,
    },
    Member {
        parent: "MgCO3",
        oxide: "MgO",
        gas: "CO2",
        metal_per_parent: 1.0,
        metal_per_oxide: 1.0,
        moles: 0.05,
        kj: 12.0,
    },
    Member {
        parent: "ZnCO3",
        oxide: "ZnO",
        gas: "CO2",
        metal_per_parent: 1.0,
        metal_per_oxide: 1.0,
        moles: 0.05,
        kj: 10.0,
    },
    // Hydroxides → oxide + water (as steam, vented).
    Member {
        parent: "Ca(OH)2",
        oxide: "CaO",
        gas: "water",
        metal_per_parent: 1.0,
        metal_per_oxide: 1.0,
        moles: 0.05,
        kj: 12.0,
    },
    Member {
        parent: "Mg(OH)2",
        oxide: "MgO",
        gas: "water",
        metal_per_parent: 1.0,
        metal_per_oxide: 1.0,
        moles: 0.05,
        kj: 10.0,
    },
    Member {
        parent: "Cu(OH)2",
        oxide: "CuO",
        gas: "water",
        metal_per_parent: 1.0,
        metal_per_oxide: 1.0,
        moles: 0.05,
        kj: 4.0,
    },
];

#[test]
fn every_member_maps_into_the_thermal_data() {
    for m in CLASS {
        assert!(
            kerotakis_cea::cea_name(m.parent).is_some(),
            "{} has no record",
            m.parent
        );
        assert!(
            kerotakis_cea::cea_name(m.oxide).is_some(),
            "{} has no record",
            m.oxide
        );
    }
    assert_eq!(kerotakis_cea::cea_name("ZnO"), Some("ZnO(cr)"));
    assert_eq!(kerotakis_cea::cea_name("ZnCO3"), Some("ZnCO3(cr)"));
    assert_eq!(kerotakis_cea::cea_name("MgCO3"), Some("MgCO3(cr)"));
    assert!(
        kerotakis_cea::cea_name("Al2O3").is_some_and(|name| name.starts_with("AL2O3")),
        "corundum maps to NASA's alumina record by composition"
    );
}

#[test]
fn each_member_stays_put_at_room_temperature() {
    for m in CLASS {
        let mut bench = Bench::new();
        let mut stack = stack();
        add(&mut bench, &mut stack, m.parent, m.moles);
        let vessel = bench.vessel(VesselId(0)).unwrap();
        assert!(
            (vessel.moles_of(&SpeciesId::new(m.parent)).0 - m.moles).abs() < 1e-9,
            "{} is stable cold: {:?}",
            m.parent,
            vessel.contents
        );
        assert!(vessel.moles_of(&SpeciesId::new(m.oxide)).0 < 1e-12);
    }
}

#[test]
fn heating_each_member_leaves_its_oxide_and_drives_off_its_gas() {
    for m in CLASS {
        let mut bench = Bench::new();
        let mut stack = stack();
        add(&mut bench, &mut stack, m.parent, m.moles);
        let events = heat(&mut bench, &mut stack, m.kj);
        let vessel = bench.vessel(VesselId(0)).unwrap();
        let parent = vessel.moles_of(&SpeciesId::new(m.parent)).0;
        let oxide = vessel.moles_of(&SpeciesId::new(m.oxide)).0;
        let summary = format!(
            "{} → {}: T = {:.0} K, contents {:?}",
            m.parent, m.oxide, vessel.temperature.0, vessel.contents
        );
        assert!(
            oxide > 0.9 * m.moles * m.metal_per_parent / m.metal_per_oxide,
            "{summary}"
        );
        assert!(parent < 0.1 * m.moles, "{summary}");
        let metal = parent * m.metal_per_parent + oxide * m.metal_per_oxide;
        assert!(
            (metal - m.moles * m.metal_per_parent).abs() < 1e-9,
            "the metal is all accounted for: {summary}"
        );
        assert!(
            events.iter().any(|e| matches!(
                e,
                Event::GasEvolved { species, .. } if species.0 == m.gas
            )),
            "{} leaves as gas: {summary}; events {events:?}",
            m.gas
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::ThermalEquilibrium { .. })),
            "the thermal route reported itself: {summary}"
        );
    }
}

#[test]
fn smithsonite_carries_its_primary_source_into_the_answer() {
    let mut bench = Bench::new();
    let mut stack = stack();
    add(&mut bench, &mut stack, "ZnCO3", 0.05);
    let events = heat(&mut bench, &mut stack, 10.0);
    let provenance = events
        .iter()
        .find_map(|e| match e {
            Event::ThermalEquilibrium { provenance, .. } => Some(provenance.clone()),
            _ => None,
        })
        .expect("thermal equilibrium reported");
    assert!(
        provenance
            .dataset_sources
            .iter()
            .any(|source| source.contains("Robie & Hemingway 1995")),
        "the supplementary record names its source: {provenance:?}"
    );
}

#[test]
fn bare_zinc_does_not_gain_an_oxidation_route_from_the_supplement() {
    // The zincite record exists so zinc carbonate has somewhere to go. It
    // must not also turn a lump of zinc metal into oxide at 650 K: that is
    // an oxidation-kinetics claim the supplement does not make.
    let mut bench = Bench::new();
    let mut stack = stack();
    step(
        &mut bench,
        &mut stack,
        Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new("Zn"),
            moles: Moles(0.1),
            at: Some(Kelvin(650.0)),
        },
    );
    let vessel = bench.vessel(VesselId(0)).unwrap();
    assert!(
        (vessel.moles_of(&SpeciesId::new("Zn")).0 - 0.1).abs() < 1e-9,
        "{:?}",
        vessel.contents
    );
    assert!(vessel.moles_of(&SpeciesId::new("ZnO")).0 < 1e-12);
}

#[test]
fn malachite_is_registered_but_its_decomposition_is_not_computed() {
    // Robie & Hemingway tabulate malachite at 298.15 K only, with no Cp(T),
    // so it has no thermal record: the route declines the vessel rather
    // than invent one, and the green solid is still there afterwards.
    assert_eq!(kerotakis_cea::cea_name("Cu2(OH)2CO3"), None);
    let mut bench = Bench::new();
    let mut stack = stack();
    add(&mut bench, &mut stack, "Cu2(OH)2CO3", 0.01);
    let events = heat(&mut bench, &mut stack, 2.0);
    let vessel = bench.vessel(VesselId(0)).unwrap();
    assert!(
        (vessel.moles_of(&SpeciesId::new("Cu2(OH)2CO3")).0 - 0.01).abs() < 1e-12,
        "{:?}",
        vessel.contents
    );
    assert!(vessel.moles_of(&SpeciesId::new("CuO")).0 < 1e-12);
    assert!(!events
        .iter()
        .any(|e| matches!(e, Event::ThermalEquilibrium { .. })));
}

#[test]
fn bare_aluminium_does_not_gain_a_burning_route_from_corundum() {
    // Corundum is on the shelf now; aluminium foil in a hot crucible still
    // keeps its skin rather than burning away, because the route admits
    // alumina as a product only through an aluminium compound.
    let mut bench = Bench::new();
    let mut stack = stack();
    step(
        &mut bench,
        &mut stack,
        Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new("Al"),
            moles: Moles(0.1),
            at: Some(Kelvin(700.0)),
        },
    );
    let vessel = bench.vessel(VesselId(0)).unwrap();
    assert!(
        (vessel.moles_of(&SpeciesId::new("Al")).0 - 0.1).abs() < 1e-9,
        "{:?}",
        vessel.contents
    );
    assert!(vessel.moles_of(&SpeciesId::new("Al2O3")).0 < 1e-12);
}
