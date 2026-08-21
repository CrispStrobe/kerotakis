//! AQ-011: transport matter first, before adding reaction.

use kerotakis_core::*;

const WATER_MOLES: f64 = 5.5509;

struct FailingSecondCell {
    calls: usize,
}

impl Equilibrator for FailingSecondCell {
    fn name(&self) -> &'static str {
        "rollback-test"
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.calls += 1;
        vessel.label.push_str(" (reacted)");
        if self.calls == 2 {
            return Err(SolveError::NotConverged {
                solver: self.name().to_string(),
                detail: "deliberate second-cell failure".to_string(),
            });
        }
        Ok(Vec::new())
    }
}

fn cell(id: usize, tracer_moles: f64, temperature_k: f64) -> Vessel {
    let mut vessel = Vessel::new(VesselId(id), format!("cell {id}"));
    vessel.deposit(SpeciesId::new("water"), Moles(WATER_MOLES), Phase::Liquid);
    vessel.deposit(
        SpeciesId::new("passive-tracer"),
        Moles(tracer_moles),
        Phase::Aqueous,
    );
    vessel.temperature = Kelvin(temperature_k);
    vessel.solute_charge = tracer_moles * 0.25;
    vessel.solution = Some(SolutionInfo {
        pe: None,
        redox: Vec::new(),
        ph: 7.0,
        ionic_strength: 0.0,
        species: Vec::new(),
        provenance: None,
    });
    vessel
}

fn assert_ledger(
    chain_before_moles: f64,
    chain_before_charge: f64,
    chain_before_energy: f64,
    chain: &CellChain,
    step: &TransportStep,
) {
    let tracer = SpeciesId::new("passive-tracer");
    assert!(
        (chain_before_moles + step.injected.moles_of(&tracer).0
            - chain.total_moles(&tracer).0
            - step.effluent.moles_of(&tracer).0)
            .abs()
            < 1e-12
    );
    assert!(
        (chain_before_charge + step.injected.solute_charge
            - chain.total_solute_charge()
            - step.effluent.solute_charge)
            .abs()
            < 1e-12
    );
    assert!(
        (chain_before_energy + step.injected.sensible_energy().0
            - chain.total_sensible_energy().0
            - step.effluent.sensible_energy().0)
            .abs()
            < 1e-8
    );
}

#[test]
fn a_passive_tracer_follows_the_upwind_binomial_and_closes_every_ledger() {
    let cells = vec![
        cell(0, 1.0, 292.0),
        cell(1, 0.0, 296.0),
        cell(2, 0.0, 300.0),
        cell(3, 0.0, 304.0),
    ];
    let mut inlet = cell(99, 0.0, 310.0);
    inlet.solute_charge = 0.0;
    let mut chain = CellChain::new(cells).unwrap();
    let tracer = SpeciesId::new("passive-tracer");

    let before_moles = chain.total_moles(&tracer).0;
    let before_charge = chain.total_solute_charge();
    let before_energy = chain.total_sensible_energy().0;
    let first = chain.advance(&inlet, 0.5).unwrap();
    assert_ledger(before_moles, before_charge, before_energy, &chain, &first);
    let first_profile: Vec<f64> = chain
        .cells()
        .iter()
        .map(|cell| cell.moles_of(&tracer).0)
        .collect();
    assert_eq!(first_profile, vec![0.5, 0.5, 0.0, 0.0]);
    assert_eq!(first.effluent.moles_of(&tracer).0, 0.0);

    let before_moles = chain.total_moles(&tracer).0;
    let before_charge = chain.total_solute_charge();
    let before_energy = chain.total_sensible_energy().0;
    let second = chain.advance(&inlet, 0.5).unwrap();
    assert_ledger(before_moles, before_charge, before_energy, &chain, &second);
    let second_profile: Vec<f64> = chain
        .cells()
        .iter()
        .map(|cell| cell.moles_of(&tracer).0)
        .collect();
    assert_eq!(second_profile, vec![0.25, 0.5, 0.25, 0.0]);

    for cell in chain.cells() {
        assert!((cell.moles_of(&SpeciesId::new("water")).0 - WATER_MOLES).abs() < 1e-12);
        assert!(cell.solution.is_none(), "transport invalidates speciation");
    }
}

