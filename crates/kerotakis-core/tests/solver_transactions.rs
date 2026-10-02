//! Independent transaction contracts: complete state, boundary flows and failures.
use kerotakis_core::{
    delta::{DeltaError, StateDelta, ThermalDelta},
    orchestrator::{diff_vessels, Orchestrator},
    solve::{Equilibrator, SolveError, SolverStack},
    vessel::Headspace,
    *,
};

fn water() -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("water"), Moles(5.0), Phase::Liquid);
    v
}
fn key(v: &Vessel) -> String {
    format!("{v:?}")
}
fn mark(v: &mut Vessel) {
    v.free_proton = 0.001;
    v.co2_partial_pressure_atm = Some(0.02);
    v.pending_co2_transfer_mol = 0.0001;
    v.excess_enthalpy_j = 17.0;
    v.aqueous_routing_said = Some("trial route".into());
    v.honesty_said = vec!["trial diagnosis".into()];
    v.solution = Some(
        serde_json::from_value(serde_json::json!({"ph": 3.0, "ionic_strength": 0.01})).unwrap(),
    );
    v.resolved.valid = true;
    v.resolved.solution = v.solution.clone();
}

#[test]
fn cloned_proposal_commits_all_solver_state_even_without_bulk_changes() {
    let before = water();
    let mut after = before.clone();
    mark(&mut after);
    after.headspace = Headspace::Sealed {
        volume: Liters(0.1),
    };
    after.pressure = Pascal(150_000.0);
    let proposal = diff_vessels(&before, &after, "test");
    assert!(!proposal.is_empty());
    let mut actual = before;
    proposal.commit_conserved(&mut actual, 1e-10).unwrap();
    assert_eq!(key(&actual), key(&after));
}

#[test]
fn stale_snapshot_cannot_erase_an_intervening_operation_or_narration() {
    for narration_only in [false, true] {
        let mut v = water();
        let mut after = v.clone();
        mark(&mut after);
        let proposal = diff_vessels(&v, &after, "test");
        if narration_only {
            v.honesty_said.push("intervening".into());
        } else {
            v.deposit(SpeciesId::new("NaCl"), Moles(0.1), Phase::Solid);
        }
        let before = key(&v);
        assert!(
            matches!(proposal.commit(&mut v), Err(errors) if errors.contains(&DeltaError::StaleProposal))
        );
        assert_eq!(key(&v), before);
    }
}

#[test]
fn rejected_snapshot_restores_derived_state_and_narration_too() {
    let mut v = water();
    let before = key(&v);
    let mut after = v.clone();
    mark(&mut after);
    after.deposit(SpeciesId::new("NaCl"), Moles(0.1), Phase::Solid);
    assert!(diff_vessels(&v, &after, "test")
        .commit_conserved(&mut v, 1e-10)
        .is_err());
    assert_eq!(key(&v), before);
}

#[test]
fn snapshot_scaling_is_explicitly_unsupported() {
    let v = water();
    let mut after = v.clone();
    mark(&mut after);
    assert!(
        matches!(diff_vessels(&v, &after, "test").inventory_limited(&v), Err(errors) if errors.contains(&DeltaError::UnscalableSnapshot))
    );
}

#[test]
fn invalid_thermal_and_snapshot_inventories_never_commit() {
    for invalid in [f64::NAN, f64::INFINITY, -1.0] {
        let mut v = water();
        let before = key(&v);
        assert!(StateDelta::new("test")
            .with_thermal(ThermalDelta::SetTemperature(Kelvin(invalid)))
            .commit(&mut v)
            .is_err());
        assert_eq!(key(&v), before);
        let mut after = v.clone();
        after.temperature = Kelvin(invalid);
        assert!(diff_vessels(&v, &after, "test").commit(&mut v).is_err());
        assert_eq!(key(&v), before);
    }
    let mut v = water();
    let before = key(&v);
    let mut after = v.clone();
    after.contents[0].moles = Moles(f64::NAN);
    assert!(diff_vessels(&v, &after, "test").commit(&mut v).is_err());
    assert_eq!(key(&v), before);
}

