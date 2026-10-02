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
    for invalid in [f64::NAN, f64::INFINITY, -1.0, 0.0] {
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
