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
        solvent_activity: None,
        scope: Default::default(),
        solvent_kg: None,
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
    // Moles and charge close to 1e-12 because they are added and subtracted.
    // Energy does not, because it is INTEGRATED: every cell's sensible energy
    // is a difference of liquid water's antiderivative, and that particular
    // fit is badly conditioned for differencing. Its terms at 300 K are of
    // order 1.2e9 J/mol and cancel to -9.2e8, so a double gives up about
    // 2.6e-7 J per mole of water per difference taken, and a step over four
    // cells plus an inlet and an effluent takes several. Ice's fit gives up
    // 4e-11 and nitrogen's 4e-12 - this is one curve's conditioning, not the
    // ledger's arithmetic, and no rearrangement of THIS test can improve it.
    //
    // Measured: 9.9e-7 J against the ~250 J the chain holds, which is 3e-9 K
    // spread over the cells. A hundredth of a millijoule is an order above
    // that, leaves room for a libm whose `ln` rounds the other way, and is
    // still far below anything the chain is asked to show.
    let energy_residue = chain_before_energy + step.injected.sensible_energy().0
        - chain.total_sensible_energy().0
        - step.effluent.sensible_energy().0;
    assert!(
        energy_residue.abs() < 1e-5,
        "sensible energy in minus sensible energy out must close: \
         {energy_residue:e} J over {chain_before_energy} J held before the step",
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
fn positive_mobile_and_stationary_traces_close_at_every_courant_boundary() {
    let salt = SpeciesId::new("NaCl");
    let iron = SpeciesId::new("Fe");
    let salt_moles = 8e-16;
    let iron_moles = 5e-16;

    for courant in [0.0, 0.5, 1.0] {
        let mut initial = cell(0, 0.0, Kelvin::STANDARD.0);
        initial.deposit(salt.clone(), Moles(salt_moles), Phase::Aqueous);
        initial.deposit(iron.clone(), Moles(iron_moles), Phase::Solid);
        let inlet = cell(99, 0.0, Kelvin::STANDARD.0);
        let mut chain = CellChain::new(vec![initial]).unwrap();

        let step = chain.advance(&inlet, courant).unwrap();
        let retained = chain.cells()[0].moles_of(&salt).0;
        let discharged = step.effluent.moles_of(&salt).0;
        assert_eq!(retained, salt_moles * (1.0 - courant), "Courant {courant}");
        assert_eq!(discharged, salt_moles * courant, "Courant {courant}");
        assert_eq!(step.injected.moles_of(&salt).0, 0.0);
        // Exact assertions deliberately avoid an absolute tolerance larger
        // than the entire inventory under test.
        assert_eq!(retained + discharged, salt_moles, "Courant {courant}");
        assert_eq!(chain.cells()[0].moles_of(&iron).0, iron_moles);
        assert_eq!(step.effluent.moles_of(&iron).0, 0.0);
        assert_eq!(step.injected.moles_of(&iron).0, 0.0);
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
fn cell_chain_refuses_duplicate_physical_cell_identities() {
    assert!(matches!(
        CellChain::new(vec![
            cell(0, 1.0, Kelvin::STANDARD.0),
            cell(0, 1.0, Kelvin::STANDARD.0)
        ]),
        Err(TransportError::DuplicateCell {
            vessel: VesselId(0)
        })
    ));
    let mut chain = CellChain::new(vec![
        cell(0, 1.0, Kelvin::STANDARD.0),
        cell(1, 0.0, Kelvin::STANDARD.0),
    ])
    .unwrap();
    chain.cells_mut()[1].id = VesselId(0);
    let before = serde_json::to_value(chain.cells()).unwrap();
    assert!(matches!(
        chain.advance(&cell(99, 0.0, Kelvin::STANDARD.0), 1.0),
        Err(TransportError::DuplicateCell {
            vessel: VesselId(0)
        })
    ));
    assert_eq!(serde_json::to_value(chain.cells()).unwrap(), before);
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
fn reactive_surface_reference_water_has_a_capacity_bounded_geometry_allowance() {
    fn surface() -> SurfaceSites {
        SurfaceSites {
            label: "hydrated oxide".to_string(),
            model: SurfaceModel::HydrousFerricOxide,
            mass: Grams(0.09),
            specific_area_m2_per_g: 600.0,
            strong_capacity: Moles(5e-6),
            weak_capacity: Moles(2e-4),
            occupancy: Vec::new(),
            water_release: Moles(0.0),
        }
    }

    let inlet = cell(99, 0.0, Kelvin::STANDARD.0);
    let mut first = cell(0, 0.0, Kelvin::STANDARD.0);
    first.surfaces.push(surface());
    let mut second = cell(1, 0.0, Kelvin::STANDARD.0);
    second.surfaces.push(surface());

    let mut within_capacity = CellChain::new(vec![first.clone(), second.clone()]).unwrap();
    within_capacity.cells_mut()[1].deposit(SpeciesId::new("water"), Moles(2e-4), Phase::Liquid);
    assert!(
        within_capacity.advance(&inlet, 0.0).is_ok(),
        "surface reference-state water may shift by no more than site capacity"
    );

    let mut beyond_capacity = CellChain::new(vec![first, second]).unwrap();
    beyond_capacity.cells_mut()[1].deposit(SpeciesId::new("water"), Moles(5e-4), Phase::Liquid);
    assert!(matches!(
        beyond_capacity.advance(&inlet, 0.0),
        Err(TransportError::NonUniformCellVolume { cell: 1, .. })
    ));
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

#[test]
fn invalid_stationary_inventory_is_refused_before_mobile_matter_moves() {
    for (species, phase) in [("Fe", Phase::Solid), ("N2", Phase::Gas)] {
        for amount in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.125] {
            let mut initial = cell(0, 0.125, Kelvin::STANDARD.0);
            initial.contents.push(Portion {
                species: SpeciesId::new(species),
                moles: Moles(amount),
                phase,
            });
            let mut chain = CellChain::new(vec![initial]).unwrap();
            let before = serde_json::to_value(chain.cells()).unwrap();
            let result = chain.advance(&cell(99, 0.0, Kelvin::STANDARD.0), 0.5);
            assert!(
                matches!(result, Err(TransportError::InvalidMobileState { .. })),
                "invalid stationary {species} amount {amount} must refuse transport"
            );
            assert_eq!(
                serde_json::to_value(chain.cells()).unwrap(),
                before,
                "refusal must preserve the original stock and all mobile state"
            );
        }
    }
}

#[test]
fn finite_stationary_inventory_including_zero_does_not_block_transport() {
    for (species, phase) in [("Fe", Phase::Solid), ("N2", Phase::Gas)] {
        for amount in [0.0, 5e-16, 0.125] {
            let mut initial = cell(0, 0.125, Kelvin::STANDARD.0);
            initial.contents.push(Portion {
                species: SpeciesId::new(species),
                moles: Moles(amount),
                phase,
            });
            let mut chain = CellChain::new(vec![initial]).unwrap();
            let step = chain
                .advance(&cell(99, 0.0, Kelvin::STANDARD.0), 0.5)
                .unwrap();
            assert_eq!(
                chain.cells()[0].moles_of(&SpeciesId::new(species)).0,
                amount
            );
            assert_eq!(step.effluent.moles_of(&SpeciesId::new(species)).0, 0.0);
            assert_eq!(
                chain.total_moles(&SpeciesId::new("passive-tracer")).0,
                0.0625
            );
            assert_eq!(
                step.effluent.moles_of(&SpeciesId::new("passive-tracer")).0,
                0.0625
            );
        }
    }
}

#[test]
fn finite_inventory_with_overflowing_mix_energy_refuses_atomically() {
    let mut initial = cell(0, 0.0, 350.0);
    initial.deposit(SpeciesId::new("NaCl"), Moles(1e306), Phase::Aqueous);
    assert!(
        initial.mass().0.is_finite(),
        "the mass itself is representable"
    );
    assert!(
        initial.heat_capacity().is_finite(),
        "the local heat capacity is representable"
    );
    let mut chain = CellChain::new(vec![initial]).unwrap();
    let before = serde_json::to_value(chain.cells()).unwrap();
    let result = chain.advance(&cell(99, 0.0, Kelvin::STANDARD.0), 0.5);
    assert!(
        matches!(result, Err(TransportError::InvalidMobileState { .. })),
        "a nonrepresentable mixing-energy endpoint must refuse rather than choose a temperature"
    );
    assert_eq!(serde_json::to_value(chain.cells()).unwrap(), before);
}

#[test]
fn moderate_aqueous_inventory_and_temperature_difference_conserve_energy() {
    let mut initial = cell(0, 0.0, 350.0);
    let salt = SpeciesId::new("NaCl");
    initial.deposit(salt.clone(), Moles(0.25), Phase::Aqueous);
    let reference = Kelvin::STANDARD.0;
    let initial_energy = initial.energy_between(reference, initial.temperature.0);
    let mut chain = CellChain::new(vec![initial]).unwrap();
    let inlet = cell(99, 0.0, reference);
    let step = chain.advance(&inlet, 0.5).unwrap();
    assert_eq!(chain.total_moles(&salt).0, 0.125);
    assert_eq!(step.effluent.moles_of(&salt).0, 0.125);
    assert!(chain.cells()[0].temperature.0 > reference && chain.cells()[0].temperature.0 < 350.0);
    let residue = initial_energy + step.injected.sensible_energy().0
        - chain.total_sensible_energy().0
        - step.effluent.sensible_energy().0;
    assert!(
        residue.abs() < 1e-5,
        "independent energy ledger residual {residue} J"
    );
}

#[test]
fn later_cell_energy_refusal_rolls_back_transport_before_reaction_starts() {
    let first = cell(0, 0.125, Kelvin::STANDARD.0);
    let mut second = cell(1, 0.0, 350.0);
    second.deposit(SpeciesId::new("NaCl"), Moles(1e306), Phase::Aqueous);
    let mut chain = CellChain::new(vec![first, second]).unwrap();
    let before = serde_json::to_value(chain.cells()).unwrap();
    let mut solver = FailingSecondCell { calls: 0 };
    let result = chain.advance_reactive(&cell(99, 0.0, 310.0), 0.5, &mut solver);
    assert!(matches!(
        result,
        Err(ReactiveTransportError::Transport(
            TransportError::InvalidMobileState { .. }
        ))
    ));
    assert_eq!(serde_json::to_value(chain.cells()).unwrap(), before);
    assert_eq!(
        solver.calls, 0,
        "failed transport must not start local reactions"
    );
}

#[test]
fn inlet_identity_is_a_boundary_template_not_an_additional_owned_cell() {
    let initial = cell(0, 0.125, Kelvin::STANDARD.0);
    let inlet = initial.clone();
    let inlet_before = serde_json::to_value(&inlet).unwrap();
    let mut chain = CellChain::new(vec![initial]).unwrap();
    let step = chain.advance(&inlet, 0.5).unwrap();
    let tracer = SpeciesId::new("passive-tracer");
    assert_eq!(chain.total_moles(&tracer).0, 0.125);
    assert_eq!(step.injected.moles_of(&tracer).0, 0.0625);
    assert_eq!(step.effluent.moles_of(&tracer).0, 0.0625);
    assert_eq!(serde_json::to_value(&inlet).unwrap(), inlet_before);
}