#[test]
fn native_bulk_prefix_and_overflow_are_rejected_before_clamping() {
    let mut v = water();
    let before = key(&v);
    let delta = StateDelta::new("test")
        .with_moles(SpeciesId::new("water"), Phase::Liquid, -6.0)
        .with_moles(SpeciesId::new("water"), Phase::Liquid, 6.0);
    assert!(delta.commit(&mut v).is_err());
    assert_eq!(key(&v), before);
    let delta = StateDelta::new("test")
        .with_moles(SpeciesId::new("water"), Phase::Liquid, f64::MAX)
        .with_moles(SpeciesId::new("water"), Phase::Liquid, f64::MAX);
    assert!(delta.commit(&mut v).is_err());
    assert_eq!(key(&v), before);
}

#[derive(Clone, Copy)]
enum Trial {
    Fail,
    Succeed,
    MixFail,
    MixDecline,
    InvalidSuccess,
    InvalidMixSuccess,
}
impl Equilibrator for Trial {
    fn name(&self) -> &'static str {
        "injected-trial"
    }
    fn chemistry_applies(&self, _: &Vessel) -> bool {
        true
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        mark(v);
        if matches!(self, Self::InvalidSuccess) {
            v.solution.as_mut().unwrap().ph = f64::NAN;
            return Ok(vec![Event::GasEvolved {
                vessel: v.id,
                species: SpeciesId::new("water"),
                moles: Moles(1.0),
            }]);
        }
        if matches!(self, Self::Fail) {
            v.contents.clear();
            Err(SolveError::NotConverged {
                solver: self.name().into(),
                detail: "injected after mutation".into(),
            })
        } else {
            Ok(vec![])
        }
    }
    fn mix(
        &mut self,
        v: &mut Vessel,
        _: &Vessel,
        _: f64,
        _: &Vessel,
        _: f64,
    ) -> Option<Result<Vec<Event>, SolveError>> {
        mark(v);
        v.contents.clear();
        match self {
            Self::InvalidMixSuccess => {
                v.temperature = Kelvin(f64::NAN);
                Some(Ok(vec![]))
            }
            Self::MixFail => Some(Err(SolveError::NotConverged {
                solver: self.name().into(),
                detail: "injected MIX failure".into(),
            })),
            _ => None,
        }
    }
}

#[test]
fn production_stack_failure_restores_state_before_next_solver() {
    let mut v = water();
    let before = key(&v);
    let mut stack = SolverStack::new(vec![Box::new(Trial::Fail)]);
    let events = stack.equilibrate(&mut v).unwrap();
    assert_eq!(key(&v), before);
    assert!(matches!(&events[..], [Event::SolverFailed { .. }]));
    assert!(matches!(
        stack.last_routes[0].outcome,
        kerotakis_core::solve::SolverRouteOutcome::Failed
    ));
    let mut stack = SolverStack::new(vec![Box::new(Trial::Fail), Box::new(Trial::Succeed)]);
    stack.equilibrate(&mut v).unwrap();
    let mut expected = water();
    mark(&mut expected);
    assert_eq!(key(&v), key(&expected));
}

#[test]
fn production_mix_failure_and_decline_restore_all_trial_state() {
    let a = water();
    let b = water();
    for trial in [Trial::MixFail, Trial::MixDecline] {
        let mut v = water();
        let before = key(&v);
        let mut stack = SolverStack::new(vec![Box::new(trial)]);
        let result = stack.mix(&mut v, &a, 0.5, &b, 0.5);
        assert_eq!(key(&v), before);
        match trial {
            Trial::MixFail => assert!(result.unwrap().is_err()),
            _ => assert!(result.is_none()),
        }
    }
}

struct Outlet {
    balanced: bool,
}
impl Equilibrator for Outlet {
    fn name(&self) -> &'static str {
        "outlet"
    }
    fn chemistry_applies(&self, _: &Vessel) -> bool {
        true
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        mark(v);
        v.withdraw_phase(&SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
        Ok(vec![Event::GasEvolved {
            vessel: v.id,
            species: SpeciesId::new("water"),
            moles: Moles(if self.balanced { 1.0 } else { 0.5 }),
        }])
    }
}

#[test]
fn orchestration_commits_boundary_events_and_complete_state_together() {
    for balanced in [false, true] {
        let mut v = water();
        let before = key(&v);
        let mut orchestrator = Orchestrator::new(vec![Box::new(Outlet { balanced })]);
        let events = orchestrator.equilibrate(&mut v).unwrap();
        if balanced {
            assert!(matches!(&events[..], [Event::GasEvolved { .. }]));
            assert_eq!(v.free_proton, 0.001);
            assert_eq!(v.moles_of(&SpeciesId::new("water")), Moles(4.0));
        } else {
            assert!(matches!(&events[..], [Event::SolverFailed { .. }]));
            assert_eq!(key(&v), before);
        }
    }
}