#[test]
fn stationary_solid_and_exchange_inventory_do_not_leave_their_cell() {
    let mut first = cell(0, 0.0, Kelvin::STANDARD.0);
    first.deposit(SpeciesId::new("CaCO3"), Moles(0.2), Phase::Solid);
    first.exchanges.push(ExchangeSites {
        label: "resin".to_string(),
        dry_mass: Grams(1.0),
        capacity: Moles(0.1),
        occupancy: vec![ExchangeOccupancy {
            ion: ExchangeIon::Sodium,
            moles: Moles(0.1),
        }],
    });
    let second = cell(1, 0.0, Kelvin::STANDARD.0);
    let inlet = cell(99, 0.0, Kelvin::STANDARD.0);
    let mut chain = CellChain::new(vec![first, second]).unwrap();

    let step = chain.advance(&inlet, 1.0).unwrap();

    assert_eq!(chain.cells()[0].moles_of(&SpeciesId::new("CaCO3")).0, 0.2);
    assert_eq!(chain.cells()[0].exchanges.len(), 1);
    assert_eq!(
        chain.cells()[0].exchanges[0].bound(ExchangeIon::Sodium).0,
        0.1
    );
    assert_eq!(chain.cells()[1].exchanges.len(), 0);
    assert_eq!(step.effluent.moles_of(&SpeciesId::new("CaCO3")).0, 0.0);
}

#[test]
fn invalid_geometry_and_courant_numbers_are_rejected_before_mutation() {
    assert!(matches!(
        CellChain::new(Vec::new()),
        Err(TransportError::EmptyChain)
    ));

    let mut smaller = cell(1, 0.0, Kelvin::STANDARD.0);
    smaller.withdraw(&SpeciesId::new("water"), Moles(1.0));
    assert!(matches!(
        CellChain::new(vec![cell(0, 0.0, Kelvin::STANDARD.0), smaller]),
        Err(TransportError::NonUniformCellVolume { cell: 1, .. })
    ));

    let mut solver_resolution = cell(1, 0.0, Kelvin::STANDARD.0);
    solver_resolution.withdraw(&SpeciesId::new("water"), Moles(WATER_MOLES * 5e-7));
    assert!(
        CellChain::new(vec![cell(0, 0.0, Kelvin::STANDARD.0), solver_resolution,]).is_ok(),
        "sub-ppm aqueous readback must not change hydraulic geometry"
    );

    let inlet = cell(99, 0.0, Kelvin::STANDARD.0);
    let mut chain = CellChain::new(vec![cell(0, 0.0, Kelvin::STANDARD.0)]).unwrap();
    let before = chain.cells()[0].moles_of(&SpeciesId::new("water"));
    assert!(matches!(
        chain.advance(&inlet, 1.01),
        Err(TransportError::InvalidCourant { .. })
    ));
    assert_eq!(chain.cells()[0].moles_of(&SpeciesId::new("water")), before);

    let mut thermostatted = cell(0, 0.0, Kelvin::STANDARD.0);
    thermostatted.thermal_mode = ThermalMode::Thermostatted(Kelvin::STANDARD);
    assert!(matches!(
        CellChain::new(vec![thermostatted]),
        Err(TransportError::ThermostattedCell { cell: 0 })
    ));
}

#[test]
fn surface_released_water_does_not_change_hydraulic_cell_geometry() {
    let reference = cell(0, 0.0, Kelvin::STANDARD.0);
    let mut with_release = cell(1, 0.0, Kelvin::STANDARD.0);
    with_release.deposit(SpeciesId::new("water"), Moles(1e-5), Phase::Liquid);
    with_release.surfaces.push(SurfaceSites {
        label: "hydrated oxide".to_string(),
        model: SurfaceModel::HydrousFerricOxide,
        mass: Grams(0.09),
        specific_area_m2_per_g: 600.0,
        strong_capacity: Moles(5e-6),
        weak_capacity: Moles(2e-4),
        occupancy: vec![SurfaceOccupancy {
            site: SurfaceSiteKind::Weak,
            sorbate: SurfaceSorbate::Sulfate,
            moles: Moles(1e-5),
        }],
        water_release: Moles(1e-5),
    });

    assert!(CellChain::new(vec![reference, with_release]).is_ok());
}

#[test]
fn a_failed_reactive_cell_restores_the_complete_pre_step_chain() {
    let inlet = cell(99, 0.0, Kelvin::STANDARD.0);
    let mut chain = CellChain::new(vec![
        cell(0, 1.0, Kelvin::STANDARD.0),
        cell(1, 0.0, Kelvin::STANDARD.0),
        cell(2, 0.0, Kelvin::STANDARD.0),
    ])
    .unwrap();
    let before = serde_json::to_value(chain.cells()).unwrap();
    let mut solver = FailingSecondCell { calls: 0 };

    let error = chain
        .advance_reactive(&inlet, 0.5, &mut solver)
        .unwrap_err();

    assert!(matches!(
        error,
        ReactiveTransportError::Reaction { cell: 1, .. }
    ));
    assert_eq!(solver.calls, 2, "the third cell must never be solved");
    assert_eq!(serde_json::to_value(chain.cells()).unwrap(), before);
}