#[test]
fn boundary_event_must_name_correct_vessel_and_finite_amount() {
    for (id, amount) in [
        (VesselId(9), 0.0),
        (VesselId(0), f64::NAN),
        (VesselId(0), -1.0),
    ] {
        let mut v = water();
        let before = key(&v);
        let event = Event::GasAbsorbed {
            vessel: id,
            species: SpeciesId::new("water"),
            moles: Moles(amount),
        };
        assert!(StateDelta::new("test")
            .commit_conserved_with_events(&mut v, 1e-10, &[event])
            .is_err());
        assert_eq!(key(&v), before);
    }
}

#[test]
fn bounded_equilibrium_sequences_preserve_complete_results_and_reload() {
    for repetitions in [1, 2, 8, 32] {
        let mut direct = water();
        let mut transactional = direct.clone();
        for step in 0..repetitions {
            let mut after = direct.clone();
            mark(&mut after);
            after.elapsed_seconds = step as f64;
            after.temperature = Kelvin(290.0 + step as f64);
            let proposal = diff_vessels(&transactional, &after, "sequence");
            proposal
                .commit_conserved(&mut transactional, 1e-10)
                .unwrap();
            direct = after;
            assert_eq!(key(&transactional), key(&direct));
            let reload: Vessel =
                serde_json::from_value(serde_json::to_value(&transactional).unwrap()).unwrap();
            assert_eq!(
                serde_json::to_value(&reload).unwrap(),
                serde_json::to_value(&direct).unwrap()
            );
        }
    }
}

#[test]
fn complete_snapshot_cannot_be_edited_into_a_different_native_delta() {
    let mut v = water();
    let before = key(&v);
    let mut after = v.clone();
    mark(&mut after);
    let proposal =
        diff_vessels(&v, &after, "test").with_moles(SpeciesId::new("NaCl"), Phase::Solid, 0.1);
    assert!(proposal.commit(&mut v).is_err());
    assert_eq!(key(&v), before);
}

#[test]
fn finite_surface_inventory_and_speciation_commit_as_one_result() {
    use kerotakis_core::vessel::{
        SurfaceModel, SurfaceOccupancy, SurfaceSiteKind, SurfaceSites, SurfaceSorbate,
    };
    let mut v = water();
    v.deposit(SpeciesId::new("Zn+2"), Moles(0.01), Phase::Aqueous);
    v.surfaces.push(SurfaceSites {
        label: "oxide".into(),
        model: SurfaceModel::HydrousFerricOxide,
        mass: Grams(1.0),
        specific_area_m2_per_g: 600.0,
        strong_capacity: Moles(0.1),
        weak_capacity: Moles(0.1),
        occupancy: vec![],
        water_release: Moles(0.0),
    });
    let mut after = v.clone();
    mark(&mut after);
    after.withdraw_phase(&SpeciesId::new("Zn+2"), Moles(0.004), Phase::Aqueous);
    after.surfaces[0].occupancy.push(SurfaceOccupancy {
        site: SurfaceSiteKind::Strong,
        sorbate: SurfaceSorbate::Zinc,
        moles: Moles(0.004),
    });
    diff_vessels(&v, &after, "surface")
        .commit_conserved(&mut v, 1e-10)
        .unwrap();
    assert_eq!(key(&v), key(&after));
    let before = key(&v);
    let mut invalid = v.clone();
    invalid.surfaces[0].occupancy[0].moles = Moles(f64::NAN);
    assert!(diff_vessels(&v, &invalid, "surface")
        .commit(&mut v)
        .is_err());
    assert_eq!(key(&v), before);
}

#[test]
fn absorbed_gas_is_accounted_and_unbacked_outlet_event_is_rejected() {
    let mut v = water();
    let delta =
        StateDelta::new("condensation").with_moles(SpeciesId::new("water"), Phase::Liquid, 0.25);
    let event = Event::GasAbsorbed {
        vessel: v.id,
        species: SpeciesId::new("water"),
        moles: Moles(0.25),
    };
    delta
        .commit_conserved_with_events(&mut v, 1e-10, &[event])
        .unwrap();
    assert_eq!(v.moles_of(&SpeciesId::new("water")), Moles(5.25));
    let before = key(&v);
    let event = Event::GasEvolved {
        vessel: v.id,
        species: SpeciesId::new("water"),
        moles: Moles(0.25),
    };
    assert!(StateDelta::new("invented outlet")
        .commit_conserved_with_events(&mut v, 1e-10, &[event])
        .is_err());
    assert_eq!(key(&v), before);
}

#[test]
fn numerically_invalid_success_discards_trial_events_and_metadata() {
    let mut v = water();
    let before = key(&v);
    let mut stack = SolverStack::new(vec![Box::new(Trial::InvalidSuccess)]);
    let events = stack.equilibrate(&mut v).unwrap();
    assert_eq!(key(&v), before);
    assert!(matches!(&events[..], [Event::SolverFailed { .. }]));
    let a = water();
    let b = water();
    let mut stack = SolverStack::new(vec![Box::new(Trial::InvalidMixSuccess)]);
    assert!(stack.mix(&mut v, &a, 0.5, &b, 0.5).unwrap().is_err());
    assert_eq!(key(&v), before);
}

#[test]
fn individually_finite_initial_inventory_and_delta_cannot_overflow_on_commit() {
    for conserved in [false, true] {
        let mut v = water();
        v.contents[0].moles = Moles(f64::MAX);
        let before = key(&v);
        let proposal = StateDelta::new("overflow").with_moles(
            SpeciesId::new("water"),
            Phase::Liquid,
            f64::MAX,
        );
        let result = if conserved {
            proposal.commit_conserved(&mut v, 1e-10)
        } else {
            proposal.commit(&mut v)
        };
        assert!(result.is_err());
        assert_eq!(key(&v), before);
    }
}

#[test]
fn invalid_boundary_and_dependent_numerical_state_never_commits() {
    let mutations: [fn(&mut Vessel); 11] = [
        |v| {
            v.headspace = Headspace::Sealed {
                volume: Liters(f64::NAN),
            }
        },
        |v| {
            v.headspace = Headspace::PressureControlled {
                pressure: Pascal(f64::INFINITY),
                volume: Liters(0.1),
            }
        },
        |v| {
            v.headspace = Headspace::Swept {
                pressure: Pascal(-1.0),
            }
        },
        |v| v.thermal_mode = ThermalMode::Thermostatted(Kelvin(f64::NAN)),
        |v| v.co2_partial_pressure_atm = Some(f64::NAN),
        |v| v.pending_co2_transfer_mol = f64::INFINITY,
        |v| v.solute_charge = f64::NAN,
        |v| v.excess_enthalpy_j = f64::NAN,
        |v| v.solution.as_mut().unwrap().pe = Some(f64::NAN),
        |v| {
            v.solution.as_mut().unwrap().redox.push(RedoxState {
                element: "Fe".into(),
                oxidation: 2,
                molality: -1.0,
            })
        },
        |v| v.resolved.solution.as_mut().unwrap().ionic_strength = f64::NAN,
    ];
    struct Invalid(fn(&mut Vessel));
    impl Equilibrator for Invalid {
        fn name(&self) -> &'static str {
            "invalid-dependent-state"
        }
        fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
            mark(v);
            (self.0)(v);
            Ok(vec![])
        }
    }
    for mutation in mutations {
        let mut v = water();
        let before = key(&v);
        let mut proposed = v.clone();
        mark(&mut proposed);
        mutation(&mut proposed);
        assert!(diff_vessels(&v, &proposed, "bad-state")
            .commit(&mut v)
            .is_err());
        assert_eq!(key(&v), before);
        let events = SolverStack::new(vec![Box::new(Invalid(mutation))])
            .equilibrate(&mut v)
            .unwrap();
        assert!(matches!(&events[..], [Event::SolverFailed { .. }]));
        assert_eq!(key(&v), before);
    }
}

struct BalancedRoute {
    flow: f64,
    remove: bool,
    target: VesselId,
}
impl Equilibrator for BalancedRoute {
    fn name(&self) -> &'static str {
        "balanced-test"
    }
    fn element_conservation_tolerance(&self) -> Option<f64> {
        Some(1e-10)
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        mark(v);
        if self.remove {
            v.withdraw(&SpeciesId::new("water"), Moles(0.25));
        }
        Ok(vec![Event::GasEvolved {
            vessel: self.target,
            species: SpeciesId::new("water"),
            moles: Moles(self.flow),
        }])
    }
    fn mix(
        &mut self,
        v: &mut Vessel,
        _a: &Vessel,
        _fa: f64,
        _b: &Vessel,
        _fb: f64,
    ) -> Option<Result<Vec<Event>, SolveError>> {
        Some(self.equilibrate(v))
    }
}

#[test]
fn production_stack_enforces_opted_in_balances_and_event_domains_atomically() {
    for (flow, remove, target, succeeds) in [
        (0.25, true, VesselId(0), true),
        (0.25, false, VesselId(0), false),
        (0.0, true, VesselId(0), false),
        (f64::NAN, true, VesselId(0), false),
        (f64::INFINITY, true, VesselId(0), false),
        (-0.25, true, VesselId(0), false),
        (0.25, true, VesselId(1), false),
    ] {
        let mut v = water();
        let before = key(&v);
        let mut stack = SolverStack::new(vec![Box::new(BalancedRoute {
            flow,
            remove,
            target,
        })]);
        let events = stack.equilibrate(&mut v).unwrap();
        assert_eq!(
            events
                .iter()
                .any(|e| matches!(e, Event::SolverFailed { .. })),
            !succeeds
        );
        if !succeeds {
            assert_eq!(key(&v), before);
        } else {
            assert_eq!(v.moles_of(&SpeciesId::new("water")).0, 4.75);
        }
    }
}

#[test]
fn finite_species_amounts_with_overflowing_aggregate_mass_are_rejected() {
    let mut v = water();
    let before = key(&v);
    let proposal = StateDelta::new("mass overflow").with_moles(
        SpeciesId::new("water"),
        Phase::Liquid,
        f64::MAX / 2.0,
    );
    assert!(proposal.commit(&mut v).is_err());
    assert_eq!(key(&v), before);
}

#[test]
fn operator_numeric_refusal_preserves_all_bench_state_and_history() {
    for amount in [f64::NAN, f64::INFINITY, f64::MAX / 2.0] {
        let mut bench = Bench::new();
        let before = format!("{bench:?}");
        let result = bench.step(Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new("water"),
            moles: Moles(amount),
            at: None,
        });
        assert!(result.is_err(), "{amount}: {result:?}");
        assert_eq!(format!("{bench:?}"), before);
    }
}

#[test]
fn accumulated_waste_overflow_rolls_back_transfer_and_journal() {
    let mut bench = Bench::new();
    // Each charge has finite mass and heat capacity; their combined waste mass overflows.
    bench.vessels[0].deposit(SpeciesId::new("BaSO4"), Moles(5e305), Phase::Solid);
    assert!(bench.vessels[0].mass().0 > 1e308);
    assert!(bench.vessels[0].mass().0.is_finite());
    assert!(bench.vessels[0].heat_capacity().is_finite());
    bench
        .step(Operator::Discard {
            vessel: VesselId(0),
        })
        .unwrap();
    // Each charge has finite mass and heat capacity; their combined waste mass overflows.
    bench.vessels[0].deposit(SpeciesId::new("BaSO4"), Moles(5e305), Phase::Solid);
    assert!(bench.vessels[0].mass().0 > 1e308);
    assert!(bench.vessels[0].mass().0.is_finite());
    assert!(bench.vessels[0].heat_capacity().is_finite());
    let before = format!("{bench:?}");
    assert!(bench
        .step(Operator::Discard {
            vessel: VesselId(0)
        })
        .is_err());
    assert_eq!(format!("{bench:?}"), before);
}

#[test]
fn production_native_mix_enforces_balances_before_committing() {
    for (flow, remove, target, succeeds) in [
        (0.25, true, VesselId(0), true),
        (0.25, false, VesselId(0), false),
        (0.0, true, VesselId(0), false),
        (f64::NAN, true, VesselId(0), false),
        (0.25, true, VesselId(1), false),
    ] {
        let mut v = water();
        let before = key(&v);
        let mut stack = SolverStack::new(vec![Box::new(BalancedRoute {
            flow,
            remove,
            target,
        })]);
        let result = stack.mix(&mut v, &water(), 0.5, &water(), 0.5).unwrap();
        assert_eq!(result.is_ok(), succeeds);
        if !succeeds {
            assert_eq!(key(&v), before);
        }
    }
}

#[test]
fn heat_input_prices_only_the_current_delivery_pass_and_is_not_persisted() {
    use std::{cell::RefCell, rc::Rc};
    let seen = Rc::new(RefCell::new(Vec::new()));
    struct Observer(Rc<RefCell<Vec<Option<kerotakis_core::vessel::HeatInput>>>>);
    impl Equilibrator for Observer {
        fn name(&self) -> &'static str {
            "heat-input-observer"
        }
        fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
            self.0.borrow_mut().push(v.heat_input.clone());
            Ok(vec![])
        }
    }
    let mut bench = Bench::new();
    bench.vessels[0].deposit(SpeciesId::new("Mg"), Moles(0.05), Phase::Solid);
    let original = bench.vessels[0].clone();
    let mut stack = SolverStack::new(vec![Box::new(Observer(seen.clone()))]);
    bench
        .step_with(
            Operator::Heat {
                vessel: VesselId(0),
                energy: Joules(1000.0),
                source: None,
            },
            &mut stack,
            &PermissiveScreen,
        )
        .unwrap();
    let recorded = seen.borrow()[0].clone().unwrap();
    assert_eq!(recorded.contents, original.contents);
    assert_eq!(recorded.temperature, original.temperature);
    assert_eq!(recorded.delivered_j, 1000.0);
    assert!(bench.vessels[0].heat_input.is_none());
    let mut transient = original;
    transient.heat_input = Some(recorded);
    assert!(serde_json::to_value(&transient)
        .unwrap()
        .get("heat_input")
        .is_none());
    let restored: Vessel =
        serde_json::from_value(serde_json::to_value(&transient).unwrap()).unwrap();
    assert!(restored.heat_input.is_none());
    seen.borrow_mut().clear();
    bench
        .step_with(
            Operator::Wait { seconds: 0.0 },
            &mut stack,
            &PermissiveScreen,
        )
        .unwrap();
    assert!(bench.vessels[0].heat_input.is_none());
    assert!(seen.borrow().iter().all(Option::is_none));
}

#[test]
fn nonfinite_energy_and_heat_input_proposals_never_commit() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut bench = Bench::new();
        bench.vessels[0] = water();
        let before = format!("{bench:?}");
        for op in [
            Operator::Heat {
                vessel: VesselId(0),
                energy: Joules(value),
                source: None,
            },
            Operator::Cool {
                vessel: VesselId(0),
                energy: Joules(value),
            },
        ] {
            assert!(bench.step(op).is_err());
            assert_eq!(format!("{bench:?}"), before);
        }
        let mut v = water();
        let mut after = v.clone();
        after.heat_input = Some(kerotakis_core::vessel::HeatInput {
            contents: v.contents.clone(),
            temperature: Kelvin(298.15),
            delivered_j: value,
        });
        let before = key(&v);
        assert!(diff_vessels(&v, &after, "heat-input")
            .commit(&mut v)
            .is_err());
        assert_eq!(key(&v), before);
    }
}

#[test]
fn disclosed_cooling_floor_can_be_reheated_without_invalid_native_context() {
    let mut bench = Bench::new();
    bench.vessels[0] = water();
    let mut stack = SolverStack::new(vec![]);
    let events = bench
        .step_with(
            Operator::Cool {
                vessel: VesselId(0),
                energy: Joules(1e6),
            },
            &mut stack,
            &PermissiveScreen,
        )
        .unwrap();
    assert_eq!(bench.vessels[0].temperature.0, 0.0);
    assert!(events.iter().any(
        |e| matches!(e, Event::NotYetModeled { what, .. } if what.contains("before absolute zero"))
    ));
    assert!(events.iter().any(|e| matches!(e, Event::EnergyTransferred { requested_j, delivered_j, .. } if delivered_j < requested_j)));
    bench
        .step_with(
            Operator::Heat {
                vessel: VesselId(0),
                energy: Joules(1000.0),
                source: None,
            },
            &mut stack,
            &PermissiveScreen,
        )
        .unwrap();
    assert!(bench.vessels[0].temperature.0 > 0.0);
    assert!(bench.vessels[0].heat_input.is_none());
}

#[test]
fn finite_mass_with_overflowed_heat_capacity_is_refused_atomically() {
    let mut bench = Bench::new();
    let before = format!("{bench:?}");
    let mut proposed = water();
    proposed.contents[0].moles = Moles(8e306);
    assert!(proposed.mass().0.is_finite());
    assert!(!proposed.heat_capacity().is_finite());
    assert!(bench
        .step(Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new("water"),
            moles: Moles(8e306),
            at: None
        })
        .is_err());
    assert_eq!(format!("{bench:?}"), before);
    let mut original = water();
    let before = key(&original);
    assert!(diff_vessels(&original, &proposed, "overflowed-capacity")
        .commit(&mut original)
        .is_err());
    assert_eq!(key(&original), before);
}

fn electrode() -> kerotakis_core::compartment::ElectrodeState {
    serde_json::from_value(serde_json::json!({
        "label":"e", "material":"Pt", "area_m2":0.001,
        "deposits":[{"species":"Cu", "moles":0.001,
            "thickness_m":1e-6, "coverage_fraction":0.5,
            "electrical_resistivity_ohm_m":1e-7}]
    }))
    .unwrap()
}

#[test]
fn snapshot_deposit_geometry_and_aggregate_electrical_overflow_are_atomic() {
    for case in 0..17 {
        let mut v = water();
        v.electrodes.push(electrode());
        let before = key(&v);
        let mut after = v.clone();
        let e = &mut after.electrodes[0];
        if (6..=8).contains(&case) {
            e.interfacial_potential_v = Some(0.0);
        }
        match case {
            0 => e.deposits[0].coverage_fraction = Some(f64::NAN),
            1 => e.deposits[0].coverage_fraction = Some(1.01),
            2 => e.deposits[0].thickness_m = Some(-1.0),
            3 => e.deposits[0].electrical_resistivity_ohm_m = Some(f64::INFINITY),
            4 => {
                e.deposits[0].thickness_m = Some(1e200);
                e.deposits[0].electrical_resistivity_ohm_m = Some(1e200);
            }
            5 => {
                e.area_m2 = 1e200;
                e.roughness = 1e200;
            }
            6 => e.double_layer_capacitance_f_per_m2 = Some(f64::NAN),
            7 => e.double_layer_capacitance_f_per_m2 = Some(-1.0),
            8 => {
                e.area_m2 = 1e200;
                e.double_layer_capacitance_f_per_m2 = Some(1e200);
            }
            9 => {
                e.deposits[0].thickness_m = Some(1e308);
                e.deposits[0].electrical_resistivity_ohm_m = Some(1.0);
                let mut other = e.deposits[0].clone();
                other.species = "Zn".into();
                e.deposits.push(other);
            }
            13 => e.double_layer_capacitance_f_per_m2 = Some(0.2),
            14 => e.interfacial_potential_v = Some(-0.2),
            _ => {
                let mut diagnostics: kerotakis_core::compartment::ElectrodeDiagnostics =
                    serde_json::from_value(serde_json::json!({
                        "seconds":1.0,"inventory_limited":false,
                        "interfacial_conditions":[],"applied_parameters":[],
                        "balance":{"electrode_potential_v":0.0,"terminal_potential_v":0.0,
                            "net_current_density_a_per_m2":0.0,
                            "partial_currents":[{"reaction_id":"r","current_density_a_per_m2":0.0}]}
                    }))
                    .unwrap();
                match case {
                    10 => {
                        diagnostics.balance.partial_currents[0].current_density_a_per_m2 = f64::NAN
                    }
                    11 => diagnostics.balance.total_current_density_a_per_m2 = f64::INFINITY,
                    12 => diagnostics.seconds = -1.0,
                    15 => {
                        e.area_m2 = 1e200;
                        diagnostics.balance.total_current_density_a_per_m2 = 1e200;
                    }
                    _ => {
                        diagnostics.balance.partial_currents[0].current_density_a_per_m2 = 1e308;
                        let mut other = diagnostics.balance.partial_currents[0].clone();
                        other.reaction_id = "other".into();
                        diagnostics.balance.partial_currents.push(other);
                    }
                }
                e.diagnostics = Some(diagnostics);
            }
        }
        assert!(
            diff_vessels(&v, &after, "test").commit(&mut v).is_err(),
            "case {case}"
        );
        assert_eq!(key(&v), before, "case {case}");
    }
}

#[test]
fn snapshot_material_progress_and_visual_geometry_are_atomic() {
    for case in 0..8 {
        let mut v = water();
        let before = key(&v);
        let mut after = v.clone();
        match case {
            0 => after.foam.volume_liters = f64::NAN,
            1 => after.foam.trapped_gas_liters = -1.0,
            2 => {
                after.surface_particles = Some(kerotakis_core::vessel::SurfaceParticleState {
                    material: "pepper".into(),
                    coverage_fraction: 1.1,
                    cleared_fraction: 0.0,
                })
            }
            3 => {
                after.emulsion = Some(kerotakis_core::vessel::EmulsionState {
                    oil_recipe_id: "oil".into(),
                    dispersed_volume_l: 0.001,
                    half_life_seconds: 0.0,
                })
            }
            4 => {
                after.lemon_paper_mark = Some(kerotakis_core::vessel::LemonPaperMarkState {
                    lemon_amount_g: 1.0,
                    paper_amount_g: 1.0,
                    dry: true,
                    browned_fraction: f64::INFINITY,
                })
            }
            5 => {
                after.soap_scum = Some(kerotakis_core::vessel::SoapScumState {
                    aggregate_mass_g: f64::NAN,
                    divalent_ion_moles: 0.001,
                    soap_equivalent_moles: 0.002,
                })
            }
            _ => after
                .material_objects
                .push(kerotakis_core::vessel::MaterialObject {
                    material: "object".into(),
                    recipe_id: "test".into(),
                    recipe_version: 1,
                    mass_g: 1.0,
                    components: vec![],
                    state: kerotakis_core::vessel::MaterialObjectState {
                        elapsed_seconds: 0.0,
                        exchanged_water_moles: if case == 6 { f64::NAN } else { 0.0 },
                        browned_fraction: if case == 7 { -0.01 } else { 0.0 },
                    },
                }),
        }
        assert!(
            diff_vessels(&v, &after, "test").commit(&mut v).is_err(),
            "case {case}"
        );
        assert_eq!(key(&v), before, "case {case}");
    }
}

#[test]
fn native_withdrawal_preserves_unrelated_bulk_and_electrode_traces() {
    use kerotakis_core::delta::ElectrodeInventory;
    let mut v = water();
    v.deposit(SpeciesId::new("Fe"), Moles(1e-16), Phase::Solid);
    let proposal = StateDelta::new("phase transfer")
        .with_moles(SpeciesId::new("water"), Phase::Liquid, -1.0)
        .with_moles(SpeciesId::new("water"), Phase::Gas, 1.0);
    proposal.commit_conserved(&mut v, 1e-12).unwrap();
    assert_eq!(v.moles_of(&SpeciesId::new("Fe")).0, 1e-16);
    let mut e = electrode();
    e.deposits[0].moles = 1e-16;
    let mut zinc = e.deposits[0].clone();
    zinc.species = "Zn".into();
    zinc.moles = 0.001;
    e.deposits.push(zinc);
    v.electrodes.push(e);
    StateDelta::new("electrode transfer")
        .with_electrode_moles(
            "e",
            ElectrodeInventory::Deposit {
                species: SpeciesId::new("Zn"),
                growth: None,
                effect: None,
            },
            -0.001,
        )
        .with_moles(SpeciesId::new("Zn+2"), Phase::Aqueous, 0.001)
        .commit_conserved(&mut v, 1e-12)
        .unwrap();
    assert_eq!(v.electrodes[0].deposits.len(), 1);
    assert_eq!(v.electrodes[0].deposits[0].species, "Cu");
    assert_eq!(v.electrodes[0].deposits[0].moles, 1e-16);
}

#[test]
fn valid_signed_electrical_state_and_fraction_boundaries_still_commit() {
    let mut v = water();
    let mut e = electrode();
    e.double_layer_capacitance_f_per_m2 = Some(0.2);
    e.interfacial_potential_v = Some(-0.2);
    v.electrodes.push(e);
    let mut after = v.clone();
    after.electrodes[0].diagnostics = Some(
        serde_json::from_value(serde_json::json!({
            "seconds":1.0,"inventory_limited":false,
            "interfacial_conditions":[],"applied_parameters":[],
            "balance":{"electrode_potential_v":-0.2,"terminal_potential_v":-0.21,
                "net_current_density_a_per_m2":-3.0,"total_current_density_a_per_m2":-3.0,
                "partial_currents":[{"reaction_id":"r","current_density_a_per_m2":-3.0}]}
        }))
        .unwrap(),
    );
    after.surface_particles = Some(kerotakis_core::vessel::SurfaceParticleState {
        material: "pepper".into(),
        coverage_fraction: 0.0,
        cleared_fraction: 1.0,
    });
    diff_vessels(&v, &after, "valid signed state")
        .commit_conserved(&mut v, 1e-12)
        .unwrap();
    assert_eq!(key(&v), key(&after));
}
